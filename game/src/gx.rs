//! General-purpose OpenGL convenience wrappers.

use std::ffi::CStr;
use std::ptr;
use std::mem;
use std::str;
use std::slice;
use std::os::raw::c_void;
use sdl2::VideoSubsystem;
use gl;
use gl::types::*;
use v::{Extent2, Rgba, Mat4};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum ObjType {
    Buffer            = gl::BUFFER,
    Shader            = gl::SHADER,
    Program           = gl::PROGRAM,
    VertexArray       = gl::VERTEX_ARRAY,
    Query             = gl::QUERY,
    ProgramPipeline   = gl::PROGRAM_PIPELINE,
    TransformFeedback = gl::TRANSFORM_FEEDBACK,
    Sampler           = gl::SAMPLER,
    Texture           = gl::TEXTURE,
    Renderbuffer      = gl::RENDERBUFFER,
    Framebuffer       = gl::FRAMEBUFFER,
}
fn set_label_dummy(_ns: ObjType, _id: GLuint, _label: &[u8]) {}
fn set_label_actual(ns: ObjType, id: GLuint, label: &[u8]) {
    unsafe {
        gl::ObjectLabel(ns as _, id, label.len() as _, label.as_ptr() as _);
    }
}
static mut SET_LABEL: fn(ObjType, GLuint, &[u8]) = set_label_dummy;

pub trait GLResource {
    fn gl_id(&self) -> GLuint;
    fn obj_type() -> ObjType;
    fn set_label(&self, label: &[u8]) {
        unsafe {
            SET_LABEL(Self::obj_type(), self.gl_id(), label)
        }
    }
}

pub unsafe fn init(video: &VideoSubsystem) {
    let mut ctxflags: GLint = 0;
    let mut ctxpmask: GLint = 0;
    let mut depth_bits: GLint = 0;
    let mut stencil_bits: GLint = 0;
    let mut double_buffer: GLboolean = 0;
    let mut stereo_buffers: GLboolean = 0;
    gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut ctxflags);
    gl::GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut ctxpmask);
    gl::GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::DEPTH, 
            gl::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE, &mut depth_bits);
    gl::GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::STENCIL, 
            gl::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE, &mut stencil_bits);
    gl::GetBooleanv(gl::DOUBLEBUFFER, &mut double_buffer);
    gl::GetBooleanv(gl::STEREO, &mut stereo_buffers);

    let ctxflags = ctxflags as GLuint;
    let ctxpmask = ctxpmask as GLuint;

    let gl_version    = CStr::from_ptr(gl::GetString(gl::VERSION) as _).to_string_lossy();
    let gl_renderer   = CStr::from_ptr(gl::GetString(gl::RENDERER) as _).to_string_lossy();
    let gl_vendor     = CStr::from_ptr(gl::GetString(gl::VENDOR) as _).to_string_lossy();
    let glsl_version  = CStr::from_ptr(gl::GetString(gl::SHADING_LANGUAGE_VERSION) as _).to_string_lossy();
    let _gl_extensions = CStr::from_ptr(gl::GetString(gl::EXTENSIONS) as _).to_string_lossy();

    let gl_major = gl_version.chars().nth(0).unwrap() as u32 - '0' as u32;
    let gl_minor = gl_version.chars().nth(2).unwrap() as u32 - '0' as u32;

    // TODO: report to gl crate.
    #[allow(non_snake_case)]
    let CONTEXT_FLAG_NO_ERROR_BIT_KHR: GLuint = 0x00000008;

    info!(
"--- Active OpenGL context settings ---
Version             : {} (parsed: {}.{})
Renderer            : {}
Vendor              : {}
GLSL version        : {}
Profile flags       : {} (bits: 0b{:08b})
Context flags       : {}{}{}{} (bits: {:08b})
Double buffering    : {}
Stereo buffers      : {}
Depth buffer bits   : {}
Stencil buffer bits : {}",
        gl_version, gl_major, gl_minor, gl_renderer, gl_vendor, glsl_version,
        if ctxpmask & gl::CONTEXT_CORE_PROFILE_BIT != 0 {
            "core"
        } else if ctxpmask & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT != 0 {
            "compatibility"
        } else { "" },
        ctxpmask,
        if ctxflags & gl::CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT != 0 { "forward_compatible " } else {""},
        if ctxflags & gl::CONTEXT_FLAG_DEBUG_BIT != 0 { "debug " } else {""},
        if ctxflags & gl::CONTEXT_FLAG_ROBUST_ACCESS_BIT != 0 { "robust_access " } else {""},
        if ctxflags &     CONTEXT_FLAG_NO_ERROR_BIT_KHR != 0 { "no_error " } else {""},
        ctxflags,
        double_buffer, stereo_buffers, depth_bits, stencil_bits,
    );

    let can_debug = gl_major > 4 
        || (gl_major == 4 && gl_minor >= 3)
        || video.gl_extension_supported("GL_KHR_debug");
        //|| video.gl_extension_supported("GL_ARB_debug_output");

    if can_debug {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(gl_dbg_msg_callback, ptr::null_mut());
        gl::DebugMessageControl(
            gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE,
            0, ptr::null_mut(), gl::TRUE
        );
        SET_LABEL = set_label_actual as _;
        let msg = b"OpenGL debugging is set up.\0";
        gl::DebugMessageInsert(
            gl::DEBUG_SOURCE_APPLICATION, gl::DEBUG_TYPE_OTHER,
            0x00000000, gl::DEBUG_SEVERITY_NOTIFICATION,
            (msg.len()-1) as _, msg.as_ptr() as _
        );
    }

    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::BLEND);                                                         
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);  
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
}

extern "system" fn gl_dbg_msg_callback(
    source: GLenum, ty: GLenum, id: GLuint, severity: GLenum, 
    length: GLsizei, message: *const GLchar, _user_param: *mut c_void,
) {
    let src = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window system",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "3rd party",
        gl::DEBUG_SOURCE_APPLICATION => "Application",
        gl::DEBUG_SOURCE_OTHER => "Other",
        _ => "",
    };
    use log::Level;
    let mut level = Level::Debug;
    let t = match ty {
        gl::DEBUG_TYPE_ERROR => { level = Level::Error; "Error" },
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => { level = Level::Warn; "Deprecated behaviour" },
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => { level = Level::Warn; "Undefined behaviour" },
        gl::DEBUG_TYPE_PERFORMANCE => "Performance",
        gl::DEBUG_TYPE_PORTABILITY => "Portability",
        gl::DEBUG_TYPE_MARKER => "Command stream annotation",
        gl::DEBUG_TYPE_PUSH_GROUP => "Push debug group",
        gl::DEBUG_TYPE_POP_GROUP => "Pop debug group",
        gl::DEBUG_TYPE_OTHER => "Other",
        _ => "",
    };
    let sev = match severity {
        gl::DEBUG_SEVERITY_HIGH         => "High",
        gl::DEBUG_SEVERITY_MEDIUM       => "Medium",
        gl::DEBUG_SEVERITY_LOW          => "Low",
        gl::DEBUG_SEVERITY_NOTIFICATION => "Info",
        _ => "",
    };
    let message = unsafe {
        slice::from_raw_parts(message as *const u8, length as _)
    };
    let message = str::from_utf8(message).unwrap();
    log!(
        level,
        "OpenGL debug message ({}, {}, {}, 0x{:X}) :\n{}",
        sev, t, src, id, message
    );
}


#[derive(Debug, Hash, PartialEq, Eq)]
struct Shader(GLuint);
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct VertexShader(Shader);
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FragmentShader(Shader);
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Program(GLuint);
#[derive(Debug, Hash, PartialEq, Eq)]
struct Buffer(GLuint);
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Vbo(Buffer);
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Vao(GLuint);
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Texture2D(GLuint);

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.gl_id());
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.gl_id());
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.gl_id());
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.0);
        }
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.0);
        }
    }
}

macro_rules! impl_shader_subtype {
    ($(($Self:ident $ty:ident))+) => {
        $(
            impl $Self {
                pub fn from_source(src: &[u8]) -> Result<Self, String> {
                    Ok($Self(Shader::from_source(gl::$ty, src)?))
                }
                pub fn info_log(&self) -> String {
                    self.0.info_log()
                }
            }
            impl GLResource for $Self {
                fn gl_id(&self) -> GLuint {
                    self.0.gl_id()
                }
                fn obj_type() -> ObjType {
                    Shader::obj_type()
                }
            }
        )+
    };
}

impl_shader_subtype!{
    (VertexShader VERTEX_SHADER)
    (FragmentShader FRAGMENT_SHADER)
}

impl Shader {
    pub fn from_source(ty: GLenum, src: &[u8]) -> Result<Self, String> {
        unsafe {
            let shader = gl::CreateShader(ty);
            assert_ne!(shader, 0);
            let mut len = src.len() as GLint;
            if src[len as usize - 1] == 0 {
                len -= 1;
            }
            let glchars = src.as_ptr() as *const GLchar;
            gl::ShaderSource(shader, 1, &glchars, &len);
            gl::CompileShader(shader);
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            
            let s = Shader(shader);
            if status == gl::TRUE as _ {
                return Ok(s);
            }
            Err(s.info_log())
        }
    }
    pub fn info_log(&self) -> String {
        unsafe {
            let mut len = 0;
            gl::GetShaderiv(self.gl_id(), gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: Vec<u8> = Vec::with_capacity((len-1) as _); // -1 to skip trailing null
            buf.set_len((len-1) as _);
            gl::GetShaderInfoLog(self.gl_id(), len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            String::from_utf8(buf).unwrap_or("<UTF-8 error>".to_owned())
        }
    }
}
impl GLResource for Shader {
    fn gl_id(&self) -> GLuint {
        self.0
    }
    fn obj_type() -> ObjType {
        ObjType::Shader
    }
}


impl Program {
    // `use` is a keyword, too bad
    pub fn use_program(&self) {
        unsafe { 
            gl::UseProgram(self.gl_id());
        }
    }
    pub fn from_vert_frag(vs: &VertexShader, fs: &FragmentShader) -> Result<Self, String> {
        unsafe {
            let program = gl::CreateProgram();
            assert_ne!(program, 0);
            gl::AttachShader(program, vs.gl_id());
            gl::AttachShader(program, fs.gl_id());
            gl::LinkProgram(program);
            gl::DetachShader(program, vs.gl_id());
            gl::DetachShader(program, fs.gl_id());
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            let s = Program(program);
            if status == gl::TRUE as _ {
                return Ok(s);
            }
            Err(s.info_log())
        }
    }
    pub fn info_log(&self) -> String {
        unsafe {
            let mut len: GLint = 0;
            gl::GetProgramiv(self.gl_id(), gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: Vec<u8> = Vec::with_capacity((len-1) as usize); // -1 to skip trailing null
            buf.set_len((len-1) as _);
            gl::GetProgramInfoLog(self.gl_id(), len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            String::from_utf8(buf).unwrap_or("<UTF-8 error>".to_owned())
        }
    }
    pub fn attrib_location(&self, name: &[u8]) -> Option<GLint> {
        assert_eq!(name[name.len()-1], 0);
        let i = unsafe {
            gl::GetAttribLocation(self.gl_id(), name.as_ptr() as *const GLchar)
        };
        match i {
            -1 => None,
            i @ _ => Some(i),
        }
    }
    pub fn uniform_location(&self, name: &[u8]) -> Option<GLint> {
        assert_eq!(name[name.len()-1], 0);
        let i = unsafe {
            gl::GetUniformLocation(self.gl_id(), name.as_ptr() as *const GLchar)
        };
        match i {
            -1 => None,
            i @ _ => Some(i),
        }
    }
    /*
    // WISH: Refactor this into a program Builer (do before linking)
    pub fn bind_attrib_location(&self, loc: GLuint, name: &[u8]) {
        assert_eq!(name[name.len()-1], 0);
        unsafe {
            gl::BindAttribLocation(self.gl_id(), loc, name.as_ptr() as *const GLchar);
        }
    }
    */
    pub fn set_uniform_4f(&self, loc: GLint, m: &[[GLfloat; 4]]) {
        unsafe {
            gl::Uniform4fv(loc, m.len() as _, &m[0][0]);
        }
    }
    pub fn set_uniform_1i(&self, loc: GLint, m: &[GLint]) {
        unsafe {
            gl::Uniform1iv(loc, m.len() as _, m.as_ptr());
        }
    }
    pub fn set_uniform_mat4(&self, loc: GLint, m: &[Mat4<f32>]) {
        let transpose = m[0].gl_should_transpose() as GLboolean;
        unsafe {
            gl::UniformMatrix4fv(loc, m.len() as _, transpose, m[0].cols[0].as_ptr());
        }
    }
}
impl GLResource for Program {
    fn gl_id(&self) -> GLuint {
        self.0
    }
    fn obj_type() -> ObjType {
        ObjType::Program
    }
}


impl Vao {
    pub fn new() -> Self {
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        assert_ne!(vao, 0);
        Vao(vao)
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.gl_id());
        }
    }
    pub fn unbind() {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}
impl GLResource for Vao {
    fn gl_id(&self) -> GLuint {
        self.0
    }
    fn obj_type() -> ObjType {
        ObjType::VertexArray
    }
}

impl Buffer {
    pub fn new() -> Self {
        let mut buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer);
        }
        assert_ne!(buffer, 0);
        Buffer(buffer)
    }
}
impl GLResource for Buffer {
    fn gl_id(&self) -> GLuint {
        self.0
    }
    fn obj_type() -> ObjType {
        ObjType::Buffer
    }
}


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum UpdateHint {
    Never,
    Occasionally,
    Often,
}

impl UpdateHint {
    pub fn into_glenum_draw(&self) -> GLenum {
        match *self {
            UpdateHint::Never => gl::STATIC_DRAW,
            UpdateHint::Occasionally => gl::DYNAMIC_DRAW,
            UpdateHint::Often => gl::STREAM_DRAW,
        }
    }
}

impl Vbo {
    pub fn new() -> Self {
        Vbo(Buffer::new())
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gl_id());
        }
    }
    pub fn set_data<T>(&self, data: &[T], hint: UpdateHint) {
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as _,
                data.as_ptr() as _,
                hint.into_glenum_draw()
            );
        }
    }
}
impl GLResource for Vbo {
    fn gl_id(&self) -> GLuint {
        self.0.gl_id()
    }
    fn obj_type() -> ObjType {
        Buffer::obj_type()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct TextureParamsI {
    pub wrap_s: GLenum,
    pub wrap_t: GLenum,
    pub min_filter: GLenum,
    pub mag_filter: GLenum,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Texture2DImage<'a, Pixel: 'a> {
    pub pixels: &'a [Pixel],
    pub size: Extent2<usize>,
    pub mipmap_level: GLint, // 0
    pub internal_format: GLenum,
    pub pixels_format: GLenum,
    pub pixel_element_type: GLenum,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Texture2DInit<'a, Pixel: 'a> {
    pub image: Texture2DImage<'a, Pixel>,
    pub params_i: TextureParamsI,
    pub do_generate_mipmaps: bool,
}

impl TextureParamsI {
    pub fn new_clamp_to_edge_linear() -> Self {
        Self {
            wrap_s: gl::CLAMP_TO_EDGE,
            wrap_t: gl::CLAMP_TO_EDGE,
            min_filter: gl::LINEAR,
            mag_filter: gl::LINEAR,
        }
    }
}

impl<'a> Texture2DImage<'a, u8> {
    pub fn from_greyscale_u8(pixels: &'a [u8], size: Extent2<usize>) -> Self {
        Self {
            pixels, size, mipmap_level: 0,
            internal_format: gl::RED,
            pixels_format: gl::RED,
            pixel_element_type: gl::UNSIGNED_BYTE,
        }
    }
}
impl<'a> Texture2DImage<'a, Rgba<u8>> {
    pub fn from_rgba_u8(pixels: &'a [Rgba<u8>], size: Extent2<usize>) -> Self {
        Self {
            pixels, size, mipmap_level: 0,
            internal_format: gl::RGBA,
            pixels_format: gl::RGBA,
            pixel_element_type: gl::UNSIGNED_BYTE,
        }
    }
}

pub fn set_active_texture(i: GLuint) {
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0 + i);
    }
}

impl Texture2D {
    pub fn new<T>(t: Texture2DInit<T>) -> Self {
        let Texture2DInit { image: img, params_i: p, do_generate_mipmaps } = t;
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            assert_ne!(id, 0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, p.wrap_s as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, p.wrap_t as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, p.min_filter as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, p.mag_filter as _);
            // NOTE: glPixelStorei(GL_UNPACK_ALIGNMENT, 1); done in gx::init().
            gl::TexImage2D(
                gl::TEXTURE_2D, img.mipmap_level, img.internal_format as _,
                img.size.w as _, img.size.h as _, 0,
                img.pixels_format, img.pixel_element_type, img.pixels.as_ptr() as *const _
            );
            if do_generate_mipmaps {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
            Texture2D(id)
        }
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.gl_id());
        }
    }
}

impl GLResource for Texture2D {
    fn gl_id(&self) -> GLuint {
        self.0
    }
    fn obj_type() -> ObjType {
        ObjType::Texture
    }
}

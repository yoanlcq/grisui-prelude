extern crate gl;
extern crate sdl2;

use std::fmt::{self, Formatter, Debug};
use std::ffi::CStr;
use std::ptr;
use std::mem;
use std::str;
use std::slice;
use std::os::raw::c_void;
use sdl2::VideoSubsystem;
use gl::types::*;

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
fn gl_object_label_dummy(_ns: ObjType, _id: GLuint, _label: &[u8]) {}
fn gl_object_label_actual(ns: ObjType, id: GLuint, label: &[u8]) {
    unsafe {
        gl::ObjectLabel(ns as _, id, label.len() as _, label.as_ptr() as _);
    }
}

pub struct Gx {
    label_fn: fn(ObjType, GLuint, &[u8]),
    gl_major: u32,
    gl_minor: u32,
}


impl Debug for Gx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Gx")
            .field("gl_major", &self.gl_major)
            .field("gl_minor", &self.gl_minor)
            .finish()
    }
}

impl Gx {
    pub fn label(&self, ns: ObjType, id: GLuint, label: &[u8]) {
        (self.label_fn)(ns, id, label)
    }
    pub unsafe fn new(video: &VideoSubsystem) -> Self {
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
        let gl_extensions = CStr::from_ptr(gl::GetString(gl::EXTENSIONS) as _).to_string_lossy();

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
    Stencil buffer bits : {}
    Extensions          : {}",
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
            gl_extensions
        );

        let can_debug = gl_major > 4 
            || (gl_major == 4 && gl_minor >= 3)
            || video.gl_extension_supported("GL_KHR_debug");
            //|| video.gl_extension_supported("GL_ARB_debug_output");

        let mut label_fn = gl_object_label_dummy;
        if can_debug {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(gl_dbg_msg_callback, ptr::null_mut());
            gl::DebugMessageControl(
                gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE,
                0, ptr::null_mut(), gl::TRUE
            );
            label_fn = mem::transmute(gl_object_label_actual);
        }
        gl::Enable(gl::DEPTH_TEST);
        Self { label_fn, gl_major, gl_minor, }
    }
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
    let t = match ty {
        gl::DEBUG_TYPE_ERROR => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated behaviour",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined behaviour",
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
    debug!(
        "OpenGL debug message ({}, {}, {}, {:X}) :\n{}",
        sev, t, src, id, message
    );
}



extern crate sdl2;
extern crate gl;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::ffi::CStr;
use std::io::Write;
use std::time::{Instant, Duration};
use std::thread;
use std::mem;
use std::ptr;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLProfile, SwapInterval};

use gl::types::*;

use log::LevelFilter;


fn main() {
    //std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "full");

    let mut builder = env_logger::Builder::new();
    builder.format(|buf, record| {
        let s = format!("{}", record.level());
        let s = s.chars().next().unwrap();
        writeln!(buf, "[{}] {}", s, record.args())
    }).filter(None, LevelFilter::Info);
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init();

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_flags().debug().set();
    gl_attr.set_context_version(3, 2);
    gl_attr.set_depth_size(24);
    gl_attr.set_stencil_size(8);
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);

    let window = video.window("Grisui - Prelude", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    window.gl_set_context_to_current().unwrap();

    gl::load_with(|s| {
        let ptr = video.gl_get_proc_address(s);
        ptr as _
    });
    video.gl_set_swap_interval(SwapInterval::LateSwapTearing);


    unsafe {
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
        //let gl_extensions = CStr::from_ptr(gl::GetString(gl::EXTENSIONS) as _).to_string_lossy();


        // TODO: report to gl crate.
        #[allow(non_snake_case)]
        let CONTEXT_FLAG_NO_ERROR_BIT_KHR: GLuint = 0x00000008;

        info!(
"--- Active OpenGL context settings ---
    Version             : {}
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
            gl_version, gl_renderer, gl_vendor, glsl_version,
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
            0 //gl_extensions
        );
    }

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }


    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER).unwrap();
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER).unwrap();
    let program = link_program(vs, fs).unwrap();

    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       mem::transmute(&VERTEX_DATA[0]),
                       gl::STATIC_DRAW);
        gl::UseProgram(program);
        gl::GetAttribLocation(program, b"out_color\0".as_ptr() as *const GLchar);

        let pos_attr = gl::GetAttribLocation(program, b"position\0".as_ptr() as *const GLchar);
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(pos_attr as GLuint, 3, gl::FLOAT,
                                gl::FALSE as GLboolean, 0, ptr::null());
    }


    let current_display_mode = window.display_mode().unwrap();
    let mut lim_last_time = Instant::now();
    let mut last_time = Instant::now();
    let mut frame_accum = 0u64;
    let mut fps_limit = 0f64;
    let fps_ceil = 60f64;
    let fps_counter_interval = 1000f64; /* Should be in [100, 1000] */

    let mut event_pump = sdl.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        /* See http://www.opengl-tutorial.org/miscellaneous/an-fps-counter/ */
        frame_accum += 1;
        let current_time = Instant::now();
        if current_time.duration_since(last_time) > Duration::from_millis(fps_counter_interval as _) {
            let fps = ((frame_accum as f64) * 1000f64 / fps_counter_interval).round() as u32;
            info!(concat!("{} frames under {} milliseconds = ",
                "{} milliseconds/frame = ",
                "{} FPS"), 
                frame_accum,
                fps_counter_interval,
                fps_counter_interval / (frame_accum as f64), 
                fps
            );
            frame_accum = 0;
            last_time += Duration::from_millis(fps_counter_interval as _);
            if fps_limit <= 0_f64 && fps as f64 > fps_ceil {
                let reason = if current_display_mode.refresh_rate != 0 {
                    fps_limit = current_display_mode.refresh_rate as _;
                    "from display mode info"
                } else {
                    fps_limit = fps_ceil;
                    "fallback"
                };
                warn!(concat!("Abnormal FPS detected (Vsync is not working). ",
                    "Now limiting FPS to {} ({})."),
                    fps_limit, reason
                );
            }
        }

        unsafe {
            gl::ClearColor(1f32, 0f32, 0f32, 1f32);
            gl::Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }

        window.gl_swap_window();

        if fps_limit > 0_f64 {
            let current_time = Instant::now();
            let a_frame = Duration::from_millis((1000_f64 / fps_limit).round() as _);
            if current_time - lim_last_time < a_frame {
                thread::sleep(a_frame - (current_time - lim_last_time));
            }
            lim_last_time = Instant::now();
        }
    }

    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}

static VERTEX_DATA: [GLfloat; 9] = [
    0.0, 0.5, 0.0,
    0.5, -0.5, 0.0,
    -0.5, -0.5, 0.0
];

static VS_SRC: &[u8] = b"
    #version 330
    layout(location=0) in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
\0";

static FS_SRC: &[u8] = b"
    #version 330
    layout(location=0) out vec4 out_color;
    void main() {
        out_color = vec4(0.0, 0.0, 1.0, 1.0);
    }
\0";

fn compile_shader(src: &[u8], ty: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(ty);
        let glchars = src.as_ptr() as *const GLchar;
        gl::ShaderSource(shader, 1, &glchars, ptr::null());
        gl::CompileShader(shader);
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        
        if status == gl::TRUE as _ {
            return Ok(shader);
        }
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf: Vec<u8> = Vec::with_capacity((len-1) as _); // -1 to skip trailing null
        buf.set_len((len-1) as _);
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        let s = String::from_utf8(buf).unwrap_or("<UTF-8 error>".to_owned());
        Err(s)
    }
}

fn link_program(vs: GLuint, fs: GLuint) -> Result<GLuint, String> {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status == gl::TRUE as _ {
            return Ok(program);
        }
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf: Vec<u8> = Vec::with_capacity((len-1) as usize); // -1 to skip trailing null
        buf.set_len((len-1) as _);
        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        let s = String::from_utf8(buf).unwrap_or("<UTF-8 error>".to_owned());
        Err(s)
    }
}

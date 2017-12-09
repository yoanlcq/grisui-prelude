extern crate sdl2;
extern crate gl;
extern crate env_logger;
#[macro_use]
extern crate log;

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

pub mod gx;
use gx::*;

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
    //gl_attr.set_context_version(3, 2);
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

    let gx = unsafe {
        Gx::new(&video)
    };

    let vs = match gx::VertexShader::from_source(VS_SRC) {
        Ok(i) => i,
        Err(s) => {
            error!("Failed to compile vertex shader:\n{}", s);
            panic!()
        },
    };
    let fs = match gx::FragmentShader::from_source(FS_SRC) {
        Ok(i) => i,
        Err(s) => {
            error!("Failed to compile fragment shader:\n{}", s);
            panic!()
        },
    };
    let program = match gx::Program::from_vert_frag(&vs, &fs) {
        Ok(i) => i,
        Err(s) => {
            error!("Failed to link GL program:\n{}", s);
            panic!()
        },
    };

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
        gl::UseProgram(program.gl_id());
        gl::GetAttribLocation(program.gl_id(), b"out_color\0".as_ptr() as *const GLchar);

        let pos_attr = gl::GetAttribLocation(program.gl_id(), b"position\0".as_ptr() as *const GLchar);
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(pos_attr as GLuint, 3, gl::FLOAT,
                                gl::FALSE as GLboolean, 0, ptr::null());
    }

    gx.label(gx::ObjType::Shader, vs.gl_id(), b"Vertex Shader");
    gx.label(gx::ObjType::Shader, fs.gl_id(), b"Fragment Shader");
    gx.label(gx::ObjType::Program, program.gl_id(), b"Program");
    gx.label(gx::ObjType::VertexArray, vao, b"VAO");
    gx.label(gx::ObjType::Buffer, vbo, b"VBO");


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


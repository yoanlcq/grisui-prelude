use std::io::Write;
use std::ptr;
use std::env;

use sdl2;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{Window, GLContext, GLProfile, SwapInterval};

use gl;
use gl::types::*;

use log::LevelFilter;

use env_logger;

use gx;
use gx::*;

pub struct Game {
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    _gl_context: GLContext,
    pub should_quit: bool,
    vao: gx::Vao,
    _vbo: gx::Vbo,
    program: gx::Program,
}

impl Game {
    pub fn new() -> Self {
        setup_env();
        setup_log();

        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        {
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_flags().debug().set();
            //gl_attr.set_context_version(3, 2);
            gl_attr.set_depth_size(24);
            gl_attr.set_stencil_size(8);
            gl_attr.set_multisample_buffers(1);
            gl_attr.set_multisample_samples(4);
        }

        let window = video.window("Grisui - Prelude", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let _gl_context = window.gl_create_context().unwrap();
        window.gl_set_context_to_current().unwrap();

        gl::load_with(|s| video.gl_get_proc_address(s) as _);
        video.gl_set_swap_interval(SwapInterval::LateSwapTearing);

        unsafe {
            gx::init(&video);
        }

        let vs = match gx::VertexShader::from_source(VS_SRC) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile vertex shader:\n{}", s);
                panic!()
            },
        };
        vs.set_label(b"Vertex Shader");
        let fs = match gx::FragmentShader::from_source(FS_SRC) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile fragment shader:\n{}", s);
                panic!()
            },
        };
        fs.set_label(b"Fragment Shader");
        let program = match gx::Program::from_vert_frag(&vs, &fs) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to link GL program:\n{}", s);
                panic!()
            },
        };
        program.set_label(b"Program");

        let vao = gx::Vao::new();
        let vbo = gx::Vbo::new();
        unsafe {
            vao.bind();
            vbo.bind();
            vao.set_label(b"VAO");
            vbo.set_label(b"VBO");
            vbo.set_data(&VERTEX_DATA, gx::UpdateHint::Never);
            program.use_program();

            let pos_attr = program.attrib_location(b"position\0").unwrap();
            gl::EnableVertexAttribArray(pos_attr as _);
            gl::VertexAttribPointer(
                pos_attr as _, 3, gl::FLOAT,
                gl::FALSE as _, 0, ptr::null());
        }

        Self {
            sdl, video, window, _gl_context, should_quit: false, vao, _vbo: vbo, program
        }
    }
    pub fn handle_sdl2_event(&mut self, event: Event) {
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                self.should_quit = true;
            },
            _ => {}
        }
    }
    pub fn render_clear(&mut self) {
        unsafe {
            gl::ClearColor(1f32, 0f32, 0f32, 1f32);
            gl::Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
        }
    }
    pub fn render(&mut self) {
        unsafe {
            self.program.use_program();
            self.vao.bind();
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
    pub fn present(&mut self) {
        self.window.gl_swap_window();
    }
}

fn setup_env() {
    //env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "full");
}

fn setup_log() {
    let mut builder = env_logger::Builder::new();

    builder.format(|buf, record| {
        let s = format!("{}", record.level());
        let s = s.chars().next().unwrap();
        writeln!(buf, "[{}] {}", s, record.args())
    }).filter(None, LevelFilter::Trace);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init();
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


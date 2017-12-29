use std::io::Write;
use std::time::Duration;
use std::env;
use std::ptr;
use std::f32::consts::PI;

use sdl2;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::{Window, GLContext, GLProfile, SwapInterval};

use gl;

use alto;
use alto::Alto;

use log::LevelFilter;

use env_logger;

use gx;
use gx::*;

use camera::PerspectiveCamera;
use Transform;

use grx;

use Mat4;
use Vec3;
use Rgba;
use Lerp;
use Extent2;

use duration_ext::DurationExt;

#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub camera: PerspectiveCamera,
    pub triangle: Transform<f32, f32, f32>,
}

impl GameState {
    pub fn new(viewport_size: Extent2<u32>) -> Self {
        Self {
            camera: PerspectiveCamera {
                transform: Default::default(),
                viewport_size,
                fov_y: 60_f32.to_radians(),
                near: 0.01,
                far: 1000.0,
            },
            triangle: Default::default(),
        }
    }
    pub fn integrate(&mut self, t_dur: Duration, dt_dur: Duration) {
        let t = t_dur.to_f64_seconds();
        let dt = dt_dur.to_f64_seconds();
        trace!("GameState: Step t={}, dt={}", t, dt);
        {
            let dt = dt as f32;
            self.triangle.position.x += 0.1 * dt;
            self.triangle.orientation.rotate_z(0.1*2.0*PI * dt);
            self.triangle.scale -= 0.1 * dt;
        }
    }
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        trace!("GameState: Lerp t={}", t);
        let camera = PerspectiveCamera::lerp(&a.camera, &b.camera, t);
        let triangle = Lerp::lerp(&a.triangle, &b.triangle, t);
        Self { camera, triangle }
    }
}

pub struct Game {
    pub should_quit: bool,
    pub frame: u64,
    pub previous_state: GameState,
    pub current_state: GameState,
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    _gl_context: GLContext,
    vao: gx::Vao,
    _vbo: gx::Vbo,
    program: grx::SimpleColorProgram,
    pub alto: Alto,
    pub alto_dev: alto::OutputDevice,
    pub alto_context: alto::Context,
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
            .resizable()
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

        let program = grx::SimpleColorProgram::new();

        let vao = gx::Vao::new();
        let vbo = gx::Vbo::new();
        unsafe {
            vao.bind();
            vbo.bind();
            vao.set_label(b"Simple Triangle VAO");
            vbo.set_label(b"Simple Triangle VBO");
            let z = -0.5_f32;
            let data = [
                grx::SimpleColorVertex { position: Vec3::new( 0.0,  0.5, z), color: Rgba::red() },
                grx::SimpleColorVertex { position: Vec3::new( 0.5, -0.5, z), color: Rgba::yellow() },
                grx::SimpleColorVertex { position: Vec3::new(-0.5, -0.5, z), color: Rgba::green() },
            ];
            assert_eq_size!(Vec3<f32>, [f32; 3]);
            assert_eq_size!(Rgba<f32>, [f32; 4]);
            vbo.set_data(&data, gx::UpdateHint::Never);

            gl::EnableVertexAttribArray(program.a_position());
            gl::EnableVertexAttribArray(program.a_color());
            gl::VertexAttribPointer(
                program.a_position(), 3, gl::FLOAT,
                gl::FALSE as _, 7*4, ptr::null()
            );
            gl::VertexAttribPointer(
                program.a_color(), 4, gl::FLOAT,
                gl::FALSE as _, 7*4, ptr::null().offset(3*4)
            );
        }

        let alto = Alto::load_default().unwrap();
        let alto_dev = alto.open(None).unwrap();
        let attrs = alto::ContextAttrs {
            frequency: Some(44100),
            refresh: None,
            mono_sources: None,
            stereo_sources: None,
            soft_hrtf: None,
            soft_hrtf_id: None,
            soft_output_limiter: None,
            max_aux_sends: None,
        };
        let alto_context = alto_dev.new_context(Some(attrs)).unwrap();
        /*
        let buf = ctx.new_buffer(data, freq).unwrap();
        let static_src = ctx.new_static_source().unwrap();
        static_src.set_looping(false);
        static_src.set_buffer(Arc::new(buf)).unwrap();
        let stream_src = ctx.new_streaming_source().unwrap();
        stream_src.queue_buffer(buf).unwrap();
        stream_src.unqueue_buffer().unwrap();
        // play, pause, stop, rewind, state, gain, position, velocity, direction
        */

        let viewport_size = window.drawable_size();
        let viewport_size = Extent2::new(viewport_size.0, viewport_size.1);
        let previous_state = GameState::new(viewport_size);
        let current_state = previous_state.clone();

        let mut slf = Self {
            should_quit: false, frame: 0,
            previous_state, current_state,
            sdl, video, window,
            _gl_context, vao, _vbo: vbo, program,
            alto, alto_dev, alto_context,
        };
        slf.reshape(viewport_size);
        slf
    }
    pub fn reshape(&mut self, viewport_size: Extent2<u32>) {
        self.previous_state.camera.viewport_size = viewport_size;
        self.current_state.camera.viewport_size = viewport_size;
        unsafe {
            gl::Viewport(0, 0, viewport_size.w as _, viewport_size.h as _);
        }
    }
    pub fn handle_sdl2_event(&mut self, event: &Event) {
        match *event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                self.should_quit = true;
            },
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                WindowEvent::SizeChanged(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                _ => {
                    trace!("Unhandled {:?}", win_event);
                },
            },
            _ => {
                trace!("Unhandled {:?}", event);
            },
        }
    }
    pub fn render_clear(&mut self) {
        unsafe {
            gl::ClearColor(1f32, 0f32, 0f32, 1f32);
            gl::Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
        }
    }
    pub fn render(&mut self, state: &GameState) {
        self.frame += 1;
        let mvp = Mat4::from(state.triangle);
        // debug!("MVP: {}", mvp);
        self.program.use_program(&mvp);
        self.vao.bind();
        unsafe {
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
    }).filter(None, LevelFilter::Debug);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init();
}

use std::time::Duration;
use sdl2::{self, Sdl, VideoSubsystem};
use sdl2::video::{Window, GLContext, SwapInterval};
use duration_ext::DurationExt;
use gl;
use grx;

pub struct Game {
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub gl_context: GLContext,
    pub should_quit: bool,
}

impl Game {
    pub fn new(name: &str, w: u32, h: u32) -> Self {
        info!("Initializing game...");
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        grx::configure_sdl2_gl_attr(video.gl_attr());
        let window = video.window(name, w, h)
            .position_centered().resizable().opengl().build().unwrap();
        let gl_context = window.gl_create_context().unwrap();
        window.gl_set_context_to_current().unwrap();
        gl::load_with(|s| video.gl_get_proc_address(s) as _);
        grx::boot_gl();
        video.gl_set_swap_interval(SwapInterval::LateSwapTearing);

        info!("... Done initializing game.");
        Self {
            sdl, video, window, gl_context,
            should_quit: false,
        }
    }

    pub fn render_clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    pub fn render(&self) {
    
    }
    pub fn present(&self) {
        self.window.gl_swap_window();
    }
    pub fn pump_events(&mut self) {
        let mut event_pump = self.sdl.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    info!("Received 'Quit' event");
                    self.should_quit = true
                },
                _ => (),
            };
        }
    }
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    pub fn replace_previous_state_by_current(&mut self) {}
    pub fn compute_gfx_state_via_lerp_previous_current(&mut self, alpha: f64) {
        let alpha = alpha as f32;
        trace!("Gfx State. alpha={}", alpha);
    }
    pub fn integrate(&mut self, t: Duration, dt: Duration) {
        let t = t.to_f64_seconds() as f32;
        let dt = dt.to_f64_seconds() as f32;
        trace!("Integrating. dt={}, t={}", dt, t);
    }
}


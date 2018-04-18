use sdl2::{self, Sdl, VideoSubsystem};
use sdl2::video::{Window, GLContext, SwapInterval};
use v::Extent2;
use esystem::ESystem;
use grx;
use gl;

pub struct Platform {
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub gl_context: GLContext,
    window_size: Extent2<u32>,
}

impl ESystem for Platform {
    fn on_canvas_resized(&mut self, size: Extent2<u32>, _by_user: bool) {
        self.window_size = size;
    }
}

impl Platform {
    pub fn new(name: &str, w: u32, h: u32) -> Self {
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

        let window_size = Extent2::new(w, h);

        Self { sdl, video, window, gl_context, window_size }
    }

    pub fn canvas_size(&self) -> Extent2<u32> {
        self.window_size
    }
    pub fn present(&self) {
        self.window.gl_swap_window();
    }
    pub fn clear_draw(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}


use std::cell::{Cell, RefCell};
use sdl2::{self, Sdl, VideoSubsystem, EventPump};
use sdl2::video::{Window, GLContext, SwapInterval};
use sdl2::mouse::{Cursor as Sdl2Cursor, SystemCursor};
use v::{Vec2, Extent2};
use game::Game;
use system::System;
use grx;
use gl;

pub struct Cursors {
    pub normal: Sdl2Cursor,
    pub text: Sdl2Cursor,
    pub hand: Sdl2Cursor,
    pub crosshair: Sdl2Cursor, 
}

impl Cursors {
    pub fn new() -> Self {
        Self {
            normal: Sdl2Cursor::from_system(SystemCursor::Arrow).unwrap(),
            text: Sdl2Cursor::from_system(SystemCursor::IBeam).unwrap(),
            hand: Sdl2Cursor::from_system(SystemCursor::Hand).unwrap(),
            crosshair: Sdl2Cursor::from_system(SystemCursor::Crosshair).unwrap(),
        }
    }
}

pub struct Platform {
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub gl_context: GLContext,
    pub sdl_event_pump: RefCell<EventPump>,
    pub(self) window_size: Cell<Extent2<u32>>,
    pub cursors: Cursors,
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

        let cursors = Cursors::new();
        let window_size = Cell::new(Extent2::new(w, h));
        let sdl_event_pump = RefCell::new(sdl.event_pump().unwrap());

        Self { sdl, video, window, gl_context, window_size, cursors, sdl_event_pump }
    }

    pub fn mouse_position(&self) -> Vec2<i32> {
        let state = self.sdl_event_pump.borrow().mouse_state();
        Vec2::new(state.x(), state.y())
    }
    pub fn canvas_size(&self) -> Extent2<u32> {
        self.window_size.get()
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

pub struct PlatformSystem;

impl System for PlatformSystem {
    fn name(&self) -> &str { "PlatformSystem" }
    fn on_canvas_resized(&mut self, g: &Game, size: Extent2<u32>, _by_user: bool) {
        g.platform.window_size.set(size);
    }
}


use std::time::Duration;
use duration_ext::DurationExt;
use esystem::{self, ESystem};
use input::Input;
use platform::Platform;

pub struct Game {
    wants_to_quit: bool,
    pub platform: Platform,
    pub input: Input,
}

impl Game {
    pub fn new(name: &str, w: u32, h: u32) -> Self {
        info!("Initializing game...");
        let platform = Platform::new(name, w, h);
        let input = Input::default();
        info!("... Done initializing game.");
        Self {
            wants_to_quit: false,
            platform,
            input,
        }
    }
    pub fn clear_draw(&self) {
        self.platform.clear_draw();
    }
    pub fn present(&self) {
        self.platform.present();
    }
    pub fn should_quit(&self) -> bool {
        self.wants_to_quit
    }
    pub fn pump_events(&mut self) {
        for event in self.platform.sdl.event_pump().unwrap().poll_iter() {
            esystem::dispatch_sdl2_event(self, &event);
            for esys in self.esystems_mut().iter_mut() {
                esystem::dispatch_sdl2_event(*esys, &event);
            }
        }
    }
    pub fn esystems_mut(&mut self) -> [&mut ESystem; 2] {
        let &mut Self {
            wants_to_quit: _,
            ref mut platform,
            ref mut input,
        } = self;
        [input, platform]
    }
}

impl ESystem for Game {
    fn on_quit_requested(&mut self) {
        info!("Game: Received 'Quit' event");
        self.wants_to_quit = true;
    }
    fn compute_gfx_state_via_lerp_previous_current(&mut self, alpha: f64) {
        trace!("Gfx State. alpha={}", alpha);
    }
    fn tick(&mut self, t: Duration, dt: Duration) {
        let t = t.to_f64_seconds() as f32;
        let dt = dt.to_f64_seconds() as f32;
        trace!("Integrating. dt={}, t={}", dt, t);
    }
    fn draw(&mut self) {
        trace!("Drawing.");
    }
}


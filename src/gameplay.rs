use gl;
use system::*;

pub struct GameplaySystem {
    is_active: bool,
}

impl GameplaySystem {
    pub fn new() -> Self {
        Self {
            is_active: false,
        }
    }
    fn on_enter_gameplay(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(0.2, 0.6, 1., 1.);
        }
        g.platform.cursors.no.set();
        self.is_active = true;
    }
    fn on_leave_gameplay(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(1., 0., 1., 1.);
        }
        g.platform.cursors.normal.set();
        self.is_active = false;
    }
}

impl System for GameplaySystem {
    fn name(&self) -> &str {
        "GameplaySystem"
    }
    fn on_message(&mut self, g: &Game, msg: &Message) {
        match *msg {
            Message::EnterGameplay => self.on_enter_gameplay(g),
            Message::LeaveGameplay => self.on_leave_gameplay(g),
            _ => (),
        };
    }
}

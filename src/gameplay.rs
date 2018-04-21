use gl;
use system::*;

pub struct GameplaySystem;

impl GameplaySystem {
    fn on_enter_gameplay(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(0.2, 0.6, 1., 1.);
        }
        g.platform.cursors.no.set();
    }
    fn on_leave_gameplay(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(1., 0., 1., 1.);
        }
        g.platform.cursors.normal.set();
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

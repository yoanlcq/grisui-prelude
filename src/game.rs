use std::time::Duration;
use std::cell::{RefCell, Cell};
use std::collections::VecDeque;
use duration_ext::DurationExt;
use system::{self, System, Message};
use input::{Input, InputSystem};
use platform::{Platform, PlatformSystem};
use editor;
use mesh;

pub struct Game {
    pub wants_to_quit: Cell<bool>,
    pub platform: Platform,
    pub input: Input,
    pub messages: RefCell<VecDeque<Message>>,
    pub systems: RefCell<Vec<Box<System>>>,

    pub mesh_gl_program: mesh::Program,
}

pub struct QuitSystem;

impl System for QuitSystem {
    fn name(&self) -> &str { "QuitSystem" }
    fn on_quit_requested(&mut self, g: &Game) {
        info!("{}: Received 'Quit' event", self.name());
        g.wants_to_quit.set(true);
    }
}


impl Game {
    pub fn new(name: &str, w: u32, h: u32) -> Self {
        info!("Game: Initializing...");

        let platform = Platform::new(name, w, h);
        let input = Input::default();
        let messages = Default::default();

        let mesh_gl_program = mesh::Program::new();

        let systems = RefCell::new(vec![
            Box::new(InputSystem) as Box<System>,
            Box::new(PlatformSystem),
            Box::new(editor::EditorSystem::new(&mesh_gl_program, platform.canvas_size(), platform.mouse_position())),
            Box::new(QuitSystem),
        ]);

        info!("Game: ... Done initializing.");
        Self {
            wants_to_quit: Cell::new(false),
            platform,
            input,
            messages,
            systems,
            mesh_gl_program,
        }
    }
    pub fn should_quit(&self) -> bool {
        self.wants_to_quit.get()
    }
    pub fn pump_events(&self) {
        // Required to shorten the RefMut's lifetime, so other system can borrow the event pump.
        let poll_event = || self.platform.sdl_event_pump.borrow_mut().poll_event();

        while let Some(event) = poll_event() {
            for s in self.systems.borrow_mut().iter_mut() {
                trace!("SDL2 Event {}... {:?}", s.name(), event);
                system::dispatch_sdl2_event(s.as_mut(), self, &event);
            }
            self.pump_messages();
        }
        // We still want to pump messages if there were no events.
        self.pump_messages();
    }
    fn pump_messages(&self) {
        // Handling messages can cause new messages to be emitted
        while !self.messages.borrow().is_empty() {
            // replace() here allows us not to borrow the message queue while iterating,
            // which allows systems to push messages to the queue while already handling messages.
            for msg in self.messages.replace(Default::default()) {
                for s in self.systems.borrow_mut().iter_mut() {
                    trace!("Message {}... {:?}", s.name(), msg);
                    s.on_message(self, &msg);
                }
            }
        }
    }
    pub fn tick(&self, t: Duration, dt: Duration) {
        for s in self.systems.borrow_mut().iter_mut() {
            trace!("Tick {}... dt={}, t={}", s.name(), dt.to_f64_seconds(), t.to_f64_seconds());
            s.tick(self, t, dt);
        }
    }
    pub fn draw(&self, p: f64) {
        self.platform.clear_draw();
        for s in self.systems.borrow_mut().iter_mut() {
            trace!("Draw {}... lerp_factor={}", s.name(), p);
            s.draw(self, p);
        }
        self.platform.present();
    }
}

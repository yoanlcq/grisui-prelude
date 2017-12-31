use std::collections::HashMap;
use sdl2::event::Event;
use sdl2::mouse::{MouseWheelDirection, MouseButton};
use sdl2::keyboard::{Keycode};
use v::{Vec2};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Input {
    pub wants_to_quit: bool,
    pub mouse: MouseInput,
    pub keyboard: KeyboardInput,
    /// The current piece of input text for this tick.
    pub text: String,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MouseInput {
    pub left: ButtonInput,
    pub middle: ButtonInput,
    pub right: ButtonInput,
    pub scroll: Vec2<i32>,
    pub position: Vec2<u32>,
    pub prev_position: Vec2<u32>,
    pub has_prev_position: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KeyboardInput {
    pub keys: HashMap<Keycode, ButtonInput>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ButtonInput {
    /// The state for the current tick.
    pub is_pressed: bool,
    /// The state at previous tick.
    pub was_pressed: bool,
    pub num_changes_since_last_tick_and_event_processing: u16,
    // XXX What if user holds down buttons while starting the game ?
}


impl Input {
    pub fn begin_tick_and_event_processing(&mut self) {
        let &mut Self { 
            wants_to_quit: _, ref mut text,
            ref mut mouse, ref mut keyboard, 
        } = self;
        text.clear();
        mouse.begin_tick_and_event_processing();
        keyboard.begin_tick_and_event_processing();
    }
    pub fn handle_sdl2_event_before_new_tick(&mut self, event: &Event) {
        match *event {
            Event::Quit {..} => {
                self.wants_to_quit = true;
            },
            // Event::TextEditing { text, start, length, .. } => {},
            Event::TextInput { ref text, .. } => {
                self.text += &text;
                info!("Text input \"{}\"", &text);
                info!("Total: \"{}\"", &self.text);
            },
            Event::KeyDown { keycode, scancode, keymod, repeat, .. } => {
                let _ = scancode;
                let _ = keymod;
                if !repeat {
                    if let Some(keycode) = keycode {
                        self.keyboard.key(keycode).press();
                    } else {
                        warn!("Some key was pressed, but keycode is None");
                    }
                }
            },
            Event::KeyUp { keycode, scancode, keymod, .. } => {
                let _ = scancode;
                let _ = keymod;
                if let Some(keycode) = keycode {
                    self.keyboard.key(keycode).release();
                } else {
                    warn!("Some key was pressed, but keycode is None");
                }
            },
            Event::MouseWheel { x, y, direction, .. } => {
                self.mouse.scroll += match direction {
                    MouseWheelDirection::Flipped => Vec2::new(-x as _, -y as _),
                    _ => Vec2::new(x as _, y as _),
                };
            },
            Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                let _ = clicks;
                self.mouse.set_position(Vec2::new(x as _, y as _));
                match mouse_btn {
                    MouseButton::Left => self.mouse.left.press(),
                    MouseButton::Middle => self.mouse.middle.press(),
                    MouseButton::Right => self.mouse.right.press(),
                    _ => (),
                };
            },
            Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
                let _ = clicks;
                self.mouse.set_position(Vec2::new(x as _, y as _));
                match mouse_btn {
                    MouseButton::Left => self.mouse.left.release(),
                    MouseButton::Middle => self.mouse.middle.release(),
                    MouseButton::Right => self.mouse.right.release(),
                    _ => (),
                };
            },
            Event::MouseMotion { mousestate, x, y, xrel, yrel, .. } => {
                let _ = mousestate;
                let _ = xrel;
                let _ = yrel;
                self.mouse.set_position(Vec2::new(x as _, y as _));
            },
            // TODO FIXME: MouseEnter and MouseLeave
            _ => (),
        };
    }
}


impl MouseInput {
    pub fn begin_tick_and_event_processing(&mut self) {
        let &mut Self {
            ref mut left, ref mut middle, ref mut right,
            ref mut scroll,
            ref mut position, ref mut prev_position,
            has_prev_position: _,
        } = self;
        left.begin_tick_and_event_processing();
        middle.begin_tick_and_event_processing();
        right.begin_tick_and_event_processing();
        *scroll = Vec2::zero();
        *prev_position = *position;
    }
    pub fn set_position(&mut self, p: Vec2<u32>) {
        if !self.has_prev_position {
            self.prev_position = p;
        }
        self.position = p;
    }
    pub fn displacement(&self) -> Vec2<u32> {
        if !self.has_prev_position {
            return Vec2::zero();
        }
        self.position - self.prev_position
    }
}

impl KeyboardInput {
    pub fn begin_tick_and_event_processing(&mut self) {
        for key in self.keys.values_mut() {
            key.begin_tick_and_event_processing();
        }
    }
    pub fn key(&mut self, keycode: Keycode) -> &mut ButtonInput {
        self.keys.entry(keycode).or_insert(Default::default())
    }
}

impl ButtonInput {
    pub fn was_just_pressed(&self) -> bool {
        if self.num_changes_since_last_tick_and_event_processing > 1 {
            return true;
        }
        self.is_pressed && !self.was_pressed
    }
    pub fn was_just_released(&self) -> bool {
        if self.num_changes_since_last_tick_and_event_processing > 1 {
            return true;
        }
        !self.is_pressed && self.was_pressed
    }
    pub fn begin_tick_and_event_processing(&mut self) {
        self.was_pressed = self.is_pressed;
        self.num_changes_since_last_tick_and_event_processing = 0;
    }
    pub fn press(&mut self) {
        if self.is_pressed { return; }
        self.num_changes_since_last_tick_and_event_processing += 1;
        self.is_pressed = true;
    }
    pub fn release(&mut self) {
        if !self.is_pressed { return; }
        self.num_changes_since_last_tick_and_event_processing += 1;
        self.is_pressed = false;
    }
}

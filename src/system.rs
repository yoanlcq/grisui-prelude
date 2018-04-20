pub use std::time::Duration;
pub use sdl2::event::{Event as Sdl2Event, WindowEvent};
pub use sdl2::mouse::{MouseWheelDirection, MouseButton};
pub use sdl2::keyboard::{Keycode};
pub use v::{Extent2, Vec2};
pub use game::Game;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Hello,
    Goodbye,
}


pub trait System {
    fn name(&self) -> &str;
    fn on_quit_requested(&mut self, _g: &Game) {}
    fn on_text_input(&mut self, _g: &Game, _text: &str) {}
    fn on_key(&mut self, _g: &Game, _keycode: Keycode, _state: KeyState) {}
    fn on_mouse_wheel(&mut self, _g: &Game, _delta: Vec2<i32>) {}
    fn on_mouse_motion(&mut self, _g: &Game, _pos: Vec2<i32>) {}
    fn on_mouse_button(&mut self, _g: &Game, _btn: MouseButton, _state: ButtonState) {}
    fn on_mouse_enter(&mut self, _g: &Game) {}
    fn on_mouse_leave(&mut self, _g: &Game) {}
    fn on_canvas_resized(&mut self, _g: &Game, _size: Extent2<u32>, _by_user: bool) {}

    fn on_message(&mut self, _g: &Game, _msg: &Message) {}

    /// Replace previous state by current, and compute current state.
    fn tick(&mut self, _g: &Game, _t: Duration, _dt: Duration) {}
    /// Computes render state via interp, then renders.
    fn draw(&mut self, _g: &Game, _interp: f64) {}
}

pub fn dispatch_message(esys: &mut System, g: &Game, msg: &Message) {
    esys.on_message(g, msg);
}

pub fn dispatch_sdl2_event(esys: &mut System, g: &Game, event: &Sdl2Event) {
    match *event {
        Sdl2Event::Window { win_event, .. } => match win_event {
            WindowEvent::Resized(w, h) => {
                esys.on_canvas_resized(g, Extent2::new(w as _, h as _), true);
            },
            WindowEvent::SizeChanged(w, h) => {
                esys.on_canvas_resized(g, Extent2::new(w as _, h as _), false);
            },
            WindowEvent::Enter => {
                esys.on_mouse_enter(g);
            },
            WindowEvent::Leave => {
                esys.on_mouse_leave(g);
            },
            _ => (),
        },
        Sdl2Event::Quit {..} => {
            esys.on_quit_requested(g);
        },
        // Sdl2Event::TextEditing { text, start, length, .. } => {},
        Sdl2Event::TextInput { ref text, .. } => {
            esys.on_text_input(g, &text);
        },
        Sdl2Event::KeyDown { keycode, repeat, scancode: _, keymod: _, .. } => {
            if !repeat {
                if let Some(keycode) = keycode {
                    esys.on_key(g, keycode, KeyState::Down);
                } else {
                    warn!("Some key was pressed, but keycode is None");
                }
            }
        },
        Sdl2Event::KeyUp { keycode, scancode: _, keymod: _, .. } => {
            if let Some(keycode) = keycode {
                esys.on_key(g, keycode, KeyState::Up);
            } else {
                warn!("Some key was released, but keycode is None");
            }
        },
        Sdl2Event::MouseWheel { x, y, direction, .. } => {
            let sign = match direction {
                MouseWheelDirection::Flipped => -1,
                _ => 1,
            };
            esys.on_mouse_wheel(g, Vec2::new(x as _, y as _) * sign);
        },
        Sdl2Event::MouseMotion { mousestate: _, x, y, xrel: _, yrel: _, .. } => {
            esys.on_mouse_motion(g, Vec2::new(x as _, y as _));
        },
        Sdl2Event::MouseButtonDown { mouse_btn, clicks: _, x, y, .. } => {
            esys.on_mouse_motion(g, Vec2::new(x as _, y as _));
            esys.on_mouse_button(g, mouse_btn, KeyState::Down);
        },
        Sdl2Event::MouseButtonUp { mouse_btn, clicks: _, x, y, .. } => {
            esys.on_mouse_motion(g, Vec2::new(x as _, y as _));
            esys.on_mouse_button(g, mouse_btn, KeyState::Up);
        },
        // TODO FIXME: MouseEnter and MouseLeave
        _ => (),
    };
}


mod key_state {
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum KeyState {
        Up, Down,
    }
    pub type ButtonState = KeyState;

    impl Default for KeyState {
        fn default() -> Self {
            KeyState::Up
        }
    }

    impl ::std::ops::Not for KeyState {
        type Output = Self;
        fn not(self) -> Self {
            match self {
                KeyState::Down => KeyState::Up,
                KeyState::Up => KeyState::Down,
            }
        }
    }

    impl KeyState {
        pub fn is_down(&self) -> bool {
            match *self {
                KeyState::Down => true,
                KeyState::Up => false,
            }
        }
        pub fn is_up(&self) -> bool {
            !self.is_down()
        }
    }
}
pub use self::key_state::*;



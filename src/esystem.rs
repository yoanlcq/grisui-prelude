use std::time::Duration;
pub use sdl2::event::{Event as Sdl2Event, WindowEvent};
pub use sdl2::mouse::{MouseWheelDirection, MouseButton};
pub use sdl2::keyboard::{Keycode};
use v::{Extent2, Vec2};

pub trait ESystem {
    fn on_quit_requested(&mut self) {}
    fn on_text_input(&mut self, _text: &str) {}
    fn on_key(&mut self, _keycode: Keycode, _state: KeyState) {}
    fn on_mouse_wheel(&mut self, _delta: Vec2<i32>) {}
    fn on_mouse_motion(&mut self, _pos: Vec2<i32>) {}
    fn on_mouse_button(&mut self, _btn: MouseButton, _state: ButtonState) {}
    fn on_canvas_resized(&mut self, _size: Extent2<u32>, _by_user: bool) {}

    fn replace_previous_state_by_current(&mut self) {}
    fn compute_gfx_state_via_lerp_previous_current(&mut self, _alpha: f64) {}
    fn tick(&mut self, _t: Duration, _dt: Duration) {}
    fn draw(&mut self) {}
}

pub fn dispatch_sdl2_event(esys: &mut ESystem, event: &Sdl2Event) {
    match *event {
        Sdl2Event::Window { win_event, .. } => match win_event {
            WindowEvent::Resized(w, h) => {
                esys.on_canvas_resized(Extent2::new(w as _, h as _), true);
            },
            WindowEvent::SizeChanged(w, h) => {
                esys.on_canvas_resized(Extent2::new(w as _, h as _), false);
            },
            _ => (),
        },
        Sdl2Event::Quit {..} => {
            esys.on_quit_requested();
        },
        // Sdl2Event::TextEditing { text, start, length, .. } => {},
        Sdl2Event::TextInput { ref text, .. } => {
            esys.on_text_input(&text);
        },
        Sdl2Event::KeyDown { keycode, repeat, scancode: _, keymod: _, .. } => {
            if !repeat {
                if let Some(keycode) = keycode {
                    esys.on_key(keycode, KeyState::Down);
                } else {
                    warn!("Some key was pressed, but keycode is None");
                }
            }
        },
        Sdl2Event::KeyUp { keycode, scancode: _, keymod: _, .. } => {
            if let Some(keycode) = keycode {
                esys.on_key(keycode, KeyState::Up);
            } else {
                warn!("Some key was released, but keycode is None");
            }
        },
        Sdl2Event::MouseWheel { x, y, direction, .. } => {
            let sign = match direction {
                MouseWheelDirection::Flipped => -1,
                _ => 1,
            };
            esys.on_mouse_wheel(Vec2::new(x as _, y as _) * sign);
        },
        Sdl2Event::MouseMotion { mousestate: _, x, y, xrel: _, yrel: _, .. } => {
            esys.on_mouse_motion(Vec2::new(x as _, y as _));
        },
        Sdl2Event::MouseButtonDown { mouse_btn, clicks: _, x, y, .. } => {
            esys.on_mouse_motion(Vec2::new(x as _, y as _));
            esys.on_mouse_button(mouse_btn, KeyState::Down);
        },
        Sdl2Event::MouseButtonUp { mouse_btn, clicks: _, x, y, .. } => {
            esys.on_mouse_motion(Vec2::new(x as _, y as _));
            esys.on_mouse_button(mouse_btn, KeyState::Up);
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



pub use sdl2::event::{Event, WindowEvent};
pub use sdl2::mouse::{MouseWheelDirection, MouseButton};
pub use sdl2::keyboard::{Keycode};
use v::{Extent2, Vec2};

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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct KeyInput {
    pub keycode: Keycode,
    pub state: KeyState,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MouseButtonInput {
    pub button: MouseButton,
    pub state: ButtonState,
}


pub trait Sdl2EventSubscriber {
    fn on_wants_to_quit(&mut self);
    fn on_text_input(&mut self, text: &str);
    fn on_key(&mut self, key: KeyInput);
    fn on_scroll(&mut self, delta: Vec2<i32>);
    fn on_mouse_motion(&mut self, pos: Vec2<i32>);
    fn on_mouse_button(&mut self, btn: MouseButtonInput);
    fn on_window_resized(&mut self, size: Extent2<u32>);
    fn on_window_size_changed(&mut self, size: Extent2<u32>);
}

pub fn dispatch_sdl2_event(slf: &mut Sdl2EventSubscriber, event: &Event) {
    match *event {
        Event::Window { win_event, .. } => match win_event {
            WindowEvent::Resized(w, h) => {
                slf.on_window_resized(Extent2::new(w as _, h as _));
            },
            WindowEvent::SizeChanged(w, h) => {
                slf.on_window_size_changed(Extent2::new(w as _, h as _));
            },
            _ => (),
        },
        Event::Quit {..} => {
            slf.on_wants_to_quit();
        },
        // Event::TextEditing { text, start, length, .. } => {},
        Event::TextInput { ref text, .. } => {
            slf.on_text_input(&text);
        },
        Event::KeyDown { keycode, repeat, scancode: _, keymod: _, .. } => {
            if !repeat {
                if let Some(keycode) = keycode {
                    slf.on_key(KeyInput { keycode, state: KeyState::Down });
                } else {
                    warn!("Some key was pressed, but keycode is None");
                }
            }
        },
        Event::KeyUp { keycode, scancode: _, keymod: _, .. } => {
            if let Some(keycode) = keycode {
                slf.on_key(KeyInput { keycode, state: KeyState::Up });
            } else {
                warn!("Some key was pressed, but keycode is None");
            }
        },
        Event::MouseWheel { x, y, direction, .. } => {
            let inc = match direction {
                MouseWheelDirection::Flipped => Vec2::new(-x as _, -y as _),
                _ => Vec2::new(x as _, y as _),
            };
            slf.on_scroll(inc);
        },
        Event::MouseButtonDown { mouse_btn, clicks: _, x, y, .. } => {
            slf.on_mouse_motion(Vec2::new(x as _, y as _));
            slf.on_mouse_button(MouseButtonInput { button: mouse_btn, state: KeyState::Down });
        },
        Event::MouseButtonUp { mouse_btn, clicks: _, x, y, .. } => {
            slf.on_mouse_motion(Vec2::new(x as _, y as _));
            slf.on_mouse_button(MouseButtonInput { button: mouse_btn, state: KeyState::Up});
        },
        Event::MouseMotion { mousestate: _, x, y, xrel: _, yrel: _, .. } => {
            slf.on_mouse_motion(Vec2::new(x as _, y as _));
        },
        // TODO FIXME: MouseEnter and MouseLeave
        _ => (),
    };
}


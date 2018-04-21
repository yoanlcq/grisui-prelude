use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use game::Game;
use v::Vec2;
use system::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Input {
    keys: RefCell<HashMap<Keycode, KeyState>>,
    mouse_buttons: RefCell<HashMap<Sdl2MouseButton, ButtonState>>,
    mouse_position: Cell<Vec2<i32>>,
}

impl Input {
    pub fn key(&self, k: Keycode) -> KeyState {
        *self.keys.borrow().get(&k).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_button(&self, btn: MouseButton) -> ButtonState {
        *self.mouse_buttons.borrow().get(&btn.button).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_position(&self) -> Vec2<i32> {
        self.mouse_position.get()
    }
}

pub struct InputSystem;

impl System for InputSystem {
    fn name(&self) -> &str { "InputSystem" }
    fn on_key(&mut self, g: &Game, key: Key) {
        if key.code.is_none() {
            return;
        }
        let keycode = key.code.unwrap();

        *g.input.keys.borrow_mut().entry(keycode).or_insert(key.state) = key.state;

        let send = |msg| g.messages.borrow_mut().push_back(msg);

        match key.code.unwrap() {
            Keycode::G => if key.is_down() {
                send(Message::EditorToggleGrid);
            },
            Keycode::F => if key.is_down() {
                send(Message::EditorToggleDrawGridFirst);
            },
            Keycode::Space => if key.is_down() {
                send(Message::EditorBeginPanCameraViaMouse);
            } else {
                send(Message::EditorEndPanCameraViaMouse);
            },
            _ => (),
        };
    }
    fn on_mouse_button(&mut self, g: &Game, btn: MouseButton) {
        *g.input.mouse_buttons.borrow_mut().entry(btn.button).or_insert(btn.state) = btn.state;
    }
    fn on_mouse_motion(&mut self, g: &Game, pos: Vec2<i32>) {
        g.input.mouse_position.set(pos);
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Key {
    pub code: Option<Keycode>,
    pub scancode: Option<Scancode>,
    pub state: KeyState,
}

impl Key {
    pub fn is_down(&self) -> bool { self.state.is_down() }
    pub fn is_up(&self) -> bool { self.state.is_up() }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MouseButton {
    pub button: Sdl2MouseButton,
    pub state: ButtonState,
}

impl MouseButton {
    pub fn is_down(&self) -> bool { self.state.is_down() }
    pub fn is_up(&self) -> bool { self.state.is_up() }
    pub fn is_left(&self) -> bool { self.button == Sdl2MouseButton::Left }
    pub fn is_middle(&self) -> bool { self.button == Sdl2MouseButton::Middle }
    pub fn is_right(&self) -> bool { self.button == Sdl2MouseButton::Right }
}


pub use self::key_state::*;
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


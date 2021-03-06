use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use game::{Game, GameMode};
use v::Vec2;
use system::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Input {
    keys: RefCell<HashMap<Keycode, KeyState>>,
    mouse_buttons: RefCell<HashMap<Sdl2MouseButton, ButtonState>>,
    mouse_position: Cell<Option<Vec2<i32>>>,
    previous_mouse_position: Cell<Option<Vec2<i32>>>,
}

impl Input {
    pub fn key(&self, k: Keycode) -> KeyState {
        *self.keys.borrow().get(&k).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_button(&self, btn: MouseButton) -> ButtonState {
        *self.mouse_buttons.borrow().get(&btn.button).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_position(&self) -> Option<Vec2<i32>> {
        self.mouse_position.get()
    }
    pub fn previous_mouse_position(&self) -> Option<Vec2<i32>> {
        self.previous_mouse_position.get()
    }
}

pub struct InputSystem;

impl System for InputSystem {
    fn name(&self) -> &str { "InputSystem" }
    fn on_mouse_button(&mut self, g: &Game, btn: MouseButton) {
        *g.input.mouse_buttons.borrow_mut().entry(btn.button).or_insert(btn.state) = btn.state;

        match btn.button {
            Sdl2MouseButton::Left => {},
            Sdl2MouseButton::Middle => {},
            Sdl2MouseButton::Right => {},
            Sdl2MouseButton::Unknown => {},
            Sdl2MouseButton::X1 => {},
            Sdl2MouseButton::X2 => {},
        };
    }
    fn on_mouse_leave(&mut self, g: &Game) {
        g.input.mouse_position.set(None);
    }
    fn on_mouse_motion(&mut self, g: &Game, pos: Vec2<i32>) {
        g.input.previous_mouse_position.set(g.input.mouse_position.get());
        g.input.mouse_position.set(Some(pos));
    }
    fn on_key(&mut self, g: &Game, key: Key) {
        if key.code.is_none() {
            return;
        }
        let keycode = key.code.unwrap();

        *g.input.keys.borrow_mut().entry(keycode).or_insert(key.state) = key.state;

        let send = |msg| g.messages.borrow_mut().push_back(msg);

        match key.code.unwrap() {
            Keycode::F8 => if key.is_down() {
                g.game_mode.set(match g.game_mode.get() {
                    GameMode::Editor => {
                        send(Message::LeaveEditor);
                        send(Message::EnterGameplay);
                        GameMode::Gameplay
                    },
                    GameMode::Gameplay => {
                        send(Message::LeaveGameplay);
                        send(Message::EnterEditor);
                        GameMode::Editor
                    },
                });
            },
            Keycode::F6 => if key.is_down() {
                unsafe {
                    ::gameplay::DO_DRAW_SHAPE_STROKE_LINES = !::gameplay::DO_DRAW_SHAPE_STROKE_LINES;
                }
            },
            Keycode::F7 => if key.is_down() {
                unsafe {
                    ::gameplay::DO_DRAW_SHAPE_STROKE_POINTS = !::gameplay::DO_DRAW_SHAPE_STROKE_POINTS;
                }
            },
            _ => (),
        };
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


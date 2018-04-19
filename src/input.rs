use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use game::Game;
use v::Vec2;
use system::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Input {
    keys: RefCell<HashMap<Keycode, KeyState>>,
    mouse_buttons: RefCell<HashMap<MouseButton, ButtonState>>,
    mouse_position: Cell<Vec2<i32>>,
}

impl Input {
    pub fn key(&self, k: Keycode) -> KeyState {
        *self.keys.borrow().get(&k).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_button(&self, btn: MouseButton) -> ButtonState {
        *self.mouse_buttons.borrow().get(&btn).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_position(&self) -> Vec2<i32> {
        self.mouse_position.get()
    }
}

pub struct InputSystem;

impl System for InputSystem {
    fn name(&self) -> &str { "InputSystem" }
    fn on_key(&mut self, g: &Game, keycode: Keycode, state: KeyState) {
        *g.input.keys.borrow_mut().entry(keycode).or_insert(state) = state;
    }
    fn on_mouse_button(&mut self, g: &Game, btn: MouseButton, state: ButtonState) {
        *g.input.mouse_buttons.borrow_mut().entry(btn).or_insert(state) = state;
    }
    fn on_mouse_motion(&mut self, g: &Game, pos: Vec2<i32>) {
        g.input.mouse_position.set(pos);
    }
}



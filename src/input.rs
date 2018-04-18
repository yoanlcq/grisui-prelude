use std::collections::HashMap;
use v::Vec2;
use esystem::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Input {
    keys: HashMap<Keycode, KeyState>,
    mouse_buttons: HashMap<MouseButton, ButtonState>,
    mouse_position: Vec2<i32>,
}

impl Input {
    pub fn key(&self, k: Keycode) -> KeyState {
        *self.keys.get(&k).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_button(&self, btn: MouseButton) -> ButtonState {
        *self.mouse_buttons.get(&btn).unwrap_or(&KeyState::Up)
    }
    pub fn mouse_position(&self) -> Vec2<i32> {
        self.mouse_position
    }
}


impl ESystem for Input {
    fn on_key(&mut self, keycode: Keycode, state: KeyState) {
        *self.keys.entry(keycode).or_insert(state) = state;
    }
    fn on_mouse_motion(&mut self, pos: Vec2<i32>) {
        self.mouse_position = pos;
    }
    fn on_mouse_button(&mut self, btn: MouseButton, state: ButtonState) {
        *self.mouse_buttons.entry(btn).or_insert(state) = state;
    }
}



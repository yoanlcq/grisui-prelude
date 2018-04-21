pub use std::time::Duration;
pub use sdl2::event::{Event as Sdl2Event, WindowEvent};
pub use sdl2::mouse::{MouseWheelDirection, MouseButton as Sdl2MouseButton};
pub use sdl2::keyboard::{Keycode, Scancode};
pub use v::{Extent2, Vec2};
pub use game::Game;
pub use message::Message;
pub use input::{Key, MouseButton, KeyState};

pub trait System {
    fn name(&self) -> &str;
    fn on_quit_requested(&mut self, _g: &Game) {}
    fn on_text_input(&mut self, _g: &Game, _text: &str) {}
    fn on_key(&mut self, _g: &Game, _key: Key) {}
    fn on_mouse_scroll(&mut self, _g: &Game, _delta: Vec2<i32>) {}
    fn on_mouse_motion(&mut self, _g: &Game, _pos: Vec2<i32>) {}
    fn on_mouse_button(&mut self, _g: &Game, _btn: MouseButton) {}
    fn on_mouse_enter(&mut self, _g: &Game) {}
    fn on_mouse_leave(&mut self, _g: &Game) {}
    fn on_canvas_resized(&mut self, _g: &Game, _size: Extent2<u32>, _by_user: bool) {}

    fn on_message(&mut self, _g: &Game, _msg: &Message) {}

    /// Replace previous state by current, and compute current state.
    fn tick(&mut self, _g: &Game, _t: Duration, _dt: Duration) {}
    /// Computes render state via interp, then renders.
    fn draw(&mut self, _g: &Game, _interp: f64) {}
}

// This function exists in case `on_message` gets split into multiple
// functions instead one day. e.g `on_editor_message`, `on_network_message`...
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
                esys.on_mouse_motion(g, g.platform.mouse_position());
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
        Sdl2Event::KeyDown { keycode, repeat, scancode, keymod: _, .. } => {
            if !repeat {
                esys.on_key(g, Key { code: keycode, scancode, state: KeyState::Down, });
            }
        },
        Sdl2Event::KeyUp { keycode, scancode, keymod: _, .. } => {
            esys.on_key(g, Key { code: keycode, scancode, state: KeyState::Up, });
        },
        Sdl2Event::MouseWheel { x, y, direction, .. } => {
            let sign = match direction {
                MouseWheelDirection::Flipped => -1,
                _ => 1,
            };
            esys.on_mouse_scroll(g, Vec2::new(x as _, y as _) * sign);
        },
        Sdl2Event::MouseMotion { mousestate: _, x, y, xrel: _, yrel: _, .. } => {
            esys.on_mouse_motion(g, Vec2::new(x as _, y as _));
        },
        Sdl2Event::MouseButtonDown { mouse_btn, clicks: _, x, y, .. } => {
            esys.on_mouse_motion(g, Vec2::new(x as _, y as _));
            esys.on_mouse_button(g, MouseButton { button: mouse_btn, state: KeyState::Down });
        },
        Sdl2Event::MouseButtonUp { mouse_btn, clicks: _, x, y, .. } => {
            esys.on_mouse_motion(g, Vec2::new(x as _, y as _));
            esys.on_mouse_button(g, MouseButton { button: mouse_btn, state: KeyState::Up });
        },
        _ => (),
    };
}


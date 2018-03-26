#[macro_use] #[allow(unused_imports)]
extern crate static_assertions;
#[macro_use] #[allow(unused_imports)]
extern crate pretty_assertions;
extern crate vek;
extern crate sdl2;
extern crate gl;
extern crate alto;
extern crate freetype_sys;
#[macro_use] #[allow(unused_imports)]
extern crate serde;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use] #[allow(unused_imports)]
extern crate id_realm;
extern crate backtrace;

pub mod early;
pub mod game;
pub mod v;
pub mod grx;
pub mod gx;

use game::Game;

fn main() {
    early::setup_panic_hook();
    early::setup_env();
    early::setup_log();
    let mut g = Game::new("Grisui - Prelude", 800, 600);

    'running: for _frame_i in 0.. {
        g.pump_events();
        if g.should_quit() {
            break 'running;
        }
        g.render_clear();
        g.render();
        g.present();
    }
}


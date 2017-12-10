extern crate sdl2;
extern crate gl;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::time::{Instant, Duration};
use std::thread;

pub mod gx;

pub mod game;
use game::Game;

fn main() {

    let mut game = Game::new();

    let recommended_refresh_rate = game.window.display_mode().unwrap().refresh_rate;
    let mut lim_last_time = Instant::now();
    let mut last_time = Instant::now();
    let mut frame_accum = 0u64;
    let mut fps_limit = 0f64;
    let fps_ceil = 60f64;
    let fps_counter_interval = 1000f64; /* Should be in [100, 1000] */

    let mut event_pump = game.sdl.event_pump().unwrap();

    while !game.should_quit {

        for event in event_pump.poll_iter() {
            game.handle_sdl2_event(event);
        }

        /* See http://www.opengl-tutorial.org/miscellaneous/an-fps-counter/ */
        frame_accum += 1;
        let current_time = Instant::now();
        if current_time.duration_since(last_time) > Duration::from_millis(fps_counter_interval as _) {
            let fps = ((frame_accum as f64) * 1000f64 / fps_counter_interval).round() as u32;
            info!(concat!("{} frames under {} milliseconds = ",
                "{} milliseconds/frame = ",
                "{} FPS"), 
                frame_accum,
                fps_counter_interval,
                fps_counter_interval / (frame_accum as f64), 
                fps
            );
            frame_accum = 0;
            last_time += Duration::from_millis(fps_counter_interval as _);
            if fps_limit <= 0_f64 && fps as f64 > fps_ceil {
                let reason = if recommended_refresh_rate != 0 {
                    fps_limit = recommended_refresh_rate as _;
                    "from display mode info"
                } else {
                    fps_limit = fps_ceil;
                    "fallback"
                };
                warn!(concat!("Abnormal FPS detected (Vsync is not working). ",
                    "Now limiting FPS to {} ({})."),
                    fps_limit, reason
                );
            }
        }

        game.render_clear();
        game.render();
        game.present();

        if fps_limit > 0_f64 {
            let current_time = Instant::now();
            let a_frame = Duration::from_millis((1000_f64 / fps_limit).round() as _);
            if current_time - lim_last_time < a_frame {
                thread::sleep(a_frame - (current_time - lim_last_time));
            }
            lim_last_time = Instant::now();
        }
    }
}


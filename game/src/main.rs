extern crate vek;
extern crate sdl2;
extern crate gl;
extern crate alto;
extern crate freetype_sys;
#[allow(unused_imports)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate log;
extern crate env_logger;


use std::time::{Instant, Duration};
use std::thread;

pub mod duration_ext;
use duration_ext::DurationExt;

pub mod gx;

pub mod game;
use game::{Game, GameState};

// TODO:
// - Display arbitrary text with FreeType;
// - Play some sounds with OpenAL;
// - Have a proper 3D scene with a movable camera;
// - Create the Bézier path editor;
// - Set up async asset pipeline;

// NOTE: The main loop is messy as hell, because it is inhabited by :
// - An FPS counter;
// - An FPS limiter;
// - A "Fix Your Timestep!" implementation.

fn main() {

    let mut game = Game::new();

    let recommended_refresh_rate = game.window.display_mode().unwrap().refresh_rate;
    let mut current_time = Instant::now();
    let mut lim_last_time = current_time;
    let mut last_time = current_time;
    let mut last_frame_time = current_time;
    let mut frame_accum = 0u64;
    let mut fps_limit = 0f64;
    let fps_ceil = 60f64;
    let fps_counter_interval = 1000f64; /* Should be in [100, 1000] */

    let mut t = Duration::default();
    let dt = Duration::from_millis(100);
    let mut accumulator = Duration::default();

    let mut event_pump = game.sdl.event_pump().unwrap();

    while !game.should_quit {

        for event in event_pump.poll_iter() {
            game.handle_sdl2_event(event);
        }

        current_time = Instant::now();

        // See http://www.opengl-tutorial.org/miscellaneous/an-fps-counter/
        frame_accum += 1;
        if current_time.duration_since(last_frame_time) > Duration::from_millis(fps_counter_interval as _) {
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
            last_frame_time += Duration::from_millis(fps_counter_interval as _);
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

        // https://gafferongames.com/post/fix_your_timestep/
        let mut frame_time = current_time - last_time;
        if frame_time > Duration::from_millis(250) {
            frame_time = Duration::from_millis(250);
        }
        last_time = current_time;

        accumulator += frame_time;

        //info!("accumulator={:?}, dt={:?}", accumulator, dt);
        while accumulator >= dt {
            game.previous_state = game.current_state.clone();
            game.current_state.integrate(t, dt);
            t += dt;
            accumulator -= dt;
        }

        let alpha = accumulator.to_f64_seconds() / dt.to_f64_seconds();
        let state = GameState::lerp(&game.previous_state, &game.current_state, alpha);

        game.render_clear();
        game.render(&state);
        game.present();

        if fps_limit > 0_f64 {
            let a_frame = Duration::from_millis((1000_f64 / fps_limit).round() as _);
            if current_time - lim_last_time < a_frame {
                thread::sleep(a_frame - (current_time - lim_last_time));
            }
            lim_last_time = Instant::now();
        }
    }
}


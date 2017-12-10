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
    let mut lim_last_time = Instant::now();
    let mut last_time = Instant::now();
    let mut frame_accum = 0u64;
    let mut fps_limit = 0f64;
    let fps_ceil = 60f64;
    let fps_counter_interval = 1000f64; /* Should be in [100, 1000] */

    let mut event_pump = game.sdl.event_pump().unwrap();

    let mut t = 0_f64;
    let dt = 0.01_f64;

    let mut cur_time: f64 = hires_time_in_seconds();
    let mut accumulator = 0_f64;

    while !game.should_quit {

        for event in event_pump.poll_iter() {
            game.handle_sdl2_event(event);
        }

        // See http://www.opengl-tutorial.org/miscellaneous/an-fps-counter/
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

        // https://gafferongames.com/post/fix_your_timestep/
        let new_time: f64 = hires_time_in_seconds();
        let mut frame_time = new_time - cur_time;
        if frame_time > 0.25_f64 {
            frame_time = 0.25_f64;
        }
        cur_time = new_time;

        accumulator += frame_time;

        info!("accumulator={}, dt={}", accumulator, dt);
        while accumulator >= dt {
            game.previous_state = game.current_state.clone();
            game.current_state.integrate(t, dt);
            t += dt;
            accumulator -= dt;
        }

        let state = GameState::lerp(&game.previous_state, &game.current_state, accumulator / dt);

        game.render_clear();
        game.render(&state);
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

fn hires_time_in_seconds() -> f64 {
    let d = Instant::now().elapsed();
    d.as_secs() as f64 + (d.subsec_nanos() as f64 / 1_000_000_000_f64)
}

// TODO:
// - Map window space to world space;
// - Use OrthographicCamera instead of PerspectiveCamera;
// - Have toggleable sky gradient;
// - Set panic handler to display a message box;
// - Display arbitrary text with FreeType;
// - Play some sounds with OpenAL;
// - Have a proper 3D scene with a movable camera;
// - Create the Bézier path editor;
// - Set up async asset pipeline;

#![allow(unused_imports)]

#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate static_assertions;
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

pub mod v {
    // NOTE: Avoid repr_simd for alignment reasons (when sending packed data to OpenGL)
    // Also, it's more convenient. repr_simd is better for mass processing.
    pub use vek::vec::repr_c::{Vec4, Rgba};
    pub use vek::quaternion::repr_c::Quaternion;
    pub use vek::vec::repr_c::{Vec2, Vec3, Rgb, Extent2};
    pub use vek::mat::repr_c::column_major::{Mat4,};
    pub use vek::mat::repr_c::column_major::{Mat3, Mat2};
    pub use vek::ops::*;
    pub use vek::geom::*;
    pub use vek::transform::repr_c::Transform;

    assert_eq_size!(mat4_f32_size; Mat4<f32>, [f32; 16]);
    assert_eq_size!(vec4_f32_size; Vec4<f32>, [f32; 4]);
    assert_eq_size!(rgba_f32_size; Rgba<f32>, [f32; 4]);
    assert_eq_size!(rgba_u8_size ; Rgba<u8>, [u8; 4]);
}

use v::{Rect, Vec2};

pub mod duration_ext;
use duration_ext::DurationExt;
pub mod transform_ext;
use transform_ext::TransformExt;
pub mod camera;
pub mod gx;
pub mod grx;
pub mod global;
pub mod lazy;
use global::{Global, TickInfo, FrameInfo, Scene};

// NOTE: The main loop is messy as hell, because it is inhabited by :
// - An FPS counter;
// - An FPS limiter;
// - A "Fix Your Timestep!" implementation.

fn main() {

    let mut g = Global::default();
    let mut scene = Scene::new_test_room(Rect::from((Vec2::zero(), g.viewport_size)));

    let mut frame_i = 0_u64;
    let recommended_refresh_rate = g.window.display_mode().unwrap().refresh_rate;
    let mut current_time = Instant::now();
    let mut lim_last_time = current_time;
    let mut last_time = current_time;
    let mut last_frame_time = current_time;
    let mut frame_accum = 0u64;
    let mut fps_limit = 0f64;
    let fps_ceil = 60f64;
    let fps_counter_interval = 1000f64; /* Should be in [100, 1000] */

    let mut tick_i = 0_u64;
    let mut t = Duration::default();
    let dt = Duration::from_millis(50);
    let mut accumulator = Duration::default();

    let mut event_pump = g.sdl.event_pump().unwrap();

    'running: loop {

        current_time = Instant::now();

        // See http://www.opengl-tutorial.org/miscellaneous/an-fps-counter/
        frame_accum += 1;
        if current_time.duration_since(last_frame_time) > Duration::from_millis(fps_counter_interval as _) {
            let fps = ((frame_accum as f64) * 1000f64 / fps_counter_interval).round() as u32;
            trace!(concat!("{} frames under {} milliseconds = ",
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

        while accumulator >= dt {
            scene.replace_previous_state_by_current();
            g.replace_previous_state_by_current();
            for event in event_pump.poll_iter() {
                g.handle_sdl2_event_before_new_tick(&event);
                scene.handle_sdl2_event_before_new_tick(&event);
            }
            scene.integrate(TickInfo {
                t, dt, tick_i, g: &mut g,
            });
            tick_i += 1;

            if g.input.wants_to_quit && scene.allows_quitting {
                break 'running;
            }

            t += dt;
            accumulator -= dt;
        }
        
        g.render_clear();
        scene.render(FrameInfo {
            frame_i,
            lerp_factor_between_previous_and_current:
                accumulator.to_f64_seconds() / dt.to_f64_seconds(),
            g: &mut g,
        });
        g.present();
        frame_i += 1;

        if fps_limit > 0_f64 {
            let a_frame = Duration::from_millis((1000_f64 / fps_limit).round() as _);
            if current_time - lim_last_time < a_frame {
                thread::sleep(a_frame - (current_time - lim_last_time));
            }
            lim_last_time = Instant::now();
        }
    }
}


// Today:
// - Use callback-driven event dispatch
// - Investigate a way to make text rendering faster
// - Clean-up text rendering
// - Clean-up Path rendering
// - Make the Bézier curve editor (separate mode)
//
// TODO:
// - Stroke style for shapes;
//   - Solution: Use GL_LINES and draw screen-space-sized disks at the caps
// - Color picker ???
// - Create the Bézier path editor;
// - post-processing FX ?????
// - Load resource files, as well as async ones. Async resource manager if you will.
//
// - Play some sounds with OpenAL;
// - Play music continusouly across loadings;
//
// WONTFIX:
// - Mouse position at the beginning: SDL_GetMouseState() just doesn't work.
//   XQueryPointer would do the trick.
// - FTP for assets:
//   Use the Git repo first, then see if we reach the limit.
//   If so, move the repo to a new one and start using ftp.
// - How to render Debug text last so it is drawn over other kinds of text ?
//   Just don't care for now.
//
// Inspirations:
// - Bionicle MNOG;
// - Night In The Woods;
// - Mimpi Dreams;

// #![allow(unused_imports)]

#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate static_assertions;
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
#[macro_use]
extern crate id_realm;
extern crate backtrace;

pub mod v {
    extern crate vek;
    // NOTE: Avoid repr_simd for alignment reasons (when sending packed data to OpenGL)
    // Also, it's more convenient. repr_simd is better for mass processing.
    pub use self::vek::vec::repr_c::{Vec4, Rgba};
    pub use self::vek::quaternion::repr_c::Quaternion;
    pub use self::vek::vec::repr_c::{Vec2, Vec3, Rgb, Extent2};
    pub use self::vek::mat::repr_c::column_major::{Mat4,};
    pub use self::vek::mat::repr_c::column_major::{Mat3, Mat2};
    pub use self::vek::ops::*;
    pub use self::vek::geom::*;
    pub use self::vek::transform::repr_c::Transform;

    assert_eq_size!(mat4_f32_size; Mat4<f32>, [f32; 16]);
    assert_eq_size!(vec4_f32_size; Vec4<f32>, [f32; 4]);
    assert_eq_size!(rgba_f32_size; Rgba<f32>, [f32; 4]);
    assert_eq_size!(rgba_u8_size ; Rgba<u8>, [u8; 4]);
}

// NOTE: Keep sorted alphabetically, for convenience
pub mod camera;
pub mod duration_ext;
pub mod fonts;
pub mod global;
pub mod grx;
pub mod gx;
pub mod ids;
// pub mod input;
pub mod events;
pub mod lazy;
pub mod mesh;
pub mod save;
pub mod scene;
pub mod transform;
pub mod transform_ext;

// NOTE: The main loop is messy as hell, because it is inhabited by :
// - An FPS counter;
// - An FPS limiter;
// - A "Fix Your Timestep!" implementation.

fn main() {

    use std::time::{Instant, Duration};
    use std::thread;
    use scene::Scene;
    use global::{Global, GlobalDataUpdatePack, FpsStats};
    use duration_ext::DurationExt;
    use events;

    info!("Initializing...");
    let mut g = Global::default();
    info!("Loading test room scene...");
    let mut scene = Scene::new_test_room(&g);

    let mut frame_i = 0_u64;
    let recommended_refresh_rate = g.window.display_mode().unwrap().refresh_rate;
    let mut current_time = Instant::now();
    let mut lim_last_time = current_time;
    let mut last_time = current_time;
    let mut last_frame_time = current_time;
    let mut frame_accum = 0u64;
    let mut fps_limit = 0f64;
    let fps_ceil = 200_f64; // If we go above that, there's a problem, and we should limit FPS manually.
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
            let fps = (frame_accum as f64) * 1000f64 / fps_counter_interval;

            g.fps_stats = FpsStats {
                frames_under_milliseconds: (frame_accum, fps_counter_interval),
                milliseconds_per_frame: fps_counter_interval / (frame_accum as f64),
                fps,
            };
            trace!(concat!("{} frames under {} milliseconds = ",
                "{} milliseconds/frame = ",
                "{} FPS"), 
                g.fps_stats.frames_under_milliseconds.0,
                g.fps_stats.frames_under_milliseconds.1,
                g.fps_stats.milliseconds_per_frame, 
                g.fps_stats.fps
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
            g    .replace_previous_state_by_current();
            for event in event_pump.poll_iter() {
                events::dispatch_sdl2_event(&mut scene, &event);
                events::dispatch_sdl2_event(&mut g    , &event);
            }
            scene.integrate(GlobalDataUpdatePack { t, dt, tick_i, frame_i, g: &mut g, });
            tick_i += 1;

            if scene.wants_to_quit && scene.allows_quitting
            && g    .wants_to_quit && g    .allows_quitting {
                break 'running;
            }

            t += dt;
            accumulator -= dt;
        }
        
        let alpha = accumulator.to_f64_seconds() / dt.to_f64_seconds();
        scene.prepare_render_state_via_lerp_previous_current(alpha);

        for event in event_pump.poll_iter() {
            events::dispatch_sdl2_event(&mut scene, &event);
            events::dispatch_sdl2_event(&mut g    , &event);
        }

        g.render_clear(scene.clear_color);
        scene.render(GlobalDataUpdatePack { t, dt, tick_i, frame_i, g: &mut g, });
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


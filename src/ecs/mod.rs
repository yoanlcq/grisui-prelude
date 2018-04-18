use std::time::Duration;
use events::{self, Sdl2EventSubscriber, Event};
use game::{PhysicsUpdate, GfxUpdate};

pub mod transforms;
pub mod shapes;
pub mod eid;

#[derive(Debug, Default, PartialEq)]
pub struct World {
    pub allows_quitting: bool,
    pub transforms: transforms::Transforms,
    pub shapes: shapes::Shapes,
}

impl Sdl2EventSubscriber for World {
    fn on_wants_to_quit(&mut self) {
        info!("World: Received 'Quit' event");
        self.allows_quitting = true;
    }
}

impl PhysicsUpdate for World {
    fn replace_previous_state_by_current(&mut self) {
        for s in &mut self.physics_update_mut_array() {
            s.replace_previous_state_by_current();
        }
    }
    fn integrate(&mut self, t: Duration, dt: Duration) {
        for s in &mut self.physics_update_mut_array() {
            s.integrate(t, dt);
        }
    }
}

impl GfxUpdate for World {
    fn compute_gfx_state_via_lerp_previous_current(&mut self, alpha: f64) {
        for s in &mut self.gfx_update_mut_array() {
            s.compute_gfx_state_via_lerp_previous_current(alpha);
        }
    }
    fn render(&mut self) {
        for s in &mut self.gfx_update_mut_array() {
            s.render();
        }
    }
}

impl World {
    pub fn dispatch_sdl2_event(&mut self, event: &Event) {
        events::dispatch_sdl2_event(self, event);
        for s in &mut self.sdl2_event_subscribers_mut_array() {
            events::dispatch_sdl2_event(*s, event);
        }
    }
    fn sdl2_event_subscribers_mut_array(&mut self) -> [&mut Sdl2EventSubscriber; 2] {
        let &mut Self {
            allows_quitting: _,
            ref mut transforms,
            ref mut shapes,
        } = self;
        [
            transforms,
            shapes,
        ]
    }
    fn physics_update_mut_array(&mut self) -> [&mut PhysicsUpdate; 2] {
        let &mut Self {
            allows_quitting: _,
            ref mut transforms,
            ref mut shapes,
        } = self;
        [
            transforms,
            shapes,
        ]
    }
    fn gfx_update_mut_array(&mut self) -> [&mut GfxUpdate; 2] {
        let &mut Self {
            allows_quitting: _,
            ref mut transforms,
            ref mut shapes,
        } = self;
        [
            transforms,
            shapes,
        ]
    }
}

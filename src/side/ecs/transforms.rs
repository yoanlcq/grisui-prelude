use std::time::Duration;
use std::ops::{Deref, DerefMut};
use super::eid::*;
use events::Sdl2EventSubscriber;
use game::{PhysicsUpdate, GfxUpdate};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Transforms {
    transforms: EIDMap<Transform>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Transform {}

impl Deref for Transforms {
    type Target = EIDMap<Transform>;
    fn deref(&self) -> &Self::Target {
        &self.transforms
    }
}
impl DerefMut for Transforms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transforms
    }
}

impl Sdl2EventSubscriber for Transforms {}
impl PhysicsUpdate for Transforms {
    fn replace_previous_state_by_current(&mut self) {}
    fn integrate(&mut self, _t: Duration, _dt: Duration) {}
}
impl GfxUpdate for Transforms {
    fn compute_gfx_state_via_lerp_previous_current(&mut self, _alpha: f64) {}
    fn render(&mut self) {}
}

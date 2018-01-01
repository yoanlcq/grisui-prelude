use std::ops::{Index, IndexMut};
use ids::*;
use v::{Rgb, Transform, Vec2, Extent2, Rect, Lerp, Mat4, Vec3};
use sdl2::event::{Event, WindowEvent};
use camera::OrthographicCamera;
use gl;
use global::GlobalDataUpdatePack;
use duration_ext::DurationExt;

#[derive(Debug)]
pub struct Scene {
    pub allows_quitting: bool,
    pub clear_color: Rgb<u8>,
    pub entity_id_domain: EntityIDDomain,
    pub transforms: EntityIDMap<SimStates<Transform<f32,f32,f32>>>,
    pub cameras: EntityIDMap<SimStates<OrthographicCamera>>,
    pub meshes: EntityIDMap<MeshID>,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SimStates<T> {
    pub previous: T,
    pub render: T,
    pub current: T,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimState {
    Previous = 0,
    Render = 1,
    Current = 2, 
}


impl<T: Copy> From<T> for SimStates<T> {
    fn from(value: T) -> Self {
        let previous = value;
        let current = value;
        let render = value;
        Self { previous, current, render }
    }
}

impl<T> SimStates<T> {
    pub fn for_states<F>(&mut self, f: F) where F: Fn(&mut T) {
        f(&mut self.previous);
        f(&mut self.render);
        f(&mut self.current);
    }
    pub fn map_states<F>(self, f: F) -> Self where F: Fn(T) -> T {
        let Self { previous, render, current } = self;
        let previous = f(previous);
        let render = f(render);
        let current = f(current);
        Self { previous, render, current }
    }
}

impl<T> Index<SimState> for SimStates<T> {
    type Output = T;
    fn index(&self, i: SimState) -> &T {
        match i {
            SimState::Previous => &self.previous,
            SimState::Render => &self.render,
            SimState::Current => &self.current,
        }
    }
}

impl<T> IndexMut<SimState> for SimStates<T> {
    fn index_mut(&mut self, i: SimState) -> &mut T {
        match i {
            SimState::Previous => &mut self.previous,
            SimState::Render => &mut self.render,
            SimState::Current => &mut self.current,
        }
    }
}


impl Scene {
    pub fn handle_sdl2_event_before_new_tick(&mut self, event: &Event) {
        match *event {
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                WindowEvent::SizeChanged(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                _ => (),
            },
            _ => (),
        };
    }
    pub fn reshape(&mut self, viewport_size: Extent2<u32>) {
        for camera in self.cameras.values_mut() {
            // NOTE: Every camera might want to handle this differently
            let vp = Rect::from((Vec2::zero(), viewport_size));
            camera.for_states(|s| s.viewport = vp);
        }
    }
    pub fn replace_previous_state_by_current(&mut self) {
        for xform in self.transforms.values_mut() {
            xform.previous = xform.current;
        }
    }
    pub fn integrate(&mut self, tick: GlobalDataUpdatePack) {
        use ::std::f32::consts::PI;
        let dt = tick.dt.to_f64_seconds() as f32;
        let t = tick.t.to_f64_seconds() as f32;
        let xform = &mut self.transforms.get_mut(&EntityID::from_raw(1)).unwrap().current;
        xform.position.x = (PI * t).sin();
        xform.orientation.rotate_z(PI * dt);
        xform.scale = Vec3::broadcast((PI * t).sin());
    }
    pub fn prepare_render_state_via_lerp_previous_current(&mut self, alpha: f64) {
        let alpha = alpha as f32;
        for xform in self.transforms.values_mut() {
            xform.render = Lerp::lerp(xform.previous, xform.current, alpha);
        }
    }
    pub fn debug_entity_id(&self, eid: EntityID) {
        let head = format!("Components of {:?}:", eid);
        let mut cpts = String::new();
        if let Some(x) = self.transforms.get(&eid) { cpts += &format!("\n- {:?}", x.current); }
        if let Some(x) = self.cameras.get(&eid) { cpts += &format!("\n- {:?}", x.current); }
        if let Some(x) = self.meshes.get(&eid) { cpts += &format!("\n- {:?}", x); }
        info!("{}{}", &head, if cpts.is_empty() { "None at all!" } else { &cpts });
    }
    pub fn render(&mut self, frame: GlobalDataUpdatePack) {
        for (camera_eid, camera) in self.cameras.iter().map(|(id, c)| (id, &c.render)) {
            let camera_xform = &self.transforms[camera_eid].render;
            let view = camera.view_matrix(camera_xform);
            let proj = camera.proj_matrix();
            let Rect { x, y, w, h } = camera.viewport;
            unsafe {
                gl::Viewport(x as _, y as _, w as _, h as _);
            }
            frame.g.gl_simple_color_program.use_program(&Mat4::identity());
            for (mesh_eid, mesh_id) in self.meshes.iter() {
                let mesh = frame.g.meshes[*mesh_id].as_ref().unwrap();
                let mesh_xform = self.transforms[mesh_eid].render;
                /*
                let mesh_xform = {
                    let mut xform = Transform::default();
                    let mut mousepos = frame.g.input.mouse.position.map(|x| x as f32);
                    mousepos.y = frame.g.viewport_size.h as f32 - mousepos.y;
                    xform.position = camera.viewport_to_world_point(
                        camera_xform, mousepos.into()
                    );
                    xform.orientation.rotate_z(frame.frame_i as f32 / 20.);
                    xform.position.z = 1.;
                    xform.scale /= 24.;
                    xform
                };
                */
                let model = Mat4::from(mesh_xform);
                let mvp = proj * view * model;
                frame.g.gl_simple_color_program.set_uniform_mvp(&mvp);
                mesh.vao.bind();
                unsafe {
                    gl::DrawArrays(mesh.gl_topology, 0, mesh.vertices.len() as _);
                }
            }
        }
    }

    pub fn new_test_room(viewport: Rect<u32, u32>) -> Self {
        let mut entity_id_domain = EntityIDDomain::new_empty();
        let hasher_builder = EntityIDHasherBuilder::default();
        let mut meshes = EntityIDMap::with_capacity_and_hasher(1, hasher_builder);
        let mut transforms = EntityIDMap::with_capacity_and_hasher(2, hasher_builder);
        let mut cameras = EntityIDMap::with_capacity_and_hasher(1, hasher_builder);

        let camera_id = EntityID::from_raw(0);
        let quad_id = EntityID::from_raw(1);
        entity_id_domain.include_id(camera_id);
        entity_id_domain.include_id(quad_id);

        transforms.insert(camera_id, Transform::default().into());
        cameras.insert(camera_id, SimStates::from(OrthographicCamera {
            viewport, ortho_right: 1., near: 0.01, far: 100.,
        }));

        transforms.insert(quad_id, {
            let mut xform = Transform::default();
            xform.position.z = 1.;
            xform.scale /= 20.;
            xform.into()
        });
        meshes.insert(quad_id, MeshID::from_raw(0));

        let slf = Self {
            entity_id_domain,
            meshes, transforms, cameras,
            allows_quitting: true,
            clear_color: Rgb::cyan(),
        };
        slf.debug_entity_id(camera_id);
        slf.debug_entity_id(quad_id);
        slf
    }
}


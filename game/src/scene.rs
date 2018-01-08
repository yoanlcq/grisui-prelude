use std::ops::{Index, IndexMut};
use ids::*;
use v::{Rgb, Vec2, Extent2, Rect, Lerp, Mat4, Vec3, Rgba};
use transform::Transform3D;
use sdl2::event::{Event, WindowEvent};
use camera::OrthographicCamera;
use gl;
use global::{Global, GlobalDataUpdatePack};
use duration_ext::DurationExt;
use grx;
use fonts::{Font, FontName};

#[derive(Debug)]
pub struct GUIText {
    /// If the entity has a Transform component, then this member is
    /// a screen-space offset for the text.
    /// If it doesn't, this member is the absolute position of the text
    /// in screen space.
    pub screen_space_offset: Vec2<i32>,
    pub text: String,
    pub font: FontName,
    pub color: Rgba<f32>,
    pub shadow_hack: Option<Rgba<f32>>,
}

#[derive(Debug)]
pub struct Scene {
    pub allows_quitting: bool,
    pub clear_color: Rgb<u8>,
    pub entity_id_domain: EntityIDDomain,
    pub transforms: EntityIDMap<SimStates<Transform3D>>,
    pub cameras: EntityIDMap<SimStates<OrthographicCamera>>,
    pub meshes: EntityIDMap<MeshID>,
    pub texts: EntityIDMap<GUIText>,
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
    pub fn reshape(&mut self, window_size: Extent2<u32>) {
        for camera in self.cameras.values_mut() {
            // NOTE: Every camera might want to handle this differently
            let vp = Rect::from((Vec2::zero(), window_size));
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
        let g = &frame.g;
        for (camera_eid, camera) in self.cameras.iter().map(|(id, c)| (id, &c.render)) {
            let camera_xform = &self.transforms[camera_eid].render;
            let view = camera.view_matrix(camera_xform);
            let proj = camera.proj_matrix();
            let Rect { x, y, w, h } = camera.viewport;
            unsafe {
                gl::Viewport(x as _, y as _, w as _, h as _);
            }

            // Render all meshes

            g.gl_simple_color_program.use_program(&Mat4::identity());
            for (mesh_eid, mesh_id) in self.meshes.iter() {
                let mesh = g.meshes[*mesh_id].as_ref().unwrap();
                let mesh_xform = self.transforms[mesh_eid].render;
                let model = Mat4::from(mesh_xform);
                let mvp = proj * view * model;
                g.gl_simple_color_program.set_uniform_mvp(&mvp);
                mesh.vao.bind();
                unsafe {
                    gl::DrawArrays(mesh.gl_topology, 0, mesh.vertices.len() as _);
                }
            }

            // Render text overlay

            unsafe {
                gl::Disable(gl::DEPTH_TEST);
            }
            let prog = &g.gl_text_program;
            let mesh = &g.font_atlas_mesh;
            prog.use_program();
            mesh.vao.bind();
            let _render_font_atlas = |font: &Font, texunit: grx::TextureUnit, color: Rgba<f32>| {
                let vp = g.window_size.map(|x| x as f32);
                let atlas_size = font.texture_size.map(|x| x as f32);
                let model = Mat4::scaling_3d(2. * atlas_size.w/vp.w);
                let mvp = proj * view * model;
                prog.set_uniform_texture(texunit);
                prog.set_uniform_color(color);
                prog.set_uniform_mvp(&mvp);
                prog.set_uniform_glyph_rect_pos(Vec2::zero());
                prog.set_uniform_glyph_rect_size(Extent2::one());
                prog.set_uniform_glyph_offset(Vec2::zero());
                unsafe {
                    gl::DrawArrays(mesh.gl_topology, 0, mesh.vertices.len() as _);
                }
            };
            let render_some_text = |ss_pos: Vec2<i32>, text: &str, font: &Font, texunit: grx::TextureUnit, color: Rgba<f32>| {
                let vp = g.window_size.map(|x| x as f32);
                let atlas_size = font.texture_size.map(|x| x as f32);
                prog.set_uniform_texture(texunit);
                prog.set_uniform_color(color);
                let world_start = Self::window_to_world(ss_pos, g, camera, camera_xform);
                let mut adv = Vec2::<i16>::zero();

                for c in text.chars() {
                    match c {
                        '\n' => {
                            adv.x = 0;
                            adv.y += font.height as i16;
                            continue;
                        },
                        ' ' => {
                            adv += font.glyph_info[&' '].advance;
                            continue;
                        },
                        '\t' => {
                            adv += font.glyph_info[&' '].advance * 4;
                            continue;
                        },
                        c if c.is_ascii_control() || c.is_ascii_whitespace() => {
                            continue;
                        },
                        _ => (),
                    };
                    let c = if font.glyph_info.contains_key(&c) { c } else { '?' };
                    let glyph = &font.glyph_info[&c];
                    let mut rect = glyph.bounds.into_rect().map(
                        |p| p as f32,
                        |e| e as f32
                    );
                    rect.x /= atlas_size.w;
                    rect.y /= atlas_size.h;
                    rect.w /= atlas_size.w;
                    rect.h /= atlas_size.h;
                    let offset = glyph.offset.map(|x| x as f32) / Vec2::from(atlas_size);
                    prog.set_uniform_glyph_rect_pos(rect.position());
                    prog.set_uniform_glyph_rect_size(rect.extent());
                    prog.set_uniform_glyph_offset(offset);
                    let mut world_adv = adv.map(|x| x as f32) * 2. / vp.w;
                    world_adv.y = -world_adv.y;
                    let model = Mat4::scaling_3d(2. * atlas_size.w/vp.w)
                        .translated_3d(world_start + world_adv);
                    let mvp = proj * view * model;
                    prog.set_uniform_mvp(&mvp);
                    unsafe {
                        gl::DrawArrays(mesh.gl_topology, 0, mesh.vertices.len() as _);
                    }
                    adv += glyph.advance;
                }
            };

            for (text_eid, text) in self.texts.iter() {
                let &GUIText {
                    ref screen_space_offset, ref text, ref font, ref color,
                    ref shadow_hack,
                } = text;
                let mut ss_pos = *screen_space_offset;
                if let Some(xform) = self.transforms.get(text_eid) {
                    ss_pos += Self::world_to_window(xform.render.position, g, camera, camera_xform);
                }
                let texunit = grx::TextureUnit::from(*font);
                let font = &g.fonts.fonts[font];
                if let &Some(ref color) = shadow_hack {
                    let ss_pos = ss_pos + 1;
                    render_some_text(ss_pos, text, font, texunit, *color);
                }
                render_some_text(ss_pos, text, font, texunit, *color);
            }
            let fontname = FontName::Debug;
            let font = &g.fonts.fonts[&fontname];
            let texunit = grx::TextureUnit::from(fontname);
            let text = format!("{}\n{}", g.input.mouse.position, Vec2::<f32>::from(Self::mouse_world_pos(g, camera, camera_xform)));
            render_some_text(g.input.mouse.position.map(|x| x as i32), &text, font, texunit, Rgba::black());

            /*
            let text = "This is some SAMPLE TEXT!!1!11\n\t(Glad that it works.) 0123456789@$";
            for (fontname, font) in g.fonts.fonts.iter() {
                let texunit = grx::TextureUnit::from(*fontname);
                render_font_atlas(font, texunit, Rgba::red());
                render_some_text(text, font, texunit, Rgba::blue());
            }
            */
            unsafe {
                gl::Enable(gl::DEPTH_TEST);
            }
        }
    }

    pub fn world_to_window(p: Vec3<f32>, g: &Global, camera: &OrthographicCamera, camera_xform: &Transform3D) -> Vec2<i32> {
        let mut p = camera.world_to_viewport_point(camera_xform, p).map(|p| p.round() as i32);
        p.y = g.window_size.h as i32 - p.y;
        p.into()
    }

    pub fn window_to_world(p: Vec2<i32>, g: &Global, camera: &OrthographicCamera, camera_xform: &Transform3D) -> Vec3<f32> {
        let mut p = p.map(|p| p as f32);
        p.y = g.window_size.h as f32 - p.y;
        camera.viewport_to_world_point(camera_xform, Vec3::from(p))
    }

    pub fn mouse_world_pos(g: &Global, camera: &OrthographicCamera, camera_xform: &Transform3D) -> Vec3<f32> {
        Self::window_to_world(g.input.mouse.position.map(|x| x as i32), g, camera, camera_xform)
    }

    pub fn new_test_room(viewport: Rect<u32, u32>) -> Self {
        let mut entity_id_domain = EntityIDDomain::new_empty();
        let hasher_builder = EntityIDHasherBuilder::default();
        let mut meshes = EntityIDMap::with_capacity_and_hasher(1, hasher_builder);
        let mut transforms = EntityIDMap::with_capacity_and_hasher(2, hasher_builder);
        let mut cameras = EntityIDMap::with_capacity_and_hasher(1, hasher_builder);
        let mut texts = EntityIDMap::with_capacity_and_hasher(1, hasher_builder);

        let camera_id = EntityID::from_raw(0);
        let quad_id = EntityID::from_raw(1);
        let inspector_id = EntityID::from_raw(2);
        entity_id_domain.include_id(camera_id);
        entity_id_domain.include_id(quad_id);
        entity_id_domain.include_id(inspector_id);

        let near = 0.01_f32;
        transforms.insert(camera_id, Transform3D {
            position: Vec3::back_lh() * (near + 0.001_f32),
            .. Default::default()
        }.into());
        cameras.insert(camera_id, OrthographicCamera {
            viewport, ortho_right: 1., near, far: 100.,
        }.into());

        transforms.insert(quad_id, {
            let mut xform = Transform3D::default();
            xform.position.z = 1.;
            xform.scale /= 20.;
            xform.into()
        });
        meshes.insert(quad_id, MeshID::from_raw(0));

        texts.insert(inspector_id, GUIText {
            screen_space_offset: Vec2::new(0, 16),
            text: "Hello! I'm Foo!!".to_string(),
            font: FontName::Debug,
            color: Rgba::blue(),
            shadow_hack: Some(Rgba::red()),
        });

        let slf = Self {
            entity_id_domain,
            meshes, transforms, cameras, texts,
            allows_quitting: true,
            clear_color: Rgb::cyan(),
        };
        slf.debug_entity_id(camera_id);
        slf.debug_entity_id(quad_id);
        slf
    }
}


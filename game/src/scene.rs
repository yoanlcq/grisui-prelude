use std::ops::{Index, IndexMut};
use ids::*;
use v::{Rgb, Vec2, Extent2, Rect, Lerp, Mat4, Vec3, Rgba};
use transform::Transform3D;
use camera::OrthographicCamera;
use gl;
use global::{Global, GlobalDataUpdatePack};
use duration_ext::DurationExt;
use gx;
use grx;
use fonts::{Font, FontName};
use mesh::Mesh;
use events::{Sdl2EventSubscriber, KeyInput, MouseButtonInput};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum HackMode {
    RenderAllMeshes,
    Intersection,
    Masking,
    TwoStars,
}

#[derive(Debug)]
pub struct Scene {
    pub wants_to_quit: bool,
    pub allows_quitting: bool,
    pub clear_color: Rgb<u8>,
    pub entity_id_domain: EntityIDDomain,
    pub names: EntityIDMap<String>,
    pub transforms: EntityIDMap<SimStates<Transform3D>>,
    pub cameras: EntityIDMap<SimStates<OrthographicCamera>>,
    pub meshes: EntityIDMap<MeshID>,
    pub texts: EntityIDMap<GUIText>,
    pub pathshapes: EntityIDMap<PathShape>,
}

#[derive(Debug)]
pub struct PathShape {
    /// Triangle fan which vertices contain the origin and each vertex
    /// of a closed polygon.
    /// Rendering it directly would result in a mess. Render it in the stencil
    /// buffer instead with GL_INVERT in order to render a mask for the polygon.
    pub polyfanmask_mesh: Mesh,
    /// Simple screen-space quad mesh that should be drawn over the polygon mesh mask.
    pub fill_color_quad: Mesh,
    /// Set of local-space gradient strips.
    pub fill_gradient_strips: Vec<Mesh>,
}

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

/* NOTE: Stub for later
impl GUIText {
    pub fn text(&self) -> &str { &self.text }
    pub fn edit_text<F>(&mut self, f: F) where F: FnMut<&mut String> {
        f(&mut self.text);
        unimplemented!{} // TODO: recompute extents
    }
    pub fn extent(&self) -> Extent2<u32> { unimplemented!{} }
}
*/

impl Sdl2EventSubscriber for Scene {
    fn on_wants_to_quit(&mut self) {
        self.wants_to_quit = true;
    }
    fn on_text_input(&mut self, _text: &str) {}
    fn on_key(&mut self, _key: KeyInput) {}
    fn on_scroll(&mut self, _delta: Vec2<i32>) {}
    fn on_mouse_motion(&mut self, _pos: Vec2<i32>) {}
    fn on_mouse_button(&mut self, _btn: MouseButtonInput) {}
    fn on_window_resized(&mut self, size: Extent2<u32>) {
        self.reshape(size);
    }
    fn on_window_size_changed(&mut self, size: Extent2<u32>) {
        self.reshape(size);
    }
}

impl Scene {
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
        let _t = tick.t.to_f64_seconds() as f32;
        let xform = &mut self.transforms.get_mut(&EntityID::from_raw(1)).unwrap().current;
        xform.orientation.rotate_z(PI * dt / 4.);
    }
    pub fn prepare_render_state_via_lerp_previous_current(&mut self, alpha: f64) {
        let alpha = alpha as f32;
        for xform in self.transforms.values_mut() {
            xform.render = Lerp::lerp(xform.previous, xform.current, alpha);
        }
    }
    pub fn debug_entity_id(&self, eid: EntityID) {
        let &Self {
            allows_quitting: _, wants_to_quit: _, clear_color: _, entity_id_domain: _,
            ref names, ref transforms, ref cameras, ref meshes, ref texts,
            ref pathshapes,
        } = self;
        let head = format!("Components of {:?}", eid);
        let mut cpts = String::new();
        if let Some(x) = names.get(&eid) { cpts += &format!(" ({:?})", x); }
        if let Some(x) = transforms.get(&eid) { cpts += &format!("\n- {:?}", x.current); }
        if let Some(x) = cameras.get(&eid) { cpts += &format!("\n- {:?}", x.current); }
        if let Some(x) = meshes.get(&eid) { cpts += &format!("\n- {:?}", x); }
        if let Some(x) = texts.get(&eid) { cpts += &format!("\n- {:#?}", x); }
        if let Some(x) = pathshapes.get(&eid) { cpts += &format!("\n- {:#?}", x); }
        info!("{}{}", &head, if cpts.is_empty() { " None at all!" } else { &cpts });
    }
    pub fn render(&mut self, mut frame: GlobalDataUpdatePack) {
        let g = &mut frame.g;

        // Update stats text

        {
            let inspector_id = EntityID::from_raw(2);
            let gui_text = self.texts.get_mut(&inspector_id).unwrap();
            gui_text.text = format!("{:?}", g.fps_stats);
        }


        for (camera_eid, camera) in self.cameras.iter().map(|(id, c)| (id, &c.render)) {
            let camera_xform = &self.transforms[camera_eid].render;
            let view = camera.view_matrix(camera_xform);
            let proj = camera.proj_matrix();
            let Rect { x, y, w, h } = camera.viewport;
            unsafe {
                gl::Viewport(x as _, y as _, w as _, h as _);
            }

            g.gl_simple_color_program.use_program(&Mat4::identity());

            let render_mesh_mvp = |mesh: &Mesh, mvp: &Mat4<f32>| {
                g.gl_simple_color_program.set_uniform_mvp(&mvp);
                mesh.vao.bind();
                unsafe {
                    gl::DrawArrays(mesh.gl_topology, 0, mesh.vertices.len() as _);
                }
            };
            let render_mesh = |mesh_eid: &EntityID, mesh: &Mesh| {
                let mesh_xform = self.transforms[mesh_eid].render;
                let model = Mat4::from(mesh_xform);
                let mvp = proj * view * model;
                render_mesh_mvp(mesh, &mvp);
            };
            let render_mesh_id = |mesh_eid: &EntityID, mesh_id: &MeshID| {
                let mesh = g.meshes[*mesh_id].as_ref().unwrap();
                render_mesh(mesh_eid, mesh);
            };
            let render_eid_mesh = |mesh_eid: &EntityID| {
                let mesh_id = &self.meshes[mesh_eid];
                render_mesh_id(mesh_eid, mesh_id);
            };

            // Perform some stencil tricks

            let lucky_quad_eid = &EntityID::from_raw(1);
            let reddisk_eid = &EntityID::from_raw(3);
            let bluedisk_eid = &EntityID::from_raw(4);
            let redshape_eid = &EntityID::from_raw(5);
            let blueshape_eid = &EntityID::from_raw(6);

            match g.hack_mode {
                HackMode::Masking => {
                    // --- Masking with lucky quad
                    unsafe {
                        gl::Enable(gl::STENCIL_TEST);
                        gl::ClearStencil(0x0); // Set clear value
                        gl::Clear(gl::STENCIL_BUFFER_BIT);

                        gl::StencilFunc(gl::ALWAYS, 0x1, 0x1);
                        gl::StencilOp(gl::REPLACE, gl::REPLACE, gl::REPLACE);
                        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
                        gl::DepthMask(gl::FALSE);
                    }
                    render_eid_mesh(lucky_quad_eid);
                    unsafe {
                        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                        gl::DepthMask(gl::TRUE);
                        gl::StencilFunc(gl::EQUAL, 0x1, 0x1);
                        gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
                    }
                    render_eid_mesh(reddisk_eid);
                    render_eid_mesh(bluedisk_eid);

                    unsafe {
                        gl::Disable(gl::STENCIL_TEST);
                        gl::Disable(gl::DEPTH_TEST);
                    }
                },
                HackMode::TwoStars => {
                    // --- Rendering two stars with fill and gradient
                    let render_shape = |eid: &EntityID| unsafe {
                        let pshape = &self.pathshapes[eid];
                        gl::Enable(gl::STENCIL_TEST);
                        gl::ClearStencil(0x0); // Set clear value
                        gl::Clear(gl::STENCIL_BUFFER_BIT);
                        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
                        gl::DepthMask(gl::FALSE);
                        gl::StencilFunc(gl::ALWAYS, 0, 1);
                        gl::StencilOp(gl::KEEP, gl::KEEP, gl::INVERT);
                        gl::StencilMask(1);
                        render_mesh(eid, &pshape.polyfanmask_mesh);
                        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                        gl::DepthMask(gl::TRUE);
                        gl::StencilFunc(gl::EQUAL, 1, 1);
                        gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
                        render_mesh_mvp(&pshape.fill_color_quad, &Mat4::identity());
                        for m in &pshape.fill_gradient_strips {
                            render_mesh(eid, m);
                        }
                        gl::Disable(gl::STENCIL_TEST);
                    };
                    render_shape(redshape_eid);
                    render_shape(blueshape_eid);
                },
                HackMode::Intersection => {
                    // --- Experimenting with shape intersection
                    unsafe {
                        let red_pshape = &self.pathshapes[redshape_eid];
                        let blu_pshape = &self.pathshapes[blueshape_eid];

                        gl::Disable(gl::DEPTH_TEST);
                        gl::Enable(gl::STENCIL_TEST);
                        gl::ClearStencil(0x0); // Set clear value
                        gl::Clear(gl::STENCIL_BUFFER_BIT);
                        gl::StencilMask(1);
                        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
                        gl::DepthMask(gl::FALSE);

                        // Fill B in stencil
                        gl::StencilFunc(gl::ALWAYS, 0, 1);
                        gl::StencilOp(gl::KEEP, gl::KEEP, gl::INVERT);
                        render_mesh(blueshape_eid, &blu_pshape.polyfanmask_mesh);

                        // Subtract A
                        gl::StencilFunc(gl::EQUAL, 1, 1);
                        gl::StencilOp(gl::KEEP, gl::KEEP, gl::INVERT);
                        render_mesh(redshape_eid, &red_pshape.polyfanmask_mesh);

                        // Fill B again, we get the intersection of A and B.
                        gl::StencilFunc(gl::ALWAYS, 0, 1);
                        gl::StencilOp(gl::KEEP, gl::KEEP, gl::INVERT);
                        render_mesh(blueshape_eid, &blu_pshape.polyfanmask_mesh);

                        // Fill intersection with solid color
                        gl::Disable(gl::DEPTH_TEST);
                        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                        gl::DepthMask(gl::TRUE);
                        gl::StencilFunc(gl::EQUAL, 1, 1);
                        gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
                        render_mesh_mvp(&red_pshape.fill_color_quad, &Mat4::identity());

                        gl::Disable(gl::STENCIL_TEST);
                    }
                },
                HackMode::RenderAllMeshes => {
                    unsafe {
                        gl::Enable(gl::DEPTH_TEST);
                    }

                    // Render all meshes
                    for (mesh_eid, mesh_id) in self.meshes.iter() {
                        if mesh_eid == lucky_quad_eid {
                            continue; // XXX Hack for stencil
                        }
                        render_mesh_id(mesh_eid, mesh_id);
                    }
                },
            };

            // Render text overlay

            if g.should_render_debug_text {

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
                        let mut ss_pos = ss_pos;
                        // PERF: This is a horrible way to do 1px text contour!
                        ss_pos.x += 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        ss_pos.y += 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        /* Comment, otherwise we lose 10 FPS
                        ss_pos.x -= 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        ss_pos.x -= 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        ss_pos.y += 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        ss_pos.y += 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        ss_pos.x += 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        ss_pos.x += 1;
                        render_some_text(ss_pos, text, font, texunit, *color);
                        */
                    }
                    render_some_text(ss_pos, text, font, texunit, *color);
                }

                // Render text near mouse pointer

                let fontname = FontName::Debug;
                let font = &g.fonts.fonts[&fontname];
                let texunit = grx::TextureUnit::from(fontname);
                let text = format!("{}\n{}",
                    g.mouse_position,
                    Vec2::<f32>::from(Self::mouse_world_pos(g, camera, camera_xform))
                );
                let mpos = g.mouse_position.map(|x| x as i32);
                {
                    let mut mpos = mpos;
                    mpos.x += 1;
                    render_some_text(mpos, &text, font, texunit, Rgba::black());
                    mpos.y += 1;
                    render_some_text(mpos, &text, font, texunit, Rgba::black());
                }
                render_some_text(mpos, &text, font, texunit, Rgba::white());

            }

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
        Self::window_to_world(g.mouse_position.map(|x| x as i32), g, camera, camera_xform)
    }

    pub fn new_test_room(g: &Global) -> Self {
        let viewport = Rect::from((Vec2::zero(), g.window_size));
        let gl_simple_color_program = &g.gl_simple_color_program;

        let mut entity_id_domain = EntityIDDomain::new_empty();
        let hasher_builder = EntityIDHasherBuilder::default();
        let mut names = EntityIDMap::with_capacity_and_hasher(5, hasher_builder);
        let mut transforms = EntityIDMap::with_capacity_and_hasher(5, hasher_builder);
        let mut cameras = EntityIDMap::with_capacity_and_hasher(1, hasher_builder);
        let mut meshes = EntityIDMap::with_capacity_and_hasher(3, hasher_builder);
        let mut texts = EntityIDMap::with_capacity_and_hasher(1, hasher_builder);
        let mut pathshapes = EntityIDMap::with_capacity_and_hasher(2, hasher_builder);

        let camera_id = EntityID::from_raw(0);
        let quad_id = EntityID::from_raw(1);
        let inspector_id = EntityID::from_raw(2);
        let reddisk_id = EntityID::from_raw(3);
        let bluedisk_id = EntityID::from_raw(4);
        let redshape_id = EntityID::from_raw(5);
        let blueshape_id = EntityID::from_raw(6);
        entity_id_domain.include_id(camera_id);
        entity_id_domain.include_id(quad_id);
        entity_id_domain.include_id(inspector_id);
        entity_id_domain.include_id(reddisk_id);
        entity_id_domain.include_id(bluedisk_id);
        entity_id_domain.include_id(redshape_id);
        entity_id_domain.include_id(blueshape_id);

        names.insert(camera_id, "Main Camera".to_owned());
        let near = 0.01_f32;
        transforms.insert(camera_id, Transform3D {
            position: Vec3::back_lh() * (near + 0.001_f32),
            .. Default::default()
        }.into());
        cameras.insert(camera_id, OrthographicCamera {
            viewport, ortho_right: 1., near, far: 100.,
        }.into());

        names.insert(quad_id, "Lucky Quad".to_owned());
        transforms.insert(quad_id, {
            let mut xform = Transform3D::default();
            xform.position.z = 1.;
            xform.scale *= 0.75;
            xform.into()
        });
        meshes.insert(quad_id, MeshID::from_raw(0));

        names.insert(inspector_id, "Inspector".to_owned());
        texts.insert(inspector_id, GUIText {
            screen_space_offset: Vec2::new(0, 16),
            text: "If the universe is infinite,\nthen there is an infinite quantity of worlds\nwhere this story is happening.".to_owned(),
            font: FontName::Debug,
            color: Rgba::white(),
            shadow_hack: Some(Rgba::grey(1./6.)),
        });

        names.insert(reddisk_id, "Red Disk".to_owned());
        transforms.insert(reddisk_id, {
            let mut xform = Transform3D::default();
            xform.position.z = 1.;
            xform.position.x = -0.25;
            xform.scale /= 2.;
            xform.into()
        });
        meshes.insert(reddisk_id, MeshID::from_raw(1));

        names.insert(bluedisk_id, "Blue Disk".to_owned());
        transforms.insert(bluedisk_id, {
            let mut xform = Transform3D::default();
            xform.position.z = 1.;
            xform.position.x = 0.25;
            xform.scale /= 2.;
            xform.into()
        });
        meshes.insert(bluedisk_id, MeshID::from_raw(2));

        names.insert(redshape_id, "Red Shape".to_owned());
        transforms.insert(redshape_id, {
            let mut xform = Transform3D::default();
            xform.position.z = 1.;
            xform.position.x = -0.25;
            xform.position.y = -0.5;
            xform.scale /= 2.;
            xform.into()
        });
        pathshapes.insert(redshape_id, PathShape {
            polyfanmask_mesh: Mesh::new_star_polyfanmask(
                &gl_simple_color_program, "Red Shape PolyFanMask", gx::UpdateHint::Occasionally
            ),
            fill_color_quad: Mesh::new_filled_quad(
                &gl_simple_color_program, "Red Shape Fill", gx::UpdateHint::Occasionally, Rgba::red(), 1.
            ),
            fill_gradient_strips: vec![
                Mesh::new_gradient_strip(
                    &gl_simple_color_program, "Red Shape Fill Gradient", gx::UpdateHint::Occasionally,
                    (Vec3::new(0.5, -0.5, 0.), Rgba::zero()), 
                    (Vec3::new(-0.5, 0.5, 0.), Rgba::black())
                )
            ],
        });

        names.insert(blueshape_id, "Blue Shape".to_owned());
        transforms.insert(blueshape_id, {
            let mut xform = Transform3D::default();
            xform.position.z = 1.;
            xform.position.x = 0.; //0.25;
            xform.position.y = -0.5;
            //xform.scale *= 2.;
            xform.into()
        });
        pathshapes.insert(blueshape_id, PathShape {
            polyfanmask_mesh: Mesh::new_star_polyfanmask(
                &gl_simple_color_program, "Blue Shape PolyFanMask", gx::UpdateHint::Occasionally
            ),
            fill_color_quad: Mesh::new_filled_quad(
                &gl_simple_color_program, "Blue Shape Fill", gx::UpdateHint::Occasionally, Rgba::blue(), 1.
            ),
            fill_gradient_strips: vec![
                Mesh::new_gradient_strip(
                    &gl_simple_color_program, "Red Shape Fill Gradient", gx::UpdateHint::Occasionally,
                    (Vec3::unit_x()/4.-0.01, Rgba::green()), 
                    (Vec3::unit_x()/4., Rgba::blue())
                )
            ],
        });

        let slf = Self {
            entity_id_domain,
            names, transforms, cameras, meshes, texts, pathshapes,
            wants_to_quit: false,
            allows_quitting: true,
            clear_color: Rgb::cyan(),
        };
        slf.debug_entity_id(camera_id);
        slf.debug_entity_id(quad_id);
        slf.debug_entity_id(inspector_id);
        slf.debug_entity_id(reddisk_id);
        slf.debug_entity_id(bluedisk_id);
        slf.debug_entity_id(redshape_id);
        slf.debug_entity_id(blueshape_id);
        slf
    }
}


use std::ptr;
use gl;
use gx::{Object, BufferUsage};
use system::*;
use v::{Vec3, Rgba, Mat4};
use camera::OrthoCamera2D;
use mesh::{self, vertex_array, color_mesh::{self, Vertex}};
use duration_ext::DurationExt;
use text::Text;
use font::FontID;

type ColorVertexArray = vertex_array::VertexArray<color_mesh::Program>;

pub struct EditorSystem {
    camera: OrthoCamera2D,
    grid_origin_vertices: ColorVertexArray,
    grid_vertices_1: ColorVertexArray,
    grid_vertices_01: ColorVertexArray,
    cursor_vertices: ColorVertexArray,
    draw_grid_first: bool,
    do_draw_grid: bool,
    is_panning_camera: bool,
    camera_rotation_speed: f32,
    prev_camera_rotation_z_radians: f32,
    next_camera_rotation_z_radians: f32,
    is_active: bool,
    primary_color: Rgba<f32>,
    draft_vertices: ColorVertexArray,
    draft_vertices_ended: bool,
    text: Text,
    text_position: Vec2<i32>,
    text_color: Rgba<f32>,
}

fn create_grid_vertices(color_mesh_gl_program: &color_mesh::Program, size: Extent2<usize>, color: Rgba<f32>, scale: Extent2<f32>) -> ColorVertexArray {
    let (w, h) = size.map(|x| x as isize).into_tuple();
    let mut vertices = Vec::with_capacity((w * h) as usize);
    for y in (-h) .. (1 + h) {
        if y == 0 {
            let color = Rgba::black();
            vertices.push(Vertex { position: Vec3::new(-w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new(                 0., y as f32 * scale.h, 0.), color, });
            let color = Rgba::red();
            vertices.push(Vertex { position: Vec3::new(                 0., y as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new( w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
        } else {
            vertices.push(Vertex { position: Vec3::new(-w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new( w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
        }
    }
    for x in (-w) .. (1 + w) {
        if x == 0 {
            let color = Rgba::new(0., 0.6, 0., 1.);
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w, -h as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,                  0., 0.), color, });
            let color = Rgba::green();
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,                  0., 0.), color, });
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,  h as f32 * scale.h, 0.), color, });
        } else {
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w, -h as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,  h as f32 * scale.h, 0.), color, });
        }
    }
    ColorVertexArray::from_vertices(&color_mesh_gl_program, "Grid Vertices", BufferUsage::StaticDraw, vertices)
}

impl EditorSystem {
    const CAMERA_ZOOM_STEP_FACTOR: f32 = 1.1;
    const CAMERA_Z_ROTATION_SPEED_DEGREES: f32 = 90.;
    const CAMERA_NEAR: f32 = 0.; // It does work for an orthographic camera.
    const CAMERA_FAR: f32 = 1024.;

    pub fn new(color_mesh_gl_program: &color_mesh::Program, text_gl_program: &mesh::text::Program, viewport_size: Extent2<u32>) -> Self {
        let grid_vertices_1 = create_grid_vertices(color_mesh_gl_program, Extent2::new(8, 8), Rgba::white(), Extent2::one());
        let grid_vertices_01 = create_grid_vertices(color_mesh_gl_program, Extent2::new(64, 64), Rgba::new(1., 1., 1., 0.2), Extent2::one()/10.);
        let grid_origin_vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Grid Origin Vertices", BufferUsage::StaticDraw,
            vec![Vertex { position: Vec3::zero(), color: Rgba::red(), }]
        );
        let cursor_vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Cursor Vertices", BufferUsage::StaticDraw,
            vec![
                Vertex { position: Vec3::zero(), color: Rgba::red(), },
                Vertex { position: Vec3::unit_x(), color: Rgba::green(), },
                Vertex { position: Vec3::unit_y(), color: Rgba::blue(), },
            ]
        );
        let draft_vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Draft Vertices", BufferUsage::DynamicDraw, vec![]
        );
        let text = Text::new(text_gl_program, "Editor Text");
        let camera = OrthoCamera2D::new(viewport_size, Self::CAMERA_NEAR, Self::CAMERA_FAR);
        Self {
            camera, cursor_vertices, grid_origin_vertices, grid_vertices_1, grid_vertices_01,
            draft_vertices,
            draft_vertices_ended: false,
            primary_color: Rgba::red(),
            draw_grid_first: true,
            do_draw_grid: true,
            is_panning_camera: false,
            camera_rotation_speed: 0.,
            prev_camera_rotation_z_radians: 0.,
            next_camera_rotation_z_radians: 0.,
            is_active: false,
            text,
            text_position: (viewport_size.map(|x| x as i32) / 2).into(),
            text_color: Rgba::black(),
        }
    }
    fn on_enter_editor(&mut self, g: &Game) {
        debug_assert!(!self.is_active);
        self.is_active = true;
        unsafe {
            gl::ClearColor(0.1, 0.2, 1., 1.);
        }
        g.platform.cursors.crosshair.set();
        self.text.string = "If the universe is infinite,\nthere is an infinite number of worlds\nwhere this story takes place.".to_owned();
        self.text.update_gl(&g.fonts.fonts[&FontID::Debug]);
    }
    fn on_leave_editor(&mut self, g: &Game) {
        debug_assert!(self.is_active);
        self.is_active = false;
        unsafe {
            gl::ClearColor(1., 1., 1., 1.);
        }
        g.platform.cursors.normal.set();
    }

    fn add_vertex_at_current_mouse_position(&mut self, g: &Game) {
        debug_assert!(self.is_active);
        if self.draft_vertices_ended {
            return;
        }
        if let Some(pos) = g.input.mouse_position() {
            let color = self.primary_color;
            let mut position = self.camera.viewport_to_world(pos, 0.);
            // position.z = 0.;
            self.draft_vertices.vertices.push(Vertex { position, color, });
            self.draft_vertices.update_vbo();
        }
    }
    fn end_polygon(&mut self, _g: &Game) {
        debug_assert!(self.is_active);
        self.draft_vertices_ended = true;
    }
    fn toggle_select_all(&mut self, _g: &Game) {
        debug_assert!(self.is_active);
        unimplemented!{}
    }
    fn deleted_selected(&mut self, _g: &Game) {
        debug_assert!(self.is_active);
        self.draft_vertices.vertices.clear();
        self.draft_vertices.update_vbo();
        self.draft_vertices_ended = false;
    }
}

impl System for EditorSystem {
    fn name(&self) -> &str {
        "EditorSystem"
    }
    fn on_canvas_resized(&mut self, _: &Game, size: Extent2<u32>, _by_user: bool) {
        self.camera.set_viewport_size(size);
    }
    fn on_mouse_motion(&mut self, g: &Game, pos: Vec2<i32>) {
        if !self.is_active {
            return;
        }
        if let Some(prev) = g.input.previous_mouse_position() {
            if self.is_panning_camera {
                let o = self.camera.viewport_to_world(prev, 0.);
                let p = self.camera.viewport_to_world(pos, 0.);
                self.camera.xform.position -= (p - o) * self.camera.xform.scale.x;
                self.camera.xform.position.z = 0.;
            }
        }
    }
    fn on_mouse_scroll(&mut self, _: &Game, delta: Vec2<i32>) {
        if !self.is_active {
            return;
        }
        self.camera.xform.scale *= Self::CAMERA_ZOOM_STEP_FACTOR.powf(delta.y as _);
    }
    fn on_message(&mut self, g: &Game, msg: &Message) {
        match *msg {
            Message::EnterEditor => { self.on_enter_editor(g); return; },
            Message::LeaveEditor => { self.on_leave_editor(g); return; },
            _ => (),
        };

        if !self.is_active {
            return;
        }

        let normal_camera_rotation_speed = Self::CAMERA_Z_ROTATION_SPEED_DEGREES.to_radians();

        match *msg {
            Message::EditorToggleDrawGridFirst => self.draw_grid_first = !self.draw_grid_first,
            Message::EditorToggleGrid => self.do_draw_grid = !self.do_draw_grid,
            Message::EditorBeginPanCameraViaMouse => self.is_panning_camera = true,
            Message::EditorEndPanCameraViaMouse => self.is_panning_camera = false,
            Message::EditorBeginRotateCameraLeft => self.camera_rotation_speed = normal_camera_rotation_speed,
            Message::EditorBeginRotateCameraRight => self.camera_rotation_speed = -normal_camera_rotation_speed,
            Message::EditorEndRotateCamera => self.camera_rotation_speed = 0.,
            Message::EditorRecenterCamera => self.camera.xform.position = Vec3::zero(),
            Message::EditorResetCameraRotation => {
                self.camera.xform.rotation_z_radians = 0.;
                self.prev_camera_rotation_z_radians = 0.;
                self.next_camera_rotation_z_radians = 0.;
            },
            Message::EditorResetCameraZoom => self.camera.xform.scale = Vec2::one(),
            Message::EditorAddVertexAtCurrentMousePosition => self.add_vertex_at_current_mouse_position(g),
            Message::EditorEndPolygon => self.end_polygon(g),
            Message::EditorToggleSelectAll => self.toggle_select_all(g),
            Message::EditorDeleteSelected => self.deleted_selected(g),
            _ => (),
        };
    }
    fn tick(&mut self, _: &Game, _: Duration, dt: Duration) {
        if !self.is_active {
            return;
        }
        let dt = dt.to_f64_seconds() as f32;
        self.prev_camera_rotation_z_radians = self.next_camera_rotation_z_radians;
        self.next_camera_rotation_z_radians += dt * self.camera_rotation_speed;
    }
    fn draw(&mut self, g: &Game, gfx_interp: f64) {
        if !self.is_active {
            return;
        }
        self.camera.xform.rotation_z_radians = ::v::Lerp::lerp(self.prev_camera_rotation_z_radians, self.next_camera_rotation_z_radians, gfx_interp as f32);
        unsafe {
            let draw_cursor = || if let Some(pos) = g.input.mouse_position() {
                let mvp = {
                    let w = self.camera.viewport_to_world(pos, 0.);
                    self.camera.view_proj_matrix() * Mat4::translation_3d(w)
                };
                g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                gl::PointSize(8.);
                gl::BindVertexArray(self.cursor_vertices.vao().gl_id());
                gl::DrawArrays(gl::POINTS, 0, self.cursor_vertices.vertices.len() as _);
                gl::DrawArrays(gl::TRIANGLES, 0, self.cursor_vertices.vertices.len() as _);
            };

            let draw_draft_vertices = || {
                let mvp = self.camera.view_proj_matrix();
                g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                gl::PointSize(8.);
                gl::LineWidth(8.);
                gl::BindVertexArray(self.draft_vertices.vao().gl_id());
                gl::DrawArrays(gl::POINTS, 0, self.draft_vertices.vertices.len() as _);
                let topology = if self.draft_vertices_ended { gl::LINE_LOOP } else { gl::LINE_STRIP };
                gl::DrawArrays(topology, 0, self.draft_vertices.vertices.len() as _);
            };

            let draw_grid = || {
                if self.do_draw_grid {
                    gl::Disable(gl::DEPTH_TEST);
                    gl::DepthMask(gl::FALSE);

                    let mvp = {
                        let pixel = self.camera.world_to_viewport(Vec3::zero()).0;
                        let w = self.camera.viewport_to_world(pixel, 0.);
                        self.camera.view_proj_matrix() * Mat4::translation_3d(w)
                    };
                    g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                    gl::LineWidth(1.);

                    gl::BindVertexArray(self.grid_vertices_01.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_vertices_01.vertices.len() as _);

                    gl::BindVertexArray(self.grid_vertices_1.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_vertices_1.vertices.len() as _);

                    gl::PointSize(8.);
                    gl::BindVertexArray(self.grid_origin_vertices.vao().gl_id());
                    gl::DrawArrays(gl::POINTS, 0, self.grid_origin_vertices.vertices.len() as _);

                    gl::DepthMask(gl::TRUE);
                    gl::Enable(gl::DEPTH_TEST);
                }
            };


            {
                let vp = self.camera.viewport_size();
                gl::Viewport(0, 0, vp.w as _, vp.h as _);
            }


            gl::UseProgram(g.color_mesh_gl_program.program().gl_id());

            if self.draw_grid_first {
                draw_grid();
                draw_cursor();
                draw_draft_vertices();
            } else {
                draw_cursor();
                draw_draft_vertices();
                draw_grid();
            }


            // Render text

            gl::Disable(gl::DEPTH_TEST);

            gl::UseProgram(g.text_gl_program.program().gl_id());
            let mvp = {
                let w = 2. * g.fonts.fonts[&FontID::Debug].texture_size.w as f32 / self.camera.viewport_size().w as f32;
                let h = 2. * g.fonts.fonts[&FontID::Debug].texture_size.h as f32 / self.camera.viewport_size().h as f32;
                Mat4::scaling_3d(Vec3::new(w, h, 1.))
            };
            g.text_gl_program.set_uniform_mvp(&mvp);
            g.text_gl_program.set_uniform_font_atlas_via_font_id(FontID::Debug);
            g.text_gl_program.set_uniform_color(self.text_color);
            gl::BindVertexArray(self.text.vertices.vao().gl_id());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.text.indices.ibo().gl_id());
            gl::DrawElements(gl::TRIANGLES, self.text.indices.indices.len() as _, gl::UNSIGNED_SHORT, ptr::null_mut());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            gl::Enable(gl::DEPTH_TEST);


            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }
}


use gl;
use gx::{Object, BufferUsage};
use system::*;
use v::{Vec3, Rgba, Mat4};
use camera::{Camera2D, CameraProjectionMode, FrustumPlanes};
use xform::Xform2D;
use mesh::{self, Mesh, Vertex};
use duration_ext::DurationExt;

pub struct EditorSystem {
    camera: Camera2D,
    grid_origin_mesh: Mesh,
    grid_mesh_1: Mesh,
    grid_mesh_01: Mesh,
    cursor_mesh: Mesh,
    draw_grid_first: bool,
    do_draw_grid: bool,
    is_panning_camera: bool,
    camera_rotation_speed: f32,
    prev_camera_rotation_z_radians: f32,
    next_camera_rotation_z_radians: f32,
    is_active: bool,
}

fn create_grid_mesh(mesh_gl_program: &mesh::Program, size: Extent2<usize>, color: Rgba<f32>, scale: Extent2<f32>) -> Mesh {
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
    Mesh::from_vertices(&mesh_gl_program, "Grid Mesh", BufferUsage::StaticDraw, vertices)
}

impl EditorSystem {
    // Z epsilon doesn't have to be equal to DEFAULT_NEAR.
    const CAMERA_Z_EPSILON: f32 = 0.001;
    const CAMERA_ZOOM_STEP_FACTOR: f32 = 1.1;
    const CAMERA_NORMAL_Z_ROTATION_SPEED_DEGREES: f32 = 90.;
    const DEFAULT_NEAR: f32 = 0.001;
    const DEFAULT_FAR: f32 = 1000.;
    const DEFAULT_CAMERA_POSITION: Vec3<f32> = Vec3 {
        x: 0.,
        y: 0.,
        z: Self::DEFAULT_NEAR - Self::CAMERA_Z_EPSILON,
    };
    pub fn new(mesh_gl_program: &mesh::Program, viewport_size: Extent2<u32>) -> Self {
        let grid_mesh_1 = create_grid_mesh(mesh_gl_program, Extent2::new(8, 8), Rgba::white(), Extent2::one());
        let grid_mesh_01 = create_grid_mesh(mesh_gl_program, Extent2::new(64, 64), Rgba::new(1., 1., 1., 0.2), Extent2::one()/10.);
        let grid_origin_mesh = Mesh::from_vertices(
            &mesh_gl_program, "Grid Origin Mesh", BufferUsage::StaticDraw,
            vec![Vertex { position: Vec3::zero(), color: Rgba::red(), }]
        );
        let cursor_mesh = Mesh::from_vertices(
            &mesh_gl_program, "Cursor Mesh", BufferUsage::StaticDraw,
            vec![
                Vertex { position: Vec3::zero(), color: Rgba::red(), },
                Vertex { position: Vec3::unit_x(), color: Rgba::green(), },
                Vertex { position: Vec3::unit_y(), color: Rgba::blue(), },
            ]
        );
        let near = Self::DEFAULT_NEAR;
        let far = Self::DEFAULT_FAR;
        let camera = Camera2D {
            xform: Xform2D {
                position: Self::DEFAULT_CAMERA_POSITION,
                .. Default::default()
            },
            projection_mode: CameraProjectionMode::Ortho,
            fov_y_radians: 60_f32.to_radians(),
            viewport_size,
            frustum: FrustumPlanes {
                left: -1., right: 1., bottom: -1., top: 1., near, far,
            },
        };
        let mut s = Self {
            camera, cursor_mesh, grid_origin_mesh, grid_mesh_1, grid_mesh_01,
            draw_grid_first: true,
            do_draw_grid: true,
            is_panning_camera: false,
            camera_rotation_speed: 0.,
            prev_camera_rotation_z_radians: 0.,
            next_camera_rotation_z_radians: 0.,
            is_active: false,
        };
        s.reshape(viewport_size);
        s
    }
    fn reshape(&mut self, size: Extent2<u32>) {
        let c = &mut self.camera;
        c.viewport_size = size;
        c.frustum.right = c.aspect_ratio();
        c.frustum.left = -c.frustum.right;
    }
    fn on_enter_editor(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(0.1, 0.2, 1., 1.);
        }
        g.platform.cursors.crosshair.set();
        self.is_active = true;
    }
    fn on_leave_editor(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(1., 1., 1., 1.);
        }
        g.platform.cursors.normal.set();
        self.is_active = false;
    }
}

impl System for EditorSystem {
    fn name(&self) -> &str {
        "EditorSystem"
    }
    fn on_canvas_resized(&mut self, _: &Game, size: Extent2<u32>, _by_user: bool) {
        self.reshape(size);
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
                self.camera.xform.position.z = Self::DEFAULT_CAMERA_POSITION.z;
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

        let normal_camera_rotation_speed = Self::CAMERA_NORMAL_Z_ROTATION_SPEED_DEGREES.to_radians();

        match *msg {
            Message::EditorToggleDrawGridFirst => self.draw_grid_first = !self.draw_grid_first,
            Message::EditorToggleGrid => self.do_draw_grid = !self.do_draw_grid,
            Message::EditorBeginPanCameraViaMouse => self.is_panning_camera = true,
            Message::EditorEndPanCameraViaMouse => self.is_panning_camera = false,
            Message::EditorBeginRotateCameraLeft => self.camera_rotation_speed = normal_camera_rotation_speed,
            Message::EditorBeginRotateCameraRight => self.camera_rotation_speed = -normal_camera_rotation_speed,
            Message::EditorEndRotateCamera => self.camera_rotation_speed = 0.,
            Message::EditorRecenterCamera => self.camera.xform.position = Self::DEFAULT_CAMERA_POSITION,
            Message::EditorResetCameraRotation => {
                self.camera.xform.rotation_z_radians = 0.;
                self.prev_camera_rotation_z_radians = 0.;
                self.next_camera_rotation_z_radians = 0.;
            },
            Message::EditorResetCameraZoom => self.camera.xform.scale = Vec2::one(),
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
            gl::UseProgram(g.mesh_gl_program.program().gl_id());
            gl::Viewport(0, 0, self.camera.viewport_size.w as _, self.camera.viewport_size.h as _);

            let draw_cursor = || if let Some(pos) = g.input.mouse_position() {
                let mvp = {
                    let mut w = self.camera.viewport_to_world(pos, 0.);
                    w.z += Self::CAMERA_Z_EPSILON; // XXX HACK
                    self.camera.view_proj_matrix() * Mat4::translation_3d(w)
                };
                g.mesh_gl_program.set_uniform_mvp(&mvp);
                gl::PointSize(8.);
                gl::BindVertexArray(self.cursor_mesh.vao().gl_id());
                gl::DrawArrays(gl::POINTS, 0, self.cursor_mesh.vertices.len() as _);
                gl::DrawArrays(gl::TRIANGLES, 0, self.cursor_mesh.vertices.len() as _);
            };

            let draw_grid = || {
                if self.do_draw_grid {
                    gl::Disable(gl::DEPTH_TEST);
                    gl::DepthMask(gl::FALSE);

                    let mvp = {
                        let pixel = self.camera.world_to_viewport(Vec3::zero()).0;
                        let mut w = self.camera.viewport_to_world(pixel, 0.);
                        w.z += Self::CAMERA_Z_EPSILON; // XXX HACK
                        self.camera.view_proj_matrix() * Mat4::translation_3d(w)
                    };
                    g.mesh_gl_program.set_uniform_mvp(&mvp);
                    gl::LineWidth(1.);

                    gl::BindVertexArray(self.grid_mesh_01.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_mesh_01.vertices.len() as _);

                    gl::BindVertexArray(self.grid_mesh_1.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_mesh_1.vertices.len() as _);

                    gl::PointSize(8.);
                    gl::BindVertexArray(self.grid_origin_mesh.vao().gl_id());
                    gl::DrawArrays(gl::POINTS, 0, self.grid_origin_mesh.vertices.len() as _);

                    gl::DepthMask(gl::TRUE);
                    gl::Enable(gl::DEPTH_TEST);
                }
            };

            if self.draw_grid_first {
                draw_grid();
                draw_cursor();
            } else {
                draw_cursor();
                draw_grid();
            }

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }
}


use mesh;
use gl;
use gx::Object;
use system::*;
use v::{Vec3, Rgba, Mat4};
use camera::{Camera2D, CameraProjectionMode, FrustumPlanes};
use xform::Xform2D;

pub struct EditorSystem {
    camera: Camera2D,
    grid_origin_mesh: mesh::Mesh,
    grid_mesh_1: mesh::Mesh,
    grid_mesh_01: mesh::Mesh,
    cursor_mesh: mesh::Mesh,
    cursor_ray_origin: Option<Vec3<f32>>,
    draw_grid_first: bool,
    do_draw_grid: bool,
    is_panning_camera: bool,
    previous_mouse_position: Vec2<i32>,
}

fn create_grid_mesh(mesh_gl_program: &mesh::Program, size: Extent2<usize>, color: Rgba<f32>, scale: Extent2<f32>) -> mesh::Mesh {
    let (w, h) = size.map(|x| x as isize).into_tuple();
    let mut vertices = Vec::with_capacity((w * h) as usize);
    for y in (-h) .. (1 + h) {
        let color = if y == 0 { Rgba::red() } else { color };
        vertices.push(mesh::Vertex { position: Vec3::new(-w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
        vertices.push(mesh::Vertex { position: Vec3::new( w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
    }
    for x in (-w) .. (1 + w) {
        let color = if x == 0 { Rgba::green() } else { color };
        vertices.push(mesh::Vertex { position: Vec3::new(x as f32 * scale.w, -h as f32 * scale.h, 0.), color, });
        vertices.push(mesh::Vertex { position: Vec3::new(x as f32 * scale.w,  h as f32 * scale.h, 0.), color, });
    }
    mesh::Mesh::from_vertices(&mesh_gl_program, "Grid Mesh", ::gx::BufferUsage::StaticDraw, vertices)
}

impl EditorSystem {
    // FIXME: It might be that the mouse isn't actually in the window at first
    pub fn new(mesh_gl_program: &mesh::Program, viewport_size: Extent2<u32>, mouse_position: Vec2<i32>) -> Self {
        let grid_mesh_1 = create_grid_mesh(mesh_gl_program, Extent2::new(8, 8), Rgba::white(), Extent2::one());
        let grid_mesh_01 = create_grid_mesh(mesh_gl_program, Extent2::new(64, 64), Rgba::new(1., 1., 1., 0.2), Extent2::one()/10.);
        let grid_origin_mesh = mesh::Mesh::from_vertices(
            &mesh_gl_program, "Grid Origin Mesh", ::gx::BufferUsage::StaticDraw,
            vec![mesh::Vertex { position: Vec3::zero(), color: Rgba::red(), }]
        );
        let cursor_mesh = mesh::Mesh::from_vertices(
            &mesh_gl_program, "Cursor Mesh", ::gx::BufferUsage::StaticDraw,
            vec![
                mesh::Vertex { position: Vec3::zero(), color: Rgba::red(), },
                mesh::Vertex { position: Vec3::unit_x(), color: Rgba::green(), },
                mesh::Vertex { position: Vec3::unit_y(), color: Rgba::blue(), },
            ]
        );
        let near = 0.001;
        let far = 1000.;
        let camera = Camera2D {
            xform: {
                let mut xform = Xform2D::default();
                xform.position.z -= near;
                xform
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
            cursor_ray_origin: None, // FIXME not always true
            draw_grid_first: true,
            do_draw_grid: true,
            is_panning_camera: false,
            previous_mouse_position: mouse_position,
        };
        s.reshape(viewport_size);
        s
    }
    pub fn cursor_mvp(&self) -> Option<Mat4<f32>> {
        self.cursor_ray_origin.map(|p| {
            let m = Mat4::translation_3d(p);
            self.camera.view_proj_matrix() * m
        })
    }
    fn reshape(&mut self, size: Extent2<u32>) {
        self.camera.viewport_size = size;
        let aspect = self.camera.aspect_ratio();
        self.camera.frustum.right = aspect;
        self.camera.frustum.left = -self.camera.frustum.right;
    }
}

impl System for EditorSystem {
    fn name(&self) -> &str { "EditorSystem" }
    fn on_canvas_resized(&mut self, _: &Game, size: Extent2<u32>, _by_user: bool) {
        self.reshape(size);
    }
    fn on_mouse_enter(&mut self, g: &Game) {
        g.platform.cursors.crosshair.set();
        let pos = g.platform.mouse_position();
        self.on_mouse_motion(g, pos);
    }
    fn on_mouse_leave(&mut self, _: &Game) {
        self.cursor_ray_origin = None;
    }
    fn on_mouse_motion(&mut self, _: &Game, pos: Vec2<i32>) {
        {
            let mut pos = pos;
            pos.y = self.camera.viewport_size.h as i32 - pos.y;
            let pos = Vec3::from(pos.map(|x| x as f32));

            let mut prev = self.previous_mouse_position;
            prev.y = self.camera.viewport_size.h as i32 - prev.y;
            let prev = Vec3::from(prev.map(|x| x as f32));

            if self.is_panning_camera {
                // FIXME: Rotate and scale
                let d = pos - prev + Vec3::from(Vec2::from(self.camera.viewport_size.map(|x| x as f32) / 2.));
                let dumb = Camera2D {
                    xform: Default::default(), 
                    .. self.camera
                };
                self.camera.xform.position -= dumb.viewport_to_world(d);
            }

            self.cursor_ray_origin = Some(self.camera.viewport_to_world(pos));
        }
        
        self.previous_mouse_position = pos;
    }
    fn on_message(&mut self, _: &Game, msg: &Message) {
        match *msg {
            Message::EditorToggleDrawGridFirst => self.draw_grid_first = !self.draw_grid_first,
            Message::EditorToggleGrid => self.do_draw_grid = !self.do_draw_grid,
            Message::EditorBeginPanCameraViaMouse => self.is_panning_camera = true,
            Message::EditorEndPanCameraViaMouse => self.is_panning_camera = false,
        };
    }
    fn draw(&mut self, g: &Game, _: f64) {
        unsafe {
            gl::UseProgram(g.mesh_gl_program.program().gl_id());
            gl::Viewport(0, 0, self.camera.viewport_size.w as _, self.camera.viewport_size.h as _);

            let draw_cursor = || if let Some(mvp) = self.cursor_mvp().as_ref() {
                g.mesh_gl_program.set_uniform_mvp(mvp);
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
                        let pixel = self.camera.world_to_viewport(Vec3::zero()).map(|x| x.round() + 0.5);
                        let m = Mat4::translation_3d(self.camera.viewport_to_world(pixel));
                        self.camera.view_proj_matrix() * m
                    };
                    g.mesh_gl_program.set_uniform_mvp(&mvp);
                    gl::LineWidth(1.);

                    gl::BindVertexArray(self.grid_mesh_01.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_mesh_01.vertices.len() as _);

                    gl::BindVertexArray(self.grid_mesh_1.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_mesh_1.vertices.len() as _);

                    gl::PointSize(16.);
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


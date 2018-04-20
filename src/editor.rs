use mesh;
use gl;
use gx::Object;
use system::*;
use v::{Vec3, Rgba, Mat4};
use camera::{Camera2D, CameraProjectionMode, FrustumPlanes};
use xform::Xform2D;

pub struct EditorSystem {
    camera: Camera2D,
    cursor_mesh: mesh::Mesh,
    cursor_ray_origin: Option<Vec3<f32>>,
}

impl EditorSystem {
    pub fn new(mesh_gl_program: &mesh::Program, viewport_size: Extent2<u32>) -> Self {
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
        let cursor_ray_origin = None;
        let mut s = Self { camera, cursor_mesh, cursor_ray_origin, };
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
    fn on_mouse_motion(&mut self, _: &Game, mut pos: Vec2<i32>) {
        pos.y = self.camera.viewport_size.h as i32 - pos.y;
        let pos = Vec3::from(pos.map(|x| x as f32));
        self.cursor_ray_origin = Some(self.camera.viewport_to_world(pos));
    }
    fn draw(&mut self, g: &Game, _: f64) {
        if let Some(mvp) = self.cursor_mvp().as_ref() {
            unsafe {
                gl::Viewport(0, 0, self.camera.viewport_size.w as _, self.camera.viewport_size.h as _);
                gl::PointSize(8.);
                gl::UseProgram(g.mesh_gl_program.program().gl_id());
                g.mesh_gl_program.set_uniform_mvp(mvp);
                gl::BindVertexArray(self.cursor_mesh.vao().gl_id());
                gl::DrawArrays(gl::POINTS, 0, self.cursor_mesh.vertices.len() as _);
                gl::DrawArrays(gl::TRIANGLES, 0, self.cursor_mesh.vertices.len() as _);
                gl::BindVertexArray(0);
                gl::UseProgram(0);
            }
        }
    }
}


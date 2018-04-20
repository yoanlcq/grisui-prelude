use xform::Xform2D;
use v::{Mat4, Vec3, Extent2, Rect,};
pub use v::FrustumPlanes;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera2D {
    pub xform: Xform2D,
    pub projection_mode: CameraProjectionMode,
    pub fov_y_radians: f32,
    pub viewport_size: Extent2<u32>,
    pub frustum: FrustumPlanes<f32>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CameraProjectionMode {
    Perspective,
    Ortho,
}

impl Camera2D {
    pub fn aspect_ratio(&self) -> f32 {
        self.viewport_size.w as f32 / self.viewport_size.h as f32
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        match self.projection_mode {
            CameraProjectionMode::Perspective => {
                let fov = self.fov_y_radians;
                let aspect = self.aspect_ratio();
                let near = self.frustum.near;
                let far = self.frustum.far;
                Mat4::perspective_lh_no(fov, aspect, near, far)
            },
            CameraProjectionMode::Ortho => {
                Mat4::orthographic_lh_no(self.frustum)
            },
        }
    }
    pub fn view_matrix(&self) -> Mat4<f32> {
        let eye = self.xform.position;
        let target = eye + self.xform.forward();
        let up = self.xform.up().into();
        Mat4::look_at(eye, target, up)
    }
    pub fn view_proj_matrix(&self) -> Mat4<f32> {
        self.proj_matrix() * self.view_matrix()
    }
    pub fn viewport(&self) -> Rect<f32, f32> {
        Rect {
            x: 0.,
            y: 0.,
            w: self.viewport_size.w as _,
            h: self.viewport_size.h as _,
        }
    }
    pub fn viewport_to_world(&self, p: Vec3<f32>) -> Vec3<f32> {
        let modelview = self.view_matrix();
        let proj = self.proj_matrix();
        let viewport = self.viewport();
        Mat4::viewport_to_world_no(p, modelview, proj, viewport)
    }
    pub fn world_to_viewport(&self, p: Vec3<f32>) -> Vec3<f32> {
        let modelview = self.view_matrix();
        let proj = self.proj_matrix();
        let viewport = self.viewport();
        Mat4::world_to_viewport_no(p, modelview, proj, viewport)
    }
}

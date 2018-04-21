use xform::Xform2D;
use v::{Mat4, Vec2, Vec3, Extent2, Rect};
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
        let mut scale = Vec3::from(self.xform.scale);
        scale.z = 1.;
        Mat4::<f32>::look_at(eye, target, up) * Mat4::scaling_3d(scale)
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
    pub fn viewport_to_world(&self, p: Vec2<i32>, z: f32) -> Vec3<f32> {
        let y = self.viewport_size.h as i32 - p.y;
        let v = Vec3::new(p.x as f32 + 0.5, y as f32 + 0.5, z);
        Mat4::viewport_to_world_no(v, self.view_matrix(), self.proj_matrix(), self.viewport())
    }
    pub fn world_to_viewport(&self, o: Vec3<f32>) -> (Vec2<i32>, f32) {
        let v = Mat4::world_to_viewport_no(o, self.view_matrix(), self.proj_matrix(), self.viewport());
        let (mut v, z) = (Vec2::from(v.map(|x| x.round() as i32)), v.z);
        v.y = self.viewport_size.h as i32 - v.y;
        (v, z)
    }
}

use transform::Transform;
use Mat4;
use Vec3;
use Extent2;
use vek::geom::FrustumPlanes;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PerspectiveCamera {
    pub transform: Transform,
    pub viewport_size: Extent2<u32>,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OrthographicCamera {
    pub transform: Transform,
    pub frustum: FrustumPlanes<f32>,
}

impl PerspectiveCamera {
    pub fn aspect_ratio(&self) -> f32 {
        self.viewport_size.w as f32 / (self.viewport_size.h as f32)
    }
    pub fn view_matrix(&self) -> Mat4<f32> {
        Mat4::look_at(
            self.transform.position, 
            self.transform.position + self.transform.forward(),
            Vec3::unit_y()
        )
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        Mat4::perspective(self.fov_y, self.aspect_ratio(), self.near, self.far)
    }
    pub fn view_proj_matrix(&self) -> Mat4<f32> {
        self.proj_matrix() * self.view_matrix()
    }
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        Self {
            transform: Transform::lerp(&a.transform, &b.transform, t),
            viewport_size: lerp(a.viewport_size, b.viewport_size, t),
            fov_y: lerp(a.fov_y, b.fov_y, t),
            near: lerp(a.near, b.near, t),
            far: lerp(a.far, b.far, t),
        }
    }
}

impl OrthographicCamera {
    pub fn view_matrix(&self) -> Mat4<f32> {
        Mat4::look_at(
            self.transform.position, 
            self.transform.position + self.transform.forward(),
            Vec3::unit_y()
        )
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        Mat4::orthographic(self.frustum)
    }
    pub fn view_proj_matrix(&self) -> Mat4<f32> {
        self.proj_matrix() * self.view_matrix()
    }
}


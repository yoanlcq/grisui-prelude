use Mat4;
use Vec3;
use SimdVec3;
use Extent2;
use Lerp;
use Clamp;
use Transform;
use FrustumPlanes;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PerspectiveCamera {
    pub transform: Transform<f32, f32, f32>,
    pub viewport_size: Extent2<u32>,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OrthographicCamera {
    pub transform: Transform<f32, f32, f32>,
    pub frustum: FrustumPlanes<f32>,
}

pub trait TransformExt {
    fn forward_lh(&self) -> Vec3<f32>;
}

impl TransformExt for Transform<f32, f32, f32> {
    fn forward_lh(&self) -> Vec3<f32> {
        self.orientation * Vec3::<f32>::forward_lh()
    }
}

impl PerspectiveCamera {
    pub fn aspect_ratio(&self) -> f32 {
        self.viewport_size.w as f32 / (self.viewport_size.h as f32)
    }
    pub fn view_matrix(&self) -> Mat4<f32> {
        Mat4::look_at(
            self.transform.position, 
            self.transform.position + self.transform.forward_lh(),
            SimdVec3::unit_y()
        )
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        Mat4::perspective_lh_no(self.fov_y, self.aspect_ratio(), self.near, self.far)
    }
    pub fn view_proj_matrix(&self) -> Mat4<f32> {
        self.proj_matrix() * self.view_matrix()
    }
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamped01();
        Self {
            transform: Lerp::lerp(&a.transform, &b.transform, t),
            viewport_size: Lerp::lerp_unclamped(
                a.viewport_size.map(|x| x as f32),
                b.viewport_size.map(|x| x as f32),
                t
            ).map(|x| x.round() as u32),
            fov_y: Lerp::lerp_unclamped(a.fov_y, b.fov_y, t),
            near: Lerp::lerp_unclamped(a.near, b.near, t),
            far: Lerp::lerp_unclamped(a.far, b.far, t),
        }
    }
}

impl OrthographicCamera {
    pub fn view_matrix(&self) -> Mat4<f32> {
        Mat4::look_at(
            self.transform.position, 
            self.transform.position + self.transform.forward_lh(),
            SimdVec3::unit_y()
        )
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        Mat4::orthographic_lh_no(self.frustum)
    }
    pub fn view_proj_matrix(&self) -> Mat4<f32> {
        self.proj_matrix() * self.view_matrix()
    }
}


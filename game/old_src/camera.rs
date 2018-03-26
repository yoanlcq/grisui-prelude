use v::{
    Mat4,
    Rect,
    Vec2,
    Vec3,
    FrustumPlanes,
};
use transform::Transform3D;
use transform_ext::TransformExt;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PerspectiveCamera {
    pub viewport: Rect<u32, u32>,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OrthographicCamera {
    pub viewport: Rect<u32, u32>,
    pub ortho_right: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ray<T> {
    pub position: Vec3<T>,
    pub direction: Vec3<T>,
}

macro_rules! impl_camera {
    ($Camera:ident) => {
        impl $Camera {
            pub fn aspect_ratio(&self) -> f32 {
                (self.viewport.w as f32) / (self.viewport.h as f32)
            }
            pub fn view_matrix(&self, xform: &Transform3D) -> Mat4<f32> {
                Mat4::look_at(
                    xform.position, 
                    xform.position + xform.forward_lh(),
                    Vec3::unit_y()
                )
            }
            pub fn view_proj_matrix(&self, xform: &Transform3D) -> Mat4<f32> {
                self.proj_matrix() * self.view_matrix(xform)
            }
            pub fn world_to_viewport_point(&self, xform: &Transform3D, p: Vec3<f32>) -> Vec3<f32> {
                Mat4::world_to_viewport_no(
                    p, 
                    self.view_matrix(xform), self.proj_matrix(),
                    self.viewport.map(|p| p as f32, |e| e as f32)
                ).into()
            }
            pub fn viewport_to_world_point(&self, xform: &Transform3D, p: Vec3<f32>) -> Vec3<f32> {
                Mat4::viewport_to_world_no(
                    p, 
                    self.view_matrix(xform), self.proj_matrix(),
                    self.viewport.map(|p| p as f32, |e| e as f32)
                ).into()
            }
            pub fn viewport_to_world_ray(&self, xform: &Transform3D, p: Vec2<f32>) -> Ray<f32> {
                let p1 = self.viewport_to_world_point(xform, Vec3::new(p.x, p.y, 0.));
                let p2 = self.viewport_to_world_point(xform, Vec3::new(p.x, p.y, 1.));
                Ray {
                    position: p1,
                    direction: (p2-p1).normalized(),
                }
            }
        }
    }
}

impl_camera!{PerspectiveCamera}
impl_camera!{OrthographicCamera}

impl PerspectiveCamera {
    pub fn proj_matrix(&self) -> Mat4<f32> {
        Mat4::perspective_lh_no(self.fov_y, self.aspect_ratio(), self.near, self.far)
    }
    /*
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamped01();
        Self {
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
    */
}

impl OrthographicCamera {
    pub fn proj_matrix(&self) -> Mat4<f32> {
        Mat4::orthographic_lh_no(self.frustum())
    }
    pub fn frustum(&self) -> FrustumPlanes<f32> {
        FrustumPlanes {
            left: -self.ortho_right,
            right: self.ortho_right,
            bottom: -self.ortho_top(),
            top: self.ortho_top(),
            near: self.near,
            far: self.far,
        }
    }
    pub fn ortho_top(&self) -> f32 {
        self.ortho_right / self.aspect_ratio()
    }
    /*
    pub fn quick_viewport_to_world(p: Vec2<f32>) -> Vec2<f32> {
        Vec2::new(-1 + 2.*x/w, 1 - 2.*y/h)
    }
    */
}


use xform::Xform2D;
use v::{Mat4, Vec2, Vec3, Extent2, Rect};
pub use v::FrustumPlanes;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OrthoCamera2D {
    pub xform: Xform2D,
    viewport_size: Extent2<u32>,
    frustum: FrustumPlanes<f32>,
}


impl OrthoCamera2D {
    fn frustum_for_viewport_size(size: Extent2<u32>, near: f32, far: f32) -> FrustumPlanes<f32> {
        let aspect = size.w as f32 / size.h as f32;
        FrustumPlanes {
            right: aspect,
            left: -aspect,
            top: 1.,
            bottom: -1.,
            near, far
        }
    }
    pub fn new(viewport_size: Extent2<u32>, near: f32, far: f32) -> Self {
        Self {
            xform: Default::default(),
            viewport_size,
            frustum: Self::frustum_for_viewport_size(viewport_size, near, far),
        }
    }
    pub fn viewport_size(&self) -> Extent2<u32> {
        self.viewport_size
    }
    pub fn set_viewport_size(&mut self, size: Extent2<u32>) {
        self.viewport_size = size;
        self.frustum = Self::frustum_for_viewport_size(size, self.frustum.near, self.frustum.far);
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.viewport_size.w as f32 / self.viewport_size.h as f32
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        Mat4::orthographic_lh_no(self.frustum)
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
        let v = Vec3::new(p.x as f32 + 0.5, y as f32 + 0.5, 0.);
        let mut w = Mat4::viewport_to_world_no(v, self.view_matrix(), self.proj_matrix(), self.viewport());
        w.z = z;
        w
    }
    pub fn world_to_viewport(&self, o: Vec3<f32>) -> (Vec2<i32>, f32) {
        let v = Mat4::world_to_viewport_no(o, self.view_matrix(), self.proj_matrix(), self.viewport());
        let (mut z, mut v) = (v.z, Vec2::from(v.map(|x| x.round() as i32)));
        if z.abs() <= 0.0001 {
            z = 0.;
        }
        v.y = self.viewport_size.h as i32 - v.y;
        (v, z)
    }
}

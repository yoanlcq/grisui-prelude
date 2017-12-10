use Vec3;
use Quaternion;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform {
    pub position: Vec3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vec3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Quaternion::identity(),
            scale: Vec3::one(),
        }
    }
}

impl Transform {
    pub fn forward(&self) -> Vec3<f32> {
        // WISH(yoanlcq): vek: Fix repr_simd quaternion vs. repr_c vec3....
        let Vec4 { x, y, z, .. } = self.rotation * Vec4::forward_rh();
        Vec3 { x, y, z }
    }
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        Self {
            position: lerp(a.position, b.position, t),
            rotation: slerp(a.rotation, b.rotation, t),
            scale: lerp(a.scale, b.scale, t),
        }
    }
}


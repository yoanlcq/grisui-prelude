use v::{Vec2, Vec3, Quaternion, Mat4};

pub type Transform3D = ::v::Transform<f32, f32, f32>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform2D {
    pub position: Vec3<f32>,
    /// I chose degrees for these reasons :
    /// - More convenient to look at in an inspector;
    /// - More convenient to hand-edit in files.
    pub z_rotation_degrees: f32,
    pub scale: Vec2<f32>,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            z_rotation_degrees: 0.,
            scale: Vec2::one()
        }
    }
}

impl Transform2D {
    pub fn into_transform_3d(self) -> Transform3D {
        Transform3D {
            position: self.position,
            orientation: Quaternion::rotation_z(self.z_rotation_degrees.to_radians()),
            scale: Vec3::from(self.scale) + Vec3::unit_z(),
        }
    }
    pub fn into_mat4(self) -> Mat4<f32> {
        self.into_transform_3d().into()
    }
}

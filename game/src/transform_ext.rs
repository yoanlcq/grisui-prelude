use transform::Transform3D;
use v::Vec3;

pub trait TransformExt {
    fn forward_lh(&self) -> Vec3<f32>;
}

impl TransformExt for Transform3D {
    fn forward_lh(&self) -> Vec3<f32> {
        self.orientation * Vec3::<f32>::forward_lh()
    }
}

use v::{Vec3, Transform};

pub trait TransformExt {
    fn forward_lh(&self) -> Vec3<f32>;
}

impl TransformExt for Transform<f32, f32, f32> {
    fn forward_lh(&self) -> Vec3<f32> {
        self.orientation * Vec3::<f32>::forward_lh()
    }
}

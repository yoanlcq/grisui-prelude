extern crate vek;

// NOTE: Avoid repr_simd for alignment reasons (when sending packed data to OpenGL)
// Also, it's more convenient. repr_simd is better for mass processing.
pub use self::vek::vec::repr_c::{Vec4, Vec2, Vec3, Rgba, Rgb, Extent2};
pub use self::vek::vec::repr_simd::{Vec3 as Simd3, Vec4 as Simd4};
pub use self::vek::mat::repr_c::column_major::{Mat4, Mat3, Mat2};
pub use self::vek::quaternion::repr_c::Quaternion;
pub use self::vek::ops::*;
pub use self::vek::geom::*;
pub use self::vek::transform::repr_c::Transform;

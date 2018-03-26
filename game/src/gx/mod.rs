pub mod get;
pub mod state;
pub mod object;
pub mod buffer;
pub mod shader;
pub mod texture_unit;
pub mod texture;

pub use self::get::*;
pub use self::object::*;
pub use self::utils::*;
pub use self::texture_unit::TextureUnit;
pub use self::texture::TexImage2D;

// TODO: report to gl crate.
pub mod fix {
    use gl::types::*;
    pub const CONTEXT_FLAG_NO_ERROR_BIT_KHR: GLuint = 0x00000008;
}

pub mod utils {
    pub fn parse_version_string(version_string: &str) -> (u32, u32) {
        (version_string.chars().nth(0).unwrap() as u32 - '0' as u32,
         version_string.chars().nth(2).unwrap() as u32 - '0' as u32)
    }
}


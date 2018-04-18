use super::object::*;
use super::texture_unit::TextureUnit;
use gl;

pub fn use_program(prog: &Program) {
    unsafe {
        gl::UseProgram(prog.gl_id());
    }
}
pub fn set_active_texture_unit(unit: TextureUnit) {
    unsafe {
        gl::ActiveTexture(unit.to_glenum());
    }
}
/*
pub fn bind_texture<'a, T: Into<Option<&'a Texture>>>(target: GLenum, tex: T) {
    unsafe {
        gl::BindTexture(target, match tex.into() {
            Some(ref tex) => tex.gl_id(),
            None => 0,
        });
    }
}
pub fn bind_texture_2d<'a, T: Into<Option<&'a Texture2D>>>(tex: T) {
    bind_texture(gl::TEXTURE_2D, tex)
}
pub fn bind_texture_cube_map<'a, T: Into<Option<&'a TextureCubeMap>>>(tex: T) {
    bind_texture(gl::TEXTURE_CUBE_MAP, tex)
}

*/

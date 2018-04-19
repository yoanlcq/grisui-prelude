use gl;
use gl::types::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TexImage2D<'a, Pixel: 'a> {
    pub pixels: &'a [Pixel],
    pub width: GLuint,
    pub height: GLuint,
    pub mipmap_level: GLint, // 0
    pub internal_format: GLenum,
    pub pixels_format: GLenum,
    pub pixel_element_type: GLenum,
}

impl<'a, Pixel: 'a> TexImage2D<'a, Pixel> {
    pub fn tex_image_2d(&self, target: GLenum) {
        unsafe {
            gl::TexImage2D(
                target, self.mipmap_level, self.internal_format as _,
                self.width as _, self.height as _, 0,
                self.pixels_format, self.pixel_element_type, self.pixels.as_ptr() as *const _
            );
        }
    }
}


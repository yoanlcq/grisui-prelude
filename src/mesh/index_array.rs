use std::mem;
use std::ffi::CString;
use std::ops::Range;
use gl::{self, types::*};
use gx::{self, Object};
use grx;

pub trait VertexIndex {
    const GL_TYPE: GLenum;
}

impl VertexIndex for u8  { const GL_TYPE: GLenum = gl::UNSIGNED_BYTE; }
impl VertexIndex for u16 { const GL_TYPE: GLenum = gl::UNSIGNED_SHORT; }
impl VertexIndex for u32 { const GL_TYPE: GLenum = gl::UNSIGNED_INT; }

#[derive(Debug)]
pub struct IndexArray<T: VertexIndex> {
    buffer_usage: gx::BufferUsage,
    pub indices: Vec<T>,
    ibo: gx::Buffer,
}

impl<T: VertexIndex> IndexArray<T> {
    pub const INDEX_GL_TYPE: GLenum = T::GL_TYPE;
    pub fn ibo(&self) -> &gx::Buffer { &self.ibo }
    pub fn update_ibo_range(&self, range: Range<usize>) {
        assert!(range.start <= range.end);
        assert!(range.end <= self.indices.capacity());
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo.gl_id());
            // NOTE: Be careful not to actually index the Vec with range.
            // Will cause panics because we care about the capacity, not the length!
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, ((range.end - range.start) * mem::size_of::<T>()) as _, self.indices.as_ptr().add(range.start) as _, self.buffer_usage as _);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
    pub fn update_ibo(&self) {
        self.update_ibo_range(0..self.indices.capacity());
    }
    pub fn from_indices(
        label: &str,
        buffer_usage: gx::BufferUsage,
        indices: Vec<T>
    ) -> Self
    {
        let ibo = gx::Buffer::new();
        grx::set_label(&ibo, &CString::new(label.to_owned() + " IBO").unwrap().into_bytes_with_nul());
        let array = Self {
            buffer_usage, indices, ibo,
        };
        array.update_ibo();
        array
    }
}


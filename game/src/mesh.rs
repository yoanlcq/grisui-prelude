use std::ffi::CString;
use std::ptr;
use gl;
use gl::types::*;
use gx;
use gx::GLResource;
use grx;
use v::{Vec3, Rgba};

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<grx::SimpleColorVertex>,
    pub gl_topology: GLenum,
    pub update_hint: gx::UpdateHint,
    pub vao: gx::Vao,
    pub vbo: gx::Vbo,
}


impl Mesh {
    pub fn update_vbo(&self) {
        self.vbo.set_data(&self.vertices, self.update_hint);
    }
    pub fn new_unit_quad(prog: &grx::SimpleColorProgram, label: &str, update_hint: gx::UpdateHint) -> Self {
        assert_eq_size!(grx::SimpleColorVertex, [f32; 7]);

        let z = 0.;
        let s = 0.5_f32;
        let vertices = vec![
            grx::SimpleColorVertex { position: Vec3::new(-s, -s, z), color: Rgba::red() },
            grx::SimpleColorVertex { position: Vec3::new( s,  s, z), color: Rgba::yellow() },
            grx::SimpleColorVertex { position: Vec3::new(-s,  s, z), color: Rgba::green() },
            grx::SimpleColorVertex { position: Vec3::new(-s, -s, z), color: Rgba::blue() },
            grx::SimpleColorVertex { position: Vec3::new( s, -s, z), color: Rgba::cyan() },
            grx::SimpleColorVertex { position: Vec3::new( s,  s, z), color: Rgba::black() },
        ];
        let gl_topology = gl::TRIANGLES;
        let vao = gx::Vao::new();
        let vbo = gx::Vbo::new();
        vao.bind();
        vbo.bind();
        vao.set_label(&CString::new(label.to_owned() + " VAO").unwrap().into_bytes_with_nul());
        vbo.set_label(&CString::new(label.to_owned() + " VBO").unwrap().into_bytes_with_nul());
        vbo.set_data(&vertices, update_hint);
        unsafe {
            gl::EnableVertexAttribArray(prog.a_position());
            gl::EnableVertexAttribArray(prog.a_color());
            gl::VertexAttribPointer(
                prog.a_position(), 3, gl::FLOAT,
                gl::FALSE as _, 7*4, ptr::null()
            );
            gl::VertexAttribPointer(
                prog.a_color(), 4, gl::FLOAT,
                gl::FALSE as _, 7*4, ptr::null::<GLvoid>().offset(3*4)
            );
        }

        Self {
            vertices, gl_topology, vbo, vao, update_hint,
        }
    }
}


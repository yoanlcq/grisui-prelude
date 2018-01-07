use std::ffi::CString;
use std::ptr;
use std::mem::size_of;
use gl;
use gl::types::*;
use gx;
use gx::GLResource;
use grx;
use v::{Vec2, Vec3, Rgba};

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
                gl::FALSE as _, size_of::<grx::SimpleColorVertex>() as _,
                ptr::null()
            );
            gl::VertexAttribPointer(
                prog.a_color(), 4, gl::FLOAT,
                gl::FALSE as _, size_of::<grx::SimpleColorVertex>() as _,
                ptr::null::<GLvoid>().offset(3*size_of::<f32>() as isize)
            );
        }

        Self {
            vertices, gl_topology, vbo, vao, update_hint,
        }
    }
}

#[derive(Debug)]
pub struct FontAtlasMesh {
    pub vertices: Vec<grx::TextVertex>,
    pub gl_topology: GLenum,
    pub update_hint: gx::UpdateHint,
    pub vao: gx::Vao,
    pub vbo: gx::Vbo,
}

impl FontAtlasMesh {
    pub fn new_font_atlas_unit_quad(prog: &grx::TextProgram, label: &str, update_hint: gx::UpdateHint) -> Self {
        let vertices = vec![
            grx::TextVertex { position: Vec2::new(0., -1.), texcoords: Vec2::new(0., 1.) },
            grx::TextVertex { position: Vec2::new(1.,  0.), texcoords: Vec2::new(1., 0.) },
            grx::TextVertex { position: Vec2::new(0.,  0.), texcoords: Vec2::new(0., 0.) },
            grx::TextVertex { position: Vec2::new(0., -1.), texcoords: Vec2::new(0., 1.) },
            grx::TextVertex { position: Vec2::new(1., -1.), texcoords: Vec2::new(1., 1.) },
            grx::TextVertex { position: Vec2::new(1.,  0.), texcoords: Vec2::new(1., 0.) },
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
            gl::EnableVertexAttribArray(prog.a_texcoords());
            gl::VertexAttribPointer(
                prog.a_position(), 2, gl::FLOAT,
                gl::FALSE as _, size_of::<grx::TextVertex>() as _,
                ptr::null()
            );
            gl::VertexAttribPointer(
                prog.a_texcoords(), 2, gl::FLOAT,
                gl::FALSE as _, size_of::<grx::TextVertex>() as _,
                ptr::null::<GLvoid>().offset(2*size_of::<f32>() as isize)
            );
        }

        Self {
            vertices, gl_topology, vbo, vao, update_hint,
        }
    }
}

use std::mem;
use std::ptr;
use std::ffi::CString;
use gx::{self, Object};
use grx;
use gl::{self, types::*};
use v::{Vec3, Rgba, Mat4};

#[derive(Debug)]
pub struct Mesh {
    buffer_usage: gx::BufferUsage,
    pub vertices: Vec<Vertex>,
    vbo: gx::Buffer,
    vao: gx::VertexArray,
}


#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec3<f32>,
    pub color: Rgba<f32>,
}
assert_eq_size!(vertex_size; Vertex, [f32; 7]);


#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Program {
    program: gx::Program,
    u_mvp: GLint,
    a_position: GLuint,
    a_color: GLuint,
}


impl Mesh {
    pub fn vao(&self) -> &gx::VertexArray { &self.vao }
    pub fn vbo(&self) -> &gx::Buffer { &self.vbo }
    pub fn update_vbo(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo.gl_id());
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(self.vertices.as_slice()) as _, self.vertices.as_ptr() as _, self.buffer_usage as _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn from_vertices(
        prog: &Program,
        label: &str,
        buffer_usage: gx::BufferUsage,
        vertices: Vec<Vertex>
    ) -> Self
    {
        let vao = gx::VertexArray::new();
        let vbo = gx::Buffer::new();
        unsafe {
            gl::BindVertexArray(vao.gl_id());
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.gl_id());
            grx::set_label(&vao, &CString::new(label.to_owned() + " VAO").unwrap().into_bytes_with_nul());
            grx::set_label(&vbo, &CString::new(label.to_owned() + " VBO").unwrap().into_bytes_with_nul());

            gl::EnableVertexAttribArray(prog.a_position());
            gl::EnableVertexAttribArray(prog.a_color());
            gl::VertexAttribPointer(
                prog.a_position(), 3, gl::FLOAT, gl::FALSE as _,
                mem::size_of::<Vertex>() as _,
                ptr::null()
            );
            gl::VertexAttribPointer(
                prog.a_color(), 4, gl::FLOAT, gl::FALSE as _,
                mem::size_of::<Vertex>() as _,
                ptr::null::<GLvoid>().add(mem::size_of::<Vec3<f32>>())
            );
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        let mesh = Self {
            vertices, vbo, vao, buffer_usage,
        };
        mesh.update_vbo();
        mesh
    }
}

impl Program {

    const VS: &'static [u8] = b"
#version 130
uniform mat4 u_mvp;
in vec3 a_position;
in vec4 a_color;
out vec4 v_color;
void main() {
    gl_Position = u_mvp * vec4(a_position, 1.0);
    v_color = a_color;
}
\0";


    const FS: &'static [u8] = b"
#version 130
in vec4 v_color;
out vec4 f_color;
void main() {
    f_color = v_color;
}
\0";

    pub fn program(&self) -> &gx::Program {
        &self.program
    }

    pub fn a_position(&self) -> GLuint {
        self.a_position
    }
    pub fn a_color(&self) -> GLuint {
        self.a_color
    }
    pub fn new() -> Self {
        let vs = match gx::VertexShader::try_from_source(Self::VS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile vertex shader:\n{}", s);
                panic!()
            },
        };
        grx::set_label(&vs, b"Mesh Vertex Shader");
        let fs = match gx::FragmentShader::try_from_source(Self::FS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile fragment shader:\n{}", s);
                panic!()
            },
        };
        grx::set_label(&fs, b"Mesh Fragment Shader");
        let program = match gx::Program::try_from_vert_frag(&vs, &fs) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to link GL program:\n{}", s);
                panic!()
            },
        };
        grx::set_label(&program, b"Mesh Program");

        let a_position = program.attrib_location(b"a_position\0").unwrap() as _;
        let a_color = program.attrib_location(b"a_color\0").unwrap() as _;
        let u_mvp = program.uniform_location(b"u_mvp\0").unwrap();

        Self {
            program, u_mvp, a_position, a_color,
        }
    }
    pub fn set_uniform_mvp(&self, m: &Mat4<f32>) {
        let transpose = m.gl_should_transpose() as GLboolean;
        unsafe {
            gl::UniformMatrix4fv(self.u_mvp, 1, transpose, m.cols[0].as_ptr());
        }
    }
}


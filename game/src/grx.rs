use gx;
use Mat4;
use Vec3;
use Rgba;

use gx::Object;

use gl::types::*;

static VS_SRC: &[u8] = b"
    #version 130
    uniform mat4 u_mvp;
    in vec3 a_position;
    in vec4 a_color;
    out vec4 v_color;
    void main() {
        vec4 pos = vec4(a_position, 1.0);
        float e = 0.00001;
        pos = (u_mvp * vec4(e,e,e,e)) + pos;
        gl_Position = pos;
        v_color = a_color;
    }
\0";

static FS_SRC: &[u8] = b"
    #version 130
    in vec4 v_color;
    out vec4 f_color;
    void main() {
        f_color = v_color;
    }
\0";

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SimpleColorProgram {
    program: gx::Program,
    u_mvp: GLint,
    a_position: GLuint,
    a_color: GLuint,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SimpleColorVertex {
    pub position: Vec3<f32>,
    pub color: Rgba<f32>,
}

impl SimpleColorProgram {
    pub fn a_position(&self) -> GLuint {
        self.a_position
    }
    pub fn a_color(&self) -> GLuint {
        self.a_color
    }
    pub fn new() -> Self {
        let vs = match gx::VertexShader::from_source(VS_SRC) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile vertex shader:\n{}", s);
                panic!()
            },
        };
        vs.set_label(b"SimpleColorProgram Vertex Shader");
        let fs = match gx::FragmentShader::from_source(FS_SRC) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile fragment shader:\n{}", s);
                panic!()
            },
        };
        fs.set_label(b"SimpleColorProgram Fragment Shader");
        let program = match gx::Program::from_vert_frag(&vs, &fs) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to link GL program:\n{}", s);
                panic!()
            },
        };
        program.set_label(b"SimpleColorProgram Program");

        let a_position = program.attrib_location(b"a_position\0").unwrap() as _;
        let a_color = program.attrib_location(b"a_color\0").unwrap() as _;
        let u_mvp = program.uniform_location(b"u_mvp\0").unwrap();

        Self {
            program, u_mvp, a_position, a_color,
        }
    }
    pub fn use_program(&mut self, mvp: &Mat4<f32>) {
        self.program.use_program();
        self.program.set_uniform_mat4(self.u_mvp, &mvp);
    }
}
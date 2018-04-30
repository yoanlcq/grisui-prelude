use gx;
use gx::GLResource;
use gl::types::*;
use v::{Mat4, Vec2, Vec3, Rgba, Extent2};


#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum TextureUnit {
    DebugFontAtlas = 1,
    TalkFontAtlas = 2,
}

pub fn set_active_texture(i: TextureUnit) {
    gx::set_active_texture(i as GLuint)
}


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
assert_eq_size!(simple_color_vertex_size; SimpleColorVertex, [f32; 7]);

impl SimpleColorProgram {

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


    pub fn a_position(&self) -> GLuint {
        self.a_position
    }
    pub fn a_color(&self) -> GLuint {
        self.a_color
    }
    pub fn new() -> Self {
        let vs = match gx::VertexShader::from_source(Self::VS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile vertex shader:\n{}", s);
                panic!()
            },
        };
        vs.set_label(b"SimpleColorProgram Vertex Shader");
        let fs = match gx::FragmentShader::from_source(Self::FS) {
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
    pub fn use_program(&self, mvp: &Mat4<f32>) {
        self.program.use_program();
        self.set_uniform_mvp(mvp);
    }
    pub fn set_uniform_mvp(&self, mvp: &Mat4<f32>) {
        self.program.set_uniform_mat4(self.u_mvp, &[*mvp]);
    }
}





#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TextProgram {
    program: gx::Program,
    u_glyph_rect_pos: GLint,
    u_glyph_rect_size: GLint,
    u_glyph_offset: GLint,
    u_mvp: GLint,
    u_texture: GLint,
    u_color: GLint,
    a_position: GLuint,
    a_texcoords: GLuint,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextVertex {
    pub position: Vec2<f32>,
    pub texcoords: Vec2<f32>,
}
assert_eq_size!(text_vertex_size; TextVertex, [f32; 4]);

impl TextProgram {

    const VS: &'static [u8] = b"
#version 130
uniform vec2 u_glyph_rect_pos;
uniform vec2 u_glyph_rect_size;
uniform vec2 u_glyph_offset;
uniform mat4 u_mvp;
in vec2 a_position;
in vec2 a_texcoords;
out vec2 v_texcoords;
void main() {
    vec2 pos = a_position;// + vec2(0.5, -0.5);
    pos = pos * u_glyph_rect_size + u_glyph_offset;
    gl_Position = u_mvp * vec4(pos, 0.0, 1.0);
    v_texcoords = a_texcoords * u_glyph_rect_size + u_glyph_rect_pos;
}
\0";

    const FS: &'static [u8] = b"
#version 130
uniform sampler2D u_texture;
uniform vec4 u_color;
in vec2 v_texcoords;
out vec4 f_color;
void main() {
    vec4 c = u_color;
    c.a = texture2D(u_texture, v_texcoords).r;
    f_color = c;
}
\0";


    pub fn a_position(&self) -> GLuint {
        self.a_position
    }
    pub fn a_texcoords(&self) -> GLuint {
        self.a_texcoords
    }
    pub fn new() -> Self {
        let vs = match gx::VertexShader::from_source(Self::VS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile vertex shader:\n{}", s);
                panic!()
            },
        };
        vs.set_label(b"TextProgram Vertex Shader");
        let fs = match gx::FragmentShader::from_source(Self::FS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile fragment shader:\n{}", s);
                panic!()
            },
        };
        fs.set_label(b"TextProgram Fragment Shader");
        let program = match gx::Program::from_vert_frag(&vs, &fs) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to link GL program:\n{}", s);
                panic!()
            },
        };
        program.set_label(b"TextProgram Program");

        let a_position = program.attrib_location(b"a_position\0").unwrap() as _;
        let a_texcoords = program.attrib_location(b"a_texcoords\0").unwrap() as _;
        let u_glyph_rect_pos = program.uniform_location(b"u_glyph_rect_pos\0").unwrap();
        let u_glyph_rect_size = program.uniform_location(b"u_glyph_rect_size\0").unwrap();
        let u_glyph_offset = program.uniform_location(b"u_glyph_offset\0").unwrap();
        let u_mvp = program.uniform_location(b"u_mvp\0").unwrap();
        let u_texture = program.uniform_location(b"u_texture\0").unwrap();
        let u_color = program.uniform_location(b"u_color\0").unwrap();

        Self {
            program, a_position, a_texcoords,
            u_glyph_rect_pos, u_glyph_rect_size, u_glyph_offset,
            u_mvp, u_texture, u_color,
        }
    }
    pub fn use_program(&self) {
        self.program.use_program();
        self.set_uniform_mvp(&Mat4::identity());
        self.set_uniform_texture(TextureUnit::DebugFontAtlas);
        self.set_uniform_color(Rgba::magenta());
        self.set_uniform_glyph_rect_pos(Vec2::zero());
        self.set_uniform_glyph_rect_size(Extent2::one());
        self.set_uniform_glyph_offset(Vec2::zero());
    }
    pub fn set_uniform_mvp(&self, mvp: &Mat4<f32>) {
        self.program.set_uniform_mat4(self.u_mvp, &[*mvp]);
    }
    pub fn set_uniform_texture(&self, tex: TextureUnit) {
        self.program.set_uniform_1i(self.u_texture, &[tex as GLuint as GLint]);
    }
    pub fn set_uniform_color(&self, rgba: Rgba<f32>) {
        self.program.set_uniform_4f(self.u_color, &[rgba.into_array()]);
    }
    pub fn set_uniform_glyph_rect_pos(&self, pos: Vec2<f32>) {
        self.program.set_uniform_2f(self.u_glyph_rect_pos, &[pos.into_array()]);
    }
    pub fn set_uniform_glyph_rect_size(&self, size: Extent2<f32>) {
        self.program.set_uniform_2f(self.u_glyph_rect_size, &[size.into_array()]);
    }
    pub fn set_uniform_glyph_offset(&self, offset: Vec2<f32>) {
        self.program.set_uniform_2f(self.u_glyph_offset, &[offset.into_array()]);
    }
}


#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ParticleRenderingProgram {
    program: gx::Program,
    u_mvp: GLint,
    a_position: GLuint,
    a_color: GLuint,
    a_point_size: GLuint,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParticleRenderingVertex {
    pub position: Vec3<f32>,
    pub color: Rgba<f32>,
    pub point_size: f32,
}
assert_eq_size!(particle_size; ParticleRenderingVertex, [f32; 8]);

impl ParticleRenderingProgram {

    pub const VS: &'static [u8] = b"
#version 130

uniform mat4 u_mvp;

in vec3 a_position;
in vec4 a_color;
in float a_point_size;

out vec4 v_color;

void main() {
    v_color = a_color;
    vec4 pos = u_mvp * vec4(a_position, 1);
    gl_PointSize = a_point_size;
    gl_Position = pos;
}
\0";

    const FS: &'static [u8] = b"
#version 130

in vec4 v_color;

out vec4 f_color;

void main() {
    vec2 from_center = gl_PointCoord - vec2(0.5f);
    float d = length(from_center);
    if(d > 0.5f)
        discard;
    f_color = v_color;
}
\0";

    pub fn a_position(&self) -> GLuint {
        self.a_position
    }
    pub fn a_color(&self) -> GLuint {
        self.a_color
    }
    pub fn a_point_size(&self) -> GLuint {
        self.a_point_size
    }
    pub fn new() -> Self {
        let vs = match gx::VertexShader::from_source(Self::VS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile vertex shader:\n{}", s);
                panic!(s)
            },
        };
        vs.set_label(b"ParticleRenderingProgram Vertex Shader");
        let fs = match gx::FragmentShader::from_source(Self::FS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile fragment shader:\n{}", s);
                panic!(s)
            },
        };
        fs.set_label(b"ParticleRenderingProgram Fragment Shader");
        let program = match gx::Program::from_vert_frag(&vs, &fs) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to link GL program:\n{}", s);
                panic!()
            },
        };
        program.set_label(b"ParticleRenderingProgram Program");

        let a_position = program.attrib_location(b"a_position\0").unwrap() as _;
        let a_color = program.attrib_location(b"a_color\0").unwrap() as _;
        let a_point_size = program.attrib_location(b"a_point_size\0").unwrap() as _;
        let u_mvp = program.uniform_location(b"u_mvp\0").unwrap();

        Self {
            program, u_mvp, a_position, a_color, a_point_size,
        }
    }
    pub fn use_program(&self, mvp: &Mat4<f32>) {
        self.program.use_program();
        self.set_uniform_mvp(mvp);
    }
    pub fn set_uniform_mvp(&self, mvp: &Mat4<f32>) {
        self.program.set_uniform_mat4(self.u_mvp, &[*mvp]);
    }
}



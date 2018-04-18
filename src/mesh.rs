use std::ffi::CString;
use std::ptr;
use std::mem::size_of;
use gl;
use gl::types::*;
use gx;
use gx::GLResource;
use grx;
use v::{Vec2, Vec3, Rgba, Mat4, Quaternion};
use transform::Transform3D;

/*
impl Mesh {
    pub fn update_vbo(&self, gx::buffer::Usage) {
        self.vbo.bind();
        self.vbo.set_data(&self.vertices, self.update_hint);
    }
    pub fn from_vertices(
        prog: &grx::SimpleColorProgram,
        label: &str,
        update_hint: gx::UpdateHint,
        gl_topology: GLenum,
        vertices: Vec<grx::SimpleColorVertex>
    ) -> Self
    {
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
        gx::Vao::unbind();

        Self {
            vertices, gl_topology, vbo, vao, update_hint,
        }
    }
    pub fn new_colored_quad(
        prog: &grx::SimpleColorProgram,
        label: &str,
        update_hint: gx::UpdateHint,
        bl: Rgba<f32>,
        br: Rgba<f32>,
        tr: Rgba<f32>,
        tl: Rgba<f32>,
        s: f32
    ) -> Self
    {
        let vertices = vec![
            grx::SimpleColorVertex { position: Vec3::new(-s, -s, 0.), color: bl },
            grx::SimpleColorVertex { position: Vec3::new( s, -s, 0.), color: br },
            grx::SimpleColorVertex { position: Vec3::new( s,  s, 0.), color: tr },
            grx::SimpleColorVertex { position: Vec3::new(-s,  s, 0.), color: tl },
        ];
        Self::from_vertices(prog, label, update_hint, gl::TRIANGLE_FAN, vertices)
    }
    pub fn new_filled_unit_quad(prog: &grx::SimpleColorProgram, label: &str, update_hint: gx::UpdateHint, color: Rgba<f32>) -> Self {
        Self::new_filled_quad(prog, label, update_hint, color, 0.5)
    }
    pub fn new_filled_quad(prog: &grx::SimpleColorProgram, label: &str, update_hint: gx::UpdateHint, color: Rgba<f32>, s: f32) -> Self {
        Self::new_colored_quad(prog, label, update_hint, color, color, color, color, s)
    }
    pub fn new_unit_disk(prog: &grx::SimpleColorProgram, label: &str, update_hint: gx::UpdateHint, vcount: usize, color: Rgba<f32>) -> Self {
        assert!(vcount > 2);
        let mut vertices = Vec::with_capacity(vcount+2); // +1 for center, and +1 for closing vertex duplicate
        vertices.push(grx::SimpleColorVertex { position: Vec3::zero(), color });
        for i in 0..(vcount+1) {
            use ::std::f32::consts::PI;
            let a = 2.*PI * (i as f32 / (vcount as f32));
            let position = Vec3::new(a.cos(), a.sin(), 0.);
            let v = grx::SimpleColorVertex { position, color };
            vertices.push(v);
        }
        Self::from_vertices(prog, label, update_hint, gl::TRIANGLE_FAN, vertices)
    }
    pub fn new_star_polyfanmask(prog: &grx::SimpleColorProgram, label: &str, update_hint: gx::UpdateHint) -> Self {
        let color = Rgba::magenta();
        let mut vertices = vec![
            grx::SimpleColorVertex { position: Vec3::zero(), color },
        ];
        let r = 0.5;
        let vcount = 5;
        let offset = Vec3::new(r, r, 0.);
        for i in 0..vcount {
            use ::std::f32::consts::PI;
            let a = 2.*PI * (i as f32 / (vcount as f32));
            let position = Vec3::new(a.cos(), a.sin(), 0.) * r + offset;
            let v = grx::SimpleColorVertex { position, color };
            vertices.push(v);
            let a = a + PI / (vcount as f32);
            let position = Vec3::new(a.cos(), a.sin(), 0.) * r / 2. + offset;
            let v = grx::SimpleColorVertex { position, color };
            vertices.push(v);
        }
        let v = vertices[1];
        vertices.push(v);
        Self::from_vertices(prog, label, update_hint, gl::TRIANGLE_FAN, vertices)
    }
    pub fn new_gradient_strip(
        prog: &grx::SimpleColorProgram, label: &str, update_hint: gx::UpdateHint,
        left: (Vec3<f32>, Rgba<f32>),
        right: (Vec3<f32>, Rgba<f32>)
    ) -> Self 
    {
        let b = 1024_f32;
        let s = 0.5_f32;
        let mut vertices = [
            (Vec3::new(-b,  b, 0.), left.1),
            (Vec3::new(-b, -b, 0.), left.1),
            (Vec3::new(-s,  b, 0.), left.1),
            (Vec3::new(-s, -b, 0.), left.1),
            (Vec3::new( s,  b, 0.), right.1),
            (Vec3::new( s, -b, 0.), right.1),
            (Vec3::new( b,  b, 0.), right.1),
            (Vec3::new( b, -b, 0.), right.1),
        ];

        let (mut p0, mut p1) = (left.0, right.0);
        p0.z = 0.;
        p1.z = 0.;
        let dir = (p1 - p0).normalized();
        let rotation_z = dir.y.atan2(dir.x);
        let position = (p0 + p1) / 2.;
        let scale = Vec3::distance(p0, p1);
        let mut scale = Vec3::broadcast(scale);
        scale.z = 1.;
        let m = Mat4::from(Transform3D {
            position, scale,
            orientation: Quaternion::rotation_z(rotation_z),
        });

        for v in &mut vertices {
            v.0 = m.mul_point(v.0);
            v.0.z = 0.;
        }

        let vertices: Vec<_> = vertices.iter().map(|&(position, color)| {
            grx::SimpleColorVertex { position, color }
        }).collect();
        Self::from_vertices(prog, label, update_hint, gl::TRIANGLE_STRIP, vertices)
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

        gx::Vao::unbind();

        Self {
            vertices, gl_topology, vbo, vao, update_hint,
        }
    }
}


#[derive(Debug)]
pub struct Particles {
    pub vertices: Vec<grx::ParticleRenderingVertex>,
    pub vao: gx::Vao,
    pub vbo: gx::Vbo,
}

impl Particles {
    pub fn update_vbo(&self) {
        self.vbo.bind();
        self.vbo.set_data(&self.vertices, gx::UpdateHint::Often);
    }
    pub fn from_vertices(
        prog: &grx::ParticleRenderingProgram,
        label: &str,
        vertices: Vec<grx::ParticleRenderingVertex>
    ) -> Self
    {
        let vao = gx::Vao::new();
        let vbo = gx::Vbo::new();
        vbo.bind();
        vbo.set_data(&vertices, gx::UpdateHint::Often);
        vao.bind();
        vbo.bind();
        unsafe {
            gl::EnableVertexAttribArray(prog.a_position());
            gl::EnableVertexAttribArray(prog.a_color());
            gl::EnableVertexAttribArray(prog.a_point_size());
            gl::VertexAttribPointer(
                prog.a_position(), 3, gl::FLOAT,
                gl::FALSE as _, size_of::<grx::ParticleRenderingVertex>() as _,
                ptr::null()
            );
            gl::VertexAttribPointer(
                prog.a_color(), 4, gl::FLOAT,
                gl::FALSE as _, size_of::<grx::ParticleRenderingVertex>() as _,
                ptr::null::<GLvoid>().offset(3*size_of::<f32>() as isize)
            );
            gl::VertexAttribPointer(
                prog.a_point_size(), 1, gl::FLOAT,
                gl::FALSE as _, size_of::<grx::ParticleRenderingVertex>() as _,
                ptr::null::<GLvoid>().offset(7*size_of::<f32>() as isize)
            );
        }
        vao.set_label(&CString::new(label.to_owned() + " VAO").unwrap().into_bytes_with_nul());
        vbo.set_label(&CString::new(label.to_owned() + " VBO").unwrap().into_bytes_with_nul());

        gx::Vao::unbind();

        Self {
            vertices, vbo, vao,
        }
    }
}
*/

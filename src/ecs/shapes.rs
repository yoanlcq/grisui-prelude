use std::time::Duration;
use std::mem;
use std::ops::{Deref, DerefMut};
use super::eid::*;
use events::Sdl2EventSubscriber;
use game::{PhysicsUpdate, GfxUpdate};
use v::{self, Vec3};
use gx;

type Curve = v::CubicBezier2<f32>;
type Color = v::Rgba<f32>;

#[derive(Debug, Default, PartialEq)]
pub struct Shapes {
    shapes: EIDMap<Shape>,
    meshes: EIDMap<Mesh>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Shape {
    pub curves: Vec<Curve>,
    pub fill: Color,
}

#[derive(Debug, PartialEq)]
pub struct Mesh {
    vertices: VertexData,
    vbo_usage: gx::buffer::Usage,
    vbo: gx::Buffer,
    vao: gx::VertexArray,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VertexData {
    pub positions: Vec<Vec3<f32>>,
    pub colors: Vec<Color>,
}

impl VertexData {
    pub fn total_size(&self) -> usize {
        self.positions_size() + self.colors_size()
    }
    pub fn positions_size(&self) -> usize {
        mem::size_of_val(self.positions.as_slice())
    }
    pub fn colors_size(&self) -> usize {
        mem::size_of_val(self.colors.as_slice())
    }
    pub fn positions_byte_offset(&self) -> usize {
        0
    }
    pub fn colors_byte_offset(&self) -> usize {
        self.positions_size()
    }
}

impl Mesh {
    pub fn from_vertices(vertices: VertexData, vbo_usage: gx::buffer::Usage) -> Self {
        let vbo = gx::Buffer::new();
        let vao = gx::VertexArray::new();

        gx::vertex_array::bind(&vao);

        // TODO attribute ptr

        let s = Self {
            vertices, vbo_usage, vbo, vao,
        };
        s.update_vbo();
        gx::vertex_array::unbind();
        s
    }
    pub fn edit_vertices<F>(&mut self, mut f: F) where F: FnMut(&mut VertexData) {
        f(&mut self.vertices);
        self.update_vbo();
    }
    fn update_vbo(&self) {
        use gx::state::buffer;
        let target = gx::buffer::Target::Array;
        let vertices = &self.vertices;

        buffer::bind(target, &vbo);
        buffer::resize(target, vertices.total_size(), self.vbo_usage);
        buffer::subdata(target, vertices.positions_byte_offset(), &vertices.positions);
        buffer::subdata(target, vertices.colors_byte_offset(), &vertices.colors);
        buffer::unbind();
    }
}


impl Deref for Shapes {
    type Target = EIDMap<Shape>;
    fn deref(&self) -> &Self::Target {
        &self.shapes
    }
}
impl DerefMut for Shapes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shapes
    }
}

impl Sdl2EventSubscriber for Shapes {}
impl PhysicsUpdate for Shapes {
    fn replace_previous_state_by_current(&mut self) {}
    fn integrate(&mut self, _t: Duration, _dt: Duration) {}
}
impl GfxUpdate for Shapes {
    fn compute_gfx_state_via_lerp_previous_current(&mut self, _alpha: f64) {}
    fn render(&mut self) {}
}

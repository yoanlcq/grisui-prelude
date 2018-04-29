use std::io;
use v::{Vec3, Rgba};
use mesh::{vertex_array, color_mesh::{self, Vertex}};
use gx::BufferUsage;

type ColorVertexArray = vertex_array::VertexArray<color_mesh::Program>;

#[derive(Debug)]
pub struct Shape {
    pub vertices: ColorVertexArray,
    pub is_path_closed: bool,
}

impl Shape {
    pub fn new(color_mesh_gl_program: &color_mesh::Program) -> Self {
        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw, vec![]
        );
        Self { vertices, is_path_closed: false }
    }
    // M = moveto
    // L = lineto
    // C = curveto
    // Q = quadratic Bézier curve
    // Z = closepath
    // Note: All of the commands above can also be expressed with lower letters. Capital letters means absolutely positioned, lower cases means relatively positioned.
    pub fn save(&self, f: &mut io::Write) -> io::Result<()> {
        for (i, v) in self.vertices.vertices.iter().enumerate() {
            let letter = if i == 0 { 'M' } else { 'L' };
            let pos = v.position;
            writeln!(f, "{} {} {}", letter, pos.x, pos.y)?;
        }
        if self.is_path_closed {
            writeln!(f, "Z")?;
        }
        Ok(())
    }
    pub fn load(color_mesh_gl_program: &color_mesh::Program, f: &mut io::Read) -> io::Result<Self> {
        let data = {
            let mut buf = String::new();
            f.read_to_string(&mut buf).unwrap();
            buf
        };

        let mut vertices = vec![];
        let mut is_path_closed = false;

        let mut words = data.split_whitespace();
        while let Some(cmd) = words.next() {
            match cmd {
                "M" | "L" => {
                    let x: f32 = words.next().unwrap().parse().unwrap();
                    let y: f32 = words.next().unwrap().parse().unwrap();
                    let position = Vec3 { x, y, z: 0. };
                    let color = Rgba::yellow();
                    vertices.push(Vertex { position, color });
                },
                "Z" | "z" => is_path_closed = true,
                _ => unimplemented!{},
            };
        }

        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw,
            vertices,
        );
        Ok(Self { vertices, is_path_closed })
    }
}

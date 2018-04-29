use std::io;
use v::{Vec3, Rgba};
use mesh::{vertex_array, color_mesh::{self, Vertex}};
use gx::BufferUsage;

type ColorVertexArray = vertex_array::VertexArray<color_mesh::Program>;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub stroke_thickness: f32,
    pub stroke_color: Rgba<f32>,
    pub fill_color: Rgba<f32>,
}

#[derive(Debug)]
pub struct Shape {
    pub vertices: ColorVertexArray,
    pub fill_color_strip: ColorVertexArray,
    pub is_path_closed: bool,
    pub style: Style,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            stroke_thickness: 2.,
            stroke_color: Rgba::black(),
            fill_color: Rgba::magenta(),
        }
    }
}

fn create_fill_color_strip(color_mesh_gl_program: &color_mesh::Program, color: Rgba<f32>) -> ColorVertexArray {
    ColorVertexArray::from_vertices(
        &color_mesh_gl_program, "Some Shape Fill Color Strip", BufferUsage::DynamicDraw,
        vec![
            Vertex { position: Vec3::new(-1.,  1., 0.), color, },
            Vertex { position: Vec3::new(-1., -1., 0.), color, },
            Vertex { position: Vec3::new( 1.,  1., 0.), color, },
            Vertex { position: Vec3::new( 1., -1., 0.), color, },
        ]
    )
}

impl Shape {
    pub fn new(color_mesh_gl_program: &color_mesh::Program) -> Self {
        let style = Style::default();
        let fill_color_strip = create_fill_color_strip(color_mesh_gl_program, style.fill_color);
        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw, vec![]
        );
        Self { style, vertices, fill_color_strip, is_path_closed: false, }
    }
    // M = moveto
    // L = lineto
    // C = curveto
    // Q = quadratic Bézier curve
    // Z = closepath
    // Note: All of the commands above can also be expressed with lower letters. Capital letters means absolutely positioned, lower cases means relatively positioned.
    pub fn save(&self, f: &mut io::Write) -> io::Result<()> {
        let &Style {
            stroke_thickness, stroke_color, fill_color
        } = &self.style;
        writeln!(f, "stroke_thickness {}", stroke_thickness)?;
        writeln!(f, "stroke_color {} {} {} {}", stroke_color.r, stroke_color.g, stroke_color.b, stroke_color.a)?;
        writeln!(f, "fill_color {} {} {} {}", fill_color.r, fill_color.g, fill_color.b, fill_color.a)?;
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
        let mut style = Style::default();

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
                "stroke_thickness" => style.stroke_thickness = words.next().unwrap().parse().unwrap(),
                "stroke_color" => {
                    let r: f32 = words.next().unwrap().parse().unwrap();
                    let g: f32 = words.next().unwrap().parse().unwrap();
                    let b: f32 = words.next().unwrap().parse().unwrap();
                    let a: f32 = words.next().unwrap().parse().unwrap();
                    style.stroke_color = Rgba { r, g, b, a };
                },
                "fill_color" => {
                    let r: f32 = words.next().unwrap().parse().unwrap();
                    let g: f32 = words.next().unwrap().parse().unwrap();
                    let b: f32 = words.next().unwrap().parse().unwrap();
                    let a: f32 = words.next().unwrap().parse().unwrap();
                    style.fill_color = Rgba { r, g, b, a };
                },
                _ => unimplemented!{},
            };
        }

        // NOTE: Do this last; "stroke_color" might be present last in the file.
        for v in &mut vertices {
            v.color = style.stroke_color;
        }

        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw,
            vertices,
        );
        let fill_color_strip = create_fill_color_strip(color_mesh_gl_program, style.fill_color);
        Ok(Self { vertices, fill_color_strip, is_path_closed, style })
    }
}

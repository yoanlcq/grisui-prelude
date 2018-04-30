use std::io;
use v::{Vec2, Vec3, Rgba, CubicBezier2, QuadraticBezier2};
use mesh::{vertex_array, color_mesh::{self, Vertex}};
use gx::BufferUsage;

type ColorVertexArray = vertex_array::VertexArray<color_mesh::Program>;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub stroke_thickness: f32,
    pub stroke_color: Rgba<f32>,
    pub fill_color: Rgba<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PathCmd {
    Line { end: Vec2<f32> },
    Cubic { ctrl0: Vec2<f32>, ctrl1: Vec2<f32>, end: Vec2<f32> },
    Quadratic { ctrl: Vec2<f32>, end: Vec2<f32> },
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Path {
    pub is_closed: bool,
    pub start: Vec2<f32>,
    pub cmds: Vec<PathCmd>,
}

#[derive(Debug)]
pub struct Shape {
    pub vertices: ColorVertexArray,
    pub fill_color_strip: ColorVertexArray,
    pub style: Style,
    pub path: Path,
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

impl Path {
    pub const DEFAULT_STEPS: u32 = 32;
    pub fn generate_vertex_positions(&self, steps: u32) -> Vec<Vec2<f32>> {
        let mut vertices = vec![self.start];
        for cmd in &self.cmds {
            match *cmd {
                PathCmd::Line { end } => vertices.push(end),
                PathCmd::Quadratic { ctrl, end } => {
                    let start = *vertices.last().unwrap();
                    let c = QuadraticBezier2 { start, ctrl, end };
                    for i in 0..(steps+1) {
                        let t = i as f32 / steps as f32;
                        vertices.push(c.evaluate(t));
                    }
                },
                PathCmd::Cubic { ctrl0, ctrl1, end } => {
                    let start = *vertices.last().unwrap();
                    let c = CubicBezier2 { start, ctrl0, ctrl1, end };
                    for i in 0..(steps+1) {
                        let t = i as f32 / steps as f32;
                        vertices.push(c.evaluate(t));
                    }
                },
            };
        }
        vertices
    }
    pub fn generate_vertices(&self, steps: u32, color: Rgba<f32>) -> Vec<Vertex> {
        self.generate_vertex_positions(steps).into_iter().map(|position| Vertex {
            position: position.into(),
            color,
        }).collect()
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
        let path = Path::default();
        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw,
            path.generate_vertices(Path::DEFAULT_STEPS, style.stroke_color)
        );
        Self { style, path, vertices, fill_color_strip, }
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
        writeln!(f, "M {} {}", self.path.start.x, self.path.start.y)?;
        for cmd in self.path.cmds.iter() {
            match *cmd {
                PathCmd::Line { end } => writeln!(f, "L {} {}", end.x, end.y)?,
                PathCmd::Quadratic { ctrl, end } => writeln!(f, "Q {} {} {} {}", ctrl.x, ctrl.y, end.x, end.y)?,
                PathCmd::Cubic { ctrl0, ctrl1, end } => writeln!(f, "C {} {} {} {} {} {}", ctrl0.x, ctrl0.y, ctrl1.x, ctrl1.y, end.x, end.y)?,
            };
        }
        if self.path.is_closed {
            writeln!(f, "Z")?;
        }
        Ok(())
    }

    pub fn load(color_mesh_gl_program: &color_mesh::Program, f: &mut io::Read) -> io::Result<Self> {
        let data = {
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            buf
        };

        let mut path = Path::default();
        let mut style = Style::default();

        let mut words = data.split_whitespace();
        while let Some(cmd) = words.next() {
            match cmd {
                "M" => {
                    // XXX: Assuming there's only one 'M' command in the file, ever.
                    let x: f32 = words.next().unwrap().parse().unwrap();
                    let y: f32 = words.next().unwrap().parse().unwrap();
                    path.start = Vec2 { x, y };
                },
                "L" => {
                    let x: f32 = words.next().unwrap().parse().unwrap();
                    let y: f32 = words.next().unwrap().parse().unwrap();
                    path.cmds.push(PathCmd::Line { end: Vec2 { x, y } });
                },
                "Q" => {
                    let mut end = Vec2::zero();
                    let mut ctrl = Vec2::zero();
                    ctrl.x = words.next().unwrap().parse().unwrap();
                    ctrl.y = words.next().unwrap().parse().unwrap();
                    end.x = words.next().unwrap().parse().unwrap();
                    end.y = words.next().unwrap().parse().unwrap();
                    path.cmds.push(PathCmd::Quadratic { ctrl, end });
                },
                "C" => {
                    let mut end = Vec2::zero();
                    let mut ctrl0 = Vec2::zero();
                    let mut ctrl1 = Vec2::zero();
                    ctrl0.x = words.next().unwrap().parse().unwrap();
                    ctrl0.y = words.next().unwrap().parse().unwrap();
                    ctrl1.x = words.next().unwrap().parse().unwrap();
                    ctrl1.y = words.next().unwrap().parse().unwrap();
                    end.x = words.next().unwrap().parse().unwrap();
                    end.y = words.next().unwrap().parse().unwrap();
                    path.cmds.push(PathCmd::Cubic { ctrl0, ctrl1, end });
                },
                "Z" | "z" => path.is_closed = true,
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

        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw,
            path.generate_vertices(Path::DEFAULT_STEPS, style.stroke_color)
        );
        let fill_color_strip = create_fill_color_strip(color_mesh_gl_program, style.fill_color);

        Ok(Self { path, style, vertices, fill_color_strip, })
    }
}

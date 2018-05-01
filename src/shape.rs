use std::io;
use std::ops::Range;
use v::{Vec2, Vec3, Rgba, CubicBezier2, QuadraticBezier2};
use mesh::{vertex_array, color_mesh::{self, Vertex}};
use gx::BufferUsage;

type ColorVertexArray = vertex_array::VertexArray<color_mesh::Program>;

pub type GradientEnd = Vertex;
pub type Gradient = Range<GradientEnd>;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub stroke_thickness: f32,
    pub stroke_color: Rgba<f32>,
    pub fill_color: Rgba<f32>,
    pub fill_gradient: Gradient,
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
    pub solid_fill_strip: ColorVertexArray,
    pub gradient_fill_strip: ColorVertexArray,
    pub style: Style,
    pub path: Path,
}

impl Default for Style {
    fn default() -> Self {
        let grad_start = GradientEnd { position: -Vec3::unit_x(), color: Rgba::green(), };
        let grad_end   = GradientEnd { position:  Vec3::unit_x(), color: Rgba::magenta(), };
        Self {
            stroke_thickness: 2.,
            stroke_color: Rgba::black(),
            fill_color: Rgba::yellow(),
            fill_gradient: grad_start .. grad_end,
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

fn create_solid_fill_strip(color_mesh_gl_program: &color_mesh::Program, color: Rgba<f32>) -> ColorVertexArray {
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

fn create_gradient_fill_strip(color_mesh_gl_program: &color_mesh::Program, gradient: &Gradient) -> ColorVertexArray {
    let &Gradient { ref start, ref end } = gradient;
    let b = 1024_f32;
    let s = 0.5_f32;
    let mut vertices = [
        (Vec3::new(-b,  b, 0.), start.color),
        (Vec3::new(-b, -b, 0.), start.color),
        (Vec3::new(-s,  b, 0.), start.color),
        (Vec3::new(-s, -b, 0.), start.color),
        (Vec3::new( s,  b, 0.), end.color),
        (Vec3::new( s, -b, 0.), end.color),
        (Vec3::new( b,  b, 0.), end.color),
        (Vec3::new( b, -b, 0.), end.color),
    ];

    let (mut p0, mut p1) = (start.position, end.position);
    p0.z = 0.;
    p1.z = 0.;
    let dir = (p1 - p0).normalized();
    let rotation_z = dir.y.atan2(dir.x);
    let position = (p0 + p1) / 2.;
    let scale = Vec3::distance(p0, p1);
    let mut scale = Vec3::broadcast(scale);
    scale.z = 1.;
    let m = ::v::Mat4::from(::v::Transform {
        position, scale,
        orientation: ::v::Quaternion::rotation_z(rotation_z),
    });

    for v in &mut vertices {
        v.0 = m.mul_point(v.0);
        v.0.z = 0.;
    }

    ColorVertexArray::from_vertices(
        &color_mesh_gl_program, "Some Shape Fill Gradient Strip", BufferUsage::DynamicDraw,
        vertices.iter().map(|&(position, color)| Vertex { position, color }).collect()
    )
}

impl Shape {
    pub fn new(color_mesh_gl_program: &color_mesh::Program) -> Self {
        let style = Style::default();
        let solid_fill_strip = create_solid_fill_strip(color_mesh_gl_program, style.fill_color);
        let gradient_fill_strip = create_gradient_fill_strip(color_mesh_gl_program, &style.fill_gradient);
        let path = Path::default();
        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw,
            path.generate_vertices(Path::DEFAULT_STEPS, style.stroke_color)
        );
        Self { style, path, vertices, solid_fill_strip, gradient_fill_strip, }
    }
    // M = moveto
    // L = lineto
    // C = curveto
    // Q = quadratic Bézier curve
    // Z = closepath
    // Note: All of the commands above can also be expressed with lower letters. Capital letters means absolutely positioned, lower cases means relatively positioned.
    pub fn save(&self, f: &mut io::Write) -> io::Result<()> {
        let &Style {
            stroke_thickness, stroke_color, fill_color, ref fill_gradient,
        } = &self.style;
        writeln!(f, "stroke_thickness {}", stroke_thickness)?;
        writeln!(f, "stroke_color {} {} {} {}", stroke_color.r, stroke_color.g, stroke_color.b, stroke_color.a)?;
        writeln!(f, "fill_color {} {} {} {}", fill_color.r, fill_color.g, fill_color.b, fill_color.a)?;
        {
            let Rgba { r, g, b, a } = fill_gradient.start.color;
            writeln!(f, "fill_gradient_start_color {} {} {} {}", r, g, b, a)?;
        }
        {
            let Rgba { r, g, b, a } = fill_gradient.end.color;
            writeln!(f, "fill_gradient_end_color {} {} {} {}", r, g, b, a)?;
        }
        {
            let Vec3 { x, y, z: _ } = fill_gradient.start.position;
            writeln!(f, "fill_gradient_start_position {} {}", x, y)?;
        }
        {
            let Vec3 { x, y, z: _ } = fill_gradient.end.position;
            writeln!(f, "fill_gradient_end_position {} {}", x, y)?;
        }
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
                "fill_gradient_start_color" => {
                    let r: f32 = words.next().unwrap().parse().unwrap();
                    let g: f32 = words.next().unwrap().parse().unwrap();
                    let b: f32 = words.next().unwrap().parse().unwrap();
                    let a: f32 = words.next().unwrap().parse().unwrap();
                    style.fill_gradient.start.color = Rgba { r, g, b, a };
                },
                "fill_gradient_end_color" => {
                    let r: f32 = words.next().unwrap().parse().unwrap();
                    let g: f32 = words.next().unwrap().parse().unwrap();
                    let b: f32 = words.next().unwrap().parse().unwrap();
                    let a: f32 = words.next().unwrap().parse().unwrap();
                    style.fill_gradient.end.color = Rgba { r, g, b, a };
                },
                "fill_gradient_start_position" => {
                    let x: f32 = words.next().unwrap().parse().unwrap();
                    let y: f32 = words.next().unwrap().parse().unwrap();
                    style.fill_gradient.start.position = Vec2 { x, y }.into();
                },
                "fill_gradient_end_position" => {
                    let x: f32 = words.next().unwrap().parse().unwrap();
                    let y: f32 = words.next().unwrap().parse().unwrap();
                    style.fill_gradient.end.position = Vec2 { x, y }.into();
                },
                whoops @ _ => panic!("Unknown command `{}`", whoops),
            };
        }

        let vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Some Shape Vertices", BufferUsage::DynamicDraw,
            path.generate_vertices(Path::DEFAULT_STEPS, style.stroke_color)
        );
        let solid_fill_strip = create_solid_fill_strip(color_mesh_gl_program, style.fill_color);
        let gradient_fill_strip = create_gradient_fill_strip(color_mesh_gl_program, &style.fill_gradient);

        Ok(Self { path, style, vertices, solid_fill_strip, gradient_fill_strip, })
    }
}

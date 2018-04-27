use std::ptr;
use std::fs::File;
use gl;
use gx::{Object, BufferUsage};
use system::*;
use v::{Vec3, Rgba, Mat4};
use camera::OrthoCamera2D;
use mesh::{self, vertex_array, color_mesh::{self, Vertex}};
use duration_ext::DurationExt;
use text::Text;
use font::FontID;

type ColorVertexArray = vertex_array::VertexArray<color_mesh::Program>;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Hsva<T> {
    pub h: T,
    pub s: T,
    pub v: T,
    pub a: T,
}

impl From<Rgba<f32>> for Hsva<f32> {
    fn from(rgba: Rgba<f32>) -> Self {
        use ::v::partial_max as max;
        use ::v::partial_min as min;

        let Rgba { r, g, b, a } = rgba;
        let cmax = max(max(r, g), b);
        let cmin = min(min(r, g), b);
        let delta = cmax - cmin;
        let v = cmax;

        let epsilon = 0.0001;
        
        if delta <= epsilon || cmax <= epsilon {
            return Hsva { h: 0., s: 0., v, a };
        }

        let s = delta / cmax;

        let mut h = if r >= cmax { 0. + (g-b) / delta }
               else if g >= cmax { 2. + (b-r) / delta }
               else              { 4. + (r-g) / delta };

        if h < 0. {
            h += 6.;
        }
        Hsva { h, s, v, a }
    }
}

fn rgba_from_hsva(hsva: Hsva<f32>) -> Rgba<f32> {
    let Hsva { h, s, v, a } = hsva;
    let c = v * s; // chroma
    let x = c * (1. - (h % 2. - 1.).abs());
    let (mut r, mut g, mut b);
    use ::v::Wrap;
    match (h as i32).wrapped(6) {
        0 => { r = c ; g = x ; b = 0.; },
        1 => { r = x ; g = c ; b = 0.; },
        2 => { r = 0.; g = c ; b = x ; },
        3 => { r = 0.; g = x ; b = c ; },
        4 => { r = x ; g = 0.; b = c ; },
        5 => { r = c ; g = 0.; b = x ; },
        _ => unreachable!{},
    };
    let m = v - c;
    r += m; g += m; b += m;
    Rgba { r, g, b, a }
}


#[derive(Debug)]
pub struct HsvaSliders {
    hsva: Hsva<f32>,
    cursor_lines: ColorVertexArray,
    strips: Hsva<ColorVertexArray>,
    strip_heights: Hsva<f32>,
    strip_y: Hsva<f32>,
}

impl HsvaSliders {
    fn new(color_mesh_gl_program: &color_mesh::Program) -> Self {
        let hsva = Hsva { h: 3., s: 1., v: 1., a: 1. };
        let rgba = rgba_from_hsva(hsva);
        let strip_heights = Hsva { h: 0.25, s: 0.25, v: 0.25, a: 0.25 };
        let strip_y = Hsva {
            h: strip_heights.a + strip_heights.v + strip_heights.s,
            s: strip_heights.a + strip_heights.v,
            v: strip_heights.a,
            a: 0.,
        };
        let hue_strip = {
            let steps = 32;
            let mut vertices = Vec::with_capacity(steps * 2);
            for hue in 0..steps {
                let progress = hue as f32 / (steps-1) as f32;
                let position = Vec3::new(progress, strip_y.h, 0.);
                let color = rgba_from_hsva(Hsva { h: progress * 6., s: 1., v: 1., a: 1. });
                vertices.push(Vertex { color, position: position + Vec3::unit_y() * strip_heights.h, });
                vertices.push(Vertex { color, position, });
            }
            ColorVertexArray::from_vertices(&color_mesh_gl_program, "HsvaSliders Hue Slider", BufferUsage::StaticDraw, vertices)
        };
        let sat_strip = {
            let mut saturated = hsva;
            let mut not_saturated = hsva;
            saturated.s = 1.;
            not_saturated.s = 0.;
            let saturated = rgba_from_hsva(saturated);
            let not_saturated = rgba_from_hsva(not_saturated);
            ColorVertexArray::from_vertices(
                &color_mesh_gl_program, "HsvaSliders Saturation Slider", BufferUsage::DynamicDraw,
                vec![
                    Vertex { position: Vec3::new(0., strip_y.s + strip_heights.s * 0., 0.), color: not_saturated, },
                    Vertex { position: Vec3::new(1., strip_y.s + strip_heights.s * 0., 0.), color: saturated, },
                    Vertex { position: Vec3::new(0., strip_y.s + strip_heights.s * 1., 0.), color: not_saturated, },
                    Vertex { position: Vec3::new(1., strip_y.s + strip_heights.s * 1., 0.), color: saturated, },
                ]
            )
        };
        let val_strip = {
            let mut hi_value = hsva;
            let mut lo_value = hsva;
            hi_value.v = 1.;
            lo_value.v = 0.;
            let hi_value = rgba_from_hsva(hi_value);
            let lo_value = rgba_from_hsva(lo_value);
            ColorVertexArray::from_vertices(
                &color_mesh_gl_program, "HsvaSliders Value Slider", BufferUsage::DynamicDraw,
                vec![
                    Vertex { position: Vec3::new(0., strip_y.v + strip_heights.v * 0., 0.), color: lo_value, },
                    Vertex { position: Vec3::new(1., strip_y.v + strip_heights.v * 0., 0.), color: hi_value, },
                    Vertex { position: Vec3::new(0., strip_y.v + strip_heights.v * 1., 0.), color: lo_value, },
                    Vertex { position: Vec3::new(1., strip_y.v + strip_heights.v * 1., 0.), color: hi_value, },
                ]
            )
        };
        let alpha_strip = {
            ColorVertexArray::from_vertices(
                &color_mesh_gl_program, "HsvaSliders Alpha Slider", BufferUsage::DynamicDraw,
                vec![
                    Vertex { position: Vec3::new(0., strip_y.a + strip_heights.a * 0., 0.), color: Rgba::from_transparent(rgba), },
                    Vertex { position: Vec3::new(1., strip_y.a + strip_heights.a * 0., 0.), color: Rgba::from_opaque(rgba), },
                    Vertex { position: Vec3::new(0., strip_y.a + strip_heights.a * 1., 0.), color: Rgba::from_transparent(rgba), },
                    Vertex { position: Vec3::new(1., strip_y.a + strip_heights.a * 1., 0.), color: Rgba::from_opaque(rgba), },
                ]
            )
        };
        let cursor_lines = {
            let Hsva { h, s, v, a } = hsva;
            let h = h / 6.;
            ColorVertexArray::from_vertices(
                &color_mesh_gl_program, "HsvaSliders Cursor Lines", BufferUsage::DynamicDraw,
                vec![
                    Vertex { position: Vec3::new(h, strip_y.h + strip_heights.h * 0., 0.), color: Rgba::black(), },
                    Vertex { position: Vec3::new(h, strip_y.h + strip_heights.h * 1., 0.), color: Rgba::black(), },
                    Vertex { position: Vec3::new(s, strip_y.s + strip_heights.s * 0., 0.), color: Rgba::black(), },
                    Vertex { position: Vec3::new(s, strip_y.s + strip_heights.s * 1., 0.), color: Rgba::black(), },
                    Vertex { position: Vec3::new(v, strip_y.v + strip_heights.v * 0., 0.), color: Rgba::black(), },
                    Vertex { position: Vec3::new(v, strip_y.v + strip_heights.v * 1., 0.), color: Rgba::black(), },
                    Vertex { position: Vec3::new(a, strip_y.a + strip_heights.a * 0., 0.), color: Rgba::black(), },
                    Vertex { position: Vec3::new(a, strip_y.a + strip_heights.a * 1., 0.), color: Rgba::black(), },
                ]
            )
        };
        let mut slf = Self {
            hsva, strip_heights, strip_y,
            cursor_lines,
            strips: Hsva {
                h: hue_strip,
                s: sat_strip,
                v: val_strip,
                a: alpha_strip,
            },
        };
        slf.update_colors_gl();
        slf
    }
    fn update_colors_gl(&mut self) {
        let hsva = self.hsva;
        let lo_sat = rgba_from_hsva(Hsva { a: 1., s: 0., .. hsva });
        let hi_sat = rgba_from_hsva(Hsva { a: 1., s: 1., .. hsva });
        let lo_val = rgba_from_hsva(Hsva { a: 1., v: 0., .. hsva });
        let hi_val = rgba_from_hsva(Hsva { a: 1., v: 1., .. hsva });
        let rgba = rgba_from_hsva(self.hsva);
        let lo_alpha = Rgba { a: 0., .. rgba };
        let hi_alpha = Rgba { a: 1., .. rgba };

        self.strips.s.vertices[0].color = lo_sat;
        self.strips.s.vertices[1].color = hi_sat;
        self.strips.s.vertices[2].color = lo_sat;
        self.strips.s.vertices[3].color = hi_sat;
        self.strips.v.vertices[0].color = lo_val;
        self.strips.v.vertices[1].color = hi_val;
        self.strips.v.vertices[2].color = lo_val;
        self.strips.v.vertices[3].color = hi_val;
        self.strips.a.vertices[0].color = lo_alpha;
        self.strips.a.vertices[1].color = hi_alpha;
        self.strips.a.vertices[2].color = lo_alpha;
        self.strips.a.vertices[3].color = hi_alpha;
        self.strips.s.update_vbo_range(0..4);
        self.strips.v.update_vbo_range(0..4);
        self.strips.a.update_vbo_range(0..4);
    }
    fn update_cursor_lines_gl(&mut self) {
        self.cursor_lines.vertices[0].position.x = self.hsva.h / 6.;
        self.cursor_lines.vertices[1].position.x = self.hsva.h / 6.;
        self.cursor_lines.vertices[2].position.x = self.hsva.s;
        self.cursor_lines.vertices[3].position.x = self.hsva.s;
        self.cursor_lines.vertices[4].position.x = self.hsva.v;
        self.cursor_lines.vertices[5].position.x = self.hsva.v;
        self.cursor_lines.vertices[6].position.x = self.hsva.a;
        self.cursor_lines.vertices[7].position.x = self.hsva.a;
        self.cursor_lines.update_vbo_range(0..8);
    }
}


pub struct EditorSystem {
    camera: OrthoCamera2D,
    grid_origin_vertices: ColorVertexArray,
    grid_vertices_1: ColorVertexArray,
    grid_vertices_01: ColorVertexArray,
    cursor_vertices: ColorVertexArray,
    draw_grid_first: bool,
    do_draw_grid: bool,
    is_panning_camera: bool,
    camera_rotation_speed: f32,
    prev_camera_rotation_z_radians: f32,
    next_camera_rotation_z_radians: f32,
    is_active: bool,
    primary_color: Rgba<f32>,
    draft_vertices: ColorVertexArray,
    draft_vertices_ended: bool,
    text: Text,
    text_position: Vec2<i32>,
    text_color: Rgba<f32>,
    font_id: FontID,
    hsva_sliders: HsvaSliders,
    hsva_sliding_speed: Hsva<f32>,
    is_entering_command: bool,
    command_text: Text,
}

fn create_grid_vertices(color_mesh_gl_program: &color_mesh::Program, size: Extent2<usize>, color: Rgba<f32>, scale: Extent2<f32>) -> ColorVertexArray {
    let (w, h) = size.map(|x| x as isize).into_tuple();
    let mut vertices = Vec::with_capacity((w * h) as usize);
    for y in (-h) .. (1 + h) {
        if y == 0 {
            let color = Rgba::black();
            vertices.push(Vertex { position: Vec3::new(-w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new(                 0., y as f32 * scale.h, 0.), color, });
            let color = Rgba::red();
            vertices.push(Vertex { position: Vec3::new(                 0., y as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new( w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
        } else {
            vertices.push(Vertex { position: Vec3::new(-w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new( w as f32 * scale.w, y as f32 * scale.h, 0.), color, });
        }
    }
    for x in (-w) .. (1 + w) {
        if x == 0 {
            let color = Rgba::new(0., 0.6, 0., 1.);
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w, -h as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,                  0., 0.), color, });
            let color = Rgba::green();
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,                  0., 0.), color, });
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,  h as f32 * scale.h, 0.), color, });
        } else {
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w, -h as f32 * scale.h, 0.), color, });
            vertices.push(Vertex { position: Vec3::new(x as f32 * scale.w,  h as f32 * scale.h, 0.), color, });
        }
    }
    ColorVertexArray::from_vertices(&color_mesh_gl_program, "Grid Vertices", BufferUsage::StaticDraw, vertices)
}

impl EditorSystem {
    const CAMERA_ZOOM_STEP_FACTOR: f32 = 1.1;
    const CAMERA_Z_ROTATION_SPEED_DEGREES: f32 = 90.;
    const CAMERA_NEAR: f32 = 0.; // It does work for an orthographic camera.
    const CAMERA_FAR: f32 = 1024.;

    pub fn new(color_mesh_gl_program: &color_mesh::Program, text_gl_program: &mesh::text::Program, viewport_size: Extent2<u32>) -> Self {
        let grid_vertices_1 = create_grid_vertices(color_mesh_gl_program, Extent2::new(8, 8), Rgba::white(), Extent2::one());
        let grid_vertices_01 = create_grid_vertices(color_mesh_gl_program, Extent2::new(64, 64), Rgba::new(1., 1., 1., 0.2), Extent2::one()/10.);
        let grid_origin_vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Grid Origin Vertices", BufferUsage::StaticDraw,
            vec![Vertex { position: Vec3::zero(), color: Rgba::red(), }]
        );
        let cursor_vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Cursor Vertices", BufferUsage::DynamicDraw,
            vec![
                Vertex { position: Vec3::zero(), color: Rgba::red(), },
                Vertex { position: Vec3::unit_x(), color: Rgba::green(), },
                Vertex { position: Vec3::unit_y(), color: Rgba::blue(), },
            ]
        );
        let draft_vertices = ColorVertexArray::from_vertices(
            &color_mesh_gl_program, "Draft Vertices", BufferUsage::DynamicDraw, vec![]
        );
        let text = Text::new(text_gl_program, "Editor Text");
        let hsva_sliders = HsvaSliders::new(&color_mesh_gl_program);
        let camera = OrthoCamera2D::new(viewport_size, Self::CAMERA_NEAR, Self::CAMERA_FAR);
        Self {
            camera, cursor_vertices, grid_origin_vertices, grid_vertices_1, grid_vertices_01,
            draft_vertices,
            draft_vertices_ended: false,
            primary_color: Rgba::red(),
            draw_grid_first: true,
            do_draw_grid: true,
            is_panning_camera: false,
            camera_rotation_speed: 0.,
            prev_camera_rotation_z_radians: 0.,
            next_camera_rotation_z_radians: 0.,
            is_active: false,
            text,
            text_position: (viewport_size.map(|x| x as i32) / 2).into(),
            text_color: Rgba::black(),
            font_id: FontID::Debug,
            hsva_sliders,
            hsva_sliding_speed: Hsva { h: 0., s: 0., v: 0., a: 0. },
            is_entering_command: false,
            command_text: Text::new(text_gl_program, "Editor Command Text"),
        }
    }
    pub const CLEAR_COLOR: Rgba<f32> = Rgba {
        r: 0.1, g: 0.2, b: 1., a: 1.,
    };
    fn on_enter_editor(&mut self, g: &Game) {
        debug_assert!(!self.is_active);
        self.is_active = true;
        unsafe {
            let Rgba { r, g, b, a } = Self::CLEAR_COLOR;
            gl::ClearColor(r, g, b, a);
        }
        g.platform.cursors.crosshair.set();
        self.text.string = "If the universe is infinite,\nthere is an infinite number of worlds\nwhere this story takes place.".to_owned();
        self.text.update_gl(&g.fonts.fonts[&self.font_id]);
    }
    fn on_leave_editor(&mut self, g: &Game) {
        debug_assert!(self.is_active);
        self.is_active = false;
        unsafe {
            gl::ClearColor(1., 1., 1., 1.);
        }
        g.platform.cursors.normal.set();
    }

    fn add_vertex_at_current_mouse_position(&mut self, g: &Game) {
        debug_assert!(self.is_active);
        debug!("Editor: Adding vertex at current mouse position");
        if self.draft_vertices_ended {
            return;
        }
        if let Some(pos) = g.input.mouse_position() {
            let color = self.primary_color;
            let mut position = self.camera.viewport_to_world(pos, 0.);
            // position.z = 0.;
            self.draft_vertices.vertices.push(Vertex { position, color, });
            self.draft_vertices.update_and_resize_vbo();
        }
    }
    fn end_polygon(&mut self, _g: &Game) {
        debug_assert!(self.is_active);
        self.draft_vertices_ended = true;
    }
    fn toggle_select_all(&mut self, _g: &Game) {
        debug_assert!(self.is_active);
        unimplemented!{}
    }
    fn deleted_selected(&mut self, _g: &Game) {
        debug_assert!(self.is_active);
        self.draft_vertices.vertices.clear();
        self.draft_vertices.update_and_resize_vbo();
        self.draft_vertices_ended = false;
    }
    fn execute_current_command(&mut self, g: &Game) {
        let cmd = self.command_text.string.clone();
        self.execute_command(g, &cmd);
    }
    fn execute_command(&mut self, g: &Game, mut cmd: &str) {
        if cmd == "" {
            return;
        }
        if cmd.chars().nth(0).unwrap() == ':' {
            cmd = &cmd[1..];
        }

        let line: Vec<_> = cmd.split_whitespace().collect();
        if line.is_empty() {
            return;
        }
        let cmd = &line[0];
        let args = &line[1..];

        match *cmd {
            "helloworld" => info!("Editor: Hello world! args={:?}", args),
            "w" => self.save_draft_mesh_to_file(g, args),
            _ => error!("`{}` is not recognized as an editor command", cmd),
        };
    }
    // M = moveto
    // L = lineto
    // C = curveto
    // Q = quadratic Bézier curve
    // Z = closepath
    // Note: All of the commands above can also be expressed with lower letters. Capital letters means absolutely positioned, lower cases means relatively positioned.
    fn save_draft_mesh_to_file(&mut self, _g: &Game, args: &[&str]) {
        let filename = args[0];
        info!("Saving draft mesh to file `{}`", filename);
    
        use ::std::io::Write;
        let mut file = File::create(filename).unwrap();
        for (i, v) in self.draft_vertices.vertices.iter().enumerate() {
            let letter = if i == 0 { 'M' } else { 'L' };
            let pos = v.position;
            writeln!(file, "{} {} {}", letter, pos.x, pos.y).unwrap();
        }
        if self.draft_vertices_ended {
            writeln!(file, "Z").unwrap();
        }
    }
}

impl System for EditorSystem {
    fn name(&self) -> &str {
        "EditorSystem"
    }
    fn on_canvas_resized(&mut self, _: &Game, size: Extent2<u32>, _by_user: bool) {
        self.camera.set_viewport_size(size);
        self.text_position = (self.camera.viewport_size() / 2).map(|x| x as i32).into();
        self.text_position.y -= 1;
    }
    fn on_mouse_motion(&mut self, g: &Game, pos: Vec2<i32>) {
        if !self.is_active {
            return;
        }
        if let Some(prev) = g.input.previous_mouse_position() {
            if self.is_panning_camera {
                let o = self.camera.viewport_to_world(prev, 0.);
                let p = self.camera.viewport_to_world(pos, 0.);
                self.camera.xform.position -= (p - o) * self.camera.xform.scale.x;
                self.camera.xform.position.z = 0.;
            }
        }
    }
    fn on_mouse_scroll(&mut self, _: &Game, delta: Vec2<i32>) {
        if !self.is_active {
            return;
        }
        self.camera.xform.scale *= Self::CAMERA_ZOOM_STEP_FACTOR.powf(delta.y as _);
    }
    fn on_text_input(&mut self, g: &Game, s: &str) {
        if !self.is_active {
            return;
        }
        if self.is_entering_command {
            self.command_text.string += s;
            self.command_text.update_gl(&g.fonts.fonts[&FontID::Debug]);
        }
    }
    fn on_key(&mut self, g: &Game, key: Key) {
        if !self.is_active {
            return;
        }
        if self.is_entering_command {
            let keycode = key.code.unwrap();
            match keycode {
                Keycode::Escape | Keycode::Return | Keycode::Return2 | Keycode::KpEnter => if key.is_down() {
                    match keycode {
                        Keycode::Return | Keycode::Return2 | Keycode::KpEnter => self.execute_current_command(g),
                        _ => (),
                    };
                    self.is_entering_command = false;
                    self.command_text.string.clear();
                    self.command_text.update_gl(&g.fonts.fonts[&FontID::Debug]);
                },
                Keycode::Backspace => if key.is_down() {
                    self.command_text.string.pop();
                    self.command_text.update_gl(&g.fonts.fonts[&FontID::Debug]);
                },
                _ => (),
            };
            return;
        }

        let normal_camera_rotation_speed = Self::CAMERA_Z_ROTATION_SPEED_DEGREES.to_radians();

        match key.code.unwrap() {
            Keycode::Colon => if key.is_down() {
                self.is_entering_command = true;
            },
            Keycode::G => if key.is_down() {
                self.do_draw_grid = !self.do_draw_grid;
            },
            Keycode::F => if key.is_down() {
                self.draw_grid_first = !self.draw_grid_first;
            },
            Keycode::Space => self.is_panning_camera = key.is_down(),
            Keycode::R => self.camera_rotation_speed = -normal_camera_rotation_speed * key.is_down() as i32 as f32,
            Keycode::T => self.camera_rotation_speed =  normal_camera_rotation_speed * key.is_down() as i32 as f32,
            Keycode::C => if key.is_down() {
                self.camera.xform = Default::default();
                self.prev_camera_rotation_z_radians = 0.;
                self.next_camera_rotation_z_radians = 0.;
            },
            Keycode::Return => if key.is_down() {
                self.end_polygon(g);
            },
            Keycode::A => if key.is_down() {
                self.toggle_select_all(g);
            },
            Keycode::Backspace | Keycode::Delete | Keycode::X => if key.is_down() {
                self.deleted_selected(g);
            },
            Keycode::J => self.hsva_sliding_speed.v =  1. * key.is_down() as i32 as f32,
            Keycode::K => self.hsva_sliding_speed.v = -1. * key.is_down() as i32 as f32,
            Keycode::L => self.hsva_sliding_speed.s = -1. * key.is_down() as i32 as f32,
            Keycode::M => self.hsva_sliding_speed.s =  1. * key.is_down() as i32 as f32,
            Keycode::U => self.hsva_sliding_speed.h = -6. * key.is_down() as i32 as f32,
            Keycode::I => self.hsva_sliding_speed.h =  6. * key.is_down() as i32 as f32,
            Keycode::O => self.hsva_sliding_speed.a =  1. * key.is_down() as i32 as f32,
            Keycode::P => self.hsva_sliding_speed.a = -1. * key.is_down() as i32 as f32,
            _ => (),
        };
    }
    fn on_mouse_button(&mut self, g: &Game, btn: MouseButton) {
        match btn.button {
            Sdl2MouseButton::Left => {
                debug!("Editor: Received Left click event");
                self.add_vertex_at_current_mouse_position(g);
            },
            Sdl2MouseButton::Middle => {},
            Sdl2MouseButton::Right => {},
            Sdl2MouseButton::Unknown => {},
            Sdl2MouseButton::X1 => {},
            Sdl2MouseButton::X2 => {},
        };
    }
    fn on_message(&mut self, g: &Game, msg: &Message) {
        match *msg {
            Message::EnterEditor => { self.on_enter_editor(g); return; },
            Message::LeaveEditor => { self.on_leave_editor(g); return; },
            _ => (),
        };
    }
    fn tick(&mut self, g: &Game, _: Duration, dt: Duration) {
        if !self.is_active {
            return;
        }
        let dt = dt.to_f64_seconds() as f32;
        self.prev_camera_rotation_z_radians = self.next_camera_rotation_z_radians;
        self.next_camera_rotation_z_radians += dt * self.camera_rotation_speed;
        self.hsva_sliders.hsva.h += dt * self.hsva_sliding_speed.h;
        self.hsva_sliders.hsva.s += dt * self.hsva_sliding_speed.s;
        self.hsva_sliders.hsva.v += dt * self.hsva_sliding_speed.v;
        self.hsva_sliders.hsva.a += dt * self.hsva_sliding_speed.a;
        use ::v::{Clamp, Wrap};
        self.hsva_sliders.hsva.h = self.hsva_sliders.hsva.h.wrapped(6.);
        self.hsva_sliders.hsva.s = self.hsva_sliders.hsva.s.clamped01();
        self.hsva_sliders.hsva.v = self.hsva_sliders.hsva.v.clamped01();
        self.hsva_sliders.hsva.a = self.hsva_sliders.hsva.a.clamped01();
        self.hsva_sliders.update_colors_gl();
        self.hsva_sliders.update_cursor_lines_gl();

        self.text.string = format!("{:?}", self.hsva_sliders.hsva);
        self.text.update_gl(&g.fonts.fonts[&self.font_id]);
        self.cursor_vertices.vertices[0].color = rgba_from_hsva(self.hsva_sliders.hsva);
        self.cursor_vertices.update_vbo_range(0..1);
    }
    fn draw(&mut self, g: &Game, gfx_interp: f64) {
        if !self.is_active {
            return;
        }
        self.camera.xform.rotation_z_radians = ::v::Lerp::lerp(self.prev_camera_rotation_z_radians, self.next_camera_rotation_z_radians, gfx_interp as f32);
        unsafe {
            let draw_cursor = || if let Some(pos) = g.input.mouse_position() {
                let mvp = {
                    let w = self.camera.viewport_to_world(pos, 0.);
                    self.camera.view_proj_matrix() * Mat4::translation_3d(w)
                };
                g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                gl::PointSize(8.);
                gl::BindVertexArray(self.cursor_vertices.vao().gl_id());
                gl::DrawArrays(gl::POINTS, 0, self.cursor_vertices.vertices.len() as _);
                gl::DrawArrays(gl::TRIANGLES, 0, self.cursor_vertices.vertices.len() as _);
            };

            let draw_draft_vertices = || {
                let mvp = self.camera.view_proj_matrix();
                g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                gl::PointSize(8.);
                gl::LineWidth(8.);
                gl::BindVertexArray(self.draft_vertices.vao().gl_id());
                gl::DrawArrays(gl::POINTS, 0, self.draft_vertices.vertices.len() as _);
                let topology = if self.draft_vertices_ended { gl::LINE_LOOP } else { gl::LINE_STRIP };
                gl::DrawArrays(topology, 0, self.draft_vertices.vertices.len() as _);
            };

            let draw_grid = || {
                if self.do_draw_grid {
                    gl::Disable(gl::DEPTH_TEST);
                    gl::DepthMask(gl::FALSE);

                    let mvp = {
                        let pixel = self.camera.world_to_viewport(Vec3::zero()).0;
                        let w = self.camera.viewport_to_world(pixel, 0.);
                        self.camera.view_proj_matrix() * Mat4::translation_3d(w)
                    };
                    g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                    gl::LineWidth(1.);

                    gl::BindVertexArray(self.grid_vertices_01.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_vertices_01.vertices.len() as _);

                    gl::BindVertexArray(self.grid_vertices_1.vao().gl_id());
                    gl::DrawArrays(gl::LINES, 0, self.grid_vertices_1.vertices.len() as _);

                    gl::PointSize(8.);
                    gl::BindVertexArray(self.grid_origin_vertices.vao().gl_id());
                    gl::DrawArrays(gl::POINTS, 0, self.grid_origin_vertices.vertices.len() as _);

                    gl::DepthMask(gl::TRUE);
                    gl::Enable(gl::DEPTH_TEST);
                }
            };

            let draw_hsva_sliders = || {
                gl::Disable(gl::DEPTH_TEST);
                gl::DepthMask(gl::FALSE);
                let mvp = {
                    let t = self.camera.viewport_to_ugly_ndc(Vec2::unit_y() * self.camera.viewport_size().h as i32);
                    let s = Mat4::scaling_3d(Vec2::new(1. / self.camera.aspect_ratio(), 1.) / 1.5);
                    Mat4::<f32>::translation_3d(t) * s
                };
                g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                let strips = &[
                    &self.hsva_sliders.strips.h,
                    &self.hsva_sliders.strips.s,
                    &self.hsva_sliders.strips.v,
                    &self.hsva_sliders.strips.a,
                ];
                for strip in strips {
                    gl::BindVertexArray(strip.vao().gl_id());
                    gl::DrawArrays(gl::TRIANGLE_STRIP, 0, strip.vertices.len() as _);
                }
                gl::LineWidth(2.);
                gl::BindVertexArray(self.hsva_sliders.cursor_lines.vao().gl_id());
                gl::DrawArrays(gl::LINES, 0, self.hsva_sliders.cursor_lines.vertices.len() as _);
                gl::DepthMask(gl::TRUE);
                gl::Enable(gl::DEPTH_TEST);
            };


            {
                let vp = self.camera.viewport_size();
                gl::Viewport(0, 0, vp.w as _, vp.h as _);
            }

            if self.is_entering_command {

                let grey = 0.1;
                gl::ClearColor(grey, grey, grey, 1.);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                gl::UseProgram(g.text_gl_program.program().gl_id());
                let command_text_position = Vec2::new(0, g.fonts.fonts[&FontID::Debug].height as i32);
                let mvp = {
                    let Extent2 { w, h } = g.fonts.fonts[&FontID::Debug].texture_size.map(|x| x as f32) * 2. / self.camera.viewport_size().map(|x| x as f32);
                    let t = self.camera.viewport_to_ugly_ndc(command_text_position);
                    Mat4::<f32>::translation_3d(t) * Mat4::scaling_3d(Vec3::new(w, h, 1.))
                };
                g.text_gl_program.set_uniform_mvp(&mvp);
                g.text_gl_program.set_uniform_font_atlas_via_font_id(FontID::Debug);
                g.text_gl_program.set_uniform_color(Rgba::white());
                gl::BindVertexArray(self.command_text.vertices.vao().gl_id());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.command_text.indices.ibo().gl_id());
                gl::DrawElements(gl::TRIANGLES, self.command_text.indices.indices.len() as _, gl::UNSIGNED_SHORT, ptr::null_mut());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                gl::BindVertexArray(0);

                gl::UseProgram(0);
                let Rgba { r, g, b, a } = Self::CLEAR_COLOR;
                gl::ClearColor(r, g, b, a);

            } else {

                gl::UseProgram(g.color_mesh_gl_program.program().gl_id());

                if self.draw_grid_first {
                    draw_grid();
                    draw_cursor();
                    draw_draft_vertices();
                    draw_hsva_sliders();
                } else {
                    draw_cursor();
                    draw_draft_vertices();
                    draw_grid();
                    draw_hsva_sliders();
                }


                // Render text (last, so it always appears on top of grid)

                gl::Disable(gl::DEPTH_TEST);
                gl::UseProgram(g.text_gl_program.program().gl_id());
                let mvp = {
                    let Extent2 { w, h } = g.fonts.fonts[&self.font_id].texture_size.map(|x| x as f32) * 2. / self.camera.viewport_size().map(|x| x as f32);
                    let t = self.camera.viewport_to_ugly_ndc(self.text_position);
                    Mat4::<f32>::translation_3d(t) * Mat4::scaling_3d(Vec3::new(w, h, 1.))
                };
                g.text_gl_program.set_uniform_mvp(&mvp);
                g.text_gl_program.set_uniform_font_atlas_via_font_id(self.font_id);
                g.text_gl_program.set_uniform_color(self.text_color);
                gl::BindVertexArray(self.text.vertices.vao().gl_id());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.text.indices.ibo().gl_id());
                gl::DrawElements(gl::TRIANGLES, self.text.indices.indices.len() as _, gl::UNSIGNED_SHORT, ptr::null_mut());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                gl::BindVertexArray(0);
                gl::Enable(gl::DEPTH_TEST);
            }


            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }
}


use gl;
use system::*;
use camera::OrthoCamera2D;
use gx::Object;
use shape::{self, Shape, Style};
use scene::ShapeInstance;
use v::Mat4;

#[derive(Debug)]
pub struct GameplaySystem {
    is_active: bool,
    current_scene_name: String,
    camera: OrthoCamera2D,
}

impl GameplaySystem {
    const CAMERA_NEAR: f32 = ::editor::EditorSystem::CAMERA_NEAR;
    const CAMERA_FAR: f32 = ::editor::EditorSystem::CAMERA_FAR;
    pub fn new(viewport_size: Extent2<u32>) -> Self {
        Self {
            is_active: false,
            current_scene_name: "default".to_owned(),
            camera: OrthoCamera2D::new(viewport_size, Self::CAMERA_NEAR, Self::CAMERA_FAR),
        }
    }
    fn on_enter_gameplay(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(0.2, 0.6, 1., 1.);
        }
        g.platform.cursors.normal.set();
        self.is_active = true;
    }
    fn on_leave_gameplay(&mut self, g: &Game) {
        unsafe {
            gl::ClearColor(1., 0., 1., 1.);
        }
        g.platform.cursors.normal.set();
        self.is_active = false;
    }
}

impl System for GameplaySystem {
    fn name(&self) -> &str {
        "GameplaySystem"
    }
    fn on_canvas_resized(&mut self, _: &Game, size: Extent2<u32>, _by_user: bool) {
        self.camera.set_viewport_size(size);
    }
    fn on_message(&mut self, g: &Game, msg: &Message) {
        match *msg {
            Message::EnterGameplay => self.on_enter_gameplay(g),
            Message::LeaveGameplay => self.on_leave_gameplay(g),
            _ => (),
        };
    }
    fn draw(&mut self, g: &Game, _gfx_interp: f64) {
        if !self.is_active {
            return;
        }
        unsafe {
            {
                let vp = self.camera.viewport_size();
                gl::Viewport(0, 0, vp.w as _, vp.h as _);
            }

            gl::UseProgram(g.color_mesh_gl_program.program().gl_id());

            for shape_instance in g.loaded_scenes.borrow()[&self.current_scene_name].shape_instances.iter() {
                draw_shape_instance(g, &self.camera, shape_instance);
            }

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }
}

pub unsafe fn draw_shape_instance(g: &Game, camera: &OrthoCamera2D, shape_instance: &ShapeInstance) {
    let &ShapeInstance {
        ref source_shape_name, name: _, xform,
    } = shape_instance;
    
    let &Shape {
        path: shape::Path {
            is_closed, start: _, cmds: _,
        },
        style: Style {
            stroke_thickness, stroke_color: _, fill_color: _,
            fill_gradient: _,
        },
        ref vertices,
        ref solid_fill_strip,
        ref gradient_fill_strip,
    } = &g.loaded_shapes.borrow()[source_shape_name];

    // Set MVP once, first.
    let mvp = camera.view_proj_matrix() * xform.model_matrix();
    g.color_mesh_gl_program.set_uniform_mvp(&mvp);
    g.color_mesh_gl_program.set_uniform_is_drawing_points(false);

    // Fill
    {
        gl::Enable(gl::STENCIL_TEST);
        gl::Disable(gl::DEPTH_TEST);

        gl::ClearStencil(0x0); // Set clear value
        gl::Clear(gl::STENCIL_BUFFER_BIT);
        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
        gl::DepthMask(gl::FALSE);
        gl::StencilFunc(gl::ALWAYS, 0, 1);
        gl::StencilOp(gl::KEEP, gl::KEEP, gl::INVERT);
        gl::StencilMask(1);

        gl::BindVertexArray(vertices.vao().gl_id());
        gl::DrawArrays(gl::TRIANGLE_FAN, 0, vertices.vertices.len() as _);

        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
        gl::DepthMask(gl::TRUE);
        gl::StencilFunc(gl::EQUAL, 1, 1);
        gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);

        g.color_mesh_gl_program.set_uniform_mvp(&Mat4::identity());
        gl::BindVertexArray(solid_fill_strip.vao().gl_id());
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, solid_fill_strip.vertices.len() as _);
        gl::BindVertexArray(gradient_fill_strip.vao().gl_id());
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, gradient_fill_strip.vertices.len() as _);
        g.color_mesh_gl_program.set_uniform_mvp(&mvp);

        gl::Enable(gl::DEPTH_TEST);
        gl::Disable(gl::STENCIL_TEST);
    }

    // Stroke
    {
        gl::BindVertexArray(vertices.vao().gl_id());
        gl::PointSize(stroke_thickness);
        gl::LineWidth(stroke_thickness);
        let topology = if is_closed { gl::LINE_LOOP } else { gl::LINE_STRIP };
        g.color_mesh_gl_program.set_uniform_is_drawing_points(true);
        gl::DrawArrays(gl::POINTS, 0, vertices.vertices.len() as _);
        g.color_mesh_gl_program.set_uniform_is_drawing_points(false);
        gl::DrawArrays(topology, 0, vertices.vertices.len() as _);
    }


}


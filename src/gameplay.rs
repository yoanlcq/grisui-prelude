use gl;
use system::*;
use camera::OrthoCamera2D;
use gx::Object;

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
                let shape = &g.loaded_shapes.borrow()[&shape_instance.source_shape_name];
                let mvp = self.camera.view_proj_matrix() * shape_instance.xform.model_matrix();
                g.color_mesh_gl_program.set_uniform_mvp(&mvp);
                gl::PointSize(8.);
                gl::LineWidth(8.);
                gl::BindVertexArray(shape.vertices.vao().gl_id());
                gl::DrawArrays(gl::POINTS, 0, shape.vertices.vertices.len() as _);
                let topology = if shape.is_path_closed { gl::LINE_LOOP } else { gl::LINE_STRIP };
                gl::DrawArrays(topology, 0, shape.vertices.vertices.len() as _);
            }

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }
}

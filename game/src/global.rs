// XXX: HashMap: insert() removes and returns the old value if any!!!
// Write some test program to ensure this!!!

use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use std::hash::{Hash, Hasher, BuildHasher};
use std::env;
use std::ffi::CString;
use std::ptr;
use std::time::Duration;
use std::fmt::{self, Formatter, Debug};
use DurationExt;
use env_logger;
use log::LevelFilter;
use sdl2;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::video::{Window, GLContext, GLProfile, SwapInterval};
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::{MouseWheelDirection, MouseButton};
use sdl2::keyboard::{Keycode};
use alto;
use alto::Alto;
use gl;
use gl::types::*;
use gx;
use gx::Object;
use grx;
use v::{Rgba, Extent2, Rect, Vec2, Vec3, Mat4, Lerp, Transform};
use camera::*;
use lazy::Lazy;

macro_rules! define_id_domain {
    (
        monotonic ($itype:ty) $write_itype:ident 
        ID: $ID:ident
        IDDomain: $IDDomain:ident
        IDHasher: $IDHasher:ident
        IDHasherBuilder: $IDHasherBuilder:ident
        IDMap: $IDMap:ident
        IDRealm: $IDRealm:ident
    ) => {
        /// A strongly-type ID value.
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $ID($itype);

        /// A monotonically-increasing ID generator.
        #[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $IDDomain {
            pub current_highest: $itype,
        }

        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        pub struct $IDHasher($itype);

        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        pub struct $IDHasherBuilder;

        pub type $IDMap<T> = HashMap<$ID, T, $IDHasherBuilder>;

        #[derive(Debug, PartialEq, Eq)]
        pub struct $IDRealm<T> {
            domain: $IDDomain,
            map: $IDMap<T>,
        }

        impl Hasher for $IDHasher {
            fn finish(&self) -> u64 {
                self.0 as _
            }
            fn write(&mut self, _bytes: &[u8]) { unreachable!{} }
            fn $write_itype(&mut self, i: $itype) { self.0 = i; }
        }

        impl BuildHasher for $IDHasherBuilder {
            type Hasher = $IDHasher;
            fn build_hasher(&self) -> Self::Hasher {
                Default::default()
            }
        }

        impl $ID {
            pub fn from_raw(value: $itype) -> Self {
                $ID(value)
            }
            pub fn get_raw(&self) -> $itype {
                self.0
            }
        }

        impl $IDDomain {
            pub fn new_empty() -> Self {
                Self {
                    current_highest: 0,
                }
            }
            pub fn from_ids<I>(ids: I) -> Self 
                where I: IntoIterator<Item=$ID>
            {
                let mut slf = Self::new_empty();
                slf.include_ids(ids);
                slf
            }
            pub fn include_ids<I>(&mut self, ids: I) 
                where I: IntoIterator<Item=$ID>
            {
                for id in ids {
                    self.include_id(id);
                }
            }
            pub fn include_id(&mut self, id: $ID) {
                self.current_highest = ::std::cmp::max(self.current_highest, id.get_raw());
            }
            pub fn generate_new_id(&mut self) -> $ID {
                self.current_highest += 1;
                $ID(self.current_highest)
            }
        }
        impl<T> $IDRealm<T> {
            pub fn from_iterator<I>(iterator: I) -> Self where I: IntoIterator<Item=($ID, T)> {
                let mut slf = Self::new_empty();
                for (id, value) in iterator {
                    slf.insert_missing(id, value);
                }
                slf
            }
            pub fn new_empty() -> Self {
                Self::with_capacity(0)
            }
            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    domain: $IDDomain::new_empty(),
                    map: $IDMap::with_capacity_and_hasher(capacity, Default::default()),
                }
            }
            pub fn insert_or_replace(&mut self, id: $ID, value: T) -> Option<T> {
                self.domain.include_id(id);
                self.map.insert(id, value)
            }
            pub fn insert_missing(&mut self, id: $ID, value: T) {
                self.domain.include_id(id);
                let old = self.insert_or_replace(id, value);
                assert!(old.is_none());
            }
            pub fn replace_existing(&mut self, id: $ID, value: T) -> T {
                self.insert_or_replace(id, value).unwrap()
            }
            pub fn insert_new_and_get_id(&mut self, value: T) -> $ID {
                let id = self.generate_new_id();
                self.insert_missing(id, value);
                id
            }
            pub fn generate_new_id(&mut self) -> $ID {
                self.domain.generate_new_id()
            }
            pub fn ids(&self) -> hash_map::Keys<$ID, T> { self.map.keys() }
            pub fn values(&self) -> hash_map::Values<$ID, T> { self.map.values() }
            pub fn values_mut(&mut self) -> hash_map::ValuesMut<$ID, T> { self.map.values_mut() }
            pub fn iter(&self) -> hash_map::Iter<$ID, T> { self.map.iter() }
            pub fn iter_mut(&mut self) -> hash_map::IterMut<$ID, T> { self.map.iter_mut() }
            pub fn entry(&mut self, id: $ID) -> hash_map::Entry<$ID, T> { self.map.entry(id) }
            pub fn len(&self) -> usize { self.map.len() }
            pub fn is_empty(&self) -> bool { self.map.is_empty() }
            pub fn drain(&mut self) -> hash_map::Drain<$ID, T> { self.map.drain() }
            pub fn clear(&mut self) { self.map.clear(); self.domain = $IDDomain::new_empty(); }
            pub fn get(&self, id: $ID) -> Option<&T> { self.map.get(&id) }
            pub fn get_mut(&mut self, id: $ID) -> Option<&mut T> { self.map.get_mut(&id) }
            pub fn contains_id(&self, id: $ID) -> bool { self.map.contains_key(&id) }
            pub fn remove(&mut self, id: $ID) -> Option<T> { self.map.remove(&id) }
        }

        impl<T> ::std::ops::Index<$ID> for $IDRealm<T> {
            type Output = T;
            fn index(&self, id: $ID) -> &T {
                &self.map[&id]
            }
        }
        impl<T> ::std::ops::IndexMut<$ID> for $IDRealm<T> {
            fn index_mut(&mut self, id: $ID) -> &mut T {
                self.get_mut(id).unwrap()
            }
        }

        impl<T> IntoIterator for $IDRealm<T> {
            type Item = ($ID, T);
            type IntoIter = hash_map::IntoIter<$ID, T>;
            fn into_iter(self) -> Self::IntoIter {
                self.map.into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a $IDRealm<T> {
            type Item = (&'a $ID, &'a T);
            type IntoIter = hash_map::Iter<'a, $ID, T>;
            fn into_iter(self) -> Self::IntoIter {
                self.map.iter()
            }
        }

        impl<'a, T> IntoIterator for &'a mut $IDRealm<T> {
            type Item = (&'a $ID, &'a mut T);
            type IntoIter = hash_map::IterMut<'a, $ID, T>;
            fn into_iter(self) -> Self::IntoIter {
                self.map.iter_mut()
            }
        }
    }
}


pub struct Global {
    // Runtime
    pub alto: Alto,
    pub alto_dev: alto::OutputDevice,
    pub alto_context: alto::Context,
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub gl_context: GLContext,
    pub gl_simple_color_program: grx::SimpleColorProgram,

    // Persistent data, ID domains
    pub tags: TagIDRealm<String>,
    pub palette_colors: PaletteColorIDRealm<Rgba<u8>>,
    pub palette_tags: TagIDMap<HashSet<PaletteColorID>>,
    pub meshes: MeshResourceIDRealm<Lazy<MeshResource>>,
    pub other_scenes: SceneIDRealm<Lazy<Scene>>,
    pub saves: SaveIDRealm<Save>,

    // Current state, current IDs
    pub save_id: SaveID,
    // Current state, ctd.
    pub input: Input,
    pub viewport_size: Extent2<u32>,
}

define_id_domain!{
    monotonic (u32) write_u32
    ID:              TagID
    IDDomain:        TagIDDomain
    IDHasher:        TagIDHasher
    IDHasherBuilder: TagIDHasherBuilder
    IDMap:           TagIDMap
    IDRealm:         TagIDRealm
}
define_id_domain!{
    monotonic (u32) write_u32
    ID:              PaletteColorID
    IDDomain:        PaletteColorIDDomain
    IDHasher:        PaletteColorIDHasher
    IDHasherBuilder: PaletteColorIDHasherBuilder
    IDMap:           PaletteColorIDMap
    IDRealm:         PaletteColorIDRealm
}
define_id_domain!{
    monotonic (u32) write_u32
    ID:              MeshResourceID
    IDDomain:        MeshResourceIDDomain
    IDHasher:        MeshResourceIDHasher
    IDHasherBuilder: MeshResourceIDHasherBuilder
    IDMap:           MeshResourceIDMap
    IDRealm:         MeshResourceIDRealm
}
define_id_domain!{
    monotonic (u32) write_u32
    ID:              SceneID
    IDDomain:        SceneIDDomain
    IDHasher:        SceneIDHasher
    IDHasherBuilder: SceneIDHasherBuilder
    IDMap:           SceneIDMap
    IDRealm:         SceneIDRealm
}
define_id_domain!{
    monotonic (u32) write_u32
    ID:              SaveID
    IDDomain:        SaveIDDomain
    IDHasher:        SaveIDHasher
    IDHasherBuilder: SaveIDHasherBuilder
    IDMap:           SaveIDMap
    IDRealm:         SaveIDRealm
}


#[derive(Debug)]
pub struct MeshResource {
    pub vertices: Vec<grx::SimpleColorVertex>,
    pub gl_topology: GLenum,
    pub vao: gx::Vao,
    pub vbo: gx::Vbo,
}

pub mod ecs {

    use super::*;

    define_id_domain!{
        monotonic (u32) write_u32
        ID:              EntityID
        IDDomain:        EntityIDDomain
        IDHasher:        EntityIDHasher
        IDHasherBuilder: EntityIDHasherBuilder
        IDMap:           EntityIDMap
        IDRealm:         EntityIDRealm
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub struct Stateful<T> {
        pub previous: T,
        pub current: T,
        pub render: T,
    }

    impl<T: Clone> From<T> for Stateful<T> {
        fn from(value: T) -> Self {
            let previous = value;
            let current = previous.clone();
            let render = previous.clone();
            Self { previous, current, render }
        }
    }

    pub type TransformComponent = Stateful<Transform<f32,f32,f32>>;
    pub type OrthographicCameraComponent = Stateful<OrthographicCamera>;

    pub type Transforms = HashMap<EntityID, TransformComponent>;
    pub type Cameras = HashMap<EntityID, OrthographicCameraComponent>;
    pub type MeshInstances = HashMap<EntityID, MeshResourceID>;
}

#[derive(Debug)]
pub struct Scene {
    pub allows_quitting: bool,
    pub mesh_instances: ecs::MeshInstances,
    pub transforms: ecs::Transforms,
    pub cameras: ecs::Cameras,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Save {
    pub scene_id: SceneID,
    pub has_unlocked_door: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Input {
    pub wants_to_quit: bool,
    pub mouse: MouseInput,
    pub keyboard: KeyboardInput,
    /// The current piece of input text for this tick.
    pub text: String,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MouseInput {
    pub left: ButtonInput,
    pub middle: ButtonInput,
    pub right: ButtonInput,
    pub scroll: Vec2<i32>,
    pub position: Vec2<u32>,
    pub prev_position: Vec2<u32>,
    pub has_prev_position: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KeyboardInput {
    pub keys: HashMap<Keycode, ButtonInput>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ButtonInput {
    /// The state for the current tick.
    pub is_pressed: bool,
    /// The state at previous tick.
    pub was_pressed: bool,
    pub num_changes_since_last_tick_and_event_processing: u16,
    // XXX What if user holds down buttons while starting the game ?
}


fn setup_env() {
    //env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "full");
}

fn setup_log() {
    use ::std::io::Write;

    let mut builder = env_logger::Builder::new();
    builder.format(|buf, record| {
        let s = format!("{}", record.level());
        let s = s.chars().next().unwrap();
        writeln!(buf, "[{}] {}", s, record.args())
    }).filter(None, LevelFilter::Debug);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init();
}

impl Default for Global {
    fn default() -> Self {
        setup_env();
        setup_log();

        let alto = Alto::load_default().unwrap();
        let alto_dev = alto.open(None).unwrap();
        let attrs = alto::ContextAttrs {
            frequency: Some(44100),
            refresh: None,
            mono_sources: None,
            stereo_sources: None,
            soft_hrtf: None,
            soft_hrtf_id: None,
            soft_output_limiter: None,
            max_aux_sends: None,
        };
        let alto_context = alto_dev.new_context(Some(attrs)).unwrap();
        /*
        let buf = ctx.new_buffer(data, freq).unwrap();
        let static_src = ctx.new_static_source().unwrap();
        static_src.set_looping(false);
        static_src.set_buffer(Arc::new(buf)).unwrap();
        let stream_src = ctx.new_streaming_source().unwrap();
        stream_src.queue_buffer(buf).unwrap();
        stream_src.unqueue_buffer().unwrap();
        // play, pause, stop, rewind, state, gain, position, velocity, direction
        */

        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        {
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_flags().debug().set();
            //gl_attr.set_context_version(3, 2);
            gl_attr.set_depth_size(24);
            gl_attr.set_stencil_size(8);
            gl_attr.set_multisample_buffers(1);
            gl_attr.set_multisample_samples(4);
        }

        let window = video.window("Grisui - Prelude", 600, 480)
            .position_centered()
            .resizable()
            .opengl()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        window.gl_set_context_to_current().unwrap();

        gl::load_with(|s| video.gl_get_proc_address(s) as _);
        video.gl_set_swap_interval(SwapInterval::LateSwapTearing);

        unsafe {
            gx::init(&video);
        }

        let gl_simple_color_program = grx::SimpleColorProgram::new();

        let tags = TagIDRealm::from_iterator(
            ["a", "b", "c"].iter().enumerate().map(|(i, s)| {
                (TagID::from_raw(i as _), s.to_string())
            })
        );

        let palette_tags = TagIDMap::default();
        let palette_colors = PaletteColorIDRealm::from_iterator(
            [Rgba::red(), Rgba::green()].iter().enumerate().map(|(i, rgba)| {
                (PaletteColorID::from_raw(i as _), *rgba)
            })
        );

        let mesh_resources = vec![
            Lazy::Loaded(MeshResource::new_unit_quad(
                &gl_simple_color_program, "Lucky Quad"
            )),
        ];
        let meshes = MeshResourceIDRealm::from_iterator(
            mesh_resources.into_iter().enumerate().map(|(i, mesh)| {
                (MeshResourceID::from_raw(i as _), mesh)
            })
        );

        let all_saves = vec![
            Save::default()
        ];
        let saves = SaveIDRealm::from_iterator(
            all_saves.into_iter().enumerate().map(|(i, save)| {
                (SaveID::from_raw(i as _), save)
            })
        );
        let save_id = SaveID::from_raw(0);

        let other_scenes = SceneIDRealm::new_empty();

        let viewport_size = Extent2::from(window.drawable_size());

        let mut g = Self {
            alto, alto_dev, alto_context,
            sdl, video, window, gl_context, gl_simple_color_program,

            tags,
            palette_colors,
            palette_tags,
            meshes,
            other_scenes,
            saves,

            save_id,
            input: Input::default(),
            viewport_size,
        };
        g.reshape(viewport_size);
        g
    }
}

macro_rules! impl_debug_for_global {
    (ignore: {$($ignore:ident,)+} fields: {$($field:ident,)+}) => {
        impl Debug for Global {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                let &Self { $($ignore: _,)+ $(ref $field,)+ } = self;

                f.debug_struct("Global")
                    $(.field(stringify!($field), $field))+
                    .finish()
            }
        }
    }
}
impl_debug_for_global!{
    ignore: {alto, alto_dev, alto_context, sdl, window, gl_context, }
    fields: {
        video, gl_simple_color_program,

        tags,
        palette_colors,
        palette_tags,
        meshes,
        other_scenes,
        saves,

        save_id,
        input,
        viewport_size,
    }
}

impl Global {
    pub fn handle_sdl2_event_before_new_tick(&mut self, event: &Event) {
        self.input.handle_sdl2_event_before_new_tick(event);
        match *event {
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                WindowEvent::SizeChanged(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                _ => (),
            },
            _ => (),
        };
    }
    pub fn reshape(&mut self, viewport_size: Extent2<u32>) {
        self.viewport_size = viewport_size;
    }

    pub fn replace_previous_state_by_current(&mut self) {
        self.input.begin_tick_and_event_processing();
    }
    pub fn render_clear(&self) {
        unsafe {
            gl::ClearColor(1f32, 0f32, 0f32, 1f32);
            gl::Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
        }
    }
    pub fn present(&self) {
        self.window.gl_swap_window();
    }
}




impl ButtonInput {
    pub fn was_just_pressed(&self) -> bool {
        if self.num_changes_since_last_tick_and_event_processing > 1 {
            return true;
        }
        self.is_pressed && !self.was_pressed
    }
    pub fn was_just_released(&self) -> bool {
        if self.num_changes_since_last_tick_and_event_processing > 1 {
            return true;
        }
        !self.is_pressed && self.was_pressed
    }
    pub fn begin_tick_and_event_processing(&mut self) {
        self.was_pressed = self.is_pressed;
        self.num_changes_since_last_tick_and_event_processing = 0;
    }
    pub fn press(&mut self) {
        if self.is_pressed { return; }
        self.num_changes_since_last_tick_and_event_processing += 1;
        self.is_pressed = true;
    }
    pub fn release(&mut self) {
        if !self.is_pressed { return; }
        self.num_changes_since_last_tick_and_event_processing += 1;
        self.is_pressed = false;
    }
}

impl MouseInput {
    pub fn begin_tick_and_event_processing(&mut self) {
        let &mut Self {
            ref mut left, ref mut middle, ref mut right,
            ref mut scroll,
            ref mut position, ref mut prev_position,
            has_prev_position: _,
        } = self;
        left.begin_tick_and_event_processing();
        middle.begin_tick_and_event_processing();
        right.begin_tick_and_event_processing();
        *scroll = Vec2::zero();
        *prev_position = *position;
    }
    pub fn set_position(&mut self, p: Vec2<u32>) {
        if !self.has_prev_position {
            self.prev_position = p;
        }
        self.position = p;
    }
    pub fn displacement(&self) -> Vec2<u32> {
        if !self.has_prev_position {
            return Vec2::zero();
        }
        self.position - self.prev_position
    }
}


impl KeyboardInput {
    pub fn begin_tick_and_event_processing(&mut self) {
        for key in self.keys.values_mut() {
            key.begin_tick_and_event_processing();
        }
    }
    pub fn key(&mut self, keycode: Keycode) -> &mut ButtonInput {
        self.keys.entry(keycode).or_insert(Default::default())
    }
}


impl Input {
    pub fn begin_tick_and_event_processing(&mut self) {
        let &mut Self { 
            wants_to_quit: _, ref mut text,
            ref mut mouse, ref mut keyboard, 
        } = self;
        text.clear();
        mouse.begin_tick_and_event_processing();
        keyboard.begin_tick_and_event_processing();
    }
    pub fn handle_sdl2_event_before_new_tick(&mut self, event: &Event) {
        match *event {
            Event::Quit {..} => {
                self.wants_to_quit = true;
            },
            // Event::TextEditing { text, start, length, .. } => {},
            Event::TextInput { ref text, .. } => {
                self.text += &text;
                info!("Text input \"{}\"", &text);
                info!("Total: \"{}\"", &self.text);
            },
            Event::KeyDown { keycode, scancode, keymod, repeat, .. } => {
                let _ = scancode;
                let _ = keymod;
                if !repeat {
                    if let Some(keycode) = keycode {
                        self.keyboard.key(keycode).press();
                    } else {
                        warn!("Some key was pressed, but keycode is None");
                    }
                }
            },
            Event::KeyUp { keycode, scancode, keymod, .. } => {
                let _ = scancode;
                let _ = keymod;
                if let Some(keycode) = keycode {
                    self.keyboard.key(keycode).release();
                } else {
                    warn!("Some key was pressed, but keycode is None");
                }
            },
            Event::MouseWheel { x, y, direction, .. } => {
                self.mouse.scroll += match direction {
                    MouseWheelDirection::Flipped => Vec2::new(-x as _, -y as _),
                    _ => Vec2::new(x as _, y as _),
                };
            },
            Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                let _ = clicks;
                self.mouse.set_position(Vec2::new(x as _, y as _));
                match mouse_btn {
                    MouseButton::Left => self.mouse.left.press(),
                    MouseButton::Middle => self.mouse.middle.press(),
                    MouseButton::Right => self.mouse.right.press(),
                    _ => (),
                };
            },
            Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
                let _ = clicks;
                self.mouse.set_position(Vec2::new(x as _, y as _));
                match mouse_btn {
                    MouseButton::Left => self.mouse.left.release(),
                    MouseButton::Middle => self.mouse.middle.release(),
                    MouseButton::Right => self.mouse.right.release(),
                    _ => (),
                };
            },
            Event::MouseMotion { mousestate, x, y, xrel, yrel, .. } => {
                let _ = mousestate;
                let _ = xrel;
                let _ = yrel;
                self.mouse.set_position(Vec2::new(x as _, y as _));
            },
            // TODO FIXME: MouseEnter and MouseLeave
            _ => (),
        };
    }
}

impl MeshResource {
    pub fn new_unit_quad(prog: &grx::SimpleColorProgram, label: &str) -> Self {
        assert_eq_size!(grx::SimpleColorVertex, [f32; 7]);

        let z = 0.;
        let s = 0.5_f32;
        let vertices = vec![
            grx::SimpleColorVertex { position: Vec3::new(-s, -s, z), color: Rgba::red() },
            grx::SimpleColorVertex { position: Vec3::new( s,  s, z), color: Rgba::yellow() },
            grx::SimpleColorVertex { position: Vec3::new(-s,  s, z), color: Rgba::green() },
            grx::SimpleColorVertex { position: Vec3::new(-s, -s, z), color: Rgba::blue() },
            grx::SimpleColorVertex { position: Vec3::new( s, -s, z), color: Rgba::cyan() },
            grx::SimpleColorVertex { position: Vec3::new( s,  s, z), color: Rgba::black() },
        ];
        let gl_topology = gl::TRIANGLES;
        let vao = gx::Vao::new();
        let vbo = gx::Vbo::new();
        vao.bind();
        vbo.bind();
        vao.set_label(&CString::new(label.to_owned() + " VAO").unwrap().into_bytes_with_nul());
        vbo.set_label(&CString::new(label.to_owned() + " VBO").unwrap().into_bytes_with_nul());
        vbo.set_data(&vertices, gx::UpdateHint::Occasionally);
        unsafe {
            gl::EnableVertexAttribArray(prog.a_position());
            gl::EnableVertexAttribArray(prog.a_color());
            gl::VertexAttribPointer(
                prog.a_position(), 3, gl::FLOAT,
                gl::FALSE as _, 7*4, ptr::null()
            );
            gl::VertexAttribPointer(
                prog.a_color(), 4, gl::FLOAT,
                gl::FALSE as _, 7*4, ptr::null::<GLvoid>().offset(3*4)
            );
        }

        Self {
            vertices, gl_topology, vbo, vao,
        }
    }
    pub fn update_vbo(&self) {
        self.vbo.set_data(&self.vertices, gx::UpdateHint::Occasionally);
    }
}

impl Default for Save {
    fn default() -> Self {
        Self {
            scene_id: SceneID::from_raw(0),
            has_unlocked_door: false,
        }
    }
}

impl Scene {
    pub fn new_test_room(viewport: Rect<u32, u32>) -> Self {
        let mut mesh_instances = ecs::MeshInstances::default();
        let mut transforms = ecs::Transforms::default();
        let mut cameras = ecs::Cameras::default();

        let camera_id = ecs::EntityID::from_raw(0);
        let quad_id = ecs::EntityID::from_raw(1);

        transforms.insert(camera_id, ecs::TransformComponent::default());
        cameras.insert(camera_id, {
            let camera = OrthographicCamera {
                viewport, ortho_right: 1., near: 0.01, far: 100.,
            };
            ecs::OrthographicCameraComponent::from(camera)
        });

        transforms.insert(quad_id, {
            let mut xform = Transform::default();
            xform.position.z = 1.;
            xform.scale /= 20.;
            ecs::TransformComponent::from(xform)
        });
        mesh_instances.insert(quad_id, MeshResourceID::from_raw(0));

        Self {
            mesh_instances, transforms, cameras,
            allows_quitting: true,
        }
    }
    pub fn handle_sdl2_event_before_new_tick(&mut self, event: &Event) {
        match *event {
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                WindowEvent::SizeChanged(w, h) => {
                    self.reshape(Extent2::new(w as _, h as _));
                },
                _ => (),
            },
            _ => (),
        };
    }
    pub fn reshape(&mut self, viewport_size: Extent2<u32>) {
        for camera in self.cameras.values_mut() {
            // NOTE: Every camera might want to handle this differently
            let vp = Rect::from((Vec2::zero(), viewport_size));
            camera.current.viewport = vp;
            camera.previous.viewport = vp;
            camera.render.viewport = vp;
        }
    }
    pub fn replace_previous_state_by_current(&mut self) {
        for xform in self.transforms.values_mut() {
            xform.previous = xform.current;
        }
    }
    pub fn integrate(&mut self, tick: TickInfo) {
        let dt = tick.dt.to_f64_seconds() as f32;
        // XXX Entity 0 is the lucky one
        let xform = &mut self.transforms.get_mut(&ecs::EntityID::from_raw(0)).unwrap().current;
        xform.position.x += 0.1 * dt;
    }
    pub fn render(&mut self, frame: FrameInfo) {
        // First, create render state by LERP-ing previous and current.
        let alpha = frame.lerp_factor_between_previous_and_current as f32;
        for xform in self.transforms.values_mut() {
            xform.render = Lerp::lerp(xform.previous, xform.current, alpha);
        }

        // Then render everything !
        for (camera_eid, camera) in self.cameras.iter().map(|(id, c)| (id, &c.render)) {
            let camera_xform = &self.transforms[camera_eid].render;
            let view = camera.view_matrix(camera_xform);
            let proj = camera.proj_matrix();
            let Rect { x, y, w, h } = camera.viewport;
            unsafe {
                gl::Viewport(x as _, y as _, w as _, h as _);
            }
            frame.g.gl_simple_color_program.use_program(&Mat4::identity());
            for (mesh_eid, mesh_resource_id) in self.mesh_instances.iter() {
                let mesh = frame.g.meshes[*mesh_resource_id].as_ref().unwrap();
                // let mesh_xform = self.transforms[mesh_eid].render;
                let mesh_xform = {
                    let mut xform = Transform::default();
                    let mut mousepos = frame.g.input.mouse.position.map(|x| x as f32);
                    mousepos.y = frame.g.viewport_size.h as f32 - mousepos.y;
                    xform.position = camera.viewport_to_world_point(
                        camera_xform, mousepos.into()
                    );
                    xform.orientation.rotate_z(frame.frame_i as f32 / 20.);
                    xform.position.z = 1.;
                    xform.scale /= 24.;
                    xform
                };
                let model = Mat4::from(mesh_xform);
                let mvp = proj * view * model;
                frame.g.gl_simple_color_program.set_uniform_mvp(&mvp);
                mesh.vao.bind();
                unsafe {
                    gl::DrawArrays(mesh.gl_topology, 0, mesh.vertices.len() as _);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct TickInfo<'a> {
    pub tick_i: u64,
    pub t: Duration,
    pub dt: Duration,
    pub g: &'a mut Global,
}

#[derive(Debug)]
pub struct FrameInfo<'a> {
    pub frame_i: u64,
    pub lerp_factor_between_previous_and_current: f64,
    pub g: &'a mut Global,
}

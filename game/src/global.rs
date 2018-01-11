use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::fs;
use std::fs::{DirEntry, ReadDir};
use std::time::Duration;
use std::fmt::{self, Formatter, Debug};
use env_logger;
use log::LevelFilter;
use sdl2;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::video::{Window, GLContext, GLProfile, SwapInterval};
use sdl2::event::{Event, WindowEvent};
use alto;
use alto::Alto;
use gl;
use gx;
use grx;
use v::{Rgb, Rgba, Extent2};
use scene::Scene;
use input::Input;
use mesh::{Mesh, FontAtlasMesh};
use ids::*;
use lazy::Lazy;
use save::Save;
use fonts::Fonts;


pub struct Global {
    // Runtime
    pub path_to_res: PathBuf,
    pub path_to_saves: PathBuf,
    pub alto: Alto,
    pub alto_dev: alto::OutputDevice,
    pub alto_context: alto::Context,
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub gl_context: GLContext,
    pub gl_simple_color_program: grx::SimpleColorProgram,
    pub gl_text_program: grx::TextProgram,
    pub fonts: Fonts,

    // Persistent data, ID domains
    pub font_atlas_mesh: FontAtlasMesh,
    pub tags: TagIDRealm<String>,
    pub palette_colors: PaletteEntryIDRealm<Rgb<u8>>,
    pub palette_tags: TagIDMap<HashSet<PaletteEntryID>>,
    pub meshes: MeshIDRealm<Lazy<Mesh>>,
    pub other_scenes: SceneIDRealm<Lazy<Scene>>,
    pub saves: SaveIDRealm<Save>,

    // Current state, current IDs
    pub save_id: SaveID,
    // Current state, ctd.
    pub input: Input,
    pub window_size: Extent2<u32>,
}


#[derive(Debug)]
pub struct GlobalDataUpdatePack<'a> {
    pub tick_i: u64,
    pub frame_i: u64,
    pub t: Duration,
    pub dt: Duration,
    pub g: &'a mut Global,
}


fn setup_panic_hook() {
    use ::std::panic;

    panic::set_hook(Box::new(|info| {

        let mut msg = match info.location() {
            Some(location) => format!("Panic occurred in file '{}' at line {}:\n", location.file(), location.line()),
            None => format!("Panic occurred in unknown location:\n"),
        };

        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            msg += payload;
        } else {
            msg += "<unknown reason>";
        }

        error!("{}", &msg);

        info!("Backtrace:");
        ::backtrace::trace(|frame| {
            let ip = frame.ip();
            let _symbol_address = frame.symbol_address();

            ::backtrace::resolve(ip, |symbol| {
                let what = || "??".to_owned();
                let filename = if let Some(filename) = symbol.filename() { format!("{}", filename.display()) } else { what() };
                let lineno = if let Some(lineno) = symbol.lineno() { format!("{}", lineno) } else { what() };
                let addr = if let Some(addr) = symbol.addr() { format!("0x{:8x}", addr as usize) } else { what() };
                let name = if let Some(name) = symbol.name() { format!("{}", name) } else { what() };
                // ^ NOTE: Do use the Display implementation for name. It demangles the symbol.
                info!("{}:{}: ({}) {}", &filename, &lineno, &addr, name);
            });

            true // keep going to the next frame
        });

        use sdl2::messagebox;
        let flags = messagebox::MESSAGEBOX_ERROR;
        let result = messagebox::show_simple_message_box(
            flags, "Fatal error", &msg, None
        );
        if let Err(e) = result {
            use sdl2::messagebox::ShowMessageError::*;
            let msg = "Couldn't display message box: ".to_owned() + &match e {
                InvalidTitle(nul_error) => format!("Invalid title: {:?}", nul_error),
                InvalidMessage(nul_error) => format!("Invalid message: {:?}", nul_error),
                InvalidButton(nul_error, i) => format!("Invalid button {}: {:?}", i, nul_error),
                SdlError(msg) => format!("SDL2 error: {}", &msg),
            };
            error!("{}", &msg);
        }
    }));
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

        fn check_if_has_res_content(parent: DirEntry, entries: ReadDir) -> Option<PathBuf> {
            let mut expected = vec![
                ("fonts", true),
                ("sounds", true),
                ("musics", true),
                ("meshes", true),
                ("palette.txt", false),
            ];
            for path in entries.filter(Result::is_ok).map(Result::unwrap).map(|x| x.path()) {
                let (is_file, is_dir) = (path.is_file(), path.is_dir());
                if !is_file && !is_dir {
                    continue;
                }
                expected.retain(|e| !(path.ends_with(e.0) && ((e.1 && is_dir) || (!e.1 && is_file))));
            }
            if expected.is_empty() {
                return Some(parent.path().to_path_buf());
            }
            let names = expected.iter().map(|x| x.0).collect::<Vec<_>>();
            warn!("res/ folder misses {:?}", names.as_slice());
            None
        }

        fn check_if_res(entry: DirEntry) -> Option<PathBuf> {
            let p = entry.path();
            if p.ends_with("res") && p.is_dir() {
                info!("Found candidate `res/` folder at `{}`", p.display());
                if let Ok(entries) = fs::read_dir(p) {
                    return check_if_has_res_content(entry, entries);
                }
            }
            None
        }

        fn look_for_res(entries: ReadDir) -> Option<PathBuf> {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(res_path) = check_if_res(entry) {
                        return Some(res_path);
                    }
                }
            }
            None
        }

        setup_panic_hook();
        setup_env();
        setup_log();

        let mut path = match env::current_exe() {
            Ok(p) => {
                info!("Path of current executable is: {}", p.display());
                p.parent().unwrap().to_path_buf()
            },
            Err(e) => {
                error!("Failed to get current exe path: {}", e);
                let p = env::current_dir().unwrap();
                info!("Starting from `{}`", p.display());
                p
            },
        };

        let path_to_res = loop {
            if let Ok(entries) = fs::read_dir(&path) {
                if let Some(res_path) = look_for_res(entries) {
                    break res_path;
                }
            }
            if let Some(_) = path.parent() {
                info!("Couldn't find `res/` in `{}`", path.display());
                path.pop();
                info!("Trying in `{}`...", path.display());
                continue; 
            }
            panic!("Couldn't find resource folder!");
        };

        info!("Resource path located at `{}`", path_to_res.display());

        let mut path_to_saves = path_to_res.clone();
        path_to_saves.pop();
        path_to_saves.push("saves");
        assert!(path_to_saves.is_dir());
        info!("Saves path located at `{}`", path_to_saves.display());

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

        let window = video.window("Grisui - Prelude", 800, 480)
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
        let gl_text_program = grx::TextProgram::new();

        let mut path_to_fonts = path_to_res.clone();
        path_to_fonts.push("fonts");
        let fonts = Fonts::from_path(&path_to_fonts).unwrap();

        let input = Input::default();

        let tags = TagIDRealm::from_iterator(
            ["a", "b", "c"].iter().enumerate().map(|(i, s)| {
                (TagID::from_raw(i as _), s.to_string())
            })
        );

        let palette_tags = TagIDMap::default();
        let palette_colors = PaletteEntryIDRealm::from_iterator(
            [Rgb::red(), Rgb::green()].iter().enumerate().map(|(i, rgba)| {
                (PaletteEntryID::from_raw(i as _), *rgba)
            })
        );

        let mesh_resources = vec![
            Lazy::Loaded(Mesh::new_colored_quad(
                &gl_simple_color_program, "Lucky Quad", gx::UpdateHint::Occasionally,
                Rgba::red(), Rgba::green(), Rgba::blue(), Rgba::yellow(), 0.5
            )),
            Lazy::Loaded(Mesh::new_unit_disk(
                &gl_simple_color_program, "Red Disk", gx::UpdateHint::Occasionally, 3, Rgba::red()
            )),
            Lazy::Loaded(Mesh::new_unit_disk(
                &gl_simple_color_program, "Blue Disk", gx::UpdateHint::Occasionally, 64, Rgba::blue()
            )),
        ];
        let meshes = MeshIDRealm::from_iterator(
            mesh_resources.into_iter().enumerate().map(|(i, mesh)| {
                (MeshID::from_raw(i as _), mesh)
            })
        );

        let font_atlas_mesh = FontAtlasMesh::new_font_atlas_unit_quad(
            &gl_text_program, "Text Atlas", gx::UpdateHint::Never
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

        let window_size = Extent2::from(window.drawable_size());

        let mut g = Self {
            path_to_res, path_to_saves,
            alto, alto_dev, alto_context,
            sdl, video, window, gl_context, 
            gl_simple_color_program,
            gl_text_program,
            fonts,

            font_atlas_mesh,
            tags,
            palette_colors,
            palette_tags,
            meshes,
            other_scenes,
            saves,

            save_id,
            input,
            window_size,
        };
        g.reshape(window_size);
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
        path_to_res, path_to_saves,
        video, gl_simple_color_program, gl_text_program,
        fonts,

        font_atlas_mesh,
        tags,
        palette_colors,
        palette_tags,
        meshes,
        other_scenes,
        saves,

        save_id,
        input,
        window_size,
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
    pub fn reshape(&mut self, window_size: Extent2<u32>) {
        self.window_size = window_size;
    }

    pub fn replace_previous_state_by_current(&mut self) {
        self.input.begin_tick_and_event_processing();
    }
    pub fn render_clear(&self, clear_color: Rgb<u8>) {
        unsafe {
            let Rgb { r, g, b } = clear_color.map(|c| (c as f32)/255.);
            gl::ClearColor(r, g, b, 1.);
            gl::Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
        }
    }
    pub fn present(&self) {
        self.window.gl_swap_window();
    }
}


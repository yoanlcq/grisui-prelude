use std::fs::{self, ReadDir, DirEntry};
use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Paths {
    pub res: PathBuf,
    pub fonts: PathBuf,
    pub saves: PathBuf,
}

impl Paths {
    pub fn new() -> Self {
    
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
            warn!("Paths: res/ folder misses {:?}", names.as_slice());
            None
        }

        fn check_if_res(entry: DirEntry) -> Option<PathBuf> {
            let p = entry.path();
            if p.ends_with("res") && p.is_dir() {
                info!("Paths: Found candidate `res/` folder at `{}`", p.display());
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

        let mut path = match env::current_exe() {
            Ok(p) => {
                info!("Paths: Path of current executable is: {}", p.display());
                p.parent().unwrap().to_path_buf()
            },
            Err(e) => {
                error!("Paths: Failed to get current exe path: {}", e);
                let p = env::current_dir().unwrap();
                info!("Paths: Starting from `{}`", p.display());
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
                info!("Paths: Couldn't find `res/` in `{}`", path.display());
                path.pop();
                info!("Paths: Trying in `{}`...", path.display());
                continue; 
            }
            panic!("Couldn't find resource folder!");
        };

        info!("Paths: Resource path located at `{}`", path_to_res.display());

        let mut path_to_saves = path_to_res.clone();
        path_to_saves.pop();
        path_to_saves.push("saves");
        assert!(path_to_saves.is_dir());
        info!("Paths: Saves path located at `{}`", path_to_saves.display());

        let mut path_to_fonts = path_to_res.clone();
        path_to_fonts.push("fonts");
        assert!(path_to_fonts.is_dir());
        info!("Paths: Fonts path located at `{}`", path_to_fonts.display());

        Self {
            res: path_to_res,
            fonts: path_to_fonts,
            saves: path_to_saves,
        }
    }
}

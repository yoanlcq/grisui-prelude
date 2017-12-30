#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Lazy<T> {
    Loaded(T),
    Unloaded,
}

use self::Lazy::*;

impl<T> Default for Lazy<T> {
    fn default() -> Self {
        Unloaded
    }
}

impl<T> From<Option<T>> for Lazy<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(x) => Loaded(x),
            None => Unloaded,
        }
    }
}
impl<T> Lazy<T> {
    pub fn into_option(self) -> Option<T> {
        match self {
            Loaded(x) => Some(x),
            Unloaded => None,
        }
    }
    pub fn is_loaded(&self) -> bool {
        if let Loaded(_) = *self { true } else { false }
    }
    pub fn is_unloaded(&self) -> bool {
        !self.is_loaded()
    }
    pub fn set(&mut self, value: T) {
        *self = Loaded(value);
    }
    pub fn unload(&mut self) {
        *self = Unloaded;
    }
    pub fn as_ref(&self) -> Lazy<&T> {
        match *self {
            Loaded(ref x) => Loaded(x),
            Unloaded => Unloaded,
        }
    }
    pub fn as_mut(&mut self) -> Lazy<&mut T> {
        match *self {
            Loaded(ref mut x) => Loaded(x),
            Unloaded => Unloaded,
        }
    }
    pub fn unwrap(self) -> T {
        self.expect("Tried to unwrap() an unloaded value")
    }
    pub fn expect(self, msg: &str) -> T {
        if let Loaded(x) = self {
            x
        } else {
            panic!("{}", msg);
        }
    }
}

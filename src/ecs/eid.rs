use std::collections::HashMap;

type Uint = u32;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct EID(Uint);

pub type EIDMap<T> = HashMap<EID, T>;

impl EID {
    pub fn from_raw(u: Uint) -> Self {
        EID(u)
    }
    pub fn to_raw(&self) -> Uint {
        self.0
    }
}

use ids::SceneID;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Save {
    pub scene_id: SceneID,
    pub has_unlocked_door: bool,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            scene_id: SceneID::from_raw(0),
            has_unlocked_door: false,
        }
    }
}


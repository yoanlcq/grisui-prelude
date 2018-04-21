#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    EditorToggleGrid,
    EditorToggleDrawGridFirst,
    EditorBeginPanCameraViaMouse,
    EditorEndPanCameraViaMouse,
}


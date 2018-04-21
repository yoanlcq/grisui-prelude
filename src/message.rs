#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    EnterEditor,
    LeaveEditor,
    EditorToggleGrid,
    EditorToggleDrawGridFirst,
    EditorBeginPanCameraViaMouse,
    EditorEndPanCameraViaMouse,
    EditorBeginRotateCameraLeft,
    EditorBeginRotateCameraRight,
    EditorEndRotateCamera,
    EditorRecenterCamera,
    EditorResetCameraRotation,
    EditorResetCameraZoom,

    EnterGameplay,
    LeaveGameplay,
}


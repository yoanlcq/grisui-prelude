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
    EditorAddVertexAtCurrentMousePosition,
    EditorEndPolygon,
    EditorToggleSelectAll,
    EditorDeleteSelected,

    EditorBeginSlideHue { speed: f32 },
    EditorEndSlideHue,
    EditorBeginSlideSaturation { speed: f32 },
    EditorEndSlideSaturation,
    EditorBeginSlideValue { speed: f32 },
    EditorEndSlideValue,
    EditorBeginSlideAlpha { speed: f32 },
    EditorEndSlideAlpha,

    EditorBeginEnterCommand,
    EditorCancelCommand,
    EditorConfirmCommand,


    EnterGameplay,
    LeaveGameplay,
}


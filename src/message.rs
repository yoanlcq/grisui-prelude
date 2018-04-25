#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    EnterEditor,
    LeaveEditor,

    EnterGameplay,
    LeaveGameplay,
}


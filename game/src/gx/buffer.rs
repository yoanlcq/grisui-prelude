use gl;

/// STREAM: The data store contents will be modified once and used at most a few times.
/// STATIC: The data store contents will be modified once and used many times.
/// DYNAMIC: The data store contents will be modified repeatedly and used many times.
/// 
/// DRAW: The data store contents are modified by the application, and used as the source for GL drawing and image specification commands.
/// READ: The data store contents are modified by reading data from the GL, and used to return that data when queried by the application.
/// COPY: The data store contents are modified by reading data from the GL, and used as the source for GL drawing and image specification commands.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Usage {
    StreamDraw  = gl::STREAM_DRAW,
    StreamRead  = gl::STREAM_READ,
    StreamCopy  = gl::STREAM_COPY,
    StaticDraw  = gl::STATIC_DRAW,
    StaticRead  = gl::STATIC_READ,
    StaticCopy  = gl::STATIC_COPY,
    DynamicDraw = gl::DYNAMIC_DRAW,
    DynamicRead = gl::DYNAMIC_READ,
    DynamicCopy = gl::DYNAMIC_COPY,
}

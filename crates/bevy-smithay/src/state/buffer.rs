use {
    super::SmithayAppRunnerState,
    smithay::{
        reexports::wayland_server::protocol::wl_buffer::WlBuffer, wayland::buffer::BufferHandler,
    },
};

impl BufferHandler for SmithayAppRunnerState {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

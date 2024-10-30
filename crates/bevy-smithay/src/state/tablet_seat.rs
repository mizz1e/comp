use {
    super::SmithayAppRunnerState,
    smithay::{
        backend::input::TabletToolDescriptor, input::pointer::CursorImageStatus,
        wayland::tablet_manager::TabletSeatHandler,
    },
};

impl TabletSeatHandler for SmithayAppRunnerState {
    fn tablet_tool_image(&mut self, _tool: &TabletToolDescriptor, _image: CursorImageStatus) {}
}

smithay::delegate_tablet_manager!(SmithayAppRunnerState);

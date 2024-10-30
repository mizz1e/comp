use {
    super::SmithayAppRunnerState,
    smithay::wayland::selection::data_device::{
        ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
    },
};

impl ClientDndGrabHandler for SmithayAppRunnerState {
    fn dropped(&mut self, _seat: smithay::input::Seat<Self>) {}

    fn started(
        &mut self,
        _source: Option<smithay::reexports::wayland_server::protocol::wl_data_source::WlDataSource>,
        _icon: Option<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
        _eat: smithay::input::Seat<Self>,
    ) {
    }
}

impl ServerDndGrabHandler for SmithayAppRunnerState {}

impl DataDeviceHandler for SmithayAppRunnerState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.smithay_state.data_device_state
    }
}

smithay::delegate_data_device!(SmithayAppRunnerState);

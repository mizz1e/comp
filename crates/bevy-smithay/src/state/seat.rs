use {
    super::SmithayAppRunnerState,
    smithay::{
        input::{SeatHandler, SeatState},
        reexports::wayland_server::protocol::wl_surface::WlSurface,
    },
};

impl SeatHandler for SmithayAppRunnerState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.smithay_state.seat_state
    }
}

smithay::delegate_seat!(SmithayAppRunnerState);

use {
    super::SmithayAppRunnerState,
    smithay::{
        desktop::Window,
        reexports::wayland_server::protocol::wl_seat::WlSeat,
        utils::Serial,
        wayland::shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler},
    },
};

impl XdgShellHandler for SmithayAppRunnerState {
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.smithay_state.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);

        self.smithay_state.space.map_element(window, (0, 0), false);
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {}

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {}

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
    }
}

smithay::delegate_xdg_shell!(SmithayAppRunnerState);

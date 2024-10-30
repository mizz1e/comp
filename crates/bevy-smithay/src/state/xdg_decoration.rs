use {
    super::SmithayAppRunnerState,
    smithay::{
        reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
        wayland::shell::xdg::{decoration::XdgDecorationHandler, ToplevelSurface},
    },
};

impl XdgDecorationHandler for SmithayAppRunnerState {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        configure(toplevel);
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, _mode: Mode) {
        configure(toplevel);
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        configure(toplevel);
    }
}

fn configure(toplevel: ToplevelSurface) {
    toplevel.with_pending_state(|state| {
        state.decoration_mode = Some(Mode::ServerSide);
    });

    if toplevel.is_initial_configure_sent() {
        toplevel.send_pending_configure();
    }
}

smithay::delegate_xdg_decoration!(SmithayAppRunnerState);

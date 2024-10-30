use {
    super::SmithayAppRunnerState,
    smithay::wayland::xdg_foreign::{XdgForeignHandler, XdgForeignState},
};

impl XdgForeignHandler for SmithayAppRunnerState {
    fn xdg_foreign_state(&mut self) -> &mut XdgForeignState {
        &mut self.smithay_state.xdg_foreign_state
    }
}

smithay::delegate_xdg_foreign!(SmithayAppRunnerState);

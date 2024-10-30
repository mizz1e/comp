use {
    super::SmithayAppRunnerState,
    smithay::wayland::selection::wlr_data_control::{DataControlHandler, DataControlState},
};

impl DataControlHandler for SmithayAppRunnerState {
    fn data_control_state(&self) -> &DataControlState {
        &self.smithay_state.data_control_state
    }
}

smithay::delegate_data_control!(SmithayAppRunnerState);

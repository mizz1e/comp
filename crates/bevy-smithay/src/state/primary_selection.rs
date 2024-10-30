use {
    super::SmithayAppRunnerState,
    smithay::wayland::selection::primary_selection::{
        PrimarySelectionHandler, PrimarySelectionState,
    },
};

impl PrimarySelectionHandler for SmithayAppRunnerState {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.smithay_state.primary_selection_state
    }
}

smithay::delegate_primary_selection!(SmithayAppRunnerState);

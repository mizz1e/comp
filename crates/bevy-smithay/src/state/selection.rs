use {super::SmithayAppRunnerState, smithay::wayland::selection::SelectionHandler};

impl SelectionHandler for SmithayAppRunnerState {
    type SelectionUserData = ();
}

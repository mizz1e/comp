use {super::SmithayAppRunnerState, smithay::wayland::output::OutputHandler};

impl OutputHandler for SmithayAppRunnerState {}

smithay::delegate_output!(SmithayAppRunnerState);

use {
    super::SmithayAppRunnerState,
    smithay::wayland::shm::{ShmHandler, ShmState},
};

impl ShmHandler for SmithayAppRunnerState {
    fn shm_state(&self) -> &ShmState {
        println!("shm state");
        &self.smithay_state.shm_state
    }
}

smithay::delegate_shm!(SmithayAppRunnerState);

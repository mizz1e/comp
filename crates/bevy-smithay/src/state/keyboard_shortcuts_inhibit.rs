use {
    super::SmithayAppRunnerState,
    smithay::wayland::keyboard_shortcuts_inhibit::{
        KeyboardShortcutsInhibitHandler, KeyboardShortcutsInhibitState, KeyboardShortcutsInhibitor,
    },
};

impl KeyboardShortcutsInhibitHandler for SmithayAppRunnerState {
    fn keyboard_shortcuts_inhibit_state(&mut self) -> &mut KeyboardShortcutsInhibitState {
        &mut self.smithay_state.keyboard_shortcuts_inhibit_state
    }

    fn new_inhibitor(&mut self, _inhibitor: KeyboardShortcutsInhibitor) {}
}

smithay::delegate_keyboard_shortcuts_inhibit!(SmithayAppRunnerState);

use {
    super::SmithayAppRunnerState,
    smithay::{
        reexports::wayland_server::protocol::wl_surface::WlSurface,
        utils::{Logical, Rectangle},
        wayland::input_method::{InputMethodHandler, PopupSurface},
    },
};

impl InputMethodHandler for SmithayAppRunnerState {
    fn new_popup(&mut self, _surface: PopupSurface) {}
    fn popup_repositioned(&mut self, _surface: PopupSurface) {}
    fn dismiss_popup(&mut self, _surface: PopupSurface) {}
    fn parent_geometry(&self, _parent: &WlSurface) -> Rectangle<i32, Logical> {
        Rectangle::default()
    }
}

smithay::delegate_input_method_manager!(SmithayAppRunnerState);

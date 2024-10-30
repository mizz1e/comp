use {
    self::state::SmithayAppRunnerState,
    bevy::{app::PluginsState, prelude::*, render::extract_resource::ExtractResourcePlugin},
    external_image::ExternalImagePlugin,
    smithay::reexports::calloop::EventLoop,
};

pub mod external_image;
pub mod state;
pub mod util;

pub struct SmithayPlugin;

impl Plugin for SmithayPlugin {
    fn build(&self, app: &mut App) {
        let event_loop = match EventLoop::<SmithayAppRunnerState>::try_new() {
            Ok(event_loop) => event_loop,
            Err(error) => {
                error!("failed to create event loop: {error}");

                return;
            }
        };

        app.add_plugins(ExternalImagePlugin)
            .insert_non_send_resource(event_loop)
            .set_runner(smithay_runner);
    }
}

pub fn smithay_runner(mut app: App) -> AppExit {
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();
    }

    let mut event_loop = app
        .world_mut()
        .remove_non_send_resource::<EventLoop<SmithayAppRunnerState>>()
        .unwrap();

    app.world_mut()
        .insert_non_send_resource(event_loop.handle());

    let mut runner_state = match SmithayAppRunnerState::try_new(&mut event_loop, app) {
        Ok(runner_state) => runner_state,
        Err(error) => {
            error!("{error}");

            return AppExit::error();
        }
    };

    //runner_state.start_xwayland();

    runner_state.run(&mut event_loop)
}

use {
    super::util,
    bevy::{
        app::PluginsState,
        prelude::*,
        render::{
            camera::{ManualTextureViewHandle, ManualTextureViews, RenderTarget},
            extract_resource::ExtractResource,
            renderer::RenderDevice,
            texture::GpuImage,
        },
        utils::HashMap,
    },
    smithay::{
        backend::{
            allocator::{
                self,
                gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
                Fourcc, Modifier,
            },
            drm::{
                gbm::Error as GbmError, DrmDevice, DrmDeviceFd, DrmError, DrmEvent, DrmNode,
                GbmBufferedSurface, PlaneClaim,
            },
            input::{Event, InputEvent, KeyboardKeyEvent as _},
            libinput::{LibinputInputBackend, LibinputSessionInterface},
            session::{libseat::LibSeatSession, Session},
            udev::{UdevBackend, UdevEvent},
        },
        desktop::{PopupManager, Space, Window, WindowSurfaceType},
        input::{keyboard::XkbConfig, Seat, SeatState},
        output::{Mode, Output, PhysicalProperties, Subpixel},
        reexports::{
            calloop::{generic::Generic, EventLoop, InsertError, Interest, PostAction},
            gbm::{DeviceDestroyedError, FdError},
            input::{event::keyboard::KeyboardKeyEvent, Libinput},
            rustix::fs::OFlags,
            wayland_server::{
                backend::{ClientData, ClientId, DisconnectReason, InitError},
                protocol::wl_shm::Format,
                BindError, Display, DisplayHandle,
            },
        },
        utils::{DeviceFd, Size, Transform, SERIAL_COUNTER},
        wayland::{
            compositor::{CompositorClientState, CompositorState},
            dmabuf::{DmabufFeedbackBuilder, DmabufGlobal, DmabufState},
            keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitState,
            output::OutputManagerState,
            security_context::SecurityContext,
            selection::{
                data_device::DataDeviceState, primary_selection::PrimarySelectionState,
                wlr_data_control::DataControlState,
            },
            shell::xdg::XdgShellState,
            shm::ShmState,
            socket::ListeningSocketSource,
            xdg_foreign::XdgForeignState,
        },
    },
    smithay_drm_extras::drm_scanner::{DrmScanner, SimpleCrtcMapper},
    std::{
        io, iter,
        time::{Duration, Instant},
    },
};

mod buffer;
mod compositor;
mod data_control;
mod data_device;
mod dmabuf;
mod input_method;
mod keyboard_shortcuts_inhibit;
mod output;
mod primary_selection;
mod seat;
mod security_context;
mod selection;
mod shm;
mod tablet_seat;
mod xdg_decoration;
mod xdg_foreign;
mod xdg_shell;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("wayland display: {0}")]
    Display(#[from] InitError),

    #[error("seat session: {0}")]
    Seat(#[from] smithay::backend::session::libseat::Error),

    #[error("udev backend: {0}")]
    Udev(io::Error),

    #[error("udev assign seat")]
    UdevAssignSeat(()),

    #[error("wayland socket: {0}")]
    Socket(#[from] BindError),

    #[error("insert event: {0}")]
    InsertError(#[from] smithay::reexports::calloop::error::Error),

    #[error("drm: {0}")]
    Drm(#[from] DrmError),

    #[error("drm scan: {0}")]
    DrmScan(io::Error),

    #[error("gbm: {0}")]
    Gbm(io::Error),

    #[error("gbm2: {0}")]
    Gbm2(#[from] GbmError),

    #[error("gbm3: {0}")]
    Gbm3(#[from] DeviceDestroyedError),

    #[error("gbm4: {0}")]
    Gbm4(#[from] FdError),
}

impl<T> From<InsertError<T>> for Error {
    fn from(error: InsertError<T>) -> Self {
        Self::InsertError(error.into())
    }
}

#[derive(Component)]
pub struct DiagnosticText;

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct MainTexture(pub RenderTarget);

#[derive(Debug, Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    pub security_context: Option<SecurityContext>,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub struct SmithayState {
    pub compositor_state: CompositorState,
    pub data_control_state: DataControlState,
    pub data_device_state: DataDeviceState,
    pub dmabuf_global: DmabufGlobal,
    pub dmabuf_state: DmabufState,
    pub keyboard_shortcuts_inhibit_state: KeyboardShortcutsInhibitState,
    pub output: Output,
    pub output_manager_state: OutputManagerState,
    pub primary_selection_state: PrimarySelectionState,
    pub popup_manager: PopupManager,
    pub seat: Seat<SmithayAppRunnerState>,
    pub seat_state: SeatState<SmithayAppRunnerState>,
    pub shm_state: ShmState,
    pub space: Space<Window>,
    pub xdg_foreign_state: XdgForeignState,
    pub xdg_shell_state: XdgShellState,
    pub start_time: Instant,
}

impl SmithayState {
    pub fn new(display_handle: &DisplayHandle, drm_node: DrmNode, seat_name: &str) -> Self {
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(display_handle, seat_name);

        let default_feedback = DmabufFeedbackBuilder::new(
            drm_node.dev_id(),
            [
                allocator::Format {
                    code: Fourcc::Abgr8888,
                    modifier: Modifier::Linear,
                },
                allocator::Format {
                    code: Fourcc::Xrgb8888,
                    modifier: Modifier::Linear,
                },
            ],
        )
        .build()
        .unwrap();

        let _pointer = seat.add_pointer();
        let _keyboard = seat.add_keyboard(XkbConfig::default(), 250, 45);

        let compositor_state = CompositorState::new::<SmithayAppRunnerState>(display_handle);
        let data_control_state =
            DataControlState::new::<SmithayAppRunnerState, _>(display_handle, None, |_client| {
                false
            });

        let data_device_state = DataDeviceState::new::<SmithayAppRunnerState>(display_handle);
        let mut dmabuf_state = DmabufState::new();
        let dmabuf_global = dmabuf_state
            .create_global_with_default_feedback::<SmithayAppRunnerState>(
                display_handle,
                &default_feedback,
            );

        let output_manager_state =
            OutputManagerState::new_with_xdg_output::<SmithayAppRunnerState>(display_handle);

        let keyboard_shortcuts_inhibit_state =
            KeyboardShortcutsInhibitState::new::<SmithayAppRunnerState>(display_handle);

        let popup_manager = PopupManager::default();

        let primary_selection_state =
            PrimarySelectionState::new::<SmithayAppRunnerState>(display_handle);

        let shm_state = ShmState::new::<SmithayAppRunnerState>(
            display_handle,
            [Format::Argb8888, Format::Xrgb8888],
        );

        let mut space = Space::default();

        let xdg_foreign_state = XdgForeignState::new::<SmithayAppRunnerState>(display_handle);
        let xdg_shell_state = XdgShellState::new::<SmithayAppRunnerState>(display_handle);

        let mode = Mode {
            size: Size::from((2560, 1440)),
            refresh: 60_000,
        };

        let output = Output::new(
            "winit".to_string(),
            PhysicalProperties {
                size: (2560, 1440).into(),
                subpixel: Subpixel::Unknown,
                make: "comp".into(),
                model: "comp".into(),
            },
        );

        let _global = output.create_global::<SmithayAppRunnerState>(display_handle);

        output.change_current_state(
            Some(mode),
            Some(Transform::Flipped180),
            None,
            Some((2560, 1440).into()),
        );

        output.set_preferred(mode);

        space.map_output(&output, (2560, 1440));

        let start_time = Instant::now();

        Self {
            compositor_state,
            data_control_state,
            data_device_state,
            dmabuf_global,
            dmabuf_state,
            keyboard_shortcuts_inhibit_state,
            output,
            output_manager_state,
            popup_manager,
            primary_selection_state,
            seat,
            seat_state,
            space,
            shm_state,
            xdg_foreign_state,
            xdg_shell_state,
            start_time,
        }
    }
}

pub struct SmithayAppRunnerState {
    pub app: App,
    pub display_handle: DisplayHandle,
    pub smithay_state: SmithayState,
    pub drm_device: DrmDevice,
    pub drm_node: DrmNode,
    pub drm_scanner: DrmScanner<SimpleCrtcMapper>,
    pub drm_plane_claim: PlaneClaim,
    pub gbm_device: GbmDevice<DrmDeviceFd>,
    pub gbm_surface: GbmBufferedSurface<GbmAllocator<DrmDeviceFd>, ()>,
}

impl SmithayAppRunnerState {
    pub fn try_new(event_loop: &mut EventLoop<Self>, app: App) -> Result<Self, Error> {
        let display = Display::<Self>::new()?;
        let display_handle = display.handle();
        let (mut session, session_notifier) = LibSeatSession::new()?;
        let seat_name = session.seat();
        let udev = UdevBackend::new(&seat_name).map_err(Error::Udev)?;
        let mut context = Libinput::new_with_udev(LibinputSessionInterface::from(session.clone()));

        context
            .udev_assign_seat(&seat_name)
            .map_err(Error::UdevAssignSeat)?;

        let backend = LibinputInputBackend::new(context.clone());
        let source = ListeningSocketSource::new_auto()?;

        event_loop
            .handle()
            .insert_source(session_notifier, |event, _metadata, state| {
                dbg!(event);
                // todo
            })?;

        event_loop
            .handle()
            .insert_source(udev, |event, _metadata, state| {
                state.on_udev_event(dbg!(event))
            })?;

        event_loop
            .handle()
            .insert_source(backend, |event, _metadata, state| {
                state.on_input_event(dbg!(event))
            })?;

        event_loop
            .handle()
            .insert_source(source, |client_stream, _metadata, state| {
                state
                    .display_handle
                    .insert_client(
                        dbg!(client_stream),
                        std::sync::Arc::new(ClientState::default()),
                    )
                    .expect("new client");
            })?;

        event_loop.handle().insert_source(
            Generic::new(
                display,
                Interest::READ,
                smithay::reexports::calloop::Mode::Level,
            ),
            |_, display, data| {
                println!("dispatch");
                unsafe {
                    display.get_mut().dispatch_clients(data).unwrap();
                }

                Ok(PostAction::Continue)
            },
        )?;

        let drm_node = util::find_best_gpu(&seat_name).unwrap();

        let drm_device_fd = dbg!(session
            .open(&dbg!(drm_node.dev_path().unwrap()), OFlags::RDWR))
            .map(DeviceFd::from)
            .map(DrmDeviceFd::new)
            .unwrap();

        let (mut drm_device, drm_device_notifier) = DrmDevice::new(drm_device_fd.clone(), true)?;

        event_loop
            .handle()
            .insert_source(drm_device_notifier, |event, _metadata, state| {
                state.on_drm_event(event)
            })?;

        let mut drm_scanner = DrmScanner::<SimpleCrtcMapper>::new();
        let _result = drm_scanner
            .scan_connectors(&drm_device)
            .map_err(Error::DrmScan)?;

        let (connector, mode) = drm_scanner
            .connectors()
            .iter()
            .find_map(|(connector, info)| {
                let mode = *info.modes().iter().next()?;

                Some((*connector, mode))
            })
            .unwrap();

        let crtc = drm_scanner.crtc_for_connector(&connector).unwrap();
        let gbm_device = GbmDevice::new(drm_device_fd).map_err(Error::Gbm)?;
        let gbm_allocator = GbmAllocator::new(gbm_device.clone(), GbmBufferFlags::SCANOUT);
        let plane = drm_device.planes(&crtc).unwrap().primary[0].handle;
        let drm_plane_claim = drm_device.claim_plane(plane, crtc).unwrap();
        let drm_surface = drm_device.create_surface(crtc, mode, &[connector]).unwrap();
        let gbm_surface = GbmBufferedSurface::new(
            drm_surface,
            gbm_allocator,
            &[Fourcc::Abgr8888, Fourcc::Xrgb8888],
            Some(allocator::Format {
                code: Fourcc::Abgr8888,
                modifier: Modifier::Linear,
            }),
        )
        .unwrap();

        let smithay_state = SmithayState::new(&display_handle, drm_node, &seat_name);

        Ok(Self {
            app,
            display_handle,
            smithay_state,
            drm_node,
            drm_device,
            drm_scanner,
            drm_plane_claim,
            gbm_device,
            gbm_surface,
        })
    }

    fn on_udev_event(&mut self, event: UdevEvent) {
        match event {
            UdevEvent::Added { device_id, path } => {
                dbg!(DrmNode::from_dev_id(device_id));
            }
            UdevEvent::Changed { device_id } => {
                dbg!(DrmNode::from_dev_id(device_id));
            }
            UdevEvent::Removed { device_id } => {
                dbg!(DrmNode::from_dev_id(device_id));
            }
        }
    }

    fn on_drm_event(&mut self, event: DrmEvent) {
        match event {
            DrmEvent::VBlank(handle) => {
                self.gbm_surface.frame_submitted().unwrap();
            }
            DrmEvent::Error(error) => {
                //
            }
        }
    }

    fn on_input_event(&mut self, event: InputEvent<LibinputInputBackend>) {
        if let InputEvent::Keyboard { event } = event {
            self.on_keyboard_event(event)
        }
    }

    fn on_keyboard_event(&mut self, event: KeyboardKeyEvent) {
        let keycode = event.key_code();
        let state = event.state();
        let serial = SERIAL_COUNTER.next_serial();
        let keyboard = self.smithay_state.seat.get_keyboard().unwrap();
        let time = event.time_msec();

        keyboard
            .input(
                self,
                keycode,
                state,
                serial,
                time,
                |state, modifiers, keysym| {
                    println!("{keysym:?}");

                    smithay::input::keyboard::FilterResult::Forward
                },
            )
            .unwrap_or(());
    }

    pub fn run(&mut self, event_loop: &mut EventLoop<Self>) -> AppExit {
        loop {
            let _result = event_loop.dispatch(Some(Duration::from_millis(1000 / 15)), self);

            let render_device = self.app.world_mut().resource::<RenderDevice>();
            let (dmabuf, slot) = self.gbm_surface.next_buffer().unwrap();

            let gbm_buffer = dmabuf
                .import_to(&self.gbm_device, GbmBufferFlags::empty())
                .unwrap();

            let handle = ManualTextureViewHandle(0);
            let (_texture, manual_texture_view) =
                util::import_texture(render_device, &gbm_buffer).unwrap();

            self.app
                .world_mut()
                .resource_mut::<ManualTextureViews>()
                .insert(handle, manual_texture_view);

            let target = RenderTarget::TextureView(handle);

            self.app.insert_resource(MainTexture(target));

            self.gbm_surface.queue_buffer(None, None, ()).unwrap();

            self.smithay_state.space.elements().for_each(|window| {
                window.send_frame(
                    &self.smithay_state.output,
                    self.smithay_state.start_time.elapsed(),
                    Some(Duration::ZERO),
                    |_, _| Some(self.smithay_state.output.clone()),
                )
            });

            self.smithay_state.space.refresh();
            self.smithay_state.popup_manager.cleanup();

            let _ = self.display_handle.flush_clients();

            if self.app.plugins_state() == PluginsState::Cleaned {
                self.app.update()
            }

            if let Some(app_exit) = self.app.should_exit() {
                return app_exit;
            }
        }
    }
}

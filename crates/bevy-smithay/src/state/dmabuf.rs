use {
    super::SmithayAppRunnerState,
    crate::{external_image::ExternalImages, util},
    bevy::{
        asset::Assets,
        render::{
            camera::{ManualTextureViewHandle, RenderTarget},
            render_asset::RenderAssetUsages,
            renderer::RenderDevice,
            texture::{GpuImage, Image, ImageSamplerDescriptor},
        },
    },
    smithay::{
        backend::allocator::dmabuf::Dmabuf,
        reexports::{gbm::BufferObjectFlags, wayland_server::protocol::wl_surface::WlSurface},
        wayland::dmabuf::{
            DmabufFeedback, DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier,
        },
    },
};

impl DmabufHandler for SmithayAppRunnerState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        println!("dmabuf state");

        &mut self.smithay_state.dmabuf_state
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        println!("dmabuf import");

        let render_device = self.app.world_mut().resource::<RenderDevice>();
        let sampler = render_device.create_sampler(&ImageSamplerDescriptor::default().as_wgpu());

        let gbm_buffer = dmabuf
            .import_to(&self.gbm_device, BufferObjectFlags::empty())
            .unwrap();

        let (texture, manual_texture_view) =
            util::import_texture(render_device, &gbm_buffer).unwrap();

        let mut images = self.app.world_mut().resource_mut::<Assets<Image>>();

        let handle = images.add({
            let size = wgpu::Extent3d {
                width: manual_texture_view.size.x,
                height: manual_texture_view.size.y,
                depth_or_array_layers: 1,
            };

            let mut image = Image::default();

            image.resize(size);
            image.asset_usage = RenderAssetUsages::RENDER_WORLD;
            image
        });

        let gpu_image = GpuImage {
            texture,
            texture_view: manual_texture_view.texture_view,
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            sampler,
            size: manual_texture_view.size,
            mip_level_count: 1,
        };

        let mut external_images = self.app.world_mut().resource_mut::<ExternalImages>();

        external_images.assets.insert(handle, gpu_image);

        notifier.successful::<Self>();
    }

    fn new_surface_feedback(
        &mut self,
        _surface: &WlSurface,
        _global: &DmabufGlobal,
    ) -> Option<DmabufFeedback> {
        None
    }
}

smithay::delegate_dmabuf!(SmithayAppRunnerState);

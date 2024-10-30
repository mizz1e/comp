use {
    ash::vk,
    bevy::{
        math::UVec2,
        render::{
            camera::ManualTextureView,
            render_resource::{Texture, TextureView},
            renderer::RenderDevice,
        },
    },
    core::slice,
    smithay::{
        backend::allocator::{gbm::GbmBuffer, Modifier},
        reexports::{
            drm,
            gbm::{DeviceDestroyedError, FdError},
        },
    },
    std::os::fd::{IntoRawFd, OwnedFd},
    wgpu::hal as wgpu_hal,
};

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("gbm: {0}")]
    GbmDestroyed(#[from] DeviceDestroyedError),

    #[error("gbm: {0}")]
    GbmFd(#[from] FdError),

    #[error("vulkan: {0}")]
    Vulkan(#[from] vk::Result),
}

pub fn import_texture(
    render_device: &RenderDevice,
    gbm_buffer: &GbmBuffer,
) -> Result<(Texture, ManualTextureView), ImportError> {
    let wgpu_device = render_device.wgpu_device();

    let dmabuf_fd = gbm_buffer.fd_for_plane(0)?;
    let drm_modifier = gbm_buffer.modifier()?;
    let offset = gbm_buffer.offset(0)?;
    let stride = gbm_buffer.stride_for_plane(0)?.into();
    let (width, height) = drm::buffer::Buffer::size(gbm_buffer);

    let (vk_image, _vk_device_memory) = unsafe {
        wgpu_device
            .as_hal::<wgpu_hal::api::Vulkan, _, _>(|hal_device| {
                let hal_device = hal_device?;
                let ash_device = hal_device.raw_device();
                let extent = vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                };

                Some(create_dmabuf_texture(
                    ash_device,
                    dmabuf_fd,
                    drm_modifier,
                    extent,
                    offset,
                    stride,
                ))
            })
            .flatten()
            .unwrap()?
    };

    let label = None;
    let mip_level_count = 1;
    let sample_count = 1;
    let dimension = wgpu::TextureDimension::D2;
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let desc = &wgpu::hal::TextureDescriptor {
        label,
        size,
        mip_level_count,
        sample_count,
        dimension,
        format,
        usage: wgpu_hal::TextureUses::COLOR_TARGET,
        memory_flags: wgpu_hal::MemoryFlags::PREFER_COHERENT,
        view_formats: vec![],
    };

    let hal_texture = unsafe { wgpu_hal::vulkan::Device::texture_from_raw(vk_image, desc, None) };

    let desc = &wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count,
        sample_count,
        dimension,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    let texture: Texture = unsafe {
        wgpu_device
            .create_texture_from_hal::<wgpu_hal::api::Vulkan>(hal_texture, desc)
            .into()
    };

    let texture_view: TextureView = texture
        .create_view(&wgpu::TextureViewDescriptor::default())
        .into();

    let size = UVec2::new(width, height);

    let manual_texture_view = ManualTextureView {
        texture_view: TextureView::from(texture_view),
        size,
        format,
    };

    Ok((texture, manual_texture_view))
}

fn create_dmabuf_texture(
    device: &ash::Device,
    dmabuf_fd: OwnedFd,
    drm_modifier: Modifier,
    extent: vk::Extent3D,
    offset: u32,
    stride: u64,
) -> Result<(vk::Image, vk::DeviceMemory), ImportError> {
    let mut external_image_info = vk::ExternalMemoryImageCreateInfo::builder()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

    let subresource_layout = vk::SubresourceLayout::builder()
        .offset(u64::from(offset))
        .row_pitch(stride);

    let mut modifier_info = vk::ImageDrmFormatModifierExplicitCreateInfoEXT::builder()
        .drm_format_modifier(u64::from(drm_modifier))
        .plane_layouts(slice::from_ref(&subresource_layout));

    let image_info = vk::ImageCreateInfo::builder()
        .array_layers(1)
        .extent(extent)
        .format(vk::Format::B8G8R8A8_SRGB)
        .image_type(vk::ImageType::TYPE_2D)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .mip_levels(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
        .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .push_next(&mut external_image_info)
        .push_next(&mut modifier_info);

    let image = unsafe { device.create_image(&image_info, None)? };
    let memory_requirements = unsafe { device.get_image_memory_requirements(image) };

    let mut import_memory_info = vk::ImportMemoryFdInfoKHR::builder()
        .fd(dmabuf_fd.into_raw_fd())
        .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

    let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::builder().image(image);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(memory_requirements.size)
        .memory_type_index(0)
        .push_next(&mut dedicated_info)
        .push_next(&mut import_memory_info);

    let device_memory = unsafe { device.allocate_memory(&memory_info, None)? };

    unsafe { device.bind_image_memory(image, device_memory, 0)? };

    Ok((image, device_memory))
}

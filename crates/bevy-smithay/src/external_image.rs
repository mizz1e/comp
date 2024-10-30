use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};

pub struct ExternalImagePlugin;

#[derive(Clone, Debug, Default, ExtractResource, Resource)]
pub struct ExternalImages {
    pub assets: HashMap<Handle<Image>, GpuImage>,
}

impl Plugin for ExternalImagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExternalImages>()
            .add_plugins(ExtractResourcePlugin::<ExternalImages>::default());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<ExternalImages>().add_systems(
                Render,
                prepare_external_images.in_set(RenderSet::PrepareAssets),
            );
        }
    }
}

pub fn prepare_external_images(
    external_images: Res<ExternalImages>,
    mut images: ResMut<RenderAssets<GpuImage>>,
) {
    for (image_id, gpu_image) in external_images.assets.iter() {
        images.insert(image_id, gpu_image.clone());
    }
}

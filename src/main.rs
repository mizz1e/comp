use {
    bevy::{
        color::palettes::css,
        diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
        prelude::*,
        render::{
            camera::RenderTarget,
            extract_resource::ExtractResource,
            render_asset::RenderAssets,
            render_resource::{
                Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            },
            texture::GpuImage,
            Extract, Render, RenderApp, RenderSet,
        },
        utils::HashMap,
    },
    bevy_smithay::{
        external_image::ExternalImages,
        state::{DiagnosticText, MainCamera, MainTexture},
        SmithayPlugin,
    },
};

#[derive(Clone, Debug, Resource)]
pub struct FontCollection {
    roboto_mono: Handle<Font>,
    noto_mono: Handle<Font>,
    noto_symbols: Handle<Font>,
}

impl FontCollection {
    fn text_style(font: &Handle<Font>, font_size: f32, color: impl Into<Color>) -> TextStyle {
        TextStyle {
            font: font.clone(),
            font_size,
            color: color.into(),
        }
    }

    pub fn roboto_mono(&self, font_size: f32, color: impl Into<Color>) -> TextStyle {
        Self::text_style(&self.roboto_mono, font_size, color)
    }

    pub fn noto_mono(&self, font_size: f32, color: impl Into<Color>) -> TextStyle {
        Self::text_style(&self.noto_mono, font_size, color)
    }

    pub fn noto_symbols(&self, font_size: f32, color: impl Into<Color>) -> TextStyle {
        Self::text_style(&self.noto_symbols, font_size, color)
    }
}

fn setup_fonts(asset_server: ResMut<AssetServer>, mut commands: Commands) {
    commands.insert_resource(FontCollection {
        roboto_mono: asset_server.load("fonts/RobotoMono-SemiBold.ttf"),
        noto_mono: asset_server.load("fonts/NotoSansMono-Bold.ttf"),
        noto_symbols: asset_server.load("fonts/NotoSansSymbols2-Regular.ttf"),
    });
}

fn update_text(
    diagnostic: Res<DiagnosticsStore>,
    mut diagnostic_text: Query<&mut Text, With<DiagnosticText>>,
    main_camera: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut diagnostic_text) = diagnostic_text.get_single_mut() else {
        return;
    };

    let main_camera = main_camera.single();

    if let Some(fps) = diagnostic.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            diagnostic_text.sections[1].value = format!("{value:.2}");
        }
    }

    let view_angle = main_camera.rotation.to_euler(EulerRot::YXZ);
    let (yaw, pitch, roll) =
        (Vec3::from(view_angle) * Vec3::splat(180.0 / std::f32::consts::PI)).into();

    let roll = if roll == 0.0 { 0.0 } else { roll };

    diagnostic_text.sections[5].value = format!("{yaw:.2}");
    diagnostic_text.sections[7].value = format!("{pitch:.2}");
    diagnostic_text.sections[9].value = format!("{roll:.2}");
}

fn setup(
    font_collection: Res<FontCollection>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    main_texture: Res<MainTexture>,
) {
    commands.spawn(DirectionalLightBundle::default());

    let main_camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    target: main_texture.0.clone(),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::Z * -180.0, Vec3::Y),
                ..default()
            },
            // InputManagerBundle::with_map(
            //     InputMap::default().with_dual_axis(MoveAction::Move, MouseMove::default()),
            // ),
            MainCamera,
        ))
        .id();

    commands
        .spawn((
            NodeBundle {
                background_color: Color::BLACK.with_alpha(0.4).into(),
                style: Style {
                    margin: UiRect::all(Val::Px(16.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            TargetCamera(main_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new("FPS ", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new("N/A", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new(" (", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new("N/Ams", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new(")\n", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new("0.00", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new(", ", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new("0.00", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new(", ", font_collection.roboto_mono(34.0, Color::BLACK)),
                        TextSection::new("0.00", font_collection.roboto_mono(34.0, Color::BLACK)),
                    ]),
                    ..default()
                },
                DiagnosticText,
            ));
        });
}

#[derive(Component)]
struct Dont;

fn setup_window(
    font_collection: Res<FontCollection>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut external_images: ResMut<ExternalImages>,
    query: Query<&Dont>,
) {
    if query.get_single().is_ok() {
        return;
    }

    let Some(texture_id) = external_images.assets.keys().next() else {
        return;
    };

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    let texture_camera = commands
        .spawn(Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(css::GREEN.into()),
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        })
        .id();

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        unlit: true,
        ..default()
    });

    let mut transform = Transform::from_xyz(0.0, 0.0, -180.0);

    transform.rotate_x(90.0_f32.to_radians());

    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(256.0, 144.0)),
        material: material_handle,
        transform,
        ..default()
    });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            TargetCamera(texture_camera),
            Dont,
        ))
        .with_children(|builder| {
            builder.spawn(ImageBundle {
                image: texture_id.clone().into(),
                ..default()
            });
        });
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin, SmithayPlugin))
        .add_systems(Startup, setup_fonts)
        .add_systems(
            Update,
            (setup.run_if(resource_added::<MainTexture>), update_text),
        )
        .run();
}

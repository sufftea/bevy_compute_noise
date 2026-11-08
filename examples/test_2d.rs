use bevy::{
    asset::{
        saver::{save_using_saver, SavedAsset},
        RenderAssetUsages,
    },
    core_pipeline::tonemapping::Tonemapping,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::{AsBindGroup, ShaderRef},
    },
    sprite::{Material2d, Material2dPlugin},
    tasks::IoTaskPool,
    window::PresentMode,
};
use bevy_compute_noise::{noise::generators::fbm::FbmFlags, prelude::*};
use iyes_perf_ui::{prelude::PerfUiDefaultEntries, PerfUiPlugin};
use ops::exp2;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            Material2dPlugin::<ImageMaterial>::default(),
            ComputeNoisePlugin,
            PerfUiPlugin,
        ))
        .add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            perform_save.run_if(input_just_pressed(KeyCode::F5)),
        )
        // .add_systems(Update, update_noise)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ImageMaterial>>,
    mut noise_queue: ResMut<ComputeNoiseQueue>,
) {
    let mut image = ComputeNoiseImage::create_image(ComputeNoiseSize::D2(512, 512));
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });
    let handle = images.add(image);

    let mut quad = Rectangle::default().mesh().build();
    if let Some(uvs) = quad.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        if let VertexAttributeValues::Float32x2(uvs) = uvs {
            for uv in uvs.iter_mut() {
                *uv = [uv[0] * 2.0, uv[1] * 2.0];
            }
        }
    }

    // noise_queue.queue(handle.clone(), (
    //     Fbm::<Perlin> {
    //         noise: Perlin {
    //             flags: PerlinFlags::TILEABLE.bits(),
    //             ..default()
    //         },
    //         persistence: exp2(-0.85),
    //         flags: FbmFlags::BILLOWY.bits(),
    //         ..default()
    //     },
    //     //Write(handle.clone(), Channel::R)
    // ));

    noise_queue.queue(
        handle.clone(),
        (
            Worley {
                seed: 0,
                frequency: 5.0,
                flags: WorleyFlags::TILEABLE.bits(),
            },
            Invert,
            // Store(Channel::R)
        ),
    );

    commands.spawn((
        Mesh2d(meshes.add(quad)),
        Transform::default().with_scale(Vec3::splat(512.)),
        MeshMaterial2d(materials.add(ImageMaterial {
            image: handle.clone(),
        })),
    ));

    commands.spawn((Camera2d, Tonemapping::None));
    commands.spawn(PerfUiDefaultEntries::default());
}

// fn update_noise(
//     mut noise_queue: ResMut<ComputeNoiseQueue>,
//     query: Query<&MeshMaterial2d<ImageMaterial>>,
//     keys: Res<ButtonInput<KeyCode>>,
//     mut local: Local<u32>,
//     materials: Res<Assets<ImageMaterial>>,
// ) {
//     if keys.just_pressed(KeyCode::Space) {
//         for material in query.iter() {
//             noise_queue.queue(
//                 materials.get(&material.0).unwrap().image.clone(),
//                 Worley {
//                     seed: *local,
//                     frequency: 5.0,
//                     flags: WorleyFlags::TILEABLE.bits(),
//                 }
//             );
//         }
//         *local = *local + 1;
//     }
// }

#[derive(Asset, AsBindGroup, Debug, Clone, Reflect)]
struct ImageMaterial {
    #[texture(101)]
    #[sampler(102)]
    image: Handle<Image>,
}

impl Material2d for ImageMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/image_shader.wesl".into()
    }
}

#[derive(Resource)]
struct ImageToSave(Handle<Image>);

fn perform_save(
    image_to_save: Res<ImageToSave>,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let image = images.get(&image_to_save.0).unwrap();

    let image = image.clone();
    let asset_server = asset_server.clone();
    IoTaskPool::get()
        .spawn(async move {
            match save_using_saver(
                asset_server.clone(),
                &ImageSaver,
                &ASSET_PATH.into(),
                SavedAsset::from_asset(&image),
                &ImageSaverSettings::default(),
            )
            .await
            {
                Ok(()) => info!("Completed save of {ASSET_PATH}"),
                Err(err) => error!("Failed to save asset: {err}"),
            }
        })
        .detach();
}

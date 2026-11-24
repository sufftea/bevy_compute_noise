use bevy::{
    asset::{saver::save_using_saver, AssetPath, AsyncWriteExt},
    core_pipeline::tonemapping::Tonemapping,
    image::{
        ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageSaver, ImageSaverSettings,
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        gpu_readback::{ReadbackComplete, ReadbackOnce},
        mesh::VertexAttributeValues,
        render_resource::AsBindGroup,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
    tasks::IoTaskPool,
    window::PresentMode,
};
use bevy_compute_noise::prelude::*;

const ASSET_PATH: &str = "saved_texture.png";

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
        ))
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, run_noise.run_if(input_just_pressed(KeyCode::F4)))
        .add_systems(
            PreUpdate,
            perform_save.run_if(input_just_pressed(KeyCode::F5)),
        )
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

    // noise_queue.queue(
    //     handle.clone(),
    //     (
    //         Worley {
    //             seed: 0,
    //             frequency: 5.0,
    //             flags: WorleyFlags::TILEABLE.bits(),
    //         },
    //         Invert,
    //         // Store(Channel::R)
    //     ),
    // );
    commands.insert_resource(ResultTexture(handle.clone()));

    commands.spawn((
        Mesh2d(meshes.add(quad)),
        Transform::default().with_scale(Vec3::splat(512.)),
        MeshMaterial2d(materials.add(ImageMaterial {
            image: handle.clone(),
        })),
    ));

    commands.spawn((Camera2d, Tonemapping::None));
}

fn run_noise(result_texture: Res<ResultTexture>, mut noise_queue: ResMut<ComputeNoiseQueue>) {
    noise_queue.queue(
        result_texture.0.clone(),
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
}

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
struct ResultTexture(Handle<Image>);

fn perform_save(mut commands: Commands, image_to_save: Res<ResultTexture>) {
    info!("Reading the image from the GPU");

    commands
        .spawn(ReadbackOnce::texture(image_to_save.0.clone()))
        .observe(
            |mut event: On<ReadbackComplete>, asset_server: Res<AssetServer>| {
                info!("ReadbackOnce complete");

                let _asset_server = asset_server.clone();
                let data = std::mem::take(&mut event.data);
                IoTaskPool::get()
                    .spawn(async move {
                        if let Err(err) = save_image(_asset_server, &data).await {
                            error!("{}", err.to_string());
                        };
                        info!("Image saved");
                    })
                    .detach();

                // IoTaskPool::get()
                //     .spawn(async move {
                //         match save_using_saver(
                //             asset_server.clone(),
                //             &ImageSaver,
                //             &ASSET_PATH.into(),
                //             SavedAsset::from_asset(&image),
                //             &ImageSaverSettings::default(),
                //         )
                //         .await
                //         {
                //             Ok(()) => info!("Completed save of {ASSET_PATH}"),
                //             Err(err) => error!("Failed to save asset: {err}"),
                //         }
                //     })
                //     .detach();
            },
        );
}

async fn save_image(asset_server: AssetServer, bytes: &[u8]) -> Result<(), std::io::Error> {
    // std::io::Error::new(kind, error)

    let path = AssetPath::from(ASSET_PATH);
    let source = asset_server
        .get_source(path.source())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "TODO"))?;
    let writer = source
        .writer()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Unsupported, "TODO"))?;

    let mut file_writer = writer.write(path.path()).await.map_err(|err| match err {
        bevy::asset::io::AssetWriterError::Io(error) => error,
    })?;

    file_writer.write_all(bytes).await?;

    Ok(())
}

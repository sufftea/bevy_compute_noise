use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{BindGroupEntry, BindGroupLayoutEntry, BindingType, BufferBindingType, IntoBinding, PipelineCache, ShaderStages, SpecializedComputePipelines, StorageTextureAccess, TextureFormat, TextureViewDimension},
        renderer::RenderDevice,
        texture::GpuImage,
    },
};

use crate::{
    noise_queue::{ComputeNoiseRenderQueue, ComputeNoiseResource, ComputeNoiseResourceQueue, RenderComputeNoise}, render::pipeline::ComputeNoisePipeline
};

pub fn prepare_render_noise(
    pipeline: Res<ComputeNoisePipeline>,
    mut pipelines: ResMut<SpecializedComputePipelines<ComputeNoisePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    queue: Res<ComputeNoiseResourceQueue>,
    mut render_queue: ResMut<ComputeNoiseRenderQueue>,
) {
    for noise in queue.queue.iter() {
        let resources: Option<Vec<_>> = noise.resources.iter()
            .map(|resource| match resource {
                ComputeNoiseResource::Image(handle) => gpu_images.get(handle)
                    .map(|image| (true, image.texture_view.into_binding())),
                ComputeNoiseResource::Buffer(buffer) => Some((false, buffer.as_entire_binding())),
            })
            .collect();
        
        if let Some(resources) = resources {
            let layout = pipeline.get_layout(noise.key);
            let bind_group = render_device.create_bind_group(
                Some("compute_noise_bind_group"),
                &layout,
                &resources.iter().enumerate()
                    .map(|(i, (_, resource))| BindGroupEntry {
                        binding: i as u32,
                        resource: resource.clone(),
                    })
                    .collect::<Vec<_>>(),
            );

            let pipeline_id = pipelines.specialize(
                &pipeline_cache, 
                &pipeline, 
                noise.key,
            );

            render_queue.queue.push(RenderComputeNoise {
                key: noise.key,
                bind_group,
                pipeline_id,
                size: noise.size,
            });
        }
    }
}

pub fn create_bind_group_layout(
    mut pipeline: ResMut<ComputeNoisePipeline>,
    render_device: Res<RenderDevice>,
    queue: Res<ComputeNoiseResourceQueue>,
) {
    for sequence in &queue.queue {
        if pipeline.layouts.contains_key(&sequence.key) { continue; }

        let mut entries = Vec::new();
        
        entries.push(BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba8Unorm,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        });

        for binding in 1..sequence.resources.len() {
            match sequence.resources[binding] {
                ComputeNoiseResource::Image(_) => {
                    entries.push(BindGroupLayoutEntry {
                        binding: binding as u32,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    });
                },
                ComputeNoiseResource::Buffer(_) => {
                    entries.push(BindGroupLayoutEntry {
                        binding: binding as u32,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    });
                },
            }
        }

        dbg!(&entries);

        let layout = render_device.create_bind_group_layout(
            Some("compute_noise_bind_group_layout"),
            &entries,
        );

        pipeline.layouts.insert(sequence.key, layout);
        pipeline.shaders.insert(sequence.key, sequence.shader.clone());
    }
}
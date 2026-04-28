use bevy::{
    core_pipeline::{Core2d, Core2dSystems, FullscreenShader},
    image::ToExtents,
    material::descriptor::{
        BindGroupLayoutDescriptor, CachedRenderPipelineId, FragmentState, RenderPipelineDescriptor,
    },
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup,
        camera::ExtractedCamera,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{
            BindGroupEntries, BindGroupLayoutEntries, ColorTargetState, ColorWrites, IntoBinding,
            Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureDescriptor,
            TextureDimension, TextureSampleType, TextureUsages, TextureViewDescriptor,
            UniformBuffer,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        texture::TextureCache,
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        view::ViewTarget,
    },
};

use crate::{DistanceFieldTextures, TEXTURE_FORMAT, distance_field, uv_to_color};

const THRESHOLD_SHADER: &str = "shaders/threshold.wgsl";

pub struct ThresholdPlugin;

impl Plugin for ThresholdPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<ThresholdSettings>::default(),
            UniformComponentPlugin::<ThresholdSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_threshold_pipeline)
            .add_systems(Render, prepare_textures)
            .add_systems(Core2d, threshold_system.in_set(Core2dSystems::PostProcess));
    }
}

// #[derive(Default)]
// pub struct ThresholdBindGroupCache {
//     cached: Option<(TextureViewId, BindGroup)>,
// }

#[derive(Component, Default, Reflect, Clone, Copy, ExtractComponent, ShaderType)]
#[reflect(Component)]
pub struct ThresholdSettings {
    pub threshold: f32,
}

#[derive(Resource)]
pub struct ThresholdPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub fn threshold_system(
    view: ViewQuery<(
        &ViewTarget,
        &ThresholdSettings,
        &DistanceFieldTextures,
        &DynamicUniformIndex<ThresholdSettings>,
    )>,
    threshold_pipeline: Option<Res<ThresholdPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<ThresholdSettings>>,
    // mut cache: Local<ThresholdBindGroupCache>,
    mut ctx: RenderContext,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(threshold_pipeline) = threshold_pipeline else {
        return;
    };

    let (view_target, _threshold_settings, distance_field_textures, settings_index) =
        view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(threshold_pipeline.pipeline_id) else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let mut count = 0;
    for texture in [
        &distance_field_textures.texture_regular_a,
        &distance_field_textures.texture_invert_a,
    ] {
        let mut uniform = UniformBuffer::from(count);

        uniform.set(count);
        uniform.write_buffer(&render_device, &render_queue);

        let bind_group = ctx.render_device().create_bind_group(
            "threshold_bind_group",
            &pipeline_cache.get_bind_group_layout(&threshold_pipeline.layout),
            &BindGroupEntries::sequential((
                post_process.source,
                &threshold_pipeline.sampler,
                settings_binding.clone(),
                uniform.into_binding(),
            )),
        );

        let mut render_pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("threshold_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture.texture.create_view(&TextureViewDescriptor {
                        format: Some(TEXTURE_FORMAT),
                        base_mip_level: 0u32,
                        mip_level_count: Some(1u32),
                        ..default()
                    }),
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        render_pass.set_pipeline(pipeline);

        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        count += 1;
    }
}

fn prepare_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera)>,
) {
    for (entity, camera) in views {
        if let Some(viewport) = camera.physical_viewport_size {
            let texture_descriptor_regular_a = TextureDescriptor {
                label: Some("texture_regular_a"),
                size: viewport
                    .as_vec2()
                    .round()
                    .as_uvec2()
                    .max(UVec2::ONE)
                    .to_extents(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TEXTURE_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };
            let texture_descriptor_invert_a = TextureDescriptor {
                label: Some("texture_invert_a"),
                size: viewport
                    .as_vec2()
                    .round()
                    .as_uvec2()
                    .max(UVec2::ONE)
                    .to_extents(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TEXTURE_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };
            let texture_descriptor_regular_b = TextureDescriptor {
                label: Some("texture_regular_b"),
                size: viewport
                    .as_vec2()
                    .round()
                    .as_uvec2()
                    .max(UVec2::ONE)
                    .to_extents(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TEXTURE_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };
            let texture_descriptor_invert_b = TextureDescriptor {
                label: Some("texture_invert_b"),
                size: viewport
                    .as_vec2()
                    .round()
                    .as_uvec2()
                    .max(UVec2::ONE)
                    .to_extents(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TEXTURE_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };

            let texture_regular_a = texture_cache.get(&render_device, texture_descriptor_regular_a);
            let texture_invert_a = texture_cache.get(&render_device, texture_descriptor_invert_a);
            let texture_regular_b = texture_cache.get(&render_device, texture_descriptor_regular_b);
            let texture_invert_b = texture_cache.get(&render_device, texture_descriptor_invert_b);

            commands.entity(entity).insert(DistanceFieldTextures {
                texture_regular_a: texture_regular_a,
                texture_invert_a: texture_invert_a,
                texture_regular_b: texture_regular_b,
                texture_invert_b: texture_invert_b,
            });
        }
    }
}

fn init_threshold_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "post_process_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                // screen texture
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::NonFiltering),
                uniform_buffer::<ThresholdSettings>(true),
                uniform_buffer::<u32>(false),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let shader = asset_server.load(THRESHOLD_SHADER);

    let vertex_state = fullscreen_shader.to_vertex_state();
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("threshold_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TEXTURE_FORMAT,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(ThresholdPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

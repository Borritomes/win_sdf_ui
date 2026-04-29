use bevy::{
    core_pipeline::{Core2d, Core2dSystems, FullscreenShader},
    material::descriptor::{
        BindGroupLayoutDescriptor, CachedRenderPipelineId, FragmentState, RenderPipelineDescriptor,
    },
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutEntries, ColorTargetState, ColorWrites,
            Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureFormat,
            TextureSampleType, TextureViewId,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        view::ViewTarget,
    },
};

use crate::{distance_field, ping_pong::SdfTextures};

const DISTANCE_TO_VALUE_SHADER: &str = "shaders/distance_to_value.wgsl";

pub struct DistanceToValuePlugin;

impl Plugin for DistanceToValuePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<DistanceToValueSettings>::default(),
            UniformComponentPlugin::<DistanceToValueSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_distance_to_value_pipeline);
        render_app.add_systems(
            Core2d,
            distance_to_value_system
                .after(distance_field::distance_field_system)
                .in_set(Core2dSystems::PostProcess),
        );
    }
}

#[derive(Default)]
pub struct DistanceToValueBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

#[derive(Component, Default, Reflect, Clone, Copy, ExtractComponent, ShaderType)]
#[reflect(Component)]
pub struct DistanceToValueSettings {
    pub threshold: f32,
    pub radius: f32,
}

#[derive(Resource)]
pub struct DistanceToValuePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub fn distance_to_value_system(
    view: ViewQuery<(
        &ViewTarget,
        &SdfTextures,
        &DistanceToValueSettings,
        &DynamicUniformIndex<DistanceToValueSettings>,
    )>,
    distance_to_value_pipeline: Option<Res<DistanceToValuePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<DistanceToValueSettings>>,
    mut cache: Local<DistanceToValueBindGroupCache>,
    mut ctx: RenderContext,
) {
    let Some(distance_to_value_pipeline) = distance_to_value_pipeline else {
        return;
    };

    let (view_target, sdf_textures, _distance_to_value_settings, settings_index) = view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(distance_to_value_pipeline.pipeline_id)
    else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    let regular_view = sdf_textures.regular.read_current();
    let invert_view = sdf_textures.invert.read_current();

    let post_process = view_target.post_process_write();

    let bind_group = match &mut cache.cached {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
        cached => {
            let bind_group = ctx.render_device().create_bind_group(
                "distance_to_value_bind_group",
                &pipeline_cache.get_bind_group_layout(&distance_to_value_pipeline.layout),
                &BindGroupEntries::sequential((
                    regular_view,
                    invert_view,
                    &distance_to_value_pipeline.sampler,
                    settings_binding.clone(),
                )),
            );

            let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
            bind_group
        }
    };

    let mut render_pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("distance_to_value_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
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

    render_pass.set_bind_group(0, bind_group, &[settings_index.index()]);
    render_pass.draw(0..3, 0..1);
}

fn init_distance_to_value_pipeline(
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
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::NonFiltering),
                uniform_buffer::<DistanceToValueSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let shader = asset_server.load(DISTANCE_TO_VALUE_SHADER);

    let vertex_state = fullscreen_shader.to_vertex_state();
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("distance_to_value_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba32Float,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(DistanceToValuePipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

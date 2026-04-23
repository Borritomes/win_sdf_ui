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

const THRESHOLD_SHADER: &str = "shaders/threshold.wgsl";

pub struct ThresholdPostProcessPlugin;

impl Plugin for ThresholdPostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<ThresholdSettings>::default(),
            UniformComponentPlugin::<ThresholdSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_threshold_pipeline);
        render_app.add_systems(Core2d, threshold_system.in_set(Core2dSystems::PostProcess));
    }
}

#[derive(Default)]
pub struct ThresholdBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

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
        &DynamicUniformIndex<ThresholdSettings>,
    )>,
    threshold_pipeline: Option<Res<ThresholdPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<ThresholdSettings>>,
    mut cache: Local<ThresholdBindGroupCache>,
    mut ctx: RenderContext,
) {
    let Some(threshold_pipeline) = threshold_pipeline else {
        return;
    };

    let (view_target, _threshold_settings, settings_index) = view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(threshold_pipeline.pipeline_id) else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let bind_group = match &mut cache.cached {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
        cached => {
            let bind_group = ctx.render_device().create_bind_group(
                "threshold_bind_group",
                &pipeline_cache.get_bind_group_layout(&threshold_pipeline.layout),
                &BindGroupEntries::sequential((
                    post_process.source,
                    &threshold_pipeline.sampler,
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
            label: Some("threshold_pass"),
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
                format: TextureFormat::Rgba8UnormSrgb,
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

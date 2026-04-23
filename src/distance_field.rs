// not well named
// this it the jump flood shader
use std::cmp::max;
use bevy::{
    core_pipeline::{Core2d, Core2dSystems, FullscreenShader}, material::descriptor::{
        BindGroupLayoutDescriptor, CachedRenderPipelineId, FragmentState, RenderPipelineDescriptor,
    }, math::ops::{ceil, log2}, prelude::*, render::{
        RenderApp, RenderStartup,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutEntries, ColorTargetState, ColorWrites,
            IntoBinding, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType, TextureViewId, UniformBuffer,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        view::ViewTarget,
    }
};

const DISTANCE_FIELD_SHADER: &str = "shaders/distance_field.wgsl";
const DISTANCE_TO_VALUE_SHADER: &str = "shaders/distance_to_value.wgsl";

pub struct DistanceFieldPlugin;

impl Plugin for DistanceFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<DistanceFieldSettings>::default(),
            UniformComponentPlugin::<DistanceFieldSettings>::default(),
            ExtractResourcePlugin::<StepSizeBuffer>::default(),
        ));
        app.insert_resource(StepSizeBuffer { step_size: 8 });
        app.add_systems(Update, change_iteration_count);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, (init_distance_field_pipeline, init_distance_to_value_pipeline));
        render_app.add_systems(
            Core2d,
            (distance_field_system, distance_to_value_system)
                .chain()
                .after(crate::color_to_uv::color_to_uv_system)
                .in_set(Core2dSystems::PostProcess),
        );
    }
}

#[derive(Resource, Clone, ExtractResource, Default, ShaderType)]
pub struct StepSizeBuffer {
    step_size: u32,
}

#[derive(Default)]
pub struct DistanceFieldBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

#[derive(Default)]
pub struct DistanceToValueBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

#[derive(Component, Default, Reflect, Clone, Copy, ExtractComponent, ShaderType)]
#[reflect(Component)]
pub struct DistanceFieldSettings {
    pub radius: u32,
}

#[derive(Resource)]
pub struct DistanceFieldPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

#[derive(Resource)]
pub struct DistanceToValuePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn change_iteration_count(input: Res<ButtonInput<KeyCode>>, query: Query<&mut DistanceFieldSettings>) {
    if input.just_pressed(KeyCode::ArrowUp) {
        for mut distance_field_settings in query {
            distance_field_settings.radius *= 2;
            println!("doubled to {}", distance_field_settings.radius)
        }
    } else if input.just_pressed(KeyCode::ArrowDown) {
        for mut distance_field_settings in query {
            distance_field_settings.radius /= 2;
            if distance_field_settings.radius < 1 {
                distance_field_settings.radius = 1;
            }
            println!("halved to {}", distance_field_settings.radius)
        }
    }
}

pub fn distance_field_system(
    view: ViewQuery<(
        &ViewTarget,
        &DistanceFieldSettings,
        &DynamicUniformIndex<DistanceFieldSettings>,
    )>,
    distance_field_pipeline: Option<Res<DistanceFieldPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<DistanceFieldSettings>>,
    mut cache: Local<DistanceFieldBindGroupCache>,
    mut ctx: RenderContext,
    step_size_buffer: Res<StepSizeBuffer>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(distance_field_pipeline) = distance_field_pipeline else {
        return;
    };

    let (view_target, _distance_field_settings, settings_index) = view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(distance_field_pipeline.pipeline_id)
    else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    let size = view_target.main_texture().size();
    // dbg!(size);
    let mut jump_dist = max(size.width * 2, size.height * 2);
    let jump_steps = ceil(log2((jump_dist) as f32)) as u32;

    for _count in 0..=jump_steps {
        let post_process = view_target.post_process_write();
        
        let mut uniform = UniformBuffer::from(&*step_size_buffer);

        let step_size_buffer = &StepSizeBuffer {
            step_size: jump_dist,
        };

        uniform.set(step_size_buffer);
        uniform.write_buffer(&render_device, &render_queue);

        let bind_group = match &mut cache.cached {
            Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
            cached => {
                let bind_group = ctx.render_device().create_bind_group(
                    "distance_field_bind_group",
                    &pipeline_cache.get_bind_group_layout(&distance_field_pipeline.layout),
                    &BindGroupEntries::sequential((
                        post_process.source,
                        &distance_field_pipeline.sampler,
                        settings_binding.clone(),
                        uniform.into_binding(),
                    )),
                );

                let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
                bind_group
            }
        };

        let mut render_pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("distance_field_pass"),
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

        jump_dist /= 2;
        if jump_dist < 1 {
            jump_dist = 1
        }
    }
}

pub fn distance_to_value_system(
    view: ViewQuery<(
        &ViewTarget,
        &DistanceFieldSettings,
        &DynamicUniformIndex<DistanceFieldSettings>,
    )>,
    distance_to_value_pipeline: Option<Res<DistanceToValuePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<DistanceFieldSettings>>,
    mut cache: Local<DistanceToValueBindGroupCache>,
    mut ctx: RenderContext,
) {
    let Some(distance_to_value_pipeline) = distance_to_value_pipeline else {
        return;
    };

    let (view_target, _distance_field_settings, settings_index) = view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(distance_to_value_pipeline.pipeline_id)
    else {
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
                "distance_field_bind_group",
                &pipeline_cache.get_bind_group_layout(&distance_to_value_pipeline.layout),
                &BindGroupEntries::sequential((
                    post_process.source,
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
            label: Some("distance_field_pass"),
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

fn init_distance_field_pipeline(
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
                uniform_buffer::<DistanceFieldSettings>(true),
                uniform_buffer::<StepSizeBuffer>(false),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let shader = asset_server.load(DISTANCE_FIELD_SHADER);

    let vertex_state = fullscreen_shader.to_vertex_state();
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("distance_field_pipeline".into()),
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

    commands.insert_resource(DistanceFieldPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

fn init_distance_to_value_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "distance_to_value_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                // screen texture
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::NonFiltering),
                uniform_buffer::<DistanceFieldSettings>(true),
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
                format: TextureFormat::Rgba8UnormSrgb,
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

use bevy::{
    core_pipeline::{Core2d, Core2dSystems, FullscreenShader},
    material::descriptor::{
        BindGroupLayoutDescriptor, CachedRenderPipelineId, FragmentState, RenderPipelineDescriptor,
    },
    math::ops::{ceil, log2},
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        camera::ExtractedCamera,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutEntries, ColorTargetState, ColorWrites,
            IntoBinding, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureSampleType, TextureViewDescriptor, TextureViewId, UniformBuffer,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        view::ViewTarget,
    },
};
use std::cmp::max;

use crate::{DistanceFieldTextures, TEXTURE_FORMAT, uv_to_color};

const DISTANCE_FIELD_SHADER: &str = "shaders/distance_field.wgsl";

pub struct DistanceFieldPlugin;

impl Plugin for DistanceFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<DistanceFieldSettings>::default(),
            UniformComponentPlugin::<DistanceFieldSettings>::default(),
            ExtractResourcePlugin::<DistanceFieldPipeline>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.insert_resource(StepSizeBuffer { step_size: 8 });

        render_app.add_systems(RenderStartup, init_distance_field_pipeline);
        render_app.add_systems(
            Core2d,
            distance_field_system
                .after(uv_to_color::uv_to_color_system)
                .in_set(Core2dSystems::PostProcess),
        );
    }
}

#[derive(Resource, Clone, ExtractResource, Default, ShaderType)]
pub struct StepSizeBuffer {
    step_size: u32,
}

// #[derive(Default)]
// pub struct DistanceFieldBindGroupCache {
//     cached: Option<(TextureViewId, BindGroup)>,
// }

#[derive(Component, Reflect, Debug, ExtractComponent, Clone, ShaderType)]
#[reflect(Component)]
pub struct DistanceFieldSettings {
    pub radius: f32,
    pub threshold: f32,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct DistanceFieldImage(pub Handle<Image>);

#[derive(Resource, ExtractResource, Clone)]
pub struct DistanceFieldPipeline {
    bind_group_layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub fn distance_field_system(
    view: ViewQuery<(
        &ExtractedCamera,
        &ViewTarget,
        &DistanceFieldTextures,
        &DistanceFieldSettings,
        &DynamicUniformIndex<DistanceFieldSettings>,
    )>,
    distance_field_pipeline: Res<DistanceFieldPipeline>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<DistanceFieldSettings>>,
    step_size_buffer: Res<StepSizeBuffer>,
    // mut cache: Local<DistanceFieldBindGroupCache>,
    mut ctx: RenderContext,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let (_camera, view_target, distance_field_textures, _distance_field_settings, settings_index) =
        view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(distance_field_pipeline.pipeline_id)
    else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        warn!("failed settings_binding");
        return;
    };

    let size = view_target.main_texture().size();
    
    // read b write a
    for (texture_a, texture_b) in [
        (
            &distance_field_textures.texture_regular_a,
            &distance_field_textures.texture_regular_b,
        ),
        (
            &distance_field_textures.texture_invert_a,
            &distance_field_textures.texture_invert_b,
        ),
    ] {
        let mut jump_dist = max(size.width * 2, size.height * 2);
        let jump_steps = ceil(log2((jump_dist) as f32)) as u32;

        let view_b = texture_b.texture.create_view(&TextureViewDescriptor {
            label: Some("uv_to_color_view_b"),
            format: Some(TEXTURE_FORMAT),
            base_mip_level: 0u32,
            mip_level_count: Some(1u32),
            ..default()
        });

        let view_a = texture_a.texture.create_view(&TextureViewDescriptor {
            label: Some("uv_to_color_view_a"),
            format: Some(TEXTURE_FORMAT),
            base_mip_level: 0u32,
            mip_level_count: Some(1u32),
            ..default()
        });

        for _count in 0..=jump_steps {
            let mut uniform = UniformBuffer::from(&*step_size_buffer);

            let step_size_buffer = &StepSizeBuffer {
                step_size: jump_dist,
            };

            uniform.set(step_size_buffer);
            uniform.write_buffer(&render_device, &render_queue);

            // let bind_group = match &mut cache.cached {
            //     Some((texture_id, bind_group)) if view_b.id() == *texture_id => bind_group,
            //     cached => {
            //         let bind_group = ctx.render_device().create_bind_group(
            //             "distance_field_bind_group",
            //             &pipeline_cache
            //                 .get_bind_group_layout(&distance_field_pipeline.bind_group_layout),
            //             &BindGroupEntries::sequential((
            //                 &view_b,
            //                 &distance_field_pipeline.sampler,
            //                 settings_binding.clone(),
            //                 uniform.into_binding(),
            //             )),
            //         );

            //         let (_, bind_group) = cached.insert((view_b.id(), bind_group));
            //         bind_group
            //     }
            // };

            let bind_group = &ctx.render_device().create_bind_group(
                "distance_field_bind_group",
                &pipeline_cache.get_bind_group_layout(&distance_field_pipeline.bind_group_layout),
                &BindGroupEntries::sequential((
                    &view_b,
                    &distance_field_pipeline.sampler,
                    settings_binding.clone(),
                    uniform.into_binding(),
                )),
            );

            let mut render_pass = ctx
                .command_encoder()
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("distance_field_pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view_a,
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
}

fn init_distance_field_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "distance_field_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
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
                format: TEXTURE_FORMAT,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(DistanceFieldPipeline {
        bind_group_layout: layout,
        sampler: sampler,
        pipeline_id: pipeline_id,
    });
}

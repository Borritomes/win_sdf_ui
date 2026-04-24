// TODO: prepare bind groups whatever that means
// {
// let size = view_target.main_texture().size();
// // dbg!(size);
// let mut jump_dist = max(size.width * 2, size.height * 2);
// let jump_steps = ceil(log2((jump_dist) as f32)) as u32;

// for _count in 0..=jump_steps {
//     let post_process = view_target.post_process_write();

//     let mut uniform = UniformBuffer::from(&*step_size_buffer);

//     let step_size_buffer = &StepSizeBuffer {
//         step_size: jump_dist,
//     };

//     uniform.set(step_size_buffer);
//     uniform.write_buffer(&render_device, &render_queue);

//     let bind_group = match &mut cache.cached {
//         Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
//         cached => {
//             let bind_group = ctx.render_device().create_bind_group(
//                 "distance_field_bind_group",
//                 &pipeline_cache.get_bind_group_layout(&distance_field_pipeline.layout),
//                 &BindGroupEntries::sequential((
//                     post_process.source,
//                     &distance_field_pipeline.sampler,
//                     settings_binding.clone(),
//                     uniform.into_binding(),
//                 )),
//             );

//             let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
//             bind_group
//         }
//     };

//     let mut render_pass = ctx
//         .command_encoder()
//         .begin_render_pass(&RenderPassDescriptor {
//             label: Some("distance_field_pass"),
//             color_attachments: &[Some(RenderPassColorAttachment {
//                 view: post_process.destination,
//                 depth_slice: None,
//                 resolve_target: None,
//                 ops: Operations::default(),
//             })],
//             depth_stencil_attachment: None,
//             timestamp_writes: None,
//             occlusion_query_set: None,
//             multiview_mask: None,
//         });

//     render_pass.set_pipeline(pipeline);

//     render_pass.set_bind_group(0, bind_group, &[settings_index.index()]);
//     render_pass.draw(0..3, 0..1);

//     jump_dist /= 2;
//     if jump_dist < 1 {
//         jump_dist = 1
//     }
// }
// }

use bevy::{image::ToExtents, prelude::*, render::{Render, RenderApp, camera::ExtractedCamera, extract_component::{ExtractComponent, ExtractComponentPlugin}, render_resource::{ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages}, renderer::RenderDevice, texture::{CachedTexture, TextureCache}}};

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba32Float;

pub struct DistanceFieldPlugin;

impl Plugin for DistanceFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ExtractComponentPlugin::<DistanceField>::default()
        );

        let render_app = app.get_sub_app_mut(RenderApp).expect("failed to get render_app");

        render_app.add_systems(Render, prepare_distance_field_textures);
    }
}

#[derive(Component, Reflect, Clone, Debug, ShaderType, ExtractComponent)]
#[reflect(Component)]
pub struct DistanceField {
    pub threshold: f32,
}

#[derive(Component)]
struct DistanceFieldTexture {
    texture: CachedTexture
}

fn prepare_distance_field_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera, &DistanceField)>,
) {
    for (entity, camera, distance_field) in views {
        if let Some(viewport) = camera.physical_viewport_size {
            let texture_descriptor = TextureDescriptor {
                label: Some("distance_field_texture"),
                size: viewport
                    .as_vec2()
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

            let texture = texture_cache.get(&render_device, texture_descriptor);

            commands
                .entity(entity)
                .insert(DistanceFieldTexture {
                    texture: texture
                });
        }
    }
}
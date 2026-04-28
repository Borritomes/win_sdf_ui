use bevy::{
    image::ToExtents,
    prelude::*,
    render::{
        Render, RenderApp,
        camera::ExtractedCamera,
        render_resource::{
            Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
        },
        renderer::RenderDevice,
        texture::{CachedTexture, TextureCache},
    },
};

use crate::TEXTURE_FORMAT;

pub struct PingPongPlugin;

impl Plugin for PingPongPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app
            .get_sub_app_mut(RenderApp)
            .expect("failed to get RenderApp");

        render_app.add_systems(Render, prepare_textures);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct PingPongMarker;

#[derive(Clone, Eq, PartialEq)]
pub enum PingPongTextureType {
    A,
    B,
}

#[derive(Clone)]
pub struct PingPongWrite<'a> {
    source: &'a TextureView,
    source_texture: &'a Texture,
    destination: &'a TextureView,
    destination_texture: &'a Texture,
}

#[derive(Clone)]
pub struct PingPongTextures {
    texture_a: CachedTexture,
    texture_b: CachedTexture,
    read: PingPongTextureType,
}

impl PingPongTextures {
    // the initial write is camera output -> texture, so we don't want to swap
    pub fn initial_write(&mut self) -> PingPongWrite<'_> {
        return PingPongWrite {
            source: &self.texture_a.default_view,
            source_texture: &self.texture_a.texture,
            destination: &self.texture_b.default_view,
            destination_texture: &self.texture_b.texture,
        };
    }

    pub fn write(&mut self) -> PingPongWrite<'_> {
        if self.read == PingPongTextureType::A {
            self.read = PingPongTextureType::B;
            return PingPongWrite {
                source: &self.texture_a.default_view,
                source_texture: &self.texture_a.texture,
                destination: &self.texture_b.default_view,
                destination_texture: &self.texture_b.texture,
            };
        } else {
            self.read = PingPongTextureType::A;
            return PingPongWrite {
                source: &self.texture_b.default_view,
                source_texture: &self.texture_b.texture,
                destination: &self.texture_a.default_view,
                destination_texture: &self.texture_a.texture,
            };
        }
    }
}

#[derive(Component, Clone)]
struct SdfTextures {
    regular: PingPongTextures,
    invert: PingPongTextures,
}

fn prepare_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera), With<PingPongMarker>>,
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
            let texture_regular_b = texture_cache.get(&render_device, texture_descriptor_regular_b);
            let texture_invert_a = texture_cache.get(&render_device, texture_descriptor_invert_a);
            let texture_invert_b = texture_cache.get(&render_device, texture_descriptor_invert_b);

            commands.entity(entity).insert(SdfTextures {
                regular: PingPongTextures {
                    texture_a: texture_regular_a,
                    texture_b: texture_regular_b,
                    read: PingPongTextureType::A,
                },
                invert: PingPongTextures {
                    texture_a: texture_invert_a,
                    texture_b: texture_invert_b,
                    read: PingPongTextureType::A,
                },
            });
        }
    }
}

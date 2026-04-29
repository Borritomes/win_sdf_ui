use bevy::{
    camera::{ImageRenderTarget, RenderTarget},
    color::palettes::css,
    prelude::*,
    render::render_resource::TextureFormat,
    window::WindowResolution,
};

use crate::{
    distance_field::{DistanceFieldImage, DistanceFieldPlugin, DistanceFieldSettings},
    distance_to_value::{DistanceToValuePlugin, DistanceToValueSettings},
    ping_pong::{PingPongMarker, PingPongPlugin},
    threshold::{ThresholdPlugin, ThresholdSettings},
    uv_to_color::{ColorToUVMarker, UVToColorPlugin},
};

mod distance_field;
mod distance_to_value;
mod ping_pong;
mod threshold;
mod uv_to_color;

pub const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
pub const RESOLUTION: u32 = 1024;

fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(512, 512),
                title: "win_sdf_ui".into(),
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest()),));

    app.add_systems(Startup, setup);
    app.add_systems(FixedUpdate, (ui_circle_move, worldspace_circle_move));
    app.add_observer(fullscreen_sprite_on_add);
    app.add_observer(on_distance_field_settings_add);
    app.add_systems(
        Update,
        (toggle_fullscreen_sprite_system, fullsceen_sprite_system).chain(),
    );

    app.add_plugins((
        PingPongPlugin,
        ThresholdPlugin,
        UVToColorPlugin,
        DistanceFieldPlugin,
        DistanceToValuePlugin,
    ));

    app.run();
}

fn on_distance_field_settings_add(
    event: On<Add, DistanceFieldSettings>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let image = Image::new_target_texture(RESOLUTION, RESOLUTION, TEXTURE_FORMAT, Some(TEXTURE_FORMAT));
    let handle = images.add(image);

    // commands.spawn((
    //     Name::new("Rgba32FloatImage"),
    //     Transform {
    //         translation: Vec3::new(-256.0, 0.0, 1.0),
    //         ..default()
    //     },
    //     Sprite::from_image(handle.clone()),
    //     FullscreenSprite::default(),
    // ));
    commands.spawn((
        Node {
            height: vh(100),
            width: vw(100),
            ..default()
        },
        ImageNode {
            image: handle.clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        }
    ));

    commands
        .entity(event.entity)
        .insert(DistanceFieldImage(handle.clone()))
        .insert(RenderTarget::Image(ImageRenderTarget {
            handle: handle,
            scale_factor: 1.0,
        }));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            ..default()
        },
        Camera {
            clear_color: ClearColorConfig::Custom(css::BLACK.into()),
            ..default()
        },
        PingPongMarker,
        DistanceFieldSettings {
            radius: 16.0,
            threshold: 0.5,
        },
        DistanceToValueSettings { threshold: 0.5, radius: 64.0 },
        ThresholdSettings { threshold: 0.5 },
        ColorToUVMarker,
    ));
    commands.spawn((
        Name::new("MainCamera"),
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Srgba::new(0.0, 0.0, 0.0, 0.0).into()),
            ..default()
        },
    ));

    commands.spawn((
        Name::new("Image"),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.5),
            ..default()
        },
        Sprite {
            image: asset_server.load("images/rect_rounded_stroke.png"),
            custom_size: Some(Vec2::new(RESOLUTION as f32 / 2.0, RESOLUTION as f32 / 2.0)),
            ..default()
        },
        FullscreenSprite::default(),
        MoveInCircle {
            rotation: 0.0,
            speed: 4.0,
            radius: 32.0
        },
    ));

    commands.spawn((
        Name::new("Image"),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.5),
            ..default()
        },
        Sprite {
            image: asset_server.load("images/rect_rounded_stroke.png"),
            custom_size: Some(Vec2::new(RESOLUTION as f32 / 2.0, RESOLUTION as f32 / 2.0)),
            ..default()
        },
        FullscreenSprite::default(),
        MoveInCircle {
            rotation: 180.0,
            speed: 2.0,
            radius: 32.0
        },
    ));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct FullscreenSprite {
    is_fullscreen: bool,
    original_size: Option<Vec2>,
}

impl Default for FullscreenSprite {
    fn default() -> Self {
        return FullscreenSprite {
            is_fullscreen: false,
            original_size: None,
        };
    }
}

fn fullscreen_sprite_on_add(
    event: On<Add, FullscreenSprite>,
    mut query_fullscreen_sprite: Query<&mut FullscreenSprite>,
    query_sprite: Query<&Sprite>,
) {
    let Ok(mut fullscreen_sprite) = query_fullscreen_sprite.get_mut(event.entity.entity()) else {
        return;
    };
    let Ok(sprite) = query_sprite.get(event.entity.entity()) else {
        return;
    };

    fullscreen_sprite.original_size = sprite.custom_size
}

fn fullsceen_sprite_system(
    query: Query<(&mut Sprite, &FullscreenSprite)>,
    window: Single<&Window>,
) {
    for (mut sprite, fullscreen_sprite) in query {
        if fullscreen_sprite.is_fullscreen {
            sprite.custom_size = Some(window.size());
        } else {
            sprite.custom_size = fullscreen_sprite.original_size;
        }
    }
}

fn toggle_fullscreen_sprite_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<&mut FullscreenSprite>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        for mut fullscreen_sprite in query {
            fullscreen_sprite.is_fullscreen = !fullscreen_sprite.is_fullscreen;
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
struct MoveInCircle {
    radius: f32,
    speed: f32,
    rotation: f32,
}

impl Default for MoveInCircle {
    fn default() -> Self {
        return MoveInCircle {
            radius: 16.0,
            speed: 1.0,
            rotation: 0.0,
        };
    }
}

fn ui_circle_move(query: Query<(&mut MoveInCircle, &mut UiTransform)>) {
    for (mut move_in_circle, mut transform) in query {
        let vec = Vec2::from_angle(0.01745329 * move_in_circle.rotation) * move_in_circle.radius;
        transform.translation = Val2::new(px(vec.x), px(vec.y));
        move_in_circle.rotation += move_in_circle.speed;
    }
}

fn worldspace_circle_move(query: Query<(&mut MoveInCircle, &mut Transform)>) {
    for (mut move_in_circle, mut transform) in query {
        let vec = Vec2::from_angle(0.01745329 * move_in_circle.rotation) * move_in_circle.radius;
        transform.translation = vec.extend(transform.translation.z);
        move_in_circle.rotation += move_in_circle.speed;
    }
}

use bevy::{prelude::*, window::WindowResolution};

use crate::distance_field::{DistanceFieldPlugin, DistanceFieldSettings};

mod distance_field;

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
    app.add_systems(FixedUpdate, ui_circle_move);
    app.add_observer(fullscreen_sprite_on_add);
    app.add_systems(
        Update,
        (toggle_fullscreen_sprite_system, fullsceen_sprite_system).chain(),
    );

    app.add_plugins(DistanceFieldPlugin);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        DistanceFieldSettings {
            radius: 16.0,
            threshold: 0.5,
        }
    ));

    commands.spawn((
        Name::new("Image"),
        Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            ..default()
        },
        Sprite {
            image: asset_server.load("images/sog.png"),
            custom_size: Some(Vec2::new(512.0, 512.0)),
            ..default()
        },
        FullscreenSprite::default(),
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

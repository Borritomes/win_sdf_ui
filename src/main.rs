use bevy::{
    color::palettes::css, ecs::entity_disabling::Disabled, prelude::*, window::WindowResolution,
};
use threshold_post_process::{ThresholdPostProcessPlugin, ThresholdSettings};

use crate::{
    color_to_uv::{ColorToUVMarker, ColorToUVPlugin},
    distance_field::{DistanceFieldPlugin, DistanceFieldSettings},
};

mod color_to_uv;
mod distance_field;
mod threshold_post_process;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(512, 512),
                    title: "win_sdf_ui".into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    app.add_systems(Startup, setup);
    app.add_systems(FixedUpdate, (circle_move, toggle_disable_system));

    app.add_plugins((
        ThresholdPostProcessPlugin,
        DistanceFieldPlugin,
        ColorToUVPlugin,
    ));

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        ThresholdSettings { threshold: 0.5 },
        DistanceFieldSettings { radius: 1024 },
        ColorToUVMarker,
    ));

    // commands.spawn((
    //     Name::new("Lock"),
    //     Transform {
    //         translation: Vec3::new(0.0, 0.0, 1.0),
    //         ..default()
    //     },
    //     Sprite {
    //         image: asset_server.load("images/locked.png"),
    //         custom_size: Some(Vec2::new(48.0, 48.0)),
    //         ..default()
    //     },
    // ));

    commands.spawn((
        Name::new("WhiteCube"),
        Transform {
            translation: Vec3::new(64.0, 0.0, 1.0),
            rotation: Quat::from_rotation_z(30.0),
            ..default()
        },
        Sprite::from_color(css::WHITE, Vec2::new(32.0, 32.0)),
    ));
    commands.spawn((
        Name::new("WhiteCube"),
        Transform {
            translation: Vec3::new(-64.0, 0.0, 1.0),
            rotation: Quat::from_rotation_z(30.0),
            ..default()
        },
        Sprite::from_color(css::WHITE, Vec2::new(32.0, 32.0)),
    ));

    // commands.spawn((
    //     Name::new("Logo"),
    //     Transform {
    //         translation: Vec3::new(256.0, -256.0, 1.0),
    //         ..default()
    //     },
    //     Sprite::from_image(asset_server.load("images/bevy_icon_dark.png")),
    // ));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ToggleDisabled(bool);

fn toggle_disable_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &mut ToggleDisabled, Has<Disabled>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (entity, mut toggle_disabled, _) in query {
            if toggle_disabled.0 == true {
                println!("enable");
                commands.entity(entity).remove::<Disabled>();
                toggle_disabled.0 = false
            } else {
                println!("disable");
                commands.entity(entity).insert(Disabled);
                toggle_disabled.0 = true
            }
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

fn circle_move(query: Query<(&mut MoveInCircle, &mut UiTransform)>) {
    for (mut move_in_circle, mut transform) in query {
        let vec = Vec2::from_angle(0.01745329 * move_in_circle.rotation) * move_in_circle.radius;
        transform.translation = Val2::new(px(vec.x), px(vec.y));
        move_in_circle.rotation += move_in_circle.speed;
    }
}

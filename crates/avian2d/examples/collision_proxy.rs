//! Demonstrates collision proxies by wrapping an object around the screen edges.

#![allow(clippy::unnecessary_cast)]

use avian2d::prelude::*;
use bevy::{
    prelude::*,
    render::camera::ScalingMode,
    window::{EnabledButtons, WindowResolution},
};
use bevy_math::CompassOctant;
use examples_common_2d::ExampleCommonPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Collision Proxies".into(),
                    resolution: WindowResolution::new(960.0, 960.0),
                    resizable: false,
                    enabled_buttons: EnabledButtons {
                        minimize: true,
                        maximize: false,
                        close: true,
                    },
                    ..default()
                }),
                ..default()
            }),
            ExampleCommonPlugin,
            PhysicsPlugins::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.01, 0.01, 0.025)))
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_ball, update_proxies, wrap_edges))
        .run();
}

#[derive(Component)]
struct Player;

fn setup(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 10.0,
                height: 10.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    let ball = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::circle(1.0),
            Sprite {
                image: assets.load("ball.png"),
                custom_size: Some(vec2(2.0, 2.0)),
                ..default()
            },
        ))
        .id();
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(1.0, 5.0),
        Transform::from_xyz(4.0, 0.0, 0.0),
        Sprite {
            custom_size: Some(vec2(1.0, 5.0)),
            ..default()
        },
    ));
    // Spawn ball proxies
    for i in 0..8 {
        commands.spawn((
            ProxyFor {
                target: ball,
                octant: CompassOctant::from_index(i),
            },
            Sprite {
                image: assets.load("ball.png"),
                custom_size: Some(vec2(2.0, 2.0)),
                ..default()
            },
        ));
    }
}

fn move_ball(keyboard: Res<ButtonInput<KeyCode>>, mut ball: Single<Forces>) {
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= Vec2::X;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += Vec2::X;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= Vec2::Y;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        direction += Vec2::Y;
    }
    ball.apply_force(direction * 15.0);
}

fn wrap_edges(mut bodies: Query<&mut Transform, With<RigidBody>>) {
    const THRESHOLD: f32 = 0.2;
    for mut body in &mut bodies {
        if body.translation.x > 5.0 + THRESHOLD {
            body.translation.x -= 10.0;
        }
        if body.translation.x < -5.0 - THRESHOLD {
            body.translation.x += 10.0;
        }
        if body.translation.y > 5.0 + THRESHOLD {
            body.translation.y -= 10.0;
        }
        if body.translation.y < -5.0 - THRESHOLD {
            body.translation.y += 10.0;
        }
    }
}

#[derive(Component)]
#[relationship_target(relationship = ProxyFor)]
struct Proxies(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Proxies)]
struct ProxyFor {
    #[relationship]
    target: Entity,
    octant: Option<CompassOctant>,
}

fn update_proxies(
    bodies: Query<(&Transform, &Proxies), Without<ProxyFor>>,
    mut proxies: Query<(&ProxyFor, &mut Transform)>,
) -> Result {
    for (body, body_proxies) in &bodies {
        for proxy in body_proxies.iter() {
            let (ProxyFor { octant, .. }, mut proxy) = proxies.get_mut(proxy)?;
            let offset = match octant.unwrap() {
                CompassOctant::North => vec2(0.0, 1.0),
                CompassOctant::NorthEast => vec2(1.0, 1.0),
                CompassOctant::East => vec2(1.0, 0.0),
                CompassOctant::SouthEast => vec2(1.0, -1.0),
                CompassOctant::South => vec2(0.0, -1.0),
                CompassOctant::SouthWest => vec2(-1.0, -1.0),
                CompassOctant::West => vec2(-1.0, 0.0),
                CompassOctant::NorthWest => vec2(-1.0, 1.0),
            };
            proxy.translation = body.translation + (offset * 10.0).extend(0.0);
            proxy.rotation = body.rotation;
        }
    }
    Ok(())
}

//! Basic 2D shooting example using the bevy_bullet_dynamics crate.
//!
//! This example demonstrates how to set up a simple 2D shooting game with
//! projectile physics, accuracy mechanics, and visual effects.

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_bullet_dynamics::prelude::*;

const PLAYER_SPEED: f32 = 300.0;
const BULLET_SPEED: f32 = 800.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(BallisticsPluginGroup)
        .insert_resource(BallisticsEnvironment {
            gravity: Vec3::ZERO, // No gravity in 2D top-down
            air_density: 1.15,   // Slightly higher for more drag
            wind: Vec3::ZERO,
            temperature: 20.0,
            altitude: 0.0,
        })
        .insert_resource(BallisticsConfig {
            use_rk4: true,
            max_projectile_lifetime: 5.0,
            max_projectile_distance: 1000.0,
            enable_penetration: false,
            enable_ricochet: false,
            debug_draw: false,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                player_shooting,
                spawn_visual_effects,
                cleanup_projectiles,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Target {
    health: f32,
}

#[derive(Resource)]
struct PlayerEntity(Entity);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera setup for 2D
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize(1.0),
            ..default()
        },
        ..default()
    });

    // Spawn player
    let player_entity = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.0, 0.8, 1.0),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                ..default()
            },
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    commands.insert_resource(PlayerEntity(player_entity));

    // Spawn some enemies
    for i in -2..=2 {
        for j in -2..=2 {
            if i == 0 && j == 0 {
                continue; // Skip player position
            }
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 0.3, 0.3),
                        custom_size: Some(Vec2::new(25.0, 25.0)),
                        ..default()
                    },
                    ..default()
                },
                Enemy,
                Transform::from_xyz(i as f32 * 100.0, j as f32 * 100.0, 0.0),
            ));
        }
    }

    // Spawn some targets
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.8, 0.8, 0.2),
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            ..default()
        },
        Target { health: 100.0 },
        Transform::from_xyz(200.0, 0.0, 0.0),
    ));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
        player_transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

fn player_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    player_entity: Res<PlayerEntity>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let player_transform = player_query.get(player_entity.0).unwrap();
        
        // Calculate shoot direction (towards mouse cursor would be better in a real game)
        let direction = Vec3::X; // Shooting right by default
        
        // Create projectile spawn parameters
        let spawn_params = ProjectileSpawnParams::new(
            player_transform.translation + direction * 20.0, // Spawn slightly in front of player
            direction,
            BULLET_SPEED,
        )
        .with_damage(25.0)
        .with_owner(player_entity.0);

        // Spawn the projectile with physics components
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.8, 0.2),
                    custom_size: Some(Vec2::new(5.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_translation(spawn_params.origin)
                    .with_rotation(Quat::from_rotation_z(
                        spawn_params.direction.y.atan2(spawn_params.direction.x),
                    )),
                ..default()
            },
            Projectile::new(spawn_params.direction * spawn_params.velocity)
                .with_owner(spawn_params.owner.unwrap())
                .with_mass(spawn_params.mass)
                .with_drag(spawn_params.drag),
            Accuracy::default(),
            Payload::Kinetic {
                damage: spawn_params.damage,
            },
            ProjectileLogic::Impact,
        ));

        // Fire event for systems to pick up
        commands.trigger(FireEvent::new(
            spawn_params.origin,
            spawn_params.direction,
            spawn_params.velocity,
        ));
    }
}

fn spawn_visual_effects(
    mut commands: Commands,
    mut hit_events: EventReader<HitEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in hit_events.read() {
        // Spawn visual effect at hit location
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.3, 0.3),
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                transform: Transform::from_translation(event.impact_point),
                ..default()
            },
            // Temporary visual effect that will be cleaned up
            TemporaryEffect { lifetime: 0.5 },
        ));
    }
}

#[derive(Component)]
struct TemporaryEffect {
    lifetime: f32,
}

fn cleanup_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TemporaryEffect)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.lifetime -= time.delta_seconds();
        if effect.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
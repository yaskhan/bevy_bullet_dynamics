//! Basic 2D shooting example using the bevy_bullet_dynamics crate.
//!
//! This example demonstrates how to set up a simple 2D shooting game with
//! projectile physics, accuracy mechanics, and visual effects.

use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;

const PLAYER_SPEED: f32 = 300.0;
const BULLET_SPEED: f32 = 800.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BallisticsPluginGroup)
        .insert_resource(BallisticsEnvironment {
            gravity: Vec3::ZERO, // No gravity in 2D top-down
            air_density: 1.15,   // Slightly higher for more drag
            wind: Vec3::ZERO,
            temperature: 20.0,
            altitude: 0.0,
            latitude: 0.0,
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

#[derive(Resource)]
struct PlayerEntity(Entity);

fn setup(mut commands: Commands) {
    // Camera setup for 2D
    commands.spawn(Camera2d);

    // Spawn player
    let player_entity = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.0, 0.8, 1.0),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    commands.insert_resource(PlayerEntity(player_entity));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Some(mut player_transform) = player_query.iter_mut().next() {
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
            player_transform.translation += direction * PLAYER_SPEED * time.delta_secs();
        }
    }
}

fn player_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    player_entity: Res<PlayerEntity>,
    mut fire_events: MessageWriter<FireEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.get(player_entity.0) {
            let direction = Vec3::X; // Shooting right
            
            let spawn_params = ProjectileSpawnParams::new(
                player_transform.translation + direction * 20.0,
                direction,
                BULLET_SPEED,
            )
            .with_damage(25.0)
            .with_owner(player_entity.0);

            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.8, 0.2),
                    custom_size: Some(Vec2::new(5.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(spawn_params.origin)
                    .with_rotation(Quat::from_rotation_z(
                        spawn_params.direction.y.atan2(spawn_params.direction.x),
                    )),
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

            fire_events.write(FireEvent::new(
                spawn_params.origin,
                spawn_params.direction,
                spawn_params.velocity,
            ));
        }
    }
}

fn spawn_visual_effects(
    mut commands: Commands,
    mut hit_events: MessageReader<HitEvent>,
) {
    for event in hit_events.read() {
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.3, 0.3),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            Transform::from_translation(event.impact_point),
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
        effect.lifetime -= time.delta_secs();
        if effect.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
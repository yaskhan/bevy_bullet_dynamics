//! Advanced 2D shooting example with accuracy mechanics and multiple weapon types.
//!
//! This example demonstrates how to use the accuracy system, different weapon presets,
//! and visual effects in a 2D top-down shooter.

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_bullet_dynamics::prelude::*;

const PLAYER_SPEED: f32 = 300.0;
const PISTOL_BULLET_SPEED: f32 = 400.0;
const RIFLE_BULLET_SPEED: f32 = 800.0;
const SHOTGUN_BULLET_SPEED: f32 = 300.0;

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
            enable_penetration: true,
            enable_ricochet: true,
            debug_draw: false,
        })
        .insert_resource(WeaponPresets::with_defaults())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                weapon_switching,
                player_shooting,
                update_ui,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerUI;

#[derive(Component)]
struct Enemy;

#[derive(Resource)]
struct PlayerEntity(Entity);

#[derive(Resource)]
struct CurrentWeapon(usize); // Index into WeaponPresets

#[derive(Resource)]
struct PlayerStats {
    weapon_index: usize,
    shots_fired: u32,
    hits: u32,
    accuracy: f32,
}

fn setup(mut commands: Commands) {
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
    commands.insert_resource(CurrentWeapon(0)); // Start with pistol
    commands.insert_resource(PlayerStats {
        weapon_index: 0,
        shots_fired: 0,
        hits: 0,
        accuracy: 0.0,
    });

    // Spawn some enemies
    for i in -3..=3 {
        for j in -3..=3 {
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
                Transform::from_xyz(i as f32 * 120.0, j as f32 * 120.0, 0.0),
            ));
        }
    }

    // Spawn UI
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "2D Ballistics Demo\n",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "WASD: Move | SPACE: Shoot | 1-3: Switch Weapon\n",
                TextStyle {
                    font_size: 20.0,
                    color: Color::YELLOW,
                    ..default()
                },
            ),
            TextSection::new(
                "Current Weapon: Pistol | Shots: 0 | Hits: 0 | Accuracy: 0%\n",
                TextStyle {
                    font_size: 18.0,
                    color: Color::GREEN,
                    ..default()
                },
            ),
            TextSection::new(
                "Press SPACE to fire!\n",
                TextStyle {
                    font_size: 16.0,
                    color: Color::BLUE,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        PlayerUI,
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

fn weapon_switching(
    mut current_weapon: ResMut<CurrentWeapon>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_stats: ResMut<PlayerStats>,
    weapon_presets: Res<WeaponPresets>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        current_weapon.0 = 0; // Pistol
        player_stats.weapon_index = 0;
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        current_weapon.0 = 1; // Rifle
        player_stats.weapon_index = 1;
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        current_weapon.0 = 2; // Sniper
        player_stats.weapon_index = 2;
    }
    
    // Reset stats when switching weapons
    if keyboard_input.any_just_pressed([KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]) {
        player_stats.shots_fired = 0;
        player_stats.hits = 0;
        player_stats.accuracy = 0.0;
    }
}

fn player_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    player_entity: Res<PlayerEntity>,
    current_weapon: Res<CurrentWeapon>,
    mut player_stats: ResMut<PlayerStats>,
    weapon_presets: Res<WeaponPresets>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let player_transform = player_query.get(player_entity.0).unwrap();
        
        // Get weapon preset
        let weapon_preset = &weapon_presets.presets[current_weapon.0];
        
        // Calculate shoot direction (towards mouse cursor would be better in a real game)
        let direction = Vec3::X; // Shooting right by default
        
        // Create projectile spawn parameters based on weapon preset
        let spawn_params = ProjectileSpawnParams::new(
            player_transform.translation + direction * 20.0, // Spawn slightly in front of player
            direction,
            weapon_preset.muzzle_velocity,
        )
        .with_damage(weapon_preset.base_damage)
        .with_mass(weapon_preset.projectile_mass)
        .with_owner(player_entity.0);

        // Apply accuracy mechanics
        let accuracy = &weapon_preset.accuracy;
        let spread_angle = accuracy.base_spread + accuracy.current_bloom;
        
        // Add some randomness to direction based on accuracy
        let random_angle = (rand::random::<f32>() - 0.5) * 2.0 * spread_angle;
        let rotated_direction = Quat::from_rotation_z(random_angle) * direction;
        
        // Spawn the projectile with physics components
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: match current_weapon.0 {
                        0 => Color::srgb(1.0, 0.8, 0.2), // Pistol - yellow
                        1 => Color::srgb(0.8, 0.2, 1.0), // Rifle - purple
                        2 => Color::srgb(0.2, 1.0, 0.8), // Sniper - cyan
                        _ => Color::srgb(1.0, 1.0, 1.0), // White
                    },
                    custom_size: Some(match current_weapon.0 {
                        0 => Vec2::new(4.0, 2.0), // Pistol
                        1 => Vec2::new(6.0, 3.0), // Rifle
                        2 => Vec2::new(8.0, 4.0), // Sniper
                        _ => Vec2::new(5.0, 3.0),
                    }),
                    ..default()
                },
                transform: Transform::from_translation(spawn_params.origin)
                    .with_rotation(Quat::from_rotation_z(
                        rotated_direction.y.atan2(rotated_direction.x),
                    )),
                ..default()
            },
            Projectile::new(rotated_direction * spawn_params.velocity)
                .with_owner(spawn_params.owner.unwrap())
                .with_mass(spawn_params.mass)
                .with_drag(weapon_preset.drag_coefficient),
            weapon_preset.accuracy.clone(),
            Payload::Kinetic {
                damage: spawn_params.damage,
            },
            ProjectileLogic::Impact,
        ));

        // Fire event for systems to pick up
        commands.trigger(FireEvent::new(
            spawn_params.origin,
            rotated_direction,
            spawn_params.velocity,
        ).with_seed(rand::random::<u64>()));

        // Update player stats
        player_stats.shots_fired += 1;
    }
}

fn update_ui(
    mut ui_query: Query<&mut Text, With<PlayerUI>>,
    current_weapon: Res<CurrentWeapon>,
    player_stats: Res<PlayerStats>,
    weapon_presets: Res<WeaponPresets>,
) {
    let mut text = ui_query.single_mut();
    
    let weapon_names = ["Pistol", "Rifle", "Sniper"];
    let current_weapon_name = weapon_names.get(current_weapon.0).unwrap_or(&"Unknown");
    
    if player_stats.shots_fired > 0 {
        text.sections[2].value = format!(
            "Current Weapon: {} | Shots: {} | Hits: {} | Accuracy: {:.1}%\n",
            current_weapon_name,
            player_stats.shots_fired,
            player_stats.hits,
            (player_stats.hits as f32 / player_stats.shots_fired as f32) * 100.0
        );
    } else {
        text.sections[2].value = format!(
            "Current Weapon: {} | Shots: {} | Hits: {} | Accuracy: {:.1}%\n",
            current_weapon_name,
            player_stats.shots_fired,
            player_stats.hits,
            0.0
        );
    }
    
    // Update weapon info
    if let Some(preset) = weapon_presets.presets.get(current_weapon.0) {
        text.sections[3].value = format!(
            "Muzzle Vel: {:.0} m/s | Damage: {:.0} | Spread: {:.4} rad\n",
            preset.muzzle_velocity,
            preset.base_damage,
            preset.accuracy.base_spread + preset.accuracy.current_bloom
        );
    }
}

// Simple random number generator for demo purposes
mod rand {
    pub fn random<T>() -> T 
    where 
        T: RandomValue 
    {
        T::generate()
    }
    
    pub trait RandomValue {
        fn generate() -> Self;
    }
    
    impl RandomValue for f32 {
        fn generate() -> Self {
            // Simple pseudo-random generator
            static mut SEED: u32 = 12345;
            unsafe {
                SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
                (SEED >> 16) as f32 / 65536.0
            }
        }
    }
    
    impl RandomValue for u64 {
        fn generate() -> Self {
            // Simple pseudo-random generator
            static mut SEED: u64 = 987654321;
            unsafe {
                SEED = SEED.wrapping_mul(2862933555777941757).wrapping_add(3037000493);
                SEED
            }
        }
    }
}
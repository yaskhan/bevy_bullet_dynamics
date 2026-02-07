//! Advanced 2D shooting example with accuracy mechanics and multiple weapon types.

use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;

const PLAYER_SPEED: f32 = 300.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BallisticsPluginGroup)
        .insert_resource(BallisticsEnvironment {
            gravity: Vec3::ZERO,
            air_density: 1.15,
            wind: Vec3::ZERO,
            temperature: 20.0,
            altitude: 0.0,
            latitude: 0.0,
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

#[derive(Resource)]
struct PlayerEntity(Entity);

#[derive(Resource)]
struct CurrentWeapon(usize);

#[derive(Resource)]
struct PlayerStats {
    weapon_index: usize,
    shots_fired: u32,
    hits: u32,
}

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
    commands.insert_resource(CurrentWeapon(0));
    commands.insert_resource(PlayerStats {
        weapon_index: 0,
        shots_fired: 0,
        hits: 0,
    });

    // Spawn UI (Bevy 0.18 style)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        PlayerUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("2D Ballistics Demo\n"),
            TextFont { font_size: 30.0, ..default() },
        ));
        parent.spawn((
            Text::new("WASD: Move | SPACE: Shoot | 1-3: Switch Weapon\n"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 0.0)),
        ));
        parent.spawn((
            Text::new("Current Weapon: Pistol | Shots: 0 | Hits: 0\n"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            WeaponInfoText,
        ));
        parent.spawn((
            Text::new("Press SPACE to fire!\n"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.0, 0.0, 1.0)),
            StatusText,
        ));
    });
}

#[derive(Component)]
struct WeaponInfoText;

#[derive(Component)]
struct StatusText;

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

fn weapon_switching(
    mut current_weapon: ResMut<CurrentWeapon>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_stats: ResMut<PlayerStats>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        current_weapon.0 = 0;
        player_stats.weapon_index = 0;
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        current_weapon.0 = 1;
        player_stats.weapon_index = 1;
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        current_weapon.0 = 2;
        player_stats.weapon_index = 2;
    }
    
    if keyboard_input.any_just_pressed([KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]) {
        player_stats.shots_fired = 0;
        player_stats.hits = 0;
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
    mut fire_events: MessageWriter<FireEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.get(player_entity.0) {
            let weapon_preset = &weapon_presets.presets[current_weapon.0];
            let direction = Vec3::X;
            
            let spawn_params = ProjectileSpawnParams::new(
                player_transform.translation + direction * 20.0,
                direction,
                weapon_preset.muzzle_velocity,
            )
            .with_damage(weapon_preset.base_damage)
            .with_mass(weapon_preset.projectile_mass)
            .with_owner(player_entity.0);

            let accuracy_comp = &weapon_preset.accuracy;
            let spread_angle = accuracy_comp.base_spread + accuracy_comp.current_bloom;
            
            let random_angle = (::rand::random::<f32>() - 0.5) * 2.0 * spread_angle;
            let rotated_direction = Quat::from_rotation_z(random_angle) * direction;
            
            commands.spawn((
                Sprite {
                    color: match current_weapon.0 {
                        0 => Color::srgb(1.0, 0.8, 0.2),
                        1 => Color::srgb(0.8, 0.2, 1.0),
                        2 => Color::srgb(0.2, 1.0, 0.8),
                        _ => Color::srgb(1.0, 1.0, 1.0),
                    },
                    custom_size: Some(match current_weapon.0 {
                        0 => Vec2::new(4.0, 2.0),
                        1 => Vec2::new(6.0, 3.0),
                        2 => Vec2::new(8.0, 4.0),
                        _ => Vec2::new(5.0, 3.0),
                    }),
                    ..default()
                },
                Transform::from_translation(spawn_params.origin)
                    .with_rotation(Quat::from_rotation_z(
                        rotated_direction.y.atan2(rotated_direction.x),
                    )),
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

            fire_events.write(FireEvent::new(
                spawn_params.origin,
                rotated_direction,
                spawn_params.velocity,
            ).with_seed(::rand::random::<u64>()));

            player_stats.shots_fired += 1;
        }
    }
}

fn update_ui(
    mut weapon_info_query: Query<&mut Text, (With<WeaponInfoText>, Without<StatusText>)>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<WeaponInfoText>)>,
    current_weapon: Res<CurrentWeapon>,
    player_stats: Res<PlayerStats>,
    weapon_presets: Res<WeaponPresets>,
) {
    let weapon_names = ["Pistol", "Rifle", "Sniper"];
    let current_weapon_name = weapon_names.get(current_weapon.0).unwrap_or(&"Unknown");
    
    if let Some(mut text) = weapon_info_query.iter_mut().next() {
        text.0 = format!(
            "Current Weapon: {} | Shots: {} | Hits: {}\n",
            current_weapon_name,
            player_stats.shots_fired,
            player_stats.hits
        );
    }
    
    if let Some(mut text) = status_query.iter_mut().next() {
        if let Some(preset) = weapon_presets.presets.get(current_weapon.0) {
            text.0 = format!(
                "Muzzle Vel: {:.0} m/s | Damage: {:.0} | Spread: {:.4} rad\n",
                preset.muzzle_velocity,
                preset.base_damage,
                preset.accuracy.base_spread + preset.accuracy.current_bloom
            );
        }
    }
}
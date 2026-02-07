//! Basic shooting example demonstrating the ballistics system.

use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BallisticsPluginGroup)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, update_ui))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
    ));

    // Target boxes
    let box_mesh = meshes.add(Cuboid::new(2.0, 2.0, 2.0));
    let box_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        ..default()
    });

    for i in 0..5 {
        commands.spawn((
            Mesh3d(box_mesh.clone()),
            MeshMaterial3d(box_material.clone()),
            Transform::from_xyz(-8.0 + i as f32 * 4.0, 1.0, -10.0),
            SurfaceMaterial::default(),
        ));
    }

    // Shooter position marker
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.8),
            emissive: LinearRgba::rgb(0.5, 0.5, 2.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 5.0),
        ShooterMarker,
    ));

    // UI instructions
    commands.spawn((
        Text::new("Press SPACE to shoot\nPress 1-4 for weapon types\nCurrent: Rifle"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        UiText,
    ));

    // Weapon state
    commands.insert_resource(WeaponState {
        weapon_type: WeaponType::Rifle,
        accuracy: bevy_bullet_dynamics::systems::accuracy::presets::rifle(),
        weapon: WeaponType::Rifle.weapon_config(),
    });
}

#[derive(Component)]
struct ShooterMarker;

#[derive(Component)]
struct UiText;

#[derive(Resource)]
struct WeaponState {
    weapon_type: WeaponType,
    accuracy: Accuracy,
    weapon: Weapon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WeaponType {
    Pistol,
    Rifle,
    Sniper,
    SMG,
    Shotgun,
    Launcher,
    Laser,
}

impl WeaponType {
    fn name(&self) -> &'static str {
        match self {
            Self::Pistol => "Pistol",
            Self::Rifle => "Rifle",
            Self::Sniper => "Sniper",
            Self::SMG => "SMG",
            Self::Shotgun => "Shotgun",
            Self::Launcher => "Launcher",
            Self::Laser => "Laser",
        }
    }

    fn muzzle_velocity(&self) -> f32 {
        match self {
            Self::Pistol => 350.0,
            Self::Rifle => 900.0,
            Self::Sniper => 1200.0,
            Self::SMG => 400.0,
            Self::Shotgun => 350.0,
            Self::Launcher => 50.0, // Slow missile
            Self::Laser => 0.0, // Instant
        }
    }

    fn accuracy(&self) -> Accuracy {
        use bevy_bullet_dynamics::systems::accuracy::presets;
        match self {
            Self::Pistol => presets::pistol(),
            Self::Rifle => presets::rifle(),
            Self::Sniper => presets::sniper(),
            Self::SMG => presets::smg(),
            Self::Shotgun => presets::shotgun(),
            Self::Launcher => presets::rifle(), // Use rifle accuracy for now
            Self::Laser => presets::sniper(), // High accuracy
        }
    }

    fn weapon_config(&self) -> Weapon {
        let mut weapon = Weapon::default();
        match self {
            Self::Pistol => { weapon.fire_rate = 5.0; }
            Self::Rifle => { weapon.fire_rate = 8.0; }
            Self::Sniper => { weapon.fire_rate = 1.0; }
            Self::SMG => { 
                weapon.fire_rate = 12.0; 
                weapon.automatic = true;
            }
            Self::Shotgun => { weapon.fire_rate = 1.5; }
            Self::Launcher => { weapon.fire_rate = 0.5; }
            Self::Laser => { weapon.fire_rate = 2.0; }
        }
        weapon
    }
}

fn handle_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut weapon_state: ResMut<WeaponState>,
    shooter: Query<&Transform, With<ShooterMarker>>,
    targets: Query<Entity, With<SurfaceMaterial>>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Weapon selection
    let mut changed = false;
    if keyboard.just_pressed(KeyCode::Digit1) {
        weapon_state.weapon_type = WeaponType::Pistol;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        weapon_state.weapon_type = WeaponType::Rifle;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        weapon_state.weapon_type = WeaponType::Sniper;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        weapon_state.weapon_type = WeaponType::SMG;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        weapon_state.weapon_type = WeaponType::Shotgun;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        weapon_state.weapon_type = WeaponType::Launcher;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Digit7) {
        weapon_state.weapon_type = WeaponType::Laser;
        changed = true;
    }

    if changed {
        weapon_state.accuracy = weapon_state.weapon_type.accuracy();
        weapon_state.weapon = weapon_state.weapon_type.weapon_config();
        
        // Reset firing state
        weapon_state.weapon.last_fire_time = 0.0;
    }

    // Fire logic
    let trigger_pulled = if weapon_state.weapon.automatic {
        keyboard.pressed(KeyCode::Space)
    } else {
        keyboard.just_pressed(KeyCode::Space)
    };

    let current_time = time.elapsed_secs_f64();
    let can_fire = weapon_state.weapon.can_fire(current_time);

    if trigger_pulled && can_fire {
        let Ok(shooter_transform) = shooter.single() else {
            return;
        };

        // Update last fire time
        weapon_state.weapon.last_fire_time = current_time;

        let origin = shooter_transform.translation;
        let direction = Vec3::new(0.0, 0.0, -1.0); // Forward

        // Apply spread
        use bevy_bullet_dynamics::systems::accuracy;
        let spread_angle = accuracy::calculate_total_spread(
            &weapon_state.accuracy,
            false, // not aiming
            false, // not moving
            false, // not airborne
            0.0,
            5.0,
        );

        // Create projectile assets
        let projectile_mesh = meshes.add(Sphere::new(0.05));
        let projectile_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.8, 0.2),
            emissive: LinearRgba::rgb(5.0, 4.0, 1.0),
            ..default()
        });

        // Determine projectile count and damage
        let (projectile_count, damage) = match weapon_state.weapon_type {
            WeaponType::Shotgun => (8, 5.0),
            WeaponType::Sniper => (1, 80.0),
            WeaponType::Pistol => (1, 30.0),
            WeaponType::SMG => (1, 15.0),
            WeaponType::Rifle => (1, 40.0),
            WeaponType::Launcher => (1, 150.0),
            WeaponType::Laser => (1, 60.0),
        };

        // Find target for homing
        let target_entity = if weapon_state.weapon_type == WeaponType::Launcher {
            // Pick random target
            targets.iter().next()
        } else {
            None
        };

        for i in 0..projectile_count {
            let seed = rand::random::<u64>().wrapping_add(i as u64);
            let final_direction = accuracy::apply_spread_to_direction(direction, spread_angle, seed);

            let velocity = final_direction * weapon_state.weapon_type.muzzle_velocity();
            
            let mut entity_cmd = commands.spawn((
                Mesh3d(projectile_mesh.clone()),
                MeshMaterial3d(projectile_material.clone()),
                Transform::from_translation(origin),
                Projectile::new(velocity),
                Payload::Kinetic { damage },
                ProjectileLogic::Impact,
            ));

            if let Some(target) = target_entity {
                entity_cmd.insert(Guidance {
                    target: Some(target),
                    turn_rate: 2.0, // Radians/sec
                    delay: 0.2,
                    elapsed: 0.0,
                });
                entity_cmd.insert(Payload::Explosive { 
                    radius: 3.0, 
                    damage,
                    falloff: 0.5 
                });
                entity_cmd.insert(ProjectileLogic::Proximity { range: 1.0 });
            }

            if weapon_state.weapon_type == WeaponType::Laser {
                 entity_cmd.insert(ProjectileLogic::Hitscan { range: 1000.0 });
                 // Laser usually has Payload too (logic.rs handles damage via payload)
            }
        }

        // Apply bloom
        accuracy::apply_shot_bloom(&mut weapon_state.accuracy);
    }
}

fn update_ui(
    weapon_state: Res<WeaponState>,
    mut ui_text: Query<&mut Text, With<UiText>>,
) {
    if weapon_state.is_changed() {
        for mut text in ui_text.iter_mut() {
            text.0 = format!(
                "Press SPACE to shoot\nPress 1-5 for weapon types\nCurrent: {}\nBloom: {:.3}",
                weapon_state.weapon_type.name(),
                weapon_state.accuracy.current_bloom
            );
        }
    }
}

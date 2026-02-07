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
        sight: Sight::default(),
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
    sight: Sight,
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
    Flamethrower,
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
            Self::Flamethrower => "Flamethrower",
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
            Self::Flamethrower => 15.0, // Slow flame
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
            Self::Flamethrower => presets::shotgun(), // Wide spread
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
            Self::Flamethrower => { 
                weapon.fire_rate = 20.0; 
                weapon.automatic = true;
            }
        }
        weapon
    }

    /// Returns (mass, drag_coefficient, diameter, spin)
    pub fn physical_properties(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::Pistol => (0.008, 0.35, 0.009, 100.0),
            Self::Rifle => (0.004, 0.25, 0.00556, 2500.0),
            Self::Sniper => (0.010, 0.20, 0.00762, 3000.0),
            Self::SMG => (0.008, 0.35, 0.009, 150.0),
            Self::Shotgun => (0.003, 0.45, 0.008, 50.0), // Buckshot
            Self::Launcher => (1.0, 0.8, 0.1, 0.0), // Missile
            Self::Laser => (0.0, 0.0, 0.0, 0.0),
            Self::Flamethrower => (0.05, 2.0, 0.1, 0.0),
        }
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
    if keyboard.just_pressed(KeyCode::Digit8) {
        weapon_state.weapon_type = WeaponType::Flamethrower;
        changed = true;
    }

    // Zeroing adjustment
    if keyboard.just_pressed(KeyCode::PageUp) {
        let current = weapon_state.sight.current_zero;
        if let Some(next) = weapon_state.sight.zero_presets.iter().find(|&&z| z > current) {
            weapon_state.sight.current_zero = *next;
        }
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        let current = weapon_state.sight.current_zero;
        if let Some(prev) = weapon_state.sight.zero_presets.iter().rev().find(|&&z| z < current) {
            weapon_state.sight.current_zero = *prev;
        }
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
            WeaponType::Flamethrower => (3, 5.0), // Many small flames
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

            let (mass, drag, diameter, spin) = weapon_state.weapon_type.physical_properties();
            let velocity_mag = weapon_state.weapon_type.muzzle_velocity();
            
            // Calculate elevation angle for zero distance
            // Simplified ballistic arc approximation:
            // theta = 0.5 * asin(g * x / v^2)
            // But with drag it's harder.
            // For now use simple gravity drop compensation.
            // drop = 0.5 * g * t^2
            // t = dist / v
            // angle ~ drop / dist
            let zero_dist = weapon_state.sight.current_zero;
            let gravity = 9.81f32;
            let time_to_target = zero_dist / velocity_mag;
            let drop = 0.5 * gravity * time_to_target * time_to_target;
            let elevation_angle = (drop / zero_dist).atan(); // Rough approximation

            // Apply elevation to direction (rotate around Right vector)
            // Assuming direction is primarily -Z. Right is +X.
            // We rotate UP (towards +Y).
            let elevation_rot = Quat::from_rotation_x(elevation_angle);
            // Wait, if forward is -Z. Right is +X.
            // Rotation AROUND +X by theta (right hand rule) -> Y goes to +Z (down)?
            // We want Y to go UP. So rotate positive X gives +Y -> +Z(back).
            // We want to pitch UP. Nose (-Z) goes to +Y.
            // Rotate around +X by +theta -> Y->Z. Mmm.
            // Basic pitch:
            // global X axis.
            let elevated_direction = elevation_rot * final_direction;
             
            let velocity = elevated_direction * velocity_mag;
            
            let mut entity_cmd = commands.spawn((
                Mesh3d(projectile_mesh.clone()),
                MeshMaterial3d(projectile_material.clone()),
                Transform::from_translation(origin),
                Projectile {
                    velocity,
                    mass,
                    drag_coefficient: drag,
                    diameter,
                    spin,
                    reference_area: std::f32::consts::PI * (diameter / 2.0).powi(2),
                    ..default()
                },
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

            if weapon_state.weapon_type == WeaponType::Flamethrower {
                // Short lived, slows down (high drag), explode on impact or time
                entity_cmd.insert(ProjectileLogic::Timed { fuse: 1.0, elapsed: 0.0 });
                entity_cmd.insert(Payload::Incendiary { 
                    radius: 2.0, 
                    damage_per_second: 10.0,
                    duration: 3.0
                });
                
                // Overwrite drag directly
                entity_cmd.insert(Projectile {
                    velocity,
                    mass: 0.05,
                    drag_coefficient: 2.0,
                    reference_area: 0.05,
                    diameter: 0.1,
                    spin: 0.0,
                    ..default()
                });
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
                "Press SPACE to shoot\nPress 1-5 for weapon types\nCurrent: {}\nBloom: {:.3}\nArgs: Zero: {:.0}m (PgUp/Dn)",
                weapon_state.weapon_type.name(),
                weapon_state.accuracy.current_bloom,
                weapon_state.sight.current_zero
            );
        }
    }
}

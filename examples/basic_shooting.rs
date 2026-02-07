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
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WeaponType {
    Pistol,
    Rifle,
    Sniper,
    SMG,
    Shotgun,
}

impl WeaponType {
    fn name(&self) -> &'static str {
        match self {
            Self::Pistol => "Pistol",
            Self::Rifle => "Rifle",
            Self::Sniper => "Sniper",
            Self::SMG => "SMG",
            Self::Shotgun => "Shotgun",
        }
    }

    fn muzzle_velocity(&self) -> f32 {
        match self {
            Self::Pistol => 350.0,
            Self::Rifle => 900.0,
            Self::Sniper => 1200.0,
            Self::SMG => 400.0,
            Self::Shotgun => 350.0,
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
        }
    }
}

fn handle_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut weapon_state: ResMut<WeaponState>,
    shooter: Query<&Transform, With<ShooterMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Weapon selection
    if keyboard.just_pressed(KeyCode::Digit1) {
        weapon_state.weapon_type = WeaponType::Pistol;
        weapon_state.accuracy = WeaponType::Pistol.accuracy();
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        weapon_state.weapon_type = WeaponType::Rifle;
        weapon_state.accuracy = WeaponType::Rifle.accuracy();
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        weapon_state.weapon_type = WeaponType::Sniper;
        weapon_state.accuracy = WeaponType::Sniper.accuracy();
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        weapon_state.accuracy = WeaponType::SMG.accuracy();
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        weapon_state.weapon_type = WeaponType::Shotgun;
        weapon_state.accuracy = WeaponType::Shotgun.accuracy();
    }

    // Fire
    if keyboard.just_pressed(KeyCode::Space) {
        let Ok(shooter_transform) = shooter.single() else {
            return;
        };

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
            _ => (1, 25.0),
        };

        for i in 0..projectile_count {
            let seed = rand::random::<u64>().wrapping_add(i as u64);
            let final_direction = accuracy::apply_spread_to_direction(direction, spread_angle, seed);

            let velocity = final_direction * weapon_state.weapon_type.muzzle_velocity();

            commands.spawn((
                Mesh3d(projectile_mesh.clone()),
                MeshMaterial3d(projectile_material.clone()),
                Transform::from_translation(origin),
                Projectile::new(velocity),
                Payload::Kinetic { damage },
                ProjectileLogic::Impact,
            ));
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

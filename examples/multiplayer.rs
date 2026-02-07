//! Multiplayer example demonstrating client-server ballistics.
//!
//! This example requires the `netcode` feature flag.
//!
//! Run with: `cargo run --example multiplayer --features netcode`

use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;

#[cfg(feature = "netcode")]
use bevy::ecs::message::MessageWriter;
#[cfg(feature = "netcode")]
use bevy_bullet_dynamics::network::{BallisticsNetPlugin, FireCommand};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(BallisticsPluginGroup);

    #[cfg(feature = "netcode")]
    app.add_plugins(BallisticsNetPlugin);

    app.add_systems(Startup, setup)
        .add_systems(Update, handle_input)
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
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        })),
    ));

    // Local player marker
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 5.0),
        LocalPlayer { id: 1 },
    ));

    // UI
    commands.spawn((
        Text::new("Multiplayer Example\n\nPress SPACE to fire\n(Simulated client-server)"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

#[derive(Component)]
struct LocalPlayer {
    id: u64,
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Query<(&Transform, &LocalPlayer)>,
    #[cfg(feature = "netcode")] mut fire_commands: MessageWriter<FireCommand>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let Ok((transform, player)) = player.single() else {
            return;
        };

        let origin = transform.translation + Vec3::Y * 0.5;
        let direction = Vec3::NEG_Z;

        #[cfg(feature = "netcode")]
        {
            // Send fire command to server (simulated)
            fire_commands.write(FireCommand {
                player_id: player.id,
                origin,
                direction,
                weapon_type: 0,
                spread_seed: rand::random(),
                timestamp: time.elapsed_secs_f64(),
            });

            println!(
                "Fire command sent: player={}, origin={:?}",
                player.id, origin
            );
        }

        #[cfg(not(feature = "netcode"))]
        {
            println!("Netcode feature not enabled. Run with --features netcode");
        }
    }
}

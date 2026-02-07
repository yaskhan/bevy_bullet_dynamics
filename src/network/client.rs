use bevy::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::netcode::NetcodeClientPlugin;
use crate::network::protocol::{Channel, PlayerInput, ServerMessage};
use crate::components::*;

pub struct BallisticsClientPlugin;

impl Plugin for BallisticsClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);
        app.add_plugins(NetcodeClientPlugin);
        
        app.add_systems(Update, (
            client_sync_system,
            client_input_system,
        ));
    }
}

fn client_sync_system(
    mut client: ResMut<RenetClient>,
    mut commands: Commands,
    // query needed for reconciliation
) {
    if !client.is_connected() { return; }
        
    // Receive messages
    while let Some(message) = client.receive_message(Channel::Unreliable.id()) {
        if let Ok(server_msg) = bincode::deserialize::<ServerMessage>(&message) {
            match server_msg {
                ServerMessage::Snapshot(_snapshot) => {
                     // Simple snapshot application (snap to pos)
                     // In real CSP, we would blend or correct prediction error.
                }
                ServerMessage::SpawnProjectile { id, owner_fmt: _, pos, vel, weapon_type: _ } => {
                    // Spawn authoritative projectile
                    // Ideally we check if we already have a predicted one matching this?
                     commands.spawn((
                        Projectile::new(vel),
                        Transform::from_translation(pos),
                        NetworkId(id),
                        Authoritative,
                    ));
                    println!("Spawned Auth Projectile {}", id);
                }
                _ => {}
            }
        }
    }
}

fn client_input_system(
    mut client: ResMut<RenetClient>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if !client.is_connected() { return; }

    let shoot = keyboard.just_pressed(KeyCode::Space);
    
    // Construct input
    let input = PlayerInput {
        move_dir: Vec2::ZERO,
        look_dir: Vec3::Z, // simplified
        shoot,
        switch_weapon: None,
    };

    // Send to server
    let message = bincode::serialize(&input).unwrap();
    client.send_message(Channel::Unreliable.id(), message);

    // CSP: If shooting, spawn local projectile VISUAL ONLY (Predicted)
    if shoot {
         commands.spawn((
            Projectile::new(Vec3::Z * 900.0),
            Transform::from_translation(Vec3::Y * 2.0),
            Predicted,
            // Add visual component but maybe no collider?
        ));
        println!("Spawned Predicted Projectile");
    }
}

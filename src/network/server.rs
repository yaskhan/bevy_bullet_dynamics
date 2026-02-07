use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use bevy_renet2::prelude::*;
use bevy_renet2::netcode::NetcodeServerPlugin;
use crate::network::protocol::{Channel, ServerMessage, GameStateSnapshot};
use crate::components::*;

pub struct BallisticsServerPlugin;

impl Plugin for BallisticsServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetServerPlugin);
        app.add_plugins(NetcodeServerPlugin);
        
        app.add_systems(Update, (
            server_update_system,
            server_network_sync,
            server_process_input,
        ));
    }
}

fn server_update_system(
    mut server_events: MessageReader<ServerEvent>,
    _commands: Commands,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Client {} connected", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {} disconnected: {}", client_id, reason);
            }
        }
    }
}

fn server_process_input(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, Channel::Unreliable.id()) {
            if let Ok(input) = bincode::deserialize::<crate::network::protocol::PlayerInput>(&message) {
                 if input.shoot {
                     // ID generation (simplified)
                     let id = 1000 + client_id * 10000; 
                     
                     let pos = Vec3::Y * 2.0; 
                     let vel = input.look_dir * 900.0;
                     
                     commands.spawn((
                         Projectile::new(vel),
                         Transform::from_translation(pos),
                         NetworkId(id),
                         Authoritative,
                     ));

                     // Broadcast spawn event
                     let msg = ServerMessage::SpawnProjectile {
                         id,
                         owner_fmt: client_id,
                         pos,
                         vel,
                         weapon_type: 0,
                     };
                     let bytes = bincode::serialize(&msg).unwrap();
                     server.broadcast_message(Channel::Unreliable.id(), bytes);
                 }
            }
        }
    }
}

fn server_network_sync(
    mut server: ResMut<RenetServer>,
    query: Query<(&Transform, &Projectile, &NetworkId)>,
) {
    let mut projectiles = Vec::new();
    for (t, p, net_id) in query.iter() {
        projectiles.push(crate::network::protocol::ProjectileState {
            id: net_id.0,
            position: t.translation,
            velocity: p.velocity,
        });
    }

    let snapshot = GameStateSnapshot {
        sequence: 0, 
        players: vec![],
        projectiles,
    };
    
    let message = bincode::serialize(&ServerMessage::Snapshot(snapshot)).unwrap();
    server.broadcast_message(Channel::Unreliable.id(), message);
}

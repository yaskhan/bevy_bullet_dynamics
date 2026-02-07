use bevy::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::renet::transport::NetcodeServerTransport;
use crate::network::protocol::{Channel, PROTOCOL_ID, ServerMessage, GameStateSnapshot};

pub struct BallisticsServerPlugin;

impl Plugin for BallisticsServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetServerPlugin);
        app.add_plugins(NetcodeServerPlugin);
        
        app.add_systems(Update, (
            server_update_system,
            server_network_sync,
        ));
    }
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Client {} connected", client_id);
                // Spawn player logic would go here
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {} disconnected: {}", client_id, reason);
            }
        }
    }
}

fn server_network_sync(
    mut server: ResMut<RenetServer>,
    // transform_query: Query<(&Transform), With<Player>>, 
) {
    // Determine snapshot
    // let snapshot = GameStateSnapshot { ... };
    // let message = bincode::serialize(&ServerMessage::Snapshot(snapshot)).unwrap();
    // server.broadcast_message(Channel::Unreliable.id(), message);
}

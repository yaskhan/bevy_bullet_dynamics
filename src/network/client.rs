use bevy::prelude::*;
use bevy_renet2::prelude::*;
use crate::network::protocol::{Channel, PlayerInput, ServerMessage};

pub struct BallisticsClientPlugin;

impl Plugin for BallisticsClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);
        app.add_plugins(NetcodeClientPlugin);
        
        app.add_systems(Update, (
            client_sync_system,
        ));
    }
}

fn client_sync_system(
    mut client: ResMut<RenetClient>,
) {
    if client.is_connected() {
        // Send inputs
        // let input_message = bincode::serialize(&PlayerInput { ... }).unwrap();
        // client.send_message(Channel::Unreliable.id(), input_message);
        
        // Receive messages
        while let Some(message) = client.receive_message(Channel::Unreliable.id()) {
            if let Ok(server_msg) = bincode::deserialize::<ServerMessage>(&message) {
                match server_msg {
                    ServerMessage::Snapshot(snapshot) => {
                        // Apply snapshot
                    }
                    _ => {}
                }
            }
        }
    }
}

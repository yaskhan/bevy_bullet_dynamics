use bevy::prelude::*;

pub mod protocol;
pub mod server;
pub mod client;

pub struct BallisticsNetworkPlugin;

impl Plugin for BallisticsNetworkPlugin {
    fn build(&self, app: &mut App) {
        // Shared configuration
        app.add_plugins(server::BallisticsServerPlugin);
        app.add_plugins(client::BallisticsClientPlugin);
    }
}

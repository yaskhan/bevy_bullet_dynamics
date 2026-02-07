//! Network module for client-server multiplayer support.
//!
//! This module is only available with the `netcode` feature flag.

use bevy::prelude::*;

use crate::components::{NetProjectile, Projectile};
use crate::events::{FireEvent, HitEvent};

/// Network ballistics plugin for multiplayer synchronization.
/// 
/// This plugin handles network synchronization for ballistics in multiplayer games,
/// including client prediction, server authority, and hit confirmation mechanisms.
/// 
/// # Systems
/// - `process_fire_commands` - Processes fire commands from clients
/// - `reconcile_server_hits` - Reconciles server hit confirmations with client predictions
/// - `cleanup_orphaned_projectiles` - Removes projectiles that have lost network connection
/// 
/// # Events
/// - `FireCommand` - Client-to-server fire commands
/// - `ServerHitConfirm` - Server-to-client hit confirmations
pub struct BallisticsNetPlugin;

impl Plugin for BallisticsNetPlugin {
    /// Builds the network ballistics plugin by adding events and systems.
    /// 
    /// This method registers network-related events and schedules the systems
    /// responsible for handling multiplayer ballistics synchronization.
    /// 
    /// # Arguments
    /// * `app` - Mutable reference to the Bevy App
    fn build(&self, app: &mut App) {
        app.add_event::<FireCommand>()
            .add_event::<ServerHitConfirm>()
            .add_systems(
                FixedUpdate,
                (
                    process_fire_commands,
                    reconcile_server_hits,
                    cleanup_orphaned_projectiles,
                ),
            );
    }
}

/// Client-to-server fire command.
/// 
/// This event represents a fire command sent from a client to the server,
/// containing all necessary information to simulate the shot on the server.
/// 
/// # Fields
/// * `player_id` - Unique identifier of the player who fired
/// * `origin` - World-space position where the shot originated
/// * `direction` - Normalized direction vector of the shot
/// * `weapon_type` - Index identifying the weapon type for preset lookup
/// * `spread_seed` - Random seed for deterministic spread calculation
/// * `timestamp` - Client timestamp for anti-cheat validation
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::network::FireCommand;
/// 
/// let fire_cmd = FireCommand {
///     player_id: 12345,
///     origin: Vec3::ZERO,
///     direction: Vec3::Z,
///     weapon_type: 0,
///     spread_seed: 9876543210,
///     timestamp: 123456.789,
/// };
/// ```
#[derive(Event, Clone)]
pub struct FireCommand {
    /// Player ID who fired
    pub player_id: u64,
    /// Origin position
    pub origin: Vec3,
    /// Fire direction
    pub direction: Vec3,
    /// Weapon type index
    pub weapon_type: usize,
    /// Random seed for spread
    pub spread_seed: u64,
    /// Client timestamp
    pub timestamp: f64,
}

/// Server-to-client hit confirmation.
/// 
/// This event confirms a hit that occurred on the server, allowing clients
/// to reconcile their predictions with the authoritative server state.
/// 
/// # Fields
/// * `projectile_id` - Network ID of the projectile that hit
/// * `target_player_id` - Optional ID of the player that was hit (if applicable)
/// * `impact_pos` - World-space position of the impact
/// * `damage` - Amount of damage dealt by the hit
/// * `server_timestamp` - Server timestamp of the hit event
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::network::ServerHitConfirm;
/// 
/// let hit_confirm = ServerHitConfirm {
///     projectile_id: 98765,
///     target_player_id: Some(54321),
///     impact_pos: Vec3::new(10.0, 0.0, 5.0),
///     damage: 25.0,
///     server_timestamp: 123456.889,
/// };
/// ```
#[derive(Event, Clone)]
pub struct ServerHitConfirm {
    /// Projectile network ID
    pub projectile_id: u64,
    /// Hit target player ID (if player)
    pub target_player_id: Option<u64>,
    /// Impact position
    pub impact_pos: Vec3,
    /// Damage dealt
    pub damage: f32,
    /// Server timestamp
    pub server_timestamp: f64,
}

/// Process fire commands from clients (server-side).
fn process_fire_commands(
    mut commands: Commands,
    mut fire_commands: EventReader<FireCommand>,
    time: Res<Time>,
) {
    for cmd in fire_commands.read() {
        // Validate timestamp (anti-cheat)
        let current_time = time.elapsed_secs_f64();
        let time_diff = (current_time - cmd.timestamp).abs();

        // Allow some network latency tolerance
        if time_diff > 0.5 {
            // Reject suspicious fire command
            continue;
        }

        // Spawn server-authoritative projectile
        // This would use weapon presets to get projectile parameters
        let velocity = cmd.direction.normalize() * 400.0; // Default velocity

        commands.spawn((
            Transform::from_translation(cmd.origin),
            Projectile::new(velocity),
            NetProjectile {
                owner_id: cmd.player_id,
                timestamp: cmd.timestamp,
                spread_seed: cmd.spread_seed,
            },
        ));
    }
}

/// Reconcile server hit confirmations with client predictions.
fn reconcile_server_hits(
    mut server_hits: EventReader<ServerHitConfirm>,
    // Client-side prediction reconciliation would go here
) {
    for hit in server_hits.read() {
        // Update client-side state based on server confirmation
        // This handles cases where client prediction was wrong
    }
}

/// Cleanup projectiles that have lost their network connection.
fn cleanup_orphaned_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    projectiles: Query<(Entity, &NetProjectile)>,
) {
    let current_time = time.elapsed_secs_f64();

    for (entity, net_proj) in projectiles.iter() {
        // Remove projectiles older than 10 seconds
        if current_time - net_proj.timestamp > 10.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Client-side prediction for responsive gameplay.
pub mod prediction {
    use super::*;

    /// Marker for client-predicted projectiles.
    /// 
    /// This component marks projectiles that were spawned on the client for
    /// immediate visual feedback, before server confirmation arrives.
    /// 
    /// # Fields
    /// * `sequence` - Sequence number used to match with server-confirmed projectiles
    /// * `local_time` - Local time when the projectile was spawned (for reconciliation)
    /// 
    /// # Example
    /// ```
    /// use bevy_bullet_dynamics::network::prediction::PredictedProjectile;
    /// 
    /// let pred_proj = PredictedProjectile {
    ///     sequence: 12345,
    ///     local_time: 123456.789,
    /// };
    /// ```
    #[derive(Component)]
    pub struct PredictedProjectile {
        /// Sequence number for reconciliation
        pub sequence: u64,
        /// Local spawn time
        pub local_time: f64,
    }

    /// Spawn a client-predicted projectile.
    /// 
    /// Creates a projectile on the client for immediate visual feedback.
    /// This projectile will later be reconciled with the server-confirmed projectile.
    /// 
    /// # Arguments
    /// * `commands` - Bevy Commands for spawning the entity
    /// * `origin` - World-space position where the projectile should spawn
    /// * `direction` - Direction vector for the projectile's initial velocity
    /// * `velocity` - Magnitude of the initial velocity in meters per second
    /// * `sequence` - Sequence number for matching with server confirmation
    /// * `local_time` - Local time of spawn for reconciliation purposes
    /// 
    /// # Returns
    /// The Entity ID of the newly spawned projectile
    pub fn spawn_predicted_projectile(
        commands: &mut Commands,
        origin: Vec3,
        direction: Vec3,
        velocity: f32,
        sequence: u64,
        local_time: f64,
    ) -> Entity {
        commands
            .spawn((
                Transform::from_translation(origin),
                Projectile::new(direction.normalize() * velocity),
                PredictedProjectile {
                    sequence,
                    local_time,
                },
            ))
            .id()
    }

    /// Reconcile prediction with server state.
    /// 
    /// Adjusts the client-predicted projectile's position and velocity to match
    /// the server-confirmed state, using smooth interpolation to minimize visual jumps.
    /// 
    /// # Arguments
    /// * `commands` - Bevy Commands for modifying the entity
    /// * `predicted_entity` - Entity ID of the client-predicted projectile
    /// * `server_pos` - Server-confirmed position of the projectile
    /// * `server_vel` - Server-confirmed velocity of the projectile
    /// * `projectiles` - Query for accessing predicted projectile components
    pub fn reconcile_prediction(
        commands: &mut Commands,
        predicted_entity: Entity,
        server_pos: Vec3,
        server_vel: Vec3,
        projectiles: &mut Query<(&mut Transform, &mut Projectile), With<PredictedProjectile>>,
    ) {
        if let Ok((mut transform, mut projectile)) = projectiles.get_mut(predicted_entity) {
            // Smooth correction towards server state
            let correction_factor = 0.1;
            transform.translation = transform.translation.lerp(server_pos, correction_factor);
            projectile.velocity = projectile.velocity.lerp(server_vel, correction_factor);
        }
    }
}

/// Network message serialization for renet2.
pub mod messages {
    use super::*;

    /// All possible network messages.
    /// 
    /// This enum represents all the different types of messages that can be
    /// sent between clients and servers in the ballistics system.
    /// 
    /// # Variants
    /// * `Fire` - Message containing a fire command from a client
    /// * `Hit` - Message containing a hit confirmation from the server
    /// * `ProjectileSync` - Message containing projectile synchronization data
    /// 
    /// # Example
    /// ```
    /// use bevy_bullet_dynamics::network::messages::BallisticsMessage;
    /// use bevy_bullet_dynamics::network::{FireCommand, ServerHitConfirm};
    /// 
    /// let fire_msg = BallisticsMessage::Fire(FireCommand {
    ///     player_id: 12345,
    ///     origin: bevy::prelude::Vec3::ZERO,
    ///     direction: bevy::prelude::Vec3::Z,
    ///     weapon_type: 0,
    ///     spread_seed: 9876543210,
    ///     timestamp: 123456.789,
    /// });
    /// ```
    #[derive(Clone)]
    pub enum BallisticsMessage {
        Fire(FireCommand),
        Hit(ServerHitConfirm),
        ProjectileSync(ProjectileSyncData),
    }

    /// Projectile position sync data.
    /// 
    /// Contains the necessary information to synchronize a projectile's state
    /// between client and server, including position, velocity, and timing.
    /// 
    /// # Fields
    /// * `id` - Unique identifier of the projectile
    /// * `position` - Current world-space position of the projectile
    /// * `velocity` - Current velocity vector of the projectile
    /// * `server_time` - Server timestamp when this data was captured
    /// 
    /// # Example
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_bullet_dynamics::network::messages::ProjectileSyncData;
    /// 
    /// let sync_data = ProjectileSyncData {
    ///     id: 12345,
    ///     position: Vec3::new(10.0, 5.0, 0.0),
    ///     velocity: Vec3::new(100.0, -5.0, 0.0),
    ///     server_time: 123456.789,
    /// };
    /// ```
    #[derive(Clone)]
    pub struct ProjectileSyncData {
        pub id: u64,
        pub position: Vec3,
        pub velocity: Vec3,
        pub server_time: f64,
    }

    // Serialization would use bincode or similar
    // Example with renet2:
    // impl BallisticsMessage {
    //     pub fn serialize(&self) -> Vec<u8> { ... }
    //     pub fn deserialize(data: &[u8]) -> Option<Self> { ... }
    // }
}

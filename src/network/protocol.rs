use bevy::prelude::*;
use renet2::{ChannelConfig, SendType};
use std::time::Duration;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Reliable,
    Unreliable,
}

impl From<Channel> for u8 {
    fn from(channel: Channel) -> u8 {
        match channel {
            Channel::Reliable => 0,
            Channel::Unreliable => 1,
        }
    }
}

impl Channel {
    pub fn id(&self) -> u8 {
        (*self).into()
    }

    pub fn config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Channel::Reliable.id(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(300),
                },
            },
            ChannelConfig {
                channel_id: Channel::Unreliable.id(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub struct PlayerInput {
    pub move_dir: Vec2,
    pub look_dir: Vec3, // Forward vector
    pub shoot: bool,
    pub switch_weapon: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
    Snapshot(GameStateSnapshot),
    SpawnProjectile {
        id: u64,
        owner_fmt: u64,
        pos: Vec3,
        vel: Vec3,
        weapon_type: u8,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub sequence: u32,
    pub players: Vec<PlayerState>,
    pub projectiles: Vec<ProjectileState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerState {
    pub id: u64,
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectileState {
    pub id: u64, // Entity bits or unique ID
    pub position: Vec3,
    pub velocity: Vec3,
}

//! Events for the ballistics system.
//! 
//! Note: In Bevy 0.18, buffered events use the `Message` trait instead of `Event`.

use bevy::prelude::*;
use bevy::ecs::message::Message;

/// Event fired when a weapon is discharged.
#[derive(Message, Debug, Reflect, Clone)]
#[reflect(Debug)]
pub struct FireEvent {
    pub origin: Vec3,
    pub direction: Vec3,
    pub muzzle_velocity: f32,
    pub shooter: Option<Entity>,
    pub spread_seed: u64,
    pub weapon_type: usize,
    pub timestamp: f64,
    pub projectile_count: u32,
    pub spread_angle: f32,
}

impl Default for FireEvent {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            direction: Vec3::NEG_Z,
            muzzle_velocity: 400.0,
            shooter: None,
            spread_seed: 0,
            weapon_type: 0,
            timestamp: 0.0,
            projectile_count: 1,
            spread_angle: 0.0,
        }
    }
}

impl FireEvent {
    pub fn new(origin: Vec3, direction: Vec3, muzzle_velocity: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            muzzle_velocity,
            ..Default::default()
        }
    }

    pub fn with_shooter(mut self, shooter: Entity) -> Self {
        self.shooter = Some(shooter);
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.spread_seed = seed;
        self
    }

    pub fn with_projectile_count(mut self, count: u32) -> Self {
        self.projectile_count = count;
        self
    }

    pub fn with_spread_angle(mut self, angle: f32) -> Self {
        self.spread_angle = angle;
        self
    }
}

/// Event fired when a projectile hits something.
#[derive(Message, Debug, Reflect, Clone)]
#[reflect(Debug)]
pub struct HitEvent {
    pub projectile: Entity,
    pub target: Entity,
    pub impact_point: Vec3,
    pub normal: Vec3,
    pub velocity: Vec3,
    pub damage: f32,
    pub penetrated: bool,
    pub ricocheted: bool,
}

/// Event fired when an explosion occurs.
#[derive(Message, Debug, Reflect, Clone)]
#[reflect(Debug)]
pub struct ExplosionEvent {
    pub center: Vec3,
    pub radius: f32,
    pub damage: f32,
    pub falloff: f32,
    pub explosion_type: ExplosionType,
    pub source: Option<Entity>,
}

/// Types of explosions.
#[derive(Debug, Reflect, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Debug)]
pub enum ExplosionType {
    #[default]
    HighExplosive,
    Incendiary,
    Flash,
    Smoke,
    Fragmentation,
    Concussion,
    EMP,
}

/// Event for projectile penetration.
#[derive(Message, Debug, Reflect, Clone)]
#[reflect(Debug)]
pub struct PenetrationEvent {
    pub projectile: Entity,
    pub entry_point: Vec3,
    pub exit_point: Vec3,
    pub target: Entity,
    pub remaining_power: f32,
}

/// Event for projectile ricochet.
#[derive(Message, Debug, Reflect, Clone)]
#[reflect(Debug)]
pub struct RicochetEvent {
    pub projectile: Entity,
    pub impact_point: Vec3,
    pub new_direction: Vec3,
    pub new_speed: f32,
    pub surface: Entity,
}

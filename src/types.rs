//! Common types and enums for the ballistics system.

use bevy::prelude::*;

/// Physics model for projectile simulation.
/// 
/// Defines the integration method used for simulating projectile physics.
/// Different variants offer trade-offs between accuracy and performance.
/// 
/// # Variants
/// * `Euler` - Simple Euler integration (faster but less accurate)
/// * `RK4` - Runge-Kutta 4th order integration (more accurate, slightly slower)
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::types::PhysicsModel;
/// 
/// let model = PhysicsModel::RK4; // For high accuracy
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum PhysicsModel {
    /// Simple Euler integration (fast, less accurate)
    Euler,
    /// Runge-Kutta 4th order (accurate, slightly slower)
    #[default]
    RK4,
}

/// Weapon category for behavior customization.
/// 
/// Categorizes weapons to allow for different behavior patterns and handling.
/// Different categories may have different mechanics applied to them.
/// 
/// # Variants
/// * `Firearm` - Standard bullet-based weapons (rifles, pistols, etc.)
/// * `Projectile` - Weapons with high drag and slow projectiles (bows, crossbows)
/// * `Throwable` - Thrown weapons with parabolic arcs (grenades, etc.)
/// * `Explosive` - Guided or unguided rocket/missile weapons
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::types::WeaponCategory;
/// 
/// let category = WeaponCategory::Firearm;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum WeaponCategory {
    #[default]
    /// Standard bullet-based weapons
    Firearm,
    /// Bow, crossbow (high drag, slow)
    Projectile,
    /// Grenades (parabolic arc, timed)
    Throwable,
    /// Rockets, missiles (guided or unguided)
    Explosive,
}

/// Hit result from raycasting.
/// 
/// Contains information about a successful raycast hit, including the hit entity,
/// world position, surface normal, and distance from origin.
/// 
/// # Fields
/// * `entity` - The entity that was hit by the raycast
/// * `point` - World-space coordinates of the hit point
/// * `normal` - Surface normal vector at the hit point
/// * `distance` - Distance from the ray origin to the hit point
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::types::HitResult;
/// 
/// let hit_result = HitResult {
///     entity: Entity::PLACEHOLDER,
///     point: Vec3::ZERO,
///     normal: Vec3::Y,
///     distance: 10.0,
/// };
/// ```
#[derive(Clone)]
pub struct HitResult {
    /// Hit entity
    pub entity: Entity,
    /// World-space hit point
    pub point: Vec3,
    /// Surface normal
    pub normal: Vec3,
    /// Distance from ray origin
    pub distance: f32,
}

/// Spatial query abstraction for 2D/3D support.
/// 
/// This trait provides a unified interface for performing spatial queries
/// regardless of whether the simulation is 2D or 3D, allowing for shared
/// code between different dimensional implementations.
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::types::{SpatialQueryExt, HitResult};
/// 
/// // This would be implemented by your physics engine
/// // trait implementation
/// ```
pub trait SpatialQueryExt {
    /// Cast a ray and return the first hit.
    /// 
    /// Performs a raycast from the given origin in the specified direction
    /// up to the maximum distance, optionally filtering out a specific entity.
    /// 
    /// # Arguments
    /// * `origin` - Starting point of the ray in world space
    /// * `direction` - Normalized direction vector of the ray
    /// * `max_dist` - Maximum distance to cast the ray
    /// * `filter` - Optional entity to exclude from the raycast
    /// 
    /// # Returns
    /// An Option containing the HitResult if a hit occurred, or None otherwise
    fn cast_projectile_ray(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_dist: f32,
        filter: Option<Entity>,
    ) -> Option<HitResult>;
}

/// Enum for projectile state tracking.
/// 
/// Tracks the current state of a projectile during its lifecycle,
/// allowing for different behaviors based on the projectile's condition.
/// 
/// # Variants
/// * `InFlight` - Projectile is currently flying through the air
/// * `Stuck` - Projectile has become stuck in a surface (e.g., arrow in wall)
/// * `Detonating` - Projectile is in the process of detonating (for explosives)
/// * `Despawning` - Projectile is marked for removal from the simulation
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::types::ProjectileState;
/// 
/// let state = ProjectileState::InFlight;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum ProjectileState {
    #[default]
    /// Flying through air
    InFlight,
    /// Stuck in surface
    Stuck,
    /// Detonating (for grenades)
    Detonating,
    /// Marked for despawn
    Despawning,
}

/// Projectile spawn parameters builder.
/// 
/// A builder-style struct for specifying all the parameters needed to spawn
/// a projectile with the correct initial properties.
/// 
/// # Fields
/// * `origin` - World-space position where the projectile should spawn
/// * `direction` - Normalized direction vector for the projectile's initial velocity
/// * `velocity` - Magnitude of the initial velocity in meters per second
/// * `mass` - Mass of the projectile in kilograms
/// * `drag` - Drag coefficient affecting the projectile's flight
/// * `damage` - Base damage that the projectile should deal on impact
/// * `owner` - Optional entity that owns this projectile (for hit detection)
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::types::ProjectileSpawnParams;
/// 
/// let params = ProjectileSpawnParams::new(
///     Vec3::new(0.0, 1.5, 0.0),
///     Vec3::Z,
///     800.0
/// ).with_damage(50.0)
///  .with_owner(Entity::PLACEHOLDER);
/// ```
#[derive(Clone)]
pub struct ProjectileSpawnParams {
    pub origin: Vec3,
    pub direction: Vec3,
    pub velocity: f32,
    pub mass: f32,
    pub drag: f32,
    pub damage: f32,
    pub owner: Option<Entity>,
}

impl Default for ProjectileSpawnParams {
    /// Creates a default ProjectileSpawnParams with reasonable values for a typical rifle bullet.
    /// 
    /// Default values:
    /// - Origin at (0, 0, 0)
    /// - Direction toward negative Z axis
    /// - Velocity of 400 m/s
    /// - Mass of 10g
    /// - Drag coefficient of 0.3
    /// - Damage of 25.0
    /// - No owner specified
    /// 
    /// # Returns
    /// A new ProjectileSpawnParams instance with default values
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            direction: Vec3::NEG_Z,
            velocity: 400.0,
            mass: 0.01,
            drag: 0.3,
            damage: 25.0,
            owner: None,
        }
    }
}

impl ProjectileSpawnParams {
    /// Creates a new ProjectileSpawnParams with the specified origin, direction, and velocity.
    /// 
    /// The direction vector will be normalized automatically.
    /// 
    /// # Arguments
    /// * `origin` - World-space position where the projectile should spawn
    /// * `direction` - Direction vector for the projectile's initial velocity (will be normalized)
    /// * `velocity` - Magnitude of the initial velocity in meters per second
    /// 
    /// # Returns
    /// A new ProjectileSpawnParams instance with the specified parameters
    pub fn new(origin: Vec3, direction: Vec3, velocity: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            velocity,
            ..Default::default()
        }
    }

    /// Sets the mass of the projectile.
    /// 
    /// # Arguments
    /// * `mass` - Mass of the projectile in kilograms
    /// 
    /// # Returns
    /// The modified ProjectileSpawnParams instance for method chaining
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Sets the damage of the projectile.
    /// 
    /// # Arguments
    /// * `damage` - Base damage that the projectile should deal on impact
    /// 
    /// # Returns
    /// The modified ProjectileSpawnParams instance for method chaining
    pub fn with_damage(mut self, damage: f32) -> Self {
        self.damage = damage;
        self
    }

    /// Sets the owner entity of the projectile.
    /// 
    /// # Arguments
    /// * `owner` - Entity that owns this projectile (for hit detection)
    /// 
    /// # Returns
    /// The modified ProjectileSpawnParams instance for method chaining
    pub fn with_owner(mut self, owner: Entity) -> Self {
        self.owner = Some(owner);
        self
    }
}

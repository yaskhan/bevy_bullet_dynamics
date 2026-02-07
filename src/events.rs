//! Events for the ballistics system.
//! 
//! Note: In Bevy 0.18, buffered events use the `Message` trait instead of `Event`.

use bevy::prelude::*;
use bevy::ecs::message::Message;

/// Event fired when a weapon is discharged.
/// 
/// This event is sent when a weapon fires, containing all the information needed
/// to spawn and initialize a projectile with the correct properties.
/// 
/// # Fields
/// * `origin` - World-space position where the shot originated
/// * `direction` - Normalized direction vector of the shot
/// * `muzzle_velocity` - Initial velocity of the projectile in meters per second
/// * `shooter` - Optional entity that fired the weapon (for ownership tracking)
/// * `spread_seed` - Random seed for deterministic spread calculation (networking)
/// * `weapon_type` - Index identifying the weapon type for preset lookup
/// * `timestamp` - Server timestamp for client-server reconciliation
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::events::FireEvent;
/// 
/// let fire_event = FireEvent::new(
///     Vec3::new(0.0, 1.5, 0.0),
///     Vec3::Z,
///     800.0
/// ).with_shooter(Entity::PLACEHOLDER);
/// ```
#[derive(Message, Clone)]
pub struct FireEvent {
    /// Origin point of the shot
    pub origin: Vec3,
    /// Direction of the shot (normalized)
    pub direction: Vec3,
    /// Initial velocity (m/s)
    pub muzzle_velocity: f32,
    /// Shooter entity (for ownership tracking)
    pub shooter: Option<Entity>,
    /// Random seed for deterministic spread (for networking)
    pub spread_seed: u64,
    /// Weapon type index (for preset lookup)
    pub weapon_type: usize,
    /// Server timestamp (for client-server reconciliation)
    pub timestamp: f64,
    /// Number of projectiles to spawn (e.g., for shotguns)
    pub projectile_count: u32,
    /// Spread angle for multiple projectiles (in radians)
    pub spread_angle: f32,
}

impl Default for FireEvent {
    /// Creates a default FireEvent with reasonable values for a typical rifle shot.
    /// 
    /// Default values:
    /// - Origin at (0, 0, 0)
    /// - Direction toward negative Z axis
    /// - Muzzle velocity of 400 m/s
    /// - No shooter specified
    /// - Seed of 0
    /// - Weapon type 0 (default)
    /// - Timestamp of 0.0
    /// 
    /// # Returns
    /// A new FireEvent instance with default values
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
    /// Creates a new FireEvent with the specified origin, direction, and muzzle velocity.
    /// 
    /// The direction vector will be normalized automatically.
    /// 
    /// # Arguments
    /// * `origin` - World-space position where the shot originated
    /// * `direction` - Direction vector of the shot (will be normalized)
    /// * `muzzle_velocity` - Initial velocity of the projectile in m/s
    /// 
    /// # Returns
    /// A new FireEvent instance with the specified parameters
    pub fn new(origin: Vec3, direction: Vec3, muzzle_velocity: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            muzzle_velocity,
            projectile_count: 1,
            spread_angle: 0.0,
            ..Default::default()
        }
    }

    /// Sets the shooter entity for ownership tracking.
    /// 
    /// # Arguments
    /// * `shooter` - Entity that fired the weapon
    /// 
    /// # Returns
    /// The modified FireEvent instance for method chaining
    pub fn with_shooter(mut self, shooter: Entity) -> Self {
        self.shooter = Some(shooter);
        self
    }

    /// Sets the random seed for deterministic spread calculation.
    /// 
    /// # Arguments
    /// * `seed` - Random seed value
    /// 
    /// # Returns
    /// The modified FireEvent instance for method chaining
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.spread_seed = seed;
        self
    }

    /// Sets the number of projectiles to spawn (e.g., for shotguns).
    pub fn with_projectile_count(mut self, count: u32) -> Self {
        self.projectile_count = count;
        self
    }

    /// Sets the spread angle for multiple projectiles.
    pub fn with_spread_angle(mut self, angle: f32) -> Self {
        self.spread_angle = angle;
        self
    }
}

/// Event fired when a projectile hits something.
/// 
/// This event is sent when a projectile collides with a target, containing all
/// the information needed to process the hit and apply appropriate effects.
/// 
/// # Fields
/// * `projectile` - Entity of the projectile that hit
/// * `target` - Entity that was hit by the projectile
/// * `impact_point` - World-space position where the impact occurred
/// * `normal` - Surface normal vector at the impact point
/// * `velocity` - Velocity vector of the projectile at impact
/// * `damage` - Amount of damage to apply to the target
/// * `penetrated` - Whether the projectile penetrated the surface
/// * `ricocheted` - Whether the projectile ricocheted off the surface
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::events::HitEvent;
/// 
/// let hit_event = HitEvent {
///     projectile: Entity::PLACEHOLDER,
///     target: Entity::PLACEHOLDER,
///     impact_point: Vec3::ZERO,
///     normal: Vec3::Y,
///     velocity: Vec3::Z,
///     damage: 25.0,
///     penetrated: false,
///     ricocheted: false,
/// };
/// ```
#[derive(Message, Clone)]
pub struct HitEvent {
    /// Projectile entity that hit
    pub projectile: Entity,
    /// Hit target entity
    pub target: Entity,
    /// Impact point in world space
    pub impact_point: Vec3,
    /// Surface normal at impact
    pub normal: Vec3,
    /// Projectile velocity at impact
    pub velocity: Vec3,
    /// Damage to apply
    pub damage: f32,
    /// Whether projectile penetrated
    pub penetrated: bool,
    /// Whether projectile ricocheted
    pub ricocheted: bool,
}

/// Event fired when an explosion occurs.
/// 
/// This event is sent when an explosive projectile detonates, containing all
/// the information needed to process the explosion and apply area-of-effect damage.
/// 
/// # Fields
/// * `center` - World-space position at the center of the explosion
/// * `radius` - Maximum radius of the explosion's effect in meters
/// * `damage` - Base damage amount at the center of the explosion
/// * `falloff` - Factor determining how damage decreases with distance from center
/// * `explosion_type` - Type of explosion, affecting its behavior and effects
/// * `source` - Optional entity that caused the explosion (grenade, rocket, etc.)
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::events::{ExplosionEvent, ExplosionType};
/// 
/// let explosion_event = ExplosionEvent {
///     center: Vec3::ZERO,
///     radius: 5.0,
///     damage: 100.0,
///     falloff: 1.5,
///     explosion_type: ExplosionType::HighExplosive,
///     source: Some(Entity::PLACEHOLDER),
/// };
/// ```
#[derive(Message, Clone)]
pub struct ExplosionEvent {
    /// Center of the explosion
    pub center: Vec3,
    /// Maximum radius of effect
    pub radius: f32,
    /// Base damage at center
    pub damage: f32,
    /// Damage falloff factor
    pub falloff: f32,
    /// Explosion type
    pub explosion_type: ExplosionType,
    /// Source entity (grenade, rocket, etc.)
    pub source: Option<Entity>,
}

/// Types of explosions.
/// 
/// Defines different categories of explosions with unique behaviors and effects.
/// 
/// # Variants
/// * `HighExplosive` - Standard high-explosive damage with radius falloff
/// * `Incendiary` - Creates a burning area that deals damage over time
/// * `Flash` - Creates visual impairment effects (flashbangs)
/// * `Smoke` - Creates an obscuring smoke cloud
/// * `Fragmentation` - Splits into multiple smaller projectiles on detonation
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::events::ExplosionType;
/// 
/// let explosion_type = ExplosionType::HighExplosive;
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExplosionType {
    /// Standard HE explosion
    HighExplosive,
    /// Incendiary (creates fire zone)
    Incendiary,
    /// Flashbang (blinds players)
    Flash,
    /// Smoke (creates obscuring cloud)
    Smoke,
    /// Fragmentation (spawns additional projectiles)
    Fragmentation,
    /// Concussion (high knockback, low damage)
    Concussion,
    /// EMP (disables electronics, no physical damage)
    EMP,
}

/// Event for projectile penetration.
/// 
/// This event is sent when a projectile successfully penetrates a surface,
/// containing information about the penetration event for further processing.
/// 
/// # Fields
/// * `projectile` - Entity of the projectile that penetrated
/// * `entry_point` - World-space position where the projectile entered the surface
/// * `exit_point` - World-space position where the projectile exited the surface
/// * `target` - Entity representing the material that was penetrated
/// * `remaining_power` - Remaining penetration power after passing through the material
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::events::PenetrationEvent;
/// 
/// let penetration_event = PenetrationEvent {
///     projectile: Entity::PLACEHOLDER,
///     entry_point: Vec3::ZERO,
///     exit_point: Vec3::X,
///     target: Entity::PLACEHOLDER,
///     remaining_power: 50.0,
/// };
/// ```
#[derive(Message, Clone)]
pub struct PenetrationEvent {
    /// Projectile entity
    pub projectile: Entity,
    /// Entry point
    pub entry_point: Vec3,
    /// Exit point
    pub exit_point: Vec3,
    /// Material penetrated
    pub target: Entity,
    /// Remaining penetration power
    pub remaining_power: f32,
}

/// Event for projectile ricochet.
/// 
/// This event is sent when a projectile ricochets off a surface,
/// containing information about the ricochet for further processing.
/// 
/// # Fields
/// * `projectile` - Entity of the projectile that ricocheted
/// * `impact_point` - World-space position where the projectile hit the surface
/// * `new_direction` - New direction vector after the ricochet
/// * `new_speed` - Reduced speed of the projectile after ricochet
/// * `surface` - Entity representing the surface that caused the ricochet
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::events::RicochetEvent;
/// 
/// let ricochet_event = RicochetEvent {
///     projectile: Entity::PLACEHOLDER,
///     impact_point: Vec3::ZERO,
///     new_direction: Vec3::Y,
///     new_speed: 200.0,
///     surface: Entity::PLACEHOLDER,
/// };
/// ```
#[derive(Message, Clone)]
pub struct RicochetEvent {
    /// Projectile entity
    pub projectile: Entity,
    /// Impact point
    pub impact_point: Vec3,
    /// New direction after ricochet
    pub new_direction: Vec3,
    /// Speed after ricochet (reduced)
    pub new_speed: f32,
    /// Surface hit
    pub surface: Entity,
}

//! Core components for the ballistics system.

use bevy::prelude::*;

/// Main projectile component with physical properties.
/// 
/// This component represents a physical projectile in the simulation with properties
/// that affect its trajectory and behavior when interacting with the environment.
/// 
/// # Fields
/// * `velocity` - Current velocity vector in meters per second
/// * `mass` - Mass of the projectile in kilograms
/// * `drag_coefficient` - Dimensionless drag coefficient (typically 0.2-0.5 for bullets)
/// * `reference_area` - Cross-sectional reference area in square meters
/// * `penetration_power` - Energy available for penetrating materials (arbitrary units)
/// * `previous_position` - Position in the previous frame for collision detection
/// * `owner` - Optional entity that owns this projectile (for hit detection)
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::components::Projectile;
/// 
/// let projectile = Projectile::new(Vec3::new(100.0, 0.0, 0.0))
///     .with_mass(0.008)
///     .with_drag(0.3)
///     .with_owner(Entity::PLACEHOLDER);
/// ```
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct Projectile {
    /// Current velocity vector (m/s)
    pub velocity: Vec3,
    /// Mass of the projectile (kg)
    pub mass: f32,
    /// Drag coefficient (Cd)
    pub drag_coefficient: f32,
    /// Cross-sectional reference area (m²)
    pub reference_area: f32,
    /// Projectile diameter (meters), used for spin drift
    pub diameter: f32,
    /// Angular velocity (spin) around flight axis (rad/s)
    pub spin: f32,
    /// Penetration power (arbitrary units of energy)
    pub penetration_power: f32,
    /// Previous frame position for collision detection
    pub previous_position: Vec3,
    /// Owner entity (for multiplayer hit detection)
    pub owner: Option<Entity>,
}

impl Projectile {
    /// Creates a new projectile with default physics parameters.
    /// 
    /// This constructor sets up a projectile with typical bullet characteristics:
    /// - 10g mass
    /// - 0.3 drag coefficient
    /// - 0.0001 m² reference area (~1cm² cross-section)
    /// - 100.0 penetration power
    /// 
    /// # Arguments
    /// * `velocity` - Initial velocity vector in meters per second
    /// 
    /// # Returns
    /// A new Projectile instance with the specified velocity and default parameters
    pub fn new(velocity: Vec3) -> Self {
        Self {
            velocity,
            mass: 0.01,           // 10g bullet
            drag_coefficient: 0.3,
            reference_area: 0.0001, // ~1cm² cross-section
            diameter: 0.01,
            spin: 0.0,
            penetration_power: 100.0,
            previous_position: Vec3::ZERO,
            owner: None,
        }
    }

    /// Builder pattern: set mass
    /// 
    /// Sets the mass of the projectile in kilograms.
    /// 
    /// # Arguments
    /// * `mass` - Mass in kilograms
    /// 
    /// # Returns
    /// The modified Projectile instance for method chaining
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Builder pattern: set drag coefficient
    /// 
    /// Sets the drag coefficient of the projectile.
    /// 
    /// # Arguments
    /// * `drag` - Drag coefficient (dimensionless, typically 0.2-0.5 for bullets)
    /// 
    /// # Returns
    /// The modified Projectile instance for method chaining
    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag_coefficient = drag;
        self
    }

    /// Builder pattern: set owner
    /// 
    /// Sets the owner entity of the projectile for hit detection purposes.
    /// 
    /// # Arguments
    /// * `owner` - Entity that owns this projectile
    /// 
    /// # Returns
    /// The modified Projectile instance for method chaining
    pub fn with_owner(mut self, owner: Entity) -> Self {
        self.owner = Some(owner);
        self
    }
}

/// Accuracy component for dynamic spread calculation.
/// 
/// This component tracks the accuracy state of a weapon, including bloom accumulation
/// and various factors that affect shot precision.
/// 
/// # Fields
/// * `current_bloom` - Current accumulated bloom in radians
/// * `base_spread` - Base spread in ideal conditions in radians
/// * `max_spread` - Maximum spread ceiling in radians
/// * `bloom_per_shot` - Bloom increase per shot in radians
/// * `recovery_rate` - Rate at which bloom recovers in radians per second
/// * `movement_penalty` - Multiplier applied when moving
/// * `ads_modifier` - Modifier when aiming down sights (0.2 = 80% reduction)
/// * `airborne_multiplier` - Multiplier when airborne
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::Accuracy;
/// 
/// let mut accuracy = Accuracy::default();
/// accuracy.current_bloom = 0.01; // 1 milliradian bloom
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Accuracy {
    /// Current accumulated bloom (radians)
    pub current_bloom: f32,
    /// Base spread in ideal conditions (radians)
    pub base_spread: f32,
    /// Maximum spread ceiling (radians)
    pub max_spread: f32,
    /// Bloom increase per shot (radians)
    pub bloom_per_shot: f32,
    /// Recovery rate (radians per second)
    pub recovery_rate: f32,
    /// Movement penalty multiplier
    pub movement_penalty: f32,
    /// ADS (aiming down sights) modifier (0.2 = 80% reduction)
    pub ads_modifier: f32,
    /// Airborne penalty multiplier
    pub airborne_multiplier: f32,
}

impl Default for Accuracy {
    /// Creates a default Accuracy instance with reasonable values for a typical rifle.
    /// 
    /// Default values:
    /// - 0.002 rad base spread (~0.1 degrees)
    /// - 0.05 rad max spread (~3 degrees)
    /// - 0.01 rad bloom per shot
    /// - 0.05 rad/s recovery rate
    /// - 2.0x movement penalty
    /// - 0.3x ADS modifier (70% accuracy improvement)
    /// - 3.0x airborne penalty
    /// 
    /// # Returns
    /// A new Accuracy instance with default values
    fn default() -> Self {
        Self {
            current_bloom: 0.0,
            base_spread: 0.002,       // ~0.1 degrees
            max_spread: 0.05,         // ~3 degrees max
            bloom_per_shot: 0.01,
            recovery_rate: 0.05,      // per second
            movement_penalty: 2.0,
            ads_modifier: 0.3,
            airborne_multiplier: 3.0,
        }
    }
}

/// Projectile behavior logic type.
/// 
/// Defines how a projectile behaves when it interacts with the environment.
/// Different variants represent different projectile types with unique behaviors.
/// 
/// # Variants
/// * `Impact` - Standard bullet that despawns or penetrates on impact
/// * `Timed` - Projectile with a fuse that explodes after a set time
/// * `Proximity` - Projectile that explodes when a target comes within range
/// * `Sticky` - Projectile that sticks to surfaces on impact (like arrows)
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::ProjectileLogic;
/// 
/// let timed_logic = ProjectileLogic::Timed {
///     fuse: 3.0,
///     elapsed: 0.0,
/// };
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub enum ProjectileLogic {
    /// Standard bullet: despawns or penetrates on impact
    Impact,
    /// Grenade: explodes after timer expires
    Timed {
        /// Fuse time in seconds
        fuse: f32,
        /// Elapsed time since spawn
        elapsed: f32,
    },
    /// Proximity mine/rocket: explodes when target is in range
    Proximity {
        /// Detection range (meters)
        range: f32,
    },
    /// Hitscan: immediate raycast, no flight time
    Hitscan {
        /// Maximum range (meters)
        range: f32,
    },
    /// Arrow/bolt: sticks on impact
    Sticky,
}

impl Default for ProjectileLogic {
    /// Creates a default ProjectileLogic instance with Impact behavior.
    /// 
    /// # Returns
    /// A new ProjectileLogic::Impact variant
    fn default() -> Self {
        Self::Impact
    }
}

/// Payload type determining what happens when projectile triggers.
/// 
/// Defines the type of damage or effect a projectile delivers upon impact or detonation.
/// Different variants represent different payload types with unique effects.
/// 
/// # Variants
/// * `Kinetic` - Direct damage from projectile impact (bullets, arrows)
/// * `Explosive` - Area damage with radius falloff (grenades, rockets)
/// * `Incendiary` - Creates burning area that damages over time
/// * `Flash` - Creates visual impairment effect (flashbangs)
/// * `Smoke` - Creates obscuring smoke screen
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::Payload;
/// 
/// let explosive_payload = Payload::Explosive {
///     damage: 100.0,
///     radius: 5.0,
///     falloff: 1.5,
/// };
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub enum Payload {
    /// Kinetic damage (bullets, arrows)
    Kinetic { damage: f32 },
    /// Explosive damage with radius falloff
    Explosive {
        damage: f32,
        radius: f32,
        falloff: f32,
    },
    /// Incendiary: creates burning area
    Incendiary {
        duration: f32,
        damage_per_second: f32,
        radius: f32,
    },
    /// Flashbang: visual impairment
    Flash {
        intensity: f32,
        duration: f32,
        radius: f32,
    },
    /// Smoke: creates obscuring area
    Smoke {
        duration: f32,
        radius: f32,
    },
}

impl Default for Payload {
    /// Creates a default Payload instance with Kinetic damage.
    /// 
    /// # Returns
    /// A new Payload::Kinetic with 25.0 damage
    fn default() -> Self {
        Self::Kinetic { damage: 25.0 }
    }
}

/// Surface material properties for interaction calculations.
/// 
/// Defines how projectiles interact with different surface materials, including
/// ricochet behavior, penetration resistance, and visual effects.
/// 
/// # Fields
/// * `ricochet_angle` - Threshold angle in radians from surface normal for ricochet
/// * `penetration_loss` - Amount of energy lost when penetrating (affects penetration chance)
/// * `thickness` - Thickness of the material in meters (affects penetration difficulty)
/// * `hit_effect` - Type of visual effect to show on impact
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::{SurfaceMaterial, HitEffectType};
/// 
/// let concrete_material = SurfaceMaterial {
///     ricochet_angle: 0.2,      // ~11 degrees
///     penetration_loss: 80.0,   // High resistance
///     thickness: 0.2,           // 20cm thick
///     hit_effect: HitEffectType::Dust,
/// };
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct SurfaceMaterial {
    /// Ricochet threshold angle (radians from normal)
    pub ricochet_angle: f32,
    /// Penetration resistance (energy loss)
    pub penetration_loss: f32,
    /// Thickness (meters)
    pub thickness: f32,
    /// Hit effect type
    pub hit_effect: HitEffectType,
}

impl Default for SurfaceMaterial {
    /// Creates a default SurfaceMaterial instance representing a generic metallic surface.
    /// 
    /// Default values:
    /// - 0.3 rad ricochet angle (~17 degrees)
    /// - 50.0 penetration loss
    /// - 0.05m thickness (5cm)
    /// - Sparks hit effect
    /// 
    /// # Returns
    /// A new SurfaceMaterial instance with default values
    fn default() -> Self {
        Self {
            ricochet_angle: 0.3,   // ~17 degrees
            penetration_loss: 50.0,
            thickness: 0.05,       // 5cm
            hit_effect: HitEffectType::Sparks,
        }
    }
}

/// Types of visual effects on hit.
/// 
/// Defines the type of visual effect to display when a projectile impacts a surface.
/// Different variants represent different materials and their corresponding effects.
/// 
/// # Variants
/// * `Sparks` - Metallic sparks for metal surfaces
/// * `Dust` - Dust clouds for concrete, stone, or earth
/// * `Blood` - Blood splatter for organic/flesh materials
/// * `WoodChips` - Wood fragments for wooden surfaces
/// * `Water` - Splash effects for liquid surfaces
/// * `Glass` - Shattered glass fragments for glass surfaces
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::HitEffectType;
/// 
/// let effect_type = HitEffectType::Sparks;
/// ```
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default)]
pub enum HitEffectType {
    #[default]
    /// Metallic sparks for metal surfaces
    Sparks,
    /// Dust clouds for concrete, stone, or earth
    Dust,
    /// Blood splatter for organic/flesh materials
    Blood,
    /// Wood fragments for wooden surfaces
    WoodChips,
    /// Splash effects for liquid surfaces
    Water,
    /// Shattered glass fragments for glass surfaces
    Glass,
}

/// Marker component for active bullet tracers.
/// 
/// This component marks entities as bullet tracers with properties controlling
/// their visual appearance and lifetime.
/// 
/// # Fields
/// * `lifetime` - Remaining lifetime in seconds before the tracer disappears
/// * `trail_length` - Length of the tracer's visual trail
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::BulletTracer;
/// 
/// let tracer = BulletTracer {
///     lifetime: 2.0,
///     trail_length: 1.5,
/// };
/// ```
#[derive(Component, Default)]
pub struct BulletTracer {
    /// Lifetime remaining (seconds)
    pub lifetime: f32,
    /// Trail length
    pub trail_length: f32,
}

/// Marker component for impact decals.
/// 
/// This component marks entities as impact decals with properties controlling
/// their lifetime and visual appearance.
/// 
/// # Fields
/// * `lifetime` - Remaining lifetime in seconds before the decal disappears
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::ImpactDecal;
/// 
/// let decal = ImpactDecal {
///     lifetime: 30.0,
/// };
/// ```
#[derive(Component, Default)]
pub struct ImpactDecal {
    /// Lifetime remaining (seconds)
    pub lifetime: f32,
}

/// Network entity marker for multiplayer synchronization.
/// 
/// This component marks projectiles that are synchronized across the network
/// in multiplayer games, containing information needed for authoritative
/// server simulation and client prediction.
/// 
/// # Fields
/// * `owner_id` - Unique identifier of the player who fired this projectile
/// * `timestamp` - Server timestamp when the projectile was created
/// * `spread_seed` - Random seed for deterministic spread calculation across clients
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::NetProjectile;
/// 
/// let net_proj = NetProjectile {
///     owner_id: 12345,
///     timestamp: 123456.789,
///     spread_seed: 9876543210,
/// };
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct NetProjectile {
    /// Owner player ID
    pub owner_id: u64,
    /// Server timestamp of creation
    pub timestamp: f64,
    /// Random seed for deterministic spread calculation
    pub spread_seed: u64,
}

/// Component for weapon zeroing (scope adjustment).
/// 
/// This component stores information about how a weapon is zeroed at a particular
/// distance, allowing for realistic ballistic drop compensation in scopes and sights.
/// 
/// # Fields
/// * `distance` - The distance in meters at which the weapon is zeroed
/// * `pitch_adjustment` - Calculated pitch adjustment in radians to compensate for bullet drop
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::WeaponZeroing;
/// 
/// let zeroing = WeaponZeroing {
///     distance: 200.0,          // Zeroed at 200 meters
///     pitch_adjustment: 0.005,  // Small upward adjustment
/// };
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct WeaponZeroing {
    /// Zeroed distance in meters
    pub distance: f32,
    /// Calculated pitch adjustment (radians)
    pub pitch_adjustment: f32,
}

impl Default for WeaponZeroing {
    /// Creates a default WeaponZeroing instance zeroed at 100 meters.
    /// 
    /// # Returns
    /// A new WeaponZeroing instance with default values (100m zero, no adjustment)
    fn default() -> Self {
        Self {
            distance: 100.0,
            pitch_adjustment: 0.0,
        }
    }
}

/// Component for muzzle flash visual effects.
/// 
/// This component marks entities as muzzle flash effects with properties
/// controlling their visual appearance and lifetime.
/// 
/// # Fields
/// * `lifetime` - Remaining lifetime in seconds before the flash disappears
/// * `intensity` - Initial intensity of the flash (affects emissive strength)
/// * `scale` - Size scale of the flash effect
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::MuzzleFlash;
/// 
/// let flash = MuzzleFlash {
///     lifetime: 0.05,
///     intensity: 5.0,
///     scale: 0.5,
/// };
/// ```
#[derive(Component, Default)]
pub struct MuzzleFlash {
    /// Lifetime remaining (seconds)
    pub lifetime: f32,
    /// Intensity of the flash
    pub intensity: f32,
    /// Scale of the flash effect
    pub scale: f32,
}

/// Component for explosion visual effects.
/// 
/// This component marks entities as explosion effects with properties
/// controlling their visual appearance and lifetime.
/// 
/// # Fields
/// * `lifetime` - Remaining lifetime in seconds before the explosion disappears
/// * `max_radius` - Maximum radius the explosion will expand to
/// * `current_radius` - Current radius of the explosion
/// * `intensity` - Light intensity of the explosion
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::components::ExplosionVFX;
/// 
/// let explosion = ExplosionVFX {
///     lifetime: 1.0,
///     max_radius: 5.0,
///     current_radius: 0.0,
///     intensity: 10.0,
/// };
/// ```
#[derive(Component, Default)]
pub struct ExplosionVFX {
    /// Lifetime remaining (seconds)
    pub lifetime: f32,
    /// Maximum radius of the explosion
    pub max_radius: f32,
    /// Current radius of the explosion
    pub current_radius: f32,
    /// Light intensity
    pub intensity: f32,
}

/// Weapon component for handling fire rate, automatic fire, and burst modes.
/// 
/// This component stores the state of a weapon, allowing for rate-limited firing,
/// automatic fire, and burst modes.
/// 
/// # Fields
/// * `fire_rate` - Maximum shots per second (0.0 for manual action)
/// * `last_fire_time` - Timestamp of the last shot
/// * `automatic` - Whether the weapon fires continuously while trigger is held
/// * `burst_count` - Number of shots in a burst (0 for standard auto/semi)
/// * `shots_in_burst` - Counter for shots fired in current burst
/// * `burst_interval` - Time between shots in a burst (seconds)
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::components::Weapon;
/// 
/// let assault_rifle = Weapon {
///     fire_rate: 10.0, // 600 RPM
///     automatic: true,
///     ..Default::default()
/// };
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Weapon {
    /// Shots per second (0.0 for manual action)
    pub fire_rate: f32,
    /// Last time the weapon was fired
    pub last_fire_time: f64,
    /// Whether the weapon is automatic (keeps firing while button held)
    pub automatic: bool,
    /// Number of shots in a burst (0 for standard auto/semi)
    pub burst_count: u32,
    /// Shots fired in current burst
    pub shots_in_burst: u32,
    /// Time between shots in a burst (seconds)
    pub burst_interval: f32,
}

impl Default for Weapon {
    /// Creates a default manual-action weapon.
    fn default() -> Self {
        Self {
            fire_rate: 0.0,
            last_fire_time: 0.0,
            automatic: false,
            burst_count: 0,
            shots_in_burst: 0,
            burst_interval: 0.1,
        }
    }
}

impl Weapon {
    /// Checks if the weapon is ready to fire based on fire rate.
    ///
    /// # Arguments
    /// * `current_time` - Current game time in seconds
    ///
    /// # Returns
    /// True if enough time has passed since the last shot
    pub fn can_fire(&self, current_time: f64) -> bool {
        if self.fire_rate <= 0.0 {
            return true;
        }
        let interval = 1.0 / self.fire_rate;
        current_time - self.last_fire_time >= interval as f64
    }
}

/// Guidance component for homing projectiles (missiles).
/// 
/// This component enables a projectile to steer towards a target entity.
/// 
/// # Fields
/// * `target` - Entity to seek
/// * `turn_rate` - Maximum turn rate in radians per second
/// * `delay` - Time before guidance activates (seconds)
/// * `elapsed` - Time since spawn
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Guidance {
    /// Target entity to follow
    pub target: Option<Entity>,
    /// Turn rate in radians per second
    pub turn_rate: f32,
    /// Delay before guidance activates (seconds)
    pub delay: f32,
    /// Time elapsed since spawn (seconds)
    pub elapsed: f32,
}

impl Default for Guidance {
    /// default: no target, no turn, delay 0.5s
    fn default() -> Self {
        Self {
            target: None,
            turn_rate: 1.0, // ~60 degrees/sec
            delay: 0.5,
            elapsed: 0.0,
        }
    }
}


//! Global resources for the ballistics system.

use bevy::prelude::*;

/// Global environment settings affecting all projectiles.
/// 
/// This resource contains global environmental parameters that affect the
/// physics simulation of all projectiles in the scene, such as gravity,
/// air density, and wind conditions.
/// 
/// # Fields
/// * `gravity` - Gravity vector in meters per second squared
/// * `air_density` - Air density in kg/m³ affecting drag calculations
/// * `wind` - Wind velocity vector in meters per second
/// * `temperature` - Ambient temperature in Celsius affecting air density
/// * `altitude` - Altitude in meters affecting air density
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::resources::BallisticsEnvironment;
/// 
/// let env = BallisticsEnvironment {
///     gravity: Vec3::new(0.0, -9.81, 0.0),
///     air_density: 1.1,
///     wind: Vec3::new(2.0, 0.0, 0.0),
///     temperature: 25.0,
///     altitude: 100.0,
/// };
/// ```
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct BallisticsEnvironment {
    /// Gravity vector (m/s²)
    pub gravity: Vec3,
    /// Air density affecting drag (kg/m³)
    pub air_density: f32,
    /// Wind velocity vector (m/s)
    pub wind: Vec3,
    /// Temperature affecting air density calculations (Celsius)
    pub temperature: f32,
    /// Altitude affecting air density (meters)
    pub altitude: f32,
}

impl Default for BallisticsEnvironment {
    /// Creates a default BallisticsEnvironment with Earth-like conditions.
    /// 
    /// Default values:
    /// - Gravity: 9.81 m/s² downward
    /// - Air density: 1.225 kg/m³ (sea level standard)
    /// - No wind
    /// - Temperature: 20°C
    /// - Altitude: Sea level (0m)
    /// 
    /// # Returns
    /// A new BallisticsEnvironment instance with default values
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            air_density: 1.225, // Standard at sea level
            wind: Vec3::ZERO,
            temperature: 20.0,
            altitude: 0.0,
        }
    }
}

impl BallisticsEnvironment {
    /// Creates environment for 2D (ignores Z component).
    /// 
    /// This constructor creates an environment suitable for 2D simulations
    /// where the Z-axis is ignored in physics calculations.
    /// 
    /// # Returns
    /// A new BallisticsEnvironment instance with 2D-appropriate settings
    pub fn new_2d() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ..Default::default()
        }
    }

    /// Calculate adjusted air density based on altitude and temperature.
    /// 
    /// Uses a simplified barometric formula to adjust air density based on
    /// the current altitude and temperature conditions.
    /// 
    /// # Returns
    /// The effective air density considering altitude and temperature
    pub fn effective_air_density(&self) -> f32 {
        // Simplified barometric formula
        let temp_kelvin = self.temperature + 273.15;
        let pressure_ratio = (-self.altitude / 8500.0).exp();
        self.air_density * pressure_ratio * (288.15 / temp_kelvin)
    }
}

/// Global configuration for the ballistics system.
/// 
/// This resource contains global configuration options that control the
/// behavior and performance of the entire ballistics system.
/// 
/// # Fields
/// * `use_rk4` - Whether to use RK4 integration (more accurate) or Euler (faster)
/// * `max_projectile_lifetime` - Maximum time in seconds before projectiles auto-despawn
/// * `max_projectile_distance` - Maximum distance in meters before projectiles auto-despawn
/// * `enable_penetration` - Whether to enable projectile penetration mechanics
/// * `enable_ricochet` - Whether to enable projectile ricochet mechanics
/// * `debug_draw` - Whether to enable debug visualization of projectile paths
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::resources::BallisticsConfig;
/// 
/// let config = BallisticsConfig {
///     use_rk4: true,
///     max_projectile_lifetime: 15.0,
///     max_projectile_distance: 3000.0,
///     enable_penetration: true,
///     enable_ricochet: false,
///     debug_draw: true,
/// };
/// ```
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct BallisticsConfig {
    /// Use RK4 integration (true) or Euler (false)
    pub use_rk4: bool,
    /// Maximum projectile lifetime before auto-despawn (seconds)
    pub max_projectile_lifetime: f32,
    /// Maximum projectile distance before auto-despawn (meters)
    pub max_projectile_distance: f32,
    /// Enable penetration system
    pub enable_penetration: bool,
    /// Enable ricochet system
    pub enable_ricochet: bool,
    /// Debug visualization
    pub debug_draw: bool,
}

impl Default for BallisticsConfig {
    /// Creates a default BallisticsConfig with recommended settings for most use cases.
    /// 
    /// Default values:
    /// - Use RK4 integration for accuracy
    /// - 10 second maximum projectile lifetime
    /// - 2000 meter maximum projectile distance
    /// - Penetration enabled
    /// - Ricochet enabled
    /// - Debug drawing disabled
    /// 
    /// # Returns
    /// A new BallisticsConfig instance with default values
    fn default() -> Self {
        Self {
            use_rk4: true,
            max_projectile_lifetime: 10.0,
            max_projectile_distance: 2000.0,
            enable_penetration: true,
            enable_ricochet: true,
            debug_draw: false,
        }
    }
}

/// Object pool for bullet tracers.
/// 
/// This resource manages an object pool of tracer entities to improve performance
/// by reusing existing entities instead of constantly spawning and despawning them.
/// 
/// # Fields
/// * `available` - Vector of inactive tracer entities available for reuse
/// * `max_size` - Maximum number of entities that can be stored in the pool
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::resources::TracerPool;
/// 
/// let mut pool = TracerPool::new(100);
/// if let Some(tracer_entity) = pool.get() {
///     // Use the tracer entity
/// } else {
///     // Pool is empty, create a new tracer
/// }
/// ```
#[derive(Resource, Default)]
pub struct TracerPool {
    /// Available (inactive) tracer entities
    pub available: Vec<Entity>,
    /// Maximum pool size
    pub max_size: usize,
}

impl TracerPool {
    /// Creates a new TracerPool with the specified maximum size.
    /// 
    /// # Arguments
    /// * `max_size` - Maximum number of tracer entities to store in the pool
    /// 
    /// # Returns
    /// A new TracerPool instance with the specified capacity
    pub fn new(max_size: usize) -> Self {
        Self {
            available: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Get a tracer from pool or None if empty.
    /// 
    /// Retrieves an available tracer entity from the pool for reuse.
    /// Returns None if the pool is empty.
    /// 
    /// # Returns
    /// An Option containing an Entity if available, or None if the pool is empty
    pub fn get(&mut self) -> Option<Entity> {
        self.available.pop()
    }

    /// Return a tracer to the pool.
    /// 
    /// Adds a tracer entity back to the pool for future reuse.
    /// The entity will only be added if the pool hasn't reached its maximum size.
    /// 
    /// # Arguments
    /// * `entity` - The tracer entity to return to the pool
    pub fn release(&mut self, entity: Entity) {
        if self.available.len() < self.max_size {
            self.available.push(entity);
        }
    }
}

/// Object pool for impact decals.
/// 
/// This resource manages an object pool of decal entities to improve performance
/// by reusing existing entities instead of constantly spawning and despawning them.
/// 
/// # Fields
/// * `available` - Vector of inactive decal entities available for reuse
/// * `max_size` - Maximum number of entities that can be stored in the pool
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::resources::DecalPool;
/// 
/// let mut pool = DecalPool::new(50);
/// if let Some(decal_entity) = pool.get() {
///     // Use the decal entity
/// } else {
///     // Pool is empty, create a new decal
/// }
/// ```
#[derive(Resource, Default)]
pub struct DecalPool {
    /// Available (inactive) decal entities
    pub available: Vec<Entity>,
    /// Maximum pool size
    pub max_size: usize,
}

impl DecalPool {
    /// Creates a new DecalPool with the specified maximum size.
    /// 
    /// # Arguments
    /// * `max_size` - Maximum number of decal entities to store in the pool
    /// 
    /// # Returns
    /// A new DecalPool instance with the specified capacity
    pub fn new(max_size: usize) -> Self {
        Self {
            available: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Get a decal from pool or None if empty.
    /// 
    /// Retrieves an available decal entity from the pool for reuse.
    /// Returns None if the pool is empty.
    /// 
    /// # Returns
    /// An Option containing an Entity if available, or None if the pool is empty
    pub fn get(&mut self) -> Option<Entity> {
        self.available.pop()
    }

    /// Return a decal to the pool.
    /// 
    /// Adds a decal entity back to the pool for future reuse.
    /// The entity will only be added if the pool hasn't reached its maximum size.
    /// 
    /// # Arguments
    /// * `entity` - The decal entity to return to the pool
    pub fn release(&mut self, entity: Entity) {
        if self.available.len() < self.max_size {
            self.available.push(entity);
        }
    }
}

/// Weapon preset definitions resource.
/// 
/// This resource contains predefined weapon configurations that can be used
/// to quickly set up different weapon types with consistent parameters.
/// 
/// # Fields
/// * `presets` - Vector of available weapon preset configurations
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::resources::WeaponPresets;
/// 
/// let presets = WeaponPresets::with_defaults();
/// let rifle_preset = &presets.presets[1]; // Assuming rifle is second preset
/// ```
#[derive(Resource, Default)]
pub struct WeaponPresets {
    pub presets: Vec<WeaponPreset>,
}

/// A preset weapon configuration.
/// 
/// This struct defines a complete configuration for a weapon type,
/// including projectile properties, damage, and accuracy characteristics.
/// 
/// # Fields
/// * `name` - Human-readable name for the weapon preset
/// * `muzzle_velocity` - Initial velocity of projectiles fired by this weapon (m/s)
/// * `projectile_mass` - Mass of projectiles fired by this weapon (kg)
/// * `drag_coefficient` - Drag coefficient affecting projectile flight
/// * `base_damage` - Base damage dealt by projectiles from this weapon
/// * `accuracy` - Accuracy characteristics including spread and bloom
/// 
/// # Example
/// ```
/// use bevy_bullet_dynamics::resources::WeaponPreset;
/// use bevy_bullet_dynamics::components::Accuracy;
/// 
/// let preset = WeaponPreset {
///     name: "Sniper Rifle".to_string(),
///     muzzle_velocity: 1200.0,
///     projectile_mass: 0.01,
///     drag_coefficient: 0.2,
///     base_damage: 100.0,
///     accuracy: Accuracy::default(),
/// };
/// ```
#[derive(Clone)]
pub struct WeaponPreset {
    pub name: String,
    pub muzzle_velocity: f32,
    pub projectile_mass: f32,
    pub drag_coefficient: f32,
    pub base_damage: f32,
    pub accuracy: crate::components::Accuracy,
}

impl Default for WeaponPreset {
    /// Creates a default WeaponPreset with reasonable values for a typical rifle.
    /// 
    /// Default values:
    /// - Name: "Default"
    /// - Muzzle velocity: 400 m/s
    /// - Projectile mass: 10g
    /// - Drag coefficient: 0.3
    /// - Base damage: 25.0
    /// - Default accuracy settings
    /// 
    /// # Returns
    /// A new WeaponPreset instance with default values
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            muzzle_velocity: 400.0,
            projectile_mass: 0.01,
            drag_coefficient: 0.3,
            base_damage: 25.0,
            accuracy: crate::components::Accuracy::default(),
        }
    }
}

/// Predefined weapon presets.
impl WeaponPresets {
    /// Creates a WeaponPresets instance with default weapon configurations.
    /// 
    /// This method returns a collection of commonly used weapon presets:
    /// - Pistol: Low velocity, moderate damage, higher spread
    /// - Rifle: High velocity, medium damage, tight accuracy
    /// - Sniper: Very high velocity, high damage, exceptional accuracy
    /// - Bow: Low velocity, high damage, moderate accuracy, no bloom
    /// 
    /// # Returns
    /// A new WeaponPresets instance with default weapon configurations
    pub fn with_defaults() -> Self {
        Self {
            presets: vec![
                WeaponPreset {
                    name: "Pistol".to_string(),
                    muzzle_velocity: 350.0,
                    projectile_mass: 0.008,
                    drag_coefficient: 0.35,
                    base_damage: 20.0,
                    accuracy: crate::components::Accuracy {
                        base_spread: 0.003,
                        bloom_per_shot: 0.015,
                        ..Default::default()
                    },
                },
                WeaponPreset {
                    name: "Rifle".to_string(),
                    muzzle_velocity: 900.0,
                    projectile_mass: 0.004,
                    drag_coefficient: 0.25,
                    base_damage: 35.0,
                    accuracy: crate::components::Accuracy {
                        base_spread: 0.001,
                        bloom_per_shot: 0.02,
                        ..Default::default()
                    },
                },
                WeaponPreset {
                    name: "Sniper".to_string(),
                    muzzle_velocity: 1200.0,
                    projectile_mass: 0.01,
                    drag_coefficient: 0.2,
                    base_damage: 100.0,
                    accuracy: crate::components::Accuracy {
                        base_spread: 0.0005,
                        bloom_per_shot: 0.03,
                        ads_modifier: 0.1,
                        ..Default::default()
                    },
                },
                WeaponPreset {
                    name: "Bow".to_string(),
                    muzzle_velocity: 80.0,
                    projectile_mass: 0.03,
                    drag_coefficient: 0.5,
                    base_damage: 45.0,
                    accuracy: crate::components::Accuracy {
                        base_spread: 0.002,
                        bloom_per_shot: 0.0,
                        ads_modifier: 0.2,
                        ..Default::default()
                    },
                },
            ],
        }
    }
}

//! # Bevy Bullet Dynamics
//!
//! High-precision ballistics simulation plugin for Bevy 0.18.
//!
//! ## Features
//! - RK4 integration for accurate projectile physics
//! - Multiple weapon types: pistols, rifles, bows, grenades
//! - 2D and 3D support via feature flags
//! - Client-server architecture ready
//! - Object pooling for performance
//! - Dynamic accuracy and spread system
//! - Surface interactions: ricochets, penetration, decals
//!
//! ## Quick Start
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_bullet_dynamics::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(BallisticsPluginGroup)
//!         .run();
//! }
//! ```

pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
pub mod types;

#[cfg(feature = "netcode")]
pub mod network;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::events::*;
    pub use crate::resources::*;
    pub use crate::types::*;
    pub use crate::BallisticsPluginGroup;
    pub use crate::{BallisticsCorePlugin, BallisticsSurfacePlugin, BallisticsVfxPlugin};
}

use bevy::prelude::*;

/// Main plugin group that includes all ballistics subsystems.
/// 
/// This plugin group bundles together the core ballistics functionality:
/// - Physics calculations and kinematics
/// - Surface interactions (penetration, ricochets)
/// - Visual effects (tracers, decals, impact effects)
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_bullet_dynamics::prelude::*;
/// 
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(BallisticsPluginGroup)
///         .run();
/// }
/// ```
#[derive(Default)]
pub struct BallisticsPluginGroup;

impl PluginGroup for BallisticsPluginGroup {
    /// Builds the plugin group by adding all ballistics plugins.
    /// 
    /// This method adds the core, surface, and VFX plugins to the application.
    /// 
    /// # Arguments
    /// * `self` - The BallisticsPluginGroup instance
    /// 
    /// # Returns
    /// A PluginGroupBuilder with all ballistics plugins added
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(BallisticsCorePlugin)
            .add(BallisticsSurfacePlugin)
            .add(BallisticsVfxPlugin)
    }
}

/// Core physics calculations plugin (RK4 integration, kinematics).
/// 
/// This plugin handles the fundamental physics calculations for projectiles,
/// including:
/// - RK4 integration for accurate projectile physics
/// - Kinematic updates for projectile positions
/// - Collision detection and handling
/// - Projectile logic processing (timed fuses, etc.)
/// 
/// # Systems
/// - `update_bloom` - Updates accuracy bloom over time
/// - `update_projectiles_kinematics` - Updates projectile positions using physics
/// - `handle_collisions` - Detects and processes projectile collisions
/// - `process_projectile_logic` - Handles timed fuses and other projectile behaviors
pub struct BallisticsCorePlugin;

impl Plugin for BallisticsCorePlugin {
    /// Builds the core ballistics plugin by registering components and adding systems.
    /// 
    /// This method registers all necessary components with reflection, initializes
    /// required resources, adds events, and schedules the core systems.
    /// 
    /// # Arguments
    /// * `app` - Mutable reference to the Bevy App
    fn build(&self, app: &mut App) {
        app.register_type::<components::Projectile>()
            .register_type::<components::Accuracy>()
            .register_type::<components::ProjectileLogic>()
            .register_type::<components::ProjectileLogic>()
            .register_type::<components::Payload>()
            .register_type::<components::Weapon>()
            .init_resource::<resources::BallisticsEnvironment>()
            .init_resource::<resources::BallisticsConfig>()
            .add_message::<events::FireEvent>()
            .add_message::<events::HitEvent>()
            .add_message::<events::ExplosionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    systems::accuracy::update_bloom,
                    systems::kinematics::update_projectiles_kinematics,
                    systems::collision::handle_collisions,
                    systems::logic::process_projectile_logic,
                    systems::logic::apply_explosion_impulse,
                )
                    .chain(),
            );
    }
}

/// Surface interaction plugin (ricochets, penetration, material effects).
/// 
/// This plugin handles how projectiles interact with different surface materials,
/// including:
/// - Ricochet calculations based on impact angle
/// - Penetration mechanics for different materials
/// - Material-specific effects and responses
/// 
/// # Systems
/// - `process_surface_interactions` - Handles penetration and ricochet logic
pub struct BallisticsSurfacePlugin;

impl Plugin for BallisticsSurfacePlugin {
    /// Builds the surface interaction plugin by registering components and adding systems.
    /// 
    /// This method registers surface material components with reflection and adds
    /// the surface interaction system to the application.
    /// 
    /// # Arguments
    /// * `app` - Mutable reference to the Bevy App
    fn build(&self, app: &mut App) {
        app.register_type::<components::SurfaceMaterial>()
            .add_systems(FixedUpdate, systems::surface::process_surface_interactions);
    }
}

/// VFX plugin (pooling, tracers, decals, impact effects).
/// 
/// This plugin manages visual effects for projectiles, including:
/// - Object pooling for tracers and decals to improve performance
/// - Visual tracer effects for projectiles
/// - Impact decals and effects
/// - Cleanup of expired visual effects
/// 
/// # Systems
/// - `update_tracers` - Updates tracer lifetimes and hides expired ones
/// - `spawn_impact_effects` - Spawns visual effects at hit locations
/// - `cleanup_expired_effects` - Cleans up expired visual effects
pub struct BallisticsVfxPlugin;

impl Plugin for BallisticsVfxPlugin {
    /// Builds the VFX plugin by initializing resources and adding systems.
    /// 
    /// This method initializes the tracer and decal pools and adds the VFX systems
    /// to the application.
    /// 
    /// # Arguments
    /// * `app` - Mutable reference to the Bevy App
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::TracerPool>()
            .init_resource::<resources::DecalPool>()
            .add_systems(
                Update,
                (
                    systems::vfx::update_tracers,
                    systems::vfx::spawn_impact_effects,
                    systems::vfx::cleanup_expired_effects,
                    systems::vfx::update_muzzle_flash,
                    systems::vfx::update_explosion_vfx,
                    systems::vfx::spawn_explosion_vfx_from_event,
                ),
            );
    }
}

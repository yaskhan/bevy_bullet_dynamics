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
            .add(BallisticsDebugPlugin)
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
            .register_type::<components::Payload>()
            .register_type::<components::Weapon>()
            .register_type::<components::Guidance>()
            .init_resource::<resources::BallisticsEnvironment>()
            .init_resource::<resources::BallisticsConfig>()
            .add_message::<events::FireEvent>()
            .add_message::<events::HitEvent>()
            .add_message::<events::ExplosionEvent>()
            .add_message::<events::PenetrationEvent>()
            .add_message::<events::RicochetEvent>()
            .add_systems(
                FixedUpdate,
                (
                    systems::accuracy::update_bloom,
                    systems::kinematics::update_guidance,
                    systems::kinematics::update_projectiles_kinematics,
                    systems::logic::process_projectile_logic,
                    systems::logic::cleanup_expired_projectiles,
                )
                    .chain(),
            );

        // 3D Physics Systems
        #[cfg(feature = "dim3")]
        {
            use avian3d::prelude::SpatialQueryPipeline;
            app.add_systems(
                FixedUpdate,
                (
                    systems::collision::handle_collisions,
                    systems::logic::apply_explosion_impulse,
                    systems::logic::process_hitscan,
                )
                    .run_if(resource_exists::<SpatialQueryPipeline>),
            );
        }

        // 2D Physics Systems
        #[cfg(feature = "dim2")]
        {
            use avian2d::prelude::SpatialQueryPipeline;
            app.add_systems(
                FixedUpdate,
                (
                    systems::collision::handle_collisions_2d,
                    systems::logic::apply_explosion_impulse_2d,
                    systems::logic::process_hitscan_2d,
                )
                    .run_if(resource_exists::<SpatialQueryPipeline>),
            );
        }
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
            .init_resource::<resources::BallisticsAssets>()
            .add_systems(Startup, setup_ballistics_assets)
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

/// Setup common ballistics assets.
fn setup_ballistics_assets(
    mut assets: ResMut<resources::BallisticsAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    assets.sphere_mesh = meshes.add(Sphere::new(1.0));
    assets.tracer_mesh = meshes.add(Cylinder::new(0.02, 1.0));
    
    assets.spark_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.7, 0.2),
        emissive: LinearRgba::rgb(5.0, 3.0, 0.5),
        ..default()
    });
    
    assets.dust_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.5, 0.4, 0.8),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    
    assets.blood_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.0, 0.0),
        emissive: LinearRgba::rgb(0.3, 0.0, 0.0),
        ..default()
    });
    
    assets.flash_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.5),
        emissive: LinearRgba::rgb(5.0, 4.0, 1.0),
        unlit: true,
        ..default()
    });
    
    assets.explosion_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.0),
        emissive: LinearRgba::rgb(10.0, 5.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
}

/// Debug plugin for ballistics visualization.
pub struct BallisticsDebugPlugin;

impl Plugin for BallisticsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, systems::debug::draw_projectile_debug);
    }
}


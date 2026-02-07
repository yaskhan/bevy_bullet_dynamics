//! Surface interaction system - penetration and ricochet logic.

use bevy::prelude::*;

use crate::components::{Projectile, SurfaceMaterial};
use crate::resources::BallisticsConfig;

/// Process surface interactions (penetration, ricochet).
/// 
/// This system handles the advanced surface interaction logic including
/// penetration and ricochet calculations based on projectile and surface properties.
/// 
/// # Arguments
/// * `_config` - Ballistics configuration resource (currently unused in this placeholder)
pub fn process_surface_interactions(
    _config: Res<BallisticsConfig>,
    // Placeholder - actual implementation depends on physics backend
    // This system processes events from the collision system
    // and handles penetration/ricochet logic
) {
    // TODO: Implement when physics backend is integrated
}

/// Calculate if a projectile can penetrate a surface.
/// 
/// Determines whether a projectile has sufficient penetration power to pass
/// through a surface, taking into account the impact angle.
/// 
/// # Arguments
/// * `projectile` - Reference to the projectile component
/// * `surface` - Reference to the surface material component
/// * `impact_angle` - Angle between the projectile's velocity and the surface normal (in radians)
/// 
/// # Returns
/// True if the projectile can penetrate the surface, false otherwise
pub fn can_penetrate(
    projectile: &Projectile,
    surface: &SurfaceMaterial,
    impact_angle: f32,
) -> bool {
    // Penetration is harder at shallow angles
    let angle_factor = impact_angle.cos().abs();
    let effective_power = projectile.penetration_power * angle_factor;

    effective_power > surface.penetration_loss
}

/// Calculate remaining penetration power after passing through material.
/// 
/// Computes how much penetration power remains after a projectile travels
/// through a certain distance of a surface material.
/// 
/// # Arguments
/// * `initial_power` - The projectile's initial penetration power
/// * `surface` - Reference to the surface material component
/// * `travel_distance` - The distance the projectile traveled through the material
/// 
/// # Returns
/// The remaining penetration power after traveling through the material
pub fn calculate_remaining_penetration(
    initial_power: f32,
    surface: &SurfaceMaterial,
    travel_distance: f32,
) -> f32 {
    // Power loss is proportional to travel distance through material
    let distance_factor = travel_distance / surface.thickness;
    let power_loss = surface.penetration_loss * distance_factor;

    (initial_power - power_loss).max(0.0)
}

/// Calculate exit velocity after penetration.
/// 
/// Computes the velocity of a projectile after it has penetrated a surface,
/// accounting for energy loss due to material resistance.
/// 
/// # Arguments
/// * `entry_velocity` - The velocity vector of the projectile when entering the surface
/// * `surface` - Reference to the surface material component
/// * `travel_distance` - The distance the projectile traveled through the material
/// 
/// # Returns
/// The velocity vector of the projectile after penetration
pub fn calculate_exit_velocity(
    entry_velocity: Vec3,
    surface: &SurfaceMaterial,
    travel_distance: f32,
) -> Vec3 {
    // Speed reduction based on material resistance and distance
    let speed = entry_velocity.length();
    let thickness_ratio = (travel_distance / surface.thickness).min(1.0);
    let speed_loss_ratio = surface.penetration_loss / 100.0 * thickness_ratio;
    let exit_speed = speed * (1.0 - speed_loss_ratio).max(0.1);

    entry_velocity.normalize() * exit_speed
}

/// Check if projectile should ricochet based on impact angle.
/// 
/// Determines whether a projectile will ricochet off a surface based on
/// the angle at which it impacts the surface.
/// 
/// # Arguments
/// * `velocity` - The velocity vector of the projectile
/// * `surface_normal` - The normal vector of the surface
/// * `surface` - Reference to the surface material component
/// 
/// # Returns
/// True if the projectile should ricochet, false otherwise
pub fn should_ricochet(
    velocity: Vec3,
    surface_normal: Vec3,
    surface: &SurfaceMaterial,
) -> bool {
    // Calculate impact angle (angle from surface normal)
    let impact_angle = velocity.normalize().dot(-surface_normal).acos();

    // Ricochet occurs when impact angle exceeds threshold (shallow impact)
    impact_angle > surface.ricochet_angle
}

/// Calculate ricochet direction and speed.
/// 
/// Computes the new direction and speed of a projectile after it ricochets
/// off a surface, accounting for energy loss during the impact.
/// 
/// # Arguments
/// * `velocity` - The velocity vector of the projectile before ricochet
/// * `surface_normal` - The normal vector of the surface
/// * `surface` - Reference to the surface material component
/// 
/// # Returns
/// A tuple containing the new direction vector and speed after ricochet
pub fn calculate_ricochet(
    velocity: Vec3,
    surface_normal: Vec3,
    surface: &SurfaceMaterial,
) -> (Vec3, f32) {
    let speed = velocity.length();
    let direction = velocity.normalize();

    // Reflect direction off surface
    let reflected = direction - 2.0 * direction.dot(surface_normal) * surface_normal;

    // Speed loss on ricochet (harder surfaces preserve more energy)
    let speed_retention = 1.0 - (surface.penetration_loss / 200.0).min(0.8);
    let new_speed = speed * speed_retention;

    (reflected.normalize(), new_speed)
}

/// Material presets for common surfaces.
pub mod materials {
    use super::*;
    use crate::components::HitEffectType;

    /// Creates a concrete surface material preset.
    /// 
    /// Concrete is a hard material that's difficult to penetrate but allows
    /// ricochets at relatively shallow angles.
    /// 
    /// # Returns
    /// A SurfaceMaterial configured for concrete
    pub fn concrete() -> SurfaceMaterial {
        SurfaceMaterial {
            ricochet_angle: 0.2,      // ~11 degrees - hard surface, easy ricochet
            penetration_loss: 80.0,   // Very hard to penetrate
            thickness: 0.2,
            hit_effect: HitEffectType::Dust,
        }
    }

    /// Creates a metal surface material preset.
    /// 
    /// Metal is extremely hard to penetrate and causes ricochets at very
    /// shallow angles, with sparks as the visual effect.
    /// 
    /// # Returns
    /// A SurfaceMaterial configured for metal
    pub fn metal() -> SurfaceMaterial {
        SurfaceMaterial {
            ricochet_angle: 0.15,     // ~8.5 degrees - very easy to ricochet
            penetration_loss: 100.0,  // Steel is hard to penetrate
            thickness: 0.01,
            hit_effect: HitEffectType::Sparks,
        }
    }

    /// Creates a wood surface material preset.
    /// 
    /// Wood is relatively easy to penetrate and requires steeper angles
    /// for ricochets, with wood chips as the visual effect.
    /// 
    /// # Returns
    /// A SurfaceMaterial configured for wood
    pub fn wood() -> SurfaceMaterial {
        SurfaceMaterial {
            ricochet_angle: 0.5,      // ~28 degrees - harder to ricochet
            penetration_loss: 30.0,   // Easy to penetrate
            thickness: 0.05,
            hit_effect: HitEffectType::WoodChips,
        }
    }

    /// Creates a flesh surface material preset.
    /// 
    /// Flesh is organic material that's difficult to ricochet off of,
    /// with blood as the visual effect.
    /// 
    /// # Returns
    /// A SurfaceMaterial configured for flesh
    pub fn flesh() -> SurfaceMaterial {
        SurfaceMaterial {
            ricochet_angle: 1.5,      // Almost impossible to ricochet
            penetration_loss: 40.0,
            thickness: 0.3,
            hit_effect: HitEffectType::Blood,
        }
    }

    /// Creates a glass surface material preset.
    /// 
    /// Glass is easy to penetrate and allows ricochets at moderate angles,
    /// with glass fragments as the visual effect.
    /// 
    /// # Returns
    /// A SurfaceMaterial configured for glass
    pub fn glass() -> SurfaceMaterial {
        SurfaceMaterial {
            ricochet_angle: 0.8,
            penetration_loss: 10.0,   // Easy to penetrate
            thickness: 0.01,
            hit_effect: HitEffectType::Glass,
        }
    }

    /// Creates a water surface material preset.
    /// 
    /// Water allows ricochets at very shallow angles and has moderate
    /// penetration resistance, with water splash as the visual effect.
    /// 
    /// # Returns
    /// A SurfaceMaterial configured for water
    pub fn water() -> SurfaceMaterial {
        SurfaceMaterial {
            ricochet_angle: 0.1,      // Very easy to ricochet at shallow angles
            penetration_loss: 20.0,
            thickness: 1.0,
            hit_effect: HitEffectType::Water,
        }
    }

    /// Creates a dirt surface material preset.
    /// 
    /// Dirt is moderately difficult to penetrate and allows ricochets
    /// at moderate angles, with dust as the visual effect.
    /// 
    /// # Returns
    /// A SurfaceMaterial configured for dirt
    pub fn dirt() -> SurfaceMaterial {
        SurfaceMaterial {
            ricochet_angle: 0.6,
            penetration_loss: 25.0,
            thickness: 0.5,
            hit_effect: HitEffectType::Dust,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ricochet_detection() {
        let surface = SurfaceMaterial {
            ricochet_angle: 0.3,
            ..Default::default()
        };

        // Shallow angle - should ricochet
        let shallow_velocity = Vec3::new(1.0, -0.1, 0.0);
        let normal = Vec3::Y;
        assert!(should_ricochet(shallow_velocity, normal, &surface));

        // Steep angle - should not ricochet
        let steep_velocity = Vec3::new(0.1, -1.0, 0.0);
        assert!(!should_ricochet(steep_velocity, normal, &surface));
    }

    #[test]
    fn test_ricochet_calculation() {
        let velocity = Vec3::new(100.0, -10.0, 0.0);
        let normal = Vec3::Y;
        let surface = materials::metal();

        let (direction, speed) = calculate_ricochet(velocity, normal, &surface);

        // Direction should be reflected (Y component flipped)
        assert!(direction.y > 0.0);
        assert!(direction.x > 0.0);

        // Speed should be reduced
        assert!(speed < velocity.length());
    }

    #[test]
    fn test_penetration_check() {
        let mut projectile = Projectile::default();
        projectile.penetration_power = 100.0;

        let weak_surface = materials::glass();
        let strong_surface = materials::metal();

        // Should penetrate glass
        assert!(can_penetrate(&projectile, &weak_surface, 0.0));

        // Should not penetrate metal with low power
        projectile.penetration_power = 50.0;
        assert!(!can_penetrate(&projectile, &strong_surface, 0.0));
    }
}

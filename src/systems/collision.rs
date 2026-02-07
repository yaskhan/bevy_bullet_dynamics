//! Collision system - raycast-based hit detection.

use bevy::prelude::*;
use bevy::ecs::message::MessageWriter;

#[cfg(feature = "dim3")]
use avian3d::prelude::*;

use crate::components::{Payload, Projectile, SurfaceMaterial};
use crate::events::HitEvent;
use crate::resources::BallisticsConfig;

/// Handle projectile collisions using raycasting between frames.
///
/// Casts ray from previous_position to current position to catch fast projectiles.
/// Uses avian3d SpatialQuery for actual physics-based collision detection.
/// 
/// # Arguments
/// * `mut commands` - Bevy Commands for entity manipulation
/// * `config` - Ballistics configuration resource
/// * `spatial_query` - Avian3D spatial query for physics-based collision detection
/// * `mut hit_events` - Event writer for sending hit events
/// * `mut projectiles` - Query for projectile entities and their components
/// * `surfaces` - Query for surface material components
#[cfg(feature = "dim3")]
pub fn handle_collisions(
    mut commands: Commands,
    config: Res<BallisticsConfig>,
    spatial_query: SpatialQuery,
    mut hit_events: MessageWriter<HitEvent>,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile, Option<&Payload>)>,
    surfaces: Query<&SurfaceMaterial>,
) {
    for (entity, transform, mut projectile, payload) in projectiles.iter_mut() {
        let ray_origin = projectile.previous_position;
        let ray_end = transform.translation;
        let ray_direction = ray_end - ray_origin;
        let ray_length = ray_direction.length();

        // Skip if projectile hasn't moved enough
        if ray_length < 0.001 {
            projectile.previous_position = transform.translation;
            continue;
        }

        // Normalize direction for raycast
        let direction = match Dir3::new(ray_direction.normalize()) {
            Ok(dir) => dir,
            Err(_) => {
                projectile.previous_position = transform.translation;
                continue;
            }
        };

        // Create filter excluding the projectile itself
        let filter = SpatialQueryFilter::default()
            .with_excluded_entities([entity]);

        // Cast ray from previous to current position
        if let Some(hit) = spatial_query.cast_ray(
            ray_origin,
            direction,
            ray_length,
            true, // solid
            &filter,
        ) {
            let hit_point = ray_origin + *direction * hit.distance;
            let surface = surfaces.get(hit.entity).ok();

            process_hit(
                &mut commands,
                &mut hit_events,
                &config,
                entity,
                &projectile,
                payload,
                hit.entity,
                hit_point,
                hit.normal,
                surface,
            );
        }

        // Update previous position for next frame
        projectile.previous_position = transform.translation;

        // Debug visualization
        if config.debug_draw {
            // Would use gizmos here for debug drawing
        }
    }
}

/// Fallback collision system when dim3 feature is not enabled.
/// 
/// This is a placeholder implementation that does minimal processing when
/// the 3D physics feature is not enabled.
/// 
/// # Arguments
/// * `_commands` - Bevy Commands for entity manipulation (unused in this implementation)
/// * `config` - Ballistics configuration resource
/// * `mut projectiles` - Query for projectile entities and their components
/// * `_surfaces` - Query for surface material components (unused in this implementation)
#[cfg(not(feature = "dim3"))]
pub fn handle_collisions(
    _commands: Commands,
    config: Res<BallisticsConfig>,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile, Option<&Payload>)>,
    _surfaces: Query<&SurfaceMaterial>,
) {
    for (_entity, _transform, mut projectile, _payload) in projectiles.iter_mut() {
        // Placeholder: no physics without dim3 feature
        projectile.previous_position = _transform.translation;

        if config.debug_draw {
            // Debug visualization placeholder
        }
    }
}

/// Process a detected hit.
/// 
/// This function handles the logic when a projectile collides with a surface,
/// determining if it penetrates, ricochets, or stops, and sending the appropriate event.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for entity manipulation
/// * `hit_events` - Event writer for sending hit events
/// * `config` - Ballistics configuration resource
/// * `projectile_entity` - Entity of the projectile that hit
/// * `projectile` - Reference to the projectile component
/// * `payload` - Optional reference to the payload component
/// * `hit_entity` - Entity that was hit by the projectile
/// * `hit_point` - World-space position where the impact occurred
/// * `hit_normal` - Surface normal vector at the impact point
/// * `surface` - Optional reference to the surface material component
#[allow(dead_code)]
pub fn process_hit(
    commands: &mut Commands,
    hit_events: &mut MessageWriter<HitEvent>,
    config: &BallisticsConfig,
    projectile_entity: Entity,
    projectile: &Projectile,
    payload: Option<&Payload>,
    hit_entity: Entity,
    hit_point: Vec3,
    hit_normal: Vec3,
    surface: Option<&SurfaceMaterial>,
) {
    let damage = match payload {
        Some(Payload::Kinetic { damage }) => *damage,
        Some(Payload::Explosive { damage, .. }) => *damage,
        _ => 25.0, // Default damage
    };

    let mut penetrated = false;
    let mut ricocheted = false;

    if let Some(surface) = surface {
        // Check for ricochet based on impact angle
        let impact_angle = projectile.velocity.normalize().dot(-hit_normal).acos();

        if config.enable_ricochet && impact_angle > surface.ricochet_angle {
            ricocheted = true;
        } else if config.enable_penetration {
            // Check penetration
            if projectile.penetration_power > surface.penetration_loss {
                penetrated = true;
            }
        }
    }

    // Send hit event
    hit_events.write(HitEvent {
        projectile: projectile_entity,
        target: hit_entity,
        impact_point: hit_point,
        normal: hit_normal,
        velocity: projectile.velocity,
        damage,
        penetrated,
        ricocheted,
    });

    // Despawn projectile if it didn't penetrate or ricochet
    if !penetrated && !ricocheted {
        commands.entity(projectile_entity).despawn();
    }
}

/// Calculate damage with distance falloff.
/// 
/// Applies a linear falloff to damage based on distance from the origin.
/// Damage remains constant up to falloff_start, then decreases linearly
/// until it reaches 50% of the original damage at falloff_end.
/// 
/// # Arguments
/// * `base_damage` - The original damage value before falloff
/// * `distance` - The distance from the origin to the target
/// * `falloff_start` - Distance at which damage falloff begins
/// * `falloff_end` - Distance at which damage reaches minimum (50% of base)
/// 
/// # Returns
/// The damage value after applying distance falloff
#[allow(dead_code)]
fn calculate_damage_falloff(base_damage: f32, distance: f32, falloff_start: f32, falloff_end: f32) -> f32 {
    if distance <= falloff_start {
        base_damage
    } else if distance >= falloff_end {
        base_damage * 0.5 // Minimum 50% damage
    } else {
        let t = (distance - falloff_start) / (falloff_end - falloff_start);
        base_damage * (1.0 - t * 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_falloff() {
        // No falloff at close range
        assert_eq!(calculate_damage_falloff(100.0, 10.0, 50.0, 100.0), 100.0);

        // Full falloff at max range
        assert_eq!(calculate_damage_falloff(100.0, 100.0, 50.0, 100.0), 50.0);

        // Partial falloff at mid range
        let mid_damage = calculate_damage_falloff(100.0, 75.0, 50.0, 100.0);
        assert!(mid_damage > 50.0 && mid_damage < 100.0);
    }
}

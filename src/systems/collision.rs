//! Collision system - raycast-based hit detection.

use bevy::prelude::*;
use bevy::ecs::message::MessageWriter;

#[cfg(feature = "dim3")]
use avian3d::prelude::*;
#[cfg(feature = "dim2")]
use avian2d::prelude::*;

use crate::components::{Payload, Projectile, SurfaceMaterial};
use crate::events::HitEvent;
use crate::resources::BallisticsConfig;
use crate::systems::surface;

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
    for (entity, mut transform, mut projectile, payload) in projectiles.iter_mut() {
        let ray_origin = projectile.previous_position;
        let ray_end = transform.translation;
        let ray_direction = ray_end - ray_origin;
        let ray_length = ray_direction.length();

        if ray_length < 0.001 {
            projectile.previous_position = transform.translation;
            continue;
        }

        let direction = match Dir3::new(ray_direction.normalize()) {
            Ok(dir) => dir,
            Err(_) => {
                projectile.previous_position = transform.translation;
                continue;
            }
        };

        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        if let Some(hit) = spatial_query.cast_ray(
            ray_origin,
            direction,
            ray_length,
            true,
            &filter,
        ) {
            let hit_point = ray_origin + *direction * hit.distance;
            let surface = surfaces.get(hit.entity).ok();

            process_hit(
                &mut commands,
                &mut hit_events,
                &config,
                entity,
                &mut transform,
                &mut projectile,
                payload,
                hit.entity,
                hit_point,
                hit.normal,
                surface,
            );
        }

        projectile.previous_position = transform.translation;
    }
}

/// Handle collisions for 2D.
#[cfg(feature = "dim2")]
pub fn handle_collisions_2d(
    mut commands: Commands,
    config: Res<BallisticsConfig>,
    spatial_query: SpatialQuery,
    mut hit_events: MessageWriter<HitEvent>,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile, Option<&Payload>)>,
    surfaces: Query<&SurfaceMaterial>,
) {
    for (entity, mut transform, mut projectile, payload) in projectiles.iter_mut() {
        let ray_origin = projectile.previous_position.xy();
        let ray_end = transform.translation.xy();
        let ray_direction = ray_end - ray_origin;
        let ray_length = ray_direction.length();

        if ray_length < 0.001 {
            projectile.previous_position = transform.translation;
            continue;
        }

        let direction = match Dir2::new(ray_direction.normalize()) {
            Ok(dir) => dir,
            Err(_) => {
                projectile.previous_position = transform.translation;
                continue;
            }
        };

        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        if let Some(hit) = spatial_query.cast_ray(
            ray_origin,
            direction,
            ray_length,
            true,
            &filter,
        ) {
            let hit_point = ray_origin + *direction * hit.distance;
            // Convert 2D hit point and normal back to 3D for process_hit
            let hit_point_3d = Vec3::new(hit_point.x, hit_point.y, transform.translation.z);
            let hit_normal_3d = Vec3::new(hit.normal.x, hit.normal.y, 0.0);
            
            let surface = surfaces.get(hit.entity).ok();

            process_hit(
                &mut commands,
                &mut hit_events,
                &config,
                entity,
                &mut transform,
                &mut projectile,
                payload,
                hit.entity,
                hit_point_3d,
                hit_normal_3d,
                surface,
            );
        }

        projectile.previous_position = transform.translation;
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
#[cfg(not(any(feature = "dim3", feature = "dim2")))]
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
    transform: &mut Transform,
    projectile: &mut Projectile,
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
        // Ricochet
        if config.enable_ricochet && surface::should_ricochet(projectile.velocity, hit_normal, surface) {
            let (new_dir, new_speed) = surface::calculate_ricochet(projectile.velocity, hit_normal, surface);
            
            if new_speed > config.min_projectile_speed {
                ricocheted = true;
                projectile.velocity = new_dir * new_speed;
                // Offset hit point slightly along normal to avoid getting stuck inside
                transform.translation = hit_point + hit_normal * 0.05;
            }
        } 
        // Penetration
        else if config.enable_penetration {
            let speed = projectile.velocity.length();
            let dynamic_power = 0.5 * projectile.mass * speed.powi(2) * 0.25;
            
            if dynamic_power > surface.penetration_loss {
                let exit_vel = surface::calculate_exit_velocity(projectile.velocity, surface, surface.thickness);
                
                if exit_vel.length() > config.min_projectile_speed {
                    penetrated = true;
                    projectile.velocity = exit_vel;
                    // Do not snap transform for penetration, it's already past hit_point
                }
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

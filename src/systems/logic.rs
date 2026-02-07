//! Projectile logic system - handles timed fuses, proximity triggers, etc.

use bevy::prelude::*;
use bevy::ecs::message::{MessageWriter, MessageReader};

use crate::components::{Payload, ProjectileLogic};
use crate::events::{ExplosionEvent, ExplosionType};

/// Process projectile-specific logic (timers, proximity triggers).
/// 
/// This system handles special projectile behaviors like timed fuses,
/// proximity triggers, and other logic that's not handled by the collision system.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for entity manipulation
/// * `time` - Bevy FixedTime resource to get delta time
/// * `explosion_events` - Message writer for explosion events
/// * `projectiles` - Query for projectile entities and their components
pub fn process_projectile_logic(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut explosion_events: MessageWriter<ExplosionEvent>,
    mut projectiles: Query<(Entity, &Transform, &mut ProjectileLogic, Option<&Payload>)>,
) {
    let dt = time.delta_secs();

    for (entity, transform, mut logic, payload) in projectiles.iter_mut() {
        match logic.as_mut() {
            ProjectileLogic::Timed { fuse, elapsed } => {
                *elapsed += dt;
                if *elapsed >= *fuse {
                    // Trigger explosion based on payload
                    trigger_explosion(
                        &mut commands,
                        &mut explosion_events,
                        entity,
                        transform.translation,
                        payload,
                    );
                }
            }
            ProjectileLogic::Proximity { range: _ } => {
                // TODO: Query nearby entities and check distance
                // For now, this is a placeholder
            }
            ProjectileLogic::Impact | ProjectileLogic::Sticky => {
                // Handled by collision system
            }
            ProjectileLogic::Hitscan { .. } => {
                // Handled by process_hitscan system (or ignored if not dim3)
            }
        }
    }
}

/// Trigger explosion based on payload type.
fn trigger_explosion(
    commands: &mut Commands,
    explosion_events: &mut MessageWriter<ExplosionEvent>,
    entity: Entity,
    position: Vec3,
    payload: Option<&Payload>,
) {
    // Send explosion event based on payload type
    if let Some(payload) = payload {
        match payload {
            Payload::Explosive { damage, radius, falloff } => {
                explosion_events.write(ExplosionEvent {
                    center: position,
                    radius: *radius,
                    damage: *damage,
                    falloff: *falloff,
                    explosion_type: ExplosionType::HighExplosive,
                    source: Some(entity),
                });
            }
            Payload::Incendiary { duration: _, damage_per_second, radius } => {
                explosion_events.write(ExplosionEvent {
                    center: position,
                    radius: *radius,
                    damage: *damage_per_second,
                    falloff: 1.0,
                    explosion_type: ExplosionType::Incendiary,
                    source: Some(entity),
                });
            }
            Payload::Flash { intensity: _, duration: _, radius } => {
                explosion_events.write(ExplosionEvent {
                    center: position,
                    radius: *radius,
                    damage: 0.0,
                    falloff: 1.0,
                    explosion_type: ExplosionType::Flash,
                    source: Some(entity),
                });
            }
            Payload::Smoke { duration: _, radius } => {
                explosion_events.write(ExplosionEvent {
                    center: position,
                    radius: *radius,
                    damage: 0.0,
                    falloff: 1.0,
                    explosion_type: ExplosionType::Smoke,
                    source: Some(entity),
                });
            }
            Payload::Kinetic { .. } => {
                // Kinetic payloads don't explode
            }
        }
    }

    // Despawn the projectile after explosion
    commands.entity(entity).despawn();
}

#[cfg(feature = "dim3")]
use avian3d::prelude::*;
#[cfg(feature = "dim3")]
use crate::events::HitEvent;
#[cfg(feature = "dim3")]
use crate::resources::BallisticsConfig;
#[cfg(feature = "dim3")]
use crate::systems::collision;

/// Process hitscan projectiles (lasers, railguns).
/// 
/// Performs an immediate raycast and despawns the projectile entity.
#[cfg(feature = "dim3")]
pub fn process_hitscan(
    mut commands: Commands,
    mut hit_events: MessageWriter<HitEvent>,
    config: Res<BallisticsConfig>,
    spatial_query: SpatialQuery,
    projectiles: Query<(Entity, &Transform, &ProjectileLogic, Option<&Payload>)>,
) {
    for (entity, transform, logic, payload) in projectiles.iter() {
        if let ProjectileLogic::Hitscan { range } = logic {
            let start = transform.translation;
            let direction = transform.forward(); // Assuming -Z is forward? No, usually Bevy forward is -Z. 
            // Transform::forward() returns Dir3 (-Z).
            
            // Filter out self (though hitscan usually spawned fresh)
            let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

            if let Some(hit) = spatial_query.cast_ray(
                start,
                direction,
                *range,
                true,
                &filter,
            ) {
                let hit_point = start + *direction * hit.distance;
                // We need to fetch surface? process_hit expects it.
                // We can try to query it? Or just pass None for now.
                // Since we don't have access to Surfaces query here easily without adding it to params.
                // Let's assume None for now or add the query.
                
                // Construct a dummy projectile component for process_hit
                // process_hit uses it for previous_position (not relevant for hitscan) and drag (not relevant).
                // But it takes &Projectile.
                let dummy_projectile = crate::components::Projectile::default();

                collision::process_hit(
                    &mut commands,
                    &mut hit_events,
                    &config,
                    entity,
                    &dummy_projectile,
                    payload,
                    hit.entity,
                    hit_point,
                    hit.normal,
                    None, // No surface info for now
                );
            }

            // Hitscan is instant, despawn immediately
            commands.entity(entity).despawn();
        }
    }
}

/// Calculate explosion damage with distance falloff.
/// 
/// Computes the damage at a given distance from an explosion center,
/// applying a power-based falloff function.
/// 
/// # Arguments
/// * `base_damage` - The maximum damage at the explosion center
/// * `distance` - The distance from the explosion center to the target
/// * `radius` - The maximum radius of the explosion effect
/// * `falloff` - The exponent controlling the rate of damage falloff
/// 
/// # Returns
/// The damage value at the specified distance
pub fn calculate_explosion_damage(
    base_damage: f32,
    distance: f32,
    radius: f32,
    falloff: f32,
) -> f32 {
    if distance >= radius {
        return 0.0;
    }

    let normalized_distance = distance / radius;
    let falloff_factor = (1.0 - normalized_distance).powf(falloff);

    base_damage * falloff_factor
}

/// Grenade presets for common throwable types.
pub mod presets {
    use super::*;

    /// Creates a fragmentation grenade preset.
    /// 
    /// This preset configures a timed explosive projectile with high damage
    /// and a medium blast radius, typical of military fragmentation grenades.
    /// 
    /// # Returns
    /// A tuple containing the ProjectileLogic and Payload for a frag grenade
    pub fn frag_grenade() -> (ProjectileLogic, Payload) {
        (
            ProjectileLogic::Timed {
                fuse: 3.0,
                elapsed: 0.0,
            },
            Payload::Explosive {
                damage: 150.0,
                radius: 10.0,
                falloff: 1.5,
            },
        )
    }

    /// Creates a flashbang grenade preset.
    /// 
    /// This preset configures a timed projectile that creates a blinding effect
    /// with a large radius but no direct damage, used for tactical advantage.
    /// 
    /// # Returns
    /// A tuple containing the ProjectileLogic and Payload for a flashbang
    pub fn flashbang() -> (ProjectileLogic, Payload) {
        (
            ProjectileLogic::Timed {
                fuse: 2.0,
                elapsed: 0.0,
            },
            Payload::Flash {
                intensity: 1.0,
                duration: 5.0,
                radius: 15.0,
            },
        )
    }

    /// Creates a smoke grenade preset.
    /// 
    /// This preset configures a timed projectile that creates a smoke screen
    /// for concealment, with a medium duration and radius.
    /// 
    /// # Returns
    /// A tuple containing the ProjectileLogic and Payload for a smoke grenade
    pub fn smoke_grenade() -> (ProjectileLogic, Payload) {
        (
            ProjectileLogic::Timed {
                fuse: 1.5,
                elapsed: 0.0,
            },
            Payload::Smoke {
                duration: 15.0,
                radius: 8.0,
            },
        )
    }

    /// Creates a molotov cocktail preset.
    /// 
    /// This preset configures an impact-triggered projectile that creates
    /// an incendiary effect with damage over time in a small area.
    /// 
    /// # Returns
    /// A tuple containing the ProjectileLogic and Payload for a molotov
    pub fn molotov() -> (ProjectileLogic, Payload) {
        (
            ProjectileLogic::Impact, // Breaks on impact
            Payload::Incendiary {
                duration: 8.0,
                damage_per_second: 15.0,
                radius: 5.0,
            },
        )
    }

    /// Creates a proximity mine preset.
    /// 
    /// This preset configures a proximity-triggered explosive device that
    /// detonates when targets come within its detection range.
    /// 
    /// # Returns
    /// A tuple containing the ProjectileLogic and Payload for a proximity mine
    pub fn proximity_mine() -> (ProjectileLogic, Payload) {
        (
            ProjectileLogic::Proximity { range: 2.0 },
            Payload::Explosive {
                damage: 200.0,
                radius: 5.0,
                falloff: 2.0,
            },
        )
    }
}

// ============================================================================
// Explosion Impulse System
// ============================================================================

/// Component marker for entities that can receive explosion impulse.
/// 
/// Add this component to entities (like players, physics objects) that should
/// be pushed by explosions.
#[derive(bevy::prelude::Component, Default)]
pub struct ExplosionAffected {
    /// Mass of the affected entity (affects impulse strength)
    pub mass: f32,
}

/// Apply physics impulse to nearby entities from explosions.
/// 
/// This system reads explosion events and applies outward impulse forces
/// to all entities with ExplosionAffected component within the blast radius.
/// Uses avian3d's LinearVelocity component for physics integration.
#[cfg(feature = "dim3")]
pub fn apply_explosion_impulse(
    mut explosion_events: MessageReader<ExplosionEvent>,
    mut affected_entities: Query<(Entity, &Transform, &ExplosionAffected, &mut avian3d::prelude::LinearVelocity)>,
) {
    for event in explosion_events.read() {
        // Base impulse strength (can be tuned per explosion type)
        let base_impulse = match event.explosion_type {
            crate::events::ExplosionType::HighExplosive => 30.0,
            crate::events::ExplosionType::Incendiary => 5.0,
            crate::events::ExplosionType::Flash => 2.0,
            crate::events::ExplosionType::Smoke => 0.5,
            crate::events::ExplosionType::Fragmentation => 25.0,
            crate::events::ExplosionType::Concussion => 50.0,
            crate::events::ExplosionType::EMP => 0.0,
        };

        if base_impulse <= 0.0 {
            continue;
        }

        for (entity, transform, affected, mut velocity) in affected_entities.iter_mut() {
            // Skip if this entity is the explosion source
            if Some(entity) == event.source {
                continue;
            }

            let to_entity = transform.translation - event.center;
            let distance = to_entity.length();

            // Skip if outside blast radius
            if distance >= event.radius || distance < 0.01 {
                continue;
            }

            // Calculate impulse with distance falloff
            let direction = to_entity.normalize();
            let normalized_distance = distance / event.radius;
            let falloff_factor = (1.0 - normalized_distance).powf(event.falloff);
            
            // Impulse inversely proportional to mass
            let mass_factor = if affected.mass > 0.0 { 1.0 / affected.mass } else { 1.0 };
            let impulse_magnitude = base_impulse * falloff_factor * mass_factor;
            
            // Add upward component for more interesting physics
            let impulse_direction = (direction + Vec3::Y * 0.3).normalize();
            let impulse = impulse_direction * impulse_magnitude;

            // Apply impulse to velocity
            velocity.0 += impulse;
        }
    }
}

/// Fallback when dim3 is not available
#[cfg(not(feature = "dim3"))]
pub fn apply_explosion_impulse(
    mut _explosion_events: MessageReader<ExplosionEvent>,
) {
    // No physics without dim3 feature
    for _ in _explosion_events.read() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explosion_damage_at_center() {
        let damage = calculate_explosion_damage(100.0, 0.0, 10.0, 1.0);
        assert_eq!(damage, 100.0);
    }

    #[test]
    fn test_explosion_damage_at_edge() {
        let damage = calculate_explosion_damage(100.0, 10.0, 10.0, 1.0);
        assert_eq!(damage, 0.0);
    }

    #[test]
    fn test_explosion_damage_falloff() {
        // Linear falloff (1.0)
        let damage_half = calculate_explosion_damage(100.0, 5.0, 10.0, 1.0);
        assert!((damage_half - 50.0).abs() < 0.01);

        // Quadratic falloff (2.0) - less damage at same distance
        let damage_quad = calculate_explosion_damage(100.0, 5.0, 10.0, 2.0);
        assert!(damage_quad < damage_half);
    }
}

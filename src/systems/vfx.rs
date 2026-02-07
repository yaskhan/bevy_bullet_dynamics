//! VFX system - tracers, decals, and impact effects with object pooling.

use bevy::prelude::*;
use bevy::ecs::message::MessageReader;

use crate::components::{BulletTracer, HitEffectType, ImpactDecal};
use crate::events::HitEvent;
use crate::resources::{DecalPool, TracerPool};

/// Update tracer lifetimes and hide expired ones.
/// 
/// This system updates the lifetime of bullet tracers and returns them to the pool
/// when they expire, rather than despawning them to improve performance.
/// 
/// # Arguments
/// * `_commands` - Bevy Commands for entity manipulation (currently unused in this function)
/// * `time` - Bevy Time resource to get delta time
/// * `pool` - Mutable reference to the tracer pool resource
/// * `tracers` - Query for tracer entities and their components
pub fn update_tracers(
    _commands: Commands,
    time: Res<Time>,
    mut pool: ResMut<TracerPool>,
    mut tracers: Query<(Entity, &mut BulletTracer, &mut Visibility)>,
) {
    let dt = time.delta_secs();

    for (entity, mut tracer, mut visibility) in tracers.iter_mut() {
        tracer.lifetime -= dt;

        if tracer.lifetime <= 0.0 {
            // Return to pool instead of despawning
            *visibility = Visibility::Hidden;
            pool.release(entity);
        }
    }
}

/// Spawn impact effects at hit locations.
/// 
/// This system listens for hit events and spawns appropriate visual effects
/// at the impact location based on the surface material and hit type.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for spawning entities
/// * `hit_events` - Event reader for hit events
pub fn spawn_impact_effects(
    mut commands: Commands,
    mut hit_events: MessageReader<HitEvent>,
    // Asset handles would go here for actual VFX
) {
    for event in hit_events.read() {
        // Spawn appropriate effect based on hit type
        // This is a placeholder - actual implementation would spawn particles/meshes

        let effect_type = HitEffectType::Sparks; // Would come from surface material

        spawn_hit_effect(
            &mut commands,
            event.impact_point,
            event.normal,
            effect_type,
        );
    }
}

/// Cleanup expired visual effects.
/// 
/// This system updates the lifetime of impact decals and returns them to the pool
/// when they expire, rather than despawning them to improve performance.
/// 
/// # Arguments
/// * `_commands` - Bevy Commands for entity manipulation (currently unused in this function)
/// * `time` - Bevy Time resource to get delta time
/// * `pool` - Mutable reference to the decal pool resource
/// * `decals` - Query for decal entities and their components
pub fn cleanup_expired_effects(
    _commands: Commands,
    time: Res<Time>,
    mut pool: ResMut<DecalPool>,
    mut decals: Query<(Entity, &mut ImpactDecal, &mut Visibility)>,
) {
    let dt = time.delta_secs();

    for (entity, mut decal, mut visibility) in decals.iter_mut() {
        decal.lifetime -= dt;

        if decal.lifetime <= 0.0 {
            *visibility = Visibility::Hidden;
            pool.release(entity);
        }
    }
}

/// Spawn a hit effect at the impact location.
/// 
/// This function spawns a visual effect at the specified location based on the
/// type of hit effect requested.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for spawning entities
/// * `meshes` - Asset storage for meshes
/// * `materials` - Asset storage for materials
/// * `position` - World-space position where the effect should appear
/// * `normal` - Surface normal vector for orienting the effect
/// * `effect_type` - Type of visual effect to spawn
pub fn spawn_hit_effect_with_assets(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    normal: Vec3,
    effect_type: HitEffectType,
) {
    let rotation = Quat::from_rotation_arc(Vec3::Y, normal);
    
    // Create effect based on type
    let (color, emissive, size) = match effect_type {
        HitEffectType::Sparks => (
            Color::srgb(1.0, 0.7, 0.2),
            LinearRgba::rgb(5.0, 3.0, 0.5),
            0.05,
        ),
        HitEffectType::Dust => (
            Color::srgba(0.6, 0.5, 0.4, 0.8),
            LinearRgba::NONE,
            0.15,
        ),
        HitEffectType::Blood => (
            Color::srgb(0.5, 0.0, 0.0),
            LinearRgba::rgb(0.3, 0.0, 0.0),
            0.1,
        ),
        HitEffectType::WoodChips => (
            Color::srgb(0.6, 0.4, 0.2),
            LinearRgba::NONE,
            0.08,
        ),
        HitEffectType::Water => (
            Color::srgba(0.4, 0.6, 0.9, 0.6),
            LinearRgba::rgb(0.2, 0.3, 0.5),
            0.12,
        ),
        HitEffectType::Glass => (
            Color::srgba(0.9, 0.95, 1.0, 0.5),
            LinearRgba::rgb(0.5, 0.6, 0.8),
            0.06,
        ),
    };

    // Spawn impact effect sphere
    let mesh = meshes.add(Sphere::new(size));
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive,
        alpha_mode: if color.alpha() < 1.0 { 
            AlphaMode::Blend 
        } else { 
            AlphaMode::Opaque 
        },
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position + normal * 0.01)
            .with_rotation(rotation),
        ImpactDecal { lifetime: 0.5 }, // Short-lived effect
    ));
}

/// Simple spawn_hit_effect for use without asset access (placeholder).
fn spawn_hit_effect(
    _commands: &mut Commands,
    _position: Vec3,
    _normal: Vec3,
    _effect_type: HitEffectType,
) {
    // Placeholder - use spawn_hit_effect_with_assets for real effects
}

/// Spawn a bullet tracer with actual mesh from pool or create new.
/// 
/// This function creates a visible tracer effect using a stretched mesh.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for spawning entities
/// * `meshes` - Asset storage for meshes
/// * `materials` - Asset storage for materials
/// * `pool` - Mutable reference to the tracer pool
/// * `origin` - World-space position where the tracer should start
/// * `direction` - Direction vector for the tracer's movement
/// * `speed` - Speed of the tracer in meters per second
/// * `settings` - Tracer visual settings
/// 
/// # Returns
/// The Entity ID of the spawned tracer
pub fn spawn_tracer_with_assets(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pool: &mut TracerPool,
    origin: Vec3,
    direction: Vec3,
    speed: f32,
    settings: &tracer_config::TracerSettings,
) -> Entity {
    let lifetime = settings.length / speed * 10.0;
    
    // Create stretched cylinder mesh for tracer
    let mesh = meshes.add(Cylinder::new(settings.width, settings.length));
    
    // Create glowing material
    let emissive_strength = settings.glow_intensity * 3.0;
    let material = materials.add(StandardMaterial {
        base_color: settings.color,
        emissive: LinearRgba::from(settings.color) * emissive_strength,
        unlit: true, // Tracers should glow
        ..default()
    });

    if let Some(entity) = pool.get() {
        // Reuse pooled tracer
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(origin).looking_to(direction, Vec3::Y),
            Visibility::Visible,
            BulletTracer {
                lifetime,
                trail_length: settings.length,
            },
        ));
        entity
    } else {
        // Create new tracer
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(origin).looking_to(direction, Vec3::Y),
                Visibility::Visible,
                BulletTracer {
                    lifetime,
                    trail_length: settings.length,
                },
            ))
            .id()
    }
}

/// Spawn a bullet tracer from pool or create new (simple version).
/// 
/// This function attempts to reuse a tracer from the pool, or creates a new one
/// if the pool is empty. This helps improve performance by reducing allocations.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for spawning entities
/// * `pool` - Mutable reference to the tracer pool
/// * `origin` - World-space position where the tracer should start
/// * `direction` - Direction vector for the tracer's movement
/// * `speed` - Speed of the tracer in meters per second
/// * `_color` - Color of the tracer effect (currently unused in this function)
/// 
/// # Returns
/// The Entity ID of the spawned tracer
pub fn spawn_tracer(
    commands: &mut Commands,
    pool: &mut TracerPool,
    origin: Vec3,
    direction: Vec3,
    speed: f32,
    _color: Color,
) -> Entity {
    let tracer_length = 2.0; // meters
    let lifetime = tracer_length / speed * 10.0; // Time visible

    if let Some(entity) = pool.get() {
        // Reuse pooled tracer
        commands.entity(entity).insert((
            Transform::from_translation(origin).looking_to(direction, Vec3::Y),
            Visibility::Visible,
            BulletTracer {
                lifetime,
                trail_length: tracer_length,
            },
        ));
        entity
    } else {
        // Create new tracer
        commands
            .spawn((
                Transform::from_translation(origin).looking_to(direction, Vec3::Y),
                Visibility::Visible,
                BulletTracer {
                    lifetime,
                    trail_length: tracer_length,
                },
            ))
            .id()
    }
}

/// Spawn an impact decal from pool or create new.
/// 
/// This function attempts to reuse a decal from the pool, or creates a new one
/// if the pool is empty. This helps improve performance by reducing allocations.
/// 
/// # Arguments
/// * `commands` - Bevy Commands for spawning entities
/// * `pool` - Mutable reference to the decal pool
/// * `position` - World-space position where the decal should appear
/// * `normal` - Surface normal vector for orienting the decal
/// * `size` - Size scale of the decal
/// * `lifetime` - Duration in seconds before the decal expires
/// 
/// # Returns
/// The Entity ID of the spawned decal
pub fn spawn_decal(
    commands: &mut Commands,
    pool: &mut DecalPool,
    position: Vec3,
    normal: Vec3,
    size: f32,
    lifetime: f32,
) -> Entity {
    let rotation = Quat::from_rotation_arc(Vec3::Y, normal);

    if let Some(entity) = pool.get() {
        // Reuse pooled decal
        commands.entity(entity).insert((
            Transform::from_translation(position)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(size)),
            Visibility::Visible,
            ImpactDecal { lifetime },
        ));
        entity
    } else {
        // Create new decal
        commands
            .spawn((
                Transform::from_translation(position)
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(size)),
                Visibility::Visible,
                ImpactDecal { lifetime },
            ))
            .id()
    }
}

/// VFX configuration for different weapon types.
pub mod tracer_config {
    use super::*;

    /// Configuration settings for bullet tracer visual effects.
    /// 
    /// This struct defines the visual properties of bullet tracers,
    /// allowing for customization based on weapon type.
    /// 
    /// # Fields
    /// * `color` - The color of the tracer effect
    /// * `width` - The visual width of the tracer
    /// * `length` - The length of the tracer effect
    /// * `glow_intensity` - The intensity of the tracer's glow effect
    pub struct TracerSettings {
        pub color: Color,
        pub width: f32,
        pub length: f32,
        pub glow_intensity: f32,
    }

    impl Default for TracerSettings {
        /// Creates a default TracerSettings instance with yellow-orange color.
        /// 
        /// # Returns
        /// A new TracerSettings instance with default values
        fn default() -> Self {
            Self {
                color: Color::srgb(1.0, 0.9, 0.3),
                width: 0.02,
                length: 2.0,
                glow_intensity: 1.0,
            }
        }
    }

    /// Creates tracer settings suitable for rifles.
    /// 
    /// Rifle tracers are typically bright yellow-orange with moderate length.
    /// 
    /// # Returns
    /// A TracerSettings instance configured for rifles
    pub fn rifle() -> TracerSettings {
        TracerSettings {
            color: Color::srgb(1.0, 0.8, 0.2),
            width: 0.015,
            length: 3.0,
            glow_intensity: 0.8,
        }
    }

    /// Creates tracer settings suitable for sniper rifles.
    /// 
    /// Sniper tracers are typically white/blue with longer length and higher intensity.
    /// 
    /// # Returns
    /// A TracerSettings instance configured for sniper rifles
    pub fn sniper() -> TracerSettings {
        TracerSettings {
            color: Color::srgb(0.9, 0.9, 1.0),
            width: 0.01,
            length: 5.0,
            glow_intensity: 1.2,
        }
    }

    /// Creates tracer settings suitable for submachine guns (SMGs).
    /// 
    /// SMG tracers are typically orange-red with shorter length and lower intensity.
    /// 
    /// # Returns
    /// A TracerSettings instance configured for SMGs
    pub fn smg() -> TracerSettings {
        TracerSettings {
            color: Color::srgb(1.0, 0.7, 0.1),
            width: 0.02,
            length: 1.5,
            glow_intensity: 0.6,
        }
    }

    /// Creates tracer settings suitable for laser effects.
    /// 
    /// Laser tracers are typically red with very long length and high intensity.
    /// 
    /// # Returns
    /// A TracerSettings instance configured for laser effects
    pub fn laser() -> TracerSettings {
        TracerSettings {
            color: Color::srgb(1.0, 0.0, 0.0),
            width: 0.005,
            length: 100.0,
            glow_intensity: 2.0,
        }
    }
}

// ============================================================================
// Muzzle Flash System
// ============================================================================

use crate::components::{MuzzleFlash, ExplosionVFX};
use crate::events::ExplosionEvent;

/// Update muzzle flash lifetimes and fade them out.
/// 
/// This system updates the lifetime of muzzle flashes and fades them out
/// as they approach zero, then despawns them.
pub fn update_muzzle_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut MuzzleFlash, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut flash, mut transform) in flashes.iter_mut() {
        flash.lifetime -= dt;

        if flash.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Scale down as lifetime decreases
            let scale_factor = flash.lifetime / 0.05; // Assuming 0.05s base lifetime
            transform.scale = Vec3::splat(flash.scale * scale_factor.min(1.0));
        }
    }
}

/// Spawn muzzle flash effect at weapon position.
/// 
/// Creates a glowing sphere effect at the muzzle position.
pub fn spawn_muzzle_flash(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    direction: Vec3,
    intensity: f32,
    scale: f32,
) -> Entity {
    let mesh = meshes.add(Sphere::new(scale));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.5),
        emissive: LinearRgba::rgb(intensity * 5.0, intensity * 4.0, intensity * 1.0),
        unlit: true,
        ..default()
    });

    let rotation = Quat::from_rotation_arc(Vec3::Z, direction);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(scale)),
        MuzzleFlash {
            lifetime: 0.05,
            intensity,
            scale,
        },
    )).id()
}

// ============================================================================
// Explosion VFX System
// ============================================================================

/// Update explosion visual effects.
/// 
/// This system updates explosion effects, expanding them and fading them out.
pub fn update_explosion_vfx(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(Entity, &mut ExplosionVFX, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut explosion, mut transform) in explosions.iter_mut() {
        explosion.lifetime -= dt;

        if explosion.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Expand explosion over time
            let progress = 1.0 - (explosion.lifetime / 1.0); // Assuming 1s base lifetime
            explosion.current_radius = explosion.max_radius * progress.min(1.0);
            transform.scale = Vec3::splat(explosion.current_radius);
        }
    }
}

/// Spawn explosion visual effect from explosion event.
pub fn spawn_explosion_vfx_from_event(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut explosion_events: MessageReader<ExplosionEvent>,
) {
    for event in explosion_events.read() {
        let (color, size_mult, lifetime, emissive_mult) = match event.explosion_type {
            crate::events::ExplosionType::HighExplosive => (Color::srgb(1.0, 0.5, 0.0), 1.0, 0.5, 5.0),
            crate::events::ExplosionType::Incendiary => (Color::srgb(1.0, 0.2, 0.0), 1.0, 2.0, 2.0),
            crate::events::ExplosionType::Flash => (Color::WHITE, 2.0, 0.1, 20.0),
            crate::events::ExplosionType::Smoke => (Color::srgb(0.5, 0.5, 0.5), 1.5, 5.0, 0.0),
            _ => (Color::srgb(1.0, 1.0, 0.0), 1.0, 1.0, 1.0),
        };

        spawn_explosion_vfx_typed(
            &mut commands,
            &mut meshes,
            &mut materials,
            event.center,
            event.radius * size_mult,
            10.0 * emissive_mult,
            color,
            lifetime,
        );
    }
}

/// Spawn explosion visual effect at position with custom props.
pub fn spawn_explosion_vfx_typed(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    radius: f32,
    intensity: f32,
    color: Color,
    lifetime: f32,
) -> Entity {
    let mesh = meshes.add(Sphere::new(1.0));
    
    // Create fiery material
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive: LinearRgba::from(color) * intensity,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position)
            .with_scale(Vec3::splat(0.1)), // Start small
        ExplosionVFX {
            lifetime,
            max_radius: radius,
            current_radius: 0.1,
            intensity,
        },
    )).id()
}


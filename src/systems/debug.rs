use bevy::prelude::*;
use crate::components::Projectile;

/// Draw debug gizmos for projectiles.
///
/// Draws velocity vectors and positions for active projectiles.
pub fn draw_projectile_debug(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Projectile)>,
    config: Res<crate::resources::BallisticsConfig>,
) {
    if !config.debug_draw {
        return;
    }

    for (transform, projectile) in query.iter() {
        // Draw projectile point
        gizmos.sphere(transform.translation, 0.05, Color::srgb(1.0, 0.0, 0.0));
        
        
        // Draw velocity vector
        let end = transform.translation + projectile.velocity * 0.1; // Scale down for visibility
        gizmos.line(transform.translation, end, Color::srgb(0.0, 1.0, 0.0));
    }
}

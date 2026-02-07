//! Kinematics system - RK4 and Euler integration for projectile physics.

use bevy::prelude::*;

use crate::components::Projectile;
use crate::resources::{BallisticsConfig, BallisticsEnvironment};

/// Update projectile positions using physics integration.
///
/// Runs in FixedUpdate for deterministic simulation.
/// Supports both RK4 (accurate) and Euler (fast) integration methods.
/// 
/// # Arguments
/// * `time` - Bevy FixedTime resource to get delta time
/// * `env` - Ballistics environment resource with physics parameters
/// * `config` - Ballistics configuration resource
/// * `query` - Query for transform and projectile components to update
pub fn update_projectiles_kinematics(
    time: Res<Time<Fixed>>,
    env: Res<BallisticsEnvironment>,
    config: Res<BallisticsConfig>,
    mut query: Query<(&mut Transform, &mut Projectile)>,
) {
    let dt = time.delta_secs();
    let effective_density = env.effective_air_density();

    query.par_iter_mut().for_each(|(mut transform, mut bullet)| {
        // Store previous position for collision detection
        bullet.previous_position = transform.translation;

        if config.use_rk4 {
            // RK4 Integration - More accurate
            integrate_rk4(&mut transform, &mut bullet, dt, &env, effective_density);
        } else {
            // Euler Integration - Simpler, faster
            integrate_euler(&mut transform, &mut bullet, dt, &env, effective_density);
        }

        // Update transform rotation to face velocity direction
        if bullet.velocity.length_squared() > 0.001 {
            transform.look_to(bullet.velocity.normalize(), Vec3::Y);
        }
    });
}

/// RK4 (Runge-Kutta 4th order) integration step.
/// 
/// Performs a 4th-order Runge-Kutta integration step to accurately compute
/// the next position and velocity of a projectile based on its acceleration.
/// 
/// # Arguments
/// * `transform` - Mutable reference to the transform component to update
/// * `bullet` - Mutable reference to the projectile component
/// * `dt` - Time step for the integration
/// * `env` - Reference to the ballistics environment
/// * `air_density` - Effective air density for drag calculations
fn integrate_rk4(
    transform: &mut Transform,
    bullet: &mut Projectile,
    dt: f32,
    env: &BallisticsEnvironment,
    air_density: f32,
) {
    let pos = transform.translation;
    let vel = bullet.velocity;

    // RK4 coefficients for acceleration
    let k1 = calculate_acceleration(bullet, vel, env, air_density);
    let k2 = calculate_acceleration(bullet, vel + k1 * (dt / 2.0), env, air_density);
    let k3 = calculate_acceleration(bullet, vel + k2 * (dt / 2.0), env, air_density);
    let k4 = calculate_acceleration(bullet, vel + k3 * dt, env, air_density);

    // Weighted average of acceleration
    let final_accel = (k1 + k2 * 2.0 + k3 * 2.0 + k4) / 6.0;

    // Update velocity and position
    bullet.velocity += final_accel * dt;
    transform.translation = pos + bullet.velocity * dt;
}

/// Simple Euler integration step.
/// 
/// Performs a simple Euler integration step to compute the next position
/// and velocity of a projectile based on its acceleration. Less accurate
/// than RK4 but computationally cheaper.
/// 
/// # Arguments
/// * `transform` - Mutable reference to the transform component to update
/// * `bullet` - Mutable reference to the projectile component
/// * `dt` - Time step for the integration
/// * `env` - Reference to the ballistics environment
/// * `air_density` - Effective air density for drag calculations
fn integrate_euler(
    transform: &mut Transform,
    bullet: &mut Projectile,
    dt: f32,
    env: &BallisticsEnvironment,
    air_density: f32,
) {
    let accel = calculate_acceleration(bullet, bullet.velocity, env, air_density);
    bullet.velocity += accel * dt;
    transform.translation += bullet.velocity * dt;
}

/// Calculate acceleration on projectile from gravity and aerodynamic drag.
///
/// Uses the drag equation: F_drag = 0.5 * ρ * v² * Cd * A
/// 
/// # Arguments
/// * `bullet` - Reference to the projectile component
/// * `vel` - Current velocity vector of the projectile
/// * `env` - Reference to the ballistics environment
/// * `air_density` - Effective air density for drag calculations
/// 
/// # Returns
/// The acceleration vector acting on the projectile
fn calculate_acceleration(
    bullet: &Projectile,
    vel: Vec3,
    env: &BallisticsEnvironment,
    air_density: f32,
) -> Vec3 {
    // Velocity relative to air (accounting for wind)
    let relative_vel = vel - env.wind;
    let speed = relative_vel.length();

    // Avoid division by zero for stationary projectiles
    if speed < 0.001 {
        return env.gravity;
    }

    let direction = relative_vel.normalize();

    // Drag force magnitude: 0.5 * ρ * v² * Cd * A
    let drag_magnitude =
        0.5 * air_density * speed.powi(2) * bullet.drag_coefficient * bullet.reference_area;

    // Drag acceleration = F_drag / mass (opposite to velocity direction)
    let drag_accel = direction * (drag_magnitude / bullet.mass);

    // Total acceleration = gravity - drag
    env.gravity - drag_accel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_calculation() {
        let bullet = Projectile {
            velocity: Vec3::new(400.0, 0.0, 0.0),
            mass: 0.01,
            drag_coefficient: 0.3,
            reference_area: 0.0001,
            ..Default::default()
        };

        let env = BallisticsEnvironment::default();
        let accel = calculate_acceleration(&bullet, bullet.velocity, &env, env.air_density);

        // Should have downward gravity component
        assert!(accel.y < 0.0);
        // Should have drag opposing velocity (negative X)
        assert!(accel.x < 0.0);
    }

    #[test]
    fn test_stationary_projectile() {
        let bullet = Projectile {
            velocity: Vec3::ZERO,
            mass: 0.01,
            ..Default::default()
        };

        let env = BallisticsEnvironment::default();
        let accel = calculate_acceleration(&bullet, bullet.velocity, &env, env.air_density);

        // Only gravity should apply
        assert_eq!(accel, env.gravity);
    }
}

//! Benchmark for projectile physics performance.

use bevy::prelude::*;
use bevy_bullet_dynamics::components::Projectile;
use bevy_bullet_dynamics::resources::BallisticsEnvironment;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_rk4_integration(c: &mut Criterion) {
    let env = BallisticsEnvironment::default();
    let air_density = env.effective_air_density();

    let mut group = c.benchmark_group("RK4 Integration");

    for projectile_count in [100, 1000, 10000].iter() {
        let projectiles: Vec<Projectile> = (0..*projectile_count)
            .map(|i| Projectile {
                velocity: Vec3::new(400.0 + i as f32, 0.0, 0.0),
                mass: 0.01,
                drag_coefficient: 0.3,
                reference_area: 0.0001,
                ..Default::default()
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(projectile_count),
            projectile_count,
            |b, &_count| {
                b.iter(|| {
                    let dt = 1.0 / 60.0;
                    for projectile in &projectiles {
                        let vel = projectile.velocity;

                        // RK4 coefficients
                        let k1 = calculate_acceleration(projectile, vel, &env, air_density);
                        let k2 = calculate_acceleration(projectile, vel + k1 * (dt / 2.0), &env, air_density);
                        let k3 = calculate_acceleration(projectile, vel + k2 * (dt / 2.0), &env, air_density);
                        let k4 = calculate_acceleration(projectile, vel + k3 * dt, &env, air_density);

                        let _final_accel = (k1 + k2 * 2.0 + k3 * 2.0 + k4) / 6.0;
                    }
                });
            },
        );
    }

    group.finish();
}

fn calculate_acceleration(
    bullet: &Projectile,
    vel: Vec3,
    env: &BallisticsEnvironment,
    air_density: f32,
) -> Vec3 {
    let relative_vel = vel - env.wind;
    let speed = relative_vel.length();

    if speed < 0.001 {
        return env.gravity;
    }

    let direction = relative_vel.normalize();
    let drag_magnitude =
        0.5 * air_density * speed.powi(2) * bullet.drag_coefficient * bullet.reference_area;
    let drag_accel = direction * (drag_magnitude / bullet.mass);

    env.gravity - drag_accel
}

fn benchmark_spread_calculation(c: &mut Criterion) {
    use bevy_bullet_dynamics::components::Accuracy;
    use bevy_bullet_dynamics::systems::accuracy;

    let accuracy_preset = accuracy::presets::rifle();

    c.bench_function("Spread Calculation", |b| {
        b.iter(|| {
            accuracy::calculate_total_spread(
                &accuracy_preset,
                false,
                true,
                false,
                3.0,
                5.0,
            )
        });
    });

    c.bench_function("Apply Spread to Direction", |b| {
        let direction = Vec3::NEG_Z;
        let spread_angle = 0.01;

        b.iter(|| {
            accuracy::apply_spread_to_direction(direction, spread_angle, 12345);
        });
    });
}

criterion_group!(benches, benchmark_rk4_integration, benchmark_spread_calculation);
criterion_main!(benches);

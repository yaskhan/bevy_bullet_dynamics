use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy_bullet_dynamics::prelude::*;
use std::time::Duration;

fn main() {
    println!("Starting Headless Ballistics Simulation...");
    println!("Tests will run for 200 physics ticks (approx 3.3 seconds)...");

    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))))
        .add_plugins(AssetPlugin::default()) // Needed for internal resources if any
        .add_plugins(BallisticsCorePlugin)
        .add_plugins(BallisticsSurfacePlugin)
        // Skip VFX and Debug plugins (headless)
        .add_systems(Startup, setup_simulation)
        .add_systems(Update, print_progress)
        .add_systems(FixedUpdate, check_projectile_status)
        .run();
}

#[derive(Component)]
struct TestTracker {
    start_pos: Vec3,
    expected_drop_min: f32,
    has_checked_100m: bool,
}

fn setup_simulation(mut commands: Commands) {
    println!("\n[SETUP] Spawning test projectile...");
    
    // 1. Standard Bullet (Supersonic)
    let velocity = Vec3::new(0.0, 0.0, 800.0); // 800 m/s forward (Z)
    commands.spawn((
        Projectile::new(velocity),
        Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
        TestTracker { 
            start_pos: Vec3::new(0.0, 10.0, 0.0), 
            expected_drop_min: 0.05,
            has_checked_100m: false,
        },
        Name::new("Test Bullet 800m/s"),
    ));

    // 2. Heavy/Slow Bullet (Subsonic)
    let velocity_sub = Vec3::new(0.0, 0.0, 300.0);
    commands.spawn((
        Projectile::new(velocity_sub),
        Transform::from_translation(Vec3::new(5.0, 10.0, 0.0)),
        TestTracker { 
            start_pos: Vec3::new(5.0, 10.0, 0.0), 
            expected_drop_min: 0.2,
            has_checked_100m: false,
        },
        Name::new("Subsonic Bullet 300m/s"),
    ));
    
    // Setup Resources
    commands.insert_resource(Time::<Fixed>::from_hz(60.0));
}

fn print_progress(time: Res<Time>, mut timer: Local<f32>) {
    *timer += time.delta_secs();
    if *timer > 1.0 {
        *timer = 0.0;
        println!("[INFO] Simulation running... (Time: {:.1}s)", time.elapsed_secs());
    }
    
    // Auto-quit after 5 seconds
    if time.elapsed_secs() > 5.0 {
        println!("[FINISHED] Simulation complete.");
        std::process::exit(0);
    }
}

fn check_projectile_status(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Projectile, &mut TestTracker, &Name)>,
) {
    for (entity, transform, projectile, mut tracker, name) in query.iter_mut() {
        let dist_z = (transform.translation.z - tracker.start_pos.z).abs();
        let drop = tracker.start_pos.y - transform.translation.y;
        
        // Print status at 100m checkpoint (now handles fast projectiles by checking once after 100m)
        if !tracker.has_checked_100m && dist_z >= 100.0 {
             println!("[CHECKPOINT] {} at {:.1}m: Drop = {:.4}m, Velocity = {:.1} m/s", 
                name, dist_z, drop, projectile.velocity.length());
             
             if drop >= tracker.expected_drop_min {
                 println!("[PASS] {} Drop check passed (Drop {:.4}m >= Expected {:.4}m)", name, drop, tracker.expected_drop_min);
             } else {
                 println!("[FAIL] {} Drop check FAILED (Drop {:.4}m < Expected {:.4}m)", name, drop, tracker.expected_drop_min);
             }
             
             tracker.has_checked_100m = true;
        }
        
        if transform.translation.y < 0.0 {
            println!("[IMPACT] {} hit ground at {:.1}m distance.", name, dist_z);
            commands.entity(entity).despawn();
        }
    }
}

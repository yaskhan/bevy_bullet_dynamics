use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;

#[cfg(feature = "dim2")]
use avian2d::prelude::*;
#[cfg(feature = "dim3")]
use avian3d::prelude::*;

const PLAYER_SPEED: f32 = 300.0;
const ROTATION_SPEED: f32 = 3.0;
const BULLET_SPEED: f32 = 800.0;
const ACTUAL_WALL_DISTANCE: f32 = 400.0; // Distance from -200 to 200

#[derive(Resource)]
struct SimulationSettings {
    virtual_range: f32, // Target distance to simulate (m)
    base_gravity: Vec3,
    base_air_density: f32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct StatsText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins({
            #[cfg(feature = "dim2")] { PhysicsPlugins::default() }
            #[cfg(feature = "dim3")] { PhysicsPlugins::default() }
            #[cfg(not(any(feature = "dim2", feature = "dim3")))] { MinimalPlugins }
        })
        .add_plugins(BallisticsPluginGroup)
        .insert_resource(SimulationSettings {
            virtual_range: ACTUAL_WALL_DISTANCE,
            base_gravity: Vec3::new(0.0, -9.81, 0.0), // Standard gravity
            base_air_density: 1.225,                  // Sea level
        })
        .insert_resource(BallisticsEnvironment::default())
        .insert_resource(BallisticsConfig {
            use_rk4: true,
            max_projectile_lifetime: 10.0,
            max_projectile_distance: 5000.0,
            enable_penetration: true,
            enable_ricochet: true,
            min_projectile_speed: 20.0,
            debug_draw: true,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_controls,
                simulation_controls,
                update_environment,
                update_ui,
                spawn_visual_effects,
                cleanup_projectiles,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);

    // Player
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.8, 1.0),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..default()
        },
        Player,
        Transform::from_xyz(-200.0, 0.0, 0.0),
    ));

    // Wall (Target)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(20.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(200.0, 0.0, 0.0),
        #[cfg(any(feature = "dim2", feature = "dim3"))]
        RigidBody::Static,
        #[cfg(feature = "dim2")]
        Collider::rectangle(20.0, 200.0),
        #[cfg(feature = "dim3")]
        Collider::cuboid(20.0, 200.0, 10.0),
        bevy_bullet_dynamics::systems::surface::materials::concrete(),
    ));

    // UI
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        StatsText,
    ));
}

fn player_controls(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut fire_events: MessageWriter<FireEvent>,
    time: Res<Time>,
) {
    let mut transform = match player_query.iter_mut().next() {
        Some(t) => t,
        None => return,
    };

    // Rotation
    if keyboard_input.pressed(KeyCode::KeyA) {
        transform.rotate_z(ROTATION_SPEED * time.delta_secs());
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        transform.rotate_z(-ROTATION_SPEED * time.delta_secs());
    }

    // Movement
    let forward = transform.up(); // In 2D with Sprite, 'up' is the direction it's pointing
    if keyboard_input.pressed(KeyCode::KeyW) {
        transform.translation += forward * PLAYER_SPEED * time.delta_secs();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        transform.translation -= forward * PLAYER_SPEED * time.delta_secs();
    }

    // Shooting
    if keyboard_input.just_pressed(KeyCode::Space) {
        let origin = transform.translation + forward * 20.0;
        let direction = forward.as_vec3();

        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.8, 0.2),
                custom_size: Some(Vec2::new(6.0, 2.0)),
                ..default()
            },
            Transform::from_translation(origin).with_rotation(transform.rotation),
            Projectile::new(direction * BULLET_SPEED),
            Accuracy::default(),
            Payload::Kinetic { damage: 50.0 },
            ProjectileLogic::Impact,
        ));

        fire_events.write(FireEvent::new(origin, direction, BULLET_SPEED));
    }
}

fn simulation_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<SimulationSettings>,
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        settings.virtual_range += 10.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        settings.virtual_range = (settings.virtual_range - 10.0).max(ACTUAL_WALL_DISTANCE);
    }
}

fn update_environment(
    settings: Res<SimulationSettings>,
    mut env: ResMut<BallisticsEnvironment>,
) {
    let factor = settings.virtual_range / ACTUAL_WALL_DISTANCE;
    
    // Scale gravity (squared factor because of t^2 in drop formula)
    env.gravity = settings.base_gravity * factor * factor;
    
    // Scale air density (linear factor for drag simulation)
    // Note: This is an approximation of "flying longer"
    env.air_density = settings.base_air_density * factor;
}

fn update_ui(
    settings: Res<SimulationSettings>,
    mut ui_query: Query<&mut Text, With<StatsText>>,
) {
    let Some(mut text) = ui_query.iter_mut().next() else { return };
    text.0 = format!(
        "Controls: W/S Move, A/D Rotate, Space Shoot\n\
         Up/Down: Adjust Virtual Distance\n\n\
         Visual Distance: {:.0}m\n\
         SIMULATED RANGE: {:.0}m\n\
         Effective Gravity: {:.2} m/s²\n\
         Effective Air Density: {:.3} kg/m³",
        ACTUAL_WALL_DISTANCE,
        settings.virtual_range,
        settings.base_gravity.y * (settings.virtual_range / ACTUAL_WALL_DISTANCE).powi(2),
        settings.base_air_density * (settings.virtual_range / ACTUAL_WALL_DISTANCE)
    );
}

fn spawn_visual_effects(
    mut commands: Commands,
    mut hit_events: MessageReader<HitEvent>,
) {
    for event in hit_events.read() {
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.2, 0.2),
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..default()
            },
            Transform::from_translation(event.impact_point),
            TemporaryEffect { lifetime: 0.8 },
        ));
    }
}

#[derive(Component)]
struct TemporaryEffect {
    lifetime: f32,
}

fn cleanup_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TemporaryEffect)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.lifetime -= time.delta_secs();
        if effect.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
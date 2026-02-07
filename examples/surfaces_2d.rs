//! 2D surface interaction example with penetration and ricochets.

use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;
use avian2d::prelude::*;

const PLAYER_SPEED: f32 = 200.0;
const BULLET_SPEED: f32 = 600.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(BallisticsPluginGroup)
        .insert_resource(BallisticsEnvironment {
            gravity: Vec3::ZERO,
            air_density: 1.2,
            wind: Vec3::ZERO,
            temperature: 20.0,
            altitude: 0.0,
            latitude: 0.0,
        })
        .insert_resource(BallisticsConfig {
            use_rk4: true,
            max_projectile_lifetime: 5.0,
            max_projectile_distance: 1000.0,
            enable_penetration: true,
            enable_ricochet: true,
            min_projectile_speed: 20.0,
            debug_draw: true,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                player_location_rotation,
                player_shooting,
                handle_hits,
                update_ui,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerUI;

#[derive(Component)]
struct Obstacle;

#[derive(Resource)]
struct PlayerEntity(Entity);

#[derive(Resource)]
struct GameStats {
    shots_fired: u32,
    penetrations: u32,
    ricochets: u32,
    hits: u32,
}

#[derive(Clone, Copy)]
enum MaterialType {
    Concrete,
    Metal,
    Wood,
    Glass,
}

impl MaterialType {
    fn color(&self) -> Color {
        match self {
            MaterialType::Concrete => Color::srgb(0.5, 0.5, 0.5),
            MaterialType::Metal => Color::srgb(0.7, 0.7, 0.8),
            MaterialType::Wood => Color::srgb(0.6, 0.4, 0.2),
            MaterialType::Glass => Color::srgb(0.7, 0.8, 0.9),
        }
    }
}

fn setup(mut commands: Commands) {
    // Camera setup for 2D
    commands.spawn(Camera2d);

    // Spawn player
    let player_entity = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.0, 0.8, 1.0),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            RigidBody::Dynamic,
            Collider::rectangle(30.0, 30.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
        ))
        .id();

    commands.insert_resource(PlayerEntity(player_entity));
    commands.insert_resource(GameStats {
        shots_fired: 0,
        penetrations: 0,
        ricochets: 0,
        hits: 0,
    });

    // Spawn obstacles
    spawn_obstacles(&mut commands);

    // Spawn UI (Bevy 0.18 style)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        PlayerUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("2D Surface Interactions Demo\n"),
            TextFont { font_size: 30.0, ..default() },
        ));
        parent.spawn((
            Text::new("WASD: Move | Mouse: Aim | SPACE: Shoot\n"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 0.0)),
        ));
        parent.spawn((
            Text::new("Shots: 0 | Hits: 0 | Penetrations: 0 | Ricochets: 0\n"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            StatsText,
        ));
    });
}

#[derive(Component)]
struct StatsText;

fn spawn_obstacles(commands: &mut Commands) {
    // Concrete
    commands.spawn((
        Sprite {
            color: MaterialType::Concrete.color(),
            custom_size: Some(Vec2::new(200.0, 30.0)),
            ..default()
        },
        Obstacle,
        Transform::from_xyz(250.0, 100.0, 0.0),
        bevy_bullet_dynamics::systems::surface::materials::concrete(),
        RigidBody::Static,
        Collider::rectangle(200.0, 30.0),
    ));

    // Metal
    commands.spawn((
        Sprite {
            color: MaterialType::Metal.color(),
            custom_size: Some(Vec2::new(150.0, 20.0)),
            ..default()
        },
        Obstacle,
        Transform::from_xyz(-200.0, 150.0, 0.0),
        bevy_bullet_dynamics::systems::surface::materials::metal(),
        RigidBody::Static,
        Collider::rectangle(150.0, 20.0),
    ));

    // Wood
    commands.spawn((
        Sprite {
            color: MaterialType::Wood.color(),
            custom_size: Some(Vec2::new(60.0, 60.0)),
            ..default()
        },
        Obstacle,
        Transform::from_xyz(150.0, -100.0, 0.0),
        bevy_bullet_dynamics::systems::surface::materials::wood(),
        RigidBody::Static,
        Collider::rectangle(60.0, 60.0),
    ));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut LinearVelocity, With<Player>>,
) {
    if let Some(mut velocity) = player_query.iter_mut().next() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec2::ZERO {
            direction = direction.normalize();
            velocity.0 = direction * PLAYER_SPEED;
        } else {
            velocity.0 = Vec2::ZERO;
        }
    }
}

fn player_location_rotation(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let (camera, camera_transform) = match camera_query.single() {
        Ok(v) => v,
        Err(_) => return,
    };
    let window = match window_query.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let mut player_transform = match player_query.single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            let diff = world_position - player_transform.translation.xy();
            let angle = diff.y.atan2(diff.x);
            player_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn player_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    player_entity: Res<PlayerEntity>,
    mut game_stats: ResMut<GameStats>,
    mut fire_events: MessageWriter<FireEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::KeyE) {
        if let Ok(player_transform) = player_query.get(player_entity.0) {
            let direction = player_transform.rotation * Vec3::X;
            
            let spawn_params = ProjectileSpawnParams::new(
                player_transform.translation + direction * 20.0,
                direction,
                BULLET_SPEED,
            )
            .with_damage(50.0)
            .with_owner(player_entity.0);

            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.8, 0.2),
                    custom_size: Some(Vec2::new(5.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(spawn_params.origin)
                    .with_rotation(player_transform.rotation),
                Projectile::new(spawn_params.direction * spawn_params.velocity)
                    .with_owner(spawn_params.owner.unwrap())
                    .with_mass(0.008)
                    .with_drag(0.25),
                bevy_bullet_dynamics::components::Accuracy::default(),
                Payload::Kinetic {
                    damage: spawn_params.damage,
                },
                bevy_bullet_dynamics::components::ProjectileLogic::Impact,
            ));

            fire_events.write(bevy_bullet_dynamics::events::FireEvent::new(
                spawn_params.origin,
                spawn_params.direction,
                spawn_params.velocity,
            ));

            game_stats.shots_fired += 1;
        }
    }
}

fn handle_hits(
    mut hit_events: MessageReader<HitEvent>,
    mut game_stats: ResMut<GameStats>,
) {
    for event in hit_events.read() {
        game_stats.hits += 1;
        
        if event.penetrated {
            game_stats.penetrations += 1;
        }
        
        if event.ricocheted {
            game_stats.ricochets += 1;
        }
    }
}

fn update_ui(
    mut ui_query: Query<&mut Text, With<StatsText>>,
    game_stats: Res<GameStats>,
) {
    if let Some(mut text) = ui_query.iter_mut().next() {
        text.0 = format!(
            "Shots: {} | Hits: {} | Penetrations: {} | Ricochets: {}\n",
            game_stats.shots_fired,
            game_stats.hits,
            game_stats.penetrations,
            game_stats.ricochets
        );
    }
}
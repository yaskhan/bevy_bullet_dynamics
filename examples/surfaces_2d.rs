//! 2D surface interaction example with penetration and ricochets.
//!
//! This example demonstrates how to use the surface interaction system in 2D,
//! including penetration, ricochets, and different material properties.

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_bullet_dynamics::prelude::*;

const PLAYER_SPEED: f32 = 200.0;
const BULLET_SPEED: f32 = 600.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(BallisticsPluginGroup)
        .insert_resource(BallisticsEnvironment {
            gravity: Vec3::ZERO, // No gravity in 2D top-down
            air_density: 1.2,   // Higher for more drag
            wind: Vec3::ZERO,
            temperature: 20.0,
            altitude: 0.0,
        })
        .insert_resource(BallisticsConfig {
            use_rk4: true,
            max_projectile_lifetime: 5.0,
            max_projectile_distance: 1000.0,
            enable_penetration: true,
            enable_ricochet: true,
            debug_draw: false,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
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
struct Wall;

#[derive(Component)]
struct Obstacle {
    material_type: MaterialType,
}

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

    fn surface_material(&self) -> SurfaceMaterial {
        match self {
            MaterialType::Concrete => systems::surface::materials::concrete(),
            MaterialType::Metal => systems::surface::materials::metal(),
            MaterialType::Wood => systems::surface::materials::wood(),
            MaterialType::Glass => systems::surface::materials::glass(),
        }
    }
}

fn setup(mut commands: Commands) {
    // Camera setup for 2D
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize(1.0),
            ..default()
        },
        ..default()
    });

    // Spawn player
    let player_entity = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.0, 0.8, 1.0),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                ..default()
            },
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    commands.insert_resource(PlayerEntity(player_entity));
    commands.insert_resource(GameStats {
        shots_fired: 0,
        penetrations: 0,
        ricochets: 0,
        hits: 0,
    });

    // Spawn various obstacles with different materials
    spawn_obstacles(&mut commands);

    // Spawn UI
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "2D Surface Interactions Demo\n",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "WASD: Move | SPACE: Shoot\n",
                TextStyle {
                    font_size: 20.0,
                    color: Color::YELLOW,
                    ..default()
                },
            ),
            TextSection::new(
                "Shots: 0 | Hits: 0 | Penetrations: 0 | Ricochets: 0\n",
                TextStyle {
                    font_size: 18.0,
                    color: Color::GREEN,
                    ..default()
                },
            ),
            TextSection::new(
                "Materials: Gray=Concrete | Silver=Metal | Brown=Wood | Blue=Glass\n",
                TextStyle {
                    font_size: 16.0,
                    color: Color::BLUE,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        PlayerUI,
    ));
}

fn spawn_obstacles(commands: &mut Commands) {
    // Spawn concrete walls
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: MaterialType::Concrete.color(),
                custom_size: Some(Vec2::new(200.0, 30.0)),
                ..default()
            },
            ..default()
        },
        Obstacle {
            material_type: MaterialType::Concrete,
        },
        Transform::from_xyz(250.0, 100.0, 0.0),
    ));

    // Spawn metal barriers
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: MaterialType::Metal.color(),
                custom_size: Some(Vec2::new(150.0, 20.0)),
                ..default()
            },
            ..default()
        },
        Obstacle {
            material_type: MaterialType::Metal,
        },
        Transform::from_xyz(-200.0, 150.0, 0.0),
    ));

    // Spawn wooden crates
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: MaterialType::Wood.color(),
                custom_size: Some(Vec2::new(60.0, 60.0)),
                ..default()
            },
            ..default()
        },
        Obstacle {
            material_type: MaterialType::Wood,
        },
        Transform::from_xyz(150.0, -100.0, 0.0),
    ));

    // Spawn glass panels
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: MaterialType::Glass.color(),
                custom_size: Some(Vec2::new(100.0, 40.0)),
                ..default()
            },
            ..default()
        },
        Obstacle {
            material_type: MaterialType::Glass,
        },
        Transform::from_xyz(-150.0, -150.0, 0.0),
    ));

    // Another concrete wall
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: MaterialType::Concrete.color(),
                custom_size: Some(Vec2::new(30.0, 200.0)),
                ..default()
            },
            ..default()
        },
        Obstacle {
            material_type: MaterialType::Concrete,
        },
        Transform::from_xyz(0.0, 200.0, 0.0),
    ));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();
    let mut direction = Vec3::ZERO;

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

    if direction != Vec3::ZERO {
        direction = direction.normalize();
        player_transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

fn player_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    player_entity: Res<PlayerEntity>,
    mut game_stats: ResMut<GameStats>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let player_transform = player_query.get(player_entity.0).unwrap();
        
        // Calculate shoot direction (towards mouse cursor would be better in a real game)
        let direction = Vec3::X; // Shooting right by default
        
        // Create projectile spawn parameters
        let spawn_params = ProjectileSpawnParams::new(
            player_transform.translation + direction * 20.0, // Spawn slightly in front of player
            direction,
            BULLET_SPEED,
        )
        .with_damage(50.0)
        .with_owner(player_entity.0);

        // Spawn the projectile with physics components
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.8, 0.2), // Yellow bullet
                    custom_size: Some(Vec2::new(5.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_translation(spawn_params.origin)
                    .with_rotation(Quat::from_rotation_z(
                        spawn_params.direction.y.atan2(spawn_params.direction.x),
                    )),
                ..default()
            },
            Projectile::new(spawn_params.direction * spawn_params.velocity)
                .with_owner(spawn_params.owner.unwrap())
                .with_mass(0.008) // Light bullet
                .with_drag(0.25), // Low drag for longer range
            Accuracy::default(),
            Payload::Kinetic {
                damage: spawn_params.damage,
            },
            ProjectileLogic::Impact,
        ));

        // Fire event for systems to pick up
        commands.trigger(FireEvent::new(
            spawn_params.origin,
            spawn_params.direction,
            spawn_params.velocity,
        ));

        // Update game stats
        game_stats.shots_fired += 1;
    }
}

fn handle_hits(
    mut hit_events: EventReader<HitEvent>,
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
    mut ui_query: Query<&mut Text, With<PlayerUI>>,
    game_stats: Res<GameStats>,
) {
    let mut text = ui_query.single_mut();
    
    text.sections[2].value = format!(
        "Shots: {} | Hits: {} | Penetrations: {} | Ricochets: {}\n",
        game_stats.shots_fired,
        game_stats.hits,
        game_stats.penetrations,
        game_stats.ricochets
    );
}
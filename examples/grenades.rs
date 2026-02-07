//! Grenades example demonstrating throwable weapons with different payloads.

use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use bevy_bullet_dynamics::prelude::*;
use bevy_bullet_dynamics::systems::logic::presets as grenade_presets;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BallisticsPluginGroup)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, handle_explosions, update_ui))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.35),
            ..default()
        })),
    ));

    // Target dummies
    let dummy_mesh = meshes.add(Capsule3d::new(0.5, 1.5));
    let dummy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.4),
        ..default()
    });

    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let radius = 8.0;
        commands.spawn((
            Mesh3d(dummy_mesh.clone()),
            MeshMaterial3d(dummy_material.clone()),
            Transform::from_xyz(angle.cos() * radius, 1.0, angle.sin() * radius),
            TargetDummy { health: 100.0 },
        ));
    }

    // Throw origin marker
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            emissive: LinearRgba::rgb(0.5, 2.0, 0.5),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 15.0),
        ThrowerMarker,
    ));

    // UI
    commands.spawn((
        Text::new("Press 1: Frag Grenade\nPress 2: Flashbang\nPress 3: Smoke\nPress 4: Molotov\n\nPress SPACE to throw"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        UiText,
    ));

    // Grenade type state
    commands.insert_resource(GrenadeState {
        grenade_type: GrenadeType::Frag,
    });
}

#[derive(Component)]
struct ThrowerMarker;

#[derive(Component)]
struct TargetDummy {
    health: f32,
}

#[derive(Component)]
struct UiText;

#[derive(Resource)]
struct GrenadeState {
    grenade_type: GrenadeType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GrenadeType {
    Frag,
    Flash,
    Smoke,
    Molotov,
}

impl GrenadeType {
    fn name(&self) -> &'static str {
        match self {
            Self::Frag => "Frag Grenade",
            Self::Flash => "Flashbang",
            Self::Smoke => "Smoke Grenade",
            Self::Molotov => "Molotov Cocktail",
        }
    }

    fn color(&self) -> Color {
        match self {
            Self::Frag => Color::srgb(0.3, 0.3, 0.3),
            Self::Flash => Color::srgb(0.9, 0.9, 0.9),
            Self::Smoke => Color::srgb(0.5, 0.5, 0.5),
            Self::Molotov => Color::srgb(0.8, 0.4, 0.1),
        }
    }

    fn logic_and_payload(&self) -> (ProjectileLogic, Payload) {
        match self {
            Self::Frag => grenade_presets::frag_grenade(),
            Self::Flash => grenade_presets::flashbang(),
            Self::Smoke => grenade_presets::smoke_grenade(),
            Self::Molotov => grenade_presets::molotov(),
        }
    }
}

fn handle_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut grenade_state: ResMut<GrenadeState>,
    thrower: Query<&Transform, With<ThrowerMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Grenade selection
    if keyboard.just_pressed(KeyCode::Digit1) {
        grenade_state.grenade_type = GrenadeType::Frag;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        grenade_state.grenade_type = GrenadeType::Flash;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        grenade_state.grenade_type = GrenadeType::Smoke;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        grenade_state.grenade_type = GrenadeType::Molotov;
    }

    // Throw
    if keyboard.just_pressed(KeyCode::Space) {
        let Ok(thrower_transform) = thrower.single() else {
            return;
        };

        let origin = thrower_transform.translation;
        // Parabolic throw: forward and up
        let direction = Vec3::new(0.0, 0.8, -1.0).normalize();
        let throw_speed = 15.0;
        let velocity = direction * throw_speed;

        let (logic, payload) = grenade_state.grenade_type.logic_and_payload();

        // Grenade mesh
        let grenade_mesh = meshes.add(Sphere::new(0.15));
        let grenade_material = materials.add(StandardMaterial {
            base_color: grenade_state.grenade_type.color(),
            ..default()
        });

        // Spawn grenade as projectile with Euler physics (slower, parabolic)
        commands.spawn((
            Mesh3d(grenade_mesh),
            MeshMaterial3d(grenade_material),
            Transform::from_translation(origin),
            Projectile {
                velocity,
                mass: 0.5,             // Heavy
                drag_coefficient: 0.5, // High drag
                reference_area: 0.01,
                penetration_power: 0.0, // Doesn't penetrate
                previous_position: origin,
                owner: None,
            },
            logic,
            payload,
        ));
    }
}

fn handle_explosions(
    mut explosion_events: MessageReader<ExplosionEvent>,
    mut targets: Query<(&Transform, &mut TargetDummy)>,
) {
    use bevy_bullet_dynamics::systems::logic::calculate_explosion_damage;

    for explosion in explosion_events.read() {
        match explosion.explosion_type {
            ExplosionType::HighExplosive => {
                // Deal damage to nearby targets
                for (transform, mut dummy) in targets.iter_mut() {
                    let distance = transform.translation.distance(explosion.center);
                    let damage = calculate_explosion_damage(
                        explosion.damage,
                        distance,
                        explosion.radius,
                        explosion.falloff,
                    );

                    if damage > 0.0 {
                        dummy.health -= damage;
                        println!(
                            "Target hit for {:.1} damage! Health: {:.1}",
                            damage, dummy.health
                        );
                    }
                }
            }
            ExplosionType::Flash => {
                // Check who is in range and facing the explosion
                for (transform, _) in targets.iter() {
                    let distance = transform.translation.distance(explosion.center);
                    if distance < explosion.radius {
                        println!("Target flashed!");
                    }
                }
            }
            ExplosionType::Smoke => {
                println!("Smoke deployed at {:?}", explosion.center);
            }
            ExplosionType::Incendiary => {
                println!("Fire started at {:?}", explosion.center);
            }
            _ => {}
        }
    }
}

fn update_ui(
    grenade_state: Res<GrenadeState>,
    mut ui_text: Query<&mut Text, With<UiText>>,
) {
    if grenade_state.is_changed() {
        for mut text in ui_text.iter_mut() {
            text.0 = format!(
                "Press 1: Frag Grenade\nPress 2: Flashbang\nPress 3: Smoke\nPress 4: Molotov\n\nSelected: {}\nPress SPACE to throw",
                grenade_state.grenade_type.name()
            );
        }
    }
}

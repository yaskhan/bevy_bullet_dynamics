//! Grenades example demonstrating throwable weapons with different payloads.

use bevy::prelude::*;
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

#[derive(Resource)]
struct GrenadeAssets {
    mesh: Handle<Mesh>,
    frag_material: Handle<StandardMaterial>,
    flash_material: Handle<StandardMaterial>,
    smoke_material: Handle<StandardMaterial>,
    molotov_material: Handle<StandardMaterial>,
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

    // Pre-load grenade assets to avoid memory leak
    let grenade_assets = GrenadeAssets {
        mesh: meshes.add(Sphere::new(0.15)),
        frag_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        }),
        flash_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 0.9),
            ..default()
        }),
        smoke_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            ..default()
        }),
        molotov_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.4, 0.1),
            ..default()
        }),
    };
    commands.insert_resource(grenade_assets);

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
        Text::new("Press 1-4: Select | SPACE: Throw\n\n1: Frag\n2: Flash\n3: Smoke\n4: Molotov"),
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
    assets: Res<GrenadeAssets>,
) {
    // Grenade selection
    if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        grenade_state.grenade_type = GrenadeType::Frag;
        info!("Selected: Frag");
    }
    if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        grenade_state.grenade_type = GrenadeType::Flash;
        info!("Selected: Flash");
    }
    if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        grenade_state.grenade_type = GrenadeType::Smoke;
        info!("Selected: Smoke");
    }
    if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) {
        grenade_state.grenade_type = GrenadeType::Molotov;
        info!("Selected: Molotov");
    }

    // Throw
    if keyboard.just_pressed(KeyCode::Space) {
        let Some(thrower_transform) = thrower.iter().next() else {
            warn!("No thrower marker found!");
            return;
        };

        info!("Throwing {}", grenade_state.grenade_type.name());

        let origin = thrower_transform.translation;
        let direction = Vec3::new(0.0, 0.8, -1.0).normalize();
        let throw_speed = 15.0;
        let velocity = direction * throw_speed;

        let (logic, payload) = grenade_state.grenade_type.logic_and_payload();

        let material = match grenade_state.grenade_type {
            GrenadeType::Frag => assets.frag_material.clone(),
            GrenadeType::Flash => assets.flash_material.clone(),
            GrenadeType::Smoke => assets.smoke_material.clone(),
            GrenadeType::Molotov => assets.molotov_material.clone(),
        };

        commands.spawn((
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(origin),
            Projectile {
                velocity,
                mass: 0.5,
                drag_coefficient: 0.5,
                reference_area: 0.01,
                diameter: 0.05,
                spin: 0.0,
                penetration_power: 0.0,
                previous_position: origin,
                age: 0.0,
                distance_travelled: 0.0,
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
        info!("Explosion detected at {:?} type {:?}", explosion.center, explosion.explosion_type);
        match explosion.explosion_type {
            ExplosionType::HighExplosive | ExplosionType::Fragmentation => {
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
                        info!("Target hit for {:.1} damage! Health: {:.1}", damage, dummy.health);
                    }
                }
            }
            _ => {
                info!("Special explosion: {:?}", explosion.explosion_type);
            }
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
                "Press 1-4: Select | SPACE: Throw\n\nSelected: {}\nPress SPACE to throw",
                grenade_state.grenade_type.name()
            );
        }
    }
}

use std::f32::consts::FRAC_PI_2;
use bevy::{
    camera::visibility::RenderLayers,
    color::palettes::tailwind,
    input::mouse::AccumulatedMouseMotion,
    light::NotShadowCaster,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_bullet_dynamics::prelude::*;
use avian3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(BallisticsPluginGroup)
        .insert_resource(BallisticsConfig {
            max_projectile_distance: 1000.0,
            ..default()
        })
        .add_systems(
            Startup,
            (
                spawn_player,
                spawn_world,
                spawn_lights,
                spawn_ui,
            ),
        )
        .add_systems(Update, (
            move_player,
            handle_shooting,
            switch_weapons,
            update_ui,
            handle_events,
            scale_projectiles,
            grab_cursor,
            update_hit_markers,
        ))
        .run();
}

// --- Components ---

#[derive(Component)]
struct Player;

#[derive(Component)]
struct WorldModelCamera;

#[derive(Component)]
struct Muzzle;

#[derive(Component, Default, Deref, DerefMut)]
struct CameraSensitivity(Vec2);

#[derive(Component)]
struct WeaponState {
    current_type: WeaponType,
    last_fire_time: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum WeaponType {
    Pistol,
    Rifle,
    Sniper,
}

#[derive(Component)]
struct Stats {
    hits: u32,
    ricochets: u32,
}

#[derive(Component)]
struct StatsText;

#[derive(Component)]
struct WeaponText;

#[derive(Component)]
struct HitMarker {
    lifetime: f32,
}

// --- Constants ---

const DEFAULT_RENDER_LAYER: usize = 0;
const VIEW_MODEL_RENDER_LAYER: usize = 1;

// --- Startup Systems ---

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arm_mesh = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));

    // Player Root
    commands.spawn((
        Player,
        Stats { hits: 0, ricochets: 0 },
        WeaponState { 
            current_type: WeaponType::Pistol, 
            last_fire_time: 0.0 
        },
        CameraSensitivity(Vec2::new(0.003, 0.002)),
        Transform::from_xyz(0.0, 1.5, 5.0),
        Visibility::default(),
    )).with_children(|parent| {
        // 1. World Model Camera (Layer 0)
        parent.spawn((
            WorldModelCamera,
            Camera3d::default(),
            Projection::from(PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }),
        ));
        
        // 2. View Model Camera (Layer 1, Renders on top)
        parent.spawn((
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Projection::from(PerspectiveProjection {
                fov: 70.0_f32.to_radians(),
                ..default()
            }),
            RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
        ));
        
        // 3. View Model (Visual Arm/Gun)
        parent.spawn((
            Mesh3d(arm_mesh),
            MeshMaterial3d(arm_material),
            Transform::from_xyz(0.3, -0.3, -0.4), // Positioned to the right/down
            RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            NotShadowCaster,
        )).with_children(|arm| {
            // Muzzle position relative to arm
            arm.spawn((
                Muzzle,
                Transform::from_xyz(0.0, 0.0, -0.3),
            ));
        });
    });
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor
    let floor_mesh = meshes.add(Plane3d::default().mesh().size(200.0, 200.0));
    let floor_material = materials.add(StandardMaterial {
        base_color: tailwind::SLATE_800.into(),
        ..default()
    });
    commands.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        SurfaceMaterial {
            ricochet_angle: 0.05, // Only grazing hits (approx 3 degrees) ricochet
            ..default()
        },
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    // Targets (Response cubes)
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let concrete_mat = materials.add(Color::from(tailwind::GRAY_400));
    
    for i in -5..5 {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(concrete_mat.clone()),
            Transform::from_xyz(i as f32 * 2.0, 0.5, -10.0),
            SurfaceMaterial {
                ricochet_angle: 0.05, // Hard Concrete, only grazes ricochet
                penetration_loss: 80.0,
                thickness: 1.0,
                hit_effect: HitEffectType::Dust,
            },
            RigidBody::Static,
            Collider::cuboid(1.0, 1.0, 1.0),
        ));
    }

    // Ricochet Plate (Slanted) - easier ricochet here
    let plate_mesh = meshes.add(Cuboid::new(10.0, 0.1, 8.0));
    let plate_material = materials.add(Color::from(tailwind::BLUE_400));
    commands.spawn((
        Mesh3d(plate_mesh),
        MeshMaterial3d(plate_material),
        Transform::from_xyz(0.0, 4.0, -20.0)
            .with_rotation(Quat::from_rotation_x(45.0_f32.to_radians())),
        SurfaceMaterial {
            ricochet_angle: 0.4, // Up to ~23 degrees grazing
            penetration_loss: 200.0,
            thickness: 0.1,
            hit_effect: HitEffectType::Sparks,
        },
        RigidBody::Static,
        Collider::cuboid(10.0, 0.1, 8.0),
    ));
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}

fn spawn_ui(mut commands: Commands) {
    // Root UI container
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|parent| {
            // Stats Top Left
            parent.spawn((
                Text::new("Hits: 0 | Ricochets: 0"),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
                StatsText,
            ));

            // Weapon Bottom Center
            parent.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|inner| {
                inner.spawn((
                    Text::new("Weapon: Pistol (Key 1)"),
                    TextFont::from_font_size(32.0),
                    TextColor(tailwind::AMBER_300.into()),
                    WeaponText,
                ));
            });

            // Help Bottom Left
            parent.spawn((
                Text::new("WASD: Move | MOUSE: Look | LCLICK: Shoot | 1-3: Switch Weapons"),
                TextFont::from_font_size(16.0),
                TextColor(tailwind::GRAY_400.into()),
            ));
        });

    // Crosshair - indicative of center
    commands.spawn((
        Node {
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect::new(Val::Px(-2.0), Val::Px(0.0), Val::Px(-2.0), Val::Px(0.0)),
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
}

// --- Update Systems ---

fn move_player(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Single<(&mut Transform, &CameraSensitivity), With<Player>>,
) {
    let (ref mut transform, ref sensitivity) = *player_query;

    let delta = accumulated_mouse_motion.delta;
    if delta != Vec2::ZERO {
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        yaw -= delta.x * sensitivity.x;
        pitch -= delta.y * sensitivity.y;
        
        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.1;
        pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    let mut move_dir = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { move_dir += *transform.forward(); }
    if keyboard.pressed(KeyCode::KeyS) { move_dir += *transform.back(); }
    if keyboard.pressed(KeyCode::KeyA) { move_dir += *transform.left(); }
    if keyboard.pressed(KeyCode::KeyD) { move_dir += *transform.right(); }
    
    move_dir.y = 0.0;
    if move_dir.length_squared() > 0.0 {
        transform.translation += move_dir.normalize() * 5.0 * time.delta_secs();
    }
}

fn switch_weapons(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Single<&mut WeaponState, With<Player>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) { state.current_type = WeaponType::Pistol; }
    if keyboard.just_pressed(KeyCode::Digit2) { state.current_type = WeaponType::Rifle; }
    if keyboard.just_pressed(KeyCode::Digit3) { state.current_type = WeaponType::Sniper; }
}

fn handle_shooting(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut player_query: Single<(&Transform, &mut WeaponState), With<Player>>,
    muzzle: Single<&GlobalTransform, With<Muzzle>>,
    ballistics_assets: Res<BallisticsAssets>,
    spatial_query: SpatialQuery,
) {
    let (player_transform, ref mut state) = *player_query;
    let now = time.elapsed_secs();

    let muzzle_transform = *muzzle;
    let muzzle_pos = muzzle_transform.translation();
    
    // DEFINTIVE GHOST FIX:
    // If GlobalTransform is uninitialized (all zeros, especially translation), 
    // it means it hasn't propagated yet. We MUST NOT spawn at (0,0,0).
    // We also check if it's too close to origin (unlikely for a real muzzle).
    if muzzle_pos.length_squared() < 0.001 {
        return;
    }

    let (fire_rate, automatic, velocity, accuracy) = match state.current_type {
        WeaponType::Pistol => (5.0, false, 400.0, bevy_bullet_dynamics::systems::accuracy::presets::pistol()),
        WeaponType::Rifle => (10.0, true, 850.0, bevy_bullet_dynamics::systems::accuracy::presets::rifle()),
        WeaponType::Sniper => (1.0, false, 1200.0, bevy_bullet_dynamics::systems::accuracy::presets::sniper()),
    };

    let can_fire = if automatic {
        mouse.pressed(MouseButton::Left) && now - state.last_fire_time >= 1.0 / fire_rate
    } else {
        mouse.just_pressed(MouseButton::Left) && now - state.last_fire_time >= 1.0 / fire_rate
    };

    if can_fire {
        state.last_fire_time = now;

        // 1. Determine Target Point (Aim from camera center)
        let ray_origin = player_transform.translation;
        let ray_dir = player_transform.forward();
        let target_point = if let Some(hit) = spatial_query.cast_ray(
            ray_origin,
            ray_dir,
            1000.0,
            false,
            &SpatialQueryFilter::default(),
        ) {
            ray_origin + *ray_dir * hit.distance
        } else {
            ray_origin + *ray_dir * 1000.0
        };

        // 2. Spawn from Muzzle
        let spawn_pos: Vec3 = muzzle.translation();
        let mut shot_dir = (target_point - spawn_pos).normalize();

        // Apply spread
        shot_dir = bevy_bullet_dynamics::systems::accuracy::apply_spread_to_direction(
            shot_dir, 
            accuracy.base_spread, 
            now.to_bits() as u64
        );

        // Spawn Projectile
        commands.spawn((
            Mesh3d(ballistics_assets.sphere_mesh.clone()),
            MeshMaterial3d(ballistics_assets.flash_material.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.01)),
            Projectile::new(shot_dir * velocity).with_previous_position(spawn_pos),
            ProjectileLogic::Impact,
            Payload::Kinetic { damage: 50.0 },
            accuracy, 
        ));

        // Muzzle Flash
        commands.spawn((
            MuzzleFlash {
                lifetime: 0.05,
                intensity: 5.0,
                scale: 0.2,
            },
            Transform::from_translation(spawn_pos),
        ));
    }
}

fn scale_projectiles(
    mut query: Query<(&mut Transform, &Projectile)>,
) {
    for (mut transform, projectile) in query.iter_mut() {
        // Grow even more for visibility: from 0.05 to 1.5 over 500 meters
        let factor = (projectile.distance_travelled / 500.0).clamp(0.0, 1.0);
        let final_scale = 0.05 + (factor * 1.45);
        transform.scale = Vec3::splat(final_scale);
    }
}

fn handle_events(
    mut commands: Commands,
    ballistics_assets: Res<BallisticsAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hit_events: MessageReader<HitEvent>,
    mut ricochet_events: MessageReader<RicochetEvent>,
    mut stats: Single<&mut Stats>,
    mut projectile_query: Query<&mut MeshMaterial3d<StandardMaterial>, With<Projectile>>,
) {
    let red_mat = materials.add(StandardMaterial {
        base_color: Color::from(tailwind::RED_500),
        emissive: LinearRgba::from(tailwind::RED_400) * 5.0,
        ..default()
    });
    let green_mat = materials.add(StandardMaterial {
        base_color: Color::from(tailwind::GREEN_500),
        emissive: LinearRgba::from(tailwind::GREEN_400) * 5.0,
        ..default()
    });

    for hit in hit_events.read() {
        if hit.ricocheted {
            continue; 
        }
        if let Ok(mut mat) = projectile_query.get_mut(hit.projectile) {
            *mat = MeshMaterial3d(red_mat.clone());
        }
        stats.hits += 1;

        // Spawn hit marker sphere
        commands.spawn((
            Mesh3d(ballistics_assets.sphere_mesh.clone()),
            MeshMaterial3d(red_mat.clone()),
            Transform::from_translation(hit.impact_point).with_scale(Vec3::splat(0.2)),
            HitMarker {
                lifetime: 3.0,
            },
        ));
    }

    for rico in ricochet_events.read() {
        if let Ok(mut mat) = projectile_query.get_mut(rico.projectile) {
            *mat = MeshMaterial3d(green_mat.clone());
        }
        stats.ricochets += 1;
    }
}

fn update_ui(
    stats: Single<&Stats>,
    weapon: Single<&WeaponState>,
    mut stats_text: Single<&mut Text, (With<StatsText>, Without<WeaponText>)>,
    mut weapon_text: Single<&mut Text, (With<WeaponText>, Without<StatsText>)>,
) {
    stats_text.0 = format!("Hits: {} | Ricochets: {}", stats.hits, stats.ricochets);

    let name = match weapon.current_type {
        WeaponType::Pistol => "Pistol (Semi)",
        WeaponType::Rifle => "Assault Rifle (Auto)",
        WeaponType::Sniper => "Sniper Rifle (Bolt)",
    };
    weapon_text.0 = format!("Weapon: {}", name);
}

fn update_hit_markers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitMarker)>,
) {
    let dt = time.delta_secs();
    for (entity, mut marker) in query.iter_mut() {
        marker.lifetime -= dt;
        if marker.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn grab_cursor(
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

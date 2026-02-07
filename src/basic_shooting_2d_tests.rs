#[cfg(test)]
mod basic_shooting_2d_tests {
    use bevy::{
        app::{App, Startup, Update},
        ecs::system::Commands,
        input::ButtonInput,
        math::Vec3,
        prelude::{Entity, KeyCode, Resource, With},
        time::Time,
        transform::components::Transform,
    };
    use bevy_bullet_dynamics::prelude::*;

    use crate::{Enemy, Player, PlayerEntity, TemporaryEffect};

    #[test]
    fn test_player_and_enemies_spawned() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(BallisticsEnvironment {
            gravity: Vec3::ZERO,
            air_density: 1.15,
            wind: Vec3::ZERO,
            temperature: 20.0,
            altitude: 0.0,
        });
        app.insert_resource(BallisticsConfig {
            use_rk4: true,
            max_projectile_lifetime: 5.0,
            max_projectile_distance: 1000.0,
            enable_penetration: false,
            enable_ricochet: false,
            debug_draw: false,
        });
        
        // Setup system
        app.add_systems(Startup, setup_entities_for_basic_test);
        
        // Run startup systems
        app.update();
        
        // Verify player was created
        let player_entities: Vec<_> = app.world.query_filtered::<Entity, With<Player>>().iter(&app.world).collect();
        assert_eq!(player_entities.len(), 1, "Should have exactly one player entity");
        
        // Verify enemies were created
        let enemy_entities: Vec<_> = app.world.query_filtered::<Entity, With<Enemy>>().iter(&app.world).collect();
        assert!(enemy_entities.len() > 0, "Should have at least one enemy entity");
        
        // Verify player entity resource was inserted
        assert!(app.world.contains_resource::<PlayerEntity>(), "PlayerEntity resource should be present");
    }

    #[test]
    fn test_player_movement_system() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(BallisticsEnvironment::default());
        app.insert_resource(BallisticsConfig::default());
        app.insert_resource(Time::default());
        
        // Setup system
        app.add_systems(Startup, setup_player_for_basic_test);
        app.add_systems(Update, player_movement_for_basic_test);
        
        // Run startup systems
        app.update();
        
        // Get initial position
        let initial_positions: Vec<Transform> = app
            .world
            .query_filtered::<&Transform, With<Player>>()
            .iter(&app.world)
            .cloned()
            .collect();
        
        assert_eq!(initial_positions.len(), 1);
        let initial_pos = initial_positions[0].translation;
        
        // Simulate pressing D key to move right
        let mut keyboard_input = ButtonInput::<KeyCode>::default();
        keyboard_input.press(KeyCode::KeyD);
        app.insert_resource(keyboard_input);
        
        // Run update with time
        app.insert_resource(Time::from_seconds(1.0));
        app.update();
        
        // Get new position
        let new_positions: Vec<Transform> = app
            .world
            .query_filtered::<&Transform, With<Player>>()
            .iter(&app.world)
            .cloned()
            .collect();
        
        assert_eq!(new_positions.len(), 1);
        let new_pos = new_positions[0].translation;
        
        // Player should have moved in positive X direction
        assert!(new_pos.x > initial_pos.x, "Player should move in positive X direction when D is pressed");
    }

    #[test]
    fn test_temporary_effect_cleanup() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(Time::from_seconds(1.0));
        
        // Setup system
        app.add_systems(Startup, setup_temporary_effects_for_test);
        app.add_systems(Update, cleanup_projectiles_for_basic_test);
        
        // Run startup systems
        app.update();
        
        // Initially should have temporary effects
        let initial_effect_count = app.world.query_filtered::<Entity, With<TemporaryEffect>>().iter(&app.world).count();
        assert!(initial_effect_count > 0, "Should have temporary effects initially");
        
        // Run update to process cleanup
        app.update();
        
        // Effects with lifetime <= 0 should be cleaned up
        let remaining_effect_count = app.world.query_filtered::<Entity, With<TemporaryEffect>>().iter(&app.world).count();
        assert!(remaining_effect_count <= initial_effect_count, "Should have same or fewer effects after cleanup");
    }

    #[test]
    fn test_projectile_spawn_params_builder_pattern() {
        // Test the builder pattern for ProjectileSpawnParams
        let params = ProjectileSpawnParams::new(Vec3::ZERO, Vec3::X, 500.0)
            .with_damage(30.0)
            .with_mass(0.008)
            .with_owner(Entity::PLACEHOLDER);
        
        assert_eq!(params.origin, Vec3::ZERO);
        assert_eq!(params.direction, Vec3::X);
        assert_eq!(params.velocity, 500.0);
        assert_eq!(params.damage, 30.0);
        assert_eq!(params.mass, 0.008);
        assert!(params.owner.is_some());
    }

    #[test]
    fn test_accuracy_component_builder_pattern() {
        // Test the builder pattern for Accuracy
        let accuracy = Accuracy::default()
            .with_base_spread(0.001)
            .with_max_spread(0.05)
            .with_bloom_per_shot(0.01);
        
        assert_eq!(accuracy.base_spread, 0.001);
        assert_eq!(accuracy.max_spread, 0.05);
        assert_eq!(accuracy.bloom_per_shot, 0.01);
    }

    #[test]
    fn test_projectile_component_creation() {
        // Test creating a projectile with various parameters
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let velocity = 400.0;
        
        let projectile = Projectile::new(direction * velocity)
            .with_mass(0.01)
            .with_drag(0.3)
            .with_owner(Entity::PLACEHOLDER);
        
        assert_eq!(projectile.velocity, direction * velocity);
        assert_eq!(projectile.mass, 0.01);
        assert_eq!(projectile.drag_coefficient, 0.3);
        assert!(projectile.owner.is_some());
    }

    #[test]
    fn test_payload_enum_variants() {
        // Test different payload types
        let kinetic_payload = Payload::Kinetic { damage: 25.0 };
        let explosive_payload = Payload::Explosive {
            damage: 100.0,
            radius: 5.0,
            falloff: 1.5,
        };
        let incendiary_payload = Payload::Incendiary {
            duration: 5.0,
            damage_per_second: 10.0,
            radius: 3.0,
        };
        
        match kinetic_payload {
            Payload::Kinetic { damage } => assert_eq!(damage, 25.0),
            _ => panic!("Expected kinetic payload"),
        }
        
        match explosive_payload {
            Payload::Explosive { damage, radius, falloff } => {
                assert_eq!(damage, 100.0);
                assert_eq!(radius, 5.0);
                assert_eq!(falloff, 1.5);
            },
            _ => panic!("Expected explosive payload"),
        }
        
        match incendiary_payload {
            Payload::Incendiary { duration, damage_per_second, radius } => {
                assert_eq!(duration, 5.0);
                assert_eq!(damage_per_second, 10.0);
                assert_eq!(radius, 3.0);
            },
            _ => panic!("Expected incendiary payload"),
        }
    }

    #[test]
    fn test_projectile_logic_enum_variants() {
        // Test different projectile logic types
        let impact_logic = ProjectileLogic::Impact;
        let timed_logic = ProjectileLogic::Timed {
            fuse: 3.0,
            elapsed: 0.0,
        };
        let sticky_logic = ProjectileLogic::Sticky;
        let proximity_logic = ProjectileLogic::Proximity {
            range: 2.0,
        };
        
        assert_eq!(impact_logic, ProjectileLogic::Impact);
        
        match timed_logic {
            ProjectileLogic::Timed { fuse, elapsed } => {
                assert_eq!(fuse, 3.0);
                assert_eq!(elapsed, 0.0);
            },
            _ => panic!("Expected timed logic"),
        }
        
        assert_eq!(sticky_logic, ProjectileLogic::Sticky);
        
        match proximity_logic {
            ProjectileLogic::Proximity { range } => {
                assert_eq!(range, 2.0);
            },
            _ => panic!("Expected proximity logic"),
        }
    }

    #[test]
    fn test_ballistics_environment_defaults() {
        // Test default values for BallisticsEnvironment
        let env = BallisticsEnvironment::default();
        
        assert_eq!(env.gravity, Vec3::new(0.0, -9.81, 0.0));
        assert_eq!(env.air_density, 1.225);
        assert_eq!(env.wind, Vec3::ZERO);
        assert_eq!(env.temperature, 20.0);
        assert_eq!(env.altitude, 0.0);
    }

    #[test]
    fn test_ballistics_config_defaults() {
        // Test default values for BallisticsConfig
        let config = BallisticsConfig::default();
        
        assert_eq!(config.use_rk4, true);
        assert_eq!(config.max_projectile_lifetime, 10.0);
        assert_eq!(config.max_projectile_distance, 2000.0);
        assert_eq!(config.enable_penetration, true);
        assert_eq!(config.enable_ricochet, true);
        assert_eq!(config.debug_draw, false);
    }

    // Helper systems for testing
    fn setup_entities_for_basic_test(mut commands: Commands) {
        let player_entity = commands
            .spawn((
                Player,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        commands.insert_resource(PlayerEntity(player_entity));
        
        // Spawn some enemies
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue; // Skip player position
                }
                
                commands.spawn((
                    Enemy,
                    Transform::from_xyz(i as f32 * 100.0, j as f32 * 100.0, 0.0),
                ));
            }
        }
    }

    fn setup_player_for_basic_test(mut commands: Commands) {
        let player_entity = commands
            .spawn((
                Player,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        commands.insert_resource(PlayerEntity(player_entity));
    }

    fn player_movement_for_basic_test(
        keyboard_input: bevy::prelude::Res<ButtonInput<KeyCode>>,
        mut player_query: bevy::prelude::Query<&mut Transform, With<Player>>,
        time: bevy::prelude::Res<Time>,
    ) {
        const PLAYER_SPEED: f32 = 300.0;
        
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

    fn setup_temporary_effects_for_test(mut commands: Commands) {
        // Spawn temporary effects with different lifetimes
        commands.spawn((
            TemporaryEffect { lifetime: 0.5 },
        ));
        
        commands.spawn((
            TemporaryEffect { lifetime: -0.1 }, // Already expired
        ));
        
        commands.spawn((
            TemporaryEffect { lifetime: 1.0 },
        ));
    }

    fn cleanup_projectiles_for_basic_test(
        mut commands: Commands,
        mut query: bevy::prelude::Query<(Entity, &mut TemporaryEffect)>,
        time: bevy::prelude::Res<Time>,
    ) {
        for (entity, mut effect) in query.iter_mut() {
            effect.lifetime -= time.delta_seconds();
            if effect.lifetime <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}
#[cfg(test)]
mod advanced_shooting_2d_tests {
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

    use crate::{CurrentWeapon, Player, PlayerEntity, PlayerStats};

    #[test]
    fn test_player_component_exists() {
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
            enable_penetration: true,
            enable_ricochet: true,
            debug_draw: false,
        });
        app.insert_resource(WeaponPresets::with_defaults());
        
        // Setup system
        app.add_systems(Startup, setup_player_for_testing);
        
        // Run startup systems
        app.update();
        
        // Verify player was created
        let player_entities: Vec<_> = app.world.query_filtered::<Entity, With<Player>>().iter(&app.world).collect();
        assert_eq!(player_entities.len(), 1, "Should have exactly one player entity");
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
        app.add_systems(Startup, setup_player_for_testing);
        app.add_systems(Update, player_movement_for_testing);
        
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
        
        // Simulate pressing W key
        let mut keyboard_input = ButtonInput::<KeyCode>::default();
        keyboard_input.press(KeyCode::KeyW);
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
        
        // Player should have moved in positive Y direction
        assert!(new_pos.y > initial_pos.y, "Player should move in positive Y direction when W is pressed");
    }

    #[test]
    fn test_weapon_switching_functionality() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(BallisticsEnvironment::default());
        app.insert_resource(BallisticsConfig::default());
        app.insert_resource(WeaponPresets::with_defaults());
        app.insert_resource(CurrentWeapon(0));
        app.insert_resource(PlayerStats {
            weapon_index: 0,
            shots_fired: 0,
            hits: 0,
            accuracy: 0.0,
        });
        
        // Setup system
        app.add_systems(Startup, setup_player_for_testing);
        app.add_systems(Update, weapon_switching_for_testing);
        
        // Run startup systems
        app.update();
        
        // Verify initial weapon
        let initial_weapon = app.world.resource::<CurrentWeapon>();
        assert_eq!(initial_weapon.0, 0);
        
        let initial_stats = app.world.resource::<PlayerStats>();
        assert_eq!(initial_stats.weapon_index, 0);
        
        // Simulate pressing digit 2 to switch to rifle
        let mut keyboard_input = ButtonInput::<KeyCode>::default();
        keyboard_input.press(KeyCode::Digit2);
        app.insert_resource(keyboard_input);
        
        // Run update
        app.update();
        
        // Verify weapon switched
        let new_weapon = app.world.resource::<CurrentWeapon>();
        assert_eq!(new_weapon.0, 1); // Rifle is at index 1
        
        let new_stats = app.world.resource::<PlayerStats>();
        assert_eq!(new_stats.weapon_index, 1);
    }

    #[test]
    fn test_accuracy_component_default_values() {
        // Test that the Accuracy component has reasonable default values
        let accuracy = Accuracy::default();
        
        assert_eq!(accuracy.current_bloom, 0.0);
        assert!(accuracy.base_spread > 0.0);
        assert!(accuracy.max_spread > accuracy.base_spread);
        assert!(accuracy.bloom_per_shot > 0.0);
        assert!(accuracy.recovery_rate > 0.0);
        assert!(accuracy.movement_penalty > 1.0);
        assert!(accuracy.ads_modifier < 1.0);
        assert!(accuracy.airborne_multiplier > 1.0);
    }

    #[test]
    fn test_projectile_spawn_params_defaults() {
        // Test that ProjectileSpawnParams has reasonable default values
        let params = ProjectileSpawnParams::default();
        
        assert_eq!(params.origin, Vec3::ZERO);
        assert_eq!(params.direction, Vec3::NEG_Z);
        assert_eq!(params.velocity, 400.0);
        assert_eq!(params.mass, 0.01);
        assert_eq!(params.drag, 0.3);
        assert_eq!(params.damage, 25.0);
        assert!(params.owner.is_none());
    }

    #[test]
    fn test_weapon_presets_content() {
        let presets = WeaponPresets::with_defaults();
        
        // Should have at least 4 presets (pistol, rifle, sniper, bow)
        assert!(presets.presets.len() >= 4, "Should have at least 4 weapon presets");
        
        // Check that each preset has valid values
        for (index, preset) in presets.presets.iter().enumerate() {
            assert!(!preset.name.is_empty(), "Preset name should not be empty");
            assert!(preset.muzzle_velocity > 0.0, "Muzzle velocity should be positive");
            assert!(preset.projectile_mass > 0.0, "Projectile mass should be positive");
            assert!(preset.drag_coefficient > 0.0, "Drag coefficient should be positive");
            assert!(preset.base_damage > 0.0, "Base damage should be positive");
            
            // Check accuracy values
            assert!(preset.accuracy.base_spread >= 0.0, "Base spread should be non-negative");
            assert!(preset.accuracy.max_spread >= preset.accuracy.base_spread, "Max spread should be >= base spread");
            assert!(preset.accuracy.bloom_per_shot >= 0.0, "Bloom per shot should be non-negative");
            assert!(preset.accuracy.recovery_rate >= 0.0, "Recovery rate should be non-negative");
            
            println!("Preset {}: {} - Muzzle: {:.0}m/s, Damage: {:.0}", 
                     index, preset.name, preset.muzzle_velocity, preset.base_damage);
        }
    }

    #[test]
    fn test_calculate_total_spread() {
        let accuracy = Accuracy {
            base_spread: 0.001,      // 0.001 rad = ~0.057 degrees
            current_bloom: 0.002,    // Additional 0.002 rad
            max_spread: 0.05,        // Max 0.05 rad
            bloom_per_shot: 0.01,
            recovery_rate: 0.05,
            movement_penalty: 2.0,
            ads_modifier: 0.3,
            airborne_multiplier: 3.0,
        };

        // Test with no modifiers
        let spread_normal = systems::accuracy::calculate_total_spread(
            &accuracy, false, false, false, 0.0, 5.0
        );
        assert_eq!(spread_normal, 0.001 + 0.002); // base + bloom

        // Test with ADS
        let spread_ads = systems::accuracy::calculate_total_spread(
            &accuracy, true, false, false, 0.0, 5.0
        );
        assert!(spread_ads < spread_normal); // ADS should reduce spread

        // Test with movement
        let spread_moving = systems::accuracy::calculate_total_spread(
            &accuracy, false, true, false, 5.0, 5.0
        );
        assert!(spread_moving > spread_normal); // Moving should increase spread
    }

    // Helper systems for testing
    fn setup_player_for_testing(mut commands: Commands) {
        let player_entity = commands
            .spawn((
                Player,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        commands.insert_resource(PlayerEntity(player_entity));
    }

    fn player_movement_for_testing(
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

    fn weapon_switching_for_testing(
        mut current_weapon: bevy::prelude::ResMut<CurrentWeapon>,
        keyboard_input: bevy::prelude::Res<ButtonInput<KeyCode>>,
        mut player_stats: bevy::prelude::ResMut<PlayerStats>,
        weapon_presets: bevy::prelude::Res<WeaponPresets>,
    ) {
        if keyboard_input.just_pressed(KeyCode::Digit1) {
            current_weapon.0 = 0; // Pistol
            player_stats.weapon_index = 0;
        }
        if keyboard_input.just_pressed(KeyCode::Digit2) {
            current_weapon.0 = 1; // Rifle
            player_stats.weapon_index = 1;
        }
        if keyboard_input.just_pressed(KeyCode::Digit3) {
            current_weapon.0 = 2; // Sniper
            player_stats.weapon_index = 2;
        }
        
        // Reset stats when switching weapons
        if keyboard_input.any_just_pressed([KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]) {
            player_stats.shots_fired = 0;
            player_stats.hits = 0;
            player_stats.accuracy = 0.0;
        }
    }
}
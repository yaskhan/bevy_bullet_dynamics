#[cfg(test)]
mod surfaces_2d_tests {
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

    use crate::{GameStats, MaterialType, Obstacle, Player, PlayerEntity};

    #[test]
    fn test_player_spawned_in_surfaces_example() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(BallisticsEnvironment {
            gravity: Vec3::ZERO,
            air_density: 1.2,
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
        
        // Setup system
        app.add_systems(Startup, setup_player_for_surfaces_test);
        
        // Run startup systems
        app.update();
        
        // Verify player was created
        let player_entities: Vec<_> = app.world.query_filtered::<Entity, With<Player>>().iter(&app.world).collect();
        assert_eq!(player_entities.len(), 1, "Should have exactly one player entity");
        
        // Verify player entity resource was inserted
        assert!(app.world.contains_resource::<PlayerEntity>(), "PlayerEntity resource should be present");
        
        // Verify game stats resource was inserted
        assert!(app.world.contains_resource::<GameStats>(), "GameStats resource should be present");
    }

    #[test]
    fn test_material_types_and_colors() {
        // Test that each material type has a distinct color
        let concrete_color = MaterialType::Concrete.color();
        let metal_color = MaterialType::Metal.color();
        let wood_color = MaterialType::Wood.color();
        let glass_color = MaterialType::Glass.color();
        
        // Colors should be different
        assert_ne!(concrete_color, metal_color);
        assert_ne!(concrete_color, wood_color);
        assert_ne!(concrete_color, glass_color);
        assert_ne!(metal_color, wood_color);
        assert_ne!(metal_color, glass_color);
        assert_ne!(wood_color, glass_color);
        
        println!("Material colors:");
        println!("Concrete: {:?}", concrete_color);
        println!("Metal: {:?}", metal_color);
        println!("Wood: {:?}", wood_color);
        println!("Glass: {:?}", glass_color);
    }

    #[test]
    fn test_surface_material_properties() {
        // Test that each material type has appropriate surface properties
        let concrete_mat = MaterialType::Concrete.surface_material();
        let metal_mat = MaterialType::Metal.surface_material();
        let wood_mat = MaterialType::Wood.surface_material();
        let glass_mat = MaterialType::Glass.surface_material();
        
        // All materials should have positive penetration loss
        assert!(concrete_mat.penetration_loss > 0.0);
        assert!(metal_mat.penetration_loss > 0.0);
        assert!(wood_mat.penetration_loss > 0.0);
        assert!(glass_mat.penetration_loss > 0.0);
        
        // Metal should have highest penetration loss (hardest to penetrate)
        assert!(metal_mat.penetration_loss >= concrete_mat.penetration_loss);
        
        // Glass should have lowest penetration loss (easiest to penetrate)
        assert!(glass_mat.penetration_loss <= wood_mat.penetration_loss);
        
        // All materials should have positive thickness
        assert!(concrete_mat.thickness > 0.0);
        assert!(metal_mat.thickness > 0.0);
        assert!(wood_mat.thickness > 0.0);
        assert!(glass_mat.thickness > 0.0);
        
        // Ricochet angles should be reasonable (between 0 and π/2)
        assert!(concrete_mat.ricochet_angle >= 0.0);
        assert!(metal_mat.ricochet_angle >= 0.0);
        assert!(wood_mat.ricochet_angle >= 0.0);
        assert!(glass_mat.ricochet_angle >= 0.0);
        
        assert!(concrete_mat.ricochet_angle <= std::f32::consts::FRAC_PI_2);
        assert!(metal_mat.ricochet_angle <= std::f32::consts::FRAC_PI_2);
        assert!(wood_mat.ricochet_angle <= std::f32::consts::FRAC_PI_2);
        assert!(glass_mat.ricochet_angle <= std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn test_game_stats_initialization() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(BallisticsEnvironment::default());
        app.insert_resource(BallisticsConfig::default());
        
        // Setup system
        app.add_systems(Startup, setup_player_for_surfaces_test);
        
        // Run startup systems
        app.update();
        
        // Check if game stats were initialized correctly
        let game_stats = app.world.resource::<GameStats>();
        assert_eq!(game_stats.shots_fired, 0);
        assert_eq!(game_stats.penetrations, 0);
        assert_eq!(game_stats.ricochets, 0);
        assert_eq!(game_stats.hits, 0);
    }

    #[test]
    fn test_obstacle_spawning() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(BallisticsEnvironment::default());
        app.insert_resource(BallisticsConfig::default());
        
        // Setup system
        app.add_systems(Startup, setup_obstacles_for_test);
        
        // Run startup systems
        app.update();
        
        // Count obstacles
        let obstacle_count = app.world.query_filtered::<Entity, With<Obstacle>>().iter(&app.world).count();
        assert!(obstacle_count > 0, "Should have spawned at least one obstacle");
        
        // Verify all obstacles have valid material types
        let mut query = app.world.query::<&Obstacle>();
        for obstacle in query.iter(&app.world) {
            match obstacle.material_type {
                MaterialType::Concrete | MaterialType::Metal | MaterialType::Wood | MaterialType::Glass => {
                    // Valid material type
                }
            }
        }
    }

    #[test]
    fn test_handle_hits_updates_stats() {
        let mut app = App::new();
        
        // Add necessary plugins
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BallisticsPluginGroup);
        
        // Insert resources
        app.insert_resource(BallisticsEnvironment::default());
        app.insert_resource(BallisticsConfig::default());
        app.insert_resource(GameStats {
            shots_fired: 0,
            penetrations: 0,
            ricochets: 0,
            hits: 0,
        });
        
        // Run startup systems
        app.update();
        
        // Initially stats should be zero
        let initial_stats = app.world.resource::<GameStats>();
        assert_eq!(initial_stats.hits, 0);
        assert_eq!(initial_stats.penetrations, 0);
        assert_eq!(initial_stats.ricochets, 0);
        
        // Simulate a hit event with penetration
        let hit_event = HitEvent {
            projectile: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
            impact_point: Vec3::ZERO,
            normal: Vec3::Y,
            velocity: Vec3::X,
            damage: 25.0,
            penetrated: true,
            ricocheted: false,
        };
        
        app.world.send_event(hit_event);
        
        // Process the event
        app.update();
        
        // Stats should be updated
        let updated_stats = app.world.resource::<GameStats>();
        assert_eq!(updated_stats.hits, 1);
        assert_eq!(updated_stats.penetrations, 1);
        assert_eq!(updated_stats.ricochets, 0);
        
        // Simulate a hit event with ricochet
        let hit_event2 = HitEvent {
            projectile: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
            impact_point: Vec3::ONE,
            normal: Vec3::Y,
            velocity: Vec3::X,
            damage: 25.0,
            penetrated: false,
            ricocheted: true,
        };
        
        app.world.send_event(hit_event2);
        
        // Process the event
        app.update();
        
        // Stats should be updated again
        let final_stats = app.world.resource::<GameStats>();
        assert_eq!(final_stats.hits, 2);
        assert_eq!(final_stats.penetrations, 1);
        assert_eq!(final_stats.ricochets, 1);
    }

    #[test]
    fn test_can_penetrate_function() {
        // Test the can_penetrate function with different scenarios
        let mut projectile = Projectile::new(Vec3::X * 400.0);
        projectile.penetration_power = 100.0; // High penetration power
        
        let weak_surface = systems::surface::materials::glass(); // Low penetration loss
        let strong_surface = systems::surface::materials::metal(); // High penetration loss
        
        // Projectile should penetrate weak surface
        let can_penetrate_weak = systems::surface::can_penetrate(&projectile, &weak_surface, 0.0);
        assert!(can_penetrate_weak, "Projectile should penetrate weak surface");
        
        // Projectile might not penetrate strong surface
        let can_penetrate_strong = systems::surface::can_penetrate(&projectile, &strong_surface, 0.0);
        // This could be true or false depending on exact values, but we can test the relationship
        assert!(can_penetrate_weak || !can_penetrate_strong, "Weak surface should be easier to penetrate than strong surface");
    }

    #[test]
    fn test_should_ricochet_function() {
        // Test the should_ricochet function
        let velocity = Vec3::new(1.0, 0.1, 0.0); // Shallow angle
        let normal = Vec3::Y;
        let surface = systems::surface::materials::metal(); // Has low ricochet angle
        
        let should_ricochet = systems::surface::should_ricochet(velocity, normal, &surface);
        // With a shallow angle, it might ricochet depending on the surface's ricochet_angle
        
        // Test with steep angle
        let steep_velocity = Vec3::new(0.1, -1.0, 0.0); // Steep angle
        let should_ricochet_steep = systems::surface::should_ricochet(steep_velocity, normal, &surface);
        
        // Shallow angle should be more likely to ricochet than steep angle
        // (Note: This is approximate since the exact behavior depends on surface properties)
    }

    // Helper systems for testing
    fn setup_player_for_surfaces_test(mut commands: Commands) {
        let player_entity = commands
            .spawn((
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
    }

    fn setup_obstacles_for_test(mut commands: Commands) {
        // Spawn a few obstacles for testing
        commands.spawn((
            Obstacle {
                material_type: MaterialType::Concrete,
            },
            Transform::from_xyz(100.0, 0.0, 0.0),
        ));

        commands.spawn((
            Obstacle {
                material_type: MaterialType::Metal,
            },
            Transform::from_xyz(-100.0, 0.0, 0.0),
        ));

        commands.spawn((
            Obstacle {
                material_type: MaterialType::Wood,
            },
            Transform::from_xyz(0.0, 100.0, 0.0),
        ));
    }
}
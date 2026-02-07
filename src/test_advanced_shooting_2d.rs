#[cfg(test)]
mod tests {
    use bevy::{
        app::{App, Startup, Update},
        ecs::system::Commands,
        math::Vec3,
        prelude::{Entity, Resource, With},
        transform::components::Transform,
    };
    use bevy_bullet_dynamics::prelude::*;

    use crate::{CurrentWeapon, Player, PlayerEntity, PlayerStats};

    #[test]
    fn test_player_spawned() {
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
        app.add_systems(Startup, setup_test_player);
        
        // Run startup systems
        app.update();
        
        // Check if player was spawned
        let player_count = app.world.query::<&Player>().iter(&app.world).count();
        assert_eq!(player_count, 1, "Player should be spawned");
        
        // Check if player entity resource was inserted
        let has_player_entity = app.world.contains_resource::<PlayerEntity>();
        assert!(has_player_entity, "PlayerEntity resource should be inserted");
    }

    #[test]
    fn test_weapon_switching() {
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
        app.insert_resource(CurrentWeapon(0));
        app.insert_resource(PlayerStats {
            weapon_index: 0,
            shots_fired: 0,
            hits: 0,
            accuracy: 0.0,
        });
        
        // Setup system
        app.add_systems(Startup, setup_test_player);
        app.add_systems(Update, test_weapon_switching_system);
        
        // Run startup systems
        app.update();
        
        // Initially weapon index should be 0
        let player_stats = app.world.resource::<PlayerStats>();
        assert_eq!(player_stats.weapon_index, 0);
        
        // Run update to switch weapon
        app.update();
        
        // After switching, weapon index should be 1
        let player_stats_after = app.world.resource::<PlayerStats>();
        assert_eq!(player_stats_after.weapon_index, 1);
    }

    #[test]
    fn test_player_stats_initialization() {
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
        app.add_systems(Startup, setup_test_player_with_stats);
        
        // Run startup systems
        app.update();
        
        // Check if player stats were initialized correctly
        let player_stats = app.world.resource::<PlayerStats>();
        assert_eq!(player_stats.shots_fired, 0);
        assert_eq!(player_stats.hits, 0);
        assert_eq!(player_stats.accuracy, 0.0);
        assert_eq!(player_stats.weapon_index, 0);
    }

    #[test]
    fn test_weapon_presets_loaded() {
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
        
        // Run startup systems
        app.update();
        
        // Check if weapon presets were loaded
        let weapon_presets = app.world.resource::<WeaponPresets>();
        assert!(!weapon_presets.presets.is_empty(), "Weapon presets should not be empty");
        
        // Check if we have at least 3 presets (pistol, rifle, sniper)
        assert!(weapon_presets.presets.len() >= 3, "Should have at least 3 weapon presets");
        
        // Check specific preset names
        if weapon_presets.presets.len() >= 3 {
            assert_eq!(weapon_presets.presets[0].name, "Pistol");
            assert_eq!(weapon_presets.presets[1].name, "Rifle");
            assert_eq!(weapon_presets.presets[2].name, "Sniper");
        }
    }

    // Helper systems for tests
    fn setup_test_player(mut commands: Commands) {
        let player_entity = commands
            .spawn((
                Player,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        commands.insert_resource(PlayerEntity(player_entity));
        commands.insert_resource(CurrentWeapon(0));
        commands.insert_resource(PlayerStats {
            weapon_index: 0,
            shots_fired: 0,
            hits: 0,
            accuracy: 0.0,
        });
    }

    fn setup_test_player_with_stats(mut commands: Commands) {
        let player_entity = commands
            .spawn((
                Player,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        commands.insert_resource(PlayerEntity(player_entity));
        commands.insert_resource(CurrentWeapon(0));
        commands.insert_resource(PlayerStats {
            weapon_index: 0,
            shots_fired: 0,
            hits: 0,
            accuracy: 0.0,
        });
    }

    fn test_weapon_switching_system(
        mut current_weapon: bevy::prelude::ResMut<CurrentWeapon>,
        mut player_stats: bevy::prelude::ResMut<PlayerStats>,
    ) {
        // Simulate weapon switching
        current_weapon.0 = 1;
        player_stats.weapon_index = 1;
    }
}
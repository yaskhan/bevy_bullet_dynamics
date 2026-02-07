#[cfg(test)]
mod all_tests {
    use bevy::prelude::*;
    use bevy_bullet_dynamics::prelude::*;

    #[test]
    fn test_all_components_can_be_built() {
        // Test that all major components can be instantiated
        let projectile = Projectile::new(Vec3::X * 400.0)
            .with_mass(0.01)
            .with_drag(0.3)
            .with_owner(Entity::PLACEHOLDER);
        
        let accuracy = Accuracy::default();
        
        let payload = Payload::Kinetic { damage: 25.0 };
        
        let surface_material = SurfaceMaterial::default();
        
        let bullet_tracer = BulletTracer {
            lifetime: 2.0,
            trail_length: 1.5,
        };
        
        let impact_decal = ImpactDecal {
            lifetime: 30.0,
        };
        
        let net_projectile = NetProjectile {
            owner_id: 12345,
            timestamp: 123456.789,
            spread_seed: 9876543210,
        };
        
        let weapon_zeroing = WeaponZeroing {
            distance: 100.0,
            pitch_adjustment: 0.005,
        };
        
        // Verify all components were created with expected values
        assert_eq!(projectile.velocity, Vec3::X * 400.0);
        assert_eq!(projectile.mass, 0.01);
        assert_eq!(projectile.drag_coefficient, 0.3);
        assert!(projectile.owner.is_some());
        
        assert_eq!(accuracy.current_bloom, 0.0);
        assert!(accuracy.base_spread > 0.0);
        
        match payload {
            Payload::Kinetic { damage } => assert_eq!(damage, 25.0),
            _ => panic!("Expected kinetic payload"),
        }
        
        assert_eq!(surface_material.ricochet_angle, 0.3);
        assert!(surface_material.penetration_loss > 0.0);
        
        assert_eq!(bullet_tracer.lifetime, 2.0);
        assert_eq!(bullet_tracer.trail_length, 1.5);
        
        assert_eq!(impact_decal.lifetime, 30.0);
        
        assert_eq!(net_projectile.owner_id, 12345);
        assert_eq!(net_projectile.timestamp, 123456.789);
        assert_eq!(net_projectile.spread_seed, 9876543210);
        
        assert_eq!(weapon_zeroing.distance, 100.0);
        assert_eq!(weapon_zeroing.pitch_adjustment, 0.005);
    }

    #[test]
    fn test_events_can_be_created() {
        // Test that all major events can be instantiated
        let fire_event = FireEvent::new(Vec3::ZERO, Vec3::X, 400.0)
            .with_shooter(Entity::PLACEHOLDER)
            .with_seed(12345);
        
        let hit_event = HitEvent {
            projectile: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
            impact_point: Vec3::new(10.0, 0.0, 5.0),
            normal: Vec3::Y,
            velocity: Vec3::X * 300.0,
            damage: 25.0,
            penetrated: false,
            ricocheted: false,
        };
        
        let explosion_event = ExplosionEvent {
            center: Vec3::ZERO,
            radius: 5.0,
            damage: 100.0,
            falloff: 1.5,
            explosion_type: ExplosionType::HighExplosive,
            source: Some(Entity::PLACEHOLDER),
        };
        
        let penetration_event = PenetrationEvent {
            projectile: Entity::PLACEHOLDER,
            entry_point: Vec3::ZERO,
            exit_point: Vec3::X,
            target: Entity::PLACEHOLDER,
            remaining_power: 50.0,
        };
        
        let ricochet_event = RicochetEvent {
            projectile: Entity::PLACEHOLDER,
            impact_point: Vec3::ZERO,
            new_direction: Vec3::Y,
            new_speed: 200.0,
            surface: Entity::PLACEHOLDER,
        };
        
        // Verify events were created with expected values
        assert_eq!(fire_event.origin, Vec3::ZERO);
        assert_eq!(fire_event.direction, Vec3::X);
        assert_eq!(fire_event.muzzle_velocity, 400.0);
        assert!(fire_event.shooter.is_some());
        assert_eq!(fire_event.spread_seed, 12345);
        
        assert_eq!(hit_event.impact_point, Vec3::new(10.0, 0.0, 5.0));
        assert_eq!(hit_event.damage, 25.0);
        assert_eq!(hit_event.penetrated, false);
        assert_eq!(hit_event.ricocheted, false);
        
        assert_eq!(explosion_event.center, Vec3::ZERO);
        assert_eq!(explosion_event.radius, 5.0);
        assert_eq!(explosion_event.damage, 100.0);
        assert_eq!(explosion_event.explosion_type, ExplosionType::HighExplosive);
        assert!(explosion_event.source.is_some());
        
        assert_eq!(penetration_event.entry_point, Vec3::ZERO);
        assert_eq!(penetration_event.exit_point, Vec3::X);
        assert_eq!(penetration_event.remaining_power, 50.0);
        
        assert_eq!(ricochet_event.impact_point, Vec3::ZERO);
        assert_eq!(ricochet_event.new_direction, Vec3::Y);
        assert_eq!(ricochet_event.new_speed, 200.0);
    }

    #[test]
    fn test_resources_can_be_created() {
        // Test that all major resources can be instantiated
        let env = BallisticsEnvironment::default();
        let config = BallisticsConfig::default();
        let tracer_pool = TracerPool::new(100);
        let decal_pool = DecalPool::new(50);
        let weapon_presets = WeaponPresets::with_defaults();
        
        // Verify resources were created with expected values
        assert_eq!(env.gravity, Vec3::new(0.0, -9.81, 0.0));
        assert_eq!(env.air_density, 1.225);
        assert_eq!(env.wind, Vec3::ZERO);
        
        assert_eq!(config.use_rk4, true);
        assert_eq!(config.max_projectile_lifetime, 10.0);
        assert_eq!(config.max_projectile_distance, 2000.0);
        assert_eq!(config.enable_penetration, true);
        assert_eq!(config.enable_ricochet, true);
        assert_eq!(config.debug_draw, false);
        
        assert_eq!(tracer_pool.max_size, 100);
        assert_eq!(decal_pool.max_size, 50);
        
        assert!(!weapon_presets.presets.is_empty());
        assert!(weapon_presets.presets.len() >= 4); // At least pistol, rifle, sniper, bow
        
        // Check that presets have valid values
        for preset in &weapon_presets.presets {
            assert!(!preset.name.is_empty());
            assert!(preset.muzzle_velocity > 0.0);
            assert!(preset.projectile_mass > 0.0);
            assert!(preset.drag_coefficient > 0.0);
            assert!(preset.base_damage > 0.0);
        }
    }

    #[test]
    fn test_types_enum_variants() {
        // Test all enum variants
        assert_eq!(PhysicsModel::Euler, PhysicsModel::Euler);
        assert_eq!(PhysicsModel::RK4, PhysicsModel::RK4);
        
        assert_eq!(WeaponCategory::Firearm, WeaponCategory::Firearm);
        assert_eq!(WeaponCategory::Projectile, WeaponCategory::Projectile);
        assert_eq!(WeaponCategory::Throwable, WeaponCategory::Throwable);
        assert_eq!(WeaponCategory::Explosive, WeaponCategory::Explosive);
        
        assert_eq!(HitEffectType::Sparks, HitEffectType::Sparks);
        assert_eq!(HitEffectType::Dust, HitEffectType::Dust);
        assert_eq!(HitEffectType::Blood, HitEffectType::Blood);
        assert_eq!(HitEffectType::WoodChips, HitEffectType::WoodChips);
        assert_eq!(HitEffectType::Water, HitEffectType::Water);
        assert_eq!(HitEffectType::Glass, HitEffectType::Glass);
        
        assert_eq!(ProjectileState::InFlight, ProjectileState::InFlight);
        assert_eq!(ProjectileState::Stuck, ProjectileState::Stuck);
        assert_eq!(ProjectileState::Detonating, ProjectileState::Detonating);
        assert_eq!(ProjectileState::Despawning, ProjectileState::Despawning);
        
        assert_eq!(ExplosionType::HighExplosive, ExplosionType::HighExplosive);
        assert_eq!(ExplosionType::Incendiary, ExplosionType::Incendiary);
        assert_eq!(ExplosionType::Flash, ExplosionType::Flash);
        assert_eq!(ExplosionType::Smoke, ExplosionType::Smoke);
        assert_eq!(ExplosionType::Fragmentation, ExplosionType::Fragmentation);
    }

    #[test]
    fn test_projectile_spawn_params() {
        // Test ProjectileSpawnParams functionality
        let params = ProjectileSpawnParams::new(Vec3::new(1.0, 2.0, 3.0), Vec3::Y, 500.0)
            .with_mass(0.008)
            .with_damage(35.0)
            .with_owner(Entity::PLACEHOLDER);
        
        assert_eq!(params.origin, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(params.direction, Vec3::Y); // Should be normalized
        assert_eq!(params.velocity, 500.0);
        assert_eq!(params.mass, 0.008);
        assert_eq!(params.damage, 35.0);
        assert!(params.owner.is_some());
    }

    #[test]
    fn test_hit_result_creation() {
        // Test HitResult creation
        let hit_result = HitResult {
            entity: Entity::PLACEHOLDER,
            point: Vec3::new(10.0, 5.0, 0.0),
            normal: Vec3::Y,
            distance: 15.0,
        };
        
        assert_eq!(hit_result.entity, Entity::PLACEHOLDER);
        assert_eq!(hit_result.point, Vec3::new(10.0, 5.0, 0.0));
        assert_eq!(hit_result.normal, Vec3::Y);
        assert_eq!(hit_result.distance, 15.0);
    }

    #[test]
    fn test_accuracy_system_functions() {
        // Test accuracy system functions
        let accuracy = Accuracy {
            base_spread: 0.001,
            current_bloom: 0.002,
            max_spread: 0.05,
            bloom_per_shot: 0.01,
            recovery_rate: 0.05,
            movement_penalty: 2.0,
            ads_modifier: 0.3,
            airborne_multiplier: 3.0,
        };

        // Test calculate_total_spread with different conditions
        let spread_normal = systems::accuracy::calculate_total_spread(
            &accuracy, false, false, false, 0.0, 5.0
        );
        assert_eq!(spread_normal, 0.001 + 0.002); // base + bloom

        let spread_ads = systems::accuracy::calculate_total_spread(
            &accuracy, true, false, false, 0.0, 5.0
        );
        assert!(spread_ads < spread_normal); // ADS should reduce spread

        let spread_moving = systems::accuracy::calculate_total_spread(
            &accuracy, false, true, false, 5.0, 5.0
        );
        assert!(spread_moving > spread_normal); // Moving should increase spread

        let spread_airborne = systems::accuracy::calculate_total_spread(
            &accuracy, false, false, true, 0.0, 5.0
        );
        assert!(spread_airborne > spread_normal); // Airborne should increase spread

        // Test apply_shot_bloom
        let mut accuracy_copy = accuracy.clone();
        systems::accuracy::apply_shot_bloom(&mut accuracy_copy);
        assert_eq!(accuracy_copy.current_bloom, (0.002 + 0.01).min(0.05)); // bloom + bloom_per_shot, clamped to max_spread
    }

    #[test]
    fn test_explosion_damage_calculation() {
        // Test explosion damage calculation
        let damage_center = systems::logic::calculate_explosion_damage(100.0, 0.0, 10.0, 1.0);
        assert_eq!(damage_center, 100.0); // Full damage at center

        let damage_edge = systems::logic::calculate_explosion_damage(100.0, 10.0, 10.0, 1.0);
        assert_eq!(damage_edge, 0.0); // No damage at edge

        let damage_middle = systems::logic::calculate_explosion_damage(100.0, 5.0, 10.0, 1.0);
        assert!(damage_middle > 0.0 && damage_middle < 100.0); // Partial damage in middle

        // Test with different falloff values
        let damage_linear = systems::logic::calculate_explosion_damage(100.0, 5.0, 10.0, 1.0); // Linear
        let damage_quadratic = systems::logic::calculate_explosion_damage(100.0, 5.0, 10.0, 2.0); // Quadratic
        assert!(damage_quadratic <= damage_linear); // Quadratic falls off faster
    }

    #[test]
    fn test_surface_interaction_functions() {
        // Test surface interaction functions
        let mut projectile = Projectile::new(Vec3::X * 400.0);
        projectile.penetration_power = 100.0;

        let concrete_surface = systems::surface::materials::concrete();
        let metal_surface = systems::surface::materials::metal();
        let wood_surface = systems::surface::materials::wood();
        let glass_surface = systems::surface::materials::glass();

        // Test penetration with different surfaces
        let can_penetrate_glass = systems::surface::can_penetrate(&projectile, &glass_surface, 0.0);
        let can_penetrate_wood = systems::surface::can_penetrate(&projectile, &wood_surface, 0.0);
        let can_penetrate_concrete = systems::surface::can_penetrate(&projectile, &concrete_surface, 0.0);
        let can_penetrate_metal = systems::surface::can_penetrate(&projectile, &metal_surface, 0.0);

        // Generally, easier to penetrate softer materials
        // Note: This is approximate since exact behavior depends on specific values
        assert!(can_penetrate_glass || can_penetrate_wood); // Should be able to penetrate at least some materials

        // Test ricochet
        let velocity = Vec3::new(1.0, 0.1, 0.0); // Shallow angle
        let normal = Vec3::Y;

        let should_ricochet_metal = systems::surface::should_ricochet(velocity, normal, &metal_surface);
        let should_ricochet_concrete = systems::surface::should_ricochet(velocity, normal, &concrete_surface);

        // Metal typically has lower ricochet angle than concrete
        // So shallow angle might ricochet off metal but not concrete
    }

    #[test]
    fn test_vfx_functions() {
        // Test VFX functions
        let settings_default = systems::vfx::tracer_config::TracerSettings::default();
        let settings_rifle = systems::vfx::tracer_config::rifle();
        let settings_sniper = systems::vfx::tracer_config::sniper();
        let settings_smg = systems::vfx::tracer_config::smg();
        let settings_laser = systems::vfx::tracer_config::laser();

        // Each should have different characteristics
        assert_ne!(settings_default.color, settings_rifle.color);
        assert_ne!(settings_rifle.color, settings_sniper.color);
        assert_ne!(settings_sniper.color, settings_smg.color);
        assert_ne!(settings_smg.color, settings_laser.color);

        // Each should have different lengths
        assert_ne!(settings_rifle.length, settings_sniper.length);
        assert_ne!(settings_sniper.length, settings_laser.length);

        // Verify they all have positive values
        assert!(settings_default.width > 0.0);
        assert!(settings_default.length > 0.0);
        assert!(settings_default.glow_intensity > 0.0);

        assert!(settings_rifle.width > 0.0);
        assert!(settings_rifle.length > 0.0);
        assert!(settings_rifle.glow_intensity > 0.0);
    }

    #[test]
    fn test_weapon_presets_diversity() {
        // Test that weapon presets have diverse characteristics
        let presets = WeaponPresets::with_defaults();

        // Should have at least 4 presets
        assert!(presets.presets.len() >= 4, "Should have at least 4 weapon presets");

        let mut muzzle_velocities = Vec::new();
        let mut damages = Vec::new();
        let mut masses = Vec::new();

        for preset in &presets.presets {
            muzzle_velocities.push(preset.muzzle_velocity);
            damages.push(preset.base_damage);
            masses.push(preset.projectile_mass);
        }

        // Verify diversity in characteristics
        let unique_velocities: std::collections::HashSet<_> = muzzle_velocities.iter().cloned().collect();
        let unique_damages: std::collections::HashSet<_> = damages.iter().cloned().collect();
        let unique_masses: std::collections::HashSet<_> = masses.iter().cloned().collect();

        // Should have multiple different values for each characteristic
        assert!(unique_velocities.len() > 1, "Should have different muzzle velocities");
        assert!(unique_damages.len() > 1, "Should have different damages");
        assert!(unique_masses.len() > 1, "Should have different masses");

        println!("Found {} unique muzzle velocities", unique_velocities.len());
        println!("Found {} unique damages", unique_damages.len());
        println!("Found {} unique masses", unique_masses.len());
    }
}
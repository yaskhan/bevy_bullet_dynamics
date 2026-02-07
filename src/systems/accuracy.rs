//! Accuracy system - dynamic spread and bloom calculation.

use bevy::prelude::*;

use crate::components::Accuracy;

/// Update bloom recovery for all weapons with accuracy components.
///
/// Runs every frame to smoothly decrease bloom over time.
/// 
/// # Arguments
/// * `time` - Bevy Time resource to get delta time
/// * `query` - Query for mutable references to Accuracy components
pub fn update_bloom(time: Res<Time>, mut query: Query<&mut Accuracy>) {
    let dt = time.delta_secs();

    for mut accuracy in query.iter_mut() {
        // Recover bloom over time
        accuracy.current_bloom -= accuracy.recovery_rate * dt;
        accuracy.current_bloom = accuracy.current_bloom.max(0.0);
    }
}

/// Calculate total spread angle based on player state.
///
/// Returns the final spread angle in radians.
/// 
/// # Arguments
/// * `accuracy` - Reference to the Accuracy component
/// * `is_aiming` - Whether the player is aiming down sights
/// * `is_moving` - Whether the player is moving
/// * `is_airborne` - Whether the player is in the air
/// * `movement_speed` - Current movement speed of the player
/// * `max_speed` - Maximum possible movement speed of the player
/// 
/// # Returns
/// The calculated total spread angle in radians
pub fn calculate_total_spread(
    accuracy: &Accuracy,
    is_aiming: bool,
    is_moving: bool,
    is_airborne: bool,
    movement_speed: f32,
    max_speed: f32,
) -> f32 {
    // Start with base spread + accumulated bloom
    let mut total_spread = accuracy.base_spread + accuracy.current_bloom;

    // Movement penalty (scaled by movement speed)
    if is_moving && max_speed > 0.0 {
        let speed_ratio = (movement_speed / max_speed).min(1.0);
        total_spread += accuracy.movement_penalty * speed_ratio * accuracy.base_spread;
    }

    // Airborne penalty (multiplicative)
    if is_airborne {
        total_spread *= accuracy.airborne_multiplier;
    }

    // ADS bonus (multiplicative reduction)
    if is_aiming {
        total_spread *= accuracy.ads_modifier;
    }

    // Clamp to max spread
    total_spread.min(accuracy.max_spread)
}

/// Apply bloom increase after firing.
/// 
/// Increases the current bloom value based on the bloom_per_shot property,
/// clamping to the maximum spread.
/// 
/// # Arguments
/// * `accuracy` - Mutable reference to the Accuracy component
pub fn apply_shot_bloom(accuracy: &mut Accuracy) {
    accuracy.current_bloom = (accuracy.current_bloom + accuracy.bloom_per_shot).min(accuracy.max_spread);
}

/// Generate a random direction within the spread cone.
///
/// Uses Gaussian distribution for more realistic center-weighted spread.
/// 
/// # Arguments
/// * `base_direction` - The original direction vector before applying spread
/// * `spread_angle` - The maximum spread angle in radians
/// * `seed` - Random seed for deterministic spread calculation (important for networking)
/// 
/// # Returns
/// A new direction vector with spread applied
pub fn apply_spread_to_direction(base_direction: Vec3, spread_angle: f32, seed: u64) -> Vec3 {
    use rand::prelude::*;
    use rand_distr::{Distribution, Normal};

    // Create seeded RNG for deterministic spread (networking)
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    // Gaussian distribution for spread (center-weighted)
    let normal = Normal::new(0.0, spread_angle as f64 / 3.0).unwrap_or(Normal::new(0.0, 0.01).unwrap());

    let angle_x = normal.sample(&mut rng) as f32;
    let angle_y = normal.sample(&mut rng) as f32;

    // Create rotation from spread angles
    let rotation = Quat::from_euler(EulerRot::XYZ, angle_x, angle_y, 0.0);

    // Apply rotation to base direction
    (rotation * base_direction).normalize()
}

/// Create accuracy preset for different weapon types.
pub mod presets {
    use super::*;

    /// Creates an Accuracy configuration suitable for a pistol.
    /// 
    /// Pistol characteristics:
    /// - Moderate base spread (0.003 rad ≈ 0.17°)
    /// - Medium maximum spread (0.08 rad ≈ 4.6°)
    /// - Moderate bloom per shot (0.015 rad)
    /// - Fast recovery rate (0.08 rad/s)
    /// - Moderate ADS improvement (50% accuracy boost)
    /// 
    /// # Returns
    /// An Accuracy instance configured for a pistol
    pub fn pistol() -> Accuracy {
        Accuracy {
            base_spread: 0.003,
            max_spread: 0.08,
            bloom_per_shot: 0.015,
            recovery_rate: 0.08,
            movement_penalty: 1.5,
            ads_modifier: 0.5,
            airborne_multiplier: 2.5,
            ..Default::default()
        }
    }

    /// Creates an Accuracy configuration suitable for a rifle.
    /// 
    /// Rifle characteristics:
    /// - Tight base spread (0.001 rad ≈ 0.06°)
    /// - Medium maximum spread (0.06 rad ≈ 3.4°)
    /// - Moderate bloom per shot (0.02 rad)
    /// - Moderate recovery rate (0.04 rad/s)
    /// - Good ADS improvement (70% accuracy boost)
    /// 
    /// # Returns
    /// An Accuracy instance configured for a rifle
    pub fn rifle() -> Accuracy {
        Accuracy {
            base_spread: 0.001,
            max_spread: 0.06,
            bloom_per_shot: 0.02,
            recovery_rate: 0.04,
            movement_penalty: 2.0,
            ads_modifier: 0.3,
            airborne_multiplier: 3.0,
            ..Default::default()
        }
    }

    /// Creates an Accuracy configuration suitable for a sniper rifle.
    /// 
    /// Sniper characteristics:
    /// - Very tight base spread (0.0005 rad ≈ 0.03°)
    /// - Low maximum spread (0.04 rad ≈ 2.3°)
    /// - High bloom per shot (0.03 rad)
    /// - Slow recovery rate (0.02 rad/s)
    /// - Excellent ADS improvement (90% accuracy boost)
    /// 
    /// # Returns
    /// An Accuracy instance configured for a sniper rifle
    pub fn sniper() -> Accuracy {
        Accuracy {
            base_spread: 0.0005,
            max_spread: 0.04,
            bloom_per_shot: 0.03,
            recovery_rate: 0.02,
            movement_penalty: 3.0,
            ads_modifier: 0.1,
            airborne_multiplier: 5.0,
            ..Default::default()
        }
    }

    /// Creates an Accuracy configuration suitable for a shotgun.
    /// 
    /// Shotgun characteristics:
    /// - Wide base spread (0.02 rad ≈ 1.15°)
    /// - High maximum spread (0.1 rad ≈ 5.7°)
    /// - Very low bloom per shot (0.005 rad)
    /// - Fast recovery rate (0.1 rad/s)
    /// - Minimal ADS improvement (30% accuracy boost)
    /// 
    /// # Returns
    /// An Accuracy instance configured for a shotgun
    pub fn shotgun() -> Accuracy {
        Accuracy {
            base_spread: 0.02,
            max_spread: 0.1,
            bloom_per_shot: 0.005,
            recovery_rate: 0.1,
            movement_penalty: 1.0,
            ads_modifier: 0.7,
            airborne_multiplier: 1.5,
            ..Default::default()
        }
    }

    /// Creates an Accuracy configuration suitable for a submachine gun (SMG).
    /// 
    /// SMG characteristics:
    /// - Moderate base spread (0.004 rad ≈ 0.23°)
    /// - High maximum spread (0.1 rad ≈ 5.7°)
    /// - Low bloom per shot (0.008 rad)
    /// - Very fast recovery rate (0.12 rad/s)
    /// - Moderate ADS improvement (60% accuracy boost)
    /// 
    /// # Returns
    /// An Accuracy instance configured for an SMG
    pub fn smg() -> Accuracy {
        Accuracy {
            base_spread: 0.004,
            max_spread: 0.1,
            bloom_per_shot: 0.008,
            recovery_rate: 0.12,
            movement_penalty: 0.8,
            ads_modifier: 0.4,
            airborne_multiplier: 2.0,
            ..Default::default()
        }
    }

    /// Creates an Accuracy configuration suitable for a bow.
    /// 
    /// Bow characteristics:
    /// - Moderate base spread (0.002 rad ≈ 0.11°)
    /// - Low maximum spread (0.03 rad ≈ 1.7°)
    /// - No bloom per shot (single shot weapon)
    /// - No recovery (single shot weapon)
    /// - Good ADS improvement (80% accuracy boost)
    /// 
    /// # Returns
    /// An Accuracy instance configured for a bow
    pub fn bow() -> Accuracy {
        Accuracy {
            base_spread: 0.002,
            max_spread: 0.03,
            bloom_per_shot: 0.0, // No bloom for single-shot
            recovery_rate: 0.0,
            movement_penalty: 2.5,
            ads_modifier: 0.2,
            airborne_multiplier: 4.0,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spread_calculation_base() {
        let accuracy = Accuracy::default();
        let spread = calculate_total_spread(&accuracy, false, false, false, 0.0, 5.0);
        assert_eq!(spread, accuracy.base_spread);
    }

    #[test]
    fn test_spread_calculation_ads() {
        let accuracy = Accuracy::default();
        let spread = calculate_total_spread(&accuracy, true, false, false, 0.0, 5.0);
        assert!(spread < accuracy.base_spread);
    }

    #[test]
    fn test_spread_calculation_moving() {
        let accuracy = Accuracy::default();
        let spread = calculate_total_spread(&accuracy, false, true, false, 5.0, 5.0);
        assert!(spread > accuracy.base_spread);
    }

    #[test]
    fn test_bloom_accumulation() {
        let mut accuracy = Accuracy::default();
        assert_eq!(accuracy.current_bloom, 0.0);

        apply_shot_bloom(&mut accuracy);
        assert_eq!(accuracy.current_bloom, accuracy.bloom_per_shot);

        apply_shot_bloom(&mut accuracy);
        assert_eq!(accuracy.current_bloom, accuracy.bloom_per_shot * 2.0);
    }
}

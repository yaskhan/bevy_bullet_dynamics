# Bevy Bullet Dynamics

🎯 **High-precision, high-performance ballistics simulation plugin for Bevy 0.18**

A production-ready physics-based projectile system designed for stability, efficiency, and accuracy. Supports everything from pistols and sniper rifles to grenades and complex throwables, with full 2D/3D support and a robust multiplayer architecture.

---

## 🚀 Key Improvements (v0.1.0+)

- **Infinite Stability**: Comprehensive memory leak resolution across VFX and physics systems.
- **Aggressive Object Pooling**: Optimized `TracerPool` and `DecalPool` for low-allocation rendering.
- **Multithreading Ready**: RK4 integration and collision detection optimized for `FixedUpdate` parallelization.
- **Smart Asset Management**: Centralized `BallisticsAssets` resource for zero-redundancy mesh and material usage.

## Features

- **RK4 Integration**: Accurate Runge-Kutta 4th order physics for realistic bullet trajectories.
- **Multiple Weapon Types**: Pistols, rifles, snipers, bows, SMGs, shotguns with unique ballistic profiles.
- **Advanced Throwables**: Grenades, flashbangs, smoke canisters with proximity and timed detonation.
- **Dynamic Accuracy**: Complex spread model with bloom, recovery, ADS modifiers, and movement penalties.
- **Surface Interactions**: Deep material system with penetration, ricochets, and impact effects.
- **Multiplayer Architecture**: Built-in support for Client-Side Prediction (CSP) and server authority using `bevy_renet2`.
- **Cross-Dimensional**: Native 2D and 3D support via feature flags.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_bullet_dynamics = "0.1"

# Enable networking
# bevy_bullet_dynamics = { version = "0.1", features = ["netcode"] }
```

## Quick Start

```rust
use bevy::prelude::*;
use bevy_bullet_dynamics::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BallisticsPluginGroup)
        .add_systems(Update, fire_weapon)
        .run();
}

fn fire_weapon(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let velocity = Vec3::NEG_Z * 900.0;
        
        commands.spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            Projectile::new(velocity),
            Payload::Kinetic { damage: 35.0 },
            ProjectileLogic::Impact,
        ));
    }
}
```

## Performance & Stability

The library is designed for long-running processes (multiplayer servers, persistent worlds):

1.  **Object Pooling**: Tracers and decals are reused via `TracerPool` and `DecalPool`. Default capacity is 256.
2.  **Lifetime Management**: Every projectile tracks its `age` and `distance_travelled`, auto-despawning based on user-defined limits or when energy is depleted.
3.  **No Allocation Updates**: Critical systems avoid heap allocation in the main simulation loop.
4.  **Resource Caching**: Meshes and materials are pre-loaded into `BallisticsAssets` to prevent GPU memory bloat.

## Architecture

### Plugin Group
Register all subsystems at once or pick only what you need:

```rust
BallisticsPluginGroup
├── BallisticsCorePlugin    // Physics, kinetics, lifetime, collision logic
├── BallisticsSurfacePlugin // Material detection, penetration, ricochets
├── BallisticsVfxPlugin     // High-perf tracers, pooled decals, muzzle flashes
└── BallisticsDebugPlugin   // Optional gizmo-based visualization
```

### Multiplayer (Netcode Feature)
The `netcode` feature provides:
- **Server Authority**: Authoritative simulation of projectiles with `NetworkId` reconciliation.
- **Client Prediction**: Immediate visual feedback for the local player with automatic "ghost" entity cleanup.
- **Low Bandwidth**: Optimized snapshots for projectile state synchronization.

## Feature Flags

| Feature | Description |
|---------|-------------|
| `dim3` | 3D physics using `avian3d` (default) |
| `dim2` | 2D physics using `avian2d` |
| `netcode` | Multiplayer support with `bevy_renet2` |

## Documentation & Examples

Refer to the included examples for implementation patterns:
- `basic_shooting`: Standard 3D implementation.
- `grenades`: Complex throwing logic and payloads.
- `multiplayer`: Full client-server demonstration with prediction.
- `surfaces_2d`: Rich 2D interaction and material physics.

## License

MIT OR Apache-2.0

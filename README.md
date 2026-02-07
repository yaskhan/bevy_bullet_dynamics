# bevy_bullet_dynamics

🎯 **High-precision ballistics simulation plugin for Bevy 0.18**

A comprehensive physics-based projectile system supporting various weapon types from pistols to sniper rifles, bows, grenades, with 2D/3D support and client-server architecture.

---

## Features

- **RK4 Integration**: Accurate Runge-Kutta 4th order physics for realistic bullet trajectories
- **Multiple Weapon Types**: Pistols, rifles, snipers, bows, SMGs, shotguns
- **Throwables**: Frag grenades, flashbangs, smoke grenades, molotov cocktails
- **Dynamic Accuracy System**: Bloom, spread, ADS modifiers, movement penalties
- **Surface Interactions**: Penetration, ricochets, material-based effects
- **Object Pooling**: High-performance tracer and decal management
- **2D/3D Support**: Feature flags for dimensional selection
- **Networking Ready**: Client-server architecture with prediction support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_bullet_dynamics = "0.1"

# Optional features
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
        let origin = Vec3::new(0.0, 1.0, 0.0);
        let direction = Vec3::NEG_Z;
        let velocity = direction * 900.0; // Rifle velocity
        
        commands.spawn((
            Transform::from_translation(origin),
            Projectile::new(velocity),
            Payload::Kinetic { damage: 35.0 },
            ProjectileLogic::Impact,
        ));
    }
}
```

## Architecture

### Plugin Group

```rust
BallisticsPluginGroup
├── BallisticsCorePlugin    // Physics, kinematics, events
├── BallisticsSurfacePlugin // Penetration, ricochets
└── BallisticsVfxPlugin     // Tracers, decals, effects
```

### Core Components

| Component | Description |
|-----------|-------------|
| `Projectile` | Physics properties: velocity, mass, drag, penetration |
| `Accuracy` | Spread system: bloom, recovery, modifiers |
| `ProjectileLogic` | Behavior: Impact, Timed, Proximity, Sticky |
| `Payload` | Effect: Kinetic, Explosive, Incendiary, Flash, Smoke |
| `SurfaceMaterial` | Surface properties: ricochet angle, penetration loss |

### Physics

The system uses RK4 integration for accurate ballistic simulation:

```
Drag Force = 0.5 × ρ × v² × Cd × A
Acceleration = Gravity - (Drag / Mass)
```

Environmental factors:
- Gravity
- Air density (altitude/temperature adjusted)
- Wind

### Accuracy System

Dynamic spread based on player state:

```
Total Spread = (Base + Bloom) × Movement × Airborne × ADS
```

Factors:
- **Bloom**: Accumulates per shot, recovers over time
- **Movement**: Penalty based on velocity
- **Airborne**: Large penalty when jumping
- **ADS**: Reduction when aiming down sights

## Feature Flags

| Feature | Description |
|---------|-------------|
| `dim3` | 3D physics (default) |
| `dim2` | 2D physics |
| `netcode` | Multiplayer support with bevy_renet2 |

## Examples

```bash
# Basic shooting
cargo run --example basic_shooting

# Grenades
cargo run --example grenades

# Multiplayer (requires netcode feature)
cargo run --example multiplayer --features netcode
```

## Weapon Presets

```rust
use bevy_bullet_dynamics::systems::accuracy::presets;

// Get accuracy preset
let rifle_accuracy = presets::rifle();
let sniper_accuracy = presets::sniper();
let smg_accuracy = presets::smg();
```

Available presets: `pistol()`, `rifle()`, `sniper()`, `shotgun()`, `smg()`, `bow()`

## Surface Materials

```rust
use bevy_bullet_dynamics::systems::surface::materials;

// Get material preset
let concrete = materials::concrete();
let metal = materials::metal();
let wood = materials::wood();
```

Available materials: `concrete()`, `metal()`, `wood()`, `flesh()`, `glass()`, `water()`, `dirt()`

## Grenade Presets

```rust
use bevy_bullet_dynamics::systems::logic::presets;

let (logic, payload) = presets::frag_grenade();
let (logic, payload) = presets::flashbang();
let (logic, payload) = presets::smoke_grenade();
let (logic, payload) = presets::molotov();
```

## Configuration

```rust
// Modify environment
app.insert_resource(BallisticsEnvironment {
    gravity: Vec3::new(0.0, -9.81, 0.0),
    air_density: 1.225,
    wind: Vec3::new(5.0, 0.0, 0.0), // 5 m/s wind
    ..default()
});

// Modify config
app.insert_resource(BallisticsConfig {
    use_rk4: true, // or false for Euler
    enable_penetration: true,
    enable_ricochet: true,
    debug_draw: false,
    ..default()
});
```

## Events

Subscribe to ballistic events:

```rust
fn handle_hits(mut events: EventReader<HitEvent>) {
    for hit in events.read() {
        println!("Hit at {:?} for {} damage", hit.impact_point, hit.damage);
    }
}

fn handle_explosions(mut events: EventReader<ExplosionEvent>) {
    for explosion in events.read() {
        match explosion.explosion_type {
            ExplosionType::HighExplosive => { /* ... */ }
            ExplosionType::Flash => { /* ... */ }
            _ => {}
        }
    }
}
```

## License

MIT OR Apache-2.0

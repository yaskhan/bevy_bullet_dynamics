# TODO List: Phase 2 - Advanced Ballistics & Systems

Following the successful migration to Bevy 0.18 and core feature implementation, here are the proposed features for the next stage of development.

## 🛡️ Armor & Destruction
- [ ] **Complex Armor System**: Add sloped armor calculations (effective thickness based on impact angle).
- [ ] **Multi-layer Penetration**: Logic for projectiles passing through multiple distinct layers (e.g., spaced armor).
- [ ] **Destructible Environment**: Basic integration with `bevy_xpbd` or `avian3d` for breaking static meshes on high-impact hits.
- [ ] **Deformation**: Simple mesh deformation or "bullet hole" decal scaling based on caliber.

## 🔫 Weapon Systems & Attachments
- [ ] **Sight System 2.0**: Visual scope overlay with working mildots and adjustable zeroing that matches the ballistics.
- [ ] **Muzzle Attachments**: Silencers (reduce flash/sound) and Compensators (reduce recoil/spread).
- [ ] **Recoil Patterns**: Scriptable recoil patterns (procedural or pre-defined paths).
- [ ] **Ammo Types**: Subsonic vs. Supersonic ammo (affecting sonic boom and drag).

## 🔊 Audio & Feedback
- [ ] **Sonic Boom**: Play a "crack" sound when a supersonic projectile passes near the camera.
- [ ] **Distance-based Delay**: Delay hit sounds based on the observer's distance to the impact point.
- [ ] **Material-specific Audio**: Different sounds for concrete, metal, wood, and flesh hits.

## 🚀 Performance & Optimization
- [ ] **Spatial Partitioning**: Optimize collision checks for thousands of projectiles using a custom grid or octree.
- [ ] **SIMD Integration**: Use SIMD for batch RK4 calculations if possible.
- [ ] **VFX Batching**: Optimize tracer rendering for high-fire-rate weapons (Miniguns).

## 🛠️ Tooling & Editor
- [ ] **Ballistics Lab Example**: A dedicated example for testing drag curves and penetration values with real-time graphs.
- [ ] **Bevy Inspector Integration**: Custom UI for `BallisticsConfig` and `Projectile` components.
- [ ] **Trajectory Pre-visualization**: Draw the predicted flight path in the editor/gizmos before firing.

## 🎮 Gameplay Integration
- [ ] **3D Character Controller**: A full "Soldier" example with movement, ADS, and ballistics integration.
- [ ] **Vehicles Integration**: Tank turret ballistics with heavy shell physics (HE/AP rounds).
- [ ] **AI Suppression**: Logic for AI entities to react when projectiles pass close to them.

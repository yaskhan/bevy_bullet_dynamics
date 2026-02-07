Bevy engine version 0.18 

Ниже представлено расширенное техническое задание и готовый каркас проекта (Scaffolding).

---

## Техническое задание (Plugin Specification)

### 1. Название и структура

* **Имя плагина:** `bevy_bullet_dynamics`
* **Тип:** Группа плагинов (`BallisticsPluginGroup`), состоящая из:
* `BallisticsCorePlugin`: Физические расчеты (RK4).
* `BallisticsSurfacePlugin`: Взаимодействие с материалами.
* `BallisticsVfxPlugin`: Управление пулом трассировщиков и декалей.



### 2. Ключевые требования

* **Детерминизм:** Использование фиксированного временного шага (`FixedUpdate`) для расчета баллистики, чтобы траектория не зависела от FPS.
* **Производительность:** Использование `Query::par_iter()` для тяжелых вычислений и `Entity Commands` для ленивого спавна эффектов.
* **Гибкость:** Поддержка различных типов интеграторов (Euler для простых проектов, RK4 для симуляторов).

---

## Архитектурный каркас (Project Scaffold)

### Структура файлов

```text
bevy_bullet_dynamics/
├── src/
│   ├── lib.rs              # Объявление плагина
│   ├── components.rs       # Projectile, Accuracy, SurfaceMaterial
│   ├── resources.rs        # BallisticsEnvironment, GlobalConfig
│   ├── systems/
│   │   ├── kinematics.rs   # Интеграция RK4 и Эйлера
│   │   ├── collision.rs    # Raycast и пробитие
│   │   └── accuracy.rs     # Расчет Bloom и Spread
│   └── vfx.rs              # Pooling и трассировщики
└── Cargo.toml

```

### 1. Основные компоненты (`src/components.rs`)

```rust
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub mass: f32,              // кг
    pub drag_coefficient: f32,  // Cd
    pub reference_area: f32,    // м^2
    pub penetration_power: f32, // Условные единицы энергии
}

#[derive(Component, Reflect)]
pub struct Accuracy {
    pub current_bloom: f32,
    pub base_spread: f32,
    pub max_spread: f32,
    pub bloom_per_shot: f32,
    pub recovery_rate: f32,
    pub movement_penalty: f32,
    pub ads_modifier: f32, 
}

impl Default for Accuracy {
    fn default() -> Self {
        Self {
            current_bloom: 0.0,
            base_spread: 0.002, // ~0.1 градуса
            max_spread: 0.05,
            bloom_per_shot: 0.01,
            recovery_rate: 0.05,
            movement_penalty: 2.0,
            ads_modifier: 0.3,
        }
    }
}

```

### 2. Математическое ядро (RK4) (`src/systems/kinematics.rs`)

```rust
use bevy::prelude::*;
use crate::components::Projectile;
use crate::resources::BallisticsEnvironment;

pub fn update_projectiles_kinematics(
    time: Res<Time<Fixed>>,
    env: Res<BallisticsEnvironment>,
    mut query: Query<(&mut Transform, &mut Projectile)>,
) {
    let dt = time.delta_secs();

    query.par_iter_mut().for_each(|(mut transform, mut bullet)| {
        let pos = transform.translation;
        let vel = bullet.velocity;

        // RK4 Integration Step
        let k1 = calculate_acceleration(&bullet, vel, &env);
        let k2 = calculate_acceleration(&bullet, vel + k1 * (dt / 2.0), &env);
        let k3 = calculate_acceleration(&bullet, vel + k2 * (dt / 2.0), &env);
        let k4 = calculate_acceleration(&bullet, vel + k3 * dt, &env);

        let final_accel = (k1 + k2 * 2.0 + k3 * 2.0 + k4) / 6.0;
        
        bullet.velocity += final_accel * dt;
        transform.translation += bullet.velocity * dt;
    });
}

fn calculate_acceleration(bullet: &Projectile, vel: Vec3, env: &BallisticsEnvironment) -> Vec3 {
    let relative_vel = vel - env.wind;
    let speed = relative_vel.length();
    if speed < 0.001 { return env.gravity; }

    let dir = relative_vel.normalize();
    let drag_force = 0.5 * env.air_density * speed.powi(2) * bullet.drag_coefficient * bullet.reference_area;
    
    env.gravity - (dir * drag_force / bullet.mass)
}

```

### 3. Главный плагин (`src/lib.rs`)

```rust
pub struct BallisticsPlugin;

impl Plugin for BallisticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Projectile>()
           .register_type::<Accuracy>()
           .init_resource::<BallisticsEnvironment>()
           // Запуск физики в фиксированном таймстепе
           .add_systems(FixedUpdate, (
               systems::accuracy::update_bloom,
               systems::kinematics::update_projectiles_kinematics,
               systems::collision::handle_collisions,
           ).chain());
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct BallisticsEnvironment {
    pub gravity: Vec3,
    pub air_density: f32,
    pub wind: Vec3,
}

impl Default for BallisticsEnvironment {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            air_density: 1.225, // Стандарт на уровне моря
            wind: Vec3::ZERO,
        }
    }
}

```

---

## Что нужно доработать (Next Steps)

1. **Spatial Query (Collisions):** В `handle_collisions` используйте `Gizmos` для отладки лучей. Нужно использовать `bevy_rapier3d` для проверки пересечений между `previous_pos` и `current_pos`.
2. **Object Pooling:** Для Bevy рекомендую создать `Resource`, хранящий `Vec<Entity>`, которые помечены как "спящие" (удалены из рендеринга, но не деспавнены), чтобы не нагружать аллокатор при стрельбе из пулемета.
3. **Гауссово распределение:** Для реализации пункта 3.2 ТЗ (Разброс) используйте библиотеку `rand_distr`, чтобы получить `StandardNormal` распределение для углов отклонения.

-------------------------------
хорошо, но я забыл упомянуть, что плагин должен поддерживать разные оружия от пистолета до снайпера. Лук, арбалет, гранаты, зажигательные гранаты , ослепительные. Так же должен поддерживать 2д и 3д. И взаимодеиствие клиент сервер

Для реализации такой универсальной системы, поддерживающей всё от пистолетов до гранат в 2D и 3D с поддержкой сети, нам нужно перейти к **архитектуре на основе данных (Data-Driven)**.

Ниже представлено расширенное техническое задание и обновленная структура проекта.

---

## 1. Расширенное Техническое Задание

### 1.1. Обобщенная модель снаряда

Вместо жесткой логики "пули", мы вводим типы поведения:

* **Ballistic**: Пули, стрелы, болты (RK4, гравитация, сопротивление воздуха).
* **Parabolic**: Гранаты (высокая дуга, малая скорость, влияние отскоков).
* **Area of Effect (AoE)**: Зажигательные (зона), ослепляющие (радиус видимости).

### 1.2. Мета-баллистика (2D + 3D)

Использование типа `glam::Vec3` для всех расчетов.

* В **2D**: компонента `Z` игнорируется или используется для слоев (Z-order).
* Система коллизий переключается через Feature Flags (`dim2` / `dim3`).

### 1.3. Сетевое взаимодействие (Client-Server)

* **Server-Authoritative**: Сервер считает физику и траекторию.
* **Client-Side Prediction**: Клиент локально спавнит визуальный трассировщик сразу после выстрела для отзывчивости.
* **Snapshot Compression**: Передача только критических изменений (выстрел, попадание, взрыв) вместо синхронизации позиции пули каждый кадр.

---

## 2. Каркас проекта: Универсальный плагин

### Структура компонентов

Для поддержки разных типов оружия мы разделим "физику" и "логику срабатывания".

```rust
use bevy::prelude::*;

// Тип поведения снаряда
#[derive(Component, Reflect)]
pub enum ProjectileLogic {
    Impact,             // Пуля: исчезает или пробивает при контакте
    Timed { fuse: f32 }, // Граната: взрыв по таймеру
    Proximity { range: f32 }, // Мина/Ракета: взрыв при приближении
}

// Тип эффекта при срабатывании
#[derive(Component, Reflect)]
pub enum Payload {
    Kinetic { damage: f32 },
    Explosive { radius: f32, falloff: f32 },
    Incendiary { duration: f32, dps: f32 },
    Flash { intensity: f32 },
}

#[derive(Component, Reflect)]
pub struct NetEntity {
    pub owner_id: u64,
    pub timestamp: f64,
}

```

### 3. Обработка 2D и 3D

Чтобы не писать код дважды, используем трейт для абстракции пространственных запросов.

```rust
pub trait SpatialQuery {
    fn cast_projectile_ray(&self, origin: Vec3, direction: Vec3, max_dist: f32) -> Option<HitResult>;
}

// Реализация для 3D (используя Bevy или Rapier3d)
pub struct SpatialQuery3d<'w, 's> {
    pub raycast: Raycast<'w, 's>,
}

// Реализация для 2D (используя Rapier2d)
pub struct SpatialQuery2d<'w, 's> {
    pub query: &'w RapierContext, // если используете Rapier
}

```

---

## 4. Система "Клиент-Сервер" (Архитектура событий)

1. **Событие `FireEvent**`: Содержит `seed` для рандома разброса, тип оружия и `timestamp`.
2. **Клиент**:
* Считает разброс по `seed`.
* Спавнит визуальный эффект (`BulletTracer`).
* Отправляет запрос на сервер.


3. **Сервер**:
* Валидирует время выстрела (анти-чит).
* Проводит симуляцию RK4.
* При попадании рассылает всем `HitEvent`.



```rust
// Сетевой плагин
pub struct BallisticsNetPlugin;

impl Plugin for BallisticsNetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireCommand>()
           .add_event::<HitEvent>()
           .add_systems(FixedUpdate, (
               process_fire_commands.run_if(on_event::<FireCommand>()),
               reconcile_hits,
           ));
    }
}

```

---

## 5. Спецификация для типов оружия (Data-Driven)

| Тип | Модель физики | Снаряд | Эффект (Payload) |
| --- | --- | --- | --- |
| **Снайперка** | RK4 (High Velocity) | Bullet | Kinetic (High) |
| **Лук** | RK4 (High Drag + Gravity) | Arrow | Kinetic (Medium) |
| **Грената** | Euler (Low Velocity) | Parabolic | Explosive |
| **Молотов** | Euler | Parabolic | Incendiary (Zone) |
| **Flashbang** | Euler | Parabolic | Flash (Screen Overlay) |

### Что делать дальше:

1. **Выбор физического движка**: Будете ли вы использовать `avian3d` ?
2. **Сетевой стек**: Какой протокол планируете (`bevy_renet2`)?

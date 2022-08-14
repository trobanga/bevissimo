use benimator::FrameRate;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    orb::Orb,
    player::Player,
    ship::energy::EnergyBundle,
    weapon::{self, Damage, FireRate, ProjectileSpeed, Weapon, WeaponBundle},
    Animation, AnimationState,
};

pub mod energy;
pub use energy::Energy;

use self::energy::{EnergyDecay, EnergyTimer};

pub struct ShipConfig<'a> {
    pub acceleration: f32,
    pub ship_sprite: &'a str,
    pub exhaust_sprite_sheet: &'a str,
    pub max_energy: f32,
    pub energy_decay: f32,
    pub energy_start_value: f32,
}

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Exhaust;

#[derive(Component)]
pub struct Acceleration(pub f32);

#[derive(Component)]
pub struct Accelerate;

#[derive(Component)]
pub struct FireWeapon;

pub fn spawn_ship(
    ship_config: ShipConfig,
    commands: &mut Commands,
    asset_server: &AssetServer,
    textures: &mut Assets<TextureAtlas>,
) -> Entity {
    commands
        .spawn()
        .insert(Ship)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert_bundle(SpriteBundle {
            texture: asset_server.load(ship_config.ship_sprite),
            ..default()
        })
        .insert(Acceleration(ship_config.acceleration))
        .insert_bundle(EnergyBundle {
            energy: Energy {
                max: ship_config.max_energy,
                current: ship_config.energy_start_value,
            },
            decay: EnergyDecay(ship_config.energy_decay),
            timer: EnergyTimer::default(),
        })
        .insert_bundle(Kinematic::new())
        .with_children(|p| {
            p.spawn_bundle(ExhaustAnimationBundle::new(
                ship_config.exhaust_sprite_sheet,
                asset_server,
                textures,
            ))
            .insert(Transform::from_translation(Vec3 {
                x: 0.0,
                y: -85.0,
                z: 0.0,
            }))
            .insert(Exhaust);

            p.spawn_bundle(WeaponBundle {
                fire_rate: FireRate::new(5.0),
                damage: Damage(1.3),
                speed: ProjectileSpeed(250.0),
                ..Default::default()
            });
        })
        .id()
}

#[derive(Bundle)]
pub struct ExhaustAnimationBundle {
    #[bundle]
    exhaust_sprite: SpriteSheetBundle,
    animation: Animation,
    animation_state: AnimationState,
}

impl ExhaustAnimationBundle {
    pub fn new(
        exhaust_sprite: &str,
        asset_server: &AssetServer,
        textures: &mut Assets<TextureAtlas>,
    ) -> Self {
        let animation = Animation(benimator::Animation::from_indices(
            0..=2,
            FrameRate::from_fps(12.0),
        ));

        Self {
            exhaust_sprite: SpriteSheetBundle {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    asset_server.load(exhaust_sprite),
                    Vec2::new(11.0, 23.0),
                    3,
                    1,
                )),
                visibility: Visibility { is_visible: false },
                ..Default::default()
            },
            animation,
            animation_state: AnimationState::default(),
        }
    }
}

#[derive(Bundle)]
pub struct Kinematic {
    rigid_body: RigidBody,
    ext_force: ExternalForce,
    velocity: Velocity,
    collider: Collider,
}

impl Kinematic {
    pub fn new() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            ext_force: ExternalForce {
                force: Vec2::new(0.0, 0.0),
                torque: 0.0,
            },
            velocity: Velocity::default(),
            collider: Collider::ball(50.0),
        }
    }
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(show_exhaust)
            .add_system(collision_event)
            .add_system(remove_exhaust)
            .add_system(fire_weapon)
            .add_system(stop_fire_weapon);
    }
}

fn show_exhaust(
    q: Query<(&Accelerate, &Children), Added<Accelerate>>,
    mut exhaust: Query<&mut Visibility, With<Exhaust>>,
) {
    for (_, children) in q.iter() {
        for &v in children.iter() {
            if let Ok(mut v) = exhaust.get_mut(v) {
                v.is_visible = true;
            }
        }
    }
}

fn remove_exhaust(
    q: RemovedComponents<Accelerate>,
    ship: Query<(&Ship, &Children)>,
    mut exhaust: Query<&mut Visibility, With<Exhaust>>,
) {
    for e in q.iter() {
        let (_, children) = ship.get(e).unwrap();
        for &v in children.iter() {
            if let Ok(mut v) = exhaust.get_mut(v) {
                v.is_visible = false;
            }
        }
    }
}

fn collision_event(
    mut collisions: EventReader<CollisionEvent>,
    mut ship_energy: Query<(&Ship, &mut Energy), With<Player>>,
    orbs: Query<&Orb>,
) {
    for collision in collisions.iter() {
        if let CollisionEvent::Started(e0, e1, _) = collision {
            if let Ok((_, mut energy)) = ship_energy.get_mut(*e0) {
                if let Ok(_) = orbs.get(*e1) {
                    (*energy).increase(10.0);
                }
            } else if let Ok((_, mut energy)) = ship_energy.get_mut(*e1) {
                if let Ok(_) = orbs.get(*e0) {
                    (*energy).increase(10.0);
                }
            }
        }
    }
}

fn fire_weapon(
    mut commands: Commands,
    ship: Query<(&Ship, &FireWeapon, &Children), Added<FireWeapon>>,
    weapon: Query<&Weapon>,
) {
    for (_, _, children) in ship.iter() {
        for &c in children {
            if let Ok(_) = weapon.get(c) {
                commands.entity(c).insert(weapon::FireWeapon);
            }
        }
    }
}

fn stop_fire_weapon(
    mut commands: Commands,
    q: RemovedComponents<FireWeapon>,
    ship: Query<(&Ship, &Children)>,
    weapon: Query<&Weapon>,
) {
    for e in q.iter() {
        let (_, children) = ship.get(e).unwrap();
        for &c in children {
            if let Ok(_) = weapon.get(c) {
                commands.entity(c).remove::<weapon::FireWeapon>();
            }
        }
    }
}

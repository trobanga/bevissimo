use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use getset::MutGetters;

use crate::{components::Hp, utils::direction};

pub struct FireWeaponEvent(pub Entity);

#[derive(Component, Default, Debug)]
pub struct Weapon;

#[derive(Component, Debug)]
pub struct FireWeapon;

#[derive(Component, MutGetters, Debug)]
#[getset(get_mut = "pub")]
pub struct FireRate {
    rate: f32,
    timer: Timer,
}

impl FireRate {
    pub fn new(rate: f32) -> Self {
        let timer = Timer::new(Duration::from_millis((1000.0 / rate) as u64), true);
        Self { rate, timer }
    }
}

impl Default for FireRate {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[derive(Component, Default)]
pub struct Damage(pub f32);

#[derive(Component, Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Default)]
pub struct ProjectileLifetime(pub f32);

#[derive(Bundle)]
pub struct WeaponBundle {
    pub weapon: Weapon,
    pub fire_rate: FireRate,
    pub damage: Damage,
    pub speed: ProjectileSpeed,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for WeaponBundle {
    fn default() -> Self {
        let fire_rate = FireRate::default();
        Self {
            weapon: Weapon,
            fire_rate,
            damage: Damage::default(),
            speed: ProjectileSpeed::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireWeaponEvent>()
            .add_system(fire_weapons)
            .add_system(collide)
            .add_system(projectile_life_time);
    }
}

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct ProjectileDamage(pub f32);

#[derive(Component)]
struct ProjectileLifeTimer(Timer);

fn fire_weapons(
    mut commands: Commands,
    mut weapons: Query<(
        &Parent,
        &Weapon,
        &Damage,
        &ProjectileSpeed,
        &GlobalTransform,
        Option<&FireWeapon>,
        &mut FireRate,
    )>,
    velocites: Query<&Velocity>,
    time: Res<Time>,
) {
    let delta = time.delta();
    for (parent, _, damage, speed, global_transform, fire, mut fire_rate) in weapons.iter_mut() {
        let timer = fire_rate.timer_mut();
        timer.tick(delta);
        if timer.just_finished() && fire.is_some() {
            info!("pew pew pew");
            let mut transform = global_transform.compute_transform();
            let (x, y) = direction(&transform);
            let v = velocites.get(**parent).unwrap();

            info!("{} {}", x, y);
            info!("{:?}", v);

            // TODO: hack, set weapon output position
            transform.translation += Transform::from_xyz(x, y, 0.0).translation * 100.0;

            commands
                .spawn_bundle(SpriteBundle {
                    transform,
                    ..Default::default()
                })
                .insert(Projectile)
                .insert(ProjectileDamage(damage.0))
                .insert(ProjectileLifeTimer(Timer::new(
                    Duration::from_millis(1500),
                    false,
                )))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(RigidBody::Dynamic)
                .insert(Velocity {
                    linvel: v.linvel + Vec2::new(x, y) * speed.0,
                    angvel: 0.0,
                })
                .insert(Collider::ball(1.0));
        }
    }
}

fn projectile_life_time(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut ProjectileLifeTimer)>,
    time: Res<Time>,
) {
    for (e, mut p) in projectiles.iter_mut() {
        p.0.tick(time.delta());
        if p.0.just_finished() {
            commands.entity(e).despawn();
        }
    }
}

fn collide(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    projectiles: Query<(&Projectile, &ProjectileDamage)>,
    mut hp: Query<&mut Hp>,
) {
    for collision in collisions.iter() {
        info!("{:?}", collision);
        if let CollisionEvent::Started(e0, e1, _) = collision {
            if let Ok(mut hp) = hp.get_mut(*e0) {
                if let Ok(p) = projectiles.get(*e1) {
                    (*hp).0 -= p.1 .0;
                    commands.entity(*e1).despawn();
                }
            } else if let Ok(mut hp) = hp.get_mut(*e1) {
                if let Ok(p) = projectiles.get(*e0) {
                    (*hp).0 -= p.1 .0;
                    commands.entity(*e0).despawn();
                }
            }
        }
    }
}

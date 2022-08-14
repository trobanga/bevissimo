use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{components::Hp, ship::Ship, GameState};

struct OrbConfig {
    max_orbs: usize,
}

#[derive(Default)]
struct OrbHandles {
    handles: Vec<Handle<Image>>,
}

#[derive(Default, Debug, Component)]
pub struct Orb;

#[derive(Component)]
pub struct OrbTimer(pub Timer);

pub struct OrbPlugin;

impl Plugin for OrbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrbHandles>()
            .insert_resource(OrbConfig { max_orbs: 5 })
            .add_startup_system(setup)
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(tick)
                    .with_system(collision)
                    .with_system(hp),
            );
    }
}

fn setup(mut commands: Commands, mut handles: ResMut<OrbHandles>, asset_server: Res<AssetServer>) {
    commands
        .spawn()
        .insert(OrbTimer(Timer::new(Duration::from_secs(1), true)));

    handles.handles.push(asset_server.load("orbs/Airless.png"));
}

fn tick(
    mut commands: Commands,
    windows: Res<Windows>,
    handles: Res<OrbHandles>,
    mut timer: Query<&mut OrbTimer>,
    orb_config: Res<OrbConfig>,
    orbs: Query<&Orb>,
    time: Res<Time>,
) {
    let mut timer = timer.single_mut();
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        if orbs.iter().len() < orb_config.max_orbs {
            spawn_orb(&mut commands, &windows, &handles);
        }
        let mut rng = rand::thread_rng();
        timer
            .0
            .set_duration(Duration::from_secs(rng.gen_range(1..=10)));
    }
}

fn collision(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    orbs: Query<&Orb>,
    ships: Query<&Ship>,
) {
    for collision in collisions.iter() {
        if let CollisionEvent::Started(e0, e1, _) = collision {
            if orbs.contains(*e1) && ships.contains(*e0) {
                commands.entity(*e1).despawn();
            } else if orbs.contains(*e0) && ships.contains(*e1) {
                commands.entity(*e0).despawn();
            }
        }
    }
}

fn spawn_orb(commands: &mut Commands, windows: &Res<Windows>, handles: &Res<OrbHandles>) {
    let w = windows.primary();
    let width = w.width();
    let height = w.height();
    let mut rng = rand::thread_rng();
    commands
        .spawn()
        .insert(Orb)
        .insert(Hp(10.0))
        .insert_bundle(SpriteBundle {
            transform: Transform::from_xyz(
                rng.gen_range(-0.5..0.5) * width,
                rng.gen_range(-0.5..0.5) * height,
                0.0,
            ),
            texture: handles.handles[0].clone(),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(32.0))
        .insert(Sensor);
}

fn hp(mut commands: Commands, hp: Query<(Entity, &Hp), With<Orb>>) {
    for (e, hp) in hp.iter() {
        if hp.0 < 0.0 {
            commands.entity(e).despawn();
        }
    }
}

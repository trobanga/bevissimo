use benimator::FrameRate;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{player::Player, ship::energy::EnergyBundle, Animation, AnimationState};

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
pub struct Accelerating(pub bool);

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
        .insert(Accelerating(false))
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
            .insert(Exhaust);
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
                transform: Transform::from_translation(Vec3 {
                    x: 0.0,
                    y: -85.0,
                    z: 0.0,
                }),
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
        app.add_system(show_exhaust).add_system(collision_event);
    }
}

fn show_exhaust(
    q: Query<(&Accelerating, (&Ship, &Children)), Changed<Accelerating>>,
    mut exhaust: Query<&mut Visibility, With<Exhaust>>,
) {
    for (f, (_, children)) in q.iter() {
        for &v in children.iter() {
            let mut v = exhaust.get_mut(v).unwrap();
            v.is_visible = f.0;
        }
    }
}

fn collision_event(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    mut ship_energy: Query<&mut Energy, With<Player>>,
) {
    for e in collisions.iter() {
        info!("{:?}", e);
        match e {
            CollisionEvent::Started(_, e1, _) => {
                commands.entity(*e1).despawn();
                (*ship_energy.single_mut()).increase(10.0);
            }
            _ => {}
        }
    }
}

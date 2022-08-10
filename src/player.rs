use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    ship::{spawn_ship, Accelerating, Acceleration, Energy, ShipConfig},
    Scoreboard,
};
use bevy_rapier2d::prelude::*;

#[derive(Component, Reflect)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<TextureAtlas>>,
) {
    let ship_config = ShipConfig {
        acceleration: 140.0,
        ship_sprite: "ships/1.png",
        exhaust_sprite_sheet: "ships/exhaust/exhaust1.png",
        max_energy: 100.0,
        energy_decay: 0.8,
        energy_start_value: 5.0,
    };

    let ship = spawn_ship(ship_config, &mut commands, &asset_server, &mut textures);
    commands
        .entity(ship)
        .insert(Player)
        .insert(ActiveEvents::COLLISION_EVENTS);
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // app.register_type::<Player>();
        app.add_startup_system(spawn_player)
            .add_system(move_player)
            .add_system(collision_event);
    }
}

fn collision_event(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    mut scoreboard: ResMut<Scoreboard>,
    mut ship_energy: Query<&mut Energy, With<Player>>,
) {
    for e in collisions.iter() {
        match e {
            CollisionEvent::Started(_, e1, _) => {
                commands.entity(*e1).despawn();
                scoreboard.score += 1;
                (*ship_energy.single_mut()).increase(10.0);
            }
            _ => {}
        }
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    transforms: Query<&Transform, With<Player>>,
    mut ext_forces: Query<&mut ExternalForce, With<Player>>,
    mut velocities: Query<&mut Velocity, With<Player>>,
    acceleration: Query<&Acceleration, With<Player>>,
    mut accelerating: Query<&mut Accelerating, With<Player>>,
) {
    let mut force = ext_forces.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        force.torque = 1.0;
    } else if keyboard_input.pressed(KeyCode::Right) {
        force.torque = -1.0;
    } else if keyboard_input.just_released(KeyCode::Left)
        || keyboard_input.just_released(KeyCode::Right)
    {
        force.torque = 0.0;
        velocities.single_mut().angvel = 0.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        let (dx, dy) = direction(&transforms.single());
        let a = acceleration.single().0;
        let (dx, dy) = (a * dx, a * dy);
        force.force = Vec2 { x: -dx, y: dy };
        (*accelerating.single_mut()).0 = true;
    } else if keyboard_input.just_released(KeyCode::Up)
        || keyboard_input.just_released(KeyCode::Down)
    {
        force.force = Vec2::ZERO;
        (*accelerating.single_mut()).0 = false;
    }
}

fn direction(transform: &Transform) -> (f32, f32) {
    let rot = transform.rotation;
    let rot = if rot.z >= 0.0 {
        rot.angle_between(Quat::IDENTITY)
    } else {
        2.0 * PI - rot.angle_between(Quat::IDENTITY)
    };
    rot.sin_cos()
}

use bevy::prelude::*;

use crate::{
    ship::{spawn_ship, Accelerate, Acceleration, FireWeapon, Ship, ShipConfig},
    utils,
};
use bevy_rapier2d::prelude::*;

#[derive(Component)]
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
    commands.entity(ship).insert(Player);
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system_to_stage(CoreStage::PreUpdate, user_input);
    }
}

fn user_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    transforms: Query<&Transform, (With<Player>, With<Ship>)>,
    mut ext_forces: Query<&mut ExternalForce, With<Player>>,
    mut velocities: Query<&mut Velocity, With<Player>>,
    acceleration: Query<&Acceleration, With<Player>>,
    ship: Query<(Entity, &Ship), With<Player>>,
) {
    let mut force = ext_forces.single_mut();

    if keyboard_input.pressed(KeyCode::A) {
        force.torque = 1.0;
    } else if keyboard_input.pressed(KeyCode::D) {
        force.torque = -1.0;
    } else if keyboard_input.just_released(KeyCode::A) || keyboard_input.just_released(KeyCode::D) {
        force.torque = 0.0;
        velocities.single_mut().angvel = 0.0;
    }

    if keyboard_input.pressed(KeyCode::W) {
        let (dx, dy) = utils::direction(&transforms.single());
        let a = acceleration.single().0;
        let (dx, dy) = (a * dx, a * dy);
        force.force = Vec2 { x: dx, y: dy };
        commands.entity(ship.single().0).insert(Accelerate);
    } else if keyboard_input.just_released(KeyCode::W) {
        force.force = Vec2::ZERO;
        commands.entity(ship.single().0).remove::<Accelerate>();
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.entity(ship.single().0).insert(FireWeapon);
    } else if keyboard_input.just_released(KeyCode::Space) {
        commands.entity(ship.single().0).remove::<FireWeapon>();
    }
}

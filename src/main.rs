use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use bevy_rapier2d::plugin::RapierConfiguration;
use bevy_rapier2d::prelude::*;

mod components;
mod hud;
mod orb;
mod player;
mod ship;
mod utils;
mod weapon;

#[derive(Component, Deref)]
struct Animation(benimator::Animation);

#[derive(Default, Component, Deref, DerefMut)]
struct AnimationState(benimator::State);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Setup,
    Playing,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevissimo!".to_string(),
            present_mode: PresentMode::AutoVsync,
            // mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        .add_state(GameState::Setup)
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(hud::HudPlugin)
        .add_plugin(ship::ShipPlugin)
        .add_plugin(ship::energy::EnergyPlugin)
        .add_plugin(orb::OrbPlugin)
        .add_plugin(weapon::WeaponPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(animate)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup(
    mut state: ResMut<State<GameState>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
) {
    rapier_configuration.gravity = Vec2::ZERO;

    let _ = state.overwrite_set(GameState::Playing);
}

fn animate(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut TextureAtlasSprite, &Animation)>,
) {
    for (mut player, mut texture, animation) in query.iter_mut() {
        // Update the state
        player.update(animation, time.delta());

        // Update the texture atlas
        texture.index = player.sprite_frame_index();
    }
}

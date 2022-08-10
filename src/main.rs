use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use bevy_rapier2d::plugin::RapierConfiguration;
use bevy_rapier2d::prelude::*;
use rand::Rng;

mod hud;
mod player;
mod ship;

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
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        .insert_resource(Scoreboard { score: 0 })
        .add_state(GameState::Setup)
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(hud::HudPlugin)
        .add_plugin(ship::ShipPlugin)
        .add_plugin(ship::energy::EnergyPlugin)
        .add_startup_system(setup_camera)
        .add_system(bevy::window::close_on_esc)
        .add_startup_system(setup)
        // .add_system_set(SystemSet::on_enter(GameState::Setup).with_system(setup))
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_orb))
        .add_system(animate)
        .add_system(update_scoreboard)
        .run();
}

#[derive(Default, Component)]
struct Orb;

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

struct Scoreboard {
    score: usize,
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn setup(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    asset_server: Res<AssetServer>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
) {
    rapier_configuration.gravity = Vec2::ZERO;
    commands.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        }),
    );

    let _ = state.overwrite_set(GameState::Playing);
}

fn spawn_orb(mut commands: Commands, windows: Res<Windows>, asset_server: Res<AssetServer>) {
    let width = windows.primary().width();
    let height = windows.primary().height();
    let mut rng = rand::thread_rng();
    commands
        .spawn()
        .insert(Orb)
        .insert_bundle(SpriteBundle {
            transform: Transform::from_xyz(
                rng.gen_range(-0.5..0.5) * width,
                rng.gen_range(-0.5..0.5) * height,
                0.0,
            ),
            texture: asset_server.load("orbs/Airless.png"),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(32.0))
        .insert(Sensor);
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

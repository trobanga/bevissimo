use bevy::log::LogSettings;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, IoTaskPool};
use bevy::window::{PresentMode, WindowMode};
use bevy_ggrs::*;
use bevy_rapier2d::plugin::RapierConfiguration;
use bevy_rapier2d::prelude::*;
use ggrs::InputStatus;
use player::Player;
use uuid::Uuid;
use webrtc_socket::peer::{RtcConfig, RtcConfigBuilder};
use webrtc_socket::{blocking, GgrsSocket, WebRTCSocket};

mod args;
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
    let args = args::Args::get();
    let rtc_config = RtcConfigBuilder::new()
        .address(args.server_address)
        .port(args.port)
        .user(args.username)
        .password(args.password)
        .build();

    let mut webrtc_socket = blocking::BlockingWebRTCSocket::connect(rtc_config).unwrap();

    let socket = webrtc_socket.ggrs_socket();

    let mut app = App::new();

    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(input)
        .with_rollback_schedule(Schedule::default().with_stage(
            "ROLLBACK_STAGE",
            SystemStage::single_threaded().with_system(move_players),
        ))
        .register_rollback_type::<Transform>()
        .build(&mut app);

    app.insert_resource(WindowDescriptor {
        title: "Bevissimo!".to_string(),
        present_mode: PresentMode::AutoVsync,
        // mode: WindowMode::BorderlessFullscreen,
        ..default()
    })
    .insert_resource(Some(socket))
    .insert_resource(LogSettings {
        level: bevy::log::Level::DEBUG,
        ..Default::default()
    })
    .insert_resource(DefaultTaskPoolOptions::with_num_threads(4).create_default_pools())
    .add_state(GameState::Setup)
    .add_plugins(DefaultPlugins)
    .add_plugin(AnimationPlugin::default())
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
    // .add_plugin(RapierDebugRenderPlugin::default())
    // .add_plugin(player::PlayerPlugin)
    .add_plugin(hud::HudPlugin)
    .add_plugin(ship::ShipPlugin)
    .add_plugin(ship::energy::EnergyPlugin)
    .add_plugin(orb::OrbPlugin)
    .add_plugin(weapon::WeaponPlugin)
    .add_startup_system(setup_camera)
    .add_startup_system(setup)
    .add_startup_system(start_socket.after(setup))
    .add_system(bevy::window::close_on_esc)
    .add_system(animate)
    .add_system(wait_for_players.exclusive_system())
    .add_system(spawn_players)
    .run();
}

fn start_socket(mut commands: Commands) {

    // commands.insert(GenTask { task });
    // let task_pool = IoTaskPool::get();
    // task_pool.spawn(socket.run()).detach();

    // commands.insert_resource(Some(socket));
}

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_FIRE: u8 = 1 << 4;

struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = Uuid;
}

fn wait_for_players(world: &mut World) {
    let mut socket = world.get_resource_mut::<Option<GgrsSocket>>().unwrap();
    let socket = socket.as_mut();

    // If there is no socket we've already started the game
    if socket.is_none() {
        return;
    }

    // Check for new connections
    let players = socket.as_ref().unwrap().players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the socket out of the resource (required because GGRS takes ownership of it)
    let socket = socket.take().unwrap();

    // start the GGRS session
    let session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    world.insert_non_send_resource(session);
    world.insert_resource(SessionType::P2PSession);
}

fn input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {
    let mut input = 0u8;

    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        input |= INPUT_DOWN;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        input |= INPUT_LEFT
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        input |= INPUT_RIGHT;
    }
    if keys.any_pressed([KeyCode::Space, KeyCode::Return]) {
        input |= INPUT_FIRE;
    }

    input
}

fn move_players(
    inputs: Res<Vec<(u8, InputStatus)>>,
    mut player_query: Query<(&mut Transform, &Player)>,
) {
    for (mut transform, player) in player_query.iter_mut() {
        let (input, _) = inputs[player.handle];

        let mut direction = Vec2::ZERO;

        if input & INPUT_UP != 0 {
            direction.y += 1.;
        }
        if input & INPUT_DOWN != 0 {
            direction.y -= 1.;
        }
        if input & INPUT_RIGHT != 0 {
            direction.x += 1.;
        }
        if input & INPUT_LEFT != 0 {
            direction.x -= 1.;
        }
        if direction == Vec2::ZERO {
            continue;
        }

        let move_speed = 0.13;
        let move_delta = (direction * move_speed).extend(0.);

        transform.translation += move_delta;
    }
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

fn spawn_players(mut commands: Commands, mut rip: ResMut<RollbackIdProvider>) {
    // Player 1
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2., 0., 0.)),
            sprite: Sprite {
                color: Color::rgb(0., 0.47, 1.),
                ..default()
            },
            ..default()
        })
        .insert(Player { handle: 0 })
        .insert(Rollback::new(rip.next_id()));

    // Player 2
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(2., 0., 0.)),
            sprite: Sprite {
                color: Color::rgb(0., 0.4, 0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player { handle: 1 })
        .insert(Rollback::new(rip.next_id()));
}

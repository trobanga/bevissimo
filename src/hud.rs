use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::WindowResized;

use crate::player::Player;
use crate::ship::Energy;

pub struct HudPlugin;

#[derive(Default, Debug)]
struct EnergyBarPosition {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct EnergyBar;

#[derive(Component)]
struct EnergyBarBg;

const OUTER_WIDTH: f32 = 400.0;
const OUTER_HEIGHT: f32 = 30.0;
const BORDER_SIZE: f32 = 2.0;
const PADDING: f32 = 20.0;
const Z_POS: f32 = 900.0;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnergyBarPosition::default())
            .add_startup_system(setup)
            .add_system(resize_notificator)
            .add_system(update);
    }
}

fn calculate_position(w: f32, h: f32) -> (f32, f32) {
    let x = w / 2.0 - (OUTER_WIDTH + PADDING);
    let y = h / 2.0 - (OUTER_HEIGHT / 2.0 + PADDING);
    (x, y)
}

fn setup(
    mut commands: Commands,
    windows: Res<Windows>,
    mut energy_bar_position: ResMut<EnergyBarPosition>,
) {
    let w = windows.primary();

    let (x, y) = calculate_position(w.width(), w.height());
    energy_bar_position.x = x;
    energy_bar_position.y = y;

    let outer = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::from((x, y, Z_POS)),
                scale: Vec3::new(OUTER_WIDTH, OUTER_HEIGHT, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::BLACK,
                anchor: Anchor::CenterLeft,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(EnergyBarBg)
        .id();

    let inner = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::from((BORDER_SIZE / OUTER_WIDTH, 0.0, 1.0)),
                scale: Vec3::new(
                    1.0 - 2.0 * BORDER_SIZE / OUTER_WIDTH,
                    1.0 - 2.0 * BORDER_SIZE / OUTER_HEIGHT,
                    1.0,
                ),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::MIDNIGHT_BLUE,
                anchor: Anchor::CenterLeft,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    let energy = commands
        .spawn()
        .insert(EnergyBar)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::from((BORDER_SIZE / OUTER_WIDTH, 0.0, 2.0)),
                scale: Vec3::splat(0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::BLUE,
                anchor: Anchor::CenterLeft,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    commands.entity(outer).push_children(&[inner, energy]);
}

fn update(
    energy: Query<&Energy, (With<Player>, Changed<Energy>)>,
    mut energy_bar: Query<&mut Transform, With<EnergyBar>>,
) {
    for e in energy.iter() {
        let mut transform = energy_bar.single_mut();
        transform.scale = Vec3::new(
            e.current_percentage() * (1.0 - 2.0 * BORDER_SIZE / OUTER_WIDTH),
            1.0 - 2.0 * BORDER_SIZE / OUTER_HEIGHT,
            0.0,
        );
    }
}

fn resize_notificator(
    resize_event: Res<Events<WindowResized>>,
    mut positions: Query<&mut Transform, With<EnergyBarBg>>,
) {
    let mut reader = resize_event.get_reader();

    for e in reader.iter(&resize_event) {
        let (x, y) = calculate_position(e.width, e.height);
        for mut pos in positions.iter_mut() {
            pos.translation = Vec3::from((x, y, Z_POS));
        }
    }
}

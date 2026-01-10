//! Enemy style gallery.
//!
//! Run: `cargo run --example enemies`

use bevy::prelude::*;
use bevy::window::WindowResolution;

use shoot::game::{spawn_background_grid, GameConfig};
use shoot::geometry::{spawn_geometry_entity, GeometryBlueprint, GeometryRendererPlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Enemies Gallery".to_string(),
                        resolution: WindowResolution::from((1100, 700)),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::log::LogPlugin>(),
        )
        .add_plugins(GeometryRendererPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.03, 0.03, 0.06)),
            ..default()
        },
    ));

    let config = GameConfig {
        window_width: 1100.0,
        window_height: 700.0,
        ..default()
    };
    spawn_background_grid(&mut commands, &config);

    let font = asset_server.load("NotoSansCJKsc-Regular.otf");

    let gallery: [(&str, Vec3, GeometryBlueprint); 4] = [
        (
            "Diamond (legacy)",
            Vec3::new(-300.0, 120.0, 0.0),
            GeometryBlueprint::default_enemy(),
        ),
        (
            "Drone Small (Raiden)",
            Vec3::new(50.0, 120.0, 0.0),
            GeometryBlueprint::raiden_enemy_drone_small(),
        ),
        (
            "Hexagon (legacy)",
            Vec3::new(-300.0, -160.0, 0.0),
            GeometryBlueprint::hexagon_enemy(),
        ),
        (
            "Tank (Raiden)",
            Vec3::new(50.0, -160.0, 0.0),
            GeometryBlueprint::raiden_enemy_tank(),
        ),
    ];

    for (name, pos, bp) in gallery {
        spawn_geometry_entity(&mut commands, &bp, pos);
        spawn_label_ui(
            &mut commands,
            &config,
            &font,
            name,
            pos + Vec3::new(0.0, 120.0, 0.0),
        );
    }

    spawn_label_ui(
        &mut commands,
        &config,
        &font,
        "敌人样式预览（不含AI/射击）",
        Vec3::new(-config.window_width / 2.0 + 180.0, config.window_height / 2.0 - 30.0, 0.0),
    );
}

fn spawn_label_ui(
    commands: &mut Commands,
    config: &GameConfig,
    font: &Handle<Font>,
    text: &str,
    world_pos: Vec3,
) {
    let x = world_pos.x + config.window_width / 2.0;
    let y = config.window_height / 2.0 - world_pos.y;

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x - 140.0),
            top: Val::Px(y - 12.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 1.0)),
            ));
        });
}


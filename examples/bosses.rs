//! Boss style gallery (visual only).
//!
//! Run: `cargo run --example bosses`

use bevy::prelude::*;
use bevy::window::WindowResolution;

use shoot::entities::{boss_blueprint_for, BossType};
use shoot::game::{spawn_background_grid, GameConfig};
use shoot::geometry::{spawn_geometry_entity, GeometryRendererPlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bosses Gallery".to_string(),
                        resolution: WindowResolution::from((1200, 800)),
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
        window_width: 1200.0,
        window_height: 800.0,
        ..default()
    };
    spawn_background_grid(&mut commands, &config);

    let font = asset_server.load("NotoSansCJKsc-Regular.otf");

    // 2 x 5 grid
    let mut i = 0;
    for &boss_type in BossType::all() {
        let col = (i % 5) as f32;
        let row = (i / 5) as f32;

        let x = -480.0 + col * 240.0;
        let y = 220.0 - row * 360.0;
        let pos = Vec3::new(x, y, 0.0);

        let bp = boss_blueprint_for(boss_type);
        spawn_geometry_entity(&mut commands, &bp, pos);
        spawn_label_ui(
            &mut commands,
            &config,
            &font,
            boss_type.name(),
            pos + Vec3::new(0.0, 130.0, 0.0),
        );

        i += 1;
    }

    spawn_label_ui(
        &mut commands,
        &config,
        &font,
        "Boss 样式预览（仅外观，不含AI/射击）",
        Vec3::new(-config.window_width / 2.0 + 220.0, config.window_height / 2.0 - 30.0, 0.0),
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
            left: Val::Px(x - 90.0),
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


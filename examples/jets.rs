//! Ship style gallery (stacked geometry jets).
//!
//! Run: `cargo run --example jets`

use bevy::prelude::*;
use bevy::window::WindowResolution;

use shoot::game::{spawn_background_grid, GameConfig};
use shoot::geometry::{
    regular_polygon_vertices, spawn_geometry_entity, CollisionShape, GeometryBlueprint,
    GeometryShape, ShapeColor, Vec2D,
};
use shoot::geometry::GeometryRendererPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Jets Gallery".to_string(),
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
    // 2D camera (this project uses components rather than bundles)
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

    // 2x4 gallery
    let gallery = [
        ("Raiden Mk-I", Vec3::new(-410.0, 190.0, 0.0), jet_raiden_mk1_blueprint()),
        ("Raiden Mk-II", Vec3::new(-140.0, 190.0, 0.0), jet_raiden_mk2_blueprint()),
        ("Raiden Stealth", Vec3::new(140.0, 190.0, 0.0), jet_raiden_stealth_blueprint()),
        ("Raptor", Vec3::new(410.0, 190.0, 0.0), jet_raptor_blueprint()),
        ("Viper", Vec3::new(-410.0, -170.0, 0.0), jet_viper_blueprint()),
        ("Arrowhead", Vec3::new(-140.0, -170.0, 0.0), jet_arrowhead_blueprint()),
        ("TwinBlade", Vec3::new(140.0, -170.0, 0.0), jet_twin_blade_blueprint()),
        ("HeavyBomber", Vec3::new(410.0, -170.0, 0.0), jet_heavy_bomber_blueprint()),
    ];

    for (name, position, blueprint) in gallery {
        spawn_geometry_entity(&mut commands, &blueprint, position);
        spawn_label_ui(&mut commands, &config, &font, name, position + Vec3::new(0.0, 120.0, 0.0));
    }

    // Top-left hint
    spawn_label_ui(
        &mut commands,
        &config,
        &font,
        "战机样式预览：雷电（Raiden）街机风格优先",
        Vec3::new(-config.window_width / 2.0 + 150.0, config.window_height / 2.0 - 30.0, 0.0),
    );
}

fn spawn_label_ui(
    commands: &mut Commands,
    config: &GameConfig,
    font: &Handle<Font>,
    text: &str,
    world_pos: Vec3,
) {
    // Convert a world coordinate (origin at center) to UI absolute (origin at top-left).
    let x = world_pos.x + config.window_width / 2.0;
    let y = config.window_height / 2.0 - world_pos.y;

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x - 60.0),
            top: Val::Px(y - 12.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 1.0)),
            ));
        });
}

fn jet_raiden_mk1_blueprint() -> GeometryBlueprint {
    // Classic Raiden-ish: sharp nose, wide delta wings, twin engine pods, center fuselage.
    let hull = ShapeColor::new(0.12, 0.55, 0.95, 1.0);
    let hull2 = ShapeColor::new(0.08, 0.32, 0.65, 1.0);
    let metal = ShapeColor::new(0.78, 0.82, 0.9, 1.0);
    let dark = ShapeColor::new(0.10, 0.12, 0.16, 1.0);
    let glow = ShapeColor::new(0.12, 0.95, 0.85, 0.9);

    GeometryBlueprint {
        name: "jet_raiden_mk1".to_string(),
        shapes: vec![
            // center fuselage (long)
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 145.0),
                    Vec2D::new(-18.0, 110.0),
                    Vec2D::new(-28.0, 40.0),
                    Vec2D::new(-30.0, -40.0),
                    Vec2D::new(-18.0, -145.0),
                    Vec2D::new(18.0, -145.0),
                    Vec2D::new(30.0, -40.0),
                    Vec2D::new(28.0, 40.0),
                    Vec2D::new(18.0, 110.0),
                ],
                color: hull,
                fill: true,
                stroke_width: 2.0,
            },
            // nose cone (highlight)
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 170.0),
                    Vec2D::new(-14.0, 125.0),
                    Vec2D::new(14.0, 125.0),
                ],
                color: metal,
                fill: true,
                stroke_width: 2.0,
            },
            // cockpit glass
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 105.0),
                    Vec2D::new(-12.0, 80.0),
                    Vec2D::new(0.0, 58.0),
                    Vec2D::new(12.0, 80.0),
                ],
                color: ShapeColor::new(0.86, 0.95, 1.0, 0.9),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-3.0, 92.0),
                    Vec2D::new(3.0, 92.0),
                    Vec2D::new(6.0, 78.0),
                    Vec2D::new(-6.0, 78.0),
                ],
                color: ShapeColor::new(1.0, 1.0, 1.0, 0.25),
                fill: true,
                stroke_width: 1.0,
            },
            // main delta wings
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-185.0, 45.0),
                    Vec2D::new(-55.0, 25.0),
                    Vec2D::new(-35.0, -55.0),
                    Vec2D::new(-145.0, -90.0),
                ],
                color: hull2,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(185.0, 45.0),
                    Vec2D::new(55.0, 25.0),
                    Vec2D::new(35.0, -55.0),
                    Vec2D::new(145.0, -90.0),
                ],
                color: hull2,
                fill: true,
                stroke_width: 2.0,
            },
            // wing root highlights
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-78.0, 25.0),
                    Vec2D::new(-30.0, 18.0),
                    Vec2D::new(-28.0, -35.0),
                    Vec2D::new(-70.0, -48.0),
                ],
                color: metal,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(78.0, 25.0),
                    Vec2D::new(30.0, 18.0),
                    Vec2D::new(28.0, -35.0),
                    Vec2D::new(70.0, -48.0),
                ],
                color: metal,
                fill: true,
                stroke_width: 2.0,
            },
            // twin engine pods
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-95.0, -40.0),
                    Vec2D::new(-60.0, -40.0),
                    Vec2D::new(-50.0, -120.0),
                    Vec2D::new(-78.0, -145.0),
                    Vec2D::new(-110.0, -120.0),
                ],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(95.0, -40.0),
                    Vec2D::new(60.0, -40.0),
                    Vec2D::new(50.0, -120.0),
                    Vec2D::new(78.0, -145.0),
                    Vec2D::new(110.0, -120.0),
                ],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            // afterburner glow rings
            GeometryShape::Circle {
                center: Vec2D::new(-78.0, -155.0),
                radius: 10.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(78.0, -155.0),
                radius: 10.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
            // belly intake
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-18.0, 30.0),
                    Vec2D::new(18.0, 30.0),
                    Vec2D::new(26.0, -22.0),
                    Vec2D::new(-26.0, -22.0),
                ],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 120.0 },
        scale: 0.46,
    }
}

fn jet_raiden_mk2_blueprint() -> GeometryBlueprint {
    // A more aggressive Raiden variant: forward canards, red/white stripe, bigger engines.
    let hull = ShapeColor::new(0.92, 0.20, 0.28, 1.0);
    let hull2 = ShapeColor::new(0.55, 0.08, 0.14, 1.0);
    let metal = ShapeColor::new(0.86, 0.88, 0.92, 1.0);
    let dark = ShapeColor::new(0.10, 0.12, 0.16, 1.0);
    let glow = ShapeColor::new(0.2, 0.9, 1.0, 0.9);

    GeometryBlueprint {
        name: "jet_raiden_mk2".to_string(),
        shapes: vec![
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 150.0),
                    Vec2D::new(-20.0, 118.0),
                    Vec2D::new(-34.0, 40.0),
                    Vec2D::new(-36.0, -35.0),
                    Vec2D::new(-18.0, -150.0),
                    Vec2D::new(18.0, -150.0),
                    Vec2D::new(36.0, -35.0),
                    Vec2D::new(34.0, 40.0),
                    Vec2D::new(20.0, 118.0),
                ],
                color: metal,
                fill: true,
                stroke_width: 2.0,
            },
            // red armor plates
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 140.0),
                    Vec2D::new(-14.0, 112.0),
                    Vec2D::new(-22.0, 50.0),
                    Vec2D::new(-22.0, -60.0),
                    Vec2D::new(0.0, -140.0),
                    Vec2D::new(22.0, -60.0),
                    Vec2D::new(22.0, 50.0),
                    Vec2D::new(14.0, 112.0),
                ],
                color: hull,
                fill: true,
                stroke_width: 2.0,
            },
            // center stripe
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-6.0, 135.0),
                    Vec2D::new(6.0, 135.0),
                    Vec2D::new(10.0, -130.0),
                    Vec2D::new(-10.0, -130.0),
                ],
                color: ShapeColor::new(1.0, 1.0, 1.0, 0.55),
                fill: true,
                stroke_width: 1.0,
            },
            // cockpit
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 105.0),
                    Vec2D::new(-12.0, 78.0),
                    Vec2D::new(0.0, 55.0),
                    Vec2D::new(12.0, 78.0),
                ],
                color: ShapeColor::new(0.86, 0.95, 1.0, 0.9),
                fill: true,
                stroke_width: 2.0,
            },
            // wings
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-190.0, 55.0),
                    Vec2D::new(-58.0, 30.0),
                    Vec2D::new(-35.0, -55.0),
                    Vec2D::new(-150.0, -105.0),
                ],
                color: hull2,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(190.0, 55.0),
                    Vec2D::new(58.0, 30.0),
                    Vec2D::new(35.0, -55.0),
                    Vec2D::new(150.0, -105.0),
                ],
                color: hull2,
                fill: true,
                stroke_width: 2.0,
            },
            // canards
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-85.0, 95.0), Vec2D::new(-28.0, 88.0), Vec2D::new(-60.0, 70.0)],
                color: hull,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(85.0, 95.0), Vec2D::new(28.0, 88.0), Vec2D::new(60.0, 70.0)],
                color: hull,
                fill: true,
                stroke_width: 2.0,
            },
            // big twin pods
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-110.0, -35.0),
                    Vec2D::new(-70.0, -35.0),
                    Vec2D::new(-55.0, -120.0),
                    Vec2D::new(-85.0, -155.0),
                    Vec2D::new(-125.0, -120.0),
                ],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(110.0, -35.0),
                    Vec2D::new(70.0, -35.0),
                    Vec2D::new(55.0, -120.0),
                    Vec2D::new(85.0, -155.0),
                    Vec2D::new(125.0, -120.0),
                ],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(-85.0, -165.0),
                radius: 11.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(85.0, -165.0),
                radius: 11.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
            // intake
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-20.0, 25.0),
                    Vec2D::new(20.0, 25.0),
                    Vec2D::new(28.0, -28.0),
                    Vec2D::new(-28.0, -28.0),
                ],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 125.0 },
        scale: 0.45,
    }
}

fn jet_raiden_stealth_blueprint() -> GeometryBlueprint {
    // "Game stealth" Raiden: very angular facets + teal glow strips.
    let hull = ShapeColor::new(0.18, 0.2, 0.26, 1.0);
    let hull2 = ShapeColor::new(0.11, 0.12, 0.16, 1.0);
    let glow = ShapeColor::new(0.10, 0.95, 0.85, 0.8);
    let glass = ShapeColor::new(0.86, 0.95, 1.0, 0.7);

    GeometryBlueprint {
        name: "jet_raiden_stealth".to_string(),
        shapes: vec![
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 155.0),
                    Vec2D::new(-26.0, 120.0),
                    Vec2D::new(-42.0, 55.0),
                    Vec2D::new(-46.0, -25.0),
                    Vec2D::new(-26.0, -155.0),
                    Vec2D::new(26.0, -155.0),
                    Vec2D::new(46.0, -25.0),
                    Vec2D::new(42.0, 55.0),
                    Vec2D::new(26.0, 120.0),
                ],
                color: hull,
                fill: true,
                stroke_width: 2.0,
            },
            // blade wings
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-210.0, 40.0),
                    Vec2D::new(-70.0, 25.0),
                    Vec2D::new(-40.0, -55.0),
                    Vec2D::new(-150.0, -120.0),
                ],
                color: hull2,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(210.0, 40.0),
                    Vec2D::new(70.0, 25.0),
                    Vec2D::new(40.0, -55.0),
                    Vec2D::new(150.0, -120.0),
                ],
                color: hull2,
                fill: true,
                stroke_width: 2.0,
            },
            // cockpit wedge
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(0.0, 105.0), Vec2D::new(-14.0, 78.0), Vec2D::new(0.0, 50.0), Vec2D::new(14.0, 78.0)],
                color: glass,
                fill: true,
                stroke_width: 2.0,
            },
            // glow strips
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-6.0, 130.0), Vec2D::new(6.0, 130.0), Vec2D::new(12.0, -120.0), Vec2D::new(-12.0, -120.0)],
                color: ShapeColor::new(0.10, 0.95, 0.85, 0.18),
                fill: true,
                stroke_width: 1.0,
            },
            GeometryShape::Line {
                start: Vec2D::new(-60.0, 30.0),
                end: Vec2D::new(-25.0, -95.0),
                color: glow,
                stroke_width: 3.0,
            },
            GeometryShape::Line {
                start: Vec2D::new(60.0, 30.0),
                end: Vec2D::new(25.0, -95.0),
                color: glow,
                stroke_width: 3.0,
            },
            // pods + exhaust
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-118.0, -30.0), Vec2D::new(-78.0, -30.0), Vec2D::new(-62.0, -118.0), Vec2D::new(-92.0, -160.0), Vec2D::new(-132.0, -118.0)],
                color: ShapeColor::new(0.07, 0.08, 0.10, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(118.0, -30.0), Vec2D::new(78.0, -30.0), Vec2D::new(62.0, -118.0), Vec2D::new(92.0, -160.0), Vec2D::new(132.0, -118.0)],
                color: ShapeColor::new(0.07, 0.08, 0.10, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(-92.0, -170.0),
                radius: 10.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(92.0, -170.0),
                radius: 10.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 135.0 },
        scale: 0.44,
    }
}

fn jet_viper_blueprint() -> GeometryBlueprint {
    // F-16-ish single engine + bubble canopy.
    let canopy = ShapeColor::new(0.86, 0.95, 1.0, 0.95);
    let intake = ShapeColor::new(0.12, 0.14, 0.18, 1.0);
    let hull = ShapeColor::new(0.82, 0.84, 0.88, 1.0);
    let wing = ShapeColor::new(0.28, 0.32, 0.45, 1.0);
    let accent = ShapeColor::new(0.95, 0.55, 0.2, 1.0);

    GeometryBlueprint {
        name: "jet_viper".to_string(),
        shapes: vec![
            // fuselage spine
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-14.0, 105.0),
                    Vec2D::new(14.0, 105.0),
                    Vec2D::new(30.0, 30.0),
                    Vec2D::new(26.0, -105.0),
                    Vec2D::new(-26.0, -105.0),
                    Vec2D::new(-30.0, 30.0),
                ],
                color: hull,
                fill: true,
                stroke_width: 2.0,
            },
            // nose cone
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(0.0, 132.0), Vec2D::new(-18.0, 96.0), Vec2D::new(18.0, 96.0)],
                color: accent,
                fill: true,
                stroke_width: 2.0,
            },
            // canopy (bubble)
            GeometryShape::Circle {
                center: Vec2D::new(0.0, 55.0),
                radius: 16.0,
                color: canopy,
                fill: true,
                stroke_width: 1.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(-6.0, 62.0),
                radius: 6.0,
                color: ShapeColor::new(1.0, 1.0, 1.0, 0.35),
                fill: true,
                stroke_width: 1.0,
            },
            // main wings
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-140.0, 25.0),
                    Vec2D::new(-30.0, 10.0),
                    Vec2D::new(-18.0, -30.0),
                    Vec2D::new(-110.0, -12.0),
                ],
                color: wing,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(140.0, 25.0),
                    Vec2D::new(30.0, 10.0),
                    Vec2D::new(18.0, -30.0),
                    Vec2D::new(110.0, -12.0),
                ],
                color: wing,
                fill: true,
                stroke_width: 2.0,
            },
            // belly intake
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-18.0, 10.0),
                    Vec2D::new(18.0, 10.0),
                    Vec2D::new(26.0, -35.0),
                    Vec2D::new(-26.0, -35.0),
                ],
                color: intake,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-10.0, 8.0),
                    Vec2D::new(10.0, 8.0),
                    Vec2D::new(14.0, -18.0),
                    Vec2D::new(-14.0, -18.0),
                ],
                color: ShapeColor::new(0.25, 0.85, 1.0, 0.25),
                fill: true,
                stroke_width: 1.0,
            },
            // single tail
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-10.0, -35.0),
                    Vec2D::new(10.0, -35.0),
                    Vec2D::new(0.0, -95.0),
                ],
                color: ShapeColor::new(0.22, 0.25, 0.3, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            // stabilizers
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-60.0, -55.0), Vec2D::new(-16.0, -55.0), Vec2D::new(-40.0, -85.0)],
                color: ShapeColor::new(0.22, 0.25, 0.3, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(60.0, -55.0), Vec2D::new(16.0, -55.0), Vec2D::new(40.0, -85.0)],
                color: ShapeColor::new(0.22, 0.25, 0.3, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            // afterburner glow ring (stroke circle)
            GeometryShape::Circle {
                center: Vec2D::new(0.0, -115.0),
                radius: 14.0,
                color: ShapeColor::new(0.15, 0.95, 0.85, 1.0),
                fill: false,
                stroke_width: 4.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(0.0, -115.0),
                radius: 7.0,
                color: ShapeColor::new(0.10, 0.40, 0.95, 0.55),
                fill: true,
                stroke_width: 1.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 95.0 },
        scale: 0.55,
    }
}

fn jet_raptor_blueprint() -> GeometryBlueprint {
    // F-22-ish: faceted body, twin tails, angular intakes.
    let hull = ShapeColor::new(0.55, 0.6, 0.68, 1.0);
    let dark = ShapeColor::new(0.18, 0.2, 0.26, 1.0);
    let wing = ShapeColor::new(0.26, 0.3, 0.36, 1.0);
    let glow = ShapeColor::new(0.1, 0.85, 1.0, 0.7);

    GeometryBlueprint {
        name: "jet_raptor".to_string(),
        shapes: vec![
            // faceted fuselage
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 135.0),
                    Vec2D::new(-22.0, 105.0),
                    Vec2D::new(-38.0, 45.0),
                    Vec2D::new(-40.0, -30.0),
                    Vec2D::new(-22.0, -125.0),
                    Vec2D::new(22.0, -125.0),
                    Vec2D::new(40.0, -30.0),
                    Vec2D::new(38.0, 45.0),
                    Vec2D::new(22.0, 105.0),
                ],
                color: hull,
                fill: true,
                stroke_width: 2.0,
            },
            // cockpit wedge
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(0.0, 95.0), Vec2D::new(-14.0, 70.0), Vec2D::new(14.0, 70.0)],
                color: ShapeColor::new(0.86, 0.95, 1.0, 0.9),
                fill: true,
                stroke_width: 2.0,
            },
            // main wings (diamond-ish)
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-165.0, 25.0),
                    Vec2D::new(-55.0, 15.0),
                    Vec2D::new(-30.0, -35.0),
                    Vec2D::new(-120.0, -65.0),
                ],
                color: wing,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(165.0, 25.0),
                    Vec2D::new(55.0, 15.0),
                    Vec2D::new(30.0, -35.0),
                    Vec2D::new(120.0, -65.0),
                ],
                color: wing,
                fill: true,
                stroke_width: 2.0,
            },
            // intakes (angled)
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-45.0, 25.0), Vec2D::new(-12.0, 15.0), Vec2D::new(-18.0, -25.0), Vec2D::new(-58.0, -10.0)],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(45.0, 25.0), Vec2D::new(12.0, 15.0), Vec2D::new(18.0, -25.0), Vec2D::new(58.0, -10.0)],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            // twin tails
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-22.0, -25.0), Vec2D::new(-4.0, -25.0), Vec2D::new(-18.0, -95.0)],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(22.0, -25.0), Vec2D::new(4.0, -25.0), Vec2D::new(18.0, -95.0)],
                color: dark,
                fill: true,
                stroke_width: 2.0,
            },
            // twin exhausts
            GeometryShape::Circle {
                center: Vec2D::new(-12.0, -132.0),
                radius: 9.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(12.0, -132.0),
                radius: 9.0,
                color: glow,
                fill: false,
                stroke_width: 4.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 105.0 },
        scale: 0.52,
    }
}

fn jet_arrowhead_blueprint() -> GeometryBlueprint {
    GeometryBlueprint {
        name: "jet_arrowhead".to_string(),
        shapes: vec![
            // main body (long hex)
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-20.0, 80.0),
                    Vec2D::new(20.0, 80.0),
                    Vec2D::new(30.0, 10.0),
                    Vec2D::new(20.0, -85.0),
                    Vec2D::new(-20.0, -85.0),
                    Vec2D::new(-30.0, 10.0),
                ],
                color: ShapeColor::new(0.86, 0.88, 0.92, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            // nose
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(0.0, 105.0), Vec2D::new(-22.0, 70.0), Vec2D::new(22.0, 70.0)],
                color: ShapeColor::ORANGE,
                fill: true,
                stroke_width: 2.0,
            },
            // swept wings
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-95.0, 10.0), Vec2D::new(-25.0, 5.0), Vec2D::new(-10.0, -40.0), Vec2D::new(-80.0, -25.0)],
                color: ShapeColor::new(0.25, 0.35, 0.6, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(95.0, 10.0), Vec2D::new(25.0, 5.0), Vec2D::new(10.0, -40.0), Vec2D::new(80.0, -25.0)],
                color: ShapeColor::new(0.25, 0.35, 0.6, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            // twin engines
            GeometryShape::Circle {
                center: Vec2D::new(-14.0, -95.0),
                radius: 10.0,
                color: ShapeColor::new(0.10, 0.90, 0.70, 1.0),
                fill: true,
                stroke_width: 1.0,
            },
            GeometryShape::Circle {
                center: Vec2D::new(14.0, -95.0),
                radius: 10.0,
                color: ShapeColor::new(0.10, 0.90, 0.70, 1.0),
                fill: true,
                stroke_width: 1.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 70.0 },
        scale: 0.7,
    }
}

fn jet_twin_blade_blueprint() -> GeometryBlueprint {
    GeometryBlueprint {
        name: "jet_twin_blade".to_string(),
        shapes: vec![
            // core capsule-ish
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-25.0, 70.0),
                    Vec2D::new(25.0, 70.0),
                    Vec2D::new(35.0, 10.0),
                    Vec2D::new(25.0, -75.0),
                    Vec2D::new(-25.0, -75.0),
                    Vec2D::new(-35.0, 10.0),
                ],
                color: ShapeColor::new(0.75, 0.2, 0.95, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            // side blades (dark)
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(-75.0, 70.0), Vec2D::new(-55.0, 75.0), Vec2D::new(-20.0, -75.0), Vec2D::new(-40.0, -80.0)],
                color: ShapeColor::new(0.12, 0.12, 0.18, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: vec![Vec2D::new(75.0, 70.0), Vec2D::new(55.0, 75.0), Vec2D::new(20.0, -75.0), Vec2D::new(40.0, -80.0)],
                color: ShapeColor::new(0.12, 0.12, 0.18, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            // bridge wing
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-95.0, -5.0),
                    Vec2D::new(95.0, -5.0),
                    Vec2D::new(95.0, -20.0),
                    Vec2D::new(-95.0, -20.0),
                ],
                color: ShapeColor::YELLOW,
                fill: true,
                stroke_width: 2.0,
            },
            // cockpit
            GeometryShape::Circle {
                center: Vec2D::new(0.0, 25.0),
                radius: 12.0,
                color: ShapeColor::new(0.9, 0.95, 1.0, 1.0),
                fill: true,
                stroke_width: 1.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 75.0 },
        scale: 0.7,
    }
}

fn jet_heavy_bomber_blueprint() -> GeometryBlueprint {
    let wing = GeometryShape::Polygon {
        vertices: vec![
            Vec2D::new(-130.0, 5.0),
            Vec2D::new(130.0, 5.0),
            Vec2D::new(130.0, -25.0),
            Vec2D::new(-130.0, -25.0),
        ],
        color: ShapeColor::new(0.35, 0.35, 0.4, 1.0),
        fill: true,
        stroke_width: 2.0,
    };

    GeometryBlueprint {
        name: "jet_heavy_bomber".to_string(),
        shapes: vec![
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-35.0, 80.0),
                    Vec2D::new(35.0, 80.0),
                    Vec2D::new(50.0, 15.0),
                    Vec2D::new(35.0, -80.0),
                    Vec2D::new(-35.0, -80.0),
                    Vec2D::new(-50.0, 15.0),
                ],
                color: ShapeColor::new(0.65, 0.65, 0.7, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            wing,
            GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(-60.0, 50.0),
                    Vec2D::new(60.0, 50.0),
                    Vec2D::new(60.0, 28.0),
                    Vec2D::new(-60.0, 28.0),
                ],
                color: ShapeColor::new(0.95, 0.35, 0.3, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            // engines (triangular exhaust)
            GeometryShape::Polygon {
                vertices: regular_polygon_vertices(3, 14.0)
                    .into_iter()
                    .map(|mut v| {
                        v.y -= 105.0;
                        v.x -= 28.0;
                        v
                    })
                    .collect(),
                color: ShapeColor::new(0.10, 0.70, 0.95, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: regular_polygon_vertices(3, 14.0)
                    .into_iter()
                    .map(|mut v| {
                        v.y -= 105.0;
                        v
                    })
                    .collect(),
                color: ShapeColor::new(0.10, 0.70, 0.95, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
            GeometryShape::Polygon {
                vertices: regular_polygon_vertices(3, 14.0)
                    .into_iter()
                    .map(|mut v| {
                        v.y -= 105.0;
                        v.x += 28.0;
                        v
                    })
                    .collect(),
                color: ShapeColor::new(0.10, 0.70, 0.95, 1.0),
                fill: true,
                stroke_width: 2.0,
            },
        ],
        collision: CollisionShape::Circle { radius: 90.0 },
        scale: 0.65,
    }
}

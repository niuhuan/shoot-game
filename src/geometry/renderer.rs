//! 几何形状渲染器
//! 将 GeometryBlueprint 转换为 Bevy 可渲染的实体

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::f32::consts::PI;

use super::shapes::*;

/// 几何渲染插件
pub struct GeometryRendererPlugin;

impl Plugin for GeometryRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .add_systems(Update, update_geometry_colors);
    }
}

/// 颜色动画组件
#[derive(Component)]
pub struct ColorPulse {
    pub base_color: ShapeColor,
    pub pulse_color: ShapeColor,
    pub speed: f32,
    pub time: f32,
}

/// 更新颜色动画
fn update_geometry_colors(time: Res<Time>, mut query: Query<(&mut Shape, &mut ColorPulse)>) {
    for (mut shape, mut pulse) in query.iter_mut() {
        pulse.time += time.delta_secs() * pulse.speed;
        let t = (pulse.time.sin() + 1.0) / 2.0;

        let r = pulse.base_color.r + (pulse.pulse_color.r - pulse.base_color.r) * t;
        let g = pulse.base_color.g + (pulse.pulse_color.g - pulse.base_color.g) * t;
        let b = pulse.base_color.b + (pulse.pulse_color.b - pulse.base_color.b) * t;
        let a = pulse.base_color.a + (pulse.pulse_color.a - pulse.base_color.a) * t;

        if let Some(fill) = shape.fill.as_mut() {
            fill.color = Color::srgba(r, g, b, a);
        }
    }
}

/// 从 GeometryBlueprint 生成渲染实体
pub fn spawn_geometry_entity(
    commands: &mut Commands,
    blueprint: &GeometryBlueprint,
    position: Vec3,
) -> Entity {
    let parent = commands
        .spawn((
            Transform::from_translation(position).with_scale(Vec3::splat(blueprint.scale)),
            Visibility::default(),
            GeometryData::new(blueprint.clone()),
        ))
        .id();

    // 为每个形状生成子实体
    for shape in &blueprint.shapes {
        let child = spawn_shape(commands, shape);
        commands.entity(parent).add_child(child);
    }

    parent
}

/// 生成单个形状实体
fn spawn_shape(commands: &mut Commands, shape: &GeometryShape) -> Entity {
    match shape {
        GeometryShape::Polygon {
            vertices,
            color,
            fill,
            stroke_width,
        } => spawn_polygon(commands, vertices, *color, *fill, *stroke_width),
        GeometryShape::Arc {
            center,
            radius,
            start_angle,
            end_angle,
            color,
            stroke_width,
        } => spawn_arc(
            commands,
            *center,
            *radius,
            *start_angle,
            *end_angle,
            *color,
            *stroke_width,
        ),
        GeometryShape::Circle {
            center,
            radius,
            color,
            fill,
            stroke_width,
        } => spawn_circle(commands, *center, *radius, *color, *fill, *stroke_width),
        GeometryShape::Line {
            start,
            end,
            color,
            stroke_width,
        } => spawn_line(commands, *start, *end, *color, *stroke_width),
    }
}

/// 生成多边形
fn spawn_polygon(
    commands: &mut Commands,
    vertices: &[Vec2D],
    color: ShapeColor,
    fill: bool,
    stroke_width: f32,
) -> Entity {
    let points: Vec<Vec2> = vertices.iter().map(|v| Vec2::from(*v)).collect();
    let polygon = shapes::Polygon {
        points,
        closed: true,
    };

    let builder = ShapeBuilder::with(&polygon);
    let shape = if fill {
        builder.fill(Fill::color(Color::from(color))).build()
    } else {
        builder
            .stroke(Stroke::new(Color::from(color), stroke_width))
            .build()
    };

    commands
        .spawn((shape, Transform::default(), Visibility::default()))
        .id()
}

/// 生成弧形
fn spawn_arc(
    commands: &mut Commands,
    center: Vec2D,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    color: ShapeColor,
    stroke_width: f32,
) -> Entity {
    // 使用多段线近似弧形
    let segments = 32;
    let angle_step = (end_angle - start_angle) / segments as f32;

    let mut points = Vec::new();
    for i in 0..=segments {
        let angle = start_angle + angle_step * i as f32;
        points.push(Vec2::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }

    let polyline = shapes::Polygon {
        points,
        closed: false,
    };

    let shape = ShapeBuilder::with(&polyline)
        .stroke(Stroke::new(Color::from(color), stroke_width))
        .build();

    commands
        .spawn((shape, Transform::default(), Visibility::default()))
        .id()
}

/// 生成圆形
fn spawn_circle(
    commands: &mut Commands,
    center: Vec2D,
    radius: f32,
    color: ShapeColor,
    fill: bool,
    stroke_width: f32,
) -> Entity {
    let circle = shapes::Circle {
        radius,
        center: Vec2::from(center),
    };

    let builder = ShapeBuilder::with(&circle);
    let shape = if fill {
        builder.fill(Fill::color(Color::from(color))).build()
    } else {
        builder
            .stroke(Stroke::new(Color::from(color), stroke_width))
            .build()
    };

    commands
        .spawn((shape, Transform::default(), Visibility::default()))
        .id()
}

/// 生成线段
fn spawn_line(
    commands: &mut Commands,
    start: Vec2D,
    end: Vec2D,
    color: ShapeColor,
    stroke_width: f32,
) -> Entity {
    let line = shapes::Line(Vec2::from(start), Vec2::from(end));

    let shape = ShapeBuilder::with(&line)
        .stroke(Stroke::new(Color::from(color), stroke_width))
        .build();

    commands
        .spawn((shape, Transform::default(), Visibility::default()))
        .id()
}

/// 辅助函数：创建规则多边形顶点
pub fn regular_polygon_vertices(sides: usize, radius: f32) -> Vec<Vec2D> {
    let mut vertices = Vec::with_capacity(sides);
    let angle_step = 2.0 * PI / sides as f32;

    for i in 0..sides {
        let angle = angle_step * i as f32 - PI / 2.0; // 从顶部开始
        vertices.push(Vec2D::new(radius * angle.cos(), radius * angle.sin()));
    }

    vertices
}

/// 辅助函数：创建星形顶点
pub fn star_vertices(points: usize, outer_radius: f32, inner_radius: f32) -> Vec<Vec2D> {
    let mut vertices = Vec::with_capacity(points * 2);
    let angle_step = PI / points as f32;

    for i in 0..(points * 2) {
        let angle = angle_step * i as f32 - PI / 2.0;
        let r = if i % 2 == 0 {
            outer_radius
        } else {
            inner_radius
        };
        vertices.push(Vec2D::new(r * angle.cos(), r * angle.sin()));
    }

    vertices
}

//! 几何形状定义
//! 支持多边形、弧形、圆形等基础形状，以及复合几何体

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// 颜色定义，用于序列化
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShapeColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ShapeColor {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);
    pub const ORANGE: Self = Self::new(1.0, 0.5, 0.0, 1.0);
}

impl From<ShapeColor> for Color {
    fn from(c: ShapeColor) -> Self {
        Color::srgba(c.r, c.g, c.b, c.a)
    }
}

impl From<Color> for ShapeColor {
    fn from(c: Color) -> Self {
        let rgba = c.to_srgba();
        Self {
            r: rgba.red,
            g: rgba.green,
            b: rgba.blue,
            a: rgba.alpha,
        }
    }
}

/// 2D 向量，用于序列化
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl From<Vec2D> for Vec2 {
    fn from(v: Vec2D) -> Self {
        Vec2::new(v.x, v.y)
    }
}

impl From<Vec2> for Vec2D {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// 基础几何形状
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeometryShape {
    /// 多边形
    Polygon {
        /// 顶点列表（相对于中心点）
        vertices: Vec<Vec2D>,
        /// 颜色
        color: ShapeColor,
        /// 是否填充
        fill: bool,
        /// 线条宽度（仅当 fill = false 时有效）
        stroke_width: f32,
    },
    /// 弧形
    Arc {
        /// 中心点偏移
        center: Vec2D,
        /// 半径
        radius: f32,
        /// 起始角度（弧度）
        start_angle: f32,
        /// 结束角度（弧度）
        end_angle: f32,
        /// 颜色
        color: ShapeColor,
        /// 线条宽度
        stroke_width: f32,
    },
    /// 圆形
    Circle {
        /// 中心点偏移
        center: Vec2D,
        /// 半径
        radius: f32,
        /// 颜色
        color: ShapeColor,
        /// 是否填充
        fill: bool,
        /// 线条宽度（仅当 fill = false 时有效）
        stroke_width: f32,
    },
    /// 线段
    Line {
        /// 起点
        start: Vec2D,
        /// 终点
        end: Vec2D,
        /// 颜色
        color: ShapeColor,
        /// 线条宽度
        stroke_width: f32,
    },
}

/// 碰撞形状
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionShape {
    /// 圆形碰撞箱
    Circle { radius: f32 },
    /// 矩形碰撞箱
    Rectangle { width: f32, height: f32 },
    /// 多边形碰撞箱
    Polygon { vertices: Vec<Vec2D> },
}

impl Default for CollisionShape {
    fn default() -> Self {
        Self::Circle { radius: 10.0 }
    }
}

/// 复合几何实体定义
/// 用于定义飞机、敌人、子弹等实体的外观
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryBlueprint {
    /// 实体名称
    pub name: String,
    /// 组成实体的形状列表
    pub shapes: Vec<GeometryShape>,
    /// 碰撞箱
    pub collision: CollisionShape,
    /// 缩放因子
    pub scale: f32,
}

impl Default for GeometryBlueprint {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            shapes: vec![],
            collision: CollisionShape::default(),
            scale: 1.0,
        }
    }
}

impl GeometryBlueprint {
    /// 创建默认玩家飞机蓝图
    pub fn default_player() -> Self {
        Self {
            name: "player".to_string(),
            shapes: vec![
                // 主体三角形
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 20.0),    // 顶点
                        Vec2D::new(-15.0, -15.0), // 左下
                        Vec2D::new(15.0, -15.0),  // 右下
                    ],
                    color: ShapeColor::CYAN,
                    fill: true,
                    stroke_width: 2.0,
                },
                // 左翼
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-15.0, -10.0),
                        Vec2D::new(-25.0, -20.0),
                        Vec2D::new(-10.0, -15.0),
                    ],
                    color: ShapeColor::BLUE,
                    fill: true,
                    stroke_width: 2.0,
                },
                // 右翼
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(15.0, -10.0),
                        Vec2D::new(25.0, -20.0),
                        Vec2D::new(10.0, -15.0),
                    ],
                    color: ShapeColor::BLUE,
                    fill: true,
                    stroke_width: 2.0,
                },
                // 座舱
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 0.0),
                    radius: 5.0,
                    color: ShapeColor::WHITE,
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 15.0 },
            scale: 1.0,
        }
    }

    /// 创建默认敌人蓝图（菱形）
    pub fn default_enemy() -> Self {
        Self {
            name: "enemy_diamond".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 15.0),
                        Vec2D::new(-12.0, 0.0),
                        Vec2D::new(0.0, -15.0),
                        Vec2D::new(12.0, 0.0),
                    ],
                    color: ShapeColor::RED,
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 4.0,
                    color: ShapeColor::YELLOW,
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 12.0 },
            scale: 1.0,
        }
    }

    /// 创建六边形敌人
    pub fn hexagon_enemy() -> Self {
        let radius = 18.0;
        let mut vertices = Vec::new();
        for i in 0..6 {
            let angle = (i as f32) * PI / 3.0 - PI / 2.0;
            vertices.push(Vec2D::new(radius * angle.cos(), radius * angle.sin()));
        }

        Self {
            name: "enemy_hexagon".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices,
                    color: ShapeColor::ORANGE,
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 6.0,
                    color: ShapeColor::RED,
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 18.0 },
            scale: 1.0,
        }
    }

    /// 创建默认子弹蓝图
    pub fn default_bullet() -> Self {
        Self {
            name: "bullet".to_string(),
            shapes: vec![GeometryShape::Circle {
                center: Vec2D::ZERO,
                radius: 4.0,
                color: ShapeColor::YELLOW,
                fill: true,
                stroke_width: 1.0,
            }],
            collision: CollisionShape::Circle { radius: 4.0 },
            scale: 1.0,
        }
    }

    /// 创建敌人子弹蓝图
    pub fn enemy_bullet() -> Self {
        Self {
            name: "enemy_bullet".to_string(),
            shapes: vec![GeometryShape::Circle {
                center: Vec2D::ZERO,
                radius: 5.0,
                color: ShapeColor::MAGENTA,
                fill: true,
                stroke_width: 1.0,
            }],
            collision: CollisionShape::Circle { radius: 5.0 },
            scale: 1.0,
        }
    }

    /// 创建护盾蓝图
    pub fn default_shield() -> Self {
        Self {
            name: "shield".to_string(),
            shapes: vec![GeometryShape::Arc {
                center: Vec2D::ZERO,
                radius: 25.0,
                start_angle: -PI * 0.75,
                end_angle: PI * 0.75,
                color: ShapeColor::new(0.0, 0.8, 1.0, 0.6),
                stroke_width: 4.0,
            }],
            collision: CollisionShape::Circle { radius: 25.0 },
            scale: 1.0,
        }
    }

    /// 创建道具蓝图（星形）
    pub fn power_up() -> Self {
        let outer_radius = 12.0;
        let inner_radius = 6.0;
        let mut vertices = Vec::new();

        for i in 0..10 {
            let angle = (i as f32) * PI / 5.0 - PI / 2.0;
            let r = if i % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            };
            vertices.push(Vec2D::new(r * angle.cos(), r * angle.sin()));
        }

        Self {
            name: "power_up".to_string(),
            shapes: vec![GeometryShape::Polygon {
                vertices,
                color: ShapeColor::GREEN,
                fill: true,
                stroke_width: 2.0,
            }],
            collision: CollisionShape::Circle { radius: 12.0 },
            scale: 1.0,
        }
    }
}

/// Bevy 组件：存储实体的几何蓝图
#[derive(Component, Clone)]
pub struct GeometryData {
    pub blueprint: GeometryBlueprint,
}

impl GeometryData {
    pub fn new(blueprint: GeometryBlueprint) -> Self {
        Self { blueprint }
    }
}

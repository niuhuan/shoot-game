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
    /// 玩家战机：雷电（Raiden）Mk-I 风格
    pub fn player_raiden_mk1() -> Self {
        // 以“尖鼻、对称大翼、双发动机舱、机腹进气道”为核心特征。
        Self {
            name: "player_raiden_mk1".to_string(),
            shapes: vec![
                // 中央机身（长条多边形）
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 28.0),
                        Vec2D::new(-4.5, 20.0),
                        Vec2D::new(-7.0, 8.0),
                        Vec2D::new(-7.5, -10.0),
                        Vec2D::new(-4.0, -30.0),
                        Vec2D::new(4.0, -30.0),
                        Vec2D::new(7.5, -10.0),
                        Vec2D::new(7.0, 8.0),
                        Vec2D::new(4.5, 20.0),
                    ],
                    color: ShapeColor::new(0.12, 0.55, 0.95, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 机鼻高光
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 34.0),
                        Vec2D::new(-3.5, 23.0),
                        Vec2D::new(3.5, 23.0),
                    ],
                    color: ShapeColor::new(0.80, 0.85, 0.95, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 座舱
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 18.0),
                        Vec2D::new(-3.5, 12.0),
                        Vec2D::new(0.0, 6.0),
                        Vec2D::new(3.5, 12.0),
                    ],
                    color: ShapeColor::new(0.86, 0.95, 1.0, 0.9),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 主翼（大三角翼）
                GeometryShape::Polygon {
                    vertices: vec![
                        // 前窄后宽：更接近雷电街机的外形
                        Vec2D::new(-22.0, 10.0),
                        Vec2D::new(-11.0, 6.0),
                        Vec2D::new(-7.0, -12.0),
                        Vec2D::new(-44.0, -22.0),
                    ],
                    color: ShapeColor::new(0.08, 0.32, 0.65, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(22.0, 10.0),
                        Vec2D::new(11.0, 6.0),
                        Vec2D::new(7.0, -12.0),
                        Vec2D::new(44.0, -22.0),
                    ],
                    color: ShapeColor::new(0.08, 0.32, 0.65, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 机腹进气道
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-4.5, 6.0),
                        Vec2D::new(4.5, 6.0),
                        Vec2D::new(6.5, -6.0),
                        Vec2D::new(-6.5, -6.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 双发动机舱（外侧短舱）
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-18.0, -8.0),
                        Vec2D::new(-12.0, -8.0),
                        Vec2D::new(-10.5, -22.0),
                        Vec2D::new(-14.5, -30.0),
                        Vec2D::new(-20.0, -22.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(18.0, -8.0),
                        Vec2D::new(12.0, -8.0),
                        Vec2D::new(10.5, -22.0),
                        Vec2D::new(14.5, -30.0),
                        Vec2D::new(20.0, -22.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 喷口辉光（描边圆）
                GeometryShape::Circle {
                    center: Vec2D::new(-14.5, -34.0),
                    radius: 3.8,
                    color: ShapeColor::new(0.12, 0.95, 0.85, 0.9),
                    fill: false,
                    stroke_width: 2.5,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(14.5, -34.0),
                    radius: 3.8,
                    color: ShapeColor::new(0.12, 0.95, 0.85, 0.9),
                    fill: false,
                    stroke_width: 2.5,
                },
            ],
            collision: CollisionShape::Circle { radius: 18.0 },
            scale: 1.0,
        }
    }

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
            name: "enemy_fighter".to_string(),
            // 敌人更像“战机”：尖鼻 + 机身 + 翼面 + 发动机喷口
            shapes: vec![
                // 机身
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 18.0),
                        Vec2D::new(-6.0, 10.0),
                        Vec2D::new(-9.0, -6.0),
                        Vec2D::new(-5.0, -18.0),
                        Vec2D::new(5.0, -18.0),
                        Vec2D::new(9.0, -6.0),
                        Vec2D::new(6.0, 10.0),
                    ],
                    color: ShapeColor::new(0.95, 0.25, 0.35, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 机鼻高光
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 22.0),
                        Vec2D::new(-4.5, 14.0),
                        Vec2D::new(4.5, 14.0),
                    ],
                    color: ShapeColor::new(0.90, 0.90, 0.95, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 主翼（前窄后宽）
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-14.0, 6.0),
                        Vec2D::new(-8.0, 4.0),
                        Vec2D::new(-6.0, -6.0),
                        Vec2D::new(-22.0, -12.0),
                    ],
                    color: ShapeColor::new(0.55, 0.08, 0.14, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(14.0, 6.0),
                        Vec2D::new(8.0, 4.0),
                        Vec2D::new(6.0, -6.0),
                        Vec2D::new(22.0, -12.0),
                    ],
                    color: ShapeColor::new(0.55, 0.08, 0.14, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 座舱/灯
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 8.0),
                    radius: 3.2,
                    color: ShapeColor::new(1.0, 0.95, 0.35, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                // 喷口辉光
                GeometryShape::Circle {
                    center: Vec2D::new(-4.5, -19.5),
                    radius: 2.6,
                    color: ShapeColor::new(0.2, 0.9, 1.0, 0.9),
                    fill: false,
                    stroke_width: 2.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(4.5, -19.5),
                    radius: 2.6,
                    color: ShapeColor::new(0.2, 0.9, 1.0, 0.9),
                    fill: false,
                    stroke_width: 2.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 16.0 },
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
                // 让它更像“重装战机”：两侧挂舱 + 机鼻灯
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-26.0, 8.0),
                        Vec2D::new(-14.0, 6.0),
                        Vec2D::new(-12.0, -12.0),
                        Vec2D::new(-24.0, -14.0),
                    ],
                    color: ShapeColor::new(0.65, 0.35, 0.05, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(26.0, 8.0),
                        Vec2D::new(14.0, 6.0),
                        Vec2D::new(12.0, -12.0),
                        Vec2D::new(24.0, -14.0),
                    ],
                    color: ShapeColor::new(0.65, 0.35, 0.05, 1.0),
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
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 10.0),
                    radius: 3.0,
                    color: ShapeColor::new(1.0, 1.0, 0.2, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 20.0 },
            scale: 1.0,
        }
    }

    /// 创建默认子弹蓝图
    pub fn default_bullet() -> Self {
        Self {
            name: "bullet".to_string(),
            // 雷电风格：细长“针弹”（菱形 + 芯点）
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 9.0),
                        Vec2D::new(-2.2, 0.0),
                        Vec2D::new(0.0, -9.0),
                        Vec2D::new(2.2, 0.0),
                    ],
                    color: ShapeColor::new(0.85, 0.95, 1.0, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 1.6,
                    color: ShapeColor::new(0.10, 0.85, 1.0, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 6.0 },
            scale: 1.0,
        }
    }

    /// 创建敌人子弹蓝图
    pub fn enemy_bullet() -> Self {
        Self {
            name: "enemy_bullet".to_string(),
            // 敌弹改为“街机小圆球”，避免三角形干扰视线
            shapes: vec![
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 5.0,
                    color: ShapeColor::new(1.0, 0.25, 0.85, 0.85),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 2.0,
                    color: ShapeColor::new(1.0, 0.9, 1.0, 0.55),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 5.0 },
            scale: 1.0,
        }
    }

    /// 敌人子弹：圆环弹（街机味更重）
    pub fn enemy_bullet_ring() -> Self {
        Self {
            name: "enemy_bullet_ring".to_string(),
            shapes: vec![
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 6.5,
                    color: ShapeColor::new(1.0, 0.25, 0.85, 0.9),
                    fill: false,
                    stroke_width: 2.5,
                },
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 1.8,
                    color: ShapeColor::new(1.0, 0.9, 1.0, 0.35),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 7.5 },
            scale: 1.0,
        }
    }

    /// 敌人子弹：红色“针弹”
    pub fn enemy_bullet_needle() -> Self {
        Self {
            name: "enemy_bullet_needle".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 10.0),
                        Vec2D::new(-2.3, 0.0),
                        Vec2D::new(0.0, -10.0),
                        Vec2D::new(2.3, 0.0),
                    ],
                    color: ShapeColor::new(1.0, 0.25, 0.25, 0.95),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 6.5 },
            scale: 1.0,
        }
    }

    /// 敌人：小型无人机（Small）
    pub fn raiden_enemy_drone_small() -> Self {
        Self {
            name: "enemy_drone_small".to_string(),
            shapes: vec![
                // silhouette
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 16.0),
                        Vec2D::new(-14.0, 5.0),
                        Vec2D::new(-9.0, -16.0),
                        Vec2D::new(9.0, -16.0),
                        Vec2D::new(14.0, 5.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 0.95),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 14.0),
                        Vec2D::new(-12.0, 4.0),
                        Vec2D::new(-8.0, -14.0),
                        Vec2D::new(8.0, -14.0),
                        Vec2D::new(12.0, 4.0),
                    ],
                    color: ShapeColor::new(1.0, 0.25, 0.35, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // fuselage stripe
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-3.0, 10.0),
                        Vec2D::new(3.0, 10.0),
                        Vec2D::new(4.0, -12.0),
                        Vec2D::new(-4.0, -12.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 0.45),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-18.0, 6.0),
                        Vec2D::new(-10.0, 2.0),
                        Vec2D::new(-14.0, -6.0),
                    ],
                    color: ShapeColor::new(0.65, 0.08, 0.14, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(18.0, 6.0),
                        Vec2D::new(10.0, 2.0),
                        Vec2D::new(14.0, -6.0),
                    ],
                    color: ShapeColor::new(0.65, 0.08, 0.14, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 2.0),
                    radius: 3.0,
                    color: ShapeColor::new(1.0, 1.0, 0.2, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                // cockpit glass
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 6.0),
                    radius: 3.8,
                    color: ShapeColor::new(0.9, 0.98, 1.0, 0.25),
                    fill: true,
                    stroke_width: 1.0,
                },
                // exhaust ring
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, -16.0),
                    radius: 4.2,
                    color: ShapeColor::new(0.15, 0.95, 0.85, 0.55),
                    fill: false,
                    stroke_width: 2.5,
                },
            ],
            collision: CollisionShape::Circle { radius: 14.0 },
            scale: 1.0,
        }
    }

    /// 敌人：装甲机（重型 Hexagon 变体）
    pub fn raiden_enemy_tank() -> Self {
        let radius = 20.0;
        let mut hull = Vec::with_capacity(6);
        for i in 0..6 {
            let angle = (i as f32) * PI / 3.0 - PI / 2.0;
            hull.push(Vec2D::new(radius * angle.cos(), radius * angle.sin() * 1.15));
        }
        Self {
            name: "enemy_tank".to_string(),
            shapes: vec![
                // silhouette
                GeometryShape::Polygon {
                    vertices: {
                        let mut v = Vec::with_capacity(6);
                        for i in 0..6 {
                            let angle = (i as f32) * PI / 3.0 - PI / 2.0;
                            v.push(Vec2D::new((radius + 3.0) * angle.cos(), (radius + 3.0) * angle.sin() * 1.15));
                        }
                        v
                    },
                    color: ShapeColor::new(0.10, 0.12, 0.16, 0.95),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: hull,
                    color: ShapeColor::new(1.0, 0.6, 0.1, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-14.0, 8.0),
                        Vec2D::new(14.0, 8.0),
                        Vec2D::new(18.0, -8.0),
                        Vec2D::new(-18.0, -8.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // armor stripe
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-6.0, 18.0),
                        Vec2D::new(6.0, 18.0),
                        Vec2D::new(10.0, -16.0),
                        Vec2D::new(-10.0, -16.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 0.40),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 0.0),
                    radius: 6.0,
                    color: ShapeColor::RED,
                    fill: true,
                    stroke_width: 1.0,
                },
                // engine glows (bottom)
                GeometryShape::Circle {
                    center: Vec2D::new(-9.0, -20.0),
                    radius: 4.2,
                    color: ShapeColor::new(0.2, 0.9, 1.0, 0.6),
                    fill: false,
                    stroke_width: 2.5,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(9.0, -20.0),
                    radius: 4.2,
                    color: ShapeColor::new(0.2, 0.9, 1.0, 0.6),
                    fill: false,
                    stroke_width: 2.5,
                },
            ],
            collision: CollisionShape::Circle { radius: 22.0 },
            scale: 1.0,
        }
    }

    /// 导弹：雷电风格（用于 Rocket）
    pub fn raiden_missile() -> Self {
        Self {
            name: "raiden_missile".to_string(),
            shapes: vec![
                // silhouette (shadow)
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 28.0),
                        Vec2D::new(-7.0, 18.0),
                        Vec2D::new(-7.5, -20.0),
                        Vec2D::new(-3.0, -30.0),
                        Vec2D::new(3.0, -30.0),
                        Vec2D::new(7.5, -20.0),
                        Vec2D::new(7.0, 18.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 0.95),
                    fill: true,
                    stroke_width: 1.0,
                },
                // main body
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 26.0),
                        Vec2D::new(-6.0, 17.0),
                        Vec2D::new(-6.0, -18.0),
                        Vec2D::new(-2.5, -27.0),
                        Vec2D::new(2.5, -27.0),
                        Vec2D::new(6.0, -18.0),
                        Vec2D::new(6.0, 17.0),
                    ],
                    color: ShapeColor::new(0.92, 0.20, 0.28, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                // nose cone highlight
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 33.0),
                        Vec2D::new(-4.2, 20.0),
                        Vec2D::new(4.2, 20.0),
                    ],
                    color: ShapeColor::new(0.86, 0.88, 0.92, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                // body stripe
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-2.0, 18.0),
                        Vec2D::new(2.0, 18.0),
                        Vec2D::new(3.0, -22.0),
                        Vec2D::new(-3.0, -22.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 0.45),
                    fill: true,
                    stroke_width: 1.0,
                },
                // guidance "eye"
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 14.5),
                    radius: 2.2,
                    color: ShapeColor::new(1.0, 0.9, 0.2, 0.95),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 14.5),
                    radius: 4.6,
                    color: ShapeColor::new(1.0, 0.9, 0.2, 0.35),
                    fill: false,
                    stroke_width: 2.0,
                },
                // fins (bigger, layered)
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-14.0, -2.0),
                        Vec2D::new(-6.0, -6.0),
                        Vec2D::new(-7.5, -12.0),
                        Vec2D::new(-16.0, -18.0),
                    ],
                    color: ShapeColor::new(0.55, 0.08, 0.14, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(14.0, -2.0),
                        Vec2D::new(6.0, -6.0),
                        Vec2D::new(7.5, -12.0),
                        Vec2D::new(16.0, -18.0),
                    ],
                    color: ShapeColor::new(0.55, 0.08, 0.14, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-11.5, -6.0),
                        Vec2D::new(-6.5, -8.0),
                        Vec2D::new(-8.5, -13.0),
                        Vec2D::new(-13.5, -14.0),
                    ],
                    color: ShapeColor::new(0.86, 0.88, 0.92, 0.55),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(11.5, -6.0),
                        Vec2D::new(6.5, -8.0),
                        Vec2D::new(8.5, -13.0),
                        Vec2D::new(13.5, -14.0),
                    ],
                    color: ShapeColor::new(0.86, 0.88, 0.92, 0.55),
                    fill: true,
                    stroke_width: 1.0,
                },
                // exhaust flame (filled)
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, -38.0),
                        Vec2D::new(-4.0, -28.0),
                        Vec2D::new(0.0, -22.0),
                        Vec2D::new(4.0, -28.0),
                    ],
                    color: ShapeColor::new(1.0, 0.65, 0.10, 0.85),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, -34.0),
                        Vec2D::new(-2.4, -28.0),
                        Vec2D::new(0.0, -25.0),
                        Vec2D::new(2.4, -28.0),
                    ],
                    color: ShapeColor::new(0.95, 0.95, 1.0, 0.55),
                    fill: true,
                    stroke_width: 1.0,
                },
                // exhaust glow ring (subtle)
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, -28.0),
                    radius: 6.2,
                    color: ShapeColor::new(0.2, 0.9, 1.0, 0.55),
                    fill: false,
                    stroke_width: 3.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 14.0 },
            scale: 1.0,
        }
    }

    /// 精英敌人：侦察机（大体积、精巧、慢速）
    pub fn elite_scout() -> Self {
        Self {
            name: "elite_scout".to_string(),
            shapes: vec![
                // 机身（细长）
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 28.0),
                        Vec2D::new(-10.0, 16.0),
                        Vec2D::new(-14.0, -6.0),
                        Vec2D::new(-8.0, -28.0),
                        Vec2D::new(8.0, -28.0),
                        Vec2D::new(14.0, -6.0),
                        Vec2D::new(10.0, 16.0),
                    ],
                    color: ShapeColor::new(0.25, 0.75, 1.0, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 座舱
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 10.0),
                    radius: 6.0,
                    color: ShapeColor::new(0.9, 0.98, 1.0, 0.65),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 主翼（前窄后宽）
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-18.0, 8.0),
                        Vec2D::new(-10.0, 6.0),
                        Vec2D::new(-8.0, -8.0),
                        Vec2D::new(-34.0, -16.0),
                    ],
                    color: ShapeColor::new(0.10, 0.35, 0.55, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(18.0, 8.0),
                        Vec2D::new(10.0, 6.0),
                        Vec2D::new(8.0, -8.0),
                        Vec2D::new(34.0, -16.0),
                    ],
                    color: ShapeColor::new(0.10, 0.35, 0.55, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 双尾翼
                GeometryShape::Polygon {
                    vertices: vec![Vec2D::new(-8.0, -6.0), Vec2D::new(-2.0, -6.0), Vec2D::new(-6.0, -22.0)],
                    color: ShapeColor::new(0.12, 0.16, 0.22, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![Vec2D::new(8.0, -6.0), Vec2D::new(2.0, -6.0), Vec2D::new(6.0, -22.0)],
                    color: ShapeColor::new(0.12, 0.16, 0.22, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 喷口辉光
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, -30.0),
                    radius: 6.0,
                    color: ShapeColor::new(0.15, 0.95, 0.85, 0.6),
                    fill: false,
                    stroke_width: 3.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 26.0 },
            scale: 1.0,
        }
    }

    /// 精英敌人：炮艇（更宽的挂舱、更高火力）
    pub fn elite_gunship() -> Self {
        Self {
            name: "elite_gunship".to_string(),
            shapes: vec![
                // 机身
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 26.0),
                        Vec2D::new(-14.0, 12.0),
                        Vec2D::new(-18.0, -10.0),
                        Vec2D::new(-10.0, -30.0),
                        Vec2D::new(10.0, -30.0),
                        Vec2D::new(18.0, -10.0),
                        Vec2D::new(14.0, 12.0),
                    ],
                    color: ShapeColor::new(1.0, 0.65, 0.15, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 装甲条纹
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-5.0, 22.0),
                        Vec2D::new(5.0, 22.0),
                        Vec2D::new(8.0, -26.0),
                        Vec2D::new(-8.0, -26.0),
                    ],
                    color: ShapeColor::new(0.10, 0.12, 0.16, 0.55),
                    fill: true,
                    stroke_width: 1.0,
                },
                // 挂舱（左右）
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-44.0, 8.0),
                        Vec2D::new(-20.0, 6.0),
                        Vec2D::new(-18.0, -18.0),
                        Vec2D::new(-40.0, -20.0),
                    ],
                    color: ShapeColor::new(0.65, 0.35, 0.05, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(44.0, 8.0),
                        Vec2D::new(20.0, 6.0),
                        Vec2D::new(18.0, -18.0),
                        Vec2D::new(40.0, -20.0),
                    ],
                    color: ShapeColor::new(0.65, 0.35, 0.05, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 核心灯
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 8.0),
                    radius: 5.0,
                    color: ShapeColor::new(1.0, 1.0, 0.2, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
                // 双喷口
                GeometryShape::Circle {
                    center: Vec2D::new(-8.0, -32.0),
                    radius: 5.0,
                    color: ShapeColor::new(0.2, 0.9, 1.0, 0.6),
                    fill: false,
                    stroke_width: 3.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(8.0, -32.0),
                    radius: 5.0,
                    color: ShapeColor::new(0.2, 0.9, 1.0, 0.6),
                    fill: false,
                    stroke_width: 3.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 30.0 },
            scale: 1.0,
        }
    }

    /// 精英敌人：守卫机（圆环护罩+机身）
    pub fn elite_guard() -> Self {
        Self {
            name: "elite_guard".to_string(),
            shapes: vec![
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 30.0,
                    color: ShapeColor::new(0.9, 0.25, 0.85, 0.16),
                    fill: false,
                    stroke_width: 4.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 24.0),
                        Vec2D::new(-14.0, 10.0),
                        Vec2D::new(-18.0, -8.0),
                        Vec2D::new(-10.0, -26.0),
                        Vec2D::new(10.0, -26.0),
                        Vec2D::new(18.0, -8.0),
                        Vec2D::new(14.0, 10.0),
                    ],
                    color: ShapeColor::new(0.55, 0.2, 0.95, 1.0),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, 6.0),
                    radius: 5.0,
                    color: ShapeColor::new(0.9, 0.98, 1.0, 0.55),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, -28.0),
                    radius: 6.5,
                    color: ShapeColor::new(0.15, 0.95, 0.85, 0.55),
                    fill: false,
                    stroke_width: 3.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 30.0 },
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

    /// 金币道具（小币）
    pub fn power_up_coin() -> Self {
        Self {
            name: "power_up_coin".to_string(),
            shapes: vec![
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 10.0,
                    color: ShapeColor::new(1.0, 0.85, 0.15, 0.95),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: 10.0,
                    color: ShapeColor::new(0.25, 0.15, 0.05, 0.35),
                    fill: false,
                    stroke_width: 2.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(-3.0, 3.0),
                    radius: 3.0,
                    color: ShapeColor::new(1.0, 1.0, 1.0, 0.22),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 10.0 },
            scale: 1.0,
        }
    }

    /// 护盾道具（小盾牌）
    pub fn power_up_shield() -> Self {
        Self {
            name: "power_up_shield".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 14.0),
                        Vec2D::new(-12.0, 8.0),
                        Vec2D::new(-10.0, -8.0),
                        Vec2D::new(0.0, -14.0),
                        Vec2D::new(10.0, -8.0),
                        Vec2D::new(12.0, 8.0),
                    ],
                    color: ShapeColor::new(0.2, 0.85, 1.0, 0.9),
                    fill: true,
                    stroke_width: 2.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, 10.0),
                        Vec2D::new(-8.0, 6.0),
                        Vec2D::new(-7.0, -6.0),
                        Vec2D::new(0.0, -10.0),
                        Vec2D::new(7.0, -6.0),
                        Vec2D::new(8.0, 6.0),
                    ],
                    color: ShapeColor::new(0.9, 0.98, 1.0, 0.28),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: 14.0 },
            scale: 1.0,
        }
    }

    /// 生命道具（心形）
    pub fn power_up_heart() -> Self {
        // 简化心形：上方两个圆 + 下方尖角多边形
        Self {
            name: "power_up_heart".to_string(),
            shapes: vec![
                GeometryShape::Circle {
                    center: Vec2D::new(-5.0, 4.0),
                    radius: 6.5,
                    color: ShapeColor::new(1.0, 0.25, 0.35, 0.92),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(5.0, 4.0),
                    radius: 6.5,
                    color: ShapeColor::new(1.0, 0.25, 0.35, 0.92),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-11.0, 3.0),
                        Vec2D::new(11.0, 3.0),
                        Vec2D::new(0.0, -14.0),
                    ],
                    color: ShapeColor::new(1.0, 0.25, 0.35, 0.92),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(-4.0, 6.0),
                    radius: 2.0,
                    color: ShapeColor::new(1.0, 1.0, 1.0, 0.18),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
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

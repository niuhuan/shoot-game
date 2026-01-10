//! Boss系统

use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::entities::{
    spawn_rocket_explosion_particles, Bullet, HitList, Pierce, RocketBullet, WeaponBullet,
    WeaponType,
};
use crate::game::{
    not_upgrading, Collider, CollisionEvent, CollisionLayer, CollisionMask, GameConfig, GameData,
    GameState,
};
use crate::geometry::{
    spawn_geometry_entity, CollisionShape, GeometryBlueprint, GeometryShape, ShapeColor, Vec2D,
};

/// 创建正多边形顶点
fn regular_polygon_vertices(sides: usize, radius: f32) -> Vec<Vec2D> {
    let mut vertices = Vec::with_capacity(sides);
    for i in 0..sides {
        let angle = -PI / 2.0 + (i as f32) * 2.0 * PI / (sides as f32);
        vertices.push(Vec2D::new(angle.cos() * radius, angle.sin() * radius));
    }
    vertices
}

/// Boss插件
pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BossState::default())
            .add_systems(OnEnter(GameState::Menu), despawn_boss)
            .add_systems(OnEnter(GameState::GameOver), despawn_boss)
            .add_systems(OnEnter(GameState::Recharge), despawn_boss)
            .add_systems(
                Update,
                (
                    check_boss_spawn,
                    boss_behavior,
                    boss_collision_handler,
                    update_boss_bullets,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(not_upgrading),
            );
    }
}

/// Boss状态资源
#[derive(Resource, Default)]
pub struct BossState {
    /// 当前是否有Boss战
    pub active: bool,
    /// 上次触发Boss的等级
    pub last_boss_level: u32,
    /// Boss总血量（用于计算百分比）
    pub total_health: i32,
    /// Boss当前血量
    pub current_health: i32,
    /// Boss名称
    pub boss_name: String,
}

impl BossState {
    pub fn health_percent(&self) -> f32 {
        if self.total_health <= 0 {
            return 0.0;
        }
        (self.current_health as f32 / self.total_health as f32 * 100.0).max(0.0)
    }
}

/// Boss组件
#[derive(Component)]
pub struct Boss {
    pub boss_type: BossType,
    pub health: i32,
    pub max_health: i32,
    pub phase: u32, // 当前阶段（血量低时切换攻击模式）
    pub attack_timer: f32,
    pub attack_pattern: u32, // 当前攻击模式
    pub move_timer: f32,
    pub score_value: u32,
    pub entered: bool, // 是否已进入战场
}

/// Boss类型（10种不同的Boss）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BossType {
    /// 巨型菱形 - 发射扇形弹幕
    DiamondKing,
    /// 六边形堡垒 - 围绕自身旋转发射
    HexFortress,
    /// 三角战机 - 高速冲刺攻击
    TriangleFighter,
    /// 星形母舰 - 召唤小型敌人
    StarMothership,
    /// 圆形护盾 - 有护盾相位
    CircleGuardian,
    /// 十字激光 - 发射十字激光
    CrossLaser,
    /// 螺旋射手 - 螺旋弹幕
    SpiralShooter,
    /// 分裂核心 - 分裂成小Boss
    SplitCore,
    /// 追踪者 - 发射大量追踪弹
    TrackerPrime,
    /// 混沌之眼 - 随机攻击模式
    ChaosEye,
}

impl BossType {
    pub fn all() -> &'static [BossType] {
        &[
            BossType::DiamondKing,
            BossType::HexFortress,
            BossType::TriangleFighter,
            BossType::StarMothership,
            BossType::CircleGuardian,
            BossType::CrossLaser,
            BossType::SpiralShooter,
            BossType::SplitCore,
            BossType::TrackerPrime,
            BossType::ChaosEye,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BossType::DiamondKing => "菱形王",
            BossType::HexFortress => "六边堡垒",
            BossType::TriangleFighter => "三角战机",
            BossType::StarMothership => "星形母舰",
            BossType::CircleGuardian => "圆形守护者",
            BossType::CrossLaser => "十字激光",
            BossType::SpiralShooter => "螺旋射手",
            BossType::SplitCore => "分裂核心",
            BossType::TrackerPrime => "追踪者Prime",
            BossType::ChaosEye => "混沌之眼",
        }
    }

    pub fn base_health(&self) -> i32 {
        match self {
            BossType::DiamondKing => 2000,
            BossType::HexFortress => 3000,
            BossType::TriangleFighter => 1600,
            BossType::StarMothership => 2400,
            BossType::CircleGuardian => 3600,
            BossType::CrossLaser => 2000,
            BossType::SpiralShooter => 1800,
            BossType::SplitCore => 1400,
            BossType::TrackerPrime => 2200,
            BossType::ChaosEye => 2600,
        }
    }

    pub fn score_value(&self) -> u32 {
        match self {
            BossType::DiamondKing => 2000,
            BossType::HexFortress => 3000,
            BossType::TriangleFighter => 1500,
            BossType::StarMothership => 2500,
            BossType::CircleGuardian => 3500,
            BossType::CrossLaser => 2000,
            BossType::SpiralShooter => 1800,
            BossType::SplitCore => 1500,
            BossType::TrackerPrime => 2200,
            BossType::ChaosEye => 2800,
        }
    }

    pub fn color(&self) -> ShapeColor {
        match self {
            BossType::DiamondKing => ShapeColor::new(1.0, 0.3, 0.3, 0.9),
            BossType::HexFortress => ShapeColor::new(0.3, 0.3, 1.0, 0.9),
            BossType::TriangleFighter => ShapeColor::new(0.3, 1.0, 0.3, 0.9),
            BossType::StarMothership => ShapeColor::new(1.0, 1.0, 0.3, 0.9),
            BossType::CircleGuardian => ShapeColor::new(0.3, 1.0, 1.0, 0.9),
            BossType::CrossLaser => ShapeColor::new(1.0, 0.5, 0.0, 0.9),
            BossType::SpiralShooter => ShapeColor::new(0.8, 0.3, 1.0, 0.9),
            BossType::SplitCore => ShapeColor::new(0.5, 0.5, 0.5, 0.9),
            BossType::TrackerPrime => ShapeColor::new(0.0, 0.8, 0.5, 0.9),
            BossType::ChaosEye => ShapeColor::new(1.0, 0.0, 0.5, 0.9),
        }
    }
}

/// Boss子弹标记
#[derive(Component)]
pub struct BossBullet {
    pub damage: i32,
    pub velocity: Vec2,
    pub lifetime: f32,
}

/// 检查是否应该生成Boss
fn check_boss_spawn(
    mut commands: Commands,
    game_data: Res<GameData>,
    mut boss_state: ResMut<BossState>,
    config: Res<GameConfig>,
    existing_boss: Query<Entity, With<Boss>>,
) {
    // 已有Boss战进行中
    if boss_state.active || !existing_boss.is_empty() {
        return;
    }

    // 每10级触发一次Boss
    let boss_level = (game_data.player_level / 10) * 10;
    if boss_level == 0 || boss_level <= boss_state.last_boss_level {
        return;
    }

    // 生成Boss
    boss_state.last_boss_level = boss_level;
    boss_state.active = true;

    // 随机选择Boss类型
    let mut rng = rand::rng();
    let boss_types = BossType::all();
    let boss_type = boss_types[rng.random_range(0..boss_types.len())];

    // 根据等级计算血量（每10级增加100%）
    let level_multiplier = 1.0 + (boss_level as f32 / 10.0 - 1.0) * 1.0;
    let health = (boss_type.base_health() as f32 * level_multiplier) as i32;

    boss_state.total_health = health;
    boss_state.current_health = health;
    boss_state.boss_name = boss_type.name().to_string();

    spawn_boss(&mut commands, &config, boss_type, health);
    log::info!("Boss spawned: {} with {} HP", boss_type.name(), health);
}

/// 生成Boss
fn spawn_boss(commands: &mut Commands, config: &GameConfig, boss_type: BossType, health: i32) {
    let blueprint = create_boss_blueprint(boss_type);
    let position = Vec3::new(0.0, config.window_height / 2.0 + 100.0, 8.0);

    let entity = spawn_geometry_entity(commands, &blueprint, position);

    commands.entity(entity).insert((
        Boss {
            boss_type,
            health,
            max_health: health,
            phase: 1,
            attack_timer: 2.0, // 进入后延迟攻击
            attack_pattern: 0,
            move_timer: 0.0,
            score_value: boss_type.score_value(),
            entered: false,
        },
        Collider::new(blueprint.collision.clone(), CollisionLayer::Enemy)
            .with_mask(CollisionMask::enemy_mask()),
    ));
}

/// 创建Boss蓝图
fn create_boss_blueprint(boss_type: BossType) -> GeometryBlueprint {
    let color = boss_type.color();
    let size = 60.0;

    match boss_type {
        BossType::DiamondKing => GeometryBlueprint {
            name: "boss_diamond_king".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, size),
                        Vec2D::new(-size, 0.0),
                        Vec2D::new(0.0, -size),
                        Vec2D::new(size, 0.0),
                    ],
                    color,
                    fill: true,
                    stroke_width: 3.0,
                },
                // 内部装饰
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(0.0, size * 0.5),
                        Vec2D::new(-size * 0.5, 0.0),
                        Vec2D::new(0.0, -size * 0.5),
                        Vec2D::new(size * 0.5, 0.0),
                    ],
                    color: ShapeColor::new(1.0, 1.0, 1.0, 0.3),
                    fill: true,
                    stroke_width: 2.0,
                },
            ],
            collision: CollisionShape::Circle { radius: size * 0.8 },
            scale: 1.0,
        },
        BossType::HexFortress => GeometryBlueprint {
            name: "boss_hex_fortress".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: regular_polygon_vertices(6, size),
                    color,
                    fill: true,
                    stroke_width: 3.0,
                },
                GeometryShape::Polygon {
                    vertices: regular_polygon_vertices(6, size * 0.6),
                    color: ShapeColor::new(0.5, 0.5, 1.0, 0.5),
                    fill: true,
                    stroke_width: 2.0,
                },
            ],
            collision: CollisionShape::Circle { radius: size * 0.9 },
            scale: 1.0,
        },
        BossType::TriangleFighter => GeometryBlueprint {
            name: "boss_triangle_fighter".to_string(),
            shapes: vec![GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, -size),
                    Vec2D::new(-size * 0.8, size * 0.6),
                    Vec2D::new(size * 0.8, size * 0.6),
                ],
                color,
                fill: true,
                stroke_width: 3.0,
            }],
            collision: CollisionShape::Circle { radius: size * 0.7 },
            scale: 1.0,
        },
        BossType::StarMothership => {
            let mut vertices = Vec::new();
            for i in 0..10 {
                let angle = (i as f32 / 10.0) * PI * 2.0 - PI / 2.0;
                let r = if i % 2 == 0 { size } else { size * 0.5 };
                vertices.push(Vec2D::new(angle.cos() * r, angle.sin() * r));
            }
            GeometryBlueprint {
                name: "boss_star_mothership".to_string(),
                shapes: vec![GeometryShape::Polygon {
                    vertices,
                    color,
                    fill: true,
                    stroke_width: 3.0,
                }],
                collision: CollisionShape::Circle { radius: size * 0.8 },
                scale: 1.0,
            }
        }
        BossType::CircleGuardian => GeometryBlueprint {
            name: "boss_circle_guardian".to_string(),
            shapes: vec![
                // UFO 底盘（扁平多边形，近似椭圆）
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-size * 1.10, 0.0),
                        Vec2D::new(-size * 0.75, size * 0.35),
                        Vec2D::new(-size * 0.20, size * 0.48),
                        Vec2D::new(size * 0.20, size * 0.48),
                        Vec2D::new(size * 0.75, size * 0.35),
                        Vec2D::new(size * 1.10, 0.0),
                        Vec2D::new(size * 0.75, -size * 0.35),
                        Vec2D::new(size * 0.20, -size * 0.48),
                        Vec2D::new(-size * 0.20, -size * 0.48),
                        Vec2D::new(-size * 0.75, -size * 0.35),
                    ],
                    color: ShapeColor::new(0.12, 0.16, 0.22, 0.95),
                    fill: true,
                    stroke_width: 3.0,
                },
                // UFO 外环描边
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: size * 0.95,
                    color: ShapeColor::new(0.3, 1.0, 1.0, 0.35),
                    fill: false,
                    stroke_width: 4.0,
                },
                // 机体本体（保留“圆形守护者”感觉）
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: size * 0.78,
                    color,
                    fill: true,
                    stroke_width: 4.0,
                },
                // 舱罩（半透明圆顶）
                GeometryShape::Circle {
                    center: Vec2D::new(0.0, size * 0.18),
                    radius: size * 0.38,
                    color: ShapeColor::new(0.9, 0.98, 1.0, 0.25),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 舱罩高光
                GeometryShape::Circle {
                    center: Vec2D::new(-size * 0.12, size * 0.28),
                    radius: size * 0.12,
                    color: ShapeColor::new(1.0, 1.0, 1.0, 0.18),
                    fill: true,
                    stroke_width: 1.0,
                },
                // 周围“灯珠”
                GeometryShape::Circle {
                    center: Vec2D::new(-size * 0.55, -size * 0.10),
                    radius: size * 0.08,
                    color: ShapeColor::new(0.15, 0.95, 0.85, 0.85),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(-size * 0.20, -size * 0.25),
                    radius: size * 0.08,
                    color: ShapeColor::new(0.15, 0.95, 0.85, 0.85),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(size * 0.20, -size * 0.25),
                    radius: size * 0.08,
                    color: ShapeColor::new(0.15, 0.95, 0.85, 0.85),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::new(size * 0.55, -size * 0.10),
                    radius: size * 0.08,
                    color: ShapeColor::new(0.15, 0.95, 0.85, 0.85),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: size },
            scale: 1.0,
        },
        BossType::CrossLaser => GeometryBlueprint {
            name: "boss_cross_laser".to_string(),
            shapes: vec![
                // 横向
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-size, -size * 0.2),
                        Vec2D::new(size, -size * 0.2),
                        Vec2D::new(size, size * 0.2),
                        Vec2D::new(-size, size * 0.2),
                    ],
                    color,
                    fill: true,
                    stroke_width: 2.0,
                },
                // 纵向
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-size * 0.2, -size),
                        Vec2D::new(size * 0.2, -size),
                        Vec2D::new(size * 0.2, size),
                        Vec2D::new(-size * 0.2, size),
                    ],
                    color,
                    fill: true,
                    stroke_width: 2.0,
                },
            ],
            collision: CollisionShape::Circle { radius: size * 0.8 },
            scale: 1.0,
        },
        BossType::SpiralShooter => GeometryBlueprint {
            name: "boss_spiral_shooter".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: regular_polygon_vertices(8, size),
                    color,
                    fill: true,
                    stroke_width: 3.0,
                },
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: size * 0.3,
                    color: ShapeColor::new(1.0, 1.0, 1.0, 0.6),
                    fill: true,
                    stroke_width: 2.0,
                },
            ],
            collision: CollisionShape::Circle { radius: size * 0.9 },
            scale: 1.0,
        },
        BossType::SplitCore => GeometryBlueprint {
            name: "boss_split_core".to_string(),
            shapes: vec![
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: size * 0.8,
                    color,
                    fill: true,
                    stroke_width: 3.0,
                },
                // 分裂线
                GeometryShape::Line {
                    start: Vec2D::new(-size * 0.8, 0.0),
                    end: Vec2D::new(size * 0.8, 0.0),
                    color: ShapeColor::new(1.0, 1.0, 1.0, 0.5),
                    stroke_width: 2.0,
                },
                GeometryShape::Line {
                    start: Vec2D::new(0.0, -size * 0.8),
                    end: Vec2D::new(0.0, size * 0.8),
                    color: ShapeColor::new(1.0, 1.0, 1.0, 0.5),
                    stroke_width: 2.0,
                },
            ],
            collision: CollisionShape::Circle { radius: size * 0.8 },
            scale: 1.0,
        },
        BossType::TrackerPrime => GeometryBlueprint {
            name: "boss_tracker_prime".to_string(),
            shapes: vec![GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, -size),
                    Vec2D::new(-size * 0.6, 0.0),
                    Vec2D::new(-size * 0.3, size * 0.8),
                    Vec2D::new(size * 0.3, size * 0.8),
                    Vec2D::new(size * 0.6, 0.0),
                ],
                color,
                fill: true,
                stroke_width: 3.0,
            }],
            collision: CollisionShape::Circle { radius: size * 0.7 },
            scale: 1.0,
        },
        BossType::ChaosEye => GeometryBlueprint {
            name: "boss_chaos_eye".to_string(),
            shapes: vec![
                // 外圈
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: size,
                    color,
                    fill: false,
                    stroke_width: 5.0,
                },
                // 眼睛
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: size * 0.5,
                    color: ShapeColor::new(1.0, 0.0, 0.5, 0.8),
                    fill: true,
                    stroke_width: 2.0,
                },
                // 瞳孔
                GeometryShape::Circle {
                    center: Vec2D::ZERO,
                    radius: size * 0.2,
                    color: ShapeColor::new(0.0, 0.0, 0.0, 1.0),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Circle { radius: size },
            scale: 1.0,
        },
    }
}

/// Boss行为
fn boss_behavior(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut boss_query: Query<(&mut Transform, &mut Boss)>,
    mut boss_state: ResMut<BossState>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut boss) in boss_query.iter_mut() {
        // 进入阶段：Boss从屏幕上方移动到战斗位置
        if !boss.entered {
            let target_y = config.window_height / 2.0 - 120.0;
            if transform.translation.y > target_y {
                transform.translation.y -= 100.0 * delta;
            } else {
                boss.entered = true;
            }
            continue;
        }

        // 更新阶段（根据血量）
        let health_percent = boss.health as f32 / boss.max_health as f32;
        boss.phase = if health_percent > 0.6 {
            1
        } else if health_percent > 0.3 {
            2
        } else {
            3
        };

        // 更新boss_state
        boss_state.current_health = boss.health;

        // 移动逻辑
        boss.move_timer += delta;
        let move_x = (boss.move_timer * 0.5).sin() * 150.0;
        transform.translation.x = move_x;

        // 攻击逻辑
        boss.attack_timer -= delta;
        if boss.attack_timer <= 0.0 {
            // 根据Boss类型和阶段执行攻击
            execute_boss_attack(&mut commands, &config, &transform, &mut boss);

            // 重置攻击计时器（阶段越高攻击越快）
            boss.attack_timer = boss_attack_cooldown(boss.boss_type, boss.phase);
        }
    }
}

fn boss_attack_cooldown(boss_type: BossType, phase: u32) -> f32 {
    // 统一提高“攻击欲望”，并对偏弱的 Boss 类型进一步加速。
    let (p1, p2, p3) = match boss_type {
        // 弹幕型：可以更频繁
        BossType::HexFortress | BossType::SpiralShooter | BossType::CircleGuardian => (1.6, 1.2, 0.9),
        // 压迫型：更高频
        BossType::TrackerPrime | BossType::SplitCore | BossType::CrossLaser => (1.4, 1.05, 0.8),
        // 默认
        _ => (1.8, 1.35, 1.0),
    };

    match phase {
        1 => p1,
        2 => p2,
        _ => p3,
    }
}

/// 执行Boss攻击
fn execute_boss_attack(
    commands: &mut Commands,
    config: &GameConfig,
    transform: &Transform,
    boss: &mut Boss,
) {
    let pos = transform.translation;
    let mut rng = rand::rng();

    match boss.boss_type {
        BossType::DiamondKing => {
            // 扇形弹幕
            let bullet_count = 5 + boss.phase * 2;
            let spread = PI / 3.0;
            for i in 0..bullet_count {
                let angle =
                    -PI / 2.0 + spread * ((i as f32 / (bullet_count - 1) as f32) - 0.5) * 2.0;
                let velocity = Vec2::new(angle.cos() * 200.0, angle.sin() * 200.0);
                spawn_boss_bullet(commands, pos, velocity, 1);
            }
        }
        BossType::HexFortress => {
            // 六向旋转弹幕
            let base_angle = boss.attack_pattern as f32 * PI / 6.0;
            for i in 0..6 {
                let angle = base_angle + (i as f32 / 6.0) * PI * 2.0;
                let velocity = Vec2::new(angle.cos() * 180.0, angle.sin() * 180.0);
                spawn_boss_bullet(commands, pos, velocity, 1);
            }
            boss.attack_pattern = (boss.attack_pattern + 1) % 12;
        }
        BossType::TriangleFighter => {
            // 三连射
            for i in 0..3 {
                let offset_x = (i as f32 - 1.0) * 30.0;
                let bullet_pos = pos + Vec3::new(offset_x, -40.0, 0.0);
                spawn_boss_bullet(commands, bullet_pos, Vec2::new(0.0, -300.0), 2);
            }
        }
        BossType::StarMothership => {
            // 五角星弹幕
            for i in 0..5 {
                let angle = (i as f32 / 5.0) * PI * 2.0 - PI / 2.0;
                let velocity = Vec2::new(angle.cos() * 150.0, angle.sin() * 150.0);
                spawn_boss_bullet(commands, pos, velocity, 1);
            }
        }
        BossType::CircleGuardian => {
            // 环形弹幕
            let bullet_count = 8 + boss.phase * 4;
            for i in 0..bullet_count {
                let angle = (i as f32 / bullet_count as f32) * PI * 2.0;
                let velocity = Vec2::new(angle.cos() * 120.0, angle.sin() * 120.0);
                spawn_boss_bullet(commands, pos, velocity, 1);
            }
        }
        BossType::CrossLaser => {
            // 十字激光
            let directions = [
                Vec2::new(0.0, -1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(-1.0, 0.0),
                Vec2::new(1.0, 0.0),
            ];
            for dir in directions {
                for j in 0..3 {
                    let offset = dir * (j as f32 * 20.0);
                    let bullet_pos = pos + Vec3::new(offset.x, offset.y, 0.0);
                    spawn_boss_bullet(commands, bullet_pos, dir * 250.0, 1);
                }
            }
        }
        BossType::SpiralShooter => {
            // 螺旋弹幕
            let base_angle = boss.attack_pattern as f32 * PI / 8.0;
            for i in 0..4 {
                let angle = base_angle + (i as f32 / 4.0) * PI * 2.0;
                let velocity = Vec2::new(angle.cos() * 160.0, angle.sin() * 160.0);
                spawn_boss_bullet(commands, pos, velocity, 1);
            }
            boss.attack_pattern = (boss.attack_pattern + 1) % 16;
        }
        BossType::SplitCore => {
            // 四向分裂弹
            let angles = [0.0, PI / 2.0, PI, PI * 3.0 / 2.0];
            for angle in angles {
                let velocity = Vec2::new(angle.cos() * 140.0, angle.sin() * 140.0);
                spawn_boss_bullet(commands, pos, velocity, 1);
            }
        }
        BossType::TrackerPrime => {
            // 向下散射
            for _ in 0..(3 + boss.phase) {
                let angle = -PI / 2.0 + rng.random_range(-PI / 4.0..PI / 4.0);
                let speed = rng.random_range(150.0..250.0);
                let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                spawn_boss_bullet(commands, pos, velocity, 1);
            }
        }
        BossType::ChaosEye => {
            // 随机模式
            let pattern = rng.random_range(0..4);
            match pattern {
                0 => {
                    // 环形
                    for i in 0..12 {
                        let angle = (i as f32 / 12.0) * PI * 2.0;
                        let velocity = Vec2::new(angle.cos() * 130.0, angle.sin() * 130.0);
                        spawn_boss_bullet(commands, pos, velocity, 1);
                    }
                }
                1 => {
                    // 直线
                    for i in 0..5 {
                        let offset_x = (i as f32 - 2.0) * 40.0;
                        spawn_boss_bullet(
                            commands,
                            pos + Vec3::new(offset_x, 0.0, 0.0),
                            Vec2::new(0.0, -200.0),
                            1,
                        );
                    }
                }
                2 => {
                    // X形
                    let angles = [PI / 4.0, 3.0 * PI / 4.0, 5.0 * PI / 4.0, 7.0 * PI / 4.0];
                    for angle in angles {
                        let velocity = Vec2::new(angle.cos() * 180.0, angle.sin() * 180.0);
                        spawn_boss_bullet(commands, pos, velocity, 1);
                    }
                }
                _ => {
                    // 随机散射
                    for _ in 0..8 {
                        let angle = rng.random_range(0.0..PI * 2.0);
                        let speed = rng.random_range(100.0..200.0);
                        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                        spawn_boss_bullet(commands, pos, velocity, 1);
                    }
                }
            }
        }
    }
}

/// 生成Boss子弹
fn spawn_boss_bullet(commands: &mut Commands, position: Vec3, velocity: Vec2, damage: i32) {
    let blueprint = GeometryBlueprint {
        name: "boss_bullet".to_string(),
        shapes: vec![GeometryShape::Circle {
            center: Vec2D::ZERO,
            radius: 8.0,
            color: ShapeColor::new(1.0, 0.3, 0.3, 0.9),
            fill: true,
            stroke_width: 2.0,
        }],
        collision: CollisionShape::Circle { radius: 8.0 },
        scale: 1.0,
    };

    let entity = spawn_geometry_entity(commands, &blueprint, position);
    commands.entity(entity).insert((
        BossBullet {
            damage,
            velocity,
            lifetime: 5.0,
        },
        Collider::new(blueprint.collision.clone(), CollisionLayer::EnemyBullet)
            .with_mask(CollisionMask::enemy_bullet_mask()),
    ));
}

/// 更新Boss子弹
fn update_boss_bullets(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(Entity, &mut Transform, &mut BossBullet)>,
) {
    let delta = time.delta_secs();
    let half_w = config.window_width / 2.0 + 50.0;
    let half_h = config.window_height / 2.0 + 50.0;

    for (entity, mut transform, mut bullet) in query.iter_mut() {
        bullet.lifetime -= delta;
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.x += bullet.velocity.x * delta;
        transform.translation.y += bullet.velocity.y * delta;

        // 边界检查
        if transform.translation.x.abs() > half_w || transform.translation.y.abs() > half_h {
            commands.entity(entity).despawn();
        }
    }
}

/// Boss碰撞处理
fn boss_collision_handler(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut game_data: ResMut<GameData>,
    mut boss_state: ResMut<BossState>,
    mut boss_query: Query<&mut Boss>,
    bullets: Query<&Bullet>,
    weapon_bullets: Query<&WeaponBullet>,
    mut pierce: Query<&mut Pierce>,
    mut hit_list: Query<&mut HitList>,
    rockets: Query<&RocketBullet>,
    transforms: Query<&Transform>,
) {
    for event in collision_events.read() {
        // 检查是否涉及Boss
        let boss_entity = if event.layer_a == CollisionLayer::Enemy {
            if boss_query.get(event.entity_a).is_ok() {
                Some(event.entity_a)
            } else {
                None
            }
        } else if event.layer_b == CollisionLayer::Enemy {
            if boss_query.get(event.entity_b).is_ok() {
                Some(event.entity_b)
            } else {
                None
            }
        } else {
            None
        };

        let Some(boss_entity) = boss_entity else {
            continue;
        };

        let other_layer = if boss_query.get(event.entity_a).is_ok() {
            event.layer_b
        } else {
            event.layer_a
        };

        let other_entity = if boss_query.get(event.entity_a).is_ok() {
            event.entity_b
        } else {
            event.entity_a
        };

        if other_layer != CollisionLayer::PlayerBullet {
            continue;
        }

        // 计算伤害
        let damage = if let Ok(bullet) = bullets.get(other_entity) {
            commands.entity(other_entity).despawn();
            bullet.damage
        } else if let Ok(weapon_bullet) = weapon_bullets.get(other_entity) {
            // 检查是否已命中
            if let Ok(mut hits) = hit_list.get_mut(other_entity) {
                if hits.entities.contains(&boss_entity) {
                    continue;
                }
                hits.entities.push(boss_entity);
            }

            // 导弹爆炸
            if weapon_bullet.weapon_type == WeaponType::Rocket {
                if let Ok(rocket) = rockets.get(other_entity) {
                    if let Ok(rocket_tf) = transforms.get(other_entity) {
                        let shard_count = ((rocket.explosion_radius / 4.0) as u32).clamp(10, 28);
                        spawn_rocket_explosion_particles(
                            &mut commands,
                            rocket_tf.translation,
                            shard_count,
                            rocket.speed,
                        );
                    }
                }
                commands.entity(other_entity).despawn();
            } else {
                // 穿透检查
                let mut should_despawn = true;
                if let Ok(mut p) = pierce.get_mut(other_entity) {
                    if p.remaining == u32::MAX {
                        should_despawn = false;
                    } else if p.remaining > 1 {
                        p.remaining -= 1;
                        should_despawn = false;
                    }
                }
                if should_despawn {
                    commands.entity(other_entity).despawn();
                }
            }
            weapon_bullet.damage
        } else {
            continue;
        };

        // 应用伤害
        if let Ok(mut boss) = boss_query.get_mut(boss_entity) {
            boss.health -= damage;
            boss_state.current_health = boss.health;

            if boss.health <= 0 {
                let score = boss.score_value;
                commands.entity(boss_entity).despawn();
                game_data.add_score(score);
                boss_state.active = false;
                boss_state.current_health = 0;
                log::info!("Boss defeated! Score: {}", score);
            }
        }
    }
}

/// 清理Boss
fn despawn_boss(
    mut commands: Commands,
    mut boss_state: ResMut<BossState>,
    query: Query<Entity, With<Boss>>,
    bullet_query: Query<Entity, With<BossBullet>>,
) {
    boss_state.active = false;
    boss_state.current_health = 0;

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in bullet_query.iter() {
        commands.entity(entity).despawn();
    }
}

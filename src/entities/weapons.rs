//! 武器系统
//! 实现各种肉鸽武器：霰弹枪、导弹、激光、自导导弹、闪电链、护身光球、光柱

use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::game::{Collider, CollisionLayer, CollisionMask, GameConfig};
use crate::geometry::{
    spawn_geometry_entity, CollisionShape, GeometryBlueprint, GeometryShape, ShapeColor, Vec2D,
};

/// 武器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    /// S: 霰弹枪 - 从机身发射空心环
    Shotgun,
    /// R: 导弹 - 随机向敌人直线飞去
    Rocket,
    /// L: 激光 - 穿透敌人的细长激光
    Laser,
    /// H: 自导导弹 - 弧线跟踪
    Homing,
    /// B: 闪电链 - 连接最近敌人
    Lightning,
    /// A: 护身光球 - 环绕战机
    Aura,
    /// C: 光柱 - 从屏幕后方推进
    Beam,
}

impl WeaponType {
    /// 获取武器名称
    pub fn name(&self) -> &'static str {
        match self {
            WeaponType::Shotgun => "霰弹枪",
            WeaponType::Rocket => "导弹",
            WeaponType::Laser => "激光",
            WeaponType::Homing => "自导导弹",
            WeaponType::Lightning => "闪电链",
            WeaponType::Aura => "护身光球",
            WeaponType::Beam => "光柱",
        }
    }

    /// 获取武器短代码
    pub fn code(&self) -> char {
        match self {
            WeaponType::Shotgun => 'S',
            WeaponType::Rocket => 'R',
            WeaponType::Laser => 'L',
            WeaponType::Homing => 'H',
            WeaponType::Lightning => 'B',
            WeaponType::Aura => 'A',
            WeaponType::Beam => 'C',
        }
    }

    /// 所有武器类型列表
    pub fn all() -> &'static [WeaponType] {
        &[
            WeaponType::Shotgun,
            WeaponType::Rocket,
            WeaponType::Laser,
            WeaponType::Homing,
            WeaponType::Lightning,
            WeaponType::Aura,
            WeaponType::Beam,
        ]
    }
}

/// 单个武器数据
#[derive(Debug, Clone)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub level: u32,    // 1-5
    pub cooldown: f32, // 冷却时间
    pub timer: f32,    // 当前计时器
}

impl Weapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        Self {
            weapon_type,
            level: 1,
            cooldown: weapon_type.base_cooldown(),
            timer: 0.0,
        }
    }

    /// 是否满级
    pub fn is_max_level(&self) -> bool {
        self.level >= 5
    }

    /// 升级
    pub fn level_up(&mut self) {
        if self.level < 5 {
            self.level += 1;
            // 升级减少冷却时间
            self.cooldown =
                self.weapon_type.base_cooldown() * (1.0 - 0.1 * (self.level - 1) as f32);
        }
    }
}

impl WeaponType {
    /// 基础冷却时间
    pub fn base_cooldown(&self) -> f32 {
        match self {
            // 霰弹枪/跟踪弹：射速与默认子弹接近
            WeaponType::Shotgun => 0.15,
            WeaponType::Rocket => 0.6, // 导弹发射速度翻倍
            WeaponType::Laser => 0.25,
            WeaponType::Homing => 0.15,
            WeaponType::Lightning => 0.5,
            WeaponType::Aura => 0.0, // 被动
            WeaponType::Beam => 2.0,
        }
    }
}

/// 玩家武器库组件
#[derive(Component, Default)]
pub struct WeaponInventory {
    pub weapons: Vec<Weapon>,
    pub has_default_bullet: bool, // 是否还有默认子弹
}

impl WeaponInventory {
    pub fn new() -> Self {
        Self {
            weapons: Vec::new(),
            has_default_bullet: true,
        }
    }

    /// 获取武器数量
    pub fn weapon_count(&self) -> usize {
        self.weapons.len()
    }

    /// 是否已满（5种武器）
    pub fn is_full(&self) -> bool {
        self.weapons.len() >= 5
    }

    /// 所有武器是否满级
    pub fn all_weapons_maxed(&self) -> bool {
        self.weapons.len() >= 5 && self.weapons.iter().all(|w| w.is_max_level())
    }

    /// 获取特定类型的武器
    pub fn get_weapon(&self, weapon_type: WeaponType) -> Option<&Weapon> {
        self.weapons.iter().find(|w| w.weapon_type == weapon_type)
    }

    /// 获取特定类型的武器（可变）
    pub fn get_weapon_mut(&mut self, weapon_type: WeaponType) -> Option<&mut Weapon> {
        self.weapons
            .iter_mut()
            .find(|w| w.weapon_type == weapon_type)
    }

    /// 添加或升级武器
    pub fn add_or_upgrade(&mut self, weapon_type: WeaponType) {
        // 获取“攻击型武器”后，移除默认子弹；仅有护身光球（A）时仍保留基础子弹
        if self.has_default_bullet && weapon_type != WeaponType::Aura {
            self.has_default_bullet = false;
        }

        if let Some(weapon) = self.get_weapon_mut(weapon_type) {
            weapon.level_up();
        } else if self.weapons.len() < 5 {
            self.weapons.push(Weapon::new(weapon_type));
        }
    }

    /// 获取可升级的武器列表（用于升级选择）
    pub fn get_upgradeable_weapons(&self) -> Vec<WeaponType> {
        let mut result = Vec::new();

        // 已有但未满级的武器
        for weapon in &self.weapons {
            if !weapon.is_max_level() {
                result.push(weapon.weapon_type);
            }
        }

        // 还没有但可以获得的新武器
        if !self.is_full() {
            for wt in WeaponType::all() {
                if self.get_weapon(*wt).is_none() {
                    result.push(*wt);
                }
            }
        }

        result
    }
}

// ========== 子弹组件 ==========

/// 基础子弹组件
#[derive(Component)]
pub struct WeaponBullet {
    pub weapon_type: WeaponType,
    pub damage: i32,
    pub velocity: Vec2,
    pub lifetime: f32,
}

/// 穿透（remaining == u32::MAX 表示无限）
#[derive(Component)]
pub struct Pierce {
    pub remaining: u32,
}

/// 记录已命中过的敌人，避免连续帧重复结算伤害
#[derive(Component, Default)]
pub struct HitList {
    pub entities: Vec<Entity>,
}

/// 临时效果的生命周期（只负责自毁，不参与碰撞）
#[derive(Component)]
pub struct EffectLifetime {
    pub remaining: f32,
}

/// 闪烁特效：用于“低频白闪”等
#[derive(Component)]
pub struct BlinkEffect {
    pub remaining: f32,
    pub period: f32,
    pub on_time: f32,
    pub phase: f32,
}

/// 霰弹枪子弹 - 空心环
#[derive(Component)]
pub struct ShotgunPellet {
    pub spread_angle: f32,
}

/// 导弹
#[derive(Component)]
pub struct RocketBullet {
    pub target: Option<Entity>,
    pub initialized: bool,
    pub speed: f32,
    pub explosion_radius: f32,
}

/// 激光
#[derive(Component)]
pub struct LaserBeam {
    pub width: f32,
    pub length: f32,
}

/// 自导导弹
#[derive(Component)]
pub struct HomingMissile {
    pub target: Option<Entity>,
    pub turn_rate: f32,
    pub max_lifetime: f32,
    pub speed: f32,
}

/// 闪电链
#[derive(Component)]
pub struct LightningChain {
    pub bounces_remaining: u32,
    pub hit_entities: Vec<Entity>,
}

/// 护身光球
#[derive(Component)]
pub struct AuraOrb {
    pub orbit_angle: f32,
    pub orbit_speed: f32,
    pub orbit_radius: f32,
}

/// 光柱
#[derive(Component)]
pub struct BeamWave {
    pub progress: f32,
    pub width: f32,
}

/// 导弹爆炸特效：大量小三角碎片（不参与碰撞）
pub fn spawn_rocket_explosion_particles(
    commands: &mut Commands,
    position: Vec3,
    count: u32,
    base_speed: f32,
) {
    let mut rng = rand::rng();
    let count = count.clamp(6, 48);

    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = base_speed * rng.random_range(0.35..0.95);
        let size = rng.random_range(3.5..6.5);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        let blueprint = GeometryBlueprint {
            name: "rocket_explosion_shard".to_string(),
            shapes: vec![GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, size),
                    Vec2D::new(-size * 0.7, -size),
                    Vec2D::new(size * 0.7, -size),
                ],
                color: ShapeColor::new(1.0, 0.72, 0.25, 0.85),
                fill: true,
                stroke_width: 1.0,
            }],
            collision: CollisionShape::Circle { radius: 0.0 },
            scale: 1.0,
        };

        let entity =
            spawn_geometry_entity(commands, &blueprint, position + Vec3::new(0.0, 0.0, 50.0));
        commands.entity(entity).insert(WeaponBullet {
            weapon_type: WeaponType::Rocket,
            damage: 0,
            velocity,
            lifetime: rng.random_range(0.18..0.42),
        });
    }
}

/// 命中火花：用少量半透明黄/橙“丝状线条”表示
pub fn spawn_hit_sparks(commands: &mut Commands, position: Vec3) {
    let mut rng = rand::rng();
    let count = rng.random_range(5..=8);
    let mut shapes = Vec::with_capacity(count);

    for i in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let len = rng.random_range(10.0..16.0);
        let (r, g, b) = if i % 2 == 0 {
            (1.0, 0.85, 0.2) // yellow
        } else {
            (1.0, 0.55, 0.15) // orange
        };
        shapes.push(GeometryShape::Line {
            start: Vec2D::ZERO,
            end: Vec2D::new(angle.cos() * len, angle.sin() * len),
            color: ShapeColor::new(r, g, b, 0.55),
            stroke_width: 1.3,
        });
    }

    let blueprint = GeometryBlueprint {
        name: "hit_sparks".to_string(),
        shapes,
        collision: CollisionShape::Circle { radius: 0.0 },
        scale: 1.0,
    };

    let entity = spawn_geometry_entity(commands, &blueprint, position + Vec3::new(0.0, 0.0, 60.0));
    commands.entity(entity).insert(EffectLifetime { remaining: 0.14 });
}

/// Boss 受击闪光：低频白闪，透明度很低
pub fn spawn_boss_hit_flash(commands: &mut Commands, position: Vec3) {
    let blueprint = GeometryBlueprint {
        name: "boss_hit_flash".to_string(),
        shapes: vec![GeometryShape::Circle {
            center: Vec2D::ZERO,
            radius: 22.0,
            color: ShapeColor::new(1.0, 1.0, 1.0, 0.10),
            fill: true,
            stroke_width: 1.0,
        }],
        collision: CollisionShape::Circle { radius: 0.0 },
        scale: 1.0,
    };

    let entity = spawn_geometry_entity(commands, &blueprint, position + Vec3::new(0.0, 0.0, 55.0));
    commands.entity(entity).insert(BlinkEffect {
        remaining: 0.75,
        period: 0.35,
        on_time: 0.16,
        phase: 0.0,
    });
}

/// 闪电链：一次性施放请求（由系统解析并结算伤害）
#[derive(Component)]
pub struct LightningCast {
    pub jumps: u32,
    pub range: f32,
    pub damage: i32,
}

// ========== 生成武器子弹 ==========

/// 生成霰弹枪子弹
pub fn spawn_shotgun_pellets(commands: &mut Commands, position: Vec3, level: u32, speed: f32) {
    // 霰弹枪：符合设定（lv1:2，lv5:10），并且角度更集中
    let pellet_count = 2 + (level - 1) * 2; // lv1: 2, lv5: 10
    let spread_angle = PI / 18.0; // 总扩散约 20°（更像“散射”而非“扇形扫射”）

    for i in 0..pellet_count {
        let t = if pellet_count <= 1 {
            0.0
        } else {
            (i as f32) / (pellet_count as f32 - 1.0)
        };
        let angle = PI / 2.0 + spread_angle * (t - 0.5) * 2.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        let blueprint = GeometryBlueprint {
            name: "shotgun_pellet".to_string(),
            shapes: vec![GeometryShape::Circle {
                center: Vec2D::ZERO,
                radius: 6.0,
                color: ShapeColor::new(0.8, 0.9, 1.0, 0.7),
                fill: false,
                stroke_width: 2.0,
            }],
            collision: CollisionShape::Circle { radius: 6.0 },
            scale: 1.0,
        };

        let entity = spawn_geometry_entity(commands, &blueprint, position);
        commands.entity(entity).insert((
            WeaponBullet {
                weapon_type: WeaponType::Shotgun,
                damage: 1,
                velocity,
                lifetime: 1.6,
            },
            ShotgunPellet {
                spread_angle: angle,
            },
            Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet)
                .with_mask(CollisionMask::player_bullet_mask()),
        ));
    }
}

/// 生成导弹
pub fn spawn_rocket(
    commands: &mut Commands,
    position: Vec3,
    level: u32,
    target: Option<Entity>,
    speed: f32,
) {
    let rocket_count = level; // lv1: 1, lv5: 5
    let base_speed = speed * (1.0 + 0.1 * (level - 1) as f32);

    for i in 0..rocket_count {
        let offset_x = if rocket_count > 1 {
            ((i as f32) - (rocket_count as f32 - 1.0) / 2.0) * 15.0
        } else {
            0.0
        };

        // 初始先向上飞，update_rocket_bullets 会在第一帧初始化方向（如果有目标则直线朝向目标）
        let velocity = Vec2::new(0.0, base_speed);

        let blueprint = GeometryBlueprint::raiden_missile();

        let pos = position + Vec3::new(offset_x, 0.0, 0.0);
        let entity = spawn_geometry_entity(commands, &blueprint, pos);
        commands.entity(entity).insert((
            WeaponBullet {
                weapon_type: WeaponType::Rocket,
                damage: 2 + level as i32,
                velocity,
                lifetime: 5.0,
            },
            RocketBullet {
                target,
                initialized: false,
                speed: base_speed,
                explosion_radius: 30.0 + 5.0 * level as f32,
            },
            Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet)
                .with_mask(CollisionMask::player_bullet_mask()),
        ));
    }
}

/// 生成激光
pub fn spawn_laser(commands: &mut Commands, position: Vec3, level: u32) {
    // 激光（L）：发射可移动的“长条”穿透弹，而不是瞬间删除的光束
    let laser_count = 1 + (level - 1) / 2; // lv1: 1, lv3: 2, lv5: 3
    let spacing = 25.0;
    // 视觉上更短，减少遮挡
    let length = 150.0;
    // 视觉上更细，避免遮挡视线；碰撞宽度也同步收窄
    let width = 4.0 + level as f32 * 0.7; // lv1: 4.7, lv5: 7.5
    let speed = 600.0; // 加快速度

    for i in 0..laser_count {
        let offset_x = if laser_count > 1 {
            ((i as f32) - (laser_count as f32 - 1.0) / 2.0) * spacing
        } else {
            0.0
        };

        // 两层：外层淡光（更透明）+ 内层亮芯（更细）
        let core_width = (width * 0.40).max(1.8);
        let blueprint = GeometryBlueprint {
            name: "laser".to_string(),
            shapes: vec![
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-width * 0.5, -length * 0.5),
                        Vec2D::new(width * 0.5, -length * 0.5),
                        Vec2D::new(width * 0.5, length * 0.5),
                        Vec2D::new(-width * 0.5, length * 0.5),
                    ],
                    color: ShapeColor::new(0.25, 1.0, 0.45, 0.12),
                    fill: true,
                    stroke_width: 1.0,
                },
                GeometryShape::Polygon {
                    vertices: vec![
                        Vec2D::new(-core_width * 0.5, -length * 0.5),
                        Vec2D::new(core_width * 0.5, -length * 0.5),
                        Vec2D::new(core_width * 0.5, length * 0.5),
                        Vec2D::new(-core_width * 0.5, length * 0.5),
                    ],
                    color: ShapeColor::new(0.75, 1.0, 0.85, 0.18),
                    fill: true,
                    stroke_width: 1.0,
                },
            ],
            collision: CollisionShape::Rectangle {
                width,
                height: length,
            },
            scale: 1.0,
        };

        let pos = position + Vec3::new(offset_x, 40.0 + length * 0.5, 0.0);
        let entity = spawn_geometry_entity(commands, &blueprint, pos);
        commands.entity(entity).insert((
            WeaponBullet {
                weapon_type: WeaponType::Laser,
                damage: 2 + level as i32, // 提升伤害
                velocity: Vec2::new(0.0, speed),
                lifetime: 2.0,
            },
            Pierce {
                remaining: u32::MAX,
            },
            HitList::default(),
            LaserBeam { width, length },
            Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet)
                .with_mask(CollisionMask::player_bullet_mask()),
        ));
    }
}

/// 生成自导导弹
pub fn spawn_homing_missile(
    commands: &mut Commands,
    position: Vec3,
    level: u32,
    target: Option<Entity>,
    speed: f32,
) {
    let missile_count = level; // lv1: 1, lv5: 5

    for i in 0..missile_count {
        let offset_x = if missile_count > 1 {
            ((i as f32) - (missile_count as f32 - 1.0) / 2.0) * 12.0
        } else {
            0.0
        };

        let velocity = Vec2::new(0.0, speed);

        let blueprint = GeometryBlueprint {
            name: "homing_missile".to_string(),
            shapes: vec![GeometryShape::Polygon {
                vertices: vec![
                    Vec2D::new(0.0, 6.0),
                    Vec2D::new(-3.0, -3.0),
                    Vec2D::new(3.0, -3.0),
                ],
                color: ShapeColor::new(0.5, 0.8, 1.0, 0.9),
                fill: true,
                stroke_width: 1.0,
            }],
            collision: CollisionShape::Circle { radius: 4.0 },
            scale: 1.0,
        };

        let pos = position + Vec3::new(offset_x, 0.0, 0.0);
        let entity = spawn_geometry_entity(commands, &blueprint, pos);
        commands.entity(entity).insert((
            WeaponBullet {
                weapon_type: WeaponType::Homing,
                damage: 2,
                velocity,
                lifetime: 4.0,
            },
            HomingMissile {
                target,
                turn_rate: 3.0 + 0.5 * level as f32,
                max_lifetime: 4.0,
                speed,
            },
            Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet)
                .with_mask(CollisionMask::player_bullet_mask()),
        ));
    }
}

/// 生成闪电链
pub fn spawn_lightning(
    commands: &mut Commands,
    position: Vec3,
    level: u32,
    _first_target: Option<Entity>,
) {
    let entity = commands
        .spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .id();

    commands.entity(entity).insert(LightningCast {
        jumps: 3 + level, // lv1:4, lv5:8（包含第一跳）
        range: (280.0 + 50.0 * level as f32).min(600.0),
        damage: 3 + level as i32, // 提升伤害
    });
}

/// 生成护身光球
pub fn spawn_aura_orbs(commands: &mut Commands, player_entity: Entity, level: u32) {
    let orb_count = 2 + level; // lv1: 3, lv5: 7
    let orbit_radius = 40.0;

    for i in 0..orb_count {
        let angle = (i as f32 / orb_count as f32) * 2.0 * PI;

        let blueprint = GeometryBlueprint {
            name: "aura_orb".to_string(),
            shapes: vec![GeometryShape::Circle {
                center: Vec2D::ZERO,
                radius: 8.0,
                color: ShapeColor::new(0.9, 0.9, 0.3, 0.6),
                fill: true,
                stroke_width: 1.0,
            }],
            collision: CollisionShape::Circle { radius: 8.0 },
            scale: 1.0,
        };

        let entity = spawn_geometry_entity(commands, &blueprint, Vec3::ZERO);
        commands.entity(entity).insert((
            WeaponBullet {
                weapon_type: WeaponType::Aura,
                damage: 1,
                velocity: Vec2::ZERO,
                lifetime: f32::MAX, // 不会自动消失
            },
            Pierce {
                remaining: u32::MAX,
            },
            HitList::default(),
            AuraOrb {
                orbit_angle: angle,
                orbit_speed: 2.0,
                orbit_radius,
            },
            AuraOwner(player_entity),
            Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet).with_mask(
                CollisionMask {
                    // 允许与敌人子弹碰撞，实现“抵消子弹”
                    enemy_bullet: true,
                    ..CollisionMask::player_bullet_mask()
                },
            ),
        ));
    }
}

/// 护身光球所属玩家
#[derive(Component)]
pub struct AuraOwner(pub Entity);

/// 生成光柱
pub fn spawn_beam_wave(commands: &mut Commands, config: &GameConfig, position: Vec3, level: u32) {
    // 能量波（C）：更像格斗游戏“气动波”
    // - 颜色更克制（偏蓝白半透明），避免“彩虹”刺眼
    // - 两端更窄（taper），中间更厚
    // - 可穿透，从玩家位置向上飞出
    //
    // 目标宽度：半屏幕（≈ window_width / 2），半圆宽度为 2*radius，因此 radius = window_width / 4。
    let radius = config.window_width * 0.25;
    let base_thickness = 10.0 + 1.2 * level as f32; // 中段厚度，整体更细
    let speed = config.bullet_speed * 0.9;

    let segments = 44;
    let mut outer: Vec<Vec2D> = Vec::with_capacity(segments + 1);
    let mut inner: Vec<Vec2D> = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32; // 0..1
        let angle = t * PI; // 0..PI

        // taper：两端更窄，中间更厚
        let edge = (t - 0.5).abs() * 2.0; // 0(中) -> 1(端)
        let mid = (1.0 - edge).clamp(0.0, 1.0);
        let thickness = base_thickness * (0.35 + 0.65 * mid.powf(1.4));

        let outer_r = radius + thickness * 0.5;
        let inner_r = (radius - thickness * 0.5).max(0.0);
        outer.push(Vec2D::new(outer_r * angle.cos(), outer_r * angle.sin()));
        inner.push(Vec2D::new(inner_r * angle.cos(), inner_r * angle.sin()));
    }

    // 闭合带状多边形：外弧 + 反向内弧
    let mut band = outer;
    for v in inner.into_iter().rev() {
        band.push(v);
    }

    let blueprint = GeometryBlueprint {
        name: "aero_wave".to_string(),
        shapes: vec![
            // 外层柔光（更透明）
            GeometryShape::Polygon {
                vertices: band,
                color: ShapeColor::new(0.35, 0.75, 1.0, 0.16),
                fill: true,
                stroke_width: 1.0,
            },
            // 内层亮芯（更细更淡）
            GeometryShape::Arc {
                center: Vec2D::ZERO,
                radius,
                start_angle: 0.0,
                end_angle: PI,
                color: ShapeColor::new(0.85, 0.95, 1.0, 0.22),
                stroke_width: (base_thickness * 0.35).max(2.5),
            },
            // 轻微轮廓线，提升辨识度但不刺眼
            GeometryShape::Arc {
                center: Vec2D::ZERO,
                radius: radius + base_thickness * 0.15,
                start_angle: 0.0,
                end_angle: PI,
                color: ShapeColor::new(0.55, 0.9, 1.0, 0.18),
                stroke_width: 2.0,
            },
        ],
        collision: CollisionShape::Circle {
            radius: radius + base_thickness * 0.6,
        },
        scale: 1.0,
    };

    let entity = spawn_geometry_entity(commands, &blueprint, position + Vec3::new(0.0, 40.0, 0.0));
    commands.entity(entity).insert((
        WeaponBullet {
            weapon_type: WeaponType::Beam,
            damage: 4 + level as i32 * 2, // 伤害翻倍
            velocity: Vec2::new(0.0, speed),
            lifetime: 2.0,
        },
        Pierce {
            remaining: u32::MAX,
        },
        HitList::default(),
        BeamWave {
            progress: 0.0,
            width: radius,
        },
        Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet)
            .with_mask(CollisionMask::player_bullet_mask()),
    ));
}

/// 生成默认子弹（小圆点）
pub fn spawn_default_bullet(commands: &mut Commands, position: Vec3, speed: f32) {
    let blueprint = GeometryBlueprint::default_bullet();
    let entity = spawn_geometry_entity(commands, &blueprint, position);

    commands.entity(entity).insert((
        WeaponBullet {
            weapon_type: WeaponType::Shotgun, // 默认类型
            damage: 1,
            velocity: Vec2::new(0.0, speed),
            lifetime: 3.0,
        },
        Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet)
            .with_mask(CollisionMask::player_bullet_mask()),
    ));
}

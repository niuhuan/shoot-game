//! 敌人系统

use bevy::prelude::*;
use rand::Rng;

use crate::entities::{
    spawn_rocket_explosion_particles, BossState, Bullet, HitList, Pierce, RocketBullet,
    WeaponBullet, WeaponType,
};
use crate::game::{
    not_upgrading, Collider, CollisionEvent, CollisionLayer, CollisionMask, GameConfig, GameData,
    GameState, Scrollable,
};
use crate::geometry::{spawn_geometry_entity, GeometryBlueprint};

/// 敌人插件
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer::default())
            .add_systems(OnEnter(GameState::Menu), despawn_all_enemies)
            .add_systems(OnEnter(GameState::GameOver), despawn_all_enemies)
            .add_systems(OnEnter(GameState::Recharge), despawn_all_enemies)
            .add_systems(
                Update,
                (
                    spawn_enemies,
                    enemy_movement,
                    enemy_shooting,
                    enemy_collision_handler,
                    despawn_offscreen_enemies,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(not_upgrading),
            );
    }
}

/// 敌人组件
#[derive(Component)]
pub struct Enemy {
    pub health: i32,
    pub max_health: i32,
    pub score_value: u32,
    pub enemy_type: EnemyType,
    pub shoot_timer: f32,
    pub shoot_interval: f32,
}

/// 敌人类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    /// 基础菱形敌人
    Diamond,
    /// 六边形敌人（更强）
    Hexagon,
    /// 快速小型敌人
    Small,
    /// 精英：侦察机（慢、体积大、弹多）
    EliteScout,
    /// 精英：炮艇（慢、体积大、弹多）
    EliteGunship,
    /// 精英：守卫机（慢、体积大、弹多）
    EliteGuard,
}

/// 敌人移动模式
#[derive(Component)]
pub enum EnemyMovement {
    /// 直线向下
    Straight { speed: f32 },
    /// 正弦波移动
    Sine {
        speed: f32,
        amplitude: f32,
        frequency: f32,
        time: f32,
    },
    /// 追踪玩家
    Homing { speed: f32, turn_speed: f32 },
    /// 停留并射击
    Stationary { target_y: f32 },
}

/// 敌人生成计时器
#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: f32,
    pub interval: f32,
    pub difficulty: f32,
}

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self {
            timer: 0.0,
            interval: 1.5,
            difficulty: 1.0,
        }
    }
}

/// 生成敌人
fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game_data: Res<GameData>,
    boss_state: Res<BossState>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
) {
    spawn_timer.timer += time.delta_secs();

    // 根据玩家等级计算难度系数
    let level = game_data.player_level;
    let difficulty = 1.0 + (level as f32 - 1.0) * 0.3; // 每级增加30%难度

    // 生成间隔随等级降低（最低0.3秒）
    spawn_timer.interval = (config.enemy_spawn_interval / difficulty).max(0.3);
    // Boss 战期间：只生成“一半频率的小兵”，不生成精英
    if boss_state.active {
        spawn_timer.interval *= 2.0;
    }

    if spawn_timer.timer >= spawn_timer.interval {
        spawn_timer.timer = 0.0;

        let mut rng = rand::rng();

        let enemy_type = if boss_state.active {
            // Boss 期间：只出小兵（不出精英，不额外刷多波）
            if rng.random_bool(0.7) {
                EnemyType::Small
            } else {
                EnemyType::Diamond
            }
        } else {
            // 非 Boss 期间：有低概率刷精英
            let elite_chance = (0.02 + level as f64 * 0.003).min(0.08); // 2% -> 8%
            if level >= 3 && rng.random_bool(elite_chance) {
                match rng.random_range(0..3) {
                    0 => EnemyType::EliteScout,
                    1 => EnemyType::EliteGunship,
                    _ => EnemyType::EliteGuard,
                }
            } else {
                // 高等级时更容易出现强敌
                let hexagon_chance = (0.2 + level as f64 * 0.02).min(0.5); // 最高50%
                let small_chance = 0.15;
                if rng.random_bool(1.0 - hexagon_chance - small_chance) {
                    EnemyType::Diamond
                } else if rng.random_bool(hexagon_chance / (hexagon_chance + small_chance)) {
                    EnemyType::Hexagon
                } else {
                    EnemyType::Small
                }
            }
        };

        // 随机X位置
        let x =
            rng.random_range(-config.window_width / 2.0 + 50.0..config.window_width / 2.0 - 50.0);
        let y = config.window_height / 2.0 + 50.0;

        // 传递难度系数
        spawn_enemy_with_difficulty(
            &mut commands,
            &config,
            Vec3::new(x, y, 5.0),
            enemy_type,
            difficulty,
        );

        // 高等级时可能同时生成多个敌人（Boss 期间不额外生成）
        if !boss_state.active && level >= 5 && rng.random_bool(0.3) {
            let x2 = rng
                .random_range(-config.window_width / 2.0 + 50.0..config.window_width / 2.0 - 50.0);
            spawn_enemy_with_difficulty(
                &mut commands,
                &config,
                Vec3::new(x2, y + 50.0, 5.0),
                EnemyType::Diamond,
                difficulty,
            );
        }
        if !boss_state.active && level >= 10 && rng.random_bool(0.2) {
            let x3 = rng
                .random_range(-config.window_width / 2.0 + 50.0..config.window_width / 2.0 - 50.0);
            spawn_enemy_with_difficulty(
                &mut commands,
                &config,
                Vec3::new(x3, y + 100.0, 5.0),
                EnemyType::Small,
                difficulty,
            );
        }
    }
}

/// 生成单个敌人
pub fn spawn_enemy(
    commands: &mut Commands,
    config: &GameConfig,
    position: Vec3,
    enemy_type: EnemyType,
) {
    spawn_enemy_with_difficulty(commands, config, position, enemy_type, 1.0);
}

/// 生成带难度系数的敌人
pub fn spawn_enemy_with_difficulty(
    commands: &mut Commands,
    config: &GameConfig,
    position: Vec3,
    enemy_type: EnemyType,
    difficulty: f32,
) {
    let mut rng = rand::rng();

    // 基础属性
    let (blueprint, base_health, base_score, base_shoot_interval) = match enemy_type {
        EnemyType::Diamond => {
            // 轻型机：菱形与无人机混合
            let bp = if rng.random_bool(0.35) {
                GeometryBlueprint::default_enemy()
            } else {
                GeometryBlueprint::raiden_enemy_drone_small()
            };
            (bp, 2, 100, 2.0)
        }
        EnemyType::Hexagon => {
            // 重型机：六边形与装甲机混合
            let bp = if rng.random_bool(0.5) {
                GeometryBlueprint::hexagon_enemy()
            } else {
                GeometryBlueprint::raiden_enemy_tank()
            };
            (bp, 5, 300, 1.5)
        }
        EnemyType::Small => (GeometryBlueprint::raiden_enemy_drone_small(), 1, 50, 2.6),
        // 精英射速更慢：攻击间隔约为原来的 2 倍
        EnemyType::EliteScout => (GeometryBlueprint::elite_scout(), 10, 900, 2.8),
        EnemyType::EliteGunship => (GeometryBlueprint::elite_gunship(), 14, 1200, 3.2),
        EnemyType::EliteGuard => (GeometryBlueprint::elite_guard(), 18, 1500, 3.6),
    };

    // 根据难度调整属性
    let health = ((base_health as f32) * difficulty).ceil() as i32;
    let score = ((base_score as f32) * difficulty) as u32;
    let shoot_interval = (base_shoot_interval / difficulty).max(0.5);

    let entity = spawn_geometry_entity(commands, &blueprint, position);

    // 速度也随难度增加；精英更慢一点
    let mut speed_multiplier = 1.0 + (difficulty - 1.0) * 0.5;
    if matches!(
        enemy_type,
        EnemyType::EliteScout | EnemyType::EliteGunship | EnemyType::EliteGuard
    ) {
        // 精英：显著更慢（约为之前的 0.2 倍）
        speed_multiplier *= 0.2;
    }

    // 随机移动模式
    let movement = match rng.random_range(0..3) {
        0 => EnemyMovement::Straight {
            speed: config.enemy_base_speed * rng.random_range(0.8..1.2) * speed_multiplier,
        },
        1 => EnemyMovement::Sine {
            speed: config.enemy_base_speed * 0.8 * speed_multiplier,
            amplitude: rng.random_range(50.0..150.0),
            frequency: rng.random_range(1.0..3.0),
            time: 0.0,
        },
        _ => EnemyMovement::Straight {
            speed: config.enemy_base_speed * speed_multiplier,
        },
    };

    commands.entity(entity).insert((
        Enemy {
            health,
            max_health: health,
            score_value: score,
            enemy_type,
            shoot_timer: rng.random_range(0.0..shoot_interval),
            shoot_interval,
        },
        movement,
        Collider::new(blueprint.collision.clone(), CollisionLayer::Enemy)
            .with_mask(CollisionMask::enemy_mask()),
        Scrollable::default(),
    ));
}

/// 敌人移动
fn enemy_movement(time: Res<Time>, mut query: Query<(&mut Transform, &mut EnemyMovement)>) {
    for (mut transform, mut movement) in query.iter_mut() {
        match movement.as_mut() {
            EnemyMovement::Straight { speed } => {
                transform.translation.y -= *speed * time.delta_secs();
            }
            EnemyMovement::Sine {
                speed,
                amplitude,
                frequency,
                time: move_time,
            } => {
                *move_time += time.delta_secs();
                transform.translation.y -= *speed * time.delta_secs();
                transform.translation.x +=
                    (*move_time * *frequency).cos() * *amplitude * time.delta_secs();
            }
            EnemyMovement::Homing {
                speed,
                turn_speed: _,
            } => {
                // 简化的追踪：直接向下
                transform.translation.y -= *speed * time.delta_secs();
            }
            EnemyMovement::Stationary { target_y } => {
                if transform.translation.y > *target_y {
                    transform.translation.y -= 100.0 * time.delta_secs();
                }
            }
        }
    }
}

/// 敌人射击
fn enemy_shooting(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(&Transform, &mut Enemy)>,
) {
    for (transform, mut enemy) in query.iter_mut() {
        enemy.shoot_timer -= time.delta_secs();

        if enemy.shoot_timer <= 0.0 {
            enemy.shoot_timer = enemy.shoot_interval;

            let bullet_pos = transform.translation + Vec3::new(0.0, -24.0, 0.0);
            let mut rng = rand::rng();

            // 精英：慢速但弹幕更密
            match enemy.enemy_type {
                EnemyType::EliteScout => {
                    // 3-way 扇形 + 1 个中心加速弹
                    let base = -std::f32::consts::FRAC_PI_2;
                    for (i, speed_mul, style) in [
                        (-1, 0.55, super::bullet::EnemyBulletStyle::Ring),
                        (0, 0.65, super::bullet::EnemyBulletStyle::Shard),
                        (1, 0.55, super::bullet::EnemyBulletStyle::Ring),
                        (0, 0.85, super::bullet::EnemyBulletStyle::Needle),
                    ] {
                        let angle = base + (i as f32) * 0.18;
                        let v = Vec2::new(angle.cos(), angle.sin()) * (config.bullet_speed * speed_mul);
                        super::bullet::spawn_enemy_bullet(&mut commands, bullet_pos, v, style);
                    }
                }
                EnemyType::EliteGunship => {
                    // 5-way 扇形（更宽）+ 两侧“压制弹”
                    let base = -std::f32::consts::FRAC_PI_2;
                    let count = 5;
                    let spread = 0.75;
                    for i in 0..count {
                        let t = if count == 1 {
                            0.0
                        } else {
                            (i as f32 / (count - 1) as f32) * 2.0 - 1.0
                        };
                        let angle = base + t * spread * 0.5;
                        let v = Vec2::new(angle.cos(), angle.sin()) * (config.bullet_speed * 0.6);
                        super::bullet::spawn_enemy_bullet(
                            &mut commands,
                            bullet_pos,
                            v,
                            super::bullet::EnemyBulletStyle::Shard,
                        );
                    }
                    for side in [-1.0, 1.0] {
                        let v = Vec2::new(side * 0.25, -1.0).normalize() * (config.bullet_speed * 0.5);
                        super::bullet::spawn_enemy_bullet(
                            &mut commands,
                            bullet_pos + Vec3::new(side * 14.0, 0.0, 0.0),
                            v,
                            super::bullet::EnemyBulletStyle::Ring,
                        );
                    }
                }
                EnemyType::EliteGuard => {
                    // 环形小弹（朝下半圆更密）
                    let count = 10;
                    for i in 0..count {
                        let t = i as f32 / (count - 1) as f32; // 0..1
                        let angle = std::f32::consts::PI * (0.15 + 0.7 * t) + std::f32::consts::PI; // mostly downward
                        let v = Vec2::new(angle.cos(), angle.sin()) * (config.bullet_speed * 0.42);
                        super::bullet::spawn_enemy_bullet(
                            &mut commands,
                            bullet_pos,
                            v,
                            super::bullet::EnemyBulletStyle::Ring,
                        );
                    }
                }
                _ => {
                    // 普通敌人：单发为主，类型决定样式
                    let style = if enemy.enemy_type == EnemyType::Hexagon {
                        super::bullet::EnemyBulletStyle::Ring
                    } else if enemy.enemy_type == EnemyType::Small {
                        super::bullet::EnemyBulletStyle::Needle
                    } else if rng.random_bool(0.25) {
                        super::bullet::EnemyBulletStyle::Needle
                    } else {
                        super::bullet::EnemyBulletStyle::Shard
                    };
                    super::bullet::spawn_enemy_bullet(
                        &mut commands,
                        bullet_pos,
                        Vec2::new(0.0, -config.bullet_speed * 0.6),
                        style,
                    );
                }
            }
        }
    }
}

/// 处理敌人碰撞
fn enemy_collision_handler(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut game_data: ResMut<GameData>,
    boss_state: Res<BossState>,
    mut enemy_set: ParamSet<(Query<(Entity, &Transform), With<Enemy>>, Query<&mut Enemy>)>,
    enemy_marker: Query<(), With<Enemy>>,
    bullets: Query<&Bullet>,
    weapon_bullets: Query<&WeaponBullet>,
    mut pierce: Query<&mut Pierce>,
    mut hit_list: Query<&mut HitList>,
    rockets: Query<&RocketBullet>,
    transforms: Query<&Transform>,
) {
    for event in collision_events.read() {
        // 检查是否涉及敌人
        let enemy_entity = if event.layer_a == CollisionLayer::Enemy {
            Some(event.entity_a)
        } else if event.layer_b == CollisionLayer::Enemy {
            Some(event.entity_b)
        } else {
            None
        };

        let Some(enemy_entity) = enemy_entity else {
            continue;
        };

        // Boss 也使用 CollisionLayer::Enemy，但这里仅处理普通敌人（避免提前把子弹 despawn 导致 Boss 不掉血）
        if enemy_marker.get(enemy_entity).is_err() {
            continue;
        }

        // 确定另一个实体的类型
        let other_layer = if event.layer_a == CollisionLayer::Enemy {
            event.layer_b
        } else {
            event.layer_a
        };

        let other_entity = if event.layer_a == CollisionLayer::Enemy {
            event.entity_b
        } else {
            event.entity_a
        };

        match other_layer {
            CollisionLayer::PlayerBullet => {
                // 确定伤害与子弹类型
                if let Ok(bullet) = bullets.get(other_entity) {
                    // 命中火花
                    let spark_pos = transforms
                        .get(other_entity)
                        .map(|t| t.translation)
                        .unwrap_or_else(|_| transforms.get(enemy_entity).map(|t| t.translation).unwrap_or_default());
                    crate::entities::spawn_hit_sparks(&mut commands, spark_pos);

                    apply_direct_damage(
                        &mut commands,
                        &mut game_data,
                        boss_state.active,
                        &mut enemy_set.p1(),
                        enemy_entity,
                        bullet.damage,
                        &transforms,
                    );
                    commands.entity(other_entity).despawn();
                    continue;
                }

                let Ok(weapon_bullet) = weapon_bullets.get(other_entity) else {
                    continue;
                };

                // 避免穿透/持续类武器在连续帧对同一敌人反复结算
                if let Ok(mut hits) = hit_list.get_mut(other_entity) {
                    if hits.entities.contains(&enemy_entity) {
                        continue;
                    }
                    hits.entities.push(enemy_entity);
                }

                // 导弹：命中立刻爆炸（AOE），不走单体伤害
                if weapon_bullet.weapon_type == WeaponType::Rocket {
                    let Ok(rocket) = rockets.get(other_entity) else {
                        continue;
                    };
                    let Ok(rocket_tf) = transforms.get(other_entity) else {
                        continue;
                    };

                    let targets: Vec<Entity> = {
                        let center2 = rocket_tf.translation.truncate();
                        let r2 = rocket.explosion_radius * rocket.explosion_radius;
                        enemy_set
                            .p0()
                            .iter()
                            .filter_map(|(e, t)| {
                                (t.translation.truncate().distance_squared(center2) <= r2)
                                    .then_some(e)
                            })
                            .collect()
                    };
                    for hit_enemy in targets {
                        apply_direct_damage(
                            &mut commands,
                            &mut game_data,
                            boss_state.active,
                            &mut enemy_set.p1(),
                            hit_enemy,
                            weapon_bullet.damage,
                            &transforms,
                        );
                    }
                    let shard_count = ((rocket.explosion_radius / 4.0) as u32).clamp(10, 28);
                    spawn_rocket_explosion_particles(
                        &mut commands,
                        rocket_tf.translation,
                        shard_count,
                        rocket.speed,
                    );
                    commands.entity(other_entity).despawn();
                    continue;
                }

                // 其它武器：结算单体伤害
                // 命中火花：长条/能量波这类“面积武器”用敌人位置更符合观感
                let enemy_pos = transforms
                    .get(enemy_entity)
                    .map(|t| t.translation)
                    .unwrap_or_default();
                let bullet_pos = transforms
                    .get(other_entity)
                    .map(|t| t.translation)
                    .unwrap_or(enemy_pos);
                let spark_pos = match weapon_bullet.weapon_type {
                    WeaponType::Laser | WeaponType::Beam => enemy_pos,
                    _ => bullet_pos,
                };
                crate::entities::spawn_hit_sparks(&mut commands, spark_pos);
                apply_direct_damage(
                    &mut commands,
                    &mut game_data,
                    boss_state.active,
                    &mut enemy_set.p1(),
                    enemy_entity,
                    weapon_bullet.damage,
                    &transforms,
                );

                // 是否需要销毁子弹（穿透则保留）
                let mut should_despawn = true;
                if let Ok(mut p) = pierce.get_mut(other_entity) {
                    if p.remaining == u32::MAX {
                        should_despawn = false;
                    } else if p.remaining > 1 {
                        p.remaining -= 1;
                        should_despawn = false;
                    } else {
                        p.remaining = 0;
                        should_despawn = true;
                    }
                }

                if should_despawn {
                    commands.entity(other_entity).despawn();
                }
            }
            _ => {}
        }
    }
}

fn apply_direct_damage(
    commands: &mut Commands,
    game_data: &mut ResMut<GameData>,
    boss_active: bool,
    enemies: &mut Query<&mut Enemy>,
    enemy_entity: Entity,
    damage: i32,
    transforms: &Query<&Transform>,
) {
    let Ok(mut enemy) = enemies.get_mut(enemy_entity) else {
        return;
    };

    enemy.health -= damage;
    if enemy.health <= 0 {
        let score = enemy.score_value;
        let position = transforms.get(enemy_entity).map(|t| t.translation).unwrap_or_default();
        commands.entity(enemy_entity).despawn();
        if boss_active {
            // Boss 战期间的小兵：不增长经验、无掉落
            game_data.add_score_only(score);
        } else {
            game_data.add_score(score);
            // 2% 概率掉落金币
            let mut rng = rand::rng();
            if rng.random_bool(0.02) {
                use crate::entities::shield::{spawn_power_up, PowerUpType};
                spawn_power_up(commands, position, PowerUpType::Coin);
            }
        }
    }
}

/// 销毁屏幕外的敌人
fn despawn_offscreen_enemies(
    mut commands: Commands,
    config: Res<GameConfig>,
    query: Query<(Entity, &Transform), With<Enemy>>,
) {
    // 敌人超出屏幕下方200像素后消失
    let despawn_y = -config.window_height / 2.0 - 100.0;

    for (entity, transform) in query.iter() {
        if transform.translation.y < despawn_y {
            commands.entity(entity).despawn();
        }
    }
}

/// 销毁所有敌人
fn despawn_all_enemies(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

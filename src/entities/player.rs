//! 玩家系统

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::game::{
    not_upgrading, Collider, CollisionEvent, CollisionLayer, CollisionMask, GameConfig, GameData,
    GameState,
};
use crate::geometry::{spawn_geometry_entity, GeometryBlueprint};

use super::bullet::ShootCooldown;
use super::weapons::*;
use super::{Boss, BossState, Enemy};

/// 玩家插件
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DragState::default())
            .insert_resource(AutoShootTimer::default())
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnEnter(GameState::Menu), despawn_player)
            .add_systems(OnEnter(GameState::GameOver), despawn_player)
            .add_systems(OnEnter(GameState::Recharge), despawn_player)
            .add_systems(
                Update,
                (
                    player_touch_movement,
                    player_keyboard_movement,
                    auto_shoot_weapons,
                    update_weapon_bullets,
                    update_rocket_bullets,
                    update_aura_orbs,
                    update_homing_missiles,
                    resolve_lightning_casts,
                    handle_player_bullet_vs_enemy_bullet,
                    update_effect_lifetimes,
                    player_collision_handler,
                    update_invincibility,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(not_upgrading),
            );
    }
}

/// 玩家组件
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub invincible: bool,
    pub invincible_timer: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 300.0,
            invincible: false,
            invincible_timer: 0.0,
        }
    }
}

/// 拖拽状态资源
#[derive(Resource, Default)]
pub struct DragState {
    pub dragging: bool,
    pub last_position: Option<Vec2>,
}

/// 自动发射计时器
#[derive(Resource)]
pub struct AutoShootTimer {
    pub timer: f32,
}

impl Default for AutoShootTimer {
    fn default() -> Self {
        Self { timer: 0.0 }
    }
}

/// 生成玩家
fn spawn_player(mut commands: Commands, config: Res<GameConfig>, existing: Query<Entity, With<Player>>) {
    // 从 Paused -> Playing 恢复时，不重复生成玩家
    if !existing.is_empty() {
        return;
    }

    let blueprint = GeometryBlueprint::player_raiden_mk1();
    let position = Vec3::new(0.0, -config.window_height / 3.0, 10.0);

    let entity = spawn_geometry_entity(&mut commands, &blueprint, position);

    commands.entity(entity).insert((
        Player {
            speed: config.player_speed,
            ..default()
        },
        Collider::new(blueprint.collision.clone(), CollisionLayer::Player)
            .with_mask(CollisionMask::player_mask()),
        ShootCooldown {
            timer: 0.0,
            cooldown: config.shoot_cooldown,
        },
        WeaponInventory::new(),
    ));

    log::info!("Player spawned");
}

/// 销毁玩家
fn despawn_player(
    mut commands: Commands,
    query: Query<Entity, With<Player>>,
    aura_query: Query<Entity, With<AuraOrb>>,
    bullet_query: Query<Entity, With<WeaponBullet>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    // 销毁护身光球
    for entity in aura_query.iter() {
        commands.entity(entity).despawn();
    }
    // 销毁所有武器子弹
    for entity in bullet_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 触摸/鼠标拖拽移动
fn player_touch_movement(
    mut drag_state: ResMut<DragState>,
    touches: Res<Touches>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    config: Res<GameConfig>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let Ok(mut transform) = player_query.single_mut() else {
        return;
    };

    // 处理触摸输入
    let mut current_position: Option<Vec2> = None;
    let mut is_touching = false;

    for touch in touches.iter() {
        // Touch 对象直接访问 position，检查是否有活跃触摸
        current_position = Some(touch.position());
        is_touching = true;
        break; // 只使用第一个触摸点
    }

    // 处理鼠标输入
    if !is_touching {
        if mouse_button.pressed(MouseButton::Left) {
            if let Some(cursor_pos) = window.cursor_position() {
                current_position = Some(cursor_pos);
                is_touching = true;
            }
        } else {
            is_touching = false;
        }
    }

    // 更新拖拽状态
    if is_touching {
        if let Some(current_pos) = current_position {
            // 将屏幕坐标转换为世界坐标
            let window_size = Vec2::new(window.width(), window.height());
            let world_pos = Vec2::new(
                current_pos.x - window_size.x / 2.0,
                window_size.y / 2.0 - current_pos.y,
            );

            if drag_state.dragging {
                if let Some(last_pos) = drag_state.last_position {
                    // 计算差值并移动
                    let last_world = Vec2::new(
                        last_pos.x - window_size.x / 2.0,
                        window_size.y / 2.0 - last_pos.y,
                    );
                    let delta = world_pos - last_world;
                    transform.translation.x += delta.x;
                    transform.translation.y += delta.y;
                }
            }

            drag_state.dragging = true;
            drag_state.last_position = Some(current_pos);
        }
    } else {
        drag_state.dragging = false;
        drag_state.last_position = None;
    }

    // 限制在屏幕范围内
    let half_width = config.window_width / 2.0 - 30.0;
    let half_height = config.window_height / 2.0 - 30.0;
    transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
    transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
}

/// 键盘移动（保留）
fn player_keyboard_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    let Ok((mut transform, player)) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        let velocity = direction * player.speed * time.delta_secs();
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;

        // 限制在屏幕范围内
        let half_width = config.window_width / 2.0 - 30.0;
        let half_height = config.window_height / 2.0 - 30.0;

        transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
        transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
    }
}

/// 自动发射武器
fn auto_shoot_weapons(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut auto_timer: ResMut<AutoShootTimer>,
    mut query: Query<(&Transform, &mut WeaponInventory, &mut ShootCooldown), With<Player>>,
    // 明确排除 Player，避免与玩家 Query 在 Transform 访问上产生潜在重叠（B0001）
    enemy_query: Query<(Entity, &Transform), (Or<(With<Enemy>, With<Boss>)>, Without<Player>)>,
) {
    let Ok((transform, mut inventory, mut cooldown)) = query.single_mut() else {
        return;
    };

    let delta = time.delta_secs();
    auto_timer.timer += delta;
    cooldown.timer -= delta;

    // 更新所有武器冷却
    for weapon in inventory.weapons.iter_mut() {
        weapon.timer -= delta;
    }

    let player_pos = transform.translation;

    // 如果没有武器，使用默认子弹
    if inventory.weapons.is_empty() || inventory.has_default_bullet {
        if cooldown.timer <= 0.0 {
            let bullet_pos = player_pos + Vec3::new(0.0, 25.0, 0.0);
            super::bullet::spawn_player_bullet(&mut commands, bullet_pos, config.bullet_speed);
            cooldown.timer = cooldown.cooldown;
        }
        if inventory.weapons.is_empty() {
            return;
        }
    }

    // 预先计算一个“最近敌人”作为需要目标的武器参考
    let half_w = config.window_width * 0.5;
    let half_h = config.window_height * 0.5;
    let nearest_enemy = enemy_query
        .iter()
        // 只锁定屏幕内（并且在玩家前方）的敌人
        .filter(|(_, t)| {
            let p = t.translation;
            p.x >= -half_w
                && p.x <= half_w
                && p.y >= -half_h
                && p.y <= half_h
                && p.y >= player_pos.y
        })
        .min_by(|(_, a), (_, b)| {
            let da = (a.translation - player_pos).length();
            let db = (b.translation - player_pos).length();
            da.partial_cmp(&db).unwrap()
        })
        .map(|(e, _)| e);

    // 发射各种武器
    for weapon in inventory.weapons.iter_mut() {
        if weapon.timer <= 0.0 {
            match weapon.weapon_type {
                WeaponType::Shotgun => {
                    spawn_shotgun_pellets(
                        &mut commands,
                        player_pos,
                        weapon.level,
                        config.bullet_speed,
                    );
                }
                WeaponType::Rocket => {
                    spawn_rocket(
                        &mut commands,
                        player_pos,
                        weapon.level,
                        nearest_enemy,
                        config.bullet_speed,
                    );
                }
                WeaponType::Laser => {
                    spawn_laser(&mut commands, player_pos, weapon.level);
                }
                WeaponType::Homing => {
                    spawn_homing_missile(
                        &mut commands,
                        player_pos,
                        weapon.level,
                        nearest_enemy,
                        config.bullet_speed,
                    );
                }
                WeaponType::Lightning => {
                    // 生成一次性“施法请求”，由 resolve_lightning_casts 解析并结算伤害
                    if nearest_enemy.is_some() {
                        spawn_lightning(&mut commands, player_pos, weapon.level, nearest_enemy);
                    }
                }
                WeaponType::Aura => {
                    // Aura不需要发射，在 update_aura_orbs 中处理
                }
                WeaponType::Beam => {
                    spawn_beam_wave(&mut commands, &config, player_pos, weapon.level);
                }
            }
            weapon.timer = weapon.cooldown;
        }
    }
}

/// 更新武器子弹
fn update_weapon_bullets(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    // RocketBullet 在 update_rocket_bullets 内处理（需要爆炸结算）
    mut query: Query<(Entity, &mut Transform, &mut WeaponBullet), Without<RocketBullet>>,
) {
    let delta = time.delta_secs();
    let half_height = config.window_height / 2.0 + 50.0;
    let half_width = config.window_width / 2.0 + 50.0;

    for (entity, mut transform, mut bullet) in query.iter_mut() {
        // 更新生命周期
        bullet.lifetime -= delta;
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // 移动子弹
        let velocity = bullet.velocity * delta;
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;

        // 边界检查
        if transform.translation.x.abs() > half_width || transform.translation.y.abs() > half_height
        {
            commands.entity(entity).despawn();
        }
    }
}

/// 更新导弹：初始化方向、移动、超时/出界爆炸
fn update_rocket_bullets(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut rocket_query: Query<(Entity, &mut Transform, &mut WeaponBullet, &mut RocketBullet)>,
    mut enemy_set: ParamSet<(
        // 明确排除 RocketBullet，确保与 rocket_query 的 Transform 可变借用不重叠（B0001）
        Query<(Entity, &Transform), (With<Enemy>, Without<RocketBullet>)>,
        Query<&mut Enemy>,
    )>,
    mut game_data: ResMut<GameData>,
) {
    let delta = time.delta_secs();
    let half_height = config.window_height / 2.0 + 50.0;
    let half_width = config.window_width / 2.0 + 50.0;

    for (entity, mut transform, mut bullet, mut rocket) in rocket_query.iter_mut() {
        bullet.lifetime -= delta;

        if !rocket.initialized {
            let mut rng = rand::rng();
            let direction = if let Some(target) = rocket.target {
                enemy_set
                    .p0()
                    .get(target)
                    .ok()
                    .map(|(_, t)| {
                        (t.translation - transform.translation)
                            .truncate()
                            .normalize_or_zero()
                    })
                    .unwrap_or_else(|| {
                        // 没有目标时给一个随机前向角度
                        let angle = std::f32::consts::FRAC_PI_2 + rng.random_range(-0.4..0.4);
                        Vec2::new(angle.cos(), angle.sin())
                    })
            } else {
                let angle = std::f32::consts::FRAC_PI_2 + rng.random_range(-0.4..0.4);
                Vec2::new(angle.cos(), angle.sin())
            };

            bullet.velocity = direction * rocket.speed;
            rocket.initialized = true;
        }

        // 移动
        let step = bullet.velocity * delta;
        transform.translation.x += step.x;
        transform.translation.y += step.y;

        let out_of_bounds = transform.translation.x.abs() > half_width
            || transform.translation.y.abs() > half_height;
        let timed_out = bullet.lifetime <= 0.0;

        if out_of_bounds || timed_out {
            let targets: Vec<Entity> = {
                let center2 = transform.translation.truncate();
                let r2 = rocket.explosion_radius * rocket.explosion_radius;
                enemy_set
                    .p0()
                    .iter()
                    .filter_map(|(e, t)| {
                        (t.translation.truncate().distance_squared(center2) <= r2).then_some(e)
                    })
                    .collect()
            };
            for enemy_entity in targets {
                if let Ok(mut enemy) = enemy_set.p1().get_mut(enemy_entity) {
                    enemy.health -= bullet.damage;
                    if enemy.health <= 0 {
                        let score = enemy.score_value;
                        let position = enemy_set
                            .p0()
                            .get(enemy_entity)
                            .map(|(_, t)| t.translation)
                            .unwrap_or(transform.translation);
                        commands.entity(enemy_entity).despawn();
                        game_data.add_score(score);
                        
                        // 2% 概率掉落金币
                        let mut rng = rand::rng();
                        if rng.random_bool(0.02) {
                            use crate::entities::shield::{spawn_power_up, PowerUpType};
                            spawn_power_up(&mut commands, position, PowerUpType::Coin);
                        }
                    }
                }
            }
            // 爆炸：发射很多小三角碎片
            let shard_count = ((rocket.explosion_radius / 4.0) as u32).clamp(10, 28);
            spawn_rocket_explosion_particles(
                &mut commands,
                transform.translation,
                shard_count,
                rocket.speed,
            );
            commands.entity(entity).despawn();
        }
    }
}

/// 更新护身光球
fn update_aura_orbs(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<(Entity, &Transform, &WeaponInventory), With<Player>>,
    mut orb_query: Query<(Entity, &mut Transform, &mut AuraOrb), Without<Player>>,
) {
    let Ok((_, player_transform, inventory)) = player_query.single() else {
        // 没有玩家，销毁所有光球
        for (entity, _, _) in orb_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    // 检查是否有护身光球武器
    let aura_weapon = inventory
        .weapons
        .iter()
        .find(|w| w.weapon_type == WeaponType::Aura);

    let Some(aura) = aura_weapon else {
        // 没有护身光球武器，销毁现有光球
        for (entity, _, _) in orb_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    let level = aura.level;
    let orb_count = level as usize + 1; // 等级1有2个，等级5有6个
    let orbit_radius = 50.0 + level as f32 * 10.0;
    let orbit_speed = 3.0 + level as f32 * 0.5;

    let player_pos = player_transform.translation;
    let delta = time.delta_secs();

    // 更新现有光球
    let mut existing_count = 0;
    for (_, mut transform, mut orb) in orb_query.iter_mut() {
        orb.orbit_angle += orbit_speed * delta;
        orb.orbit_radius = orbit_radius;

        let x = player_pos.x + orb.orbit_radius * orb.orbit_angle.cos();
        let y = player_pos.y + orb.orbit_radius * orb.orbit_angle.sin();
        transform.translation = Vec3::new(x, y, player_pos.z + 1.0);
        existing_count += 1;
    }

    // 如果光球数量不够，生成新的
    if existing_count < orb_count {
        for i in existing_count..orb_count {
            let angle = (i as f32 / orb_count as f32) * std::f32::consts::TAU;
            // 需要玩家实体来生成光球
            // spawn_aura_orbs 需要 player_entity，我们直接在这里创建
            use crate::geometry::{
                spawn_geometry_entity, CollisionShape, GeometryBlueprint, GeometryShape,
                ShapeColor, Vec2D,
            };
            let blueprint = GeometryBlueprint {
                name: "aura_orb".to_string(),
                shapes: vec![GeometryShape::Circle {
                    radius: 8.0,
                    center: Vec2D::new(0.0, 0.0),
                    color: ShapeColor::new(1.0, 0.9, 0.3, 0.8),
                    fill: true,
                    stroke_width: 2.0,
                }],
                collision: CollisionShape::Circle { radius: 8.0 },
                scale: 1.0,
            };

            let orb_x = player_pos.x + orbit_radius * angle.cos();
            let orb_y = player_pos.y + orbit_radius * angle.sin();
            let pos = Vec3::new(orb_x, orb_y, player_pos.z + 1.0);

            let entity = spawn_geometry_entity(&mut commands, &blueprint, pos);
            commands.entity(entity).insert((
                WeaponBullet {
                    weapon_type: WeaponType::Aura,
                    damage: 1,
                    velocity: Vec2::ZERO,
                    lifetime: f32::MAX,
                },
                Pierce {
                    remaining: u32::MAX,
                },
                HitList::default(),
                AuraOrb {
                    orbit_angle: angle,
                    orbit_speed,
                    orbit_radius,
                },
                Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet).with_mask(
                    CollisionMask {
                        enemy_bullet: true,
                        ..CollisionMask::player_bullet_mask()
                    },
                ),
            ));
        }
    }
}

/// 更新自导导弹
fn update_homing_missiles(
    time: Res<Time>,
    // 明确排除 Enemy，避免与 enemy_query 在 Transform 访问上产生潜在重叠（B0001）
    mut missile_query: Query<(&mut Transform, &mut WeaponBullet, &HomingMissile), Without<Enemy>>,
    enemy_query: Query<&Transform, (Or<(With<Enemy>, With<Boss>)>, Without<HomingMissile>)>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut bullet, homing) in missile_query.iter_mut() {
        // 找最近的敌人
        let missile_pos = transform.translation;
        let closest_enemy = enemy_query.iter().map(|t| t.translation).min_by(|a, b| {
            let da = (*a - missile_pos).length();
            let db = (*b - missile_pos).length();
            da.partial_cmp(&db).unwrap()
        });

        if let Some(target_pos) = closest_enemy {
            let direction = (target_pos - missile_pos).truncate().normalize_or_zero();
            let target_velocity = direction * homing.speed;
            let turn_t = (homing.turn_rate * delta).clamp(0.0, 1.0);
            bullet.velocity = bullet.velocity.lerp(target_velocity, turn_t);
        }

        // 强制保持速度，避免数值漂移导致“静止”
        let dir = bullet.velocity.normalize_or_zero();
        bullet.velocity = if dir == Vec2::ZERO {
            Vec2::new(0.0, 1.0) * homing.speed
        } else {
            dir * homing.speed
        };

        // 根据速度朝向旋转（只是视觉）
        if bullet.velocity != Vec2::ZERO {
            transform.rotation = Quat::from_rotation_z(
                bullet.velocity.y.atan2(bullet.velocity.x) - std::f32::consts::FRAC_PI_2,
            );
        }
    }
}

/// 解析并结算闪电链：从玩家位置跳向若干个敌人（不重复）
fn resolve_lightning_casts(
    mut commands: Commands,
    mut casts: Query<(Entity, &Transform, &LightningCast)>,
    mut enemy_set: ParamSet<(Query<(Entity, &Transform), With<Enemy>>, Query<&mut Enemy>)>,
    mut boss_set: ParamSet<(Query<(Entity, &Transform), With<Boss>>, Query<&mut Boss>)>,
    mut boss_state: ResMut<BossState>,
    mut game_data: ResMut<GameData>,
) {
    for (cast_entity, cast_transform, cast) in casts.iter_mut() {
        let mut current = cast_transform.translation.truncate();
        let mut remaining = cast.jumps;
        let mut hit: Vec<(Entity, Vec2)> = Vec::new();
        let mut segments: Vec<(Vec2, Vec2)> = Vec::new();

        while remaining > 0 {
            let next_enemy = enemy_set
                .p0()
                .iter()
                .filter(|(e, t)| {
                    !hit.iter().any(|(he, _)| he == e)
                        && t.translation.truncate().distance(current) <= cast.range
                })
                .min_by(|(_, a), (_, b)| {
                    let da = a.translation.truncate().distance_squared(current);
                    let db = b.translation.truncate().distance_squared(current);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|(e, t)| (e, t.translation.truncate()));

            let next_boss = boss_set
                .p0()
                .iter()
                .filter(|(e, t)| {
                    !hit.iter().any(|(he, _)| he == e)
                        && t.translation.truncate().distance(current) <= cast.range
                })
                .min_by(|(_, a), (_, b)| {
                    let da = a.translation.truncate().distance_squared(current);
                    let db = b.translation.truncate().distance_squared(current);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|(e, t)| (e, t.translation.truncate()));

            let next = match (next_enemy, next_boss) {
                (None, None) => None,
                (Some(v), None) => Some(v),
                (None, Some(v)) => Some(v),
                (Some(a), Some(b)) => {
                    let da = a.1.distance_squared(current);
                    let db = b.1.distance_squared(current);
                    if da <= db { Some(a) } else { Some(b) }
                }
            };

            let Some((enemy_entity, enemy_pos)) = next else {
                break;
            };

            segments.push((current, enemy_pos));
            hit.push((enemy_entity, enemy_pos));
            current = enemy_pos;
            remaining -= 1;
        }

        // 结算伤害
        for (enemy_entity, enemy_pos) in &hit {
            if let Ok(mut enemy) = enemy_set.p1().get_mut(*enemy_entity) {
                enemy.health -= cast.damage;
                if enemy.health <= 0 {
                    let score = enemy.score_value;
                    let position = Vec3::new(enemy_pos.x, enemy_pos.y, 0.0);
                    commands.entity(*enemy_entity).despawn();
                    game_data.add_score(score);
                    
                    // 2% 概率掉落金币
                    let mut rng = rand::rng();
                    if rng.random_bool(0.02) {
                        use crate::entities::shield::{spawn_power_up, PowerUpType};
                        spawn_power_up(&mut commands, position, PowerUpType::Coin);
                    }
                }
            } else if let Ok(mut boss) = boss_set.p1().get_mut(*enemy_entity) {
                boss.health -= cast.damage;
                boss_state.current_health = boss.health;
                if boss.health <= 0 {
                    let score = boss.score_value;
                    commands.entity(*enemy_entity).despawn();
                    game_data.add_score(score);
                    boss_state.active = false;
                    boss_state.current_health = 0;
                    log::info!("Boss defeated! Score: {}", score);
                }
            }
        }

        // 生成视觉效果（线段）
        if !segments.is_empty() {
            use crate::geometry::{
                spawn_geometry_entity, CollisionShape, GeometryBlueprint, GeometryShape,
                ShapeColor, Vec2D,
            };
            let shapes: Vec<GeometryShape> = segments
                .iter()
                .map(|(a, b)| GeometryShape::Line {
                    start: Vec2D::new(a.x, a.y),
                    end: Vec2D::new(b.x, b.y),
                    color: ShapeColor::new(0.75, 0.75, 1.0, 0.8),
                    stroke_width: 3.0,
                })
                .collect();

            let blueprint = GeometryBlueprint {
                name: "lightning_fx".to_string(),
                shapes,
                collision: CollisionShape::Circle { radius: 0.0 },
                scale: 1.0,
            };

            let fx_entity = spawn_geometry_entity(&mut commands, &blueprint, Vec3::ZERO);
            commands
                .entity(fx_entity)
                .insert(EffectLifetime { remaining: 0.12 });
        }

        commands.entity(cast_entity).despawn();
    }
}

/// 护身光球抵消敌人子弹
fn handle_player_bullet_vs_enemy_bullet(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    weapon_bullets: Query<&WeaponBullet>,
) {
    for event in collision_events.read() {
        let (player_bullet, enemy_bullet) = match (event.layer_a, event.layer_b) {
            (CollisionLayer::PlayerBullet, CollisionLayer::EnemyBullet) => {
                (event.entity_a, event.entity_b)
            }
            (CollisionLayer::EnemyBullet, CollisionLayer::PlayerBullet) => {
                (event.entity_b, event.entity_a)
            }
            _ => continue,
        };

        let Ok(wb) = weapon_bullets.get(player_bullet) else {
            continue;
        };
        if wb.weapon_type == WeaponType::Aura {
            commands.entity(enemy_bullet).despawn();
        }
    }
}

fn update_effect_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EffectLifetime)>,
) {
    let delta = time.delta_secs();
    for (entity, mut life) in query.iter_mut() {
        life.remaining -= delta;
        if life.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// explode_at 已内联到 update_rocket_bullets，避免 Query 冲突（B0001）

/// 处理玩家碰撞
fn player_collision_handler(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Player>,
    power_up_query: Query<&crate::entities::shield::PowerUp>,
) {
    for event in collision_events.read() {
        // 检查是否涉及玩家
        let player_entity = if event.layer_a == CollisionLayer::Player {
            Some(event.entity_a)
        } else if event.layer_b == CollisionLayer::Player {
            Some(event.entity_b)
        } else {
            None
        };

        let Some(player_entity) = player_entity else {
            continue;
        };

        let Ok(mut player) = player_query.get_mut(player_entity) else {
            continue;
        };

        // 如果玩家无敌，跳过
        if player.invincible {
            continue;
        }

        // 确定另一个实体的类型
        let other_layer = if event.layer_a == CollisionLayer::Player {
            event.layer_b
        } else {
            event.layer_a
        };

        let other_entity = if event.layer_a == CollisionLayer::Player {
            event.entity_b
        } else {
            event.entity_a
        };

        match other_layer {
            CollisionLayer::Enemy | CollisionLayer::EnemyBullet => {
                // 玩家受伤 - 先扣护盾，再扣血
                if game_data.shield > 0 {
                    game_data.shield -= 1;
                    log::info!("Shield absorbed damage! Shield: {}", game_data.shield);
                } else if game_data.lives > 0 {
                    game_data.lives -= 1;
                    player.invincible = true;
                    player.invincible_timer = 2.0; // 2秒无敌时间
                    log::info!("Player hit! Lives remaining: {}", game_data.lives);
                }

                // 销毁敌人子弹
                if other_layer == CollisionLayer::EnemyBullet {
                    commands.entity(other_entity).despawn();
                }

                if game_data.lives == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
            CollisionLayer::PowerUp => {
                let power_type = power_up_query
                    .get(other_entity)
                    .map(|p| p.power_type)
                    .ok();

                match power_type {
                    Some(crate::entities::shield::PowerUpType::Coin) => {
                        game_data.coins += 1;
                        log::info!("Coin collected! Coins: {}", game_data.coins);
                    }
                    Some(crate::entities::shield::PowerUpType::Shield) => {
                        game_data.restore_shield(1);
                        log::info!(
                            "Shield restored! Shield: {}/{}",
                            game_data.shield,
                            game_data.max_shield
                        );
                    }
                    Some(crate::entities::shield::PowerUpType::ExtraLife) => {
                        game_data.heal(1);
                        log::info!(
                            "Life restored! Lives: {}/{}",
                            game_data.lives,
                            game_data.max_lives
                        );
                    }
                    Some(crate::entities::shield::PowerUpType::WeaponUpgrade) => {
                        game_data.upgrading = true;
                        log::info!("Weapon upgrade triggered");
                    }
                    None => {
                        // 兼容旧版本：没有 PowerUp 组件也当作小收益
                        game_data.coins += 1;
                    }
                }

                commands.entity(other_entity).despawn();
            }
            _ => {}
        }
    }
}

/// 更新无敌状态
fn update_invincibility(
    time: Res<Time>,
    mut query: Query<(&mut Player, &Children)>,
    mut visibility_query: Query<&mut Visibility>,
) {
    for (mut player, children) in query.iter_mut() {
        if player.invincible {
            player.invincible_timer -= time.delta_secs();

            // 闪烁效果
            let visible = (player.invincible_timer * 10.0) as i32 % 2 == 0;
            for child in children.iter() {
                if let Ok(mut visibility) = visibility_query.get_mut(child) {
                    *visibility = if visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }

            if player.invincible_timer <= 0.0 {
                player.invincible = false;
                // 确保可见
                for child in children.iter() {
                    if let Ok(mut visibility) = visibility_query.get_mut(child) {
                        *visibility = Visibility::Visible;
                    }
                }
            }
        }
    }
}

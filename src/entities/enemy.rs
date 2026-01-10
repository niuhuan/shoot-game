//! 敌人系统

use bevy::prelude::*;
use rand::Rng;

use crate::game::{Collider, CollisionEvent, CollisionLayer, CollisionMask, GameConfig, GameData, GameState, Scrollable};
use crate::geometry::{spawn_geometry_entity, GeometryBlueprint};

/// 敌人插件
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer::default())
            .add_systems(OnExit(GameState::Playing), despawn_all_enemies)
            .add_systems(
                Update,
                (
                    spawn_enemies,
                    enemy_movement,
                    enemy_shooting,
                    enemy_collision_handler,
                    despawn_offscreen_enemies,
                ).run_if(in_state(GameState::Playing)),
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
}

/// 敌人移动模式
#[derive(Component)]
pub enum EnemyMovement {
    /// 直线向下
    Straight { speed: f32 },
    /// 正弦波移动
    Sine { speed: f32, amplitude: f32, frequency: f32, time: f32 },
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
    mut spawn_timer: ResMut<EnemySpawnTimer>,
) {
    spawn_timer.timer += time.delta_secs();
    
    // 根据游戏时间增加难度
    spawn_timer.difficulty = 1.0 + game_data.play_time / 60.0;
    spawn_timer.interval = (config.enemy_spawn_interval / spawn_timer.difficulty).max(0.3);
    
    if spawn_timer.timer >= spawn_timer.interval {
        spawn_timer.timer = 0.0;
        
        let mut rng = rand::thread_rng();
        
        // 随机选择敌人类型
        let enemy_type = if rng.gen_bool(0.7) {
            EnemyType::Diamond
        } else if rng.gen_bool(0.5) {
            EnemyType::Hexagon
        } else {
            EnemyType::Small
        };
        
        // 随机X位置
        let x = rng.gen_range(-config.window_width / 2.0 + 50.0..config.window_width / 2.0 - 50.0);
        let y = config.window_height / 2.0 + 50.0;
        
        spawn_enemy(&mut commands, &config, Vec3::new(x, y, 5.0), enemy_type);
    }
}

/// 生成单个敌人
pub fn spawn_enemy(
    commands: &mut Commands,
    config: &GameConfig,
    position: Vec3,
    enemy_type: EnemyType,
) {
    let mut rng = rand::thread_rng();
    
    let (blueprint, health, score, shoot_interval) = match enemy_type {
        EnemyType::Diamond => (
            GeometryBlueprint::default_enemy(),
            2,
            100,
            2.0,
        ),
        EnemyType::Hexagon => (
            GeometryBlueprint::hexagon_enemy(),
            5,
            300,
            1.5,
        ),
        EnemyType::Small => (
            GeometryBlueprint::default_enemy(), // TODO: 创建小型敌人蓝图
            1,
            50,
            3.0,
        ),
    };
    
    let entity = spawn_geometry_entity(commands, &blueprint, position);
    
    // 随机移动模式
    let movement = match rng.gen_range(0..3) {
        0 => EnemyMovement::Straight { 
            speed: config.enemy_base_speed * rng.gen_range(0.8..1.2),
        },
        1 => EnemyMovement::Sine { 
            speed: config.enemy_base_speed * 0.8,
            amplitude: rng.gen_range(50.0..150.0),
            frequency: rng.gen_range(1.0..3.0),
            time: 0.0,
        },
        _ => EnemyMovement::Straight { 
            speed: config.enemy_base_speed,
        },
    };
    
    commands.entity(entity).insert((
        Enemy {
            health,
            max_health: health,
            score_value: score,
            enemy_type,
            shoot_timer: rng.gen_range(0.0..shoot_interval),
            shoot_interval,
        },
        movement,
        Collider::new(blueprint.collision.clone(), CollisionLayer::Enemy)
            .with_mask(CollisionMask::enemy_mask()),
        Scrollable::default(),
    ));
}

/// 敌人移动
fn enemy_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EnemyMovement)>,
) {
    for (mut transform, mut movement) in query.iter_mut() {
        match movement.as_mut() {
            EnemyMovement::Straight { speed } => {
                transform.translation.y -= *speed * time.delta_secs();
            }
            EnemyMovement::Sine { speed, amplitude, frequency, time: move_time } => {
                *move_time += time.delta_secs();
                transform.translation.y -= *speed * time.delta_secs();
                transform.translation.x += (*move_time * *frequency).cos() * *amplitude * time.delta_secs();
            }
            EnemyMovement::Homing { speed, turn_speed: _ } => {
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
            
            // 向下发射子弹
            let bullet_pos = transform.translation + Vec3::new(0.0, -20.0, 0.0);
            super::bullet::spawn_enemy_bullet(
                &mut commands, 
                bullet_pos, 
                Vec2::new(0.0, -config.bullet_speed * 0.6),
            );
        }
    }
}

/// 处理敌人碰撞
fn enemy_collision_handler(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut game_data: ResMut<GameData>,
    mut enemy_query: Query<&mut Enemy>,
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
                // 敌人被击中
                if let Ok(mut enemy) = enemy_query.get_mut(enemy_entity) {
                    enemy.health -= 1;
                    
                    // 销毁子弹
                    commands.entity(other_entity).despawn();
                    
                    if enemy.health <= 0 {
                        game_data.add_score(enemy.score_value);
                        commands.entity(enemy_entity).despawn();
                        log::info!("Enemy destroyed! Score: {}", game_data.score);
                    }
                }
            }
            _ => {}
        }
    }
}

/// 销毁屏幕外的敌人
fn despawn_offscreen_enemies(
    mut commands: Commands,
    config: Res<GameConfig>,
    query: Query<(Entity, &Transform), With<Enemy>>,
) {
    let despawn_y = -config.window_height / 2.0 - 100.0;
    
    for (entity, transform) in query.iter() {
        if transform.translation.y < despawn_y {
            commands.entity(entity).despawn();
        }
    }
}

/// 销毁所有敌人
fn despawn_all_enemies(
    mut commands: Commands,
    query: Query<Entity, With<Enemy>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

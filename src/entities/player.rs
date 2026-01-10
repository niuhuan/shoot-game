//! 玩家系统

use bevy::prelude::*;

use crate::game::{Collider, CollisionEvent, CollisionLayer, CollisionMask, GameConfig, GameData, GameState};
use crate::geometry::{spawn_geometry_entity, GeometryBlueprint};

use super::bullet::ShootCooldown;

/// 玩家插件
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnExit(GameState::Playing), despawn_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    player_shooting,
                    player_collision_handler,
                    update_invincibility,
                ).run_if(in_state(GameState::Playing)),
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

/// 生成玩家
fn spawn_player(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    let blueprint = GeometryBlueprint::default_player();
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
    ));
    
    log::info!("Player spawned");
}

/// 销毁玩家
fn despawn_player(
    mut commands: Commands,
    query: Query<Entity, With<Player>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 玩家移动
fn player_movement(
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

/// 玩家射击
fn player_shooting(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut query: Query<(&Transform, &mut ShootCooldown), With<Player>>,
) {
    let Ok((transform, mut cooldown)) = query.single_mut() else {
        return;
    };
    
    cooldown.timer -= time.delta_secs();
    
    if (keyboard.pressed(KeyCode::Space) || keyboard.pressed(KeyCode::KeyZ)) 
        && cooldown.timer <= 0.0 
    {
        let bullet_pos = transform.translation + Vec3::new(0.0, 25.0, 0.0);
        super::bullet::spawn_player_bullet(&mut commands, bullet_pos, config.bullet_speed);
        cooldown.timer = cooldown.cooldown;
    }
}

/// 处理玩家碰撞
fn player_collision_handler(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Player>,
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
                // 玩家受伤
                if game_data.lives > 0 {
                    game_data.lives -= 1;
                    player.invincible = true;
                    player.invincible_timer = 2.0; // 2秒无敌时间
                    log::info!("Player hit! Lives remaining: {}", game_data.lives);
                    
                    // 销毁敌人子弹
                    if other_layer == CollisionLayer::EnemyBullet {
                        commands.entity(other_entity).despawn();
                    }
                }
                
                if game_data.lives == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
            CollisionLayer::PowerUp => {
                // 拾取道具
                game_data.coins += 10;
                commands.entity(other_entity).despawn();
                log::info!("Power-up collected! Coins: {}", game_data.coins);
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
                    *visibility = if visible { Visibility::Visible } else { Visibility::Hidden };
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

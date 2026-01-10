//! 子弹系统

use bevy::prelude::*;

use crate::game::{not_upgrading, Collider, CollisionLayer, CollisionMask, GameConfig, GameState};
use crate::geometry::{spawn_geometry_entity, GeometryBlueprint};

/// 敌人子弹样式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyBulletStyle {
    Shard,
    Needle,
    Ring,
}

/// 子弹插件
pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), despawn_all_bullets)
            .add_systems(OnEnter(GameState::GameOver), despawn_all_bullets)
            .add_systems(OnEnter(GameState::Recharge), despawn_all_bullets)
            .add_systems(
                Update,
                (update_bullets, despawn_offscreen_bullets)
                    .run_if(in_state(GameState::Playing))
                    .run_if(not_upgrading),
            );
    }
}

/// 子弹组件
#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
    pub damage: i32,
    pub is_player_bullet: bool,
}

/// 射击冷却组件
#[derive(Component)]
pub struct ShootCooldown {
    pub timer: f32,
    pub cooldown: f32,
}

/// 生成玩家子弹
pub fn spawn_player_bullet(commands: &mut Commands, position: Vec3, speed: f32) {
    let blueprint = GeometryBlueprint::default_bullet();
    let entity = spawn_geometry_entity(commands, &blueprint, position);

    commands.entity(entity).insert((
        Bullet {
            velocity: Vec2::new(0.0, speed),
            damage: 1,
            is_player_bullet: true,
        },
        Collider::new(blueprint.collision.clone(), CollisionLayer::PlayerBullet)
            .with_mask(CollisionMask::player_bullet_mask()),
    ));
}

/// 生成敌人子弹
pub fn spawn_enemy_bullet(
    commands: &mut Commands,
    position: Vec3,
    velocity: Vec2,
    style: EnemyBulletStyle,
) {
    let blueprint = match style {
        EnemyBulletStyle::Shard => GeometryBlueprint::enemy_bullet(),
        EnemyBulletStyle::Needle => GeometryBlueprint::enemy_bullet_needle(),
        EnemyBulletStyle::Ring => GeometryBlueprint::enemy_bullet_ring(),
    };
    let entity = spawn_geometry_entity(commands, &blueprint, position);

    commands.entity(entity).insert((
        Bullet {
            velocity,
            damage: 1,
            is_player_bullet: false,
        },
        Collider::new(blueprint.collision.clone(), CollisionLayer::EnemyBullet)
            .with_mask(CollisionMask::enemy_bullet_mask()),
    ));
}

/// 更新子弹位置
fn update_bullets(time: Res<Time>, mut query: Query<(&mut Transform, &Bullet)>) {
    for (mut transform, bullet) in query.iter_mut() {
        transform.translation.x += bullet.velocity.x * time.delta_secs();
        transform.translation.y += bullet.velocity.y * time.delta_secs();
    }
}

/// 销毁屏幕外的子弹
fn despawn_offscreen_bullets(
    mut commands: Commands,
    config: Res<GameConfig>,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    let margin = 50.0;
    let half_width = config.window_width / 2.0 + margin;
    let half_height = config.window_height / 2.0 + margin;

    for (entity, transform) in query.iter() {
        let pos = transform.translation;
        if pos.x < -half_width || pos.x > half_width || pos.y < -half_height || pos.y > half_height
        {
            commands.entity(entity).despawn();
        }
    }
}

/// 销毁所有子弹
fn despawn_all_bullets(mut commands: Commands, query: Query<Entity, With<Bullet>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

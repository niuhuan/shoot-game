//! 护盾系统

use bevy::prelude::*;

use crate::game::{Collider, CollisionLayer, CollisionMask, GameState};
use crate::geometry::{spawn_geometry_entity, CollisionShape, GeometryBlueprint};

/// 护盾插件
pub struct ShieldPlugin;

impl Plugin for ShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_shield,
                shield_rotation,
            ).run_if(in_state(GameState::Playing)),
        );
    }
}

/// 护盾组件
#[derive(Component)]
pub struct Shield {
    pub health: i32,
    pub max_health: i32,
    pub rotation_speed: f32,
}

impl Default for Shield {
    fn default() -> Self {
        Self {
            health: 3,
            max_health: 3,
            rotation_speed: 1.0,
        }
    }
}

/// 为玩家生成护盾
pub fn spawn_player_shield(
    commands: &mut Commands,
    player_entity: Entity,
) {
    let blueprint = GeometryBlueprint::default_shield();
    
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            Transform::default(),
            Visibility::default(),
            Shield::default(),
            Collider::new(
                CollisionShape::Circle { radius: 25.0 },
                CollisionLayer::Player,
            ).with_mask(CollisionMask::player_mask()),
        ));
    });
}

/// 更新护盾状态
fn update_shield(
    mut commands: Commands,
    mut query: Query<(Entity, &Shield)>,
) {
    for (entity, shield) in query.iter_mut() {
        if shield.health <= 0 {
            commands.entity(entity).despawn();
            log::info!("Shield destroyed!");
        }
    }
}

/// 护盾旋转动画
fn shield_rotation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Shield)>,
) {
    for (mut transform, shield) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(
            transform.rotation.to_euler(EulerRot::ZYX).0 
            + shield.rotation_speed * time.delta_secs()
        );
    }
}

/// 道具组件
#[derive(Component)]
pub struct PowerUp {
    pub power_type: PowerUpType,
}

/// 道具类型
#[derive(Debug, Clone, Copy)]
pub enum PowerUpType {
    /// 护盾
    Shield,
    /// 增加生命
    ExtraLife,
    /// 武器升级
    WeaponUpgrade,
    /// 金币
    Coin,
}

/// 生成道具
pub fn spawn_power_up(
    commands: &mut Commands,
    position: Vec3,
    power_type: PowerUpType,
) {
    let blueprint = GeometryBlueprint::power_up();
    let entity = spawn_geometry_entity(commands, &blueprint, position);
    
    commands.entity(entity).insert((
        PowerUp { power_type },
        Collider::new(blueprint.collision.clone(), CollisionLayer::PowerUp)
            .with_mask(CollisionMask::default()),
        crate::game::Scrollable::default(),
    ));
}

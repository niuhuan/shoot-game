//! 护盾系统

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::game::{Collider, CollisionLayer, CollisionMask, GameState};
use crate::geometry::{spawn_geometry_entity, CollisionShape, ColorPulse, GeometryBlueprint, ShapeColor};
use crate::game::GameData;
use crate::entities::Player;

/// 护盾插件
pub struct ShieldPlugin;

impl Plugin for ShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_shield, shield_rotation, update_player_shield_vfx)
                .run_if(in_state(GameState::Playing)),
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
pub fn spawn_player_shield(commands: &mut Commands, player_entity: Entity) {
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            Transform::default(),
            Visibility::default(),
            Shield::default(),
            Collider::new(
                CollisionShape::Circle { radius: 25.0 },
                CollisionLayer::Player,
            )
            .with_mask(CollisionMask::player_mask()),
        ));
    });
}

/// 更新护盾状态
fn update_shield(mut commands: Commands, mut query: Query<(Entity, &Shield)>) {
    for (entity, shield) in query.iter_mut() {
        if shield.health <= 0 {
            commands.entity(entity).despawn();
            log::info!("Shield destroyed!");
        }
    }
}

/// 护盾旋转动画
fn shield_rotation(time: Res<Time>, mut query: Query<(&mut Transform, &Shield)>) {
    for (mut transform, shield) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(
            transform.rotation.to_euler(EulerRot::ZYX).0
                + shield.rotation_speed * time.delta_secs(),
        );
    }
}

/// 玩家护盾视觉特效（极低透明度慢闪烁，不盖住机身）
#[derive(Component)]
struct PlayerShieldVfx;

fn update_player_shield_vfx(
    mut commands: Commands,
    game_data: Res<GameData>,
    player_query: Query<Entity, With<Player>>,
    existing_vfx: Query<Entity, With<PlayerShieldVfx>>,
) {
    let Ok(player_entity) = player_query.single() else {
        for e in existing_vfx.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    if game_data.shield == 0 {
        for e in existing_vfx.iter() {
            commands.entity(e).despawn();
        }
        return;
    }

    if !existing_vfx.is_empty() {
        return;
    }

    // 用 lyon 直接画一个淡淡的光球，并用 ColorPulse 慢速呼吸（0.05~0.1）
    let circle = shapes::Circle {
        radius: 34.0,
        center: Vec2::ZERO,
    };
    let base = ShapeColor::new(0.15, 0.95, 0.85, 0.05);
    let pulse = ShapeColor::new(0.15, 0.95, 0.85, 0.10);
    let shape = ShapeBuilder::with(&circle)
        .fill(Fill::color(Color::from(base)))
        .build();

    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            shape,
            Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            Visibility::default(),
            PlayerShieldVfx,
            ColorPulse {
                base_color: base,
                pulse_color: pulse,
                speed: 0.7,
                time: 0.0,
            },
        ));
    });
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
pub fn spawn_power_up(commands: &mut Commands, position: Vec3, power_type: PowerUpType) {
    let blueprint = match power_type {
        PowerUpType::Coin => GeometryBlueprint::power_up_coin(),
        PowerUpType::Shield => GeometryBlueprint::power_up_shield(),
        PowerUpType::ExtraLife => GeometryBlueprint::power_up_heart(),
        PowerUpType::WeaponUpgrade => GeometryBlueprint::power_up(),
    };
    let entity = spawn_geometry_entity(commands, &blueprint, position);

    commands.entity(entity).insert((
        PowerUp { power_type },
        Collider::new(blueprint.collision.clone(), CollisionLayer::PowerUp)
            .with_mask(CollisionMask::default()),
        crate::game::Scrollable::default(),
    ));
}

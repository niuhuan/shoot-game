//! 碰撞检测系统

use bevy::prelude::*;

use crate::geometry::CollisionShape;

use super::states::{not_upgrading, GameState};

/// 碰撞系统插件
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CollisionEvent>()
            .add_systems(
                Update,
                detect_collisions
                    .run_if(in_state(GameState::Playing))
                    .run_if(not_upgrading),
            );
    }
}

/// 碰撞器组件
#[derive(Component)]
pub struct Collider {
    pub shape: CollisionShape,
    pub layer: CollisionLayer,
    pub mask: CollisionMask,
}

impl Collider {
    pub fn new(shape: CollisionShape, layer: CollisionLayer) -> Self {
        Self {
            shape,
            layer,
            mask: CollisionMask::default(),
        }
    }

    pub fn with_mask(mut self, mask: CollisionMask) -> Self {
        self.mask = mask;
        self
    }
}

/// 碰撞层
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionLayer {
    Player,
    PlayerBullet,
    Enemy,
    EnemyBullet,
    PowerUp,
}

/// 碰撞掩码
#[derive(Debug, Clone, Copy)]
pub struct CollisionMask {
    pub player: bool,
    pub player_bullet: bool,
    pub enemy: bool,
    pub enemy_bullet: bool,
    pub power_up: bool,
}

impl Default for CollisionMask {
    fn default() -> Self {
        Self {
            player: true,
            player_bullet: true,
            enemy: true,
            enemy_bullet: true,
            power_up: true,
        }
    }
}

impl CollisionMask {
    pub fn player_mask() -> Self {
        Self {
            player: false,
            player_bullet: false,
            enemy: true,
            enemy_bullet: true,
            power_up: true,
        }
    }

    pub fn player_bullet_mask() -> Self {
        Self {
            player: false,
            player_bullet: false,
            enemy: true,
            enemy_bullet: false,
            power_up: false,
        }
    }

    pub fn enemy_mask() -> Self {
        Self {
            player: true,
            player_bullet: true,
            enemy: false,
            enemy_bullet: false,
            power_up: false,
        }
    }

    pub fn enemy_bullet_mask() -> Self {
        Self {
            player: true,
            player_bullet: false,
            enemy: false,
            enemy_bullet: false,
            power_up: false,
        }
    }

    pub fn can_collide_with(&self, layer: CollisionLayer) -> bool {
        match layer {
            CollisionLayer::Player => self.player,
            CollisionLayer::PlayerBullet => self.player_bullet,
            CollisionLayer::Enemy => self.enemy,
            CollisionLayer::EnemyBullet => self.enemy_bullet,
            CollisionLayer::PowerUp => self.power_up,
        }
    }
}

/// 碰撞事件
#[derive(Message)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub layer_a: CollisionLayer,
    pub layer_b: CollisionLayer,
}

/// 检测碰撞
fn detect_collisions(
    query: Query<(Entity, &Transform, &Collider)>,
    mut collision_events: MessageWriter<CollisionEvent>,
) {
    let entities: Vec<_> = query.iter().collect();
    
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (entity_a, transform_a, collider_a) = entities[i];
            let (entity_b, transform_b, collider_b) = entities[j];
            
            // 检查碰撞掩码
            if !collider_a.mask.can_collide_with(collider_b.layer) 
                && !collider_b.mask.can_collide_with(collider_a.layer) {
                continue;
            }
            
            // 检测碰撞
            if check_collision(
                &transform_a.translation.truncate(),
                &collider_a.shape,
                &transform_b.translation.truncate(),
                &collider_b.shape,
            ) {
                collision_events.write(CollisionEvent {
                    entity_a,
                    entity_b,
                    layer_a: collider_a.layer,
                    layer_b: collider_b.layer,
                });
            }
        }
    }
}

/// 检查两个形状是否碰撞
fn check_collision(
    pos_a: &Vec2,
    shape_a: &CollisionShape,
    pos_b: &Vec2,
    shape_b: &CollisionShape,
) -> bool {
    match (shape_a, shape_b) {
        (CollisionShape::Circle { radius: r_a }, CollisionShape::Circle { radius: r_b }) => {
            circle_circle_collision(*pos_a, *r_a, *pos_b, *r_b)
        }
        (CollisionShape::Circle { radius }, CollisionShape::Rectangle { width, height }) => {
            circle_rect_collision(*pos_a, *radius, *pos_b, *width, *height)
        }
        (CollisionShape::Rectangle { width, height }, CollisionShape::Circle { radius }) => {
            circle_rect_collision(*pos_b, *radius, *pos_a, *width, *height)
        }
        (CollisionShape::Rectangle { width: w_a, height: h_a }, 
         CollisionShape::Rectangle { width: w_b, height: h_b }) => {
            rect_rect_collision(*pos_a, *w_a, *h_a, *pos_b, *w_b, *h_b)
        }
        // 多边形碰撞简化为外接圆
        (CollisionShape::Polygon { vertices: v_a }, CollisionShape::Polygon { vertices: v_b }) => {
            let r_a = polygon_bounding_radius(v_a);
            let r_b = polygon_bounding_radius(v_b);
            circle_circle_collision(*pos_a, r_a, *pos_b, r_b)
        }
        (CollisionShape::Polygon { vertices }, CollisionShape::Circle { radius }) => {
            let poly_r = polygon_bounding_radius(vertices);
            circle_circle_collision(*pos_a, poly_r, *pos_b, *radius)
        }
        (CollisionShape::Circle { radius }, CollisionShape::Polygon { vertices }) => {
            let poly_r = polygon_bounding_radius(vertices);
            circle_circle_collision(*pos_a, *radius, *pos_b, poly_r)
        }
        (CollisionShape::Polygon { vertices }, CollisionShape::Rectangle { width, height }) => {
            let poly_r = polygon_bounding_radius(vertices);
            circle_rect_collision(*pos_a, poly_r, *pos_b, *width, *height)
        }
        (CollisionShape::Rectangle { width, height }, CollisionShape::Polygon { vertices }) => {
            let poly_r = polygon_bounding_radius(vertices);
            circle_rect_collision(*pos_b, poly_r, *pos_a, *width, *height)
        }
    }
}

/// 圆形与圆形碰撞检测
fn circle_circle_collision(pos_a: Vec2, r_a: f32, pos_b: Vec2, r_b: f32) -> bool {
    let distance_sq = pos_a.distance_squared(pos_b);
    let radius_sum = r_a + r_b;
    distance_sq <= radius_sum * radius_sum
}

/// 圆形与矩形碰撞检测
fn circle_rect_collision(
    circle_pos: Vec2,
    radius: f32,
    rect_pos: Vec2,
    width: f32,
    height: f32,
) -> bool {
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    
    let closest_x = circle_pos.x.clamp(rect_pos.x - half_w, rect_pos.x + half_w);
    let closest_y = circle_pos.y.clamp(rect_pos.y - half_h, rect_pos.y + half_h);
    
    let distance_sq = (circle_pos.x - closest_x).powi(2) + (circle_pos.y - closest_y).powi(2);
    distance_sq <= radius * radius
}

/// 矩形与矩形碰撞检测
fn rect_rect_collision(
    pos_a: Vec2,
    w_a: f32,
    h_a: f32,
    pos_b: Vec2,
    w_b: f32,
    h_b: f32,
) -> bool {
    let half_wa = w_a / 2.0;
    let half_ha = h_a / 2.0;
    let half_wb = w_b / 2.0;
    let half_hb = h_b / 2.0;
    
    (pos_a.x - half_wa) < (pos_b.x + half_wb)
        && (pos_a.x + half_wa) > (pos_b.x - half_wb)
        && (pos_a.y - half_ha) < (pos_b.y + half_hb)
        && (pos_a.y + half_ha) > (pos_b.y - half_hb)
}

/// 计算多边形的外接圆半径
fn polygon_bounding_radius(vertices: &[crate::geometry::Vec2D]) -> f32 {
    vertices
        .iter()
        .map(|v| (v.x * v.x + v.y * v.y).sqrt())
        .fold(0.0f32, |a, b| a.max(b))
}

//! 卷轴系统
//! 实现自下而上的自动滚动效果

use bevy::prelude::*;

use super::states::{GameConfig, GameState};

/// 卷轴系统插件
pub struct ScrollPlugin;

impl Plugin for ScrollPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScrollState::default())
            .add_systems(
                Update,
                (
                    update_scroll,
                    update_scrollable_entities,
                    update_background_scroll,
                ).run_if(in_state(GameState::Playing)),
            );
    }
}

/// 卷轴状态
#[derive(Resource, Default)]
pub struct ScrollState {
    /// 当前滚动偏移量
    pub offset: f32,
    /// 总滚动距离
    pub total_distance: f32,
}

/// 标记组件：受卷轴影响的实体
#[derive(Component)]
pub struct Scrollable {
    /// 是否随卷轴移动
    pub moves_with_scroll: bool,
    /// 额外的速度倍数
    pub speed_multiplier: f32,
}

impl Default for Scrollable {
    fn default() -> Self {
        Self {
            moves_with_scroll: true,
            speed_multiplier: 1.0,
        }
    }
}

/// 背景层组件
#[derive(Component)]
pub struct BackgroundLayer {
    /// 视差系数 (0.0 = 不动, 1.0 = 全速)
    pub parallax_factor: f32,
    /// 层级（用于排序）
    pub layer: i32,
}

/// 更新卷轴状态
fn update_scroll(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut scroll_state: ResMut<ScrollState>,
) {
    let delta = config.scroll_speed * time.delta_secs();
    scroll_state.offset += delta;
    scroll_state.total_distance += delta;
}

/// 更新受卷轴影响的实体位置
fn update_scrollable_entities(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &Scrollable)>,
) {
    let scroll_delta = config.scroll_speed * time.delta_secs();
    
    for (mut transform, scrollable) in query.iter_mut() {
        if scrollable.moves_with_scroll {
            // 实体向下移动（玩家视角向上）
            transform.translation.y -= scroll_delta * scrollable.speed_multiplier;
        }
    }
}

/// 更新背景滚动
fn update_background_scroll(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &BackgroundLayer)>,
) {
    let scroll_delta = config.scroll_speed * time.delta_secs();
    
    for (mut transform, layer) in query.iter_mut() {
        transform.translation.y -= scroll_delta * layer.parallax_factor;
        
        // 循环背景
        if transform.translation.y < -config.window_height {
            transform.translation.y += config.window_height * 2.0;
        }
    }
}

/// 生成背景网格
pub fn spawn_background_grid(
    commands: &mut Commands,
    config: &GameConfig,
) {
    let grid_size = 50.0;
    let color = Color::srgba(0.1, 0.2, 0.3, 0.5);
    
    // 创建背景容器
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, -100.0)),
        Visibility::default(),
        BackgroundLayer {
            parallax_factor: 0.3,
            layer: -1,
        },
    )).with_children(|parent| {
        // 水平线
        let h_lines = (config.window_height * 3.0 / grid_size) as i32;
        for i in -h_lines..=h_lines {
            let y = i as f32 * grid_size;
            parent.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(config.window_width, 1.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, y, 0.0)),
            ));
        }
        
        // 垂直线
        let v_lines = (config.window_width / grid_size) as i32;
        for i in -v_lines..=v_lines {
            let x = i as f32 * grid_size;
            parent.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, config.window_height * 3.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, 0.0, 0.0)),
            ));
        }
    });
}

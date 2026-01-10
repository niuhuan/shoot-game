//! 几何射击游戏
//! 一款基于 Bevy 的几何风格射击游戏

pub mod game;
pub mod geometry;
pub mod entities;
pub mod storage;
pub mod ui;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use game::{CollisionPlugin, GameConfig, GameStatePlugin, ScrollPlugin};
use geometry::GeometryRendererPlugin;
use entities::{BulletPlugin, EnemyPlugin, PlayerPlugin, ShieldPlugin};
use storage::{RechargePlugin, StoragePlugin};
use ui::{HudPlugin, InputPlugin, MenuPlugin};

/// 游戏主插件
pub struct ShootGamePlugin;

impl Plugin for ShootGamePlugin {
    fn build(&self, app: &mut App) {
        app
            // 核心游戏系统
            .add_plugins(GameStatePlugin)
            .add_plugins(ScrollPlugin)
            .add_plugins(CollisionPlugin)
            // 几何渲染
            .add_plugins(GeometryRendererPlugin)
            // 实体系统
            .add_plugins(PlayerPlugin)
            .add_plugins(EnemyPlugin)
            .add_plugins(BulletPlugin)
            .add_plugins(ShieldPlugin)
            // 存储和网络
            .add_plugins(StoragePlugin)
            .add_plugins(RechargePlugin)
            // UI
            .add_plugins(MenuPlugin)
            .add_plugins(HudPlugin)
            .add_plugins(InputPlugin)
            // 初始化
            .add_systems(Startup, setup_game)
            .add_systems(
                Update,
                transition_to_menu.run_if(in_state(game::GameState::Loading)),
            );
    }
}

/// 初始化游戏
fn setup_game(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    // 创建相机
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.02, 0.08)),
            ..default()
        },
    ));
    
    // 创建背景网格
    game::spawn_background_grid(&mut commands, &config);
    
    log::info!("Game initialized");
}

/// 加载完成后切换到菜单
fn transition_to_menu(
    mut next_state: ResMut<NextState<game::GameState>>,
    time: Res<Time>,
    mut elapsed: Local<f32>,
) {
    *elapsed += time.delta_secs();
    
    // 等待一小段时间确保资源加载完成
    if *elapsed > 0.5 {
        next_state.set(game::GameState::Menu);
    }
}

/// 获取默认窗口插件配置
pub fn default_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "几何射击 - Geometry Shooter".to_string(),
            resolution: WindowResolution::from((480, 720)),
            resizable: false,
            canvas: Some("#game-canvas".to_string()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }
}

/// WASM 入口点
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    // 设置 panic hook
    console_error_panic_hook::set_once();
    
    // 设置日志
    console_log::init_with_level(log::Level::Info).expect("Failed to init logger");
    
    log::info!("Starting WASM game...");
    
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(default_window_plugin())
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu=error,naga=warn".to_string(),
                    ..default()
                }),
        )
        .add_plugins(ShootGamePlugin)
        .run();
}

/// 从 JS 调用的充值提交函数
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn submit_recharge_form(username: String, order_id: String) {
    storage::enqueue_recharge_submit(username, order_id);
}

/// 从 JS 调用：取消充值并返回菜单
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn cancel_recharge() {
    storage::enqueue_recharge_cancel();
}

/// 兼容旧接口：把 code 当作用户名传入（将触发校验失败并提示改用新表单）
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn submit_recharge_code(code: String) {
    storage::enqueue_recharge_submit(code, String::new());
}

/// 从 JS 获取游戏状态
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_game_info() -> String {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "Geometry Shooter"
    }).to_string()
}

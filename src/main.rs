//! 几何射击游戏 - 原生入口点

use bevy::prelude::*;
use shoot::{default_window_plugin, ShootGamePlugin};

fn main() {
    // 初始化日志
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    
    log::info!("Starting Geometry Shooter...");
    
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

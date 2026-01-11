//! Web Storage 存储系统
//! 使用浏览器的 LocalStorage 保存游戏数据

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::GameState;

/// 存储键名前缀
const STORAGE_PREFIX: &str = "shoot_game_";

/// 存储插件
pub struct StoragePlugin;

impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveData::default())
            .add_systems(Startup, load_game_data)
            .add_systems(OnEnter(GameState::GameOver), auto_save)
            .add_systems(OnEnter(GameState::Menu), auto_save);
    }
}

/// 可保存的游戏数据
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveData {
    /// 最高分
    pub high_score: u32,
    /// 累计金币
    pub total_coins: u32,
    /// 已解锁的飞机
    pub unlocked_ships: Vec<String>,
    /// 游戏设置
    pub settings: GameSettings,
    /// 打赏记录 (仅记录是否有打赏)
    pub has_purchased: bool,
    /// 机身强化等级（0-2）
    #[serde(default)]
    pub hull_upgrade_level: u8,
    /// 护盾强化等级（0-2）
    #[serde(default)]
    pub shield_upgrade_level: u8,
}

/// 游戏设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// 音效开关
    pub sound_enabled: bool,
    /// 音乐开关
    pub music_enabled: bool,
    /// 音效音量 (0.0 - 1.0)
    pub sound_volume: f32,
    /// 音乐音量 (0.0 - 1.0)
    pub music_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            music_enabled: true,
            sound_volume: 0.7,
            music_volume: 0.5,
        }
    }
}

/// 加载游戏数据
fn load_game_data(mut save_data: ResMut<SaveData>) {
    match load_from_storage() {
        Ok(data) => {
            *save_data = data;
            log::info!("Game data loaded successfully");
        }
        Err(e) => {
            log::warn!("Failed to load game data: {}", e);
        }
    }
}

/// 自动保存
fn auto_save(save_data: Res<SaveData>) {
    if let Err(e) = save_to_storage(&save_data) {
        log::error!("Failed to save game data: {}", e);
    } else {
        log::info!("Game data saved");
    }
}

/// 从 LocalStorage 加载数据
#[cfg(target_arch = "wasm32")]
fn load_from_storage() -> Result<SaveData, String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No window")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage")?
        .ok_or("No localStorage")?;

    let key = format!("{}save", STORAGE_PREFIX);
    let data = storage
        .get_item(&key)
        .map_err(|_| "Failed to get item")?
        .ok_or("No saved data")?;

    serde_json::from_str(&data).map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
fn load_from_storage() -> Result<SaveData, String> {
    use std::fs;

    let path = get_save_path()?;
    let data = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Parse error: {}", e))
}

/// 保存到 LocalStorage
#[cfg(target_arch = "wasm32")]
fn save_to_storage(data: &SaveData) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage")?
        .ok_or("No localStorage")?;

    let key = format!("{}save", STORAGE_PREFIX);
    let json = serde_json::to_string(data).map_err(|e| format!("Serialize error: {}", e))?;

    storage
        .set_item(&key, &json)
        .map_err(|_| "Failed to set item".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_to_storage(data: &SaveData) -> Result<(), String> {
    use std::fs;

    let path = get_save_path()?;
    let json = serde_json::to_string_pretty(data).map_err(|e| format!("Serialize error: {}", e))?;

    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Dir error: {}", e))?;
    }

    fs::write(path, json).map_err(|e| format!("Write error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
fn get_save_path() -> Result<std::path::PathBuf, String> {
    let mut path = dirs::data_local_dir().ok_or("No data dir")?;
    path.push("shoot_game");
    path.push("save.json");
    Ok(path)
}

/// 公开的保存函数
pub fn save_game(save_data: &SaveData) -> Result<(), String> {
    save_to_storage(save_data)
}

/// 公开的加载函数
pub fn load_game() -> Result<SaveData, String> {
    load_from_storage()
}

/// 清除所有保存数据
#[cfg(target_arch = "wasm32")]
pub fn clear_save_data() -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage")?
        .ok_or("No localStorage")?;

    let key = format!("{}save", STORAGE_PREFIX);
    storage
        .remove_item(&key)
        .map_err(|_| "Failed to remove item".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_save_data() -> Result<(), String> {
    use std::fs;
    let path = get_save_path()?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Remove error: {}", e))?;
    }
    Ok(())
}

//! 游戏状态管理

use bevy::prelude::*;
use bevy::state::state::StateTransitionEvent;

/// 游戏主状态
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    /// 加载资源
    #[default]
    Loading,
    /// 主菜单
    Menu,
    /// 游戏中
    Playing,
    /// 暂停
    Paused,
    /// 游戏结束
    GameOver,
    /// 打赏界面
    Recharge,
}

/// 游戏数据资源
#[derive(Resource, Default)]
pub struct GameData {
    /// 当前分数
    pub score: u32,
    /// 最高分
    pub high_score: u32,
    /// 金币数量
    pub coins: u32,
    /// 当前关卡
    pub level: u32,
    /// 玩家生命值
    pub lives: u32,
    /// 玩家最大生命值
    pub max_lives: u32,
    /// 护盾值
    pub shield: u32,
    /// 最大护盾值
    pub max_shield: u32,
    /// 游戏时间（秒）
    pub play_time: f32,
    /// 当前经验值
    pub experience: u32,
    /// 战机等级
    pub player_level: u32,
    /// 是否正在升级选择
    pub upgrading: bool,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            score: 0,
            high_score: 0,
            coins: 0,
            level: 1,
            lives: 3,
            max_lives: 5,
            shield: 0,
            max_shield: 3,
            play_time: 0.0,
            experience: 0,
            player_level: 1,
            upgrading: false,
        }
    }

    pub fn reset(&mut self) {
        self.score = 0;
        self.coins = 0;
        self.level = 1;
        self.lives = 3;
        self.max_lives = 5;
        self.shield = 0;
        self.max_shield = 3;
        self.play_time = 0.0;
        self.experience = 0;
        self.player_level = 1;
        self.upgrading = false;
    }

    pub fn add_score(&mut self, points: u32) {
        self.score += points;
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        // 添加经验值
        self.add_experience(points);
    }

    /// 仅增加分数（不增加经验）
    pub fn add_score_only(&mut self, points: u32) {
        self.score += points;
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }

    /// 计算升级所需经验值（曲线公式）
    pub fn exp_for_level(level: u32) -> u32 {
        // 经验曲线：base * level^1.5
        let base = 1300.0; // 基础经验值
        (base * (level as f32).powf(1.5)) as u32
    }

    /// 添加经验值并检查升级
    pub fn add_experience(&mut self, exp: u32) {
        self.experience += exp;
        let required = Self::exp_for_level(self.player_level);
        if self.experience >= required {
            self.experience -= required;
            self.player_level += 1;
            self.upgrading = true; // 标记需要升级选择
            log::info!("Level up! Now level {}", self.player_level);
        }
    }

    /// 经验值进度 (0.0 - 1.0)
    pub fn exp_progress(&self) -> f32 {
        let required = Self::exp_for_level(self.player_level);
        (self.experience as f32 / required as f32).min(1.0)
    }

    /// 恢复生命值
    pub fn heal(&mut self, amount: u32) {
        self.lives = (self.lives + amount).min(self.max_lives);
    }

    /// 恢复护盾
    pub fn restore_shield(&mut self, amount: u32) {
        self.shield = (self.shield + amount).min(self.max_shield);
    }

    /// 计算敌人难度系数
    pub fn difficulty_multiplier(&self) -> f32 {
        1.0 + (self.player_level as f32 - 1.0) * 0.15
    }
}

/// 游戏配置
#[derive(Resource)]
pub struct GameConfig {
    /// 游戏窗口宽度
    pub window_width: f32,
    /// 游戏窗口高度
    pub window_height: f32,
    /// 玩家移动速度
    pub player_speed: f32,
    /// 子弹速度
    pub bullet_speed: f32,
    /// 敌人基础速度
    pub enemy_base_speed: f32,
    /// 卷轴速度
    pub scroll_speed: f32,
    /// 敌人生成间隔（秒）
    pub enemy_spawn_interval: f32,
    /// 射击冷却时间（秒）
    pub shoot_cooldown: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            window_width: 480.0,
            window_height: 720.0,
            player_speed: 300.0,
            bullet_speed: 500.0,
            enemy_base_speed: 150.0,
            scroll_speed: 50.0,
            enemy_spawn_interval: 1.5,
            shoot_cooldown: 0.15,
        }
    }
}

/// 游戏状态插件
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(GameData::new())
            .insert_resource(GameConfig::default())
            .add_systems(OnExit(GameState::Playing), on_exit_playing)
            .add_systems(OnEnter(GameState::GameOver), on_enter_game_over)
            .add_systems(Update, log_state_transitions)
            .add_systems(
                Update,
                (update_game_time, handle_pause_input).run_if(in_state(GameState::Playing)),
            );
    }
}

fn on_exit_playing() {
    log::info!("Exiting playing state");
}

fn on_enter_game_over(
    game_data: Res<GameData>,
    mut save_data: ResMut<crate::storage::SaveData>,
) {
    log::info!("Game Over! Score: {}", game_data.score);
    // 将游戏中的金币累加到总金币
    save_data.total_coins += game_data.coins;
    // 更新最高分
    save_data.high_score = save_data.high_score.max(game_data.score);
    // 立即保存，避免与 StoragePlugin 的 OnEnter(GameOver) 执行顺序产生竞态
    if let Err(e) = crate::storage::save_game(&save_data) {
        log::error!("Failed to save settled data: {}", e);
    }
}

fn log_state_transitions(mut transitions: MessageReader<StateTransitionEvent<GameState>>) {
    // 每帧最多 1 次转换事件（由状态机保证），这里只做轻量日志便于排查“状态未切换”的问题
    for t in transitions.read() {
        log::info!("State transition: {:?} -> {:?}", t.exited, t.entered);
    }
}

fn update_game_time(time: Res<Time>, mut game_data: ResMut<GameData>) {
    game_data.play_time += time.delta_secs();
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

/// 暂停状态系统
pub fn handle_unpause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

/// 升级选择时暂停游戏逻辑（但不改变 GameState）
pub fn not_upgrading(game_data: Res<GameData>) -> bool {
    !game_data.upgrading
}

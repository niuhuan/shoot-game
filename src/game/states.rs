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
    /// 充值界面
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
    /// 游戏时间（秒）
    pub play_time: f32,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            score: 0,
            high_score: 0,
            coins: 0,
            level: 1,
            lives: 3,
            play_time: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.score = 0;
        self.level = 1;
        self.lives = 3;
        self.play_time = 0.0;
    }

    pub fn add_score(&mut self, points: u32) {
        self.score += points;
        if self.score > self.high_score {
            self.high_score = self.score;
        }
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
            .add_systems(OnEnter(GameState::Playing), on_enter_playing)
            .add_systems(OnExit(GameState::Playing), on_exit_playing)
            .add_systems(OnEnter(GameState::GameOver), on_enter_game_over)
            .add_systems(Update, log_state_transitions)
            .add_systems(
                Update,
                (
                    update_game_time,
                    handle_pause_input,
                ).run_if(in_state(GameState::Playing)),
            );
    }
}

fn on_enter_playing(mut game_data: ResMut<GameData>) {
    game_data.reset();
    log::info!("Game started!");
}

fn on_exit_playing() {
    log::info!("Exiting playing state");
}

fn on_enter_game_over(game_data: Res<GameData>) {
    log::info!("Game Over! Score: {}", game_data.score);
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

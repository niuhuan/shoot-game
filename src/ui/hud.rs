//! HUD (Head-Up Display) 游戏内界面

use bevy::prelude::*;
use bevy::state::prelude::DespawnOnExit;

use crate::game::{GameData, GameState};

/// HUD 插件
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(OnExit(GameState::Playing), cleanup_hud)
            .add_systems(
                Update,
                update_hud.run_if(in_state(GameState::Playing)),
            );
    }
}

/// HUD 根节点标记
#[derive(Component)]
struct HudRoot;

/// 分数文本标记
#[derive(Component)]
struct ScoreText;

/// 生命值文本标记
#[derive(Component)]
struct LivesText;

/// 金币文本标记
#[derive(Component)]
struct CoinsText;

/// 设置 HUD
fn setup_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("NotoSansCJKsc-Regular.otf");
    
    // HUD 根节点
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            HudRoot,
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            // 顶部状态栏
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // 左侧：分数
                    parent.spawn((
                        Text::new("分数: 0"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        ScoreText,
                    ));
                    
                    // 中间：金币
                    parent.spawn((
                        Text::new("金币: 0"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.0)),
                        CoinsText,
                    ));
                    
                    // 右侧：生命值
                    parent.spawn((
                        Text::new("❤️❤️❤️"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        LivesText,
                    ));
                });
        });
}

/// 清理 HUD
fn cleanup_hud(
    mut commands: Commands,
    query: Query<Entity, With<HudRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 更新 HUD
fn update_hud(
    game_data: Res<GameData>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<LivesText>, Without<CoinsText>)>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<CoinsText>)>,
    mut coins_query: Query<&mut Text, (With<CoinsText>, Without<ScoreText>, Without<LivesText>)>,
) {
    // 更新分数
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("分数: {}", game_data.score);
    }
    
    // 更新生命值
    if let Ok(mut text) = lives_query.single_mut() {
        let hearts = "❤️".repeat(game_data.lives as usize);
        let empty = "💔".repeat(3_usize.saturating_sub(game_data.lives as usize));
        **text = format!("{}{}", hearts, empty);
    }
    
    // 更新金币
    if let Ok(mut text) = coins_query.single_mut() {
        **text = format!("金币: {}", game_data.coins);
    }
}

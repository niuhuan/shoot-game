//! HUD (Head-Up Display) 游戏内界面

use bevy::prelude::*;

use crate::entities::{BossState, Player, WeaponInventory, WeaponType};
use crate::game::{GameData, GameState};

/// HUD 插件
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(OnEnter(GameState::Menu), cleanup_hud)
            .add_systems(OnEnter(GameState::GameOver), cleanup_hud)
            .add_systems(OnEnter(GameState::Recharge), cleanup_hud)
            .add_systems(
                Update,
                (update_hud, update_boss_hud).run_if(in_state(GameState::Playing)),
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

/// 护盾文本标记
#[derive(Component)]
struct ShieldText;

/// 经验值条标记
#[derive(Component)]
struct ExpBarText;

/// 武器列表标记
#[derive(Component)]
struct WeaponsText;

/// 等级文本标记
#[derive(Component)]
struct LevelText;

/// Boss血量条根节点
#[derive(Component)]
struct BossHudRoot;

/// Boss名称文本
#[derive(Component)]
struct BossNameText;

/// Boss血量百分比文本
#[derive(Component)]
struct BossHealthText;

/// Boss血量条背景
#[derive(Component)]
struct BossHealthBarBg;

/// Boss血量条填充
#[derive(Component)]
struct BossHealthBarFill;

/// 设置 HUD
fn setup_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<HudRoot>>,
) {
    // 从 Paused -> Playing 恢复时，不重复生成 HUD
    if !existing.is_empty() {
        return;
    }

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
        ))
        .with_children(|parent| {
            // 顶部状态栏
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },))
                .with_children(|parent| {
                    // 左侧：分数和等级
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },))
                        .with_children(|parent| {
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
                            parent.spawn((
                                Text::new("等级: 1"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 1.0, 0.5)),
                                LevelText,
                            ));
                        });

                    // 中间：金币和经验条
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
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
                            parent.spawn((
                                Text::new("经验: [----------]"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 1.0)),
                                ExpBarText,
                            ));
                        });

                    // 右侧：生命值和护盾
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::End,
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("生命: ♥♥♥"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                                LivesText,
                            ));
                            parent.spawn((
                                Text::new("护盾: "),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.3, 0.7, 1.0)),
                                ShieldText,
                            ));
                        });
                });

            // 底部武器栏
            parent
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("武器: 无"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        WeaponsText,
                    ));
                });

            // Boss血量条（初始隐藏）
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(60.0),
                        left: Val::Percent(10.0),
                        width: Val::Percent(80.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        display: Display::None, // 初始隐藏
                        ..default()
                    },
                    BossHudRoot,
                ))
                .with_children(|parent| {
                    // Boss名称
                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        BossNameText,
                    ));

                    // 血量条容器
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(20.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                            BossHealthBarBg,
                        ))
                        .with_children(|parent| {
                            // 血量填充条
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(1.0, 0.2, 0.2)),
                                BossHealthBarFill,
                            ));
                        });

                    // 血量百分比文字
                    parent.spawn((
                        Text::new("100%"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::top(Val::Px(3.0)),
                            ..default()
                        },
                        BossHealthText,
                    ));
                });
        });
}

/// 清理 HUD
fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 更新 HUD
fn update_hud(
    game_data: Res<GameData>,
    player_query: Query<&WeaponInventory, With<Player>>,
    mut score_query: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<LivesText>,
            Without<CoinsText>,
            Without<ShieldText>,
            Without<ExpBarText>,
            Without<WeaponsText>,
            Without<LevelText>,
        ),
    >,
    mut lives_query: Query<
        &mut Text,
        (
            With<LivesText>,
            Without<ScoreText>,
            Without<CoinsText>,
            Without<ShieldText>,
            Without<ExpBarText>,
            Without<WeaponsText>,
            Without<LevelText>,
        ),
    >,
    mut coins_query: Query<
        &mut Text,
        (
            With<CoinsText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<ShieldText>,
            Without<ExpBarText>,
            Without<WeaponsText>,
            Without<LevelText>,
        ),
    >,
    mut shield_query: Query<
        &mut Text,
        (
            With<ShieldText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<CoinsText>,
            Without<ExpBarText>,
            Without<WeaponsText>,
            Without<LevelText>,
        ),
    >,
    mut exp_query: Query<
        &mut Text,
        (
            With<ExpBarText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<CoinsText>,
            Without<ShieldText>,
            Without<WeaponsText>,
            Without<LevelText>,
        ),
    >,
    mut weapons_query: Query<
        &mut Text,
        (
            With<WeaponsText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<CoinsText>,
            Without<ShieldText>,
            Without<ExpBarText>,
            Without<LevelText>,
        ),
    >,
    mut level_query: Query<
        &mut Text,
        (
            With<LevelText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<CoinsText>,
            Without<ShieldText>,
            Without<ExpBarText>,
            Without<WeaponsText>,
        ),
    >,
) {
    // 更新分数
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("分数: {}", game_data.score);
    }

    // 更新等级
    if let Ok(mut text) = level_query.single_mut() {
        **text = format!("等级: {}", game_data.player_level);
    }

    // 更新生命值 - 使用简单文字代替 emoji
    if let Ok(mut text) = lives_query.single_mut() {
        let max_lives = game_data.max_lives;
        let hearts = "♥".repeat(game_data.lives as usize);
        let empty = "○".repeat(max_lives.saturating_sub(game_data.lives) as usize);
        **text = format!("生命: {}{}", hearts, empty);
    }

    // 更新护盾
    if let Ok(mut text) = shield_query.single_mut() {
        if game_data.max_shield > 0 {
            let shields = "◆".repeat(game_data.shield as usize);
            let empty = "◇".repeat(game_data.max_shield.saturating_sub(game_data.shield) as usize);
            **text = format!("护盾: {}{}", shields, empty);
        } else {
            **text = "护盾: 无".to_string();
        }
    }

    // 更新金币
    if let Ok(mut text) = coins_query.single_mut() {
        **text = format!("金币: {}", game_data.coins);
    }

    // 更新经验条
    if let Ok(mut text) = exp_query.single_mut() {
        let exp_needed = GameData::exp_for_level(game_data.player_level);
        let progress = (game_data.experience as f32 / exp_needed as f32 * 10.0).min(10.0) as usize;
        let filled = "█".repeat(progress);
        let empty = "░".repeat(10 - progress);
        **text = format!(
            "经验: [{}{}] {}/{}",
            filled, empty, game_data.experience, exp_needed
        );
    }

    // 更新武器列表
    if let Ok(mut text) = weapons_query.single_mut() {
        if let Ok(inventory) = player_query.single() {
            if inventory.weapons.is_empty() {
                **text = "武器: 默认子弹".to_string();
            } else {
                let weapons_str: Vec<String> = inventory
                    .weapons
                    .iter()
                    .map(|w| {
                        let name = match w.weapon_type {
                            WeaponType::Shotgun => "霰",
                            WeaponType::Rocket => "导",
                            WeaponType::Laser => "激",
                            WeaponType::Homing => "追",
                            WeaponType::Lightning => "电",
                            WeaponType::Aura => "球",
                            WeaponType::Beam => "波",
                        };
                        format!("{}Lv{}", name, w.level)
                    })
                    .collect();
                **text = format!("武器: {}", weapons_str.join(" "));
            }
        }
    }
}

/// 更新Boss血量HUD
fn update_boss_hud(
    boss_state: Res<BossState>,
    mut boss_hud_query: Query<&mut Node, With<BossHudRoot>>,
    mut boss_name_query: Query<&mut Text, (With<BossNameText>, Without<BossHealthText>)>,
    mut boss_health_query: Query<&mut Text, (With<BossHealthText>, Without<BossNameText>)>,
    mut boss_bar_query: Query<&mut Node, (With<BossHealthBarFill>, Without<BossHudRoot>)>,
) {
    // 显示/隐藏Boss HUD
    if let Ok(mut node) = boss_hud_query.single_mut() {
        node.display = if boss_state.active {
            Display::Flex
        } else {
            Display::None
        };
    }

    if !boss_state.active {
        return;
    }

    // 更新Boss名称
    if let Ok(mut text) = boss_name_query.single_mut() {
        **text = format!("◆ {} ◆", boss_state.boss_name);
    }

    // 更新血量百分比
    let percent = boss_state.health_percent();
    if let Ok(mut text) = boss_health_query.single_mut() {
        **text = format!(
            "{}/{} ({:.1}%)",
            boss_state.current_health.max(0),
            boss_state.total_health.max(1),
            percent
        );
    }

    // 更新血量条宽度
    if let Ok(mut node) = boss_bar_query.single_mut() {
        node.width = Val::Percent(percent);
    }
}

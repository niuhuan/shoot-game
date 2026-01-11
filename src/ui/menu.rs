//! 菜单 UI

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::state::prelude::DespawnOnExit;

use crate::game::GameData;
use crate::game::GameState;
use crate::storage::SaveData;

/// 菜单插件
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)
            .add_systems(
                Update,
                (
                    menu_button_system,
                    menu_upgrade_button_system,
                    menu_keyboard_start,
                    update_menu_stats,
                )
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(
                Update,
                (game_over_button_system,).run_if(in_state(GameState::GameOver)),
            )
            .add_systems(
                Update,
                (pause_button_system, crate::game::handle_unpause_input)
                    .run_if(in_state(GameState::Paused)),
            );
    }
}

/// 菜单 UI 根节点标记
#[derive(Component)]
struct MenuRoot;

/// 游戏结束 UI 根节点标记
#[derive(Component)]
struct GameOverRoot;

/// 暂停 UI 根节点标记
#[derive(Component)]
struct PauseRoot;

/// 按钮类型
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuButton {
    Start,
    Recharge,
    Quit,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum UpgradeButtonKind {
    Hull,
    Shield,
}

#[derive(Component)]
struct UpgradeButton {
    kind: UpgradeButtonKind,
}

#[derive(Component)]
struct HullUpgradeText;

#[derive(Component)]
struct ShieldUpgradeText;

#[derive(Component, Clone, Copy)]
enum GameOverButton {
    Restart,
    Menu,
}

#[derive(Component, Clone, Copy)]
enum PauseButton {
    Resume,
    Menu,
}

/// 菜单金币文本标记
#[derive(Component)]
struct MenuCoinsText;

/// 菜单最高分文本标记
#[derive(Component)]
struct MenuHighScoreText;

/// 字体资源
#[derive(Resource)]
pub struct GameFonts {
    pub main_font: Handle<Font>,
}

/// 设置主菜单
fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>, save_data: Res<SaveData>) {
    log::info!("Setting up Menu UI");
    let font = asset_server.load("NotoSansCJKsc-Regular.otf");

    commands.insert_resource(GameFonts {
        main_font: font.clone(),
    });

    // 根节点
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.95)),
            MenuRoot,
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("几何射击"),
                TextFont {
                    font: font.clone(),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.8, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // 最高分
            parent.spawn((
                Text::new(format!("最高分: {}", save_data.high_score)),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                MenuHighScoreText,
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 金币和打赏按钮（同一行）
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ))
            .with_children(|parent| {
                // 金币数量
                parent.spawn((
                    Text::new(format!("金币: {}", save_data.total_coins)),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.0)),
                    MenuCoinsText,
                    Node {
                        margin: UiRect::right(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // 打赏按钮（文本样式，小字体，带下划线）
                parent.spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    MenuButton::Recharge,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("打赏"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.0, 0.8, 1.0)),
                            ));
                            // underline (avoid relying on combining underline glyphs)
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(2.0),
                                    margin: UiRect::top(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.0, 0.8, 1.0)),
                            ));
                        });
                });
            });

            // 强化系统（金币下方）
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    margin: UiRect::bottom(Val::Px(35.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("强化"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.9, 1.0)),
                    ));

                    // 机身强化
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(upgrade_status_text(
                                    UpgradeButtonKind::Hull,
                                    save_data.hull_upgrade_level,
                                )),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                HullUpgradeText,
                            ));
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(64.0),
                                        height: Val::Px(26.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
                                    BorderColor::all(Color::srgb(0.0, 0.8, 1.0)),
                                    BorderRadius::all(Val::Px(4.0)),
                                    UpgradeButton {
                                        kind: UpgradeButtonKind::Hull,
                                    },
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("升级"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // 护盾强化
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(upgrade_status_text(
                                    UpgradeButtonKind::Shield,
                                    save_data.shield_upgrade_level,
                                )),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                ShieldUpgradeText,
                            ));
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(64.0),
                                        height: Val::Px(26.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
                                    BorderColor::all(Color::srgb(0.0, 0.8, 1.0)),
                                    BorderRadius::all(Val::Px(4.0)),
                                    UpgradeButton {
                                        kind: UpgradeButtonKind::Shield,
                                    },
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("升级"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });

            // 开始按钮
            spawn_button(parent, &font, "开始游戏", MenuButton::Start);
        });
}

/// 创建按钮
fn spawn_button<T: Component + Clone>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    text: &str,
    button_type: T,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                margin: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
            BorderColor::all(Color::srgb(0.0, 0.8, 1.0)),
            BorderRadius::all(Val::Px(5.0)),
            button_type,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn upgrade_cost(current_level: u8) -> Option<u32> {
    match current_level {
        0 => Some(30),
        1 => Some(100),
        _ => None,
    }
}

fn upgrade_status_text(kind: UpgradeButtonKind, current_level: u8) -> String {
    let label = match kind {
        UpgradeButtonKind::Hull => "机身",
        UpgradeButtonKind::Shield => "护盾",
    };
    match upgrade_cost(current_level) {
        Some(c) => format!("{label}：Lv{}/2（{}金币）", current_level, c),
        None => format!("{label}：Lv2/2（已满）"),
    }
}

/// 清理菜单
fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 菜单按钮交互
fn menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    save_data: Res<SaveData>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match button {
                    MenuButton::Start => {
                        *bg_color = BackgroundColor(Color::srgb(0.0, 0.6, 0.8));
                        log::info!("Menu: start pressed");
                        game_data.reset();
                        // 应用强化：机身每级 +1 最大生命；护盾每级 +2 最大护盾
                        game_data.max_lives = 5 + save_data.hull_upgrade_level as u32;
                        game_data.max_shield = 4 + save_data.shield_upgrade_level as u32 * 2;
                        game_data.lives = game_data.lives.min(game_data.max_lives);
                        game_data.shield = game_data.shield.min(game_data.max_shield);
                        next_state.set(GameState::Playing);
                    }
                    MenuButton::Recharge => {
                        log::info!("Menu: recharge pressed");
                        next_state.set(GameState::Recharge);
                    }
                    MenuButton::Quit => {
                        *bg_color = BackgroundColor(Color::srgb(0.0, 0.6, 0.8));
                        // WASM 环境下不能退出
                        #[cfg(not(target_arch = "wasm32"))]
                        std::process::exit(0);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = match button {
                    MenuButton::Recharge => BackgroundColor(Color::NONE),
                    _ => BackgroundColor(Color::srgb(0.2, 0.2, 0.35)),
                };
            }
            Interaction::None => {
                *bg_color = match button {
                    MenuButton::Recharge => BackgroundColor(Color::NONE),
                    _ => BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
                };
            }
        }
    }
}

fn menu_keyboard_start(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        log::info!("Menu: keyboard start");
        game_data.reset();
        next_state.set(GameState::Playing);
    }
}

fn menu_upgrade_button_system(
    mut interaction_query: Query<(&Interaction, &UpgradeButton), Changed<Interaction>>,
    mut save_data: ResMut<SaveData>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let current_level = match button.kind {
            UpgradeButtonKind::Hull => save_data.hull_upgrade_level,
            UpgradeButtonKind::Shield => save_data.shield_upgrade_level,
        };
        let Some(cost) = upgrade_cost(current_level) else {
            continue;
        };
        if save_data.total_coins < cost {
            continue;
        }

        save_data.total_coins -= cost;
        match button.kind {
            UpgradeButtonKind::Hull => save_data.hull_upgrade_level = (current_level + 1).min(2),
            UpgradeButtonKind::Shield => save_data.shield_upgrade_level = (current_level + 1).min(2),
        }

        if let Err(e) = crate::storage::save_game(&save_data) {
            log::error!("Failed to save upgrades: {}", e);
        }
    }
}

/// 设置游戏结束界面
fn setup_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_data: Res<crate::game::GameData>,
) {
    let font = asset_server.load("NotoSansCJKsc-Regular.otf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.9)),
            GameOverRoot,
            DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("游戏结束"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(format!("得分: {}", game_data.score)),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(format!("最高分: {}", game_data.high_score)),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            spawn_button(parent, &font, "重新开始", GameOverButton::Restart);
            spawn_button(parent, &font, "返回菜单", GameOverButton::Menu);
        });
}

fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn game_over_button_system(
    mut interaction_query: Query<
        (&Interaction, &GameOverButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.0, 0.6, 0.8));
                match button {
                    GameOverButton::Restart => {
                        game_data.reset();
                        next_state.set(GameState::Playing);
                    }
                    GameOverButton::Menu => {
                        next_state.set(GameState::Menu);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.35));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.25));
            }
        }
    }
}

/// 设置暂停菜单
fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("NotoSansCJKsc-Regular.otf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseRoot,
            DespawnOnExit(GameState::Paused),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("暂停"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            spawn_button(parent, &font, "继续游戏", PauseButton::Resume);
            spawn_button(parent, &font, "返回菜单", PauseButton::Menu);
        });
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn pause_button_system(
    mut interaction_query: Query<
        (&Interaction, &PauseButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.0, 0.6, 0.8));
                match button {
                    PauseButton::Resume => {
                        next_state.set(GameState::Playing);
                    }
                    PauseButton::Menu => {
                        next_state.set(GameState::Menu);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.35));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.25));
            }
        }
    }
}

fn update_menu_stats(
    save_data: Res<SaveData>,
    mut coins_query: Query<&mut Text, With<MenuCoinsText>>,
    mut high_score_query: Query<&mut Text, (With<MenuHighScoreText>, Without<MenuCoinsText>)>,
    mut hull_query: Query<&mut Text, (With<HullUpgradeText>, Without<MenuCoinsText>)>,
    mut shield_query: Query<&mut Text, (With<ShieldUpgradeText>, Without<MenuCoinsText>)>,
) {
    if let Ok(mut text) = coins_query.single_mut() {
        **text = format!("金币: {}", save_data.total_coins);
    }
    if let Ok(mut text) = high_score_query.single_mut() {
        **text = format!("最高分: {}", save_data.high_score);
    }
    if let Ok(mut text) = hull_query.single_mut() {
        **text = upgrade_status_text(UpgradeButtonKind::Hull, save_data.hull_upgrade_level);
    }
    if let Ok(mut text) = shield_query.single_mut() {
        **text = upgrade_status_text(UpgradeButtonKind::Shield, save_data.shield_upgrade_level);
    }
}

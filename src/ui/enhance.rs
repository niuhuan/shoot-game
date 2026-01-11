//! 强化界面（主菜单中的永久强化）

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::state::prelude::DespawnOnExit;

use crate::game::GameState;
use crate::storage::SaveData;

pub struct EnhancePlugin;

impl Plugin for EnhancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Enhance), setup_enhance_ui)
            .add_systems(OnExit(GameState::Enhance), cleanup_enhance_ui)
            .add_systems(
                Update,
                (enhance_button_system, update_enhance_ui).run_if(in_state(GameState::Enhance)),
            );
    }
}

#[derive(Component)]
struct EnhanceRoot;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum EnhanceButton {
    HullStartUpgrade,
    ShieldStartUpgrade,
    MaxLivesUpgrade,
    MaxShieldUpgrade,
    Back,
}

#[derive(Component)]
struct CoinsText;

#[derive(Component)]
struct HullStartStatusText;

#[derive(Component)]
struct ShieldStartStatusText;

#[derive(Component)]
struct MaxLivesStatusText;

#[derive(Component)]
struct MaxShieldStatusText;

fn start_upgrade_cost(current_level: u8) -> Option<u32> {
    match current_level {
        0 => Some(30),
        1 => Some(100),
        _ => None,
    }
}

fn cap_upgrade_cost(current_level: u8) -> Option<u32> {
    match current_level {
        0 => Some(50),
        _ => None,
    }
}

fn start_upgrade_status_text(label: &str, current_level: u8) -> String {
    match start_upgrade_cost(current_level) {
        Some(c) => format!("{label}：Lv{}/2（{}金币）", current_level, c),
        None => format!("{label}：Lv2/2（已满）"),
    }
}

fn cap_upgrade_status_text(label: &str, current_level: u8) -> String {
    match cap_upgrade_cost(current_level) {
        Some(c) => format!("{label}：Lv{}/1（{}金币）", current_level, c),
        None => format!("{label}：Lv1/1（已满）"),
    }
}

fn setup_enhance_ui(mut commands: Commands, asset_server: Res<AssetServer>, save_data: Res<SaveData>) {
    let font = asset_server.load("NotoSansCJKsc-Regular.otf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.95)),
            EnhanceRoot,
            DespawnOnExit(GameState::Enhance),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("强化"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.8, 1.0)),
            ));

            parent.spawn((
                Text::new(format!("金币: {}", save_data.total_coins)),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
                CoinsText,
            ));

            // 两条强化项
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_upgrade_row(
                        parent,
                        &font,
                        start_upgrade_status_text("初始生命", save_data.hull_upgrade_level),
                        HullStartStatusText,
                        EnhanceButton::HullStartUpgrade,
                    );
                    spawn_upgrade_row(
                        parent,
                        &font,
                        start_upgrade_status_text("初始护盾", save_data.shield_upgrade_level),
                        ShieldStartStatusText,
                        EnhanceButton::ShieldStartUpgrade,
                    );
                    spawn_upgrade_row(
                        parent,
                        &font,
                        cap_upgrade_status_text("生命上限", save_data.max_lives_upgrade_level),
                        MaxLivesStatusText,
                        EnhanceButton::MaxLivesUpgrade,
                    );
                    spawn_upgrade_row(
                        parent,
                        &font,
                        cap_upgrade_status_text("护盾上限", save_data.max_shield_upgrade_level),
                        MaxShieldStatusText,
                        EnhanceButton::MaxShieldUpgrade,
                    );
                });

            // 说明（简短）
            parent.spawn((
                Text::new("初始生命每级 +1；初始护盾每级 +2；生命/护盾上限各可升级 1 次"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node {
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
            ));

            // 返回
            spawn_button(parent, &font, "返回", EnhanceButton::Back);
        });
}

fn spawn_upgrade_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    status_text: String,
    status_marker: impl Component,
    button: EnhanceButton,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(status_text),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                status_marker,
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(72.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
                    BorderColor::all(Color::srgb(0.0, 0.8, 1.0)),
                    BorderRadius::all(Val::Px(6.0)),
                    button,
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
}

fn spawn_button(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, text: &str, button: EnhanceButton) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
            BorderColor::all(Color::srgb(0.0, 0.8, 1.0)),
            BorderRadius::all(Val::Px(5.0)),
            button,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn cleanup_enhance_ui(mut commands: Commands, query: Query<Entity, With<EnhanceRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn enhance_button_system(
    mut interaction_query: Query<(&Interaction, &EnhanceButton, &mut BackgroundColor), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut save_data: ResMut<SaveData>,
) {
    for (interaction, button, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgb(0.0, 0.6, 0.8));
                match button {
                    EnhanceButton::Back => {
                        next_state.set(GameState::Menu);
                    }
                    EnhanceButton::HullStartUpgrade
                    | EnhanceButton::ShieldStartUpgrade
                    | EnhanceButton::MaxLivesUpgrade
                    | EnhanceButton::MaxShieldUpgrade => {
                        let (current_level, cost) = match button {
                            EnhanceButton::HullStartUpgrade => {
                                let lv = save_data.hull_upgrade_level;
                                (lv, start_upgrade_cost(lv))
                            }
                            EnhanceButton::ShieldStartUpgrade => {
                                let lv = save_data.shield_upgrade_level;
                                (lv, start_upgrade_cost(lv))
                            }
                            EnhanceButton::MaxLivesUpgrade => {
                                let lv = save_data.max_lives_upgrade_level;
                                (lv, cap_upgrade_cost(lv))
                            }
                            EnhanceButton::MaxShieldUpgrade => {
                                let lv = save_data.max_shield_upgrade_level;
                                (lv, cap_upgrade_cost(lv))
                            }
                            EnhanceButton::Back => (0, None),
                        };
                        let Some(cost) = cost else {
                            continue;
                        };
                        if save_data.total_coins < cost {
                            continue;
                        }
                        save_data.total_coins -= cost;
                        match button {
                            EnhanceButton::HullStartUpgrade => {
                                save_data.hull_upgrade_level = (current_level + 1).min(2);
                            }
                            EnhanceButton::ShieldStartUpgrade => {
                                save_data.shield_upgrade_level = (current_level + 1).min(2);
                            }
                            EnhanceButton::MaxLivesUpgrade => {
                                save_data.max_lives_upgrade_level = (current_level + 1).min(1);
                            }
                            EnhanceButton::MaxShieldUpgrade => {
                                save_data.max_shield_upgrade_level = (current_level + 1).min(1);
                            }
                            EnhanceButton::Back => {}
                        };

                        if let Err(e) = crate::storage::save_game(&save_data) {
                            log::error!("Failed to save upgrades: {}", e);
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.2, 0.2, 0.35));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.15, 0.15, 0.25));
            }
        }
    }
}

fn update_enhance_ui(
    save_data: Res<SaveData>,
    mut text_set: ParamSet<(
        Query<&mut Text, With<CoinsText>>,
        Query<&mut Text, With<HullStartStatusText>>,
        Query<&mut Text, With<ShieldStartStatusText>>,
        Query<&mut Text, With<MaxLivesStatusText>>,
        Query<&mut Text, With<MaxShieldStatusText>>,
    )>,
) {
    if let Ok(mut t) = text_set.p0().single_mut() {
        **t = format!("金币: {}", save_data.total_coins);
    }
    if let Ok(mut t) = text_set.p1().single_mut() {
        **t = start_upgrade_status_text("初始生命", save_data.hull_upgrade_level);
    }
    if let Ok(mut t) = text_set.p2().single_mut() {
        **t = start_upgrade_status_text("初始护盾", save_data.shield_upgrade_level);
    }
    if let Ok(mut t) = text_set.p3().single_mut() {
        **t = cap_upgrade_status_text("生命上限", save_data.max_lives_upgrade_level);
    }
    if let Ok(mut t) = text_set.p4().single_mut() {
        **t = cap_upgrade_status_text("护盾上限", save_data.max_shield_upgrade_level);
    }
}

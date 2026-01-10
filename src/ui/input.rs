//! 输入处理模块
//! 处理充值界面的文字输入（原生：Bevy UI；Web：HTML 覆盖层）

use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use bevy::ecs::hierarchy::ChildSpawnerCommands;
#[cfg(not(target_arch = "wasm32"))]
use bevy::state::prelude::DespawnOnExit;

use crate::game::GameState;
use crate::storage::{RechargeEvent, RechargeState};
#[cfg(not(target_arch = "wasm32"))]
use crate::storage::RechargeField;

/// 输入插件
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Recharge), setup_recharge_ui)
            .add_systems(OnExit(GameState::Recharge), cleanup_recharge_ui)
            .add_systems(
                Update,
                (
                    handle_recharge_buttons,
                    #[cfg(not(target_arch = "wasm32"))]
                    handle_native_text_input,
                )
                    .run_if(in_state(GameState::Recharge)),
            );
    }
}

/// 充值 UI 根节点
#[derive(Component)]
struct RechargeRoot;

/// 充值按钮类型
#[derive(Component, Clone, Copy)]
enum RechargeButton {
    Submit,
    Cancel,
}

#[derive(Component)]
struct UsernameDisplay;

#[derive(Component)]
struct OrderDisplay;

#[derive(Component)]
struct MessageDisplay;

fn setup_recharge_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Web 环境使用 HTML 覆盖层；这里不生成 Bevy UI，避免双层 UI 混乱
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (commands, asset_server);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let font: Handle<Font> = asset_server.load("NotoSansCJKsc-Regular.otf");

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
                RechargeRoot,
                DespawnOnExit(GameState::Recharge),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("充值中心"),
                    TextFont {
                        font: font.clone(),
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 0.8, 1.0)),
                    Node {
                        margin: UiRect::bottom(Val::Px(25.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Text::new("填写用户名与订单号（Tab 切换，Esc 返回）"),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Text::new("用户名: _"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    UsernameDisplay,
                ));

                parent.spawn((
                    Text::new("订单号: _"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    OrderDisplay,
                ));

                parent.spawn((
                    Text::new(
                        "提示：用户名 3-20 位、字母开头、仅允许字母/数字/_；订单号仅允许字母/数字/-/_，最多 64 位。",
                    ),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    Node {
                        margin: UiRect::bottom(Val::Px(15.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Text::new(""),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    MessageDisplay,
                ));

                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        spawn_recharge_button(parent, &font, "确认", RechargeButton::Submit);
                        spawn_recharge_button(parent, &font, "返回", RechargeButton::Cancel);
                    });
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_recharge_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    text: &str,
    button_type: RechargeButton,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
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
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn cleanup_recharge_ui(mut commands: Commands, query: Query<Entity, With<RechargeRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_recharge_buttons(
    mut interaction_query: Query<
        (&Interaction, &RechargeButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
    recharge_state: Res<RechargeState>,
    mut recharge_events: MessageWriter<RechargeEvent>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.0, 0.6, 0.8));
                match button {
                    RechargeButton::Submit => {
                        recharge_events.write(RechargeEvent {
                            username: recharge_state.username.clone(),
                            order_id: recharge_state.order_id.clone(),
                        });
                    }
                    RechargeButton::Cancel => {
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

#[cfg(not(target_arch = "wasm32"))]
fn handle_native_text_input(
    mut key_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut recharge_state: ResMut<RechargeState>,
    mut texts: ParamSet<(
        Query<&mut Text, With<UsernameDisplay>>,
        Query<&mut Text, With<OrderDisplay>>,
        Query<(&mut Text, &mut TextColor), With<MessageDisplay>>,
    )>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use bevy::input::ButtonState;

    let mut input_changed = false;

    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            KeyCode::Escape => next_state.set(GameState::Menu),
            KeyCode::Tab => {
                recharge_state.active_field = match recharge_state.active_field {
                    RechargeField::Username => RechargeField::OrderId,
                    RechargeField::OrderId => RechargeField::Username,
                };
                input_changed = true;
            }
            KeyCode::Backspace => {
                match recharge_state.active_field {
                    RechargeField::Username => {
                        recharge_state.username.pop();
                    }
                    RechargeField::OrderId => {
                        recharge_state.order_id.pop();
                    }
                }
                input_changed = true;
            }
            _ => {
                let Some(text) = &event.text else { continue };
                match recharge_state.active_field {
                    RechargeField::Username => {
                        for ch in text.chars() {
                            if recharge_state.username.chars().count() >= 20 {
                                break;
                            }
                            if recharge_state.username.is_empty() {
                                if ch.is_ascii_alphabetic() {
                                    recharge_state.username.push(ch);
                                    input_changed = true;
                                }
                            } else if ch.is_ascii_alphanumeric() || ch == '_' {
                                recharge_state.username.push(ch);
                                input_changed = true;
                            }
                        }
                    }
                    RechargeField::OrderId => {
                        for ch in text.chars() {
                            if recharge_state.order_id.chars().count() >= 64 {
                                break;
                            }
                            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                                recharge_state.order_id.push(ch);
                                input_changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if input_changed {
        if let Ok(mut text) = texts.p0().single_mut() {
            let cursor = if recharge_state.active_field == RechargeField::Username {
                "_"
            } else {
                ""
            };
            **text = format!("用户名: {}{}", recharge_state.username, cursor);
        }
        if let Ok(mut text) = texts.p1().single_mut() {
            let cursor = if recharge_state.active_field == RechargeField::OrderId {
                "_"
            } else {
                ""
            };
            **text = format!("订单号: {}{}", recharge_state.order_id, cursor);
        }
    }

    if let Ok((mut text, mut color)) = texts.p2().single_mut() {
        if let Some(msg) = &recharge_state.success_message {
            **text = msg.clone();
            *color = TextColor(Color::srgb(0.3, 1.0, 0.3));
        } else if let Some(msg) = &recharge_state.error_message {
            **text = msg.clone();
            *color = TextColor(Color::srgb(1.0, 0.4, 0.4));
        }
    }
}

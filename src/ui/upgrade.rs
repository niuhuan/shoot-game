//! 升级选择界面

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::{GameData, GameState};
use crate::entities::{Player, WeaponInventory, WeaponType};

/// 升级界面插件
pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_level_up,
                handle_upgrade_selection,
                update_upgrade_ui,
            ).run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_upgrade_ui);
    }
}

/// 升级界面根节点
#[derive(Component)]
struct UpgradeRoot;

/// 升级选项按钮
#[derive(Component)]
struct UpgradeButton {
    choice: UpgradeChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpgradeChoice {
    Weapon { weapon_type: WeaponType, is_new: bool },
    RestoreLives,
    RestoreShield,
}

/// 检查是否升级
fn check_level_up(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    asset_server: Res<AssetServer>,
    player_query: Query<&WeaponInventory, With<Player>>,
    existing_ui: Query<Entity, With<UpgradeRoot>>,
) {
    // 经验与等级提升在 `GameData::add_experience` 内完成；
    // 这里仅在需要升级选择时展示卡牌 UI。
    if !game_data.upgrading {
        return;
    }

    // 已经显示了升级 UI，不重复生成
    if !existing_ui.is_empty() {
        return;
    }
    
    // 获取玩家武器库
    let Ok(inventory) = player_query.single() else {
        game_data.upgrading = false;
        return;
    };
    
    // 获取可选择的武器
    let mut options = get_upgrade_options(&game_data, inventory);
    
    if options.is_empty() {
        // 没有可升级的武器，直接完成升级（后续可扩展为“回血/回盾”等）
        game_data.upgrading = false;
        return;
    }
    
    // 随机选择最多3个选项
    let mut rng = rand::rng();
    options.shuffle(&mut rng);
    options.truncate(3);
    
    // 创建升级选择界面
    spawn_upgrade_ui(&mut commands, &asset_server, inventory, &options);
}

/// 获取可升级的武器选项
fn get_upgrade_options(game_data: &GameData, inventory: &WeaponInventory) -> Vec<UpgradeChoice> {
    let mut options = Vec::new();

    // 所有武器都满级后，只能选择回血/回盾
    if inventory.all_weapons_maxed() {
        if game_data.lives < game_data.max_lives {
            options.push(UpgradeChoice::RestoreLives);
        }
        if game_data.shield < game_data.max_shield {
            options.push(UpgradeChoice::RestoreShield);
        }
        // 如果都满了，就允许依然给两个选项（无效但可选）
        if options.is_empty() {
            options.push(UpgradeChoice::RestoreLives);
            options.push(UpgradeChoice::RestoreShield);
        }
        return options;
    }
    
    // 已有但未满级的武器
    for weapon in &inventory.weapons {
        if weapon.level < 5 {
            options.push(UpgradeChoice::Weapon { weapon_type: weapon.weapon_type, is_new: false });
        }
    }
    
    // 还没有的新武器（如果武器槽未满）
    if inventory.weapons.len() < 5 {
        for wt in WeaponType::all() {
            if inventory.get_weapon(*wt).is_none() {
                options.push(UpgradeChoice::Weapon { weapon_type: *wt, is_new: true });
            }
        }
    }
    
    options
}

/// 获取武器信息
fn get_upgrade_info(choice: UpgradeChoice) -> (&'static str, &'static str, Color) {
    match choice {
        UpgradeChoice::Weapon { weapon_type, .. } => match weapon_type {
            WeaponType::Shotgun => ("霰弹枪", "空心环霰弹\n升级增加弹丸数", Color::srgb(1.0, 0.7, 0.3)),
            WeaponType::Rocket => ("导弹", "命中爆炸范围伤害\n升级增加数量与速度", Color::srgb(1.0, 0.4, 0.4)),
            WeaponType::Laser => ("激光", "细长穿透激光\n升级增加密度", Color::srgb(0.3, 1.0, 0.5)),
            WeaponType::Homing => ("自导导弹", "弧线追踪敌人\n升级增加数量", Color::srgb(0.5, 0.8, 1.0)),
            WeaponType::Lightning => ("闪电链", "跳跃连锁攻击\n升级增加跳跃次数", Color::srgb(0.7, 0.7, 1.0)),
            WeaponType::Aura => ("护身光球", "环绕并抵消子弹\n升级增加光球数", Color::srgb(1.0, 0.9, 0.3)),
            WeaponType::Beam => ("能量波", "月牙形穿透能量波\n升级增加伤害", Color::srgb(0.8, 0.4, 1.0)),
        },
        UpgradeChoice::RestoreLives => ("恢复生命", "立即恢复 1 点生命值", Color::srgb(0.9, 0.3, 0.4)),
        UpgradeChoice::RestoreShield => ("恢复护盾", "立即恢复 2 点护盾值", Color::srgb(0.3, 0.8, 1.0)),
    }
}

/// 生成升级选择界面
fn spawn_upgrade_ui(
    commands: &mut Commands,
    asset_server: &AssetServer,
    inventory: &WeaponInventory,
    options: &[UpgradeChoice],
) {
    let font = asset_server.load("NotoSansCJKsc-Regular.otf");
    
    // 预先收集按钮数据
    let button_data: Vec<_> = options.iter().map(|choice| {
        let (name, desc, color) = get_upgrade_info(*choice);
        let level_text = match choice {
            UpgradeChoice::Weapon { weapon_type, is_new } => {
                let current_level = if *is_new { 0 } else {
                    inventory.get_weapon(*weapon_type).map_or(0, |w| w.level)
                };
                let next_level = current_level + 1;
                if *is_new { "新武器!".to_string() } else { format!("Lv{} → Lv{}", current_level, next_level) }
            }
            UpgradeChoice::RestoreLives => "恢复".to_string(),
            UpgradeChoice::RestoreShield => "恢复".to_string(),
        };
        (*choice, name, desc, color, level_text)
    }).collect();
    
    // 半透明背景遮罩
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        UpgradeRoot,
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("等级提升!"),
            TextFont {
                font: font.clone(),
                font_size: 36.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.9, 0.3)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("选择一项升级:"),
            TextFont {
                font: font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));
        
        // 选项按钮容器
        parent.spawn((
            Node {
                width: Val::Percent(95.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                row_gap: Val::Px(20.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                ..default()
            },
        )).with_children(|button_parent| {
            // 生成每个按钮
            for (choice, name, desc, color, level_text) in &button_data {
                button_parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(140.0),
                        height: Val::Px(190.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(15.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BorderColor::all(*color),
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
                    UpgradeButton { choice: *choice },
                )).with_children(|btn_content| {
                    // 武器名称
                    btn_content.spawn((
                        Text::new(*name),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(*color),
                    ));
                    
                    // 等级变化
                    btn_content.spawn((
                        Text::new(level_text.clone()),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(match choice {
                            UpgradeChoice::Weapon { is_new: true, .. } => Color::srgb(0.3, 1.0, 0.3),
                            _ => Color::srgb(0.7, 0.7, 1.0),
                        }),
                        Node {
                            margin: UiRect::vertical(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                    
                    // 武器描述
                    btn_content.spawn((
                        Text::new(*desc),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });
            }
        });
    });
}

/// 处理升级选择
fn handle_upgrade_selection(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut player_query: Query<&mut WeaponInventory, With<Player>>,
    interaction_query: Query<(&Interaction, &UpgradeButton), Changed<Interaction>>,
    upgrade_ui: Query<Entity, With<UpgradeRoot>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button.choice {
                UpgradeChoice::Weapon { weapon_type, .. } => {
                    if let Ok(mut inventory) = player_query.single_mut() {
                        inventory.add_or_upgrade(weapon_type);
                    }
                }
                UpgradeChoice::RestoreLives => {
                    game_data.heal(1);
                }
                UpgradeChoice::RestoreShield => {
                    game_data.restore_shield(2);
                }
            }
            
            // 关闭升级界面
            game_data.upgrading = false;
            for entity in upgrade_ui.iter() {
                commands.entity(entity).despawn();
            }
            
            break;
        }
    }
}

/// 更新升级界面按钮视觉效果
fn update_upgrade_ui(
    mut query: Query<(&Interaction, &mut BackgroundColor), With<UpgradeButton>>,
) {
    for (interaction, mut bg_color) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.5, 0.9));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.4, 0.9));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9));
            }
        }
    }
}

/// 清理升级界面
fn cleanup_upgrade_ui(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    query: Query<Entity, With<UpgradeRoot>>,
) {
    game_data.upgrading = false;
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

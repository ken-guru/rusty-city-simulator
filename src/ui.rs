use crate::entities::*;
use crate::hovered::HoveredEntity;
use crate::roads::RoadNetwork;
use crate::save::{save_game, SaveRequestEvent};
use crate::time::GameTime;
use crate::world::CityWorld;
use bevy::prelude::*;

#[derive(Component)]
pub struct TimeText;

#[derive(Component)]
pub struct InfoText;

/// Marks the quit confirmation dialog root entity.
#[derive(Component)]
struct QuitDialogRoot;

/// Actions for toolbar buttons.
#[derive(Component, Clone, Debug)]
pub enum ToolbarAction {
    TogglePause,
    SetSpeed(f32),
    Save,
    Quit,
}

/// Actions for buttons inside the quit dialog.
#[derive(Component, Clone, Debug)]
enum QuitDialogAction {
    SaveAndQuit,
    QuitNoSave,
    Cancel,
}

/// Tracks whether the quit confirmation dialog is visible.
#[derive(Resource, Default)]
struct QuitDialogVisible(bool);

/// Pending quit: set to trigger a clean exit at end of frame.
/// `save_first` = true means save synchronously before exiting.
#[derive(Resource, Default)]
struct PendingQuit {
    active: bool,
    save_first: bool,
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuitDialogVisible>()
            .init_resource::<PendingQuit>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    update_time_ui,
                    update_hovered_info,
                    toolbar_interaction,
                    quit_dialog_interaction,
                    sync_quit_dialog_visibility,
                    sync_toolbar_button_states,
                ),
            )
            // Run the actual exit at the very end of the frame so any
            // pending events (e.g. save) have already been dispatched.
            .add_systems(Last, handle_pending_quit);
    }
}

fn setup_ui(mut commands: Commands) {
    // Time + population (top left)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 16.0, ..Default::default() },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                TimeText,
            ));
        });

    // Citizen info on hover (top right)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Hover over a citizen for info"),
                TextFont { font_size: 14.0, ..Default::default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                InfoText,
            ));
        });

    // Legend (bottom left, above toolbar)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(70.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    "* Blue: Male   * Pink: Female\n\
                     # Brown: Home   # Blue: Office   # Yellow: Shop   # Green: Park\n\
                     WASD/Arrows: Pan  |  Right-click drag: Pan\n\
                     Scroll/Pinch: Zoom  |  Space: Pause\n\
                     1/2/3/4: Speed (0.5x/1x/2x/4x)  |  F5: Save",
                ),
                TextFont { font_size: 12.0, ..Default::default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));
        });

    // Toolbar (bottom, full width)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            height: Val::Px(56.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(8.0),
            padding: UiRect::horizontal(Val::Px(12.0)),
            ..Default::default()
        })
        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)))
        .with_children(|parent| {
            toolbar_button(parent, "Pause",   ToolbarAction::TogglePause);
            toolbar_button(parent, "0.5x",    ToolbarAction::SetSpeed(0.5));
            toolbar_button(parent, "1x",      ToolbarAction::SetSpeed(1.0));
            toolbar_button(parent, "2x",      ToolbarAction::SetSpeed(2.0));
            toolbar_button(parent, "4x",      ToolbarAction::SetSpeed(4.0));
            toolbar_button(parent, "Save",    ToolbarAction::Save);
            toolbar_button(parent, "Quit",    ToolbarAction::Quit);
        });

    // Quit confirmation dialog (hidden by default)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ZIndex(100),
            QuitDialogRoot,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(16.0),
                        padding: UiRect::all(Val::Px(32.0)),
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.15, 0.20, 0.97)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|dialog| {
                    dialog.spawn((
                        Text::new("Quit the game?"),
                        TextFont { font_size: 22.0, ..Default::default() },
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                    ));
                    dialog.spawn((
                        Text::new("Any unsaved progress will be lost."),
                        TextFont { font_size: 14.0, ..Default::default() },
                        TextColor(Color::srgb(0.65, 0.65, 0.65)),
                    ));
                    // Button row
                    dialog
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            ..Default::default()
                        })
                        .with_children(|row| {
                            dialog_button(row, "Save & Quit",       QuitDialogAction::SaveAndQuit, Color::srgba(0.15, 0.40, 0.20, 0.95));
                            dialog_button(row, "Quit Without Saving", QuitDialogAction::QuitNoSave, Color::srgba(0.40, 0.15, 0.15, 0.95));
                            dialog_button(row, "Cancel",             QuitDialogAction::Cancel,     Color::srgba(0.20, 0.22, 0.28, 0.95));
                        });
                });
        });
}

fn toolbar_button(parent: &mut ChildBuilder, label: &str, action: ToolbarAction) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BorderRadius::all(Val::Px(6.0)),
            BackgroundColor(Color::srgba(0.18, 0.22, 0.28, 0.9)),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 14.0, ..Default::default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

fn dialog_button(parent: &mut ChildBuilder, label: &str, action: QuitDialogAction, bg: Color) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BorderRadius::all(Val::Px(6.0)),
            BackgroundColor(bg),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 14.0, ..Default::default() },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ));
        });
}

fn toolbar_interaction(
    mut interaction_query: Query<
        (&Interaction, &ToolbarAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_time: ResMut<GameTime>,
    mut save_events: EventWriter<SaveRequestEvent>,
    mut quit_visible: ResMut<QuitDialogVisible>,
) {
    for (interaction, action, mut bg) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.35, 0.45, 0.60, 0.95));
                match action {
                    ToolbarAction::TogglePause => {
                        if game_time.time_scale == 0.0 {
                            game_time.time_scale = 1.0;
                        } else {
                            game_time.time_scale = 0.0;
                        }
                    }
                    ToolbarAction::SetSpeed(s) => {
                        game_time.time_scale = *s;
                    }
                    ToolbarAction::Save => {
                        save_events.send(SaveRequestEvent);
                    }
                    ToolbarAction::Quit => {
                        quit_visible.0 = true;
                    }
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.25, 0.32, 0.42, 0.95));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.18, 0.22, 0.28, 0.9));
            }
        }
    }
}

fn quit_dialog_interaction(
    mut interaction_query: Query<
        (&Interaction, &QuitDialogAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut pending_quit: ResMut<PendingQuit>,
    mut quit_visible: ResMut<QuitDialogVisible>,
) {
    for (interaction, action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                QuitDialogAction::SaveAndQuit => {
                    pending_quit.active = true;
                    pending_quit.save_first = true;
                }
                QuitDialogAction::QuitNoSave => {
                    pending_quit.active = true;
                    pending_quit.save_first = false;
                }
                QuitDialogAction::Cancel => {
                    quit_visible.0 = false;
                }
            }
        }
    }
}

/// Performs the actual quit at the end of the frame.
/// Saves synchronously (if requested) then calls process::exit — this is
/// reliable on macOS where Bevy's AppExit event can deadlock the Metal renderer.
fn handle_pending_quit(
    pending: Res<PendingQuit>,
    world: Res<CityWorld>,
    game_time: Res<GameTime>,
    road_network: Res<RoadNetwork>,
) {
    if !pending.active {
        return;
    }
    if pending.save_first {
        if let Err(e) = save_game(&world, &game_time, &road_network) {
            eprintln!("Failed to save before quit: {e}");
        }
    }
    std::process::exit(0);
}

/// Highlight the active speed / pause button each frame.
fn sync_toolbar_button_states(
    game_time: Res<GameTime>,
    mut button_query: Query<(&ToolbarAction, &mut BackgroundColor, &Interaction), With<Button>>,
) {
    const DEFAULT: Color = Color::srgba(0.18, 0.22, 0.28, 0.9);
    const ACTIVE:  Color = Color::srgba(0.15, 0.55, 0.25, 0.95);
    const HOVERED: Color = Color::srgba(0.25, 0.32, 0.42, 0.95);

    for (action, mut bg, interaction) in &mut button_query {
        let is_active = match action {
            ToolbarAction::TogglePause => game_time.time_scale == 0.0,
            ToolbarAction::SetSpeed(s) => {
                game_time.time_scale != 0.0
                    && (game_time.time_scale - s).abs() < 0.001
            }
            _ => false,
        };

        *bg = if is_active {
            BackgroundColor(ACTIVE)
        } else if *interaction == Interaction::Hovered {
            BackgroundColor(HOVERED)
        } else {
            BackgroundColor(DEFAULT)
        };
    }
}

/// Show or hide the quit dialog based on the `QuitDialogVisible` resource.
fn sync_quit_dialog_visibility(
    quit_visible: Res<QuitDialogVisible>,
    mut dialog_query: Query<&mut Node, With<QuitDialogRoot>>,
) {
    if !quit_visible.is_changed() {
        return;
    }
    for mut node in &mut dialog_query {
        node.display = if quit_visible.0 { Display::Flex } else { Display::None };
    }
}

fn update_time_ui(
    mut text_query: Query<&mut Text, With<TimeText>>,
    game_time: Res<GameTime>,
    world: Res<CityWorld>,
) {
    let Ok(mut text) = text_query.get_single_mut() else { return };

    let day = game_time.current_day() as u32;
    let hour = game_time.current_hour();
    let speed_label = if game_time.time_scale == 0.0 {
        "PAUSED".to_string()
    } else {
        format!("{}x", game_time.time_scale)
    };

    let pop = world.citizens.len();
    let homes = world.buildings.iter().filter(|b| b.building_type == BuildingType::Home).count();

    text.0 = format!(
        "Day {day}  {hour:04.1}h  [{speed_label}]\nPop: {pop}  |  Homes: {homes}"
    );
}

fn update_hovered_info(
    mut text_query: Query<&mut Text, With<InfoText>>,
    hovered: Res<HoveredEntity>,
    citizens: Query<&Citizen>,
) {
    let Ok(mut text) = text_query.get_single_mut() else { return };

    if let Some(entity) = hovered.0 {
        if let Ok(c) = citizens.get(entity) {
            let gender_label = match c.gender {
                Gender::Male   => "M",
                Gender::Female => "F",
            };
            let activity = match c.current_activity {
                ActivityType::Idle         => "Idle",
                ActivityType::Walking      => "Walking",
                ActivityType::Eating       => "Eating",
                ActivityType::Sleeping     => "Sleeping",
                ActivityType::Working      => "Working",
                ActivityType::Socializing  => "Socialising",
                ActivityType::VisitingPark => "At Park",
            };
            text.0 = format!(
                "{} ({}) -- {}\nAge: {:.1}  [{}]\nActivity: {}\n\
                 Hunger:  {:.0}%  Energy: {:.0}%\n\
                 Social:  {:.0}%  Hygiene:{:.0}%",
                c.name,
                gender_label,
                c.get_age_group(),
                c.age,
                c.id.split('-').next().unwrap_or(""),
                activity,
                c.hunger  * 100.0,
                c.energy  * 100.0,
                c.social  * 100.0,
                c.hygiene * 100.0,
            );
            return;
        }
    }
    text.0 = "Hover over a citizen for info".into();
}

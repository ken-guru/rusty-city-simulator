use crate::entities::*;
use crate::hovered::HoveredEntity;
use crate::roads::{ConstructionQueue, RoadNetwork};
use crate::save::{save_game, sync_citizens_to_world, SaveRequestEvent};
use crate::time::GameTime;
use crate::world::CityWorld;
use crate::AppState;
use crate::{ActiveRoute, BuildingSelection};
use bevy::prelude::*;

#[derive(Component)]
pub struct TimeText;

#[derive(Component)]
pub struct InfoText;

/// Marks the citizen tooltip panel root node.
#[derive(Component)]
struct CitizenTooltipPanel;

/// Marks the text inside the citizen tooltip.
#[derive(Component)]
struct CitizenTooltipText;

/// Marks the construction queue panel root node.
#[derive(Component)]
struct QueuePanel;

/// Marks a single interactive row inside the queue panel.
#[derive(Component)]
struct QueueItemRow(pub usize);

/// Tags any entity spawned for construction-queue hover highlights (building outlines + path lines).
#[derive(Component)]
pub struct QueueHighlightMarker;

/// Tags the persistent outline entity shown around the currently selected building.
#[derive(Component)]
pub struct SelectedBuildingHighlightMarker;

/// Which queue row is currently hovered (if any).
#[derive(Resource, Default)]
pub struct HoveredQueueItem(pub Option<usize>);

/// Marks the game toolbar (bottom bar) so it can be hidden on the start screen.
#[derive(Component)]
struct ToolbarRoot;

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
    ReturnToMenu,
    Cancel,
}

/// Tracks whether the quit confirmation dialog is visible.
#[derive(Resource, Default)]
struct QuitDialogVisible(bool);

/// Pending quit: set to trigger a clean exit at end of frame.
/// `save_first` = true means save synchronously before exiting.
/// `return_to_menu` = true means transition to StartScreen instead of process::exit.
#[derive(Resource, Default)]
struct PendingQuit {
    active: bool,
    save_first: bool,
    return_to_menu: bool,
}

#[derive(Component)]
pub struct BuildingInfoPanel;

#[derive(Component)]
pub struct BuildingInfoText;

#[derive(Component)]
pub struct RouteInfoPanel;

#[derive(Component)]
pub struct RouteInfoText;


#[derive(Component, Clone, Debug)]
pub enum BuildingPanelAction {
    Close,
    GetDirections,
}

#[derive(Component, Clone, Debug)]
pub enum RoutePanelAction {
    Close,
    SuggestOptimisation,
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuitDialogVisible>()
            .init_resource::<PendingQuit>()
            .init_resource::<HoveredQueueItem>()
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
                    sync_toolbar_visibility,
                    sync_building_info_panel,
                    sync_route_info_panel,
                    building_panel_interaction,
                    update_citizen_tooltip,
                    rebuild_queue_panel,
                    sync_queue_hover_state,
                    sync_queue_highlight,
                    sync_selected_building_highlight,
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

    // Construction queue panel (top left, below time display) — hidden when queue is empty.
    // Children (interactive rows) are managed dynamically by rebuild_queue_panel.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(48.0),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            display: Display::None,
            ..Default::default()
        },
        BackgroundColor(Color::srgba(0.08, 0.12, 0.18, 0.88)),
        BorderRadius::all(Val::Px(6.0)),
        ZIndex(40),
        QueuePanel,
    ));

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

    // Toolbar (bottom, full width) — hidden on start screen
    commands
        .spawn((
            Node {
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
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            ToolbarRoot,
        ))
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
                            dialog_button(row, "Save & Quit",       QuitDialogAction::SaveAndQuit,   Color::srgba(0.15, 0.40, 0.20, 0.95));
                            dialog_button(row, "Quit Without Saving", QuitDialogAction::QuitNoSave,  Color::srgba(0.40, 0.15, 0.15, 0.95));
                            dialog_button(row, "Return to Menu",    QuitDialogAction::ReturnToMenu,  Color::srgba(0.20, 0.30, 0.45, 0.95));
                            dialog_button(row, "Cancel",            QuitDialogAction::Cancel,        Color::srgba(0.20, 0.22, 0.28, 0.95));
                        });
                });
        });

    // Building info panel (bottom-right, above toolbar)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(70.0),
                width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(14.0)),
                display: Display::None,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.10, 0.13, 0.18, 0.93)),
            BorderRadius::all(Val::Px(8.0)),
            ZIndex(50),
            BuildingInfoPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont { font_size: 13.0, ..Default::default() },
                TextColor(Color::srgb(0.92, 0.92, 0.92)),
                BuildingInfoText,
            ));
            panel.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..Default::default()
            }).with_children(|row| {
                building_panel_button(row, "Close",          BuildingPanelAction::Close);
                building_panel_button(row, "Get Directions", BuildingPanelAction::GetDirections);
            });
        });

    // Route info panel (bottom-right, above building panel)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(240.0),
                width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(14.0)),
                display: Display::None,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.10, 0.14, 0.20, 0.93)),
            BorderRadius::all(Val::Px(8.0)),
            ZIndex(50),
            RouteInfoPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont { font_size: 13.0, ..Default::default() },
                TextColor(Color::srgb(0.92, 0.92, 0.92)),
                RouteInfoText,
            ));
            panel.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..Default::default()
            }).with_children(|row| {
                building_panel_button(row, "Close",                RoutePanelAction::Close);
                building_panel_button(row, "Suggest Optimisation", RoutePanelAction::SuggestOptimisation);
            });
        });

    // Citizen tooltip: small floating panel near cursor, hidden by default.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                display: Display::None,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.90)),
            BorderRadius::all(Val::Px(5.0)),
            ZIndex(80),
            CitizenTooltipPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont { font_size: 12.0, ..Default::default() },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                CitizenTooltipText,
            ));
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

fn building_panel_button<A: Component + Clone>(parent: &mut ChildBuilder, label: &str, action: A) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(7.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BorderRadius::all(Val::Px(5.0)),
            BackgroundColor(Color::srgba(0.20, 0.25, 0.35, 0.9)),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 13.0, ..Default::default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

/// Show the toolbar only when in the InGame state; hide it on the start screen.
fn sync_toolbar_visibility(
    state: Res<State<AppState>>,
    mut toolbar_query: Query<&mut Node, With<ToolbarRoot>>,
) {
    if !state.is_changed() { return; }
    let Ok(mut node) = toolbar_query.get_single_mut() else { return };
    node.display = if *state.get() == AppState::InGame {
        Display::Flex
    } else {
        Display::None
    };
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
                    pending_quit.return_to_menu = false;
                }
                QuitDialogAction::QuitNoSave => {
                    pending_quit.active = true;
                    pending_quit.save_first = false;
                    pending_quit.return_to_menu = false;
                }
                QuitDialogAction::ReturnToMenu => {
                    pending_quit.active = true;
                    pending_quit.save_first = false;
                    pending_quit.return_to_menu = true;
                }
                QuitDialogAction::Cancel => {
                    quit_visible.0 = false;
                }
            }
        }
    }
}

/// Performs the actual quit at the end of the frame.
/// Saves synchronously (if requested) then either transitions to StartScreen
/// or calls process::exit — the latter is reliable on macOS where Bevy's
/// AppExit event can deadlock the Metal renderer.
fn handle_pending_quit(
    mut pending: ResMut<PendingQuit>,
    mut world: ResMut<CityWorld>,
    game_time: Res<GameTime>,
    road_network: Res<RoadNetwork>,
    mut next_state: ResMut<NextState<AppState>>,
    mut quit_visible: ResMut<QuitDialogVisible>,
    citizen_query: Query<&Citizen>,
) {
    if !pending.active {
        return;
    }
    if pending.save_first {
        // Sync live ECS citizen state into world.citizens before serialising.
        let ecs_citizens: Vec<Citizen> = citizen_query.iter().cloned().collect();
        sync_citizens_to_world(&mut world, &ecs_citizens);

        if let Err(e) = save_game(&world, &game_time, &road_network) {
            eprintln!("Failed to save before quit: {e}");
        }
    }
    if pending.return_to_menu {
        // Reset pending state before transitioning so it doesn't fire again.
        pending.active = false;
        pending.save_first = false;
        pending.return_to_menu = false;
        quit_visible.0 = false;
        next_state.set(AppState::StartScreen);
    } else {
        std::process::exit(0);
    }
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

fn sync_building_info_panel(
    selection: Res<BuildingSelection>,
    active_route: Res<ActiveRoute>,
    world: Res<CityWorld>,
    mut panel_query: Query<&mut Node, With<BuildingInfoPanel>>,
    mut text_query: Query<&mut Text, With<BuildingInfoText>>,
) {
    // Hide the building panel while a route is displayed (route panel takes over).
    let visible = selection.selected_id.is_some() && active_route.waypoints.is_empty();
    for mut node in &mut panel_query {
        node.display = if visible { Display::Flex } else { Display::None };
    }
    let Ok(mut text) = text_query.get_single_mut() else { return };
    if let Some(ref id) = selection.selected_id {
        if let Some(b) = world.buildings.iter().find(|b| &b.id == id) {
            let type_label = match b.building_type {
                BuildingType::Home   => "Residence",
                BuildingType::Office => "Office",
                BuildingType::Shop   => "Shop",
                BuildingType::Public => "Public",
            };
            let residents = if !b.resident_ids.is_empty() {
                format!("Residents: {}/{}\n", b.resident_ids.len(), b.capacity_residents)
            } else {
                String::new()
            };
            let workers = if !b.worker_ids.is_empty() {
                format!("Workers: {}/{}\n", b.worker_ids.len(), b.capacity_workers)
            } else {
                String::new()
            };
            let status = if selection.awaiting_direction_pick {
                "\n[Click starting building...]".to_string()
            } else {
                String::new()
            };
            text.0 = format!(
                "{}\nType: {}\nFounded: Day {:.0}\n{}{}{}",
                b.name, type_label, b.founded_day, residents, workers, status,
            );
        }
    } else {
        text.0.clear();
    }
}

fn sync_route_info_panel(
    active_route: Res<ActiveRoute>,
    world: Res<CityWorld>,
    mut panel_query: Query<&mut Node, With<RouteInfoPanel>>,
    mut text_query: Query<&mut Text, With<RouteInfoText>>,
) {
    let visible = !active_route.waypoints.is_empty();
    for mut node in &mut panel_query {
        node.display = if visible { Display::Flex } else { Display::None };
    }
    if !visible {
        return;
    }
    let Ok(mut text) = text_query.get_single_mut() else { return };

    let from_name = active_route.from_id.as_deref()
        .and_then(|id| world.buildings.iter().find(|b| b.id == id))
        .map(|b| b.name.as_str())
        .unwrap_or("?");
    let to_name = active_route.to_id.as_deref()
        .and_then(|id| world.buildings.iter().find(|b| b.id == id))
        .map(|b| b.name.as_str())
        .unwrap_or("?");

    let distance_px: f32 = active_route.waypoints.windows(2)
        .map(|w| (w[1] - w[0]).length())
        .sum();
    let cell_size = crate::grid::CELL_SIZE;
    let distance_units = distance_px / cell_size;
    let travel_secs = distance_px / 60.0;
    let travel_mins = travel_secs / 60.0;

    let notable: Vec<&str> = world.buildings.iter()
        .filter(|b| {
            active_route.from_id.as_deref() != Some(b.id.as_str())
                && active_route.to_id.as_deref() != Some(b.id.as_str())
        })
        .filter(|b| {
            active_route.waypoints.iter().any(|&wp| {
                (b.position - wp).length() < cell_size * 0.75
            })
        })
        .map(|b| b.name.as_str())
        .take(3)
        .collect();

    let landmarks = if notable.is_empty() {
        "None".to_string()
    } else {
        notable.join(", ")
    };

    text.0 = format!(
        "Route: {} -> {}\nDist: {:.1} blocks  Time: {:.1} min\nNear: {}",
        from_name, to_name, distance_units, travel_mins, landmarks
    );
}

fn building_panel_interaction(
    mut building_button_query: Query<
        (&Interaction, &BuildingPanelAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, Without<RoutePanelAction>),
    >,
    mut route_button_query: Query<
        (&Interaction, &RoutePanelAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, Without<BuildingPanelAction>),
    >,
    mut selection: ResMut<BuildingSelection>,
    mut active_route: ResMut<ActiveRoute>,
    mut construction_queue: ResMut<ConstructionQueue>,
    world: Res<CityWorld>,
) {
    for (interaction, action, mut bg) in &mut building_button_query {
        match interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.35, 0.45, 0.60, 0.95));
                match action {
                    BuildingPanelAction::Close => {
                        selection.selected_id = None;
                        selection.awaiting_direction_pick = false;
                        selection.route_from_id = None;
                        active_route.clear_route();
                    }
                    BuildingPanelAction::GetDirections => {
                        selection.awaiting_direction_pick = true;
                    }
                }
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgba(0.28, 0.35, 0.48, 0.95)),
            Interaction::None    => *bg = BackgroundColor(Color::srgba(0.20, 0.25, 0.35, 0.9)),
        }
    }

    for (interaction, action, mut bg) in &mut route_button_query {
        match interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.35, 0.45, 0.60, 0.95));
                match action {
                    RoutePanelAction::Close => {
                        active_route.clear_route();
                    }
                    RoutePanelAction::SuggestOptimisation => {
                        if active_route.waypoints.len() >= 2 {
                            let from_building = active_route.from_id.as_ref()
                                .and_then(|id| world.buildings.iter().find(|b| &b.id == id));
                            let to_building = active_route.to_id.as_ref()
                                .and_then(|id| world.buildings.iter().find(|b| &b.id == id));
                            let from_name = from_building.map(|b| b.name.clone()).unwrap_or_else(|| "?".to_string());
                            let to_name   = to_building.map(|b| b.name.clone()).unwrap_or_else(|| "?".to_string());
                            let from_pos  = from_building.map(|b| b.position).unwrap_or(Vec2::ZERO);
                            let to_pos    = to_building.map(|b| b.position).unwrap_or(Vec2::ZERO);
                            construction_queue.projects.push(crate::roads::ConstructionProject {
                                waypoints: active_route.waypoints.clone(),
                                built_count: 0,
                                created_day: 0.0,
                                label: format!("Player: {} -> {}", from_name, to_name),
                                from_pos,
                                to_pos,
                            });
                        }
                        active_route.clear_route();
                    }
                }
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgba(0.28, 0.35, 0.48, 0.95)),
            Interaction::None    => *bg = BackgroundColor(Color::srgba(0.20, 0.25, 0.35, 0.9)),
        }
    }
}

fn update_citizen_tooltip(
    hovered: Res<HoveredEntity>,
    citizens: Query<&Citizen>,
    world: Res<CityWorld>,
    windows: Query<&Window>,
    mut panel_query: Query<&mut Node, With<CitizenTooltipPanel>>,
    mut text_query: Query<&mut Text, With<CitizenTooltipText>>,
) {
    let Ok(mut panel_node) = panel_query.get_single_mut() else { return };
    let Ok(mut text) = text_query.get_single_mut() else { return };
    let Some(window) = windows.iter().next() else { return };

    if let Some(entity) = hovered.0 {
        if let Ok(c) = citizens.get(entity) {
            // Position tooltip near cursor.
            if let Some(cursor) = window.cursor_position() {
                // Offset so the tooltip doesn't overlap the cursor.
                let x = (cursor.x + 20.0).min(window.width() - 200.0);
                let y = (cursor.y - 40.0).max(0.0);
                panel_node.left = Val::Px(x);
                panel_node.top  = Val::Px(y);
            }
            panel_node.display = Display::Flex;

            // Role label from assigned buildings.
            let role = if let Some(ref home_id) = c.home_building_id {
                if world.buildings.iter().any(|b| &b.id == home_id) {
                    "Resident"
                } else { "Unhoused" }
            } else { "Unhoused" };

            let job = if let Some(ref work_id) = c.workplace_building_id {
                world.buildings.iter()
                    .find(|b| &b.id == work_id)
                    .map(|b| match b.building_type {
                        BuildingType::Office => "office worker",
                        BuildingType::Shop   => "shop worker",
                        _                    => "employed",
                    })
                    .unwrap_or("unemployed")
            } else { "unemployed" };

            let gender_icon = match c.gender { Gender::Male => "M", Gender::Female => "F" };
            text.0 = format!("{} ({gender_icon})\n{role}, {job}", c.name);
            return;
        }
    }

    panel_node.display = Display::None;
}

/// Rebuilds the queue panel children whenever the queue resource changes.
/// Each project gets its own interactive row so hover can be detected.
fn rebuild_queue_panel(
    queue: Res<ConstructionQueue>,
    panel_query: Query<Entity, With<QueuePanel>>,
    mut panel_node_query: Query<&mut Node, With<QueuePanel>>,
    mut commands: Commands,
) {
    if !queue.is_changed() { return; }
    let Ok(panel_entity) = panel_query.get_single() else { return };
    let Ok(mut panel_node) = panel_node_query.get_single_mut() else { return };

    commands.entity(panel_entity).despawn_descendants();

    if queue.projects.is_empty() {
        panel_node.display = Display::None;
        return;
    }

    panel_node.display = Display::Flex;

    commands.entity(panel_entity).with_children(|panel| {
        // Header row
        panel.spawn((
            Text::new(format!("=== Construction Queue ({}) ===", queue.projects.len())),
            TextFont { font_size: 12.0, ..Default::default() },
            TextColor(Color::srgb(0.7, 0.85, 1.0)),
        ));

        for (i, p) in queue.projects.iter().enumerate() {
            let total = p.total_segments();
            let progress = if total == 0 {
                "done".to_string()
            } else {
                format!("{}/{}", p.built_count, total)
            };
            let row_text = format!("  {} [{}]", p.label, progress);

            panel.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                    ..Default::default()
                },
                Button,
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                BorderRadius::all(Val::Px(3.0)),
                Interaction::default(),
                QueueItemRow(i),
            )).with_children(|row| {
                row.spawn((
                    Text::new(row_text),
                    TextFont { font_size: 11.0, ..Default::default() },
                    TextColor(Color::srgb(0.85, 0.92, 0.85)),
                ));
            });
        }
    });
}

/// Reads Interaction on queue rows each frame and updates HoveredQueueItem.
fn sync_queue_hover_state(
    row_query: Query<(&Interaction, &QueueItemRow)>,
    mut hovered: ResMut<HoveredQueueItem>,
) {
    let found = row_query.iter()
        .find(|(i, _)| matches!(i, Interaction::Hovered | Interaction::Pressed))
        .map(|(_, r)| r.0);
    if hovered.0 != found {
        hovered.0 = found;
    }
}

/// Spawns/despawns highlight overlays when the hovered queue item changes.
fn sync_queue_highlight(
    hovered: Res<HoveredQueueItem>,
    queue: Res<ConstructionQueue>,
    road_network: Res<RoadNetwork>,
    buildings: Query<&Building>,
    highlight_entities: Query<Entity, With<QueueHighlightMarker>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut last_idx: Local<Option<usize>>,
) {
    if !hovered.is_changed() { return; }

    // Despawn previous highlights
    for e in &highlight_entities {
        commands.entity(e).despawn();
    }
    *last_idx = hovered.0;

    let Some(idx) = hovered.0 else { return };
    let Some(project) = queue.projects.get(idx) else { return };

    let outline_color   = Color::srgba(1.0, 0.7, 0.1, 0.55);
    let road_path_color = Color::srgba(1.0, 0.82, 0.1, 0.75);
    let planned_color   = Color::srgba(0.1, 0.8, 0.65, 0.75);

    // Highlight the two buildings involved
    for &bpos in &[project.from_pos, project.to_pos] {
        if let Some(b) = buildings.iter().find(|b| (b.position - bpos).length() < 10.0) {
            let sz = b.size + Vec2::splat(8.0);
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(sz.x, sz.y))),
                MeshMaterial2d(materials.add(outline_color)),
                Transform::from_xyz(bpos.x, bpos.y, 0.5),
                QueueHighlightMarker,
            ));
        } else {
            // Fallback if no building entity found at that pos
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(120.0, 120.0))),
                MeshMaterial2d(materials.add(outline_color)),
                Transform::from_xyz(bpos.x, bpos.y, 0.5),
                QueueHighlightMarker,
            ));
        }
    }

    // Current road path (yellow)
    if let Some(path) = road_network.find_road_path(project.from_pos, project.to_pos) {
        let mat = materials.add(road_path_color);
        for window in path.windows(2) {
            let (a, b) = (window[0], window[1]);
            let diff = b - a;
            let length = diff.length();
            if length < 1.0 { continue; }
            let angle = diff.y.atan2(diff.x);
            let mid = (a + b) * 0.5;
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(length, 3.0))),
                MeshMaterial2d(mat.clone()),
                Transform {
                    translation: Vec3::new(mid.x, mid.y, 2.5),
                    rotation: Quat::from_rotation_z(angle),
                    ..Default::default()
                },
                QueueHighlightMarker,
            ));
        }
    }

    // Unbuilt construction waypoints (teal)
    let unbuilt = &project.waypoints[project.built_count..];
    if unbuilt.len() >= 2 {
        let mat = materials.add(planned_color);
        for window in unbuilt.windows(2) {
            let (a, b) = (window[0], window[1]);
            let diff = b - a;
            let length = diff.length();
            if length < 1.0 { continue; }
            let angle = diff.y.atan2(diff.x);
            let mid = (a + b) * 0.5;
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(length, 3.0))),
                MeshMaterial2d(mat.clone()),
                Transform {
                    translation: Vec3::new(mid.x, mid.y, 2.5),
                    rotation: Quat::from_rotation_z(angle),
                    ..Default::default()
                },
                QueueHighlightMarker,
            ));
        }
    }
}

/// Shows a persistent orange outline around the currently selected building.
fn sync_selected_building_highlight(
    selection: Res<BuildingSelection>,
    active_route: Res<ActiveRoute>,
    world: Res<CityWorld>,
    highlight_entities: Query<Entity, With<SelectedBuildingHighlightMarker>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut last_id: Local<Option<String>>,
) {
    // Only update when selection changes
    let current_id = if active_route.waypoints.is_empty() {
        selection.selected_id.clone()
    } else {
        None // hide while route is shown
    };

    if *last_id == current_id { return; }
    *last_id = current_id.clone();

    // Despawn previous highlight
    for e in &highlight_entities {
        commands.entity(e).despawn();
    }

    let Some(id) = current_id else { return };
    let Some(building) = world.buildings.iter().find(|b| b.id == id) else { return };

    let sz = building.size + Vec2::splat(8.0);
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(sz.x, sz.y))),
        MeshMaterial2d(materials.add(Color::srgba(1.0, 0.7, 0.1, 0.55))),
        Transform::from_xyz(building.position.x, building.position.y, 0.5),
        SelectedBuildingHighlightMarker,
    ));
}


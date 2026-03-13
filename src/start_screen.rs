use bevy::prelude::*;
use crate::AppState;
use crate::save::{self, PendingLoad, SaveMeta};

// ─── Colors ──────────────────────────────────────────────────────────────────

const BG_COLOR:   Color = Color::srgb(0.15, 0.15, 0.15);
const BTN_COLOR:  Color = Color::srgb(0.25, 0.25, 0.25);
const BTN_HOVER:  Color = Color::srgb(0.35, 0.35, 0.35);
const TITLE_COLOR: Color = Color::srgb(0.9, 0.85, 0.4);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const WARN_COLOR: Color = Color::srgb(0.9, 0.6, 0.1);
const ERR_COLOR:  Color = Color::srgb(0.9, 0.2, 0.2);

// ─── State ───────────────────────────────────────────────────────────────────

#[derive(Default, Clone)]
enum StartScreenPanel {
    #[default]
    Main,
    SaveList,
    Error(String),
}

#[derive(Resource)]
struct StartScreenState {
    panel: StartScreenPanel,
    dirty: bool,
    saves: Vec<SaveMeta>,
}

impl Default for StartScreenState {
    fn default() -> Self {
        Self { panel: StartScreenPanel::Main, dirty: true, saves: Vec::new() }
    }
}

// ─── Components ──────────────────────────────────────────────────────────────

/// Root node for the entire start-screen UI — despawned on exit.
#[derive(Component)]
struct StartScreenRoot;

/// Button actions on the start screen.
#[derive(Component, Clone, Debug)]
enum StartScreenAction {
    NewGame,
    LoadGame,
    LoadSave(usize),
    Back,
    Quit,
}

// ─── Plugin ──────────────────────────────────────────────────────────────────

pub struct StartScreenPlugin;

impl Plugin for StartScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StartScreenState>()
            .add_systems(OnEnter(AppState::StartScreen), setup_start_screen)
            .add_systems(OnExit(AppState::StartScreen),  cleanup_start_screen)
            .add_systems(
                Update,
                (rebuild_panel, handle_buttons, button_hover)
                    .run_if(in_state(AppState::StartScreen)),
            );
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────────

fn setup_start_screen(
    mut commands: Commands,
    mut state: ResMut<StartScreenState>,
) {
    state.panel = StartScreenPanel::Main;
    state.dirty = true;
    state.saves = Vec::new();

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BG_COLOR),
        StartScreenRoot,
    ));
}

fn cleanup_start_screen(
    mut commands: Commands,
    query: Query<Entity, With<StartScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn rebuild_panel(
    mut commands: Commands,
    mut state: ResMut<StartScreenState>,
    root_query: Query<Entity, With<StartScreenRoot>>,
) {
    if !state.dirty { return; }
    state.dirty = false;

    let Ok(root) = root_query.single() else { return };

    // Clear previous children and rebuild.
    commands.entity(root).despawn_children();

    let panel = state.panel.clone();
    let saves = state.saves.clone();

    commands.entity(root).with_children(|parent| {
        // Title
        parent
            .spawn(Node {
                margin: UiRect::bottom(Val::Px(64.0)),
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    Text::new("City Sim"),
                    TextFont { font_size: 72.0, ..default() },
                    TextColor(TITLE_COLOR),
                ));
            });

        match panel {
            StartScreenPanel::Main => {
                spawn_menu_button(parent, "New Game",  StartScreenAction::NewGame);
                spawn_menu_button(parent, "Load Game", StartScreenAction::LoadGame);
                spawn_menu_button(parent, "Quit",      StartScreenAction::Quit);
            }

            StartScreenPanel::SaveList => {
                if saves.is_empty() {
                    parent
                        .spawn(Node {
                            margin: UiRect::bottom(Val::Px(24.0)),
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn((
                                Text::new("No saves found."),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                } else {
                    for (i, meta) in saves.iter().take(10).enumerate() {
                        let mut label =
                            format!("{}  v{}", meta.display_time, meta.game_version);
                        if !meta.is_current_version    { label.push_str("  [OLD VERSION]"); }
                        if meta.is_known_incompatible   { label.push_str("  [INCOMPATIBLE]"); }

                        let color = if meta.is_known_incompatible {
                            ERR_COLOR
                        } else if !meta.is_current_version {
                            WARN_COLOR
                        } else {
                            TEXT_COLOR
                        };

                        spawn_save_button(parent, &label, i, color);
                    }
                }
                spawn_menu_button(parent, "Back", StartScreenAction::Back);
            }

            StartScreenPanel::Error(msg) => {
                parent
                    .spawn(Node {
                        margin: UiRect::bottom(Val::Px(24.0)),
                        max_width: Val::Px(640.0),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn((
                            Text::new(format!("Error: {msg}")),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(ERR_COLOR),
                        ));
                    });
                spawn_menu_button(parent, "Back", StartScreenAction::Back);
            }
        }
    });
}

fn handle_buttons(
    interaction_query: Query<
        (&Interaction, &StartScreenAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<StartScreenState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending_load: ResMut<PendingLoad>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            StartScreenAction::NewGame => {
                next_state.set(AppState::InGame);
            }
            StartScreenAction::LoadGame => {
                state.saves = save::list_saves();
                state.panel = StartScreenPanel::SaveList;
                state.dirty = true;
            }
            StartScreenAction::LoadSave(index) => {
                if let Some(meta) = state.saves.get(*index) {
                    // Validate before transitioning.
                    match save::load_save(&meta.path) {
                        Ok(_) => {
                            pending_load.0 = Some(meta.path.clone());
                            next_state.set(AppState::InGame);
                        }
                        Err(e) => {
                            save::mark_incompatible(&meta.filename);
                            state.panel = StartScreenPanel::Error(e.to_string());
                            state.dirty = true;
                        }
                    }
                }
            }
            StartScreenAction::Back => {
                state.panel = StartScreenPanel::Main;
                state.dirty = true;
            }
            StartScreenAction::Quit => {
                std::process::exit(0);
            }
        }
    }
}

/// Lighten / darken button background on hover.
fn button_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<StartScreenAction>),
    >,
) {
    for (interaction, mut bg) in &mut query {
        *bg = match interaction {
            Interaction::Hovered => BackgroundColor(BTN_HOVER),
            _                   => BackgroundColor(BTN_COLOR),
        };
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn spawn_menu_button(parent: &mut ChildSpawnerCommands, label: &str, action: StartScreenAction) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(260.0),
                height: Val::Px(56.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(16.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BTN_COLOR),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 22.0, ..default() },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn spawn_save_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    index: usize,
    text_color: Color,
) {
    parent
        .spawn((
            Button,
            Node {
                min_width: Val::Px(560.0),
                height: Val::Px(44.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(BTN_COLOR),
            StartScreenAction::LoadSave(index),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 14.0, ..default() },
                TextColor(text_color),
            ));
        });
}


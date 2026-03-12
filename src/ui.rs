use crate::entities::*;
use crate::hovered::HoveredEntity;
use crate::save::SaveRequestEvent;
use crate::time::GameTime;
use crate::world::CityWorld;
use bevy::prelude::*;

#[derive(Component)]
pub struct TimeText;

#[derive(Component)]
pub struct InfoText;

/// Marks a toolbar button and what it does.
#[derive(Component, Clone, Debug)]
pub enum ToolbarAction {
    TogglePause,
    SetSpeed(f32),
    Save,
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (update_time_ui, update_hovered_info, toolbar_interaction));
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
                    "● Blue: Male   ● Pink: Female\n\
                     ■ Brown: Home   ■ Blue: Office   ■ Yellow: Shop   ■ Green: Park\n\
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
            toolbar_button(parent, "⏸  Pause", ToolbarAction::TogglePause);
            toolbar_button(parent, "0.5×",     ToolbarAction::SetSpeed(0.5));
            toolbar_button(parent, "1×",       ToolbarAction::SetSpeed(1.0));
            toolbar_button(parent, "2×",       ToolbarAction::SetSpeed(2.0));
            toolbar_button(parent, "4×",       ToolbarAction::SetSpeed(4.0));
            toolbar_button(parent, "💾  Save", ToolbarAction::Save);
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

fn toolbar_interaction(
    mut interaction_query: Query<
        (&Interaction, &ToolbarAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_time: ResMut<GameTime>,
    mut save_events: EventWriter<SaveRequestEvent>,
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
            let gender_sym = match c.gender {
                Gender::Male   => "♂",
                Gender::Female => "♀",
            };
            let activity = match c.current_activity {
                ActivityType::Idle         => "🔵 Idle",
                ActivityType::Walking      => "🚶 Walking",
                ActivityType::Eating       => "🍽 Eating",
                ActivityType::Sleeping     => "💤 Sleeping",
                ActivityType::Working      => "💼 Working",
                ActivityType::Socializing  => "💬 Socialising",
                ActivityType::VisitingPark => "🌳 At Park",
            };
            text.0 = format!(
                "{} {} — {}\nAge: {:.1}  ({})\nActivity: {}\n\
                 Hunger:   {:.0}%  Energy: {:.0}%\n\
                 Social:   {:.0}%  Hygiene:{:.0}%",
                c.name,
                gender_sym,
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

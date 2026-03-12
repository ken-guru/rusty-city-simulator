use crate::entities::*;
use crate::hovered::HoveredEntity;
use crate::time::GameTime;
use crate::world::CityWorld;
use bevy::prelude::*;

#[derive(Component)]
pub struct TimeText;

#[derive(Component)]
pub struct InfoText;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (update_time_ui, update_hovered_info));
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

    // Legend (bottom left)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(30.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    "● Blue: Male citizen   ● Pink: Female citizen\n\
                     ■ Brown: Home   ■ Blue: Office   ■ Yellow: Shop\n\
                     WASD/Arrows: Pan  |  Scroll: Zoom  |  Space: Pause\n\
                     1/2/3/4: Speed (0.5x/1x/2x/4x)  |  S: Save",
                ),
                TextFont { font_size: 12.0, ..Default::default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));
        });
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
                ActivityType::Idle        => "🔵 Idle",
                ActivityType::Walking     => "🚶 Walking",
                ActivityType::Eating      => "🍽 Eating",
                ActivityType::Sleeping    => "💤 Sleeping",
                ActivityType::Working     => "💼 Working",
                ActivityType::Socializing => "💬 Socialising",
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

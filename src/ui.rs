use bevy::prelude::*;
use crate::entities::*;
use crate::time::GameTime;
use crate::hovered::HoveredEntity;

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
    // Time display (top left)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                TimeText,
            ));
        });

    // Info display (top right)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                InfoText,
            ));
        });

    // Controls help (bottom left)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("WASD/Arrows: Pan | Scroll: Zoom | Space: Pause\n1/2/3/4: Speed Control | S: Save"),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn update_time_ui(
    mut text_query: Query<&mut Text, With<TimeText>>,
    game_time: Res<GameTime>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        let day = game_time.current_day();
        let hour = game_time.current_hour();
        let speed_str = if game_time.time_scale == 0.0 {
            "PAUSED".to_string()
        } else {
            format!("{}x", game_time.time_scale)
        };
        text.0 = format!("Day: {:.1} | Hour: {:.1} | Speed: {}", day, hour, speed_str);
    }
}

fn update_hovered_info(
    mut text_query: Query<&mut Text, With<InfoText>>,
    hovered: Res<HoveredEntity>,
    citizens: Query<&Citizen>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        if let Some(entity) = hovered.0 {
            if let Ok(citizen) = citizens.get(entity) {
                text.0 = format!(
                    "{} ({})\nAge: {:.1}\nHunger: {:.0}% | Energy: {:.0}%\nSocial: {:.0}% | Hygiene: {:.0}%",
                    citizen.name,
                    citizen.get_age_group(),
                    citizen.age,
                    citizen.hunger * 100.0,
                    citizen.energy * 100.0,
                    citizen.social * 100.0,
                    citizen.hygiene * 100.0,
                );
            } else {
                text.0 = String::new();
            }
        } else {
            text.0 = String::from("Hover over a citizen for info");
        }
    }
}

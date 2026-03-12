use bevy::prelude::*;
use crate::entities::*;
use crate::time::GameTime;

#[derive(Component)]
pub struct InfoText;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, update_ui);
    }
}

fn setup_ui(mut commands: Commands) {
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
                    font_size: 14.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                InfoText,
            ));
        });
}

fn update_ui(
    mut text_query: Query<&mut Text, With<InfoText>>,
    game_time: Res<GameTime>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        let day = game_time.current_day();
        let hour = game_time.current_hour();
        text.0 = format!("Day: {:.1} | Hour: {:.1}", day, hour);
    }
}

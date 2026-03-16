use bevy::prelude::*;
use std::collections::VecDeque;
use rand::RngExt;

use crate::AppState;
use crate::economy::DebugMode;
use crate::time::GameTime;

/// Message fired when the player selects an option in the event modal.
#[derive(Message, Clone)]
pub struct EventOptionChosen {
    pub consequence: EventConsequence,
}

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RandomEventQueue>()
           .init_resource::<EventModalState>()
           .add_message::<EventOptionChosen>()
           .add_systems(Update, (
               spawn_random_events,
               check_trigger_random_event,
               auto_resolve_event_modal,
               apply_event_consequences,
           ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventConsequence {
    pub balance_delta: f32,
    pub happiness_delta: f32,
    pub happiness_duration_days: f32,
    pub citizen_delta: i32,
    /// Reserved: future events can spawn a building of this type.
    #[allow(dead_code)]
    pub building_to_spawn: Option<crate::entities::BuildingType>,
    /// Reserved: future events can destroy a specific building by id.
    #[allow(dead_code)]
    pub building_to_destroy: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EventOption {
    pub label: String,
    pub consequence: EventConsequence,
}

#[derive(Clone, Debug)]
pub struct CityEvent {
    pub title: String,
    pub description: String,
    pub options: Vec<EventOption>,
}

#[derive(Resource)]
pub struct RandomEventQueue {
    pub pending: VecDeque<CityEvent>,
    pub cooldown_days: f32,
}

impl Default for RandomEventQueue {
    fn default() -> Self {
        Self {
            pending: VecDeque::new(),
            // Start the first event after 20 game-days so the city has time to
            // grow before being interrupted by modal dialogs.
            cooldown_days: 20.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct EventModalState {
    pub active_event: Option<CityEvent>,
    /// Real-time timestamp (seconds since startup) when the modal was opened.
    /// Used to auto-resolve the dialog if the player is idle.
    pub opened_at_real_secs: Option<f32>,
    /// Reserved for future hover highlighting of option buttons.
    #[allow(dead_code)]
    pub hovered_option: Option<usize>,
}

/// After this many real-world seconds with no response, auto-select option 0.
const AUTO_RESOLVE_SECS: f32 = 60.0;

impl RandomEventQueue {
    pub const COOLDOWN_MIN: f32 = 10.0;
    pub const COOLDOWN_MAX: f32 = 30.0;
    
    pub fn trigger_random_event(&mut self) -> Option<CityEvent> {
        self.pending.pop_front()
    }
    
    pub fn queue_event(&mut self, event: CityEvent) {
        self.pending.push_back(event);
    }
}

fn spawn_random_events(
    mut event_queue: ResMut<RandomEventQueue>,
    _time: Res<GameTime>,
) {
    if event_queue.pending.is_empty() && event_queue.cooldown_days <= 0.0 {
        let mut rng = rand::rng();
        let events = create_random_events();
        let event_idx = rng.random_range(0..events.len());
        event_queue.queue_event(events[event_idx].clone());
        event_queue.cooldown_days = rng.random_range(RandomEventQueue::COOLDOWN_MIN..RandomEventQueue::COOLDOWN_MAX);
    }
}

fn check_trigger_random_event(
    mut event_queue: ResMut<RandomEventQueue>,
    mut modal_state: ResMut<EventModalState>,
    game_time: Res<GameTime>,
    real_time: Res<Time>,
    policies: Res<crate::policies::ActivePolicies>,
    debug: Res<DebugMode>,
) {
    if modal_state.active_event.is_some() {
        return;
    }

    // Advance cooldown using real elapsed time scaled by game speed
    let delta_days = real_time.delta_secs() * game_time.time_scale / game_time.day_length_secs;
    // Open City policy shortens the cooldown, making events arrive sooner
    event_queue.cooldown_days -= delta_days * policies.migration_frequency_multiplier();

    if event_queue.cooldown_days <= 0.0 {
        if let Some(event) = event_queue.trigger_random_event() {
            crate::economy::log_event_modal(&debug, &event.title, game_time.current_day());
            modal_state.opened_at_real_secs = Some(real_time.elapsed_secs());
            modal_state.active_event = Some(event);
            let mut rng = rand::rng();
            event_queue.cooldown_days = rng.random_range(RandomEventQueue::COOLDOWN_MIN..RandomEventQueue::COOLDOWN_MAX);
        }
    }
}

/// If a modal has been open for longer than AUTO_RESOLVE_SECS of real time with no
/// player response, automatically choose option 0 (the first, typically "safe" choice).
fn auto_resolve_event_modal(
    mut modal_state: ResMut<EventModalState>,
    mut chosen: MessageWriter<EventOptionChosen>,
    real_time: Res<Time>,
    mut news: ResMut<crate::news::CityNewsLog>,
    game_time: Res<GameTime>,
    debug: Res<DebugMode>,
) {
    let Some(opened_at) = modal_state.opened_at_real_secs else { return };
    let elapsed = real_time.elapsed_secs() - opened_at;
    if elapsed < AUTO_RESOLVE_SECS { return; }

    let Some(ref event) = modal_state.active_event.clone() else { return };
    if let Some(first_option) = event.options.first() {
        chosen.write(EventOptionChosen { consequence: first_option.consequence.clone() });
        crate::economy::log_event_resolved(
            &debug, &event.title, &first_option.label, true, game_time.current_day()
        );
        news.push(
            game_time.current_day(),
            "⏱",
            format!("\"{}\" auto-resolved: \"{}\"", event.title, first_option.label),
        );
    }
    modal_state.active_event = None;
    modal_state.opened_at_real_secs = None;
}

pub fn create_random_events() -> Vec<CityEvent> {
    vec![
        CityEvent {
            title: "New Residents Arriving".to_string(),
            description: "A family of 4 wants to move to your city. Will you welcome them?".to_string(),
            options: vec![
                EventOption {
                    label: "Welcome them (+4 citizens, −$8,000)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -8000.0,
                        citizen_delta: 4,
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Politely decline".to_string(),
                    consequence: EventConsequence::default(),
                },
            ],
        },
        CityEvent {
            title: "Business Opportunity".to_string(),
            description: "An entrepreneur wants to open a new shop in your city.".to_string(),
            options: vec![
                EventOption {
                    label: "Fund the shop (−$5,000)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -5000.0,
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Let them find their own funding".to_string(),
                    consequence: EventConsequence::default(),
                },
            ],
        },
        CityEvent {
            title: "Community Festival".to_string(),
            description: "Citizens are requesting a community festival to improve morale.".to_string(),
            options: vec![
                EventOption {
                    label: "Host the festival (−$3,000, +20 happiness for 10 days)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -3000.0,
                        happiness_delta: 0.2,
                        happiness_duration_days: 10.0,
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Skip it, keep the budget".to_string(),
                    consequence: EventConsequence::default(),
                },
            ],
        },
        CityEvent {
            title: "Market District Fire!".to_string(),
            description: "A fire has broken out in the market district. You must act quickly!".to_string(),
            options: vec![
                EventOption {
                    label: "Fund firefighting (−$4,000, save the shops)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -4000.0,
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Let it burn (save budget, but shops are lost)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: 0.0,
                        ..Default::default()
                    },
                },
            ],
        },
    ]
}

/// Applies the consequence of a chosen event option.
fn apply_event_consequences(
    mut events: MessageReader<EventOptionChosen>,
    mut economy: ResMut<crate::economy::Economy>,
    mut city_happiness: ResMut<crate::happiness::CityHappiness>,
    game_time: Res<GameTime>,
    mut news: ResMut<crate::news::CityNewsLog>,
    mut spawn_immigrants: MessageWriter<crate::reproduction::SpawnImmigrantsMessage>,
) {
    for chosen in events.read() {
        let c = &chosen.consequence;

        if c.balance_delta != 0.0 {
            economy.balance += c.balance_delta;
            let sign = if c.balance_delta > 0.0 { "+" } else { "" };
            news.push(game_time.current_day(), "💰", format!("City event: balance changed by {sign}${:.0}", c.balance_delta));
        }

        if c.happiness_delta != 0.0 && c.happiness_duration_days > 0.0 {
            city_happiness.apply_boost(c.happiness_delta, c.happiness_duration_days, game_time.current_day());
            let sign = if c.happiness_delta > 0.0 { "+" } else { "" };
            news.push(game_time.current_day(), "😊", format!("City happiness {sign}{:.0}% for {:.0} days", c.happiness_delta * 100.0, c.happiness_duration_days));
        }

        if c.citizen_delta > 0 {
            spawn_immigrants.write(crate::reproduction::SpawnImmigrantsMessage { count: c.citizen_delta as u32 });
        } else if c.citizen_delta < 0 {
            news.push(game_time.current_day(), "👤", format!("{} citizens left the city.", -c.citizen_delta));
        }
    }
}

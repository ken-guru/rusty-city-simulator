use bevy::prelude::*;
use std::collections::VecDeque;
use rand::RngExt;

use crate::AppState;
use crate::time::GameTime;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RandomEventQueue>()
           .init_resource::<EventModalState>()
           .add_systems(Update, (
               spawn_random_events,
               check_trigger_random_event,
           ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventConsequence {
    pub balance_delta: f32,
    pub happiness_delta: f32,
    pub happiness_duration_days: f32,
    pub citizen_delta: i32,
    pub building_to_spawn: Option<crate::entities::BuildingType>,
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

#[derive(Resource, Default)]
pub struct RandomEventQueue {
    pub pending: VecDeque<CityEvent>,
    pub cooldown_days: f32,
}

#[derive(Resource, Default)]
pub struct EventModalState {
    pub active_event: Option<CityEvent>,
    pub hovered_option: Option<usize>,
}

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
    _time: Res<GameTime>,
) {
    if modal_state.active_event.is_some() {
        return;
    }
    
    event_queue.cooldown_days -= 0.01;
    
    if event_queue.cooldown_days <= 0.0 {
        if let Some(event) = event_queue.trigger_random_event() {
            modal_state.active_event = Some(event);
            let mut rng = rand::rng();
            event_queue.cooldown_days = rng.random_range(RandomEventQueue::COOLDOWN_MIN..RandomEventQueue::COOLDOWN_MAX);
        }
    }
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

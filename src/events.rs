//! Random city events system: event catalogue, `RandomEventQueue`,
//! `EventModalState`, and the system that triggers and resolves modal dialogs
//! mid-game.

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
    /// When set, a building of this type will be auto-placed in the city.
    pub building_to_spawn: Option<crate::entities::BuildingType>,
    /// When set, a random building of this type will be demolished.
    pub building_type_to_destroy: Option<crate::entities::BuildingType>,
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
    /// Accumulated real-world seconds since the modal was opened.
    /// Incremented each frame using `Time<Real>` so it is unaffected by
    /// game speed, pausing, or virtual-time scaling.
    pub open_duration_secs: f32,
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
    citizens: Query<&crate::entities::Citizen>,
    world: Res<crate::world::CityWorld>,
    economy: Res<crate::economy::Economy>,
) {
    if event_queue.pending.is_empty() && event_queue.cooldown_days <= 0.0 {
        let mut rng = rand::rng();
        let pop = citizens.iter().count();
        let park_count = world.park_cells.len();
        let balance = economy.balance;
        let events = city_events(pop, park_count, balance);
        if !events.is_empty() {
            let event_idx = rng.random_range(0..events.len());
            event_queue.queue_event(events[event_idx].clone());
            event_queue.cooldown_days = rng.random_range(RandomEventQueue::COOLDOWN_MIN..RandomEventQueue::COOLDOWN_MAX);
        }
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
            modal_state.open_duration_secs = 0.0;
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
    real_time: Res<Time<bevy::time::Real>>,
    mut news: ResMut<crate::news::CityNewsLog>,
    game_time: Res<GameTime>,
    debug: Res<DebugMode>,
) {
    if modal_state.active_event.is_none() { return; }

    modal_state.open_duration_secs += real_time.delta_secs();
    if modal_state.open_duration_secs < AUTO_RESOLVE_SECS { return; }

    let Some(ref event) = modal_state.active_event.clone() else { return };
    if let Some(first_option) = event.options.first() {
        chosen.write(EventOptionChosen { consequence: first_option.consequence.clone() });
        crate::economy::log_event_resolved(
            &debug, &event.title, &first_option.label, true, game_time.current_day()
        );
        news.push(
            game_time.current_day(),
            "t",
            format!("\"{}\" auto-resolved: \"{}\"", event.title, first_option.label),
        );
    }
    modal_state.active_event = None;
    modal_state.open_duration_secs = 0.0;
}

/// Returns the catalogue of eligible city events given the current city state.
/// Events with city-state conditions are only included when applicable.
pub fn city_events(pop: usize, park_count: usize, balance: f32) -> Vec<CityEvent> {
    use crate::entities::BuildingType;

    let mut events = vec![
        // ── Always available ─────────────────────────────────────────────────
        CityEvent {
            title: "New Residents Arriving".to_string(),
            description: "A family of 4 wants to move to your city. Will you welcome them?".to_string(),
            options: vec![
                EventOption {
                    label: "Welcome them (+4 citizens, -$8,000)".to_string(),
                    consequence: EventConsequence { balance_delta: -8000.0, citizen_delta: 4, ..Default::default() },
                },
                EventOption {
                    label: "Politely decline".to_string(),
                    consequence: EventConsequence::default(),
                },
            ],
        },
        CityEvent {
            title: "Business Opportunity".to_string(),
            description: "An entrepreneur wants to open a new shop. City funding would help them get started.".to_string(),
            options: vec![
                EventOption {
                    label: "Fund the shop (-$5,000, new shop built)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -5000.0,
                        building_to_spawn: Some(BuildingType::Shop),
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
                    label: "Host the festival (-$3,000, +20% happiness for 10 days)".to_string(),
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
                    label: "Fund firefighting (-$4,000, control the fire)".to_string(),
                    consequence: EventConsequence { balance_delta: -4000.0, ..Default::default() },
                },
                EventOption {
                    label: "Let it burn (save money, a shop is lost)".to_string(),
                    consequence: EventConsequence {
                        building_type_to_destroy: Some(BuildingType::Shop),
                        ..Default::default()
                    },
                },
            ],
        },
        CityEvent {
            title: "Heat Wave".to_string(),
            description: "A prolonged heat wave is exhausting citizens. They need cooling support.".to_string(),
            options: vec![
                EventOption {
                    label: "Open cooling centres (-$2,500)".to_string(),
                    consequence: EventConsequence { balance_delta: -2500.0, ..Default::default() },
                },
                EventOption {
                    label: "Issue guidance only (citizens suffer, -15% happiness for 8 days)".to_string(),
                    consequence: EventConsequence {
                        happiness_delta: -0.15,
                        happiness_duration_days: 8.0,
                        ..Default::default()
                    },
                },
            ],
        },
        CityEvent {
            title: "Volunteer Day".to_string(),
            description: "Citizens have organised a volunteer clean-up day, boosting city pride.".to_string(),
            options: vec![
                EventOption {
                    label: "Endorse it (free! +15% happiness for 6 days)".to_string(),
                    consequence: EventConsequence {
                        happiness_delta: 0.15,
                        happiness_duration_days: 6.0,
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Stay neutral (no effect)".to_string(),
                    consequence: EventConsequence::default(),
                },
            ],
        },
        CityEvent {
            title: "Infrastructure Collapse".to_string(),
            description: "Aging pipes have burst under the main street. Repairs are unavoidable.".to_string(),
            options: vec![
                EventOption {
                    label: "Full emergency repair (-$15,000)".to_string(),
                    consequence: EventConsequence { balance_delta: -15000.0, ..Default::default() },
                },
                EventOption {
                    label: "Temporary patch (-$6,000, -15% happiness for 7 days)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -6000.0,
                        happiness_delta: -0.15,
                        happiness_duration_days: 7.0,
                        ..Default::default()
                    },
                },
            ],
        },
        CityEvent {
            title: "Unexpected Windfall".to_string(),
            description: "A regional grant has been awarded to your city.".to_string(),
            options: vec![
                EventOption {
                    label: "Take the cash (+$12,000)".to_string(),
                    consequence: EventConsequence { balance_delta: 12000.0, ..Default::default() },
                },
                EventOption {
                    label: "Invest in community (+$8,000, +10% happiness for 5 days)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: 8000.0,
                        happiness_delta: 0.1,
                        happiness_duration_days: 5.0,
                        ..Default::default()
                    },
                },
            ],
        },
        CityEvent {
            title: "Power Outage".to_string(),
            description: "A substation failure has knocked out power. Citizens are frustrated.".to_string(),
            options: vec![
                EventOption {
                    label: "Emergency contractor (-$4,000, power restored quickly)".to_string(),
                    consequence: EventConsequence { balance_delta: -4000.0, ..Default::default() },
                },
                EventOption {
                    label: "In-house fix (-$1,500 but -10% happiness for 4 days)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -1500.0,
                        happiness_delta: -0.1,
                        happiness_duration_days: 4.0,
                        ..Default::default()
                    },
                },
            ],
        },
        CityEvent {
            title: "Emergency Housing Project".to_string(),
            description: "Demand for housing has spiked. The city can intervene to build emergency accommodation.".to_string(),
            options: vec![
                EventOption {
                    label: "Build emergency residence (-$8,000, new home built)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -8000.0,
                        building_to_spawn: Some(BuildingType::Home),
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Leave it to the market".to_string(),
                    consequence: EventConsequence::default(),
                },
            ],
        },
    ];

    // ── Conditional on population > 20 ───────────────────────────────────────
    if pop > 20 {
        events.push(CityEvent {
            title: "Tech Investment".to_string(),
            description: "A tech firm wants to open a local office, but needs city backing.".to_string(),
            options: vec![
                EventOption {
                    label: "Back the firm (-$10,000, new office built)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -10000.0,
                        building_to_spawn: Some(BuildingType::Office),
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Decline".to_string(),
                    consequence: EventConsequence::default(),
                },
            ],
        });
    }

    // ── Conditional on population > 30 ───────────────────────────────────────
    if pop > 30 {
        events.push(CityEvent {
            title: "Disease Outbreak".to_string(),
            description: "A contagious illness is spreading through the city. Without intervention, citizens will leave.".to_string(),
            options: vec![
                EventOption {
                    label: "Fund containment (-$5,000)".to_string(),
                    consequence: EventConsequence { balance_delta: -5000.0, ..Default::default() },
                },
                EventOption {
                    label: "Rely on natural immunity (3 citizens leave)".to_string(),
                    consequence: EventConsequence { citizen_delta: -3, ..Default::default() },
                },
            ],
        });
    }

    // ── Conditional on population > 50 ───────────────────────────────────────
    if pop > 50 {
        events.push(CityEvent {
            title: "Emigration Wave".to_string(),
            description: "Dissatisfied residents are talking about leaving. An outreach programme could change their minds.".to_string(),
            options: vec![
                EventOption {
                    label: "Fund outreach (-$15,000, no one leaves)".to_string(),
                    consequence: EventConsequence { balance_delta: -15000.0, ..Default::default() },
                },
                EventOption {
                    label: "Let them go (5 citizens leave)".to_string(),
                    consequence: EventConsequence { citizen_delta: -5, ..Default::default() },
                },
            ],
        });
    }

    // ── Conditional on healthy balance ───────────────────────────────────────
    if balance > 25000.0 {
        events.push(CityEvent {
            title: "Economic Recession".to_string(),
            description: "Regional economic conditions have deteriorated. The city budget will take a hit regardless.".to_string(),
            options: vec![
                EventOption {
                    label: "Draw down reserves (-$18,000, citizens reassured)".to_string(),
                    consequence: EventConsequence { balance_delta: -18000.0, ..Default::default() },
                },
                EventOption {
                    label: "Austerity measures (-$10,000, -10% happiness for 12 days)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -10000.0,
                        happiness_delta: -0.1,
                        happiness_duration_days: 12.0,
                        ..Default::default()
                    },
                },
            ],
        });
    }

    // ── Conditional on parks existing ────────────────────────────────────────
    if park_count > 0 {
        events.push(CityEvent {
            title: "Drought".to_string(),
            description: "A dry spell has damaged the parks and stressed outdoor workers.".to_string(),
            options: vec![
                EventOption {
                    label: "Emergency irrigation (-$8,000, parks recover)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -8000.0,
                        happiness_delta: -0.05,
                        happiness_duration_days: 5.0,
                        ..Default::default()
                    },
                },
                EventOption {
                    label: "Wait it out (-$3,000, -15% happiness for 10 days)".to_string(),
                    consequence: EventConsequence {
                        balance_delta: -3000.0,
                        happiness_delta: -0.15,
                        happiness_duration_days: 10.0,
                        ..Default::default()
                    },
                },
            ],
        });
    }

    events
}


/// Applies the consequence of a chosen event option.
fn apply_event_consequences(
    mut events: MessageReader<EventOptionChosen>,
    mut economy: ResMut<crate::economy::Economy>,
    mut city_happiness: ResMut<crate::happiness::CityHappiness>,
    game_time: Res<GameTime>,
    mut news: ResMut<crate::news::CityNewsLog>,
    mut spawn_immigrants: MessageWriter<crate::reproduction::SpawnImmigrantsMessage>,
    mut remove_citizens: MessageWriter<crate::reproduction::RemoveCitizensMessage>,
    mut spawn_building: MessageWriter<crate::housing::SpawnBuildingRequest>,
    mut demolish_building: MessageWriter<crate::housing::DemolishRandomBuildingRequest>,
) {
    for chosen in events.read() {
        let c = &chosen.consequence;

        if c.balance_delta != 0.0 {
            economy.balance += c.balance_delta;
            let sign = if c.balance_delta > 0.0 { "+" } else { "" };
            news.push(game_time.current_day(), "$", format!("City event: balance changed by {sign}${:.0}", c.balance_delta));
        }

        if c.happiness_delta != 0.0 && c.happiness_duration_days > 0.0 {
            city_happiness.apply_boost(c.happiness_delta, c.happiness_duration_days, game_time.current_day());
            let sign = if c.happiness_delta > 0.0 { "+" } else { "" };
            news.push(game_time.current_day(), "s", format!("City happiness {sign}{:.0}% for {:.0} days", c.happiness_delta * 100.0, c.happiness_duration_days));
        }

        if c.citizen_delta > 0 {
            spawn_immigrants.write(crate::reproduction::SpawnImmigrantsMessage { count: c.citizen_delta as u32 });
        } else if c.citizen_delta < 0 {
            remove_citizens.write(crate::reproduction::RemoveCitizensMessage { count: (-c.citizen_delta) as u32 });
        }

        if let Some(kind) = c.building_to_spawn {
            spawn_building.write(crate::housing::SpawnBuildingRequest { kind });
        }

        if let Some(kind) = c.building_type_to_destroy {
            demolish_building.write(crate::housing::DemolishRandomBuildingRequest { kind });
        }
    }
}

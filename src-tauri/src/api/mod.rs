pub mod client;
pub mod endpoints;

pub use client::{CreditInfo, OddsApiClient, BASE_URL};
pub use endpoints::{
    get_event_odds, get_events, get_odds, get_props_for_events, get_scores, get_sports, OddsQuery,
};

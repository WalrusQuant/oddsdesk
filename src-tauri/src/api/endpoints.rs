use crate::api::client::OddsApiClient;
use crate::errors::AppResult;
use crate::models::{Event, Score, Sport};
use futures::stream::{self, StreamExt};

#[derive(Debug, Clone)]
pub struct OddsQuery<'a> {
    pub regions: &'a str,
    pub markets: &'a str,
    pub odds_format: &'a str,
    pub bookmakers: Option<&'a [String]>,
}

impl<'a> Default for OddsQuery<'a> {
    fn default() -> Self {
        Self {
            regions: "us",
            markets: "h2h,spreads,totals",
            odds_format: "american",
            bookmakers: None,
        }
    }
}

impl<'a> OddsQuery<'a> {
    fn to_params(&self) -> Vec<(&'static str, String)> {
        let mut v = vec![
            ("regions", self.regions.to_string()),
            ("markets", self.markets.to_string()),
            ("oddsFormat", self.odds_format.to_string()),
        ];
        if let Some(books) = self.bookmakers {
            if !books.is_empty() {
                v.push(("bookmakers", books.join(",")));
            }
        }
        v
    }
}

pub async fn get_sports(client: &OddsApiClient) -> AppResult<Vec<Sport>> {
    client.get("/sports", &[]).await
}

pub async fn get_odds(
    client: &OddsApiClient,
    sport: &str,
    q: OddsQuery<'_>,
) -> AppResult<Vec<Event>> {
    let path = format!("/sports/{sport}/odds");
    let params = q.to_params();
    let params_ref: Vec<(&str, String)> =
        params.iter().map(|(k, v)| (*k, v.clone())).collect();
    client.get(&path, &params_ref).await
}

pub async fn get_scores(
    client: &OddsApiClient,
    sport: &str,
    days_from: u32,
) -> AppResult<Vec<Score>> {
    let path = format!("/sports/{sport}/scores");
    let params = [("daysFrom", days_from.to_string())];
    client.get(&path, &params).await
}

pub async fn get_events(
    client: &OddsApiClient,
    sport: &str,
) -> AppResult<Vec<serde_json::Value>> {
    let path = format!("/sports/{sport}/events");
    client.get(&path, &[]).await
}

pub async fn get_event_odds(
    client: &OddsApiClient,
    sport: &str,
    event_id: &str,
    q: OddsQuery<'_>,
) -> AppResult<Event> {
    let path = format!("/sports/{sport}/events/{event_id}/odds");
    let params = q.to_params();
    let params_ref: Vec<(&str, String)> =
        params.iter().map(|(k, v)| (*k, v.clone())).collect();
    client.get(&path, &params_ref).await
}

pub async fn get_props_for_events(
    client: &OddsApiClient,
    sport: &str,
    event_ids: &[String],
    q: OddsQuery<'_>,
    max_concurrent: usize,
) -> Vec<Event> {
    let max = max_concurrent.max(1);
    let results: Vec<Option<Event>> = stream::iter(event_ids.iter().cloned())
        .map(|eid| {
            let q = q.clone();
            async move {
                match get_event_odds(client, sport, &eid, q).await {
                    Ok(event) => Some(event),
                    Err(err) => {
                        tracing::warn!(event_id = %eid, error = %err, "prop fetch failed");
                        None
                    }
                }
            }
        })
        .buffer_unordered(max)
        .collect()
        .await;

    results.into_iter().flatten().collect()
}

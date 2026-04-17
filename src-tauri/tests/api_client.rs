use oddsdesk_lib::api::{get_event_odds, get_odds, get_props_for_events, get_scores, get_sports, OddsApiClient, OddsQuery};
use oddsdesk_lib::errors::AppError;
use std::path::PathBuf;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {:?}: {e}", p))
}

async fn test_client(server: &MockServer) -> OddsApiClient {
    OddsApiClient::with_base_url("test-key", server.uri()).expect("client")
}

#[tokio::test]
async fn get_sports_parses_fixture() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sports"))
        .and(query_param("apiKey", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(fixture("sports.json"), "application/json"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let sports = get_sports(&client).await.expect("get_sports");

    assert_eq!(sports.len(), 4);
    let nba = sports.iter().find(|s| s.key == "basketball_nba").unwrap();
    assert_eq!(nba.title, "NBA");
    assert_eq!(nba.group, "Basketball");
    assert!(nba.active);
    assert!(!nba.has_outrights);
}

#[tokio::test]
async fn get_odds_parses_fixture() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sports/basketball_nba/odds"))
        .and(query_param("regions", "us"))
        .and(query_param("markets", "h2h,spreads,totals"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(fixture("odds_nba.json"), "application/json"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let events = get_odds(&client, "basketball_nba", OddsQuery::default())
        .await
        .expect("get_odds");

    assert_eq!(events.len(), 2);
    let first = &events[0];
    assert_eq!(first.home_team, "Lakers");
    assert_eq!(first.bookmakers.len(), 4);

    let fd = first.bookmakers.iter().find(|b| b.key == "fanduel").unwrap();
    let spreads = fd.markets.iter().find(|m| m.key == "spreads").unwrap();
    assert_eq!(spreads.outcomes[0].point, Some(-3.5));
}

#[tokio::test]
async fn get_scores_parses_and_helpers_work() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sports/basketball_nba/scores"))
        .and(query_param("daysFrom", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(fixture("scores_nba.json"), "application/json"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let scores = get_scores(&client, "basketball_nba", 1)
        .await
        .expect("get_scores");

    assert_eq!(scores.len(), 2);
    let live = &scores[0];
    assert_eq!(live.home_score(), "55");
    assert_eq!(live.away_score(), "52");
    assert!(!live.completed);

    let done = &scores[1];
    assert!(done.completed);
    assert_eq!(done.home_score(), "108");
    assert_eq!(done.away_score(), "112");
}

#[tokio::test]
async fn api_key_injected_in_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sports"))
        .and(query_param("apiKey", "my-secret-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .mount(&server)
        .await;

    let client = OddsApiClient::with_base_url("my-secret-key", server.uri()).unwrap();
    let sports = get_sports(&client).await.expect("get_sports");
    assert!(sports.is_empty());
}

#[tokio::test]
async fn credit_headers_parsed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sports"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-requests-remaining", "42")
                .insert_header("x-requests-used", "8")
                .set_body_raw("[]", "application/json"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let _ = get_sports(&client).await.unwrap();
    let credits = client.last_credits().await;
    assert_eq!(credits.remaining, Some(42));
    assert_eq!(credits.used, Some(8));
}

#[tokio::test]
async fn non_2xx_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sports"))
        .respond_with(
            ResponseTemplate::new(429).set_body_raw("rate limited", "text/plain"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let err = get_sports(&client).await.unwrap_err();
    match err {
        AppError::Api { status, msg } => {
            assert_eq!(status, 429);
            assert!(msg.contains("rate limited"));
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn props_resilience_drops_failures() {
    let server = MockServer::start().await;
    let event_body = fixture("event_odds_props.json");

    // Event 1 + 2 succeed, event 3 fails with 500.
    Mock::given(method("GET"))
        .and(path("/sports/basketball_nba/events/event-1/odds"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(event_body.clone(), "application/json"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/sports/basketball_nba/events/event-2/odds"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(event_body.clone(), "application/json"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/sports/basketball_nba/events/event-3/odds"))
        .respond_with(ResponseTemplate::new(500).set_body_raw("boom", "text/plain"))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let ids: Vec<String> = vec!["event-1".into(), "event-2".into(), "event-3".into()];
    let q = OddsQuery {
        markets: "player_points",
        ..Default::default()
    };
    let events = get_props_for_events(&client, "basketball_nba", &ids, q, 3).await;

    assert_eq!(events.len(), 2, "failed event should be dropped, not surfaced");
}

#[tokio::test]
async fn get_event_odds_parses_props_fixture() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sports/basketball_nba/events/event-nba-001/odds"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(fixture("event_odds_props.json"), "application/json"),
        )
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let q = OddsQuery {
        markets: "player_points",
        ..Default::default()
    };
    let event = get_event_odds(&client, "basketball_nba", "event-nba-001", q)
        .await
        .expect("get_event_odds");

    assert_eq!(event.bookmakers.len(), 3);
    let fd = event.bookmakers.iter().find(|b| b.key == "fanduel").unwrap();
    let market = &fd.markets[0];
    assert_eq!(market.key, "player_points");
    assert_eq!(market.outcomes.len(), 4);
    assert_eq!(
        market.outcomes[0].description.as_deref(),
        Some("LeBron James")
    );
}

use dct_core::events::GameEvent;
use dct_core::{PayloadKind, PayloadProcessor, deduplicate_events};
use serde_json::json;

#[test]
fn processor_preserves_original_payload_and_classifies_unknown_shape() {
    let input = br#"{"provider":{"name":"Dota 2"},"future_block":{"enabled":true}}"#;
    let processed = PayloadProcessor::new()
        .process(input)
        .expect("valid GSI JSON");

    assert_eq!(processed.kind, PayloadKind::Unknown);
    assert_eq!(processed.original["future_block"]["enabled"], json!(true));
}

#[test]
fn payload_kind_covers_playing_spectating_and_post_game() {
    let provider = json!({"name": "Dota 2", "appid": 570, "version": 1, "timestamp": 1});
    let playing = json!({"provider": provider, "player": {"steamid": "1", "activity": "playing"}})
        .to_string();
    let provider = json!({"name": "Dota 2", "appid": 570, "version": 1, "timestamp": 1});
    let spectating = json!({"provider": provider, "player": {"team2": {"player0": {"steamid": "1", "activity": "playing"}}}}).to_string();
    let provider = json!({"name": "Dota 2", "appid": 570, "version": 1, "timestamp": 1});
    let post_game =
        json!({"provider": provider, "map": {"game_state": "DOTA_GAMERULES_STATE_POST_GAME"}})
            .to_string();
    let processor = PayloadProcessor::new();
    assert_eq!(
        processor.process(playing.as_bytes()).unwrap().kind,
        PayloadKind::Playing
    );
    assert_eq!(
        processor.process(spectating.as_bytes()).unwrap().kind,
        PayloadKind::Spectating
    );
    assert_eq!(
        processor.process(post_game.as_bytes()).unwrap().kind,
        PayloadKind::PostGame
    );
    assert_eq!(PayloadKind::PostGame.as_str(), "post_game");
}

#[test]
fn repeated_events_are_deduplicated_by_time_and_kind() {
    let events = vec![
        GameEvent {
            game_time: 10,
            kind: "chat".into(),
            detail: Default::default(),
        },
        GameEvent {
            game_time: 10,
            kind: "chat".into(),
            detail: Default::default(),
        },
        GameEvent {
            game_time: 10,
            kind: "kill".into(),
            detail: Default::default(),
        },
    ];

    assert_eq!(deduplicate_events(events).len(), 2);
}

#[test]
fn draft_wire_format_is_transformed_like_reference_project() {
    let payload = serde_json::json!({
        "activeteam": 2,
        "pick": true,
        "activeteam_time_remaining": 10,
        "radiant_bonus_time": 118,
        "dire_bonus_time": 117,
        "team2": {
            "home_team": false,
            "pick0_id": 19,
            "pick0_class": "tiny",
            "ban0_id": 135,
            "ban0_class": "dawnbreaker"
        },
        "team3": {"home_team": true}
    });
    let draft: dct_core::draft::Draft = serde_json::from_value(payload).expect("draft parses");
    assert_eq!(draft.active_team, 2);
    assert_eq!(draft.radiant.picks[0].name, "tiny");
    assert_eq!(draft.radiant.bans[0].id, 135);
    assert!(draft.dire.picks.is_empty());
}

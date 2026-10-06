//! Public boundary for tolerant GSI payload parsing.

use serde_json::Value;

use crate::map::GamePhase;
use crate::{CoreError, Frame};

/// The broad shape inferred from a payload. This is descriptive rather than
/// restrictive: unknown payloads remain parseable when their JSON is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    Playing,
    Spectating,
    PostGame,
    Unknown,
}

impl PayloadKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Spectating => "spectating",
            Self::PostGame => "post_game",
            Self::Unknown => "unknown",
        }
    }
}

/// A parsed payload with both the typed frame and its original JSON tree.
#[derive(Debug, Clone, PartialEq)]
pub struct RawGameState {
    pub frame: Frame,
    pub original: Value,
    pub kind: PayloadKind,
}

/// Parse a GSI body without discarding fields unknown to this version.
pub fn parse_payload(input: &[u8]) -> Result<RawGameState, CoreError> {
    let original: Value = serde_json::from_slice(input).map_err(CoreError::InvalidJson)?;
    let frame = serde_json::from_value(original.clone()).map_err(CoreError::InvalidJson)?;
    let kind = classify(&frame);
    Ok(RawGameState {
        frame,
        original,
        kind,
    })
}

pub fn classify(frame: &Frame) -> PayloadKind {
    if matches!(
        frame.map.as_ref().map(|map| &map.phase),
        Some(GamePhase::PostGame)
    ) {
        return PayloadKind::PostGame;
    }
    match frame.players.as_ref() {
        Some(crate::player::Players::Playing(_)) => PayloadKind::Playing,
        Some(crate::player::Players::Spectating(_)) => PayloadKind::Spectating,
        None => PayloadKind::Unknown,
    }
}

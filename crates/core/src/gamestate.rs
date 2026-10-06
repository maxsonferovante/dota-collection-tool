//! Explicit processing pipeline over the existing domain model.

use serde_json::Value;

use crate::json::{PayloadKind, RawGameState, parse_payload};
use crate::{CoreError, Frame};

/// Normalized game state. `Frame` is retained as the compatibility model used
/// by the store and existing consumers.
pub type GameState = Frame;

/// Output of the complete parse/normalize/derive pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedGameState {
    pub state: GameState,
    pub original: Value,
    pub kind: PayloadKind,
}

/// Stateless processor for GSI payloads.
#[derive(Debug, Default, Clone, Copy)]
pub struct PayloadProcessor;

impl PayloadProcessor {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &[u8]) -> Result<RawGameState, CoreError> {
        parse_payload(input)
    }

    pub fn normalize(&self, raw: RawGameState) -> Result<GameState, CoreError> {
        Ok(raw.frame)
    }

    pub fn process(&self, input: &[u8]) -> Result<ProcessedGameState, CoreError> {
        let raw = self.parse(input)?;
        let kind = raw.kind;
        let original = raw.original.clone();
        let state = self.normalize(raw)?;
        Ok(ProcessedGameState {
            state,
            original,
            kind,
        })
    }
}

/// Deduplicate repeated event entries emitted by consecutive frames.
pub fn deduplicate_events(
    events: impl IntoIterator<Item = crate::events::GameEvent>,
) -> Vec<crate::events::GameEvent> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    events
        .into_iter()
        .filter(|event| seen.insert((event.game_time, event.kind.clone())))
        .collect()
}

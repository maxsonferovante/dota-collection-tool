//! Domain types for collected game frames.
//!
//! Tolerant parsing of the full protocol surface plus derived happenings.
//! Parsing is total: unknown fields are ignored, empty objects mean absent,
//! and no valid input panics.

pub mod abilities;
pub mod buildings;
pub mod couriers;
pub mod draft;
pub mod error;
pub mod events;
pub mod frame;
pub mod happening;
pub mod hero;
pub mod items;
pub mod keys;
pub mod league;
pub mod map;
pub mod minimap;
pub mod neutralitems;
pub mod player;
pub mod provider;
pub mod roshan;
pub mod support;
pub mod wearables;

pub use error::CoreError;
pub use frame::Frame;
pub use happening::{Happening, HappeningKind, derive_happenings};
pub use keys::{AbilitySlot, ItemSlot, PlayerSlot, Side};

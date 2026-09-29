//! Match-level state: clock, phase, score and Roshan.

use serde::{Deserialize, Serialize};

use crate::keys::Side;

/// Match phase reported by the game rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum GamePhase {
    Disconnected,
    InProgress,
    HeroSelection,
    Starting,
    Ending,
    PostGame,
    PreGame,
    StrategyTime,
    WaitingForMap,
    WaitingForPlayers,
    CustomGameSetup,
    Undefined(String),
}

impl From<String> for GamePhase {
    fn from(raw: String) -> Self {
        match raw.as_str() {
            "DOTA_GAMERULES_STATE_DISCONNECT" => GamePhase::Disconnected,
            "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS" => GamePhase::InProgress,
            "DOTA_GAMERULES_STATE_HERO_SELECTION" => GamePhase::HeroSelection,
            "DOTA_GAMERULES_STATE_INIT" => GamePhase::Starting,
            "DOTA_GAMERULES_STATE_LAST" => GamePhase::Ending,
            "DOTA_GAMERULES_STATE_POST_GAME" => GamePhase::PostGame,
            "DOTA_GAMERULES_STATE_PRE_GAME" => GamePhase::PreGame,
            "DOTA_GAMERULES_STATE_STRATEGY_TIME" => GamePhase::StrategyTime,
            "DOTA_GAMERULES_STATE_WAIT_FOR_MAP_TO_LOAD" => GamePhase::WaitingForMap,
            "DOTA_GAMERULES_STATE_WAIT_FOR_PLAYERS_TO_LOAD" => GamePhase::WaitingForPlayers,
            "DOTA_GAMERULES_STATE_CUSTOM_GAME_SETUP" => GamePhase::CustomGameSetup,
            _ => GamePhase::Undefined(raw),
        }
    }
}

impl From<GamePhase> for String {
    fn from(phase: GamePhase) -> Self {
        match phase {
            GamePhase::Disconnected => "DOTA_GAMERULES_STATE_DISCONNECT".to_owned(),
            GamePhase::InProgress => "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS".to_owned(),
            GamePhase::HeroSelection => "DOTA_GAMERULES_STATE_HERO_SELECTION".to_owned(),
            GamePhase::Starting => "DOTA_GAMERULES_STATE_INIT".to_owned(),
            GamePhase::Ending => "DOTA_GAMERULES_STATE_LAST".to_owned(),
            GamePhase::PostGame => "DOTA_GAMERULES_STATE_POST_GAME".to_owned(),
            GamePhase::PreGame => "DOTA_GAMERULES_STATE_PRE_GAME".to_owned(),
            GamePhase::StrategyTime => "DOTA_GAMERULES_STATE_STRATEGY_TIME".to_owned(),
            GamePhase::WaitingForMap => "DOTA_GAMERULES_STATE_WAIT_FOR_MAP_TO_LOAD".to_owned(),
            GamePhase::WaitingForPlayers => {
                "DOTA_GAMERULES_STATE_WAIT_FOR_PLAYERS_TO_LOAD".to_owned()
            }
            GamePhase::CustomGameSetup => "DOTA_GAMERULES_STATE_CUSTOM_GAME_SETUP".to_owned(),
            GamePhase::Undefined(raw) => raw,
        }
    }
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase::Undefined(String::new())
    }
}

/// Roshan pit state, only reported while spectating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum RoshanPit {
    Alive,
    RespawnBase,
    RespawnVariable,
    Undefined(String),
}

impl From<String> for RoshanPit {
    fn from(raw: String) -> Self {
        match raw.as_str() {
            "ALIVE" => RoshanPit::Alive,
            "RESPAWN_BASE" => RoshanPit::RespawnBase,
            "RESPAWN_VARIABLE" => RoshanPit::RespawnVariable,
            _ => RoshanPit::Undefined(raw),
        }
    }
}

impl From<RoshanPit> for String {
    fn from(pit: RoshanPit) -> Self {
        match pit {
            RoshanPit::Alive => "ALIVE".to_owned(),
            RoshanPit::RespawnBase => "RESPAWN_BASE".to_owned(),
            RoshanPit::RespawnVariable => "RESPAWN_VARIABLE".to_owned(),
            RoshanPit::Undefined(raw) => raw,
        }
    }
}

/// Match-level snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Map {
    #[serde(default)]
    pub name: String,
    #[serde(alias = "matchid", default)]
    pub match_id: String,
    #[serde(default)]
    pub game_time: u32,
    #[serde(default)]
    pub clock_time: i32,
    #[serde(default)]
    pub daytime: bool,
    #[serde(default)]
    pub nightstalker_night: bool,
    #[serde(alias = "game_state", default)]
    pub phase: GamePhase,
    #[serde(default)]
    pub paused: bool,
    #[serde(alias = "win_team", default)]
    pub winning_side: Side,
    #[serde(alias = "customgamename", default)]
    pub custom_game_name: String,
    #[serde(default)]
    pub ward_purchase_cooldown: Option<u16>,
    #[serde(default)]
    pub radiant_ward_purchase_cooldown: Option<u16>,
    #[serde(default)]
    pub dire_ward_purchase_cooldown: Option<u16>,
    #[serde(default)]
    pub radiant_score: Option<u32>,
    #[serde(default)]
    pub dire_score: Option<u32>,
    #[serde(alias = "roshan_state", default)]
    pub roshan_pit: Option<RoshanPit>,
    #[serde(alias = "roshan_state_end_time", default)]
    pub roshan_pit_end: Option<u32>,
}

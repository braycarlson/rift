//! Hand-written models for the League Live Client Data API.
//!
//! Field names follow Riot's on-the-wire JSON. Rarely-critical fields are
//! optional so a client patch that drops one does not break deserialization.

use rustc_hash::FxHashMap;

/// The complete snapshot returned by `/liveclientdata/allgamedata`.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllGameData {
    /// The local player's live state.
    pub active_player: ActivePlayer,
    /// Every player in the game, including bots.
    pub all_players: Vec<Player>,
    /// The ordered event log for the match.
    pub events: EventData,
    /// Match-wide statistics.
    pub game_data: GameStats,
}

/// The local player's live state.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePlayer {
    /// The player's ability levels and descriptions.
    #[serde(default)]
    pub abilities: Option<Abilities>,
    /// The player's live champion stats, keyed by Riot's stat names.
    #[serde(default)]
    pub champion_stats: FxHashMap<String, f64>,
    /// Current gold on hand.
    #[serde(default)]
    pub current_gold: f64,
    /// The player's full rune page.
    #[serde(default)]
    pub full_runes: Option<FullRunes>,
    /// Champion level.
    #[serde(default)]
    pub level: i32,
    /// The player's Riot id, on newer clients.
    #[serde(default)]
    pub riot_id: Option<String>,
    /// The player's display name, when exposed by the client.
    #[serde(default)]
    pub summoner_name: Option<String>,
}

/// The five ability slots of the active player.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Abilities {
    /// Passive ability.
    #[serde(rename = "Passive")]
    pub passive: Ability,
    /// Q ability.
    #[serde(rename = "Q")]
    pub q: Ability,
    /// W ability.
    #[serde(rename = "W")]
    pub w: Ability,
    /// E ability.
    #[serde(rename = "E")]
    pub e: Ability,
    /// R ability.
    #[serde(rename = "R")]
    pub r: Ability,
}

/// A single champion ability.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ability {
    /// Current rank of the ability.
    #[serde(default)]
    pub ability_level: i32,
    /// Human-readable ability name.
    #[serde(default)]
    pub display_name: String,
    /// Internal ability id.
    #[serde(default)]
    pub id: String,
}

/// The active player's full rune page.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullRunes {
    /// Non-keystone runes in the primary tree.
    #[serde(default)]
    pub general_runes: Vec<Rune>,
    /// The chosen keystone.
    pub keystone: Rune,
    /// The primary rune tree.
    pub primary_rune_tree: Rune,
    /// The secondary rune tree.
    pub secondary_rune_tree: Rune,
    /// The chosen stat shards.
    #[serde(default)]
    pub stat_runes: Vec<StatRune>,
}

/// A single rune or rune tree.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rune {
    /// Human-readable rune name.
    #[serde(default)]
    pub display_name: String,
    /// Numeric rune id.
    #[serde(default)]
    pub id: i64,
    /// Raw description key.
    #[serde(default)]
    pub raw_description: String,
}

/// A stat shard selection.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRune {
    /// Numeric stat-rune id.
    #[serde(default)]
    pub id: i64,
    /// Raw description key.
    #[serde(default)]
    pub raw_description: String,
}

/// One player in the game.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    /// Display champion name.
    #[serde(default)]
    pub champion_name: String,
    /// Whether the player is a bot.
    #[serde(default)]
    pub is_bot: bool,
    /// Whether the player is currently dead.
    #[serde(default)]
    pub is_dead: bool,
    /// The player's items.
    #[serde(default)]
    pub items: Vec<Item>,
    /// Champion level.
    #[serde(default)]
    pub level: i32,
    /// Assigned lane or role.
    #[serde(default)]
    pub position: String,
    /// Internal champion name.
    #[serde(default)]
    pub raw_champion_name: String,
    /// Seconds until respawn, zero when alive.
    #[serde(default)]
    pub respawn_timer: f64,
    /// The player's Riot id, on newer clients.
    #[serde(default)]
    pub riot_id: Option<String>,
    /// Kills, deaths, assists, and scores.
    pub scores: Scores,
    /// The player's display name, on older clients.
    #[serde(default)]
    pub summoner_name: Option<String>,
    /// Team identifier, `ORDER` or `CHAOS`.
    #[serde(default)]
    pub team: String,
}

/// A single item in a player's inventory.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    /// Stack count.
    #[serde(default)]
    pub count: i32,
    /// Human-readable item name.
    #[serde(default)]
    pub display_name: String,
    /// Numeric item id.
    #[serde(default)]
    pub item_id: i64,
    /// Total purchase price.
    #[serde(default)]
    pub price: i32,
    /// Inventory slot index.
    #[serde(default)]
    pub slot: i32,
}

/// A player's kill, death, assist, and score totals.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scores {
    /// Assist count.
    #[serde(default)]
    pub assists: i32,
    /// Minion and monster kills.
    #[serde(default)]
    pub creep_score: i32,
    /// Death count.
    #[serde(default)]
    pub deaths: i32,
    /// Kill count.
    #[serde(default)]
    pub kills: i32,
    /// Vision score.
    #[serde(default)]
    pub ward_score: f64,
}

/// The ordered list of game events.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EventData {
    /// Every event so far, in occurrence order.
    #[serde(rename = "Events", default)]
    pub events: Vec<Event>,
}

/// A single game event, such as a kill or objective.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Event {
    /// Monotonic event id.
    #[serde(rename = "EventID", default)]
    pub event_id: i64,
    /// Event kind, such as `ChampionKill`.
    #[serde(rename = "EventName", default)]
    pub event_name: String,
    /// Seconds into the match.
    #[serde(rename = "EventTime", default)]
    pub event_time: f64,
    /// Any event-specific fields (participant names, positions, ...).
    #[serde(flatten)]
    pub extra: FxHashMap<String, serde_json::Value>,
}

/// Match-wide statistics.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStats {
    /// The game mode, such as `CLASSIC`.
    #[serde(default)]
    pub game_mode: String,
    /// Elapsed game time in seconds.
    #[serde(default)]
    pub game_time: f64,
    /// The map's display name.
    #[serde(default)]
    pub map_name: String,
    /// The numeric map id.
    #[serde(default)]
    pub map_number: i32,
    /// The map terrain variant.
    #[serde(default)]
    pub map_terrain: String,
}

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Vec<Data>,
    pub additional: Additional,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub id: String,
    pub details: Details,
    pub player: Player,
    pub played_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Additional {
    pub gamertag: String,
}

#[derive(Debug, Deserialize)]
pub struct Details {
    pub category: Category,
    pub map: GameMap,
}

#[derive(Debug, Deserialize)]
pub struct Category {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GameMap {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub stats: Stats,
    pub rank: usize,
    pub outcome: Outcome,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Win,
    Loss,
}

#[derive(Debug, Deserialize)]
pub struct Stats {
    pub core: CoreStats,
}

#[derive(Debug, Deserialize)]
pub struct CoreStats {
    pub summary: Summary,
}

#[derive(Debug, Deserialize)]
pub struct Summary {
    pub kills: usize,
    pub deaths: usize,
    pub assists: usize,
}

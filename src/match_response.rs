use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MatchResponse {
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
    pub playlist: Playlist,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub properties: Properties,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    pub queue: Option<Queue>,
    pub input: Option<Input>,
    pub ranked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Queue {
    SoloDuo,
    Open,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Input {
    Controller,
    Mnk,
    Crossplay,
}

#[derive(Debug, Deserialize)]
pub struct Category {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GameMap {
    pub name: String,
    pub asset: MapAsset,
}

#[derive(Debug, Deserialize)]
pub struct MapAsset {
    pub thumbnail_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub stats: Stats,
    pub rank: usize,
    pub outcome: Outcome,
    pub progression: Option<Progression>,
}

#[derive(Debug, Deserialize)]
pub struct Progression {
    pub csr: Csr,
}

#[derive(Debug, Deserialize)]
pub struct Csr {
    pub pre_match: CsrResult,
    pub post_match: CsrResult,
}

#[derive(Debug, Deserialize)]
pub struct CsrResult {
    pub tier: Tier,
    pub value: isize,
    pub tier_start: usize,
    pub sub_tier: usize,
    pub tier_image_url: String,
}

#[derive(Debug, Deserialize)]
pub enum Tier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Onyx,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

#[derive(Debug, Deserialize)]
pub struct Stats {
    pub core: CoreStats,
}

#[derive(Debug, Deserialize)]
pub struct CoreStats {
    pub summary: Summary,
    pub damage: Damage,
    pub shots: Shots,
    pub breakdowns: Breakdowns,
}

#[derive(Debug, Deserialize)]
pub struct Breakdowns {
    pub medals: Vec<Medal>,
}

#[derive(Debug, Deserialize)]
pub struct Medal {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Summary {
    pub kills: usize,
    pub deaths: usize,
    pub assists: usize,
}

#[derive(Debug, Deserialize)]
pub struct Damage {
    pub taken: usize,
    pub dealt: usize,
}

#[derive(Debug, Deserialize)]
pub struct Shots {
    pub accuracy: f64,
}

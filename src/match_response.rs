use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MatchResponse {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub teams: Teams,
    pub players: Vec<Player>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub stats: Stats,
}

#[derive(Debug, Deserialize)]
pub struct Teams {
    pub details: Vec<TeamDetail>,
}

#[derive(Debug, Deserialize)]
pub struct TeamDetail {
    pub team: Team,
    pub stats: Stats,
}

#[derive(Debug, Deserialize)]
pub struct Stats {
    pub core: CoreStats,
}

#[derive(Debug, Deserialize)]
pub struct CoreStats {
    pub damage: Damage,
    pub summary: Summary,
}

#[derive(Debug, Deserialize)]
pub struct Summary {
    pub kills: usize,
    pub deaths: usize,
    pub assists: usize,
}

#[derive(Debug, Deserialize)]
pub struct Damage {
    pub dealt: usize,
    pub taken: usize,
}

#[derive(Debug, Deserialize)]
pub struct Team {
    pub id: usize,
    pub name: String,
    pub skill: Skill,
}

#[derive(Debug, Deserialize)]
pub struct Skill {
    pub mmr: f64,
}

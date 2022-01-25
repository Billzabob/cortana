use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MatchResponse {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub teams: Teams,
}

#[derive(Debug, Deserialize)]
pub struct Teams {
    pub details: Vec<TeamDetail>,
}

#[derive(Debug, Deserialize)]
pub struct TeamDetail {
    pub team: Team,
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

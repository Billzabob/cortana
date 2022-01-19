use serde::Serialize;

#[derive(Serialize)]
pub struct MatchRequest<'a> {
    pub gamertag: &'a str,
    pub limit: Limit,
}

#[derive(Serialize)]
pub struct Limit {
    pub count: usize,
}

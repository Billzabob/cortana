use serde::Serialize;

#[derive(Serialize)]
pub struct MatchesRequest<'a> {
    pub gamertag: &'a str,
    pub limit: Limit,
}

#[derive(Serialize)]
pub struct Limit {
    pub count: usize,
}

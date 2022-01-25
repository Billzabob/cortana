use serde::Serialize;

#[derive(Serialize)]
pub struct MatchRequest<'a> {
    pub id: &'a str,
}

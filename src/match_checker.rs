mod request;
mod response;

use futures::{future, FutureExt};
use request::{Limit, Request};
use response::Response;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use tokio_postgres::{Client, Statement};

pub async fn check_for_new_matches(client: &Client) {
    let mut interval = time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;

        let statement = client
            .prepare("update users set latest_match_id = $1 where gamertag = $2")
            .await
            .expect("prepare");

        let new_games = get_new_games(client).await;

        let updates: Vec<_> = new_games
            .iter()
            .map(|game| update_match(game, client, &statement))
            .collect();

        future::join_all(updates).await;
    }
}

async fn update_match(
    game: &Response,
    client: &Client,
    statement: &Statement,
) -> Result<u64, tokio_postgres::Error> {
    let latest_match_id = game.data[0].id.as_str();
    let gamertag = game.additional.gamertag.as_str();

    client
        .execute(statement, &[&latest_match_id, &gamertag])
        .await
}

async fn get_new_games(client: &Client) -> Vec<Response> {
    let rows = client
        .query(
            "select gamertag, latest_match_id from users where enabled",
            &[],
        )
        .await
        .expect("new matches SQL");

    let new_games: Vec<_> = rows
        .iter()
        .map(|row| {
            let gamertag: &str = row.get(0);
            let last_match_id: Option<&str> = row.get(1);

            get_latest_match(gamertag).map(move |game| match game {
                Ok(game) => {
                    if game.data.first().map(|data| data.id.as_str()) != last_match_id {
                        Some(game)
                    } else {
                        None
                    }
                }
                _ => None,
            })
        })
        .collect();

    future::join_all(new_games)
        .await
        .into_iter()
        .filter_map(std::convert::identity)
        .collect()
}

async fn get_latest_match(gamertag: &str) -> Result<Response, Box<dyn Error>> {
    let request = Request {
        gamertag: gamertag,
        limit: Limit { count: 1 },
    };

    let token = std::env::var("HALO_API_TOKEN")?;

    let response = reqwest::Client::new()
        .post("https://halo.api.stdlib.com/infinite@0.3.3/stats/matches/list/")
        .bearer_auth(token)
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

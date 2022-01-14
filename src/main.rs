mod match_checker;

use futures::StreamExt;
use match_checker::check_for_new_matches;
use match_checker::response::{Outcome, Response};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use serenity::{
    async_trait,
    http::Http,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use std::error::Error;
use std::{env, sync::Arc};

struct Handler;

static EMOJIS: [&str; 22] = [
    "<:Boogeyman:931631927486734417>",
    "<:Demon:931631928120066088>",
    "<:DoubleKill:931631928023597066>",
    "<:Extermination:931631929239949343>",
    "<:GrappleJack:931631927788728342>",
    "<:GrimReaper:931631928136826900>",
    "<:KillingFrenzy:931631928497541171>",
    "<:KillingSpree:931631928476598302>",
    "<:Killionaire:931631929311244370>",
    "<:Killjoy:931631928585642055>",
    "<:Killtastrophe:931631929667780608>",
    "<:Killtrocity:931631929642590228>",
    "<:Nightmare:931631929067986944>",
    "<:Ninja:931631929697136770>",
    "<:NoScope:931631929139277874>",
    "<:Overkill:931631929617448970>",
    "<:Perfection:931631929646796870>",
    "<:Quigley:931631929663569980>",
    "<:Rampage:931631929420296202>",
    "<:RunningRiot:931631929621622875>",
    "<:Snipe:931631929575473192>",
    "<:TripleKill:931631929185411124>",
];

fn name_to_emoji(name: &str) -> Option<&str> {
    let name: String = name.chars().filter(|c| !c.is_whitespace()).collect();
    EMOJIS
        .iter()
        .find(|emoji| emoji.contains(&format!(":{}:", name)))
        .copied()
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        println!("Content: {}", msg.content);
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn send_match_results(
    http: &Arc<Http>,
    channel_id: ChannelId,
    response: Response,
) -> Result<Message, Box<dyn Error>> {
    let data = response.data.first().ok_or("not at least one match")?;
    let outcome = &data.player.outcome;
    let timestamp = &data.played_at;

    let (result, color) = match outcome {
        Outcome::Win => ("WON", (0, 255, 0)),
        Outcome::Loss => ("LOST", (255, 0, 0)),
    };

    let stats = &data.player.stats.core;

    let emblem_url = get_emblem(&response.additional.gamertag)
        .await
        .expect("emblem")
        .data
        .emblem_url;

    let medals = &data.player.stats.core.breakdowns.medals;
    let medal_string = medals
        .iter()
        .map(|m| m.name.as_str())
        .filter_map(|name| name_to_emoji(name))
        .fold(String::new(), |acc, a| acc + a);

    let message = channel_id
        .send_message(http, |m| {
            m.embed(|e| {
                e.title(format!(
                    "{} {} a game of {}!",
                    response.additional.gamertag, result, data.details.category.name
                ))
                .color(color)
                .field(
                    "KDA",
                    format!(
                        "{}/{}/{}",
                        stats.summary.kills, stats.summary.deaths, stats.summary.assists
                    ),
                    true,
                )
                .field(
                    "Accuracy",
                    format!("{}%", stats.shots.accuracy.round()),
                    true,
                )
                .field("Damage Dealt", stats.damage.dealt, true)
                .field("Damage Taken", stats.damage.taken, true)
                .field("Medals", medal_string, true)
                .image(&data.details.map.asset.thumbnail_url)
                .url(format!(
                    "https://halotracker.com/halo-infinite/match/{}",
                    data.id
                ))
                .thumbnail(emblem_url)
                .timestamp(timestamp)
            })
        })
        .await?;

    Ok(message)
}

async fn connect_to_db() -> Result<tokio_postgres::Client, Box<dyn Error>> {
    let connection_string = env::var("DB_CONNECTION_STRING")?;

    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_ca_file("ca-certificate.crt")?;
    let connector = MakeTlsConnector::new(builder.build());

    let (client, connection) = tokio_postgres::connect(&connection_string, connector).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

async fn send_matches(client: &tokio_postgres::Client, http: Arc<Http>) {
    let new_matches = check_for_new_matches(client);

    futures::pin_mut!(new_matches);

    let channel = ChannelId(689701123967156423);

    while let Some(game) = new_matches.next().await {
        println!("{}", game.additional.gamertag);

        if let Err(why) = send_match_results(&http, channel, game).await {
            println!("Failed sending message: {}", why)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sql_client = connect_to_db().await?;

    let rows = sql_client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    let value: &str = rows[0].get(0);
    println!("{:#?}", value);
    assert_eq!(value, "hello world");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")?;
    let mut client = Client::builder(&token).event_handler(Handler).await?;

    let http = Arc::clone(&client.cache_and_http.http);

    if let (Err(why), _) = tokio::join!(client.start(), send_matches(&sql_client, http)) {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Request {
    gamertag: String,
}

#[derive(Debug, Deserialize)]
struct Resp {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    emblem_url: String,
}

async fn get_emblem(gamertag: &str) -> Result<Resp, Box<dyn Error>> {
    let request = Request {
        gamertag: gamertag.to_owned(),
    };

    let token = std::env::var("HALO_API_TOKEN")?;

    let response = reqwest::Client::new()
        .post("https://halo.api.stdlib.com/infinite@0.3.3/appearance")
        .bearer_auth(token)
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

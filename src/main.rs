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

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.content.starts_with("player:") {
            let gamertag = &msg.content[8..];

            println!("Requested player: {}", gamertag);
        }
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

    let message = channel_id
        .send_message(http, |m| {
            m.embed(|e| {
                e.title(format!(
                    "{} {} a match!",
                    response.additional.gamertag, result
                ))
                .description("This is a description")
                .color(color)
                .fields(vec![
                    ("This is the first field", "This is a field body", true),
                    ("This is the second field", "Both fields are inline", true),
                ])
                .field(
                    "This is the third field",
                    "This is not an inline field",
                    false,
                )
                .footer(|f| f.text("This is a footer"))
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

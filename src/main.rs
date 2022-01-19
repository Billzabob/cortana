mod emblem_request;
mod emblem_response;
mod match_checker;
mod match_request;
mod match_response;

use crate::emblem_request::EmblemRequest;
use crate::emblem_response::EmblemResponse;
use crate::match_checker::check_for_new_matches;
use crate::match_response::Input::*;
use crate::match_response::Queue::*;
use crate::match_response::Tier::*;
use crate::match_response::{MatchResponse, Outcome};
use futures::StreamExt;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use serenity::model::id::{GuildId, UserId};
use serenity::{
    async_trait,
    http::Http,
    model::{
        channel::Message,
        gateway::Ready,
        id::ChannelId,
        interactions::{
            application_command::{
                ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};
use std::error::Error;
use std::{env, sync::Arc};

struct Handler {
    client: Arc<tokio_postgres::Client>,
}

static EMOJIS: [&str; 36] = [
    "<:AchillesSpine:932071031106072607>",
    "<:BackSmack:932071030825058384>",
    "<:Boogeyman:931631927486734417>",
    "<:BoomBlock:932071031072518144>",
    "<:Boxer:932071030766333982>",
    "<:Demon:931631928120066088>",
    "<:DoubleKill:931631928023597066>",
    "<:Extermination:931631929239949343>",
    "<:Fastball:932071030950867057>",
    "<:FromtheGrave:932071030795677717>",
    "<:Fumble:932071030762119168>",
    "<:GrappleJack:931631927788728342>",
    "<:GrimReaper:931631928136826900>",
    "<:GuardianAngel:932071031152214016>",
    "<:KillingFrenzy:931631928497541171>",
    "<:KillingSpree:931631928476598302>",
    "<:Killionaire:931631929311244370>",
    "<:Killjoy:931631928585642055>",
    "<:Killtastrophe:931631929667780608>",
    "<:Killtrocity:931631929642590228>",
    "<:LastShot:932071030862790706>",
    "<:Marksman:932071031047340073>",
    "<:Nightmare:931631929067986944>",
    "<:Ninja:931631929697136770>",
    "<:NoScope:931631929139277874>",
    "<:Overkill:931631929617448970>",
    "<:Perfect:932071031181570078>",
    "<:Perfection:931631929646796870>",
    "<:Quigley:931631929663569980>",
    "<:Rampage:931631929420296202>",
    "<:Reversal:932071031101853736>",
    "<:RunningRiot:931631929621622875>",
    "<:Snipe:931631929575473192>",
    "<:TripleKill:931631929185411124>",
    "<:Wingman:932071030732767283>",
    "<:YardSale:932071031068307456>",
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
    // async fn message(&self, _ctx: Context, message: Message) {
    //     println!("{}", message.content)
    // }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "register" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected a gamertag option")
                        .resolved
                        .as_ref()
                        .expect("Expected a gamertag");

                    match options {
                        ApplicationCommandInteractionDataOptionValue::String(gamertag) => {
                            register_gamertag(gamertag, command.user.id, &self.client).await
                        }
                        _ => unreachable!("Command type"),
                    }
                }
                "toggle" => toggle_user(command.user.id, &self.client).await,
                _ => unreachable!("Unknown command"),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let guild_id = GuildId(460204093722591232);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("register")
                        .description("Register yourself for match updates")
                        .create_option(|option| {
                            option
                                .name("gamertag")
                                .description("Your GamerTag")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("toggle")
                        .description("Toggle whether your games are displayed")
                })
        })
        .await;

        if let Err(why) = commands {
            println!("Error creating commands: {}", why);
        }

        println!("{} is connected!", ready.user.name);
    }
}

async fn register_gamertag(
    gamertag: &str,
    user_id: UserId,
    client: &tokio_postgres::Client,
) -> String {
    let user_id = user_id.0 as i64;
    let result = client
        .execute(
            "insert into users (discord_id, gamertag) values ($1, $2) on conflict (discord_id) do update set gamertag = EXCLUDED.gamertag",
            &[&user_id, &gamertag.to_lowercase()],
        )
        .await;
    match result {
        Ok(_) => format!("Registered {}", gamertag),
        Err(_) => format!("Someone has already registered as {}", gamertag),
    }
}

async fn toggle_user(user_id: UserId, client: &tokio_postgres::Client) -> String {
    let user_id = user_id.0 as i64;
    let result = client
        .query(
            "update users set enabled = not enabled where discord_id = $1 returning enabled",
            &[&user_id],
        )
        .await;
    let toggle: bool = result.unwrap().first().unwrap().get(0);
    if toggle {
        "Your matches will now be shown again".to_owned()
    } else {
        "You will no longer see your matches".to_owned()
    }
}

async fn send_match_results(
    http: &Arc<Http>,
    channel_id: ChannelId,
    response: MatchResponse,
) -> Result<Message, Box<dyn Error>> {
    let data = response.data.first().ok_or("not at least one match")?;
    let outcome = &data.player.outcome;
    let timestamp = &data.played_at;

    let (result, color) = match outcome {
        Outcome::Win => ("WON", (0, 255, 0)),
        Outcome::Loss => ("LOST", (255, 0, 0)),
        Outcome::Draw => ("TIED", (0, 0, 255)),
    };

    let stats = &data.player.stats.core;

    let emblem_url = get_emblem(&response.additional.gamertag)
        .await
        .expect("emblem")
        .data
        .emblem_url;

    let medals = &data.player.stats.core.breakdowns.medals;
    let mut medal_string = medals
        .iter()
        .map(|m| m.name.as_str())
        .filter_map(|name| name_to_emoji(name))
        .fold(String::new(), |acc, a| acc + a);

    if medal_string.is_empty() {
        medal_string = "Nothing special 😔".to_owned();
    }

    let csr = &data.player.progression.as_ref().expect("progression").csr;
    let csr_change = csr.post_match.value - csr.pre_match.value;
    let csr_change = if csr_change > 0 {
        format!("+{}", csr_change)
    } else {
        csr_change.to_string()
    };

    let rank = match csr.post_match.tier {
        Bronze => format!(
            "<:Bronze_Rank_Icon:933098600471363624> Bronze {}",
            csr.post_match.sub_tier
        ),
        Silver => format!(
            "<:Silver_Rank_Icon:933098600609775646> Silver {}",
            csr.post_match.sub_tier
        ),
        Gold => format!(
            "<:Gold_Rank_Icon:933098600437776465> Gold {}",
            csr.post_match.sub_tier
        ),
        Platinum => format!(
            "<:Platinum_Rank_Icon:933098600718802954> Platinum {}",
            csr.post_match.sub_tier
        ),
        Diamond => format!(
            "<:Diamond_Rank_Icon:933098600488116294> Diamond {}",
            csr.post_match.sub_tier
        ),
        Onyx => "<:Onyx_Rank_Icon:933098600332931143> Onyx".to_owned(),
    };

    let input = match data.details.playlist.properties.input {
        Some(Mnk) => "M+K",
        Some(Controller) => "Controller",
        Some(Crossplay) => "Crossplay",
        None => "Unknown",
    };

    let queue = match data.details.playlist.properties.queue {
        Some(SoloDuo) => "Solo/Duo",
        Some(Open) => "Open",
        None => "Unknown",
    };

    let playlist = format!("{} {}", queue, input);

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
                .field("CSR change", csr_change, true)
                .field("Rank", rank, true)
                .field("CSR", csr.post_match.value, true)
                .field("Playlist", playlist, true)
                .field(
                    "Accuracy",
                    format!("{}%", stats.shots.accuracy.round()),
                    true,
                )
                .field("Damage Dealt", stats.damage.dealt, true)
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

async fn get_emblem(gamertag: &str) -> Result<EmblemResponse, Box<dyn Error>> {
    let request = EmblemRequest {
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

async fn send_matches(client: Arc<tokio_postgres::Client>, http: Arc<Http>) {
    let new_matches = check_for_new_matches(&client);

    futures::pin_mut!(new_matches);

    let channel = ChannelId(931701787658965032);

    while let Some(game) = new_matches.next().await {
        println!("{}", game.additional.gamertag);

        if let Err(why) = send_match_results(&http, channel, game).await {
            println!("Failed sending message: {}", why)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sql_client = Arc::new(connect_to_db().await?);

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")?;
    let mut client = Client::builder(&token)
        .event_handler(Handler {
            client: Arc::clone(&sql_client),
        })
        .application_id(928312197489229825)
        .await?;

    let http = Arc::clone(&client.cache_and_http.http);

    if let (Err(why), _) = tokio::join!(client.start(), send_matches(sql_client, http)) {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

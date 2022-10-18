use std::borrow::BorrowMut;
use std::sync;
use std::sync::atomic::AtomicBool;

use bimap::BiMap;
use irc::client::prelude::Config as IrcConfig;
use irc_bridge::BridgeMessage;
use poise::serenity_prelude::{self, Mutex, json};

mod irc_bridge;

mod commands;
mod database;
mod util;
mod smtp;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub struct Data {
    irc_running_bridge: AtomicBool,
    irc_tx: Mutex<Option<irc_bridge::Sender>>,
    irc_channel_map: BiMap<u64, String>,
    irc_config: IrcConfig,
    irc_webhook_avatar: String,
    verify_role: u64,
    verify_emails: Vec<String>
}

use poise::serenity_prelude::{Activity, OnlineStatus};
use tokio::io::AsyncReadExt;
use std::path::Path;

pub async fn event_listener(
    ctx: &serenity_prelude::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    /*
        Runs an event listener using Serenity's built-in listener to set the status and presence to online
    */
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is connected!", data_about_bot.user.name);

            let activity = Activity::playing("vim");
            let status = OnlineStatus::Online;

            ctx.set_presence(Some(activity), status).await;

            // Initialize IRC bridge
            if !user_data.irc_running_bridge.swap(true, std::sync::atomic::Ordering::SeqCst) {
                println!("Initializing IRC bridge...");
                let tx = irc_bridge::run(
                    sync::Arc::new(ctx.clone()), 
                    user_data.irc_config.clone(), 
                    user_data.irc_channel_map.clone(),
                    user_data.irc_webhook_avatar.clone(),
                );
                // Store tx so we can send to it when we get messages later
                *user_data.irc_tx.lock().await = Some(tx);
            }
        },
        poise::Event::Message { new_message } => {
            if !new_message.author.bot {
                // Check if channel is bridged to IRC
                let channel = user_data.irc_channel_map.get_by_left(new_message.channel_id.as_u64());
                if let Some(channel) = channel {
                    let author = new_message.author
                        .nick_in(&ctx.http, new_message.guild_id.unwrap())
                        .await
                        .unwrap_or_else(|| new_message.author.name.clone());
                    let mut content = new_message.content.clone();
                    for attachment in &new_message.attachments {
                        content += "\nAttachment: ";
                        content += &attachment.url;
                    }
                    let msg = BridgeMessage {
                        author,
                        channel: channel.to_owned(),
                        message: content
                    };
                    let mut tx = user_data.irc_tx.lock().await;
                    if tx.is_some() {
                        if let Err(e) = tx.as_mut().unwrap().send(msg).await {
                            println!("warning: error sending to IRC bridge: {}", e);
                        }
                    } else {
                        println!("warning: IRC bridge not yet initialized");
                    }
                }
            }
        },
        poise::Event::GuildMemberAddition { new_member } => {
            let guild_id = new_member.guild_id.0;
            let role_id = user_data.verify_role;
            if role_id < 1 { return Ok(()); }
            let user_id = new_member.user.id.0;
            if database::auth::get_user(&guild_id, &user_id)?.is_none() {
                return Ok(());
            };
            new_member.to_owned().borrow_mut().add_role(&ctx, &serenity_prelude::RoleId(role_id)).await?;
        }
        _ => {}
    }

    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {

    database::init().unwrap();
    
    dotenv::dotenv().ok();
    let token = std::env::var("BOT_TOKEN").expect("Could not find BOT_TOKEN in environment variables");

    if !Path::new("config.json").exists() {
        tokio::fs::write("config.json", "{}").await.expect("Unable to create default config.json file! Do i have write permissions?");
    }

    let mut config_file = tokio::fs::File::open("config.json").await.expect("Could not open config.json");
    let mut buf = String::new();
    config_file.read_to_string(&mut buf).await.expect("Could not read config.json");
    let config: json::Value = json::prelude::from_str(&buf).expect("config.json contained invalid JSON");

    let mut data = Data {
        irc_tx: Mutex::new(None),
        irc_running_bridge: true.into(), // set to `true` to disable bridge by default
        irc_channel_map: BiMap::new(),
        irc_config: IrcConfig::default(),
        irc_webhook_avatar: "".to_owned(),
        verify_role: 0,
        verify_emails: Vec::new()
    };

    database::init().expect("Failed to load database!");

    // If the `irc` object exists in the config 
    // file, load IRC config into the user data object
    if !config["irc"].is_null() {
        irc_bridge::load_data_from_config(&mut data, &config["irc"]);
    }

    // If the `verify` object exists in the config 
    // file, load verify config into the user data object
    if !config["verify"].is_null() {
        let verify = config["verify"].as_object().unwrap();
        match verify["role"].as_u64() {
            None => {},
            Some(id) => {
                data.verify_role = id;
            }
        }
        match verify["allowed_emails"].as_array() {
            None => {},
            Some(array) => {
                data.verify_emails = array.iter().map(|v| v.as_str() ).flatten().map(|v| v.to_string() ).collect::<Vec<String>>();
            }
        }
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                register(),
                commands::ping(),
                commands::addrole(),
                commands::deleterole(),
                commands::addrolepage(),
                commands::deleterolepage(),
                commands::roles(),
                commands::verify(),
                commands::bridge(),
            ],
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(
                    ctx, event, framework, user_data,
                ))
            },
            ..Default::default()
        })
        .token(token)
        .intents(
            serenity_prelude::GatewayIntents::MESSAGE_CONTENT | 
            serenity_prelude::GatewayIntents::GUILD_MESSAGES | 
            serenity_prelude::GatewayIntents::GUILD_MEMBERS | 
            serenity_prelude::GatewayIntents::non_privileged()
        ).user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(data) }));

    framework.run().await.unwrap();
}



mod irc_bridge;

use std::sync;
use std::sync::atomic::AtomicBool;

use bimap::BiMap;
use irc::client::prelude::Config as IrcConfig;
use irc_bridge::BridgeMessage;
use poise::serenity_prelude::{self, Mutex, json};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    irc_running_bridge: AtomicBool,
    irc_tx: Mutex<Option<irc_bridge::Sender>>,
    irc_channel_map: BiMap<u64, String>,
    irc_config: IrcConfig,
}

use poise::serenity_prelude::{Activity, OnlineStatus};
use tokio::io::AsyncReadExt;

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
                    user_data.irc_channel_map.clone());
                *user_data.irc_tx.lock().await = Some(tx);
            }
        },
        poise::Event::Message { new_message } => {
            if !new_message.author.bot {
                let channel = user_data.irc_channel_map.get_by_left(new_message.channel_id.as_u64());
                if let Some(channel) = channel {
                    let msg = BridgeMessage {
                        author: new_message.author.name.clone(),
                        channel: channel.to_owned(),
                        message: new_message.content.clone(),
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
        }
        _ => {}
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = std::env::var("BOT_TOKEN").expect("Could not find BOT_TOKEN in environment variables");

    let mut config_file = tokio::fs::File::open("config.json").await.expect("Could not open config.json");
    let mut buf = String::new();
    config_file.read_to_string(&mut buf).await.expect("Could not read config.json");
    let config: json::Value = json::prelude::from_str(&buf).expect("config.json contained invalid JSON");

    let mut data = Data {
        irc_tx: Mutex::new(None),
        irc_running_bridge: true.into(), // set to `true` to disable bridge by default
        irc_channel_map: BiMap::new(),
        irc_config: IrcConfig::default(),
    };

    if !config["irc"].is_null() {
        data.irc_running_bridge = false.into();
        let channels = config["irc"]["channels"]
            .as_object()
            .expect("config.json: `irc.channels` must exist if `irc` exists");
        for (k, v) in channels {
            let irc_channel = k.to_owned();
            let dc_channel = v.as_u64().expect("config.json: values in `irc.channels` must be integers");
            data.irc_channel_map.insert(dc_channel, irc_channel.clone());
            data.irc_config.channels.push(irc_channel);
        }
        data.irc_config.server = Some(config["irc"]["server"]
            .as_str()
            .expect("config.json: `irc.server` must exist if `irc` exists")
            .to_owned()
        );
        data.irc_config.nickname = Some(config["irc"]["nickname"]
            .as_str()
            .expect("config.json: `irc.nickname` must exist if `irc` exists")
            .to_owned()
        );
        data.irc_config.use_tls = config["irc"]["use_tls"].as_bool();
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                // Add stuff here
            ],
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(
                    ctx, event, framework, user_data,
                ))
            },
            ..Default::default()
        })
        .token(token)
        .intents(serenity_prelude::GatewayIntents::MESSAGE_CONTENT | serenity_prelude::GatewayIntents::GUILD_MESSAGES)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(data) }));

    framework.run().await.unwrap();
}



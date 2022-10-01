use poise::serenity_prelude::{self, json};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {}

use poise::serenity_prelude::{Activity, OnlineStatus};
use tokio::io::AsyncReadExt;

pub async fn event_listener(
    _ctx: &serenity_prelude::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    /*
        Runs an event listener using Serenity's built-in listener to set the status and presence to online
    */
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is connected!", data_about_bot.user.name);

            let activity = Activity::playing("vim");
            let status = OnlineStatus::Online;

            _ctx.set_presence(Some(activity), status).await;
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
    let _config: json::Value = json::prelude::from_str(&buf).expect("config.json contained invalid JSON");
    
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
        .intents(serenity_prelude::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));

    framework.run().await.unwrap();
}



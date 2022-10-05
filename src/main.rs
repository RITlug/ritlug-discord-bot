use poise::serenity_prelude;

mod commands;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub struct Data {}

use poise::serenity_prelude::{Activity, OnlineStatus};

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
    let token = "MTAwOTk2MjQ1NzQzMjg1MDQzMg.GS9IWG.cgx03rgzvwsx3pi9BMXqlINDDWuBY9o08qzXcg";
    
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![
                commands::ping()
            ],
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(
                    ctx, event, framework, user_data,
                ))
            },
            ..Default::default()
        })
        .token(token)
        .intents(serenity_prelude::GatewayIntents::MESSAGE_CONTENT)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));

    framework.run().await.unwrap();
}



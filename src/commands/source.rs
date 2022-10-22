use crate::{Context, Error};

/// Pong!
#[poise::command(slash_command)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Source code for this bot can be accessed at https://github.com/RITlug/ritlug-discord-bot").await?;
    Ok(())
}

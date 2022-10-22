use crate::{Context, Error};

/// Get the link to the source code for the bot
#[poise::command(slash_command)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Source code for this bot can be accessed at https://github.com/RITlug/ritlug-discord-bot").await?;
    Ok(())
}

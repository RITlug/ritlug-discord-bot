use crate::{Context, Error};

/// Get the link to the source code for the bot
#[poise::command(slash_command)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This bot is fully closed-source and proprietary, as Bill Gates and Steve Ballmer intended.").await?;
    Ok(())
}

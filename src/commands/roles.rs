use poise::serenity_prelude as serenity;

use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn addrole(
    ctx: Context<'_>,
    #[description = "Role to add"] role: serenity::Role,
    #[description = "Page to add role too"] page: u32,
) -> Result<(), Error> {
    if role.name == "@everyone" {
      ctx.say("ROLE IS EVERYONE!!").await?;
    }
    let response = format!("adding role <@&{}> to page {}", role.id, page);
    ctx.say(response).await?;
    Ok(())
}
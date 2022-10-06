use poise::serenity_prelude as serenity;

use crate::{Context, Error};

use crate::database;
use crate::util;

#[poise::command(slash_command)]
pub async fn addrole(
    ctx: Context<'_>,
    #[description = "Role to add"] role: serenity::Role,
    #[description = "Page to add role too"] page: u64,
) -> Result<(), Error> {
    if role.name == "@everyone" || role.name == "@here" {
      util::error(&ctx, ":x: Role is not allowed").await?;
      return Ok(())
    }
    let guild_id = util::get_guild_id(&ctx).await?;
    
    let data = database::get_page(&guild_id, &page)?;
    
    let response;
    match data {
      Some(x) => response = format!("page {} data is: {}", page, x),
      None => response = format!("page {} not found", page)
    }

    ctx.say(response).await?;
    Ok(())
}
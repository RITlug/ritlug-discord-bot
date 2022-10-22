use poise::serenity_prelude as serenity;

use crate::{Context, Error};

pub async fn error(ctx: &Context<'_>, msg: &str) -> Result<(), Error> {
  ctx.send(|b| b.ephemeral(true).embed(|b| b.description(msg).color(serenity::Color::RED))).await?;
  Ok(())
}

pub async fn critical_error(ctx: &Context<'_>, msg: &str) -> Result<(), Error> {
  ctx.send(|b| b.ephemeral(true).embed(|b| b.title(":x: Fatal Error").description(msg).color(serenity::Color::RED))).await?;
  Ok(())
}

pub async fn get_guild_id(ctx: &Context<'_>) -> Result<u64, String> {
  match ctx.guild_id() {
    Some(x) => Ok(x.0),
    None => {
      error(&ctx, ":x: Failed to fetch guild id").await.unwrap();
      return Err("Failed to fetch guild id".to_string())
    }
  }
}
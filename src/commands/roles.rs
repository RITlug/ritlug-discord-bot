use poise::serenity_prelude as serenity;
use serde_json::json;

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
    let mut json: serde_json::Value;
    match data {
      None => {
        util::error(&ctx, format!(":x: Page {} does not exist", guild_id).as_str()).await?;
        return Ok(());
      },
      Some(x) => json = serde_json::from_str(x.as_str())?
    }

    let mut roles = json["roles"].as_array().unwrap_or(&Vec::new()).to_owned();
    for value in &roles {
      match value.as_u64() {
        None => continue,
        Some(id) => {
          if id == role.id.0 {
            util::error(&ctx, ":x: Role is already on page").await?;
            return Ok(())
          }
        }
      }
    }

    roles.push(serde_json::Value::from(role.id.0));
    json["roles"] = json!(roles);

    database::set_page(&guild_id, &page, json.to_string().as_str())?;

    ctx.send(|b| b.content(format!(":white_check_mark: Sucessfully added role <@&{}> to page {}", &role.id.0, &page))).await?;

    Ok(())
}
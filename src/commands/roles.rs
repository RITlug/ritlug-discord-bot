use poise::CreateReply;
use poise::futures_util::StreamExt;
use poise::serenity_prelude::{self as serenity, InteractionResponseType};
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
    
    let data = database::roles::get_page(&guild_id, &page)?;
    let mut json: serde_json::Value;
    match data {
      None => {
        util::error(&ctx, format!(":x: Page {} does not exist", guild_id).as_str()).await?;
        return Ok(());
      },
      Some(x) => json = serde_json::from_str(x.as_str())?
    }

    let mut ids = json["roles"].as_array().unwrap_or(&Vec::new()).to_owned().iter().map(|value| value.as_u64()).flatten().collect::<Vec<_>>();

    if ids.contains(&role.id.0) {
      util::error(&ctx, ":x: Role is already on page").await?;
      return Ok(())
    }

    ids.push(role.id.0);

    let roles = ids.iter().map(|value| serde_json::Value::from(value.to_owned())).collect::<Vec<_>>();
    json["roles"] = json!(roles);

    database::roles::set_page(&guild_id, &page, json.to_string().as_str())?;

    ctx.send(|b| b.content(format!(":white_check_mark: Sucessfully added role <@&{}> to page {}", &role.id.0, &page))).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn deleterole(
    ctx: Context<'_>,
    #[description = "Role to delete"] role: serenity::Role,
    #[description = "Page to delete role from"] page: u64,
) -> Result<(), Error> {
    if role.name == "@everyone" || role.name == "@here" {
      util::error(&ctx, ":x: Role is not allowed").await?;
      return Ok(())
    }
    let guild_id = util::get_guild_id(&ctx).await?;
    
    let data = database::roles::get_page(&guild_id, &page)?;
    let mut json: serde_json::Value;
    match data {
      None => {
        util::error(&ctx, format!(":x: Page {} does not exist", guild_id).as_str()).await?;
        return Ok(());
      },
      Some(x) => json = serde_json::from_str(x.as_str())?
    }

    let mut ids = json["roles"].as_array().unwrap_or(&Vec::new()).to_owned().iter().map(|value| value.as_u64()).flatten().collect::<Vec<_>>();

    if !ids.contains(&role.id.0) {
      util::error(&ctx, ":x: Role is already not on page").await?;
      return Ok(())
    }

    ids.retain(|e| e != &role.id.0);

    let roles = ids.iter().map(|value| serde_json::Value::from(value.to_owned())).collect::<Vec<_>>();
    json["roles"] = json!(roles);

    database::roles::set_page(&guild_id, &page, json.to_string().as_str())?;

    ctx.send(|b| b.content(format!(":white_check_mark: Sucessfully removed role <@&{}> from page {}", &role.id.0, &page))).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn addrolepage(
    ctx: Context<'_>,
    #[description = "Title of the page"] title: String,
    #[description = "Description of the page"] description: String,
) -> Result<(), Error> {
  let guild_id = util::get_guild_id(&ctx).await?;
  let page_count = database::roles::get_page_amount(&guild_id)?.unwrap_or(0);
  let json = format!("{{\"title\":\"{}\",\"description\":\"{}\",\"roles\":[]}}", &title, &description);
  
  database::roles::set_page(&guild_id, &(page_count+1), json.as_str())?;

  ctx.send(|b| b.content(format!(":white_check_mark: Sucessfully added page {} as page number {}", &title, &(page_count+1)))).await?;
  
  Ok(())
}

#[poise::command(slash_command)]
pub async fn deleterolepage(
    ctx: Context<'_>,
    #[description = "Page to delete"] page: u64,
) -> Result<(), Error> {
  let guild_id = util::get_guild_id(&ctx).await?;
  let page_count = database::roles::get_page_amount(&guild_id)?.unwrap_or(0);

  if page_count < page || page < 1{
    util::error(&ctx, ":x: That page doesnt exist").await?;
    return Ok(())
  }
  
  database::roles::delete_page(&guild_id, &page)?;

  ctx.send(|b| b.content(format!(":white_check_mark: Sucessfully deleted page number {}", &page))).await?;
  
  Ok(())
}

#[poise::command(slash_command)]
pub async fn roles(
    ctx: Context<'_>
) -> Result<(), Error> {
  let guild_id = util::get_guild_id(&ctx).await?;
  let page_count = database::roles::get_page_amount(&guild_id)?.unwrap_or(0);

  if page_count < 1 {
    util::error(&ctx, ":x: Self roles arent enabled on this server").await?;
    return Ok(())
  }

  let page_number = 1;
  let content = get_role_embed(&ctx, &guild_id, &page_number, &page_count)?;
  let reply = ctx.send(|b| { b.clone_from(&content); return b; }).await?;
  let message = reply.message().await?;
  let mut collector = message.await_component_interactions(ctx.discord()).author_id(ctx.author().id).build();

  while let Some(interaction) = collector.next().await {
    match interaction.data.custom_id.as_str() {
      "previous" => {
        let page_count = database::roles::get_page_amount(&guild_id)?.unwrap_or(0);
        let mut page_number = interaction.message.embeds[0].footer.as_ref().unwrap().text[5..6].parse::<u64>().unwrap() - 1;
        if page_number < 1 { page_number = page_count; }
        if page_number > page_count { page_number = 1 }
        let content = get_role_embed(&ctx, &guild_id, &page_number, &page_count)?;
        interaction.create_interaction_response(
          ctx.discord(), |b| b
          .kind(InteractionResponseType::UpdateMessage)
          .interaction_response_data(|b| b.set_embeds(content.embeds).set_components(content.components.unwrap()))
        ).await?;
      }
      "next" => {
        let page_count = database::roles::get_page_amount(&guild_id)?.unwrap_or(0);
        let mut page_number = interaction.message.embeds[0].footer.as_ref().unwrap().text[5..6].parse::<u64>().unwrap() + 1;
        if page_number < 1 { page_number = page_count; }
        if page_number > page_count { page_number = 1 }
        let content = get_role_embed(&ctx, &guild_id, &page_number, &page_count)?;
        interaction.create_interaction_response(
          ctx.discord(), |b| b
          .kind(InteractionResponseType::UpdateMessage)
          .interaction_response_data(|b| b.set_embeds(content.embeds).set_components(content.components.unwrap()))
        ).await?;
      }
      "selection" => {
        let id = interaction.data.values[0].as_str().parse::<u64>().ok();
        match id {
          None => {
            interaction.create_interaction_response(ctx.discord(), |b| b
                  .interaction_response_data(|b| b.ephemeral(true).content(":x: Page empty or invalid role"))).await?;
          }
          Some(x) => {
            let lookup = ctx.guild().unwrap().roles;
            let role_id = serenity::RoleId(x);
            let role_option = lookup.get(&role_id);
            match role_option {
              None => {
                interaction.create_interaction_response(ctx.discord(), |b| b
                  .interaction_response_data(|b| b.ephemeral(true).content(":x: Role no longer exists on server"))).await?;
              }
              Some(role) => {
                let mut member = interaction.as_ref().to_owned().member.unwrap();
                if member.roles.contains(&role_id) {
                  member.remove_role(ctx.discord(), &role_id).await?;
                  interaction.create_interaction_response(ctx.discord(), |b| b
                    .interaction_response_data(|b| b.ephemeral(true).content(format!(":white_check_mark: Sucessfully removed <@&{}>", role.id.0)))).await?;
                } else {
                  member.add_role(ctx.discord(), &role_id).await?;
                  interaction.create_interaction_response(ctx.discord(), |b| b
                    .interaction_response_data(|b| b.ephemeral(true).content(format!(":white_check_mark: Sucessfully added <@&{}>", role.id.0)))).await?;
                }
              }
            }
          }
        }
      }
      "exit" => {
        interaction.message.delete(ctx.discord()).await?;
        return Ok(())
      }
      _ => {}
    }
  }
  
  Ok(())
}

fn get_role_embed<'a>(ctx: &'a Context<'a>, &guild_id: &'a u64, page_number: &'a u64, page_count: &'a u64) -> Result<CreateReply<'a>, Error> {

  let binding = database::roles::get_page(&guild_id, &page_number)?.unwrap();
  let data = binding.as_str();

  let json: serde_json::Value = serde_json::from_str(data)?;
  let ids = json["roles"].as_array().unwrap_or(&Vec::new()).to_owned().iter().map(|value| value.as_u64()).flatten().collect::<Vec<_>>();
  let title = json["title"].as_str().unwrap();
  let mut description = json["description"].as_str().unwrap().to_string();

  let lookup = ctx.guild().unwrap().roles;
  let roles = ids.iter().map(|&id| lookup.get(&serenity::RoleId(id))).flatten().collect::<Vec<_>>();

  let mut options:Vec<serenity::CreateSelectMenuOption> = Vec::new();
  description.push_str("\n");
  let mut i = 1;
  for &role in &roles {
    options.push(serenity::CreateSelectMenuOption::new(
      role.name.as_str(),
      role.id.0
    ));
    description.push_str(format!("\n`{}`: <@&{}>", &i, &role.id.0).as_str());
    i += 1;
  }

  if options.len() < 1 {
    options.push(serenity::CreateSelectMenuOption::new(
      "There are no roles on this page",
      "empty"
    ));
  }

  let reply = CreateReply::default()
    .embed(|b| b
      .author(|b| b
        .name("Role Selection") 
      )
      .title(title)
      .description(description)
      .footer(|b| b
        .text(format!("Page {}/{}", &page_number, &page_count))
      )
    ).components(|b| b
      .create_action_row(|b| b
        .create_button(|b| b
          .custom_id("previous")
          .label("Previous")
          .style(serenity::ButtonStyle::Secondary)
        )
        .create_button(|b| b
          .custom_id("next")
          .label("Next")
          .style(serenity::ButtonStyle::Secondary)
        )
        .create_button(|b| b
          .custom_id("exit")
          .label("Exit")
          .style(serenity::ButtonStyle::Danger)
        )
      ).create_action_row(|b| b
        .create_select_menu(|b| b
          .custom_id("selection")
          .placeholder("Select a role to add")
          .options(|b| b.set_options(options))
        )
      )
    ).to_owned();

  Ok(reply)

} 
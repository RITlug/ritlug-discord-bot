use poise::serenity_prelude::{self as serenity};
use regex::Regex;

use crate::{Context, Error};

use crate::database;
use crate::util;
use crate::smtp;

use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::sync::Mutex;
use rand::Rng;

lazy_static! {
    static ref MAP: Mutex<HashMap<u64, i32>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

#[poise::command(slash_command, subcommands("confirm"))]
pub async fn verify(
    ctx: Context<'_>,
    #[description = "f"] email: String
) -> Result<(), Error> {

    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&email) {
        util::error(&ctx, format!(":x: Invalid email").as_str()).await?;
        return Ok(());
    }

    let handle = "@rit.edu";
    if email[email.len()-handle.len()-1..email.len()-1].to_string() != handle {
        util::error(&ctx, format!(":x: Email must be a valid {} email", handle).as_str()).await?;
        return Ok(());
    }

    let guild_id = util::get_guild_id(&ctx).await?;
    let role_option = database::guild::get_setting(&guild_id, "verify_role")?;
    let role_id;

    match role_option {
        None => {
            util::error(&ctx, format!(":x: Verification isnt enabled on this server").as_str()).await?;
            return Ok(());
        }
        Some(role_str) => {
            role_id = role_str.parse::<u64>()?;
        }
    }

    let member_option = ctx.author_member().await;
    let member;
    match member_option {
        None => {
            util::error(&ctx, format!(":x: Failed to get member information").as_str()).await?;
            return Ok(());
        }
        Some(m) => member = m
    }

    if member.roles.contains(&serenity::RoleId(role_id)) {
        util::error(&ctx, format!(":x: You are already verified on this server").as_str()).await?;
        return Ok(());
    }

    let user_id = ctx.author().id.0;

    match database::auth::get_user(&guild_id, &user_id)? {
        None => {},
        Some(row) => {
            if row.email == email {
                util::error(&ctx, format!(":x: Email is already verified!").as_str()).await?;
                return Ok(());
            }
        }
    }

    let pin = rand::thread_rng().gen_range(10000000..99999999);
    let mut map = MAP.lock().await;
    map.insert(user_id, pin);

    smtp::send_email(&email, "Discord Authentication", format!("Auth Pin: {}", &pin).as_str())?;

    ctx.send(|b| b.ephemeral(true).content(":white_check_mark: Sucessfully sent authentication email")).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn confirm(
    ctx: Context<'_>,
    #[description = "f"] pin: i32
) -> Result<(), Error> {

    let guild_id = util::get_guild_id(&ctx).await?;
    let role_option = database::guild::get_setting(&guild_id, "verify_role")?;
    let role_id;

    match role_option {
        None => {
            util::error(&ctx, format!(":x: Verification isnt enabled on this server").as_str()).await?;
            return Ok(());
        }
        Some(role_str) => {
            role_id = role_str.parse::<u64>()?;
        }
    }

    let mut map = MAP.lock().await;
    let user_id = ctx.author().id.0;
    let real_pin;
    if map.contains_key(&user_id) { 
        real_pin = map.get(&user_id).unwrap();
    } else {
        util::error(&ctx, format!(":x: You do not have a verification request queued").as_str()).await?;
        return Ok(());
    }

    if pin != *real_pin {
        util::error(&ctx, format!(":x: Incorrect pin!").as_str()).await?;
        return Ok(());
    }

    let member_option = ctx.author_member().await;
    let mut member;
    match member_option {
        None => {
            util::error(&ctx, format!(":x: Failed to get member information").as_str()).await?;
            return Ok(());
        }
        Some(m) => member = m
    }

    match member.to_mut().add_role(ctx.discord(), &serenity::RoleId(role_id)).await {
        Err(_) => {
            util::error(&ctx, format!(":x: Failed to add verified role").as_str()).await?;
            return Ok(());
        }
        Ok(_) => {}
    }

    map.remove(&user_id).unwrap();
    ctx.send(|b| b.ephemeral(true).content(":white_check_mark: You have been sucessfully verified!")).await?;

    Ok(())
}
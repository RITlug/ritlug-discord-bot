use poise::serenity_prelude::{self as serenity};
use regex::Regex;

use crate::{Context, Error};

use crate::database;
use crate::util;
use crate::smtp;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use rand::Rng;

struct Auth {
    email: String,
    pin: i32
}

lazy_static! {
    static ref MAP: Mutex<HashMap<u64, Auth>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

#[poise::command(slash_command, subcommands("request", "toggle", "purge"))]
pub async fn verify(
    _ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, subcommands("code", "confirm"))]
pub async fn request(
    _ctx: Context<'_>
) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn code(
    ctx: Context<'_>,
    #[description = "Email to send pin to"] email: String
) -> Result<(), Error> {

    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&email) {
        util::error(&ctx, format!(":x: Invalid email").as_str()).await?;
        return Ok(());
    }

    let handle = "@rit.edu";
    if email[email.len()-handle.len()..].to_string() != handle {
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

    match database::auth::get_email(&guild_id, &email)? {
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
    let copy = email.clone();
    map.insert(user_id, Auth{email: copy, pin});

    let reply = ctx.send(|b| b.ephemeral(true).content(":wrench: Sending authentication email")).await?;

    match smtp::send_email(&email, "Discord Authentication", format!("Auth Pin: {}", &pin).as_str()) {
        Ok(_) => { 
            reply.edit(ctx, |b| b.ephemeral(true).content(":white_check_mark: Sucessfully sent aithentication email")).await?;
         }
        Err(e) => {
            reply.edit(ctx, |b| b.ephemeral(true).content(format!(":x: Failed to send email, {}", e.to_string()))).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn confirm(
    ctx: Context<'_>,
    #[description = "Verify pin sent to your email"] pin: i32
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

    if pin != real_pin.pin {
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
        Err(e) => {
            util::error(&ctx, format!(":x: Failed to add verified role, {}", e.to_string()).as_str()).await?;
            return Ok(());
        }
        Ok(_) => {}
    }

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let now = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    database::auth::set_user(&guild_id, &user_id, &real_pin.email, &now)?;

    map.remove(&user_id).unwrap();
    ctx.send(|b| b.ephemeral(true).content(":white_check_mark: You have been sucessfully verified!")).await?;

    Ok(())
}

#[poise::command(slash_command, subcommands("enable", "disable"))]
pub async fn toggle(
    _ctx: Context<'_>
) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_GUILD")]
pub async fn enable(
    ctx: Context<'_>,
    #[description = "Verify role to set"] role: serenity::Role,
) -> Result<(), Error> {

    if role.name == "@everyone" || role.name == "@here" {
        util::error(&ctx, ":x: Role is not allowed").await?;
        return Ok(())
    }
    let guild_id = util::get_guild_id(&ctx).await?;

    database::guild::set_setting(&guild_id, "verify_role", &format!("{}",&role.id.0))?;

    ctx.say(format!(":white_check_mark: Sucessfully enabled verfiy commands! Verfiy role: <@&{}>", &role.id.0)).await?;

    Ok(())

}

#[poise::command(slash_command, required_permissions="MANAGE_GUILD")]
pub async fn disable(
    ctx: Context<'_>
) -> Result<(), Error> {

    let guild_id = util::get_guild_id(&ctx).await?;

    database::guild::delete_setting(&guild_id, "verify_role")?;

    ctx.say(":white_check_mark: Sucessfully disabled verfiy commands").await?;

    Ok(())

}

#[poise::command(slash_command, subcommands("email", "user"))]
pub async fn purge(
    _ctx: Context<'_>
) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_GUILD")]
pub async fn email(
    ctx: Context<'_>,
    #[description = "Email to purge"] email: String,
) -> Result<(), Error> {

    let guild_id = util::get_guild_id(&ctx).await?;

    let row = database::auth::get_email(&guild_id, &email)?;
    match row {
        None => {
            util::error(&ctx, ":x: Email doesn't exist in system").await?;
        }
        Some(data) => {
            database::auth::delete_user(&guild_id, &data.user_id)?;
            ctx.say(format!(":white_check_mark: Sucessfully purged {} from the database", &email)).await?;
        }
    }

    Ok(())

}

#[poise::command(slash_command, required_permissions="MANAGE_GUILD")]
pub async fn user(
    ctx: Context<'_>,
    #[description = "User to purge"] member: serenity::Member,
) -> Result<(), Error> {

    let guild_id = util::get_guild_id(&ctx).await?;

    let row = database::auth::get_user(&guild_id, &member.user.id.0)?;
    match row {
        None => {
            util::error(&ctx, ":x: User doesn't exist in system").await?;
        }
        Some(data) => {
            database::auth::delete_user(&guild_id, &data.user_id)?;
            ctx.say(format!(":white_check_mark: Sucessfully purged <@{}> from the database", &member.user.id.0)).await?;
        }
    }

    Ok(())

}
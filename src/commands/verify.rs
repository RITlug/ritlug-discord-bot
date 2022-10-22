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

/// Verify yourself using your email address.
#[poise::command(slash_command, subcommands("request", "confirm", "purge"))]
pub async fn verify(
    _ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

/// Request to be verified. Specify the email you want to use as an argument.
#[poise::command(slash_command)]
pub async fn request(
    ctx: Context<'_>,
    #[description = "Email to send pin to"] mut email: String
) -> Result<(), Error> {

    // If the role id isnt set in config.json, verification isnt enabled
    if ctx.data().verify_role < 1 {
        util::error(&ctx, format!(":x: Verification isn't enabled on this server").as_str()).await?;
        return Ok(());
    }

    if email.len() > 320 {
        util::error(&ctx, format!(":x: Email is too long").as_str()).await?;
        return Ok(());
    }

    email = email.to_lowercase();

    // Check if email is a valid email
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&email) {
        util::error(&ctx, format!(":x: Invalid email").as_str()).await?;
        return Ok(());
    }

    // Check if the given email's domain is accepted in the server
    let handles = &ctx.data().verify_emails;
    let mut equals = false;
    let index = email.find("@").unwrap();
    for handle in handles {
        if email[index+1..].to_string().eq(handle) {
            equals = true;
            break;
        }
    }

    // Error if email's domain is not accepted
    if !equals || email.find("+").is_some() {
        util::error(&ctx, ":x: That email domain is not accepted in this server").await?;
        return Ok(());
    }

    let guild_id = util::get_guild_id(&ctx).await?;
    let role_id = ctx.data().verify_role;

    // Get member object from guild
    let member_option = ctx.author_member().await;
    let member;
    match member_option {
        None => {
            util::error(&ctx, format!(":x: Failed to get member information").as_str()).await?;
            return Ok(());
        }
        Some(m) => member = m
    }

    // Error if the user has already verified in the past
    if member.roles.contains(&serenity::RoleId(role_id)) {
        util::error(&ctx, format!(":x: You are already verified on this server").as_str()).await?;
        return Ok(());
    }

    let user_id = ctx.author().id.0;

    // Error if the email has already been used to verifiy someone else in the past
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

    // Notify that the email is being sent
    let reply = ctx.send(|b| b.ephemeral(true).content(":wrench: Sending authentication email")).await?;

    // Attempt to send the authentication email with the pin
    match smtp::send_email(&email, "Discord Authentication", format!("Auth Pin: {}", &pin).as_str()) {
        Ok(_) => { 
            // Email sent sucessfully
            reply.edit(ctx, |b| b.ephemeral(true).content(":white_check_mark: Sucessfully sent authentication email")).await?;
         }
        Err(e) => {
            // Failed to send email, show error
            reply.edit(ctx, |b| b.ephemeral(true).content(format!(":x: Failed to send email, {}", e.to_string()))).await?;
        }
    }

    Ok(())
}

/// Confirm your verification by providing the PIN sent to your email.
#[poise::command(slash_command)]
pub async fn confirm(
    ctx: Context<'_>,
    #[description = "Verify pin sent to your email"] pin: i32
) -> Result<(), Error> {

    // If the role id isnt set in config.json, verification isnt enabled
    if ctx.data().verify_role < 1 {
        util::error(&ctx, format!(":x: Verification isn't enabled on this server").as_str()).await?;
        return Ok(());
    }

    let guild_id = util::get_guild_id(&ctx).await?;
    let role_id = ctx.data().verify_role;

    // Check if the user who send the confirm request really has a request pending
    let mut map = MAP.lock().await;
    let user_id = ctx.author().id.0;
    let real_pin;
    if map.contains_key(&user_id) { 
        real_pin = map.get(&user_id).unwrap();
    } else {
        util::error(&ctx, format!(":x: You do not have a verification request queued").as_str()).await?;
        return Ok(());
    }

    // Error if the pin is incorrect
    if pin != real_pin.pin {
        util::error(&ctx, format!(":x: Incorrect pin!").as_str()).await?;
        return Ok(());
    }

    // Error if unable to get member information
    let member_option = ctx.author_member().await;
    let mut member;
    match member_option {
        None => {
            util::error(&ctx, format!(":x: Failed to get member information").as_str()).await?;
            return Ok(());
        }
        Some(m) => member = m
    }

    // Attempty to add verify role to user, error if failed
    match member.to_mut().add_role(ctx.discord(), &serenity::RoleId(role_id)).await {
        Err(e) => {
            util::error(&ctx, format!(":x: Failed to add verified role, {}", e.to_string()).as_str()).await?;
            return Ok(());
        }
        Ok(_) => {}
    }

    // Get current time in miliseconds
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let now = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    // Update database information
    database::auth::set_user(&guild_id, &user_id, &real_pin.email, &now)?;

    map.remove(&user_id).unwrap();
    ctx.send(|b| b.ephemeral(true).content(":white_check_mark: You have been sucessfully verified!")).await?;

    Ok(())
}

/// Purge verified users
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
            if let Some(guild) = ctx.guild() {
                if let Ok(mut member) = guild.member(ctx.discord(), data.user_id).await {
                    let _ = member.remove_role(ctx.discord(), &serenity::RoleId(ctx.data().verify_role)).await;
                }
            }
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
            if let Some(guild) = ctx.guild() {
                if let Ok(mut member) = guild.member(ctx.discord(), data.user_id).await {
                    let _ = member.remove_role(ctx.discord(), &serenity::RoleId(ctx.data().verify_role)).await;
                }
            }
        }
    }

    Ok(())

}

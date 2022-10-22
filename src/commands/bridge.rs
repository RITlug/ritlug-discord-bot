/// Show information about where this channel is bridged to.
/// 
/// "Bridging" a channel to somewhere else means that any messages sent within
/// the channel are automatically sent to that place, and vice versa.
#[poise::command(slash_command)]
pub async fn bridge(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    let dc_channel = ctx.channel_id();
    let dc_channel = dc_channel.as_u64();
    let irc_channel = ctx.data().irc_channel_map.get_by_left(&dc_channel);
    let msg = match irc_channel {
        Some(ch) => format!("Bridged to `{}` at `{}`", ch, ctx.data().irc_config.server.as_ref().unwrap()),
        None => format!("Not bridged to an IRC channel.")
    };
    ctx.say(msg).await?;
    Ok(())
}

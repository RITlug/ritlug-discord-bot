use std::sync::Arc;

use bimap::BiMap;
use poise::serenity_prelude::{Context, ChannelId};

use crate::Error;

use super::{Receiver, formatting::irc_to_dc};

// Send bridged messages to Discord
pub async fn run_bridge(
    ctx: Arc<Context>,
    mut rx: Receiver,
    channel_mapping: BiMap<u64, String>
) -> Result<(), Error> {
    while let Some(msg) = rx.recv().await {
        let channel = channel_mapping.get_by_right(&msg.channel);
        
        if let Some(channel) = channel {
            let channel = ChannelId::from(*channel);
            let message = irc_to_dc(&msg.message);
            let mut webhooks = channel.webhooks(&ctx.http).await?;

            // Create a new webhook for this channel if none exist
            let webhook = if webhooks.is_empty() {
                channel.create_webhook(&ctx.http, "IRC").await?
            } else {
                webhooks.swap_remove(0) // get 0th webhook without cloning
            };

            // Send a message via the webhook
            webhook.execute(&ctx.http, false, |hook| {
                hook.username(msg.author).content(message)
            }).await?;
        }
    }
    Ok(())
}

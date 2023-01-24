use std::sync::Arc;

use bimap::BiMap;
use poise::serenity_prelude::{Context, ChannelId};

use crate::Error;

use super::Receiver;

// Send bridged messages to Discord
pub async fn run_bridge(
    ctx: Arc<Context>,
    mut rx: Receiver,
    channel_mapping: BiMap<u64, String>,
    avatar_url: String,
) -> Result<(), Error> {
    while let Some(msg) = rx.recv().await {
        let channel = channel_mapping.get_by_right(&msg.channel);
        
        if let Some(channel) = channel {
            let channel = ChannelId::from(*channel);
            let webhook = channel.create_webhook(&ctx.http, "IRC").await?;

            // Send a message via the webhook
            webhook.execute(&ctx.http, false, |hook| {
                hook.username(msg.author)
                    .content(msg.message)
                    .avatar_url(&avatar_url)
            }).await?;
        }
    }
    Ok(())
}

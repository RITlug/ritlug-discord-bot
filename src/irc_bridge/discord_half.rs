use std::sync::Arc;

use bimap::BiMap;
use poise::serenity_prelude::{Context, ChannelId};

use crate::Error;

use super::{Receiver, formatting::irc_to_dc};

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
            let webhooks = channel.webhooks(&ctx.http).await.unwrap();
            println!("{}", webhooks.len());
            if webhooks.len() > 0 {
                webhooks[0].execute(&ctx.http, false, |hook| {
                    hook.username(msg.author).content(message)
                }).await?;
            } else {
                let webhook = channel.create_webhook(&ctx.http, "IRC").await?;
                webhook.execute(&ctx.http, false, |hook| {
                    hook.username(msg.author).content(message)
                }).await?;
            }
        }
    }
    Ok(())
}

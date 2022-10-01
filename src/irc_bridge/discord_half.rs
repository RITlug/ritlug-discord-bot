use std::sync::Arc;

use bimap::BiMap;
use poise::serenity_prelude::{Context, ChannelId};

use crate::Error;

use super::Receiver;

pub async fn run_bridge(
    ctx: Arc<Context>,
    mut rx: Receiver,
    channel_mapping: BiMap<u64, String>
) -> Result<(), Error> {
    while let Some(msg) = rx.recv().await {
        let channel = channel_mapping.get_by_right(&msg.channel);
        if let Some(channel) = channel {
            let message = format!("<{}> {}", msg.author, msg.message);
            ChannelId::from(*channel).say(&ctx.http, message).await?;
        }
    }
    Ok(())
}

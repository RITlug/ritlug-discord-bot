use std::{sync::Arc, collections::HashMap};

use bimap::BiMap;
use poise::serenity_prelude::{Context, ChannelId, Webhook};

use crate::Error;

use super::Receiver;

// Send bridged messages to Discord
pub async fn run_bridge(
    ctx: Arc<Context>,
    mut rx: Receiver,
    channel_mapping: BiMap<u64, String>,
    avatar_url: String,
) -> Result<(), Error> {
    let mut webhook_map: HashMap<u64, Webhook> = HashMap::new();
    while let Some(msg) = rx.recv().await {
        let channel = channel_mapping.get_by_right(&msg.channel);
        
        if let Some(chanid) = channel {

            if !webhook_map.contains_key(chanid) {
                let channel = ChannelId::from(*chanid);
                let webhooks = channel.webhooks(&ctx.http).await?;
                let mut found_hook = false;
                for webhook in webhooks {
                    if webhook.token.is_none() {
                        let _ = webhook.delete(&ctx.http).await;
                    } else {
                        found_hook = true;
                        webhook_map.insert(*chanid, webhook);
                    }
                }
                if !found_hook {
                    let webhook = channel.create_webhook(&ctx.http, "IRC").await?;
                    webhook_map.insert(*chanid, webhook);
                }
            }
            
            let webhook = webhook_map.get(chanid).unwrap();

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

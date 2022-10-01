use std::sync::Arc;

use bimap::BiMap;
use irc::client::prelude::Config;
use poise::serenity_prelude::Context;
use tokio::sync::mpsc;

mod irc_half;
mod discord_half;


#[derive(Debug)]
pub struct BridgeMessage {
    pub author: String,
    pub channel: String,
    pub message: String
}

pub type Sender = mpsc::Sender<BridgeMessage>;
pub type Receiver = mpsc::Receiver<BridgeMessage>;

pub fn run(
    ctx: Arc<Context>, 
    irc_config: Config, 
    channel_mapping: BiMap<u64, String>
) -> Sender {
    // channel from discord to irc
    let (tx_di, rx_di) = mpsc::channel(64);
    // channel from irc to discord
    let (tx_id, rx_id) = mpsc::channel(64);
    tokio::spawn(async {
        if let Err(e) = irc_half::run_bridge(irc_config, rx_di, tx_id).await {
            println!("Error in IRC bridge: {}", e);
        }
    });
    tokio::spawn(async {
        if let Err(e) = discord_half::run_bridge(ctx, rx_id, channel_mapping).await {
            println!("Error in IRC bridge: {}", e);
        }
    });
    return tx_di;
}

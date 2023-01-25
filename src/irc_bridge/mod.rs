use std::{sync::Arc, collections::HashMap};

use bimap::BiMap;
use irc::client::prelude::Config;
use poise::serenity_prelude::{Context, json};
use regex::Regex;
use tokio::sync::mpsc;

use crate::Data;

mod irc_half;
mod discord_half;

#[derive(Clone, Debug)]
pub struct FlattenBridge {
    syntax: Regex,
    suffix: String,
}

// A message crossing the bridge
#[derive(Debug)]
pub struct BridgeMessage {
    pub author: String,
    pub channel: String,
    pub message: String
}

// Channel for sending and receiving `BridgeMessage`s
pub type Sender = mpsc::Sender<BridgeMessage>;
pub type Receiver = mpsc::Receiver<BridgeMessage>;

// Load user data with config options from the `irc` object in `config.json`
pub fn load_data_from_config(data: &mut Data, irc_config: &json::Value) {
    // Mark that the IRC bridge has not started yet
    data.irc_running_bridge = false.into();
    // Load channels into the channel map and the IRC config object
    let channels = irc_config["channels"]
        .as_object()
        .expect("config.json: `irc.channels` must exist if `irc` exists");
    for (k, v) in channels {
        let irc_channel = k.to_owned();
        let dc_channel = v.as_u64().expect("config.json: values in `irc.channels` must be integers");
        data.irc_channel_map.insert(dc_channel, irc_channel.clone());
        data.irc_config.channels.push(irc_channel);
    }
    // set the IRC server
    data.irc_config.server = Some(irc_config["server"]
        .as_str()
        .expect("config.json: `irc.server` must exist if `irc` exists")
        .to_owned()
    );
    // set the IRC channels
    data.irc_config.nickname = Some(irc_config["nickname"]
        .as_str()
        .expect("config.json: `irc.nickname` must exist if `irc` exists")
        .to_owned()
    );
    if irc_config["flatten_bridges"].is_object() {
        for (k, v) in irc_config["flatten_bridges"].as_object().unwrap() {
            let fb = FlattenBridge {
                syntax: Regex::new(v["syntax"].as_str().unwrap()).unwrap(),
                suffix: v["suffix"].as_str().unwrap().to_owned()
            };
            data.irc_flatten_bridges.insert(k.to_owned(), fb);
        }
    }
    // discord avatar URL
    data.irc_webhook_avatar = irc_config["avatar"].as_str().unwrap_or("").to_owned();
    // enable/disable TLS (default: enabled)
    data.irc_config.use_tls = irc_config["use_tls"].as_bool();
}

// Start running the IRC bridge. This function creates two new
// asynchronous tasks, one for the IRC client and one that
// transmits new messages to Discord. Recieving messages is
// done by the main program via the returned `Sender`.
pub fn run(
    ctx: Arc<Context>, 
    irc_config: Config, 
    channel_mapping: BiMap<u64, String>,
    flatten_bridges: HashMap<String, FlattenBridge>,
    avatar_url: String,
) -> Sender {
    // channel from discord to irc
    let (tx_di, rx_di) = mpsc::channel(64);
    // channel from irc to discord
    let (tx_id, rx_id) = mpsc::channel(64);
    tokio::spawn(async {
        if let Err(e) = irc_half::run_bridge(irc_config, rx_di, tx_id, flatten_bridges).await {
            println!("Error in IRC bridge: {}", e);
        }
    });
    tokio::spawn(async {
        if let Err(e) = discord_half::run_bridge(ctx, rx_id, channel_mapping, avatar_url).await {
            println!("Error in Discord bridge: {}", e);
        }
    });
    tx_di
}

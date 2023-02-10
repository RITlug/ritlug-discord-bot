use std::{time::Duration, collections::HashMap, str::FromStr};

use irc::client::prelude::*;
use poise::{futures_util::StreamExt};
use tokio::sync::mpsc;
use url::Url;

use crate::{Error, database::avatar};

use super::{BridgeMessage, FlattenBridge};

// Run the IRC client
pub async fn run_bridge(
    config: Config, 
    mut rx: mpsc::Receiver<BridgeMessage>, 
    tx: mpsc::Sender<BridgeMessage>,
    bridges: HashMap<String, FlattenBridge>,
) -> Result<(), Error> {
    loop {
        if let Err(e) = run_bridge_inner(config.clone(), &mut rx, &tx, &bridges).await {
            println!("Error in IRC bridge: {}. Reconnecting...", e);
        } else {
            println!("Connection to IRC ended. Reconnecting...");
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

pub async fn run_bridge_inner(
    config: Config, 
    rx: &mut mpsc::Receiver<BridgeMessage>, 
    tx: &mpsc::Sender<BridgeMessage>,
    bridges: &HashMap<String, FlattenBridge>,
) -> Result<(), Error> {
    let mut client = Client::from_config(config).await?;
    client.identify()?;
    println!("IRC bridge connected");
    
    let mut stream = client.stream()?;
    loop {
        tokio::select! {
            // New message from the IRC server
            msg = stream.next() 
            => if let Some(msg) = msg.transpose()? {
                match prepare_message(msg, bridges) {
                    PrepareMessageResult::Bridged(bridge_message)
                        => tx.send(bridge_message).await?,
                    PrepareMessageResult::Return(channel, content)
                        => client.send(Command::PRIVMSG(channel, content))?,
                    PrepareMessageResult::None => (),
                }
            } else {
                break
            },
            // Message from the bridge that needs to be sent to IRC
            bridge_msg = rx.recv() => if let Some(msg) = bridge_msg {
                for line in msg.message.split('\n') {
                    if !line.trim().is_empty() {
                        let content = format!("<{}> {}", msg.author, line);
                        client.send(Command::PRIVMSG(msg.channel.clone(), content))?;
                    }
                }
            } else {
                break
            }
        }
    }
    Ok(()) 
}

enum PrepareMessageResult {
    Bridged(BridgeMessage),
    Return(String, String),
    None
}

fn prepare_message(msg: Message, bridges: &HashMap<String, FlattenBridge>) -> PrepareMessageResult {
    if let Command::PRIVMSG(channel, mut message) = msg.command {
        let mut author = match msg.prefix {
            Some(Prefix::Nickname(nick, _user, _host)) => nick,
            Some(Prefix::ServerName(servname)) => servname,
            None => todo!(),
        };
        if let Some(bf) = bridges.get(&author) {
            if let Some(captures) = bf.syntax.captures(&message) {
                if captures.len() >= 3 {
                    author = captures[1].to_owned() + " [" + &bf.suffix + "]";
                    message = captures[2].to_owned()
                }
            }
        }
        // command
        if let Some(command) = message.strip_prefix("r!") {
            let mut parts = command.splitn(2, " ");
            let cmd = parts.next().unwrap_or_default();
            let rest = parts.next().unwrap_or_default();
            PrepareMessageResult::Return(channel, handle_command(author, cmd, rest))
        } else {
            PrepareMessageResult::Bridged(BridgeMessage {
                author, channel, message
            })
        }
    } else {
        PrepareMessageResult::None
    }
}

fn handle_command(author: String, command: &str, rest: &str) -> String {
    match command {
        "avatar" => {
            if rest == "" {
                if let Err(e) = avatar::update_avatar(&author, rest) {
                    return format!("Error: {}", e)
                } else {
                    return format!("Reset avatar for user {}", author)
                }
            }
            match Url::from_str(rest) {
                Ok(url) if url.scheme() == "http" || url.scheme() == "https" => {
                    if let Err(e) = avatar::update_avatar(&author, rest) {
                        format!("Error: {}", e)
                    } else {
                        format!("Updated avatar for user {}", author)
                    }
                }
                _ => "Invalid URL".to_owned()
            }
        },
        "help" => "Available commands: 'avatar [url]'".to_owned(),
        _ => "Invalid command.".to_owned(),
    }
}

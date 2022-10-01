use irc::client::prelude::*;
use poise::{futures_util::StreamExt};
use tokio::sync::mpsc;

use crate::Error;

use super::BridgeMessage;

pub async fn run_bridge(
    config: Config, 
    mut rx: mpsc::Receiver<BridgeMessage>, 
    tx: mpsc::Sender<BridgeMessage>,
) -> Result<(), Error> {
    let mut client = Client::from_config(config).await?;
    client.identify()?;
    println!("IRC bridge connected");
    
    let mut stream = client.stream()?;
    loop {
        tokio::select! {
            msg = stream.next() 
            => if let Some(msg) = msg.transpose()? {
                if let Command::PRIVMSG(channel, message) = msg.command {
                    let author = match msg.prefix {
                        Some(Prefix::Nickname(nick, _user, _host)) => nick,
                        Some(Prefix::ServerName(servname)) => servname,
                        None => todo!(),
                    };
                    tx.send(BridgeMessage {
                        author, channel, message
                    }).await?;
                }
            } else {
                break
            },
            bridge_msg = rx.recv() => if let Some(msg) = bridge_msg {
                let content = format!("<{}> {}", msg.author, msg.message);
                client.send(Command::PRIVMSG(msg.channel, content))?;
            } else {
                break
            }
        }
    }
    println!("IRC bridge closed");
    Ok(()) 
}
/*
 */

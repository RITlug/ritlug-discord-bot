use irc::client::prelude::*;
use poise::{futures_util::StreamExt};
use tokio::sync::mpsc;

use crate::Error;

use super::BridgeMessage;

// Run the IRC client
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
            // New message from the IRC server
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
    println!("IRC bridge closed");
    Ok(()) 
}

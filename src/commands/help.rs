use crate::{Error, Context};

/// Show a list of commands or information about a specific command.
/// 
/// Use `/help` without an argument to show a list of commands with short 
/// descriptions for each. Use `/help <command>` to show more detailed 
/// information about a specific command.
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    command: Option<String>,
) -> Result<(), Error> {

    match command {
        Some(cmd) => help_command(ctx, cmd).await?,
        None => help_general(ctx).await?
    }

    Ok(())
}

async fn help_general(ctx: Context<'_>) -> Result<(), Error> {
    let commands = ctx.framework().options.commands
        .iter()
        .filter(|c| !c.hide_in_help);
    
    let mut body = "Use `/help <command>` to view more detailed help information.\n\n".to_owned();
    for cmd in commands {
        body += "**`/";
        body += &cmd.name;
        body += "`**";
        if let Some(desc) = cmd.description.as_ref() {
            body += " - ";
            body += desc;
        }
        body.push('\n');
    }
    
    ctx.send(|b|
        b.embed(|e| {
            e.title("RITwug Bot Help");
            e.description(body);
            e
        }).ephemeral(true)
    ).await?;

    Ok(())
}

async fn help_command(ctx: Context<'_>, command: String) -> Result<(), Error> {
    let cmd = ctx.framework().options.commands.iter().find(|item| !item.hide_in_help && item.name == command);

    if let Some(cmd) = cmd {
        let mut body = cmd.description.clone().unwrap_or_else(|| String::new());
        if let Some(help) = cmd.help_text {
            body += "\n\n";
            body += &help();
        }

        let mut fields = vec![];

        let subcommands: Vec<_> = cmd.subcommands.iter().filter(|c| !c.hide_in_help).collect();
        if subcommands.len() != 0 {
            let mut text = String::new();
            for subcmd in subcommands {
                text += "**`/";
                text += &subcmd.name;
                text += "`**";
                if let Some(desc) = subcmd.description.as_ref() {
                    text += " - ";
                    text += desc;
                }
                text.push('\n');
            }
            fields.push(("Subcommands", text, false));
        }
        
        ctx.send(|b| 
            b.embed(|e| {
                e.title(format!("Help for command `/{}`", command));
                e.description(body);
                e.fields(fields);
                e
            }).ephemeral(true)
        ).await?;
    } else {
        ctx.send(|b| 
            b.content(
                format!("No command with the name `/{}`. Use `/help` for a list of commands.", command)
            ).ephemeral(true)
        ).await?;
    }

    Ok(())
}

/// I'd just like to interject for a moment...
#[poise::command(slash_command)]
pub async fn linux(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    let text = "https://learn.microsoft.com/en-us/windows/wsl/install";

    ctx.say(text).await?;
    Ok(())
}

/// I use Alpine!
#[poise::command(slash_command)]
pub async fn linuxresponse(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    let text = "https://www.youtube.com/watch?v=Vhh_GeBPOhs";
    
    ctx.say(text).await?;
    Ok(())
}

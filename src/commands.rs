use crate::{Context, Error};

#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Show help"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn gamble(
    ctx: Context<'_>,
    #[description = "How much to gamble (number or all)"] amount: String,
) -> Result<(), Error> {
    let value = match amount.parse::<u64>() {
        Ok(val) => val,
        Err(_) => match amount.contains("all") {
            true => 10,
            false => 0,
        },
    };
    let response;

    // Won
    if rand::random::<bool>() {
        response = format!("You won {}", value * 2);
    } else {
        response = format!("You lost :(");
    }
    ctx.say(response).await?;
    Ok(())
}

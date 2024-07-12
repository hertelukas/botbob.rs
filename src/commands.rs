use crate::models::*;
use crate::{Context, Error};
use diesel::prelude::*;
use poise::serenity_prelude as serenity;

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

#[poise::command(prefix_command, slash_command)]
pub async fn leaders(ctx: Context<'_>) -> Result<(), Error> {
    use crate::schema::users::dsl::*;
    let connection = &mut crate::establish_connection();

    let gamblers: Vec<User> = users
        .order(points.desc())
        .limit(5)
        .select(User::as_select())
        .load(connection)
        .expect("Error loading leaders");

    let mut msg: String = "Leaderboard\n".to_owned();

    for gambler in gamblers {
        msg.push_str(format!("{}: {} points\n", gambler.username, gambler.points).as_str());
    }

    ctx.say(msg.as_str()).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn points(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    use crate::schema::users::dsl::*;
    let connection = &mut crate::establish_connection();
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let gambler: Vec<User> = users
        .filter(id.eq(u.id.get() as i64))
        .select(User::as_select())
        .load(connection)
        .expect("Error loading user");

    let response;
    match gambler.first() {
        Some(g) => response = format!("{} has {} points", u.name, g.points),
        None => response = format!("User not found :("),
    }

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn gamble(
    ctx: Context<'_>,
    #[description = "How much to gamble (number or all)"] amount: String,
) -> Result<(), Error> {
    use crate::schema::users::dsl::*;

    let connection = &mut crate::establish_connection();

    let gambler: Vec<User> = users
        .filter(id.eq(ctx.author().id.get() as i64))
        .select(User::as_select())
        .load(connection)
        .expect("Error loading user");

    let available = match gambler.first() {
        Some(u) => u.points as u64,
        None => 100,
    };

    let value = match amount.parse::<u64>() {
        Ok(val) => val,
        Err(_) => match amount == "all" {
            true => available,
            // TODO handle as error
            false => 0,
        },
    };

    let value = std::cmp::min(available, value);

    let response;
    let gain;
    // Won
    if rand::random::<bool>() {
        response = format!("You won {} and now have {}", value * 2, available + value);
        gain = value as i64;
    } else {
        response = format!(
            "You lost and now have {}",
            (available as i64) - (value as i64)
        );
        gain = -(value as i64);
    }

    match gambler.first() {
        // Update
        Some(_) => {
            diesel::update(users.find(ctx.author().id.get() as i64))
                .set(points.eq(points + gain))
                .execute(connection)
                .unwrap();
        }
        // Insert
        None => {
            use crate::schema::users;
            let new_user = NewUser {
                id: ctx.author().id.get() as i64,
                points: 100 + gain,
                username: ctx.author().name.as_str(),
            };

            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(connection)
                .expect("Failed to insert new user");
        }
    }
    ctx.say(response).await?;
    Ok(())
}

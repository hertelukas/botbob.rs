mod commands;
mod models;
mod schema;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::SqliteConnection;
use models::User;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::async_trait;
use poise::serenity_prelude::EventHandler;
use poise::serenity_prelude::Message;
use schema::users;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: poise::serenity_prelude::Context, msg: Message) {
        use crate::schema::users::dsl::*;
        let connection = &mut establish_connection();
        diesel::update(users.find(msg.author.id.get() as i64))
            .set(points.eq(points + 1))
            .execute(connection)
            .unwrap();
    }
}

pub fn establish_connection() -> SqliteConnection {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");

    let mut s = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    let _ = s.batch_execute("PRAGMA busy_timeout = 4000;");
    s
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::help(),
                commands::gamble(),
                commands::points(),
                commands::leaders(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await;
    client.unwrap().start().await.unwrap();
}

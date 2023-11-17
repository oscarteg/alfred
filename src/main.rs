use dotenv::dotenv;
use log::error;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::event::ResumedEvent;
use serenity::model::id::GuildId;
use serenity::model::prelude::{Interaction, InteractionResponseType};
use serenity::model::Permissions;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;

mod commands;

/// Shard manager is used to restart the bot
/// This is used to restart the bot when a command is run
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(quit, am_i_admin)]
struct General;

#[group]
#[commands(dnd, quit)]
struct Dnd;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::imdb::register(command))
        })
        .await;

        println!(
            "Successfully registered application commands: {:?}",
            commands
        );
    }

    async fn resume(&self, _ctx: Context, _: ResumedEvent) {
        println!("Resumed");
    }
    async fn interaction_create(&self, _ctx: Context, _interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = _interaction {
            // Run the command
            let content = match command.data.name.as_str() {
                "imdb" => commands::imdb::run(&command.data.options),
                _ => "not implemented :(".to_string(),
            };

            // Log the reason the interaction commands failed
            if let Err(why) = command
                .create_interaction_response(&_ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("-")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP)
        .group(&DND_GROUP);

    dotenv().ok();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");

    // Initialize the logger to use environment variables.
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    // Start listening for events by starting a single shard.
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

// We could also use
// #[required_permissions(ADMINISTRATOR)]
// but that would not let us reply when it fails.
#[command]
async fn am_i_admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role
                .to_role_cached(&ctx.cache)
                .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
            {
                msg.channel_id.say(&ctx.http, "Yes, you are.").await?;

                return Ok(());
            }
        }
    }

    msg.channel_id.say(&ctx.http, "No, you are not.").await?;

    Ok(())
}

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;

        return Ok(());
    }

    Ok(())
}

#[command]
async fn dnd(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "dnd!").await?;

    Ok(())
}

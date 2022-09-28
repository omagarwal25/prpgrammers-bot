use dotenv::dotenv;
use once_cell::sync::OnceCell;
use poise::serenity_prelude::{self as serenity, Activity, OnlineStatus};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
    struct Data {}

pub static CTX: OnceCell<serenity::Context> = OnceCell::new();

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn event_listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    framework: poise::FrameworkContext<'_, Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is ready", data_about_bot.user.name);
            let _ = CTX
                .set(ctx.to_owned())
                .map_err(|_| println!("Global CTX already set"));
            println!("Setting presence");
            ctx.set_presence(
                Some(Activity::playing(format!("water game"))),
                OnlineStatus::Online,
            )
            .await;
        }
        poise::Event::ReactionAdd { add_reaction } => {
            if add_reaction.emoji.unicode_eq("📌") {
                let msg = add_reaction.message(&ctx).await?;
                msg.pin(&ctx).await?;
            }
        }
        poise::Event::ReactionRemove { removed_reaction } => {
            if removed_reaction.emoji.unicode_eq("📌") {
                if removed_reaction
                    .message(&ctx)
                    .await?
                    .reactions
                    .iter()
                    .find(|r| r.reaction_type.unicode_eq("📌"))
                    .is_none()
                {
                    removed_reaction.message(&ctx).await?.unpin(&ctx).await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![register()],
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(ctx, event, framework, user_data))
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));

    framework.run().await.unwrap();
}

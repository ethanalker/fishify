mod commands;

use std::env::var;
use poise::serenity_prelude as serenity;
use anyhow::{ Result, Error, };
use rspotify::{ 
    Credentials, OAuth, Config, scopes,
    AuthCodeSpotify, ClientError, 
    clients::{ BaseClient, OAuthClient, },
    http::HttpError,
};

type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    spotify: AuthCodeSpotify,
}

// error handler
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            if let Some(ClientError::Http(http)) = error.downcast_ref::<ClientError>() {
                if let HttpError::StatusCode(response) = http.as_ref() {
                    if response.status() == 404 {
                        println!("TODO: Try device connect and retry command");
                        // if success: return
                    }
                }
            }
            let errmsg = format!("Error in command `{}`: {:?}", ctx.command().name, error,);
            println!("{}", &errmsg);
            if let Err(e) = ctx.say(&errmsg).await {
                println!("Error while responding with error: {}", e);
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // spotify
    let creds = Credentials::from_env().expect("Missing `RSPOTIFY_CLIENT_ID` or `RSPOTIFY_CLIENT_SECRET` env var.");
    let oauth = OAuth::from_env(scopes!(
        "user-modify-playback-state", 
        "user-read-playback-state"
    )).expect("Missing `RSPOTIFY_REDIRECT_URI` env var.");
    let config = Config {
        token_refreshing: true,
        token_cached: true,
        cache_path: var("CACHE_PATH").expect("Missing `CACHE_PATH` env var.").into(),
        ..Default::default()
    };
    let spotify_auth = AuthCodeSpotify::with_config(creds, oauth, config);

    spotify_auth.refresh_token().await?;
    if spotify_auth.get_token().lock().await.unwrap().is_none() {
        let url = spotify_auth.get_authorize_url(false).unwrap();
        spotify_auth.prompt_for_token(&url).await.unwrap();
    }

    // discord
    let options = poise::FrameworkOptions {
        // commands go here
        commands: vec![
            commands::register(),
            commands::play(),
            commands::queue(),
            commands::search(),
            commands::queue_list(),
            commands::pause(),
            commands::skip(),
            commands::status(),
            commands::device_list(),
            commands::device_connect(),
            commands::device_status(),
            commands::set_volume(),
            commands::set_shuffle(),
            commands::set_repeat(),
        ],
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // use this for permissions at some point
        //command_check: Some(|ctx| {
        //    Box::pin(async move {
        //        if ctx.author().id == 123456789 {
        //            return Ok(false);
        //        }
        //        Ok(true)
        //    })
        //}),
        //skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!("Got an event in event handler: {:?}", event.name());
                Ok(())
            })
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(
            var("DISCORD_TOKEN")
                .expect("Missing `DISCORD_TOKEN` env var, see README for more information."),
        )
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    spotify: spotify_auth,
                })
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
        .unwrap();

    Ok(())
}

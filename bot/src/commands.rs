use crate::{
    Context,
};

use poise::{
    command,
    //builtins::autocomplete_command,   
};
use fishify_lib::spotify::Fishify;
use anyhow::Result;
use rspotify::{
    model::{
        enums::{
            types::SearchType,
            misc::RepeatState,
        },
    },
};

fn format_response(spotify: &Fishify) -> String {
    if spotify.show {
        spotify.response.iter().fold(String::new(), |r, s| format!("{r}> {s}\n"))
    } else {
        spotify.response.join("\n")
    }
}

// idk how to do this nicely with enums i don't own
#[derive(Debug, poise::ChoiceParameter)]
pub enum SearchTypeChoice {
    Track,
    Album,
    Playlist,
    Artist,
    Episode,
    Show,
}

impl From<SearchTypeChoice> for SearchType {
    fn from(_type: SearchTypeChoice) -> Self {
        match _type {
            SearchTypeChoice::Track => Self::Track,
            SearchTypeChoice::Album => Self::Album,
            SearchTypeChoice::Playlist => Self::Playlist,
            SearchTypeChoice::Artist => Self::Artist,
            SearchTypeChoice::Episode => Self::Episode,
            SearchTypeChoice::Show => Self::Show,
        }
    }
}

/// Register slash commands
#[derive(Debug, poise::ChoiceParameter)]
pub enum RepeatStateChoice {
    Off,
    Track,
    Context,
}

impl From<RepeatStateChoice> for RepeatState {
    fn from(state: RepeatStateChoice) -> Self {
        match state {
            RepeatStateChoice::Off => Self::Off,
            RepeatStateChoice::Track => Self::Track,
            RepeatStateChoice::Context => Self::Context,
        }
    }
}

#[command(slash_command, owners_only = true)]
pub async fn register(ctx: Context<'_>) -> Result<()> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Play music
#[command(slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Search query, or url"]
    query: Option<String>,
    #[description = "Search type"]
    #[rename = "type"]
    _type: Option<SearchTypeChoice>,
    #[description = "Whether to treat query as url"]
    is_url: Option<bool>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.play(query, _type.map(|x| x.into()), is_url.unwrap_or(false), false).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Queue music
#[command(slash_command)]
pub async fn queue(
    ctx: Context<'_>,
    #[description = "Search query, or url"]
    query: Option<String>,
    #[description = "Search type"]
    #[rename = "type"]
    _type: Option<SearchTypeChoice>,
    #[description = "Whether to treat query as url"]
    is_url: Option<bool>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.play(query, _type.map(|x| x.into()), is_url.unwrap_or(false), true).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Search for music
#[command(slash_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Search query"]
    query: String,
    #[description = "Search type"]
    #[rename = "type"]
    _type: Option<SearchTypeChoice>,
    #[description = "Limit number of results"]
    limit: Option<u32>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.search(query, _type.map(|x| x.into()), limit).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// List the current queue
#[command(slash_command)]
pub async fn queue_list(
    ctx: Context<'_>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.queue_list().await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Pause the music
#[command(slash_command)]
pub async fn pause(
    ctx: Context<'_>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.pause().await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Skip to next song
#[command(slash_command)]
pub async fn skip(
    ctx: Context<'_>,
    #[description = "Number of songs to skip"]
    count: Option<u8>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.skip(count.unwrap_or(1)).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Playback status
#[command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.status().await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// List available devices
#[command(slash_command)]
pub async fn device_list(
    ctx: Context<'_>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.device_list().await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// List available devices
#[command(slash_command)]
pub async fn device_connect(
    ctx: Context<'_>,
    #[description = "Name of device"]
    name: Option<String>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.device_connect(name).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Get device status
#[command(slash_command)]
pub async fn device_status(
    ctx: Context<'_>,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.device_status().await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Set the volume
#[command(slash_command)]
pub async fn set_volume(
    ctx: Context<'_>,
    #[description = "Volume level"]
    level: u8,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.set_volume(level).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Set shuffle
#[command(slash_command)]
pub async fn set_shuffle(
    ctx: Context<'_>,
    #[description = "Shuffle state"]
    state: bool,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.set_shuffle(state).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}

/// Set repeat
#[command(slash_command)]
pub async fn set_repeat(
    ctx: Context<'_>,
    #[description = "Repeat state"]
    state: RepeatStateChoice,
) -> Result<()> {
    let mut fishify = Fishify::from(&ctx.data().spotify);
    fishify.set_repeat(state.into()).await?;
    ctx.say(format_response(&fishify)).await?;

    Ok(())
}


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

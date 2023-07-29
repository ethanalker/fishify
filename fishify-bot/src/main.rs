use fishify_lib::{
    spotify_init,
    spotify::{ FishifyClient, },
};

use anyhow::Result;
use rspotify::{
    scopes,
};

#[tokio::main]
async fn main() -> Result<()> {
    let spotify = spotify_init(scopes!("user-modify-playback-state", "user-read-playback-state")).await?;

    todo!();

    Ok(())
}

use fishify_lib::{
    spotify_init,
    spotify::{ Fishify, },
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let spotify_auth = spotify_init().await?;

    todo!();

    Ok(())
}

mod cli;

use cli::{ Cli, Commands, QueueCommands, DeviceCommands, SetCommands, };

use fishify_lib::{
    spotify_init,
    spotify::{ FishifyClient, },
};

use anyhow::Result;
use rspotify::{
    scopes,
};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let spotify = spotify_init(scopes!("user-modify-playback-state", "user-read-playback-state")).await?;

    let cli = Cli::parse();

    let response = match cli.command {
        Commands::Play{query, url, _type} => spotify.play(query, _type, url, false).await?,
        Commands::Queue{query, url, _type, command} => {
            match command {
                Some(QueueCommands::List) => spotify.queue_list().await?,
                None => spotify.play(query, _type, url, true).await?,
            }
        },
        Commands::Pause => spotify.pause().await?,
        Commands::Skip{count} => spotify.skip(count).await?,
        Commands::Status => spotify.status().await?,
        Commands::Device{command} => {
            match command {
                DeviceCommands::Connect{name} => spotify.device_connect(name).await?,
                DeviceCommands::List => spotify.device_list().await?,
                DeviceCommands::Status => spotify.device_status().await?,
            }
        }
        Commands::Set{command} => {
            match command {
                SetCommands::Volume{level} => spotify.set_volume(level).await?,
                SetCommands::Shuffle{state} => spotify.set_shuffle(state).await?,
                SetCommands::Repeat{state} => spotify.set_repeat(state).await?,
            }
        }
    };

    let response_str = response.join("\n");
    println!("{response_str}");

    Ok(())
}

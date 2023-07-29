use clap::{ Subcommand, Parser, };

use rspotify::model::enums::{
    misc::RepeatState,
    types::SearchType,
};

fn volume_parser(s: &str) -> Result<u8, String> {
    if let Ok(val) = s.parse::<u8>() {
        if val > 100 {
            Err(format!("cannot exceed 100"))
        } else {
            Ok(val)
        }
    } else {
        Err(format!("must be a positive number below 100"))
    }
}

fn type_parser(s: &str) -> Result<SearchType, String> {
    match &*s.to_ascii_lowercase() {
        "track" => Ok(SearchType::Track),
        "album" => Ok(SearchType::Album),
        "playlist" => Ok(SearchType::Playlist),
        "artist" => Ok(SearchType::Artist),
        "episode" => Ok(SearchType::Episode),
        "show" => Ok(SearchType::Show),
        _ => Err(format!("must be 'track', 'album', 'playlist', 'artist', 'episode', or 'show'"))
    }
}

fn shuffle_parser(s: &str) -> Result<bool, String> {
    match &*s.to_ascii_lowercase() {
        "true" | "on" => Ok(true),
        "false" | "off" => Ok(false),
        _ => Err(format!("must be 'true' or 'false'"))
    }
}

fn repeat_parser(s: &str) -> Result<RepeatState, String> {
    match &*s.to_ascii_lowercase() {
        "true" | "on" | "context" => Ok(RepeatState::Context),
        "track" => Ok(RepeatState::Track),
        "false" | "off" => Ok(RepeatState::Off),
        _ => Err(format!("must be 'on', 'context', 'track', or 'off'"))
    }
}

#[derive(Debug, Parser)]
#[command(name = "fishify")]
#[command(about = "A spotify client CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Play music. Unpause if no arguments are supplied
    Play {
        /// Search query for music, or url if --url is supplied
        query: Option<String>,
        /// Treat query as a url
        #[arg(short, long)]
        url: bool,
        /// Type of music to be played, can be 'track', 'album', 'playlist', 'artist', 'episode', or 'show'
        #[arg(short, long, value_parser = type_parser)]
        _type: Option<SearchType>,
    },
    #[command(arg_required_else_help = true, args_conflicts_with_subcommands = true)]
    /// Queue music
    Queue {
        /// Search query for music, or url if --url is supplied
        query: Option<String>,
        /// If query should be treated as a url
        #[arg(short, long)]
        url: bool,
        /// Type of music to be played, can be 'track', 'album', 'playlist', 'artist', 'episode', or 'show'
        #[arg(short, long, value_parser = type_parser)]
        _type: Option<SearchType>,

        #[command(subcommand)]
        command: Option<QueueCommands>,
    },
    /// Pause music
    Pause,
    /// Get playback status
    Status,
    /// Skip tracks
    Skip {
        /// Number of songs to skip
        #[arg(default_value_t = 1)]
        count: u8,  
    },
    #[command(arg_required_else_help = true)]
    Device {
        #[command(subcommand)]
        command: DeviceCommands,
    },
    #[command(arg_required_else_help = true)]
    Set {
        #[command(subcommand)]
        command: SetCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueueCommands {
    /// List the current queue
    List,
}

#[derive(Debug, Subcommand)]
pub enum SetCommands {
    #[command(arg_required_else_help = true)]
    /// Set volume
    Volume {
        #[arg(value_parser = volume_parser)]
        /// 0-100
        level: u8,
    },
    #[command(arg_required_else_help = true)]
    /// Enable/disable shuffle
    Shuffle {
        #[arg(value_parser = shuffle_parser)]
        /// Boolean
        state: bool,
    },
    #[command(arg_required_else_help = true)]
    /// Set repeat mode
    Repeat {
        #[arg(value_parser = repeat_parser)]
        /// Repeat mode, can be 'on', 'context', 'track', or 'off'
        state: RepeatState,
    },
}

#[derive(Debug, Subcommand)]
pub enum DeviceCommands {
    /// Connect to a device
    Connect {
        name: Option<String>,
    },
    /// List devices to conntect
    List,
    /// Get current device status
    Status,
}

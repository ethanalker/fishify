pub mod config;
pub mod spotify;
pub mod model;

use config::ClientConfig;

use anyhow::Result;
use rspotify::{
    AuthCodeSpotify, Credentials, OAuth, Config,
    scopes,
    clients::{ OAuthClient, BaseClient, },
};

// init with sensible defaults, if you want more control do it manually
// AuthCodeSpotify must be used, OAuthClient is Sized and therefore cannot be a trait object, so i gotta use a real type
pub async fn spotify_init() -> Result<AuthCodeSpotify> {
    let mut client_config = ClientConfig::new();
    client_config.load_config()?;

    let config_paths = client_config.get_or_build_paths()?;

    let creds = Credentials::new(&client_config.client_id, &client_config.client_secret);
    let oauth = OAuth {
        redirect_uri: client_config.get_redirect_uri(),
        scopes: scopes!(
            "user-modify-playback-state", 
            "user-read-playback-state"
        ),
        ..Default::default()
    };
    let config = Config {
        token_refreshing: true,
        token_cached: true,
        cache_path: config_paths.token_cache_path,
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    spotify.refresh_token().await?;
    if spotify.get_token().lock().await.unwrap().is_none() {
        let url = spotify.get_authorize_url(false).unwrap();
        spotify.prompt_for_token(&url).await.unwrap();
    }

    return Ok(spotify);
}

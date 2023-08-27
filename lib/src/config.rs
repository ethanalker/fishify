use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

const DEFAULT_PORT: u16 = 8888;
const CONFIG_FILE: &str = "client.yml";
const CONFIG_DIR: &str = ".config";
const APP_CONFIG_DIR: &str = "fishify";
const TOKEN_CACHE_FILE: &str = ".spotify_token_cache.json";

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub device_id: Option<String>,
    pub port: Option<u16>,
}

pub struct ConfigPaths {
    pub config_file_path: PathBuf,
    pub token_cache_path: PathBuf,
}

impl ClientConfig {
    pub fn new() -> ClientConfig {
        ClientConfig {
            client_id: "".to_string(),
            client_secret: "".to_string(),
            device_id: None,
            port: None,
        }
    }

    pub fn get_redirect_uri(&self) -> String {
        format!("http://localhost:{}/callback", self.get_port())
    }

    pub fn get_port(&self) -> u16 {
        self.port.unwrap_or(DEFAULT_PORT)
    }

    pub fn get_or_build_paths(&self) -> Result<ConfigPaths> {
        match dirs::home_dir() {
            Some(home) => {
                let path = Path::new(&home);
                let home_config_dir = path.join(CONFIG_DIR);
                let app_config_dir = home_config_dir.join(APP_CONFIG_DIR);

                if !home_config_dir.exists() {
                    fs::create_dir(&home_config_dir)?;
                }

                if !app_config_dir.exists() {
                    fs::create_dir(&app_config_dir)?;
                }

                let config_file_path = &app_config_dir.join(CONFIG_FILE);
                let token_cache_path = &app_config_dir.join(TOKEN_CACHE_FILE);

                let paths = ConfigPaths {
                    config_file_path: config_file_path.to_path_buf(),
                    token_cache_path: token_cache_path.to_path_buf(),
                };

                Ok(paths)
            } 
            None => Err(anyhow!("No $HOME directory found for client config")),
        }
    }

    pub fn load_config(&mut self) -> Result<()> {
        let paths = self.get_or_build_paths()?;
        if paths.config_file_path.exists() {
            let config_string = fs::read_to_string(&paths.config_file_path)?;
            let config: ClientConfig = serde_yaml::from_str(&config_string)?;

            self.client_id = config.client_id;
            self.client_secret = config.client_secret;
            self.device_id = config.device_id;

            Ok(())
        } else {
            Err(anyhow!("Configure client_id and client_secret in {}", paths.config_file_path.display()))
        }
    }
}

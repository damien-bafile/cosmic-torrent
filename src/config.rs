use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub download_directory: PathBuf,
    pub upload_limit: Option<u64>, // KB/s, None for unlimited
    pub download_limit: Option<u64>, // KB/s, None for unlimited
    pub max_torrents: u32,
    pub max_peers_per_torrent: u32,
    pub listen_port: u16,
    pub enable_dht: bool,
    pub enable_upnp: bool,
    pub seed_ratio_limit: Option<f32>, // Stop seeding after this ratio
    pub seed_time_limit: Option<u64>, // Stop seeding after this many seconds
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            download_directory: dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
                .join("CosmicTorrent"),
            upload_limit: None,
            download_limit: None,
            max_torrents: 50,
            max_peers_per_torrent: 80,
            listen_port: 6881,
            enable_dht: true,
            enable_upnp: true,
            seed_ratio_limit: Some(2.0),
            seed_time_limit: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("cosmic-torrent");
        
        let config_path = config_dir.join("config.toml");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("cosmic-torrent");
        
        std::fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
}

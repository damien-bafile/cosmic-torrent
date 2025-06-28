use lava_torrent::torrent::v1::Torrent;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;
use url::Url;

/// Metadata and file information for a torrent.
#[derive(Debug, Clone)]
pub struct TorrentInfo {
    /// The display name of the torrent.
    pub name: String,
    /// Total size of the torrent in bytes.
    pub size: u64,
    /// The info hash (hex-encoded) of the torrent.
    pub info_hash: String,
    /// List of announce (tracker) URLs.
    pub announce_urls: Vec<String>,
    /// List of files contained in the torrent.
    pub files: Vec<FileInfo>,
}

/// Information about a single file in a torrent.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The file path relative to the torrent root.
    pub path: PathBuf,
    /// The file size in bytes.
    pub size: u64,
}

/// Download/upload statistics and progress for a torrent.
#[derive(Debug, Clone)]
pub struct TorrentStats {
    /// Total bytes downloaded.
    pub downloaded: u64,
    /// Total bytes uploaded.
    pub uploaded: u64,
    /// Current download rate in bytes per second.
    pub download_rate: u64,
    /// Current upload rate in bytes per second.
    pub upload_rate: u64,
    /// Download progress (0.0 to 1.0).
    pub progress: f32,
    /// Number of connected peers.
    pub peers: u32,
    /// Number of connected seeds.
    pub seeds: u32,
}

/// Events emitted by the torrent engine to notify about state changes.
#[derive(Debug, Clone)]
pub enum TorrentEvent {
    /// A torrent was added (info_hash, info).
    Added(String, TorrentInfo),
    /// Progress update for a torrent (info_hash, stats).
    Progress(String, TorrentStats),
    /// Torrent download completed (info_hash).
    Completed(String),
    /// An error occurred (info_hash, error message).
    Error(String, String),
    /// Torrent was paused (info_hash).
    Paused(String),
    /// Torrent was resumed (info_hash).
    Resumed(String),
}

/// Main engine for managing torrents and their state.
pub struct TorrentEngine {
    /// Channel for sending events to the UI or other consumers.
    event_sender: mpsc::UnboundedSender<TorrentEvent>,
    /// Map of info_hash to torrent handles.
    torrents: HashMap<String, TorrentHandle>,
    /// Default download path for torrent data.
    download_path: PathBuf,
}

/// Internal handle for a managed torrent.
struct TorrentHandle {
    /// Torrent metadata and file info.
    info: TorrentInfo,
    /// Current statistics for the torrent.
    stats: TorrentStats,
    /// Whether the torrent is paused.
    paused: bool,
}

impl TorrentEngine {
    /// Create a new torrent engine and event receiver.
    pub fn new() -> (Self, mpsc::UnboundedReceiver<TorrentEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        // Set default download path
        let download_path = dirs::download_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp")));

        (
            Self {
                event_sender: tx,
                torrents: HashMap::new(),
                download_path,
            },
            rx,
        )
    }

    /// Add a torrent from a magnet URL.
    ///
    /// Returns the info hash on success.
    pub async fn add_magnet(&mut self, magnet_url: &str) -> Result<String, String> {
        // Parse magnet URL
        let url = Url::parse(magnet_url).map_err(|e| format!("Invalid magnet URL: {}", e))?;

        if url.scheme() != "magnet" {
            return Err("Not a magnet URL".to_string());
        }

        // Extract info hash from magnet URL
        let info_hash = self.extract_info_hash(&url)?;

        // Create mock torrent info for demo
        let torrent_info: TorrentInfo = TorrentInfo {
            name: format!("Torrent {}", info_hash),
            size: 1024 * 1024 * 100, // 100MB
            info_hash: info_hash.clone(),
            announce_urls: vec!["http://tracker.example.com:8080/announce".to_string()],
            files: vec![FileInfo {
                path: PathBuf::from("example_file.txt"),
                size: 1024 * 1024 * 100,
            }],
        };

        let stats: TorrentStats = TorrentStats {
            downloaded: 0,
            uploaded: 0,
            download_rate: 0,
            upload_rate: 0,
            progress: 0.0,
            peers: 0,
            seeds: 0,
        };

        let handle = TorrentHandle {
            info: torrent_info.clone(),
            stats,
            paused: false,
        };

        self.torrents.insert(info_hash.clone(), handle);

        // Send event
        let _ = self
            .event_sender
            .send(TorrentEvent::Added(info_hash.clone(), torrent_info));

        Ok(info_hash)
    }

    /// Add a torrent from a .torrent file.
    ///
    /// Returns the info hash on success.
    pub async fn add_torrent_file(&mut self, file_path: &str) -> Result<String, String> {
        // Parse .torrent file using lava_torrent
        let torrent_data =
            fs::read(file_path).map_err(|e| format!("Failed to read torrent file: {}", e))?;

        let torrent = Torrent::read_from_bytes(&torrent_data)
            .map_err(|e| format!("Failed to parse torrent file: {}", e))?;

        // Calculate info hash
        let info_hash = hex::encode(torrent.info_hash());

        // Extract file information
        let files = if let Some(files) = torrent.files.as_ref() {
            files
                .iter()
                .map(|file| FileInfo {
                    path: PathBuf::from(&file.path.join("/")),
                    size: file.length as u64,
                })
                .collect()
        } else {
            // Single file torrent
            vec![FileInfo {
                path: PathBuf::from(&torrent.name),
                size: torrent.length as u64,
            }]
        };

        let total_size = files.iter().map(|f| f.size).sum();

        let torrent_info = TorrentInfo {
            name: torrent.name.clone(),
            size: total_size,
            info_hash: info_hash.clone(),
            announce_urls: {
                let mut urls = vec![];
                if let Some(announce) = &torrent.announce {
                    urls.push(announce.clone());
                }
                if let Some(announce_list) = &torrent.announce_list {
                    for tier in announce_list {
                        for url in tier {
                            urls.push(url.clone());
                        }
                    }
                }
                urls
            },
            files,
        };

        let stats = TorrentStats {
            downloaded: 0,
            uploaded: 0,
            download_rate: 0,
            upload_rate: 0,
            progress: 0.0,
            peers: 0,
            seeds: 0,
        };

        let handle = TorrentHandle {
            info: torrent_info.clone(),
            stats,
            paused: false,
        };

        self.torrents.insert(info_hash.clone(), handle);

        // Send event
        let _ = self
            .event_sender
            .send(TorrentEvent::Added(info_hash.clone(), torrent_info));

        Ok(info_hash)
    }

    /// Pause a torrent by its info hash.
    pub fn pause_torrent(&mut self, info_hash: &str) -> Result<(), String> {
        if let Some(handle) = self.torrents.get_mut(info_hash) {
            handle.paused = true;
            let _ = self
                .event_sender
                .send(TorrentEvent::Paused(info_hash.to_string()));
            Ok(())
        } else {
            Err("Torrent not found".to_string())
        }
    }

    /// Resume a paused torrent by its info hash.
    pub fn resume_torrent(&mut self, info_hash: &str) -> Result<(), String> {
        if let Some(handle) = self.torrents.get_mut(info_hash) {
            handle.paused = false;
            let _ = self
                .event_sender
                .send(TorrentEvent::Resumed(info_hash.to_string()));
            Ok(())
        } else {
            Err("Torrent not found".to_string())
        }
    }

    /// Remove a torrent by its info hash.
    pub fn remove_torrent(&mut self, info_hash: &str) -> Result<(), String> {
        if self.torrents.remove(info_hash).is_some() {
            Ok(())
        } else {
            Err("Torrent not found".to_string())
        }
    }

    /// Start the periodic update loop for torrent statistics and progress.
    pub async fn start_update_loop(&mut self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            // Update stats for all active torrents
            for (info_hash, handle) in &mut self.torrents {
                if !handle.paused && handle.stats.progress < 1.0 {
                    // Simulate progress
                    handle.stats.progress = (handle.stats.progress + 0.01).min(1.0);
                    // Simulate some activity with simple incrementing values
                    handle.stats.download_rate =
                        (handle.stats.download_rate + 1024) % (1024 * 1024);
                    handle.stats.upload_rate = (handle.stats.upload_rate + 512) % (1024 * 512);
                    handle.stats.peers = (handle.stats.peers % 50) + 1;
                    handle.stats.seeds = (handle.stats.seeds % 20) + 1;
                    handle.stats.downloaded =
                        (handle.info.size as f32 * handle.stats.progress) as u64;

                    let _ = self.event_sender.send(TorrentEvent::Progress(
                        info_hash.clone(),
                        handle.stats.clone(),
                    ));

                    if handle.stats.progress >= 1.0 {
                        let _ = self
                            .event_sender
                            .send(TorrentEvent::Completed(info_hash.clone()));
                    }
                }
            }
        }
    }

    /// Extract the info hash from a magnet URL.
    fn extract_info_hash(&self, url: &Url) -> Result<String, String> {
        // Extract info hash from magnet URL
        for (key, value) in url.query_pairs() {
            if key == "xt" && value.starts_with("urn:btih:") {
                return Ok(value.strip_prefix("urn:btih:").unwrap().to_string());
            }
        }
        Err("No info hash found in magnet URL".to_string())
    }
}

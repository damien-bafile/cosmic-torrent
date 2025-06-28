mod torrent_engine;

use cosmic::prelude::*;
use cosmic::widget::container;
use std::collections::HashMap;

fn main() -> cosmic::iced::Result {
    env_logger::init();
    cosmic::action::app("Cosmic Torrent")
        .title("Cosmic Torrent")
        .size(800, 600)
        .run(
            CosmicTorrent::new,
            CosmicTorrent::update,
            CosmicTorrent::view,
        )
}

/// Represents a torrent with its metadata and status.
#[derive(Clone, Debug)]
struct Torrent {
    /// The name of the torrent.
    name: String,
    /// The size of the torrent in bytes.
    size: u64,
    /// The download progress (0.0 to 1.0).
    progress: f32,
    /// The current status of the torrent.
    status: TorrentStatus,
    /// The current download speed in bytes per second.
    download_speed: u64,
    /// The current upload speed in bytes per second.
    upload_speed: u64,
}

/// The possible statuses for a torrent.
#[derive(Clone, Debug)]
enum TorrentStatus {
    /// The torrent is currently downloading.
    Downloading,
    /// The torrent is seeding.
    Seeding,
    /// The torrent is paused.
    Paused,
    /// The torrent encountered an error.
    Error(String),
    /// The torrent download is completed.
    Completed,
}

/// The main application state for Cosmic Torrent.
struct CosmicTorrent {
    /// A map of torrent identifiers to their corresponding Torrent structs.
    torrents: std::collections::HashMap<String, Torrent>,
    /// The current input value for adding a new torrent.
    add_torrent_input: String,
}

/// Messages that represent user actions or events in the application.
#[derive(Clone, Debug)]
enum Message {
    /// Triggered to add a new torrent.
    AddTorrent,
    /// Triggered when the add torrent input changes.
    AddTorrentInputChanged(String),
    /// Triggered to pause a torrent with the given ID.
    PauseTorrent(String),
    /// Triggered to resume a torrent with the given ID.
    ResumeTorrent(String),
    /// Triggered to remove a torrent with the given ID.
    RemoveTorrent(String),
    /// Triggered on a periodic timer tick to update torrent states.
    Tick,
}

impl CosmicTorrent {
    /// Creates a new instance of the application state and an initial task.
    fn new() -> (Self, Task<Message>) {
        (
            CosmicTorrent {
                torrents: HashMap::new(),
                add_torrent_input: String::new(),
            },
            Task::none(),
        )
    }

    /// Handles updates to the application state in response to messages.
    fn update(&mut self, message: Message) {
        match message {
            Message::AddTorrent => {
                if !self.add_torrent_input.is_empty() {
                    // Here you would parse the magnet link or .torrent file
                    let torrent = Torrent {
                        name: format!("Torrent {}", self.torrents.len() + 1),
                        size: 1024 * 1024 * 100, // 100MB example
                        progress: 0.0,
                        status: TorrentStatus::Downloading,
                        download_speed: 0,
                        upload_speed: 0,
                    };
                    self.torrents
                        .insert(self.add_torrent_input.clone(), torrent);
                    self.add_torrent_input.clear();
                }
            }
            Message::AddTorrentInputChanged(input) => {
                self.add_torrent_input = input;
            }
            Message::PauseTorrent(id) => {
                if let Some(torrent) = self.torrents.get_mut(&id) {
                    torrent.status = TorrentStatus::Paused;
                }
            }
            Message::ResumeTorrent(id) => {
                if let Some(torrent) = self.torrents.get_mut(&id) {
                    torrent.status = TorrentStatus::Downloading;
                }
            }
            Message::RemoveTorrent(id) => {
                self.torrents.remove(&id);
            }
            Message::Tick => {
                // Update torrent progress and speeds
                for torrent in self.torrents.values_mut() {
                    if matches!(torrent.status, TorrentStatus::Downloading) {
                        torrent.progress = (torrent.progress + 0.01).min(1.0);
                        torrent.download_speed = (torrent.download_speed + 1024) % (1024 * 100);
                        if torrent.progress >= 1.0 {
                            torrent.status = TorrentStatus::Completed;
                        }
                    }
                }
            }
        }
    }

    /// Returns the UI representation of the application state.
    fn view(&self) -> Element<Message> {
        let add_section = row![
            text_input(
                "Enter magnet link or torrent file path",
                &self.add_torrent_input
            )
            .on_input(Message::AddTorrentInputChanged)
            .width(Length::Fill),
            button("Add Torrent").on_press(Message::AddTorrent)
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let mut torrent_list = column![].spacing(10);

        for (id, torrent) in &self.torrents {
            let status_text = match &torrent.status {
                TorrentStatus::Downloading => {
                    format!("Downloading ({:.1}%)", torrent.progress * 100.0)
                }
                TorrentStatus::Seeding => "Seeding".to_string(),
                TorrentStatus::Paused => "Paused".to_string(),
                TorrentStatus::Error(err) => format!("Error: {}", err),
                TorrentStatus::Completed => "Completed".to_string(),
            };

            let torrent_row = row![
                column![
                    text(&torrent.name).size(16),
                    text(format!(
                        "Size: {:.1} MB",
                        torrent.size as f64 / (1024.0 * 1024.0)
                    ))
                    .size(12),
                    text(status_text).size(12),
                    text(format!(
                        "↓ {:.1} KB/s ↑ {:.1} KB/s",
                        torrent.download_speed as f64 / 1024.0,
                        torrent.upload_speed as f64 / 1024.0
                    ))
                    .size(10)
                ]
                .width(Length::Fill),
                row![
                    button("Pause").on_press(Message::PauseTorrent(id.clone())),
                    button("Resume").on_press(Message::ResumeTorrent(id.clone())),
                    button("Remove").on_press(Message::RemoveTorrent(id.clone()))
                ]
                .spacing(5)
            ]
            .spacing(10)
            .align_y(Alignment::Center);

            torrent_list = torrent_list.push(torrent_row);
        }

        let content = column![add_section, torrent_list].spacing(20).padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

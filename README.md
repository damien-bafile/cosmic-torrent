# Cosmic Torrent

A modern BitTorrent client built with libcosmic and Rust, designed for the COSMIC desktop environment.

## Features

- Modern, native UI using libcosmic
- Support for magnet links and .torrent files
- Real-time download/upload statistics
- Torrent management (pause, resume, remove)
- Multi-torrent support
- Built with Rust for safety and performance

## Current Status

This is a demo/prototype implementation. The torrent engine currently simulates BitTorrent functionality for UI development purposes. To make this a fully functional torrent client, you'll need to integrate a real BitTorrent library.

## Prerequisites

- Rust (latest stable)
- libcosmic development dependencies
- Linux (COSMIC desktop environment recommended)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Architecture

The application is structured with:

- `main.rs` - COSMIC application UI and main loop
- `torrent_engine.rs` - BitTorrent protocol handling (currently mock implementation)

## Next Steps for Full Implementation

1. **Choose a BitTorrent Library**: Consider libraries like:
   - `cratetorrent` - Pure Rust BitTorrent implementation
   - `torrent-rs` - Another Rust torrent library
   - Or implement custom BitTorrent protocol handling

2. **File Management**: 
   - Add download directory selection
   - File priority management
   - Partial file handling

3. **Advanced Features**:
   - DHT support
   - Peer exchange
   - Bandwidth limiting
   - Scheduling
   - RSS feed support

4. **Configuration**:
   - Settings dialog
   - Port configuration
   - Tracker management

5. **Persistence**:
   - Save torrent state
   - Resume incomplete downloads
   - Session management

## Dependencies

Key dependencies include:
- `libcosmic` - UI framework
- `tokio` - Async runtime  
- `serde` - Serialization
- `url` - URL parsing for magnet links

## Contributing

This is a starting template. Contributions welcome for:
- Real BitTorrent protocol implementation
- UI improvements
- Additional features
- Bug fixes

## License

GPL-3.0-or-later

# crab-dlna

[![CICD](https://github.com/gabrielmagno/crab-dlna/actions/workflows/CICD.yml/badge.svg)](https://github.com/gabrielmagno/crab-dlna/actions/workflows/CICD.yml)
[![Version info](https://img.shields.io/crates/v/crab-dlna.svg)](https://crates.io/crates/crab-dlna)

crab-dlna is a minimal UPnP/DLNA media streamer, available both as a standlone CLI (command line interface) application and a Rust library.

It allows you to play a local video file in your TV (or any other DLNA compatible device).

## Features

- Searching available DLNA devices in the local network
- Streaming audio
- Streaming video, with subtitle support
- **Terminal User Interface (TUI)** with comprehensive media control
- **Interactive keyboard control** (space to pause/resume, q to quit)
- **Playlist support** for playing multiple files or entire directories
- **Pause/Resume control** via DLNA commands
- **Directory scanning** for automatic playlist creation
- **Real-time playback status** and progress display
- **Device information** and transport state monitoring

## Installation

In the GitHub Releases of this repository we provide [archives of precompiled binaries](https://github.com/gabrielmagno/crab-dlna/releases) of crab-dlna, available for **Linux**, **Windows**, and **macOS**.

### cargo

Installation via cargo is done by installing the `crab-dlna` crate:

```bash
# If required, update Rust on the stable channel
rustup update stable

cargo install crab-dlna

# Alternatively, --locked may be required due to how cargo install works
cargo install crab-dlna --locked
```

## Usage (CLI)

You can list all the CLI commands by running:

```
crab-dlna --help
```

### List

Scan compatible devices and list the available ones:

```bash
crab-dlna list
```

If your device is not being listed, you might need to increase the search timeout:

```bash
crab-dlna -t 20 list
```

### Play

Play a video, automatically loading the subtitles if available, selecting a random device:

```bash
crab-dlna play That.Movie.mkv
```

Play a video with interactive keyboard control (space to pause/resume, q to quit):

```bash
crab-dlna play That.Movie.mkv --interactive
```

Play all media files in a directory with playlist mode:

```bash
crab-dlna play ./Movies --playlist --interactive
```

Play a video, specifying the device through query (scan devices before playing):

```bash
crab-dlna play That.Movie.mkv -q "osmc" --interactive
```

Play a video, specifying the device through its exact location (no scan, faster):

```bash
crab-dlna play That.Movie.mkv -d "http://192.168.1.13:1082/" --interactive
```

Play with subtitle synchronization and interactive control:

```bash
crab-dlna play That.Movie.mkv --subtitle-sync --interactive
```

### TUI Mode

Launch the Terminal User Interface for comprehensive media control:

```bash
crab-dlna play That.Movie.mkv --tui
```

Launch TUI with playlist mode for an entire directory:

```bash
crab-dlna play ./Movies --playlist --tui
```

The TUI provides:

- **Real-time playback status** with transport state and position
- **Interactive playlist navigation** with visual indicators
- **Progress bar** showing current playback position
- **Device information** display
- **Keyboard shortcuts** help system
- **Error and status messages** in real-time

#### TUI Keyboard Controls

- `SPACE` / `P` - Toggle play/pause
- `S` - Stop playback
- `↑` / `K` - Navigate up in playlist
- `↓` / `J` - Navigate down in playlist
- `ENTER` - Play selected item
- `R` - Refresh status
- `H` / `F1` - Show help dialog
- `D` - Show device information
- `Q` / `ESC` - Quit application

## Usage (library)

Add `crab-dlna` and `tokio` to your dependencies:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
crab-dlna = "0.2"
```

### Example: discover and list devices

crab-dlna provides a function to discover a list devices in the network.

```rust
use crab_dlna::Render;

#[tokio::main]
async fn main() {
    let discover_timeout_secs = 5;
    let renders_discovered = Render::discover(discover_timeout_secs).await.unwrap();
    for render in renders_discovered {
        println!("{}", render);
    }
}
```

### Example: play a video in a device

We can specify a DLNA device render trough a query string,
and then play a certain video in it, automatically detecting
the subtitle file.

```rust
use std::path::PathBuf;
use crab_dlna::{
    Render,
    RenderSpec,
    MediaStreamingServer,
    STREAMING_PORT_DEFAULT,
    get_local_ip,
    infer_subtitle_from_video,
    Error,
    play,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let discover_timeout_secs = 5;
    let render_spec = RenderSpec::Query(discover_timeout_secs, "Kodi".to_string());
    let render = Render::new(render_spec).await?;
    let host_ip = get_local_ip().await?;
    let host_port = STREAMING_PORT_DEFAULT;
    let video_path = PathBuf::from("/home/crab/Videos/my_video.mp4");
    let inferred_subtitle_path = infer_subtitle_from_video(&video_path);
    let media_streaming_server = MediaStreamingServer::new(
        &video_path,
        &inferred_subtitle_path,
        &host_ip,
        &host_port,
    )?;
    let config = Config::default();
    play(render, media_streaming_server, None, &config).await
}
```

### Example: interactive control and playlist

```rust
use std::path::PathBuf;
use crab_dlna::{
    Render, RenderSpec, Playlist, start_interactive_control,
    toggle_play_pause, Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let discover_timeout_secs = 5;
    let render_spec = RenderSpec::First(discover_timeout_secs);
    let render = Render::new(render_spec).await?;

    // Create playlist from directory
    let mut playlist = Playlist::from_directory("./media")?;
    playlist.set_loop(true);

    println!("Playlist created with {} files", playlist.len());

    // Start interactive control in background
    let render_clone = render.clone();
    tokio::spawn(async move {
        start_interactive_control(render_clone).await
    });

    // Toggle play/pause programmatically
    toggle_play_pause(&render).await?;

    Ok(())
}
```

### Example: TUI application

```rust
use crab_dlna::{Render, RenderSpec, Playlist, start_tui, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Discover DLNA device
    let render_spec = RenderSpec::First(5);
    let render = Render::new(render_spec).await?;

    // Create playlist from directory
    let playlist = Playlist::from_directory("./media")?;

    // Start TUI - provides full interactive interface
    start_tui(render, playlist).await?;

    Ok(())
}
```

You can access the full [documentation](https://docs.rs/crab-dlna/) to see more details about the library.

## License

Copyright (c) 2022 Gabriel Magno.

`crab-dlna` is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files for license details.

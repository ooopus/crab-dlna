use crab_dlna::{
    Error, Playlist, Render, RenderSpec, start_tui,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logger
    simple_logger::SimpleLogger::new().init().unwrap();

    println!("=== crab-dlna TUI Demo ===\n");

    // Configuration
    let discover_timeout_secs = 5;
    
    // Discover render device
    println!("1. Discovering DLNA devices...");
    let render_spec = RenderSpec::First(discover_timeout_secs);
    let render = Render::new(render_spec).await?;
    println!("   Found device: {}\n", render);

    // Create a demo playlist
    println!("2. Creating demo playlist...");
    let mut playlist = Playlist::new();
    
    // Try to create playlist from a directory first
    let media_dir = PathBuf::from("./media");
    if media_dir.exists() && media_dir.is_dir() {
        match Playlist::from_directory(&media_dir) {
            Ok(dir_playlist) => {
                playlist = dir_playlist;
                println!("   Created playlist from directory: {}", media_dir.display());
            }
            Err(_) => {
                println!("   No media files found in directory, creating demo playlist...");
                add_demo_files(&mut playlist);
            }
        }
    } else {
        println!("   Media directory not found, creating demo playlist...");
        add_demo_files(&mut playlist);
    }
    
    playlist.set_loop(true);
    println!("   Playlist created with {} files", playlist.len());
    
    // Display playlist contents
    println!("   Files in playlist:");
    for (i, file) in playlist.files().iter().enumerate() {
        println!("     {}: {}", i + 1, file.display());
    }
    
    println!("\n3. Starting TUI...");
    println!("   Use the following controls in the TUI:");
    println!("   - SPACE/P: Toggle play/pause");
    println!("   - S: Stop playback");
    println!("   - ↑/↓: Navigate playlist");
    println!("   - ENTER: Play selected item");
    println!("   - R: Refresh status");
    println!("   - H/F1: Show help");
    println!("   - D: Show device info");
    println!("   - Q/ESC: Quit");
    println!("\n   Press any key to start the TUI...");
    
    // Wait for user input
    let _ = std::io::stdin().read_line(&mut String::new());
    
    // Start the TUI
    start_tui(render, playlist).await?;
    
    println!("TUI demo completed!");
    Ok(())
}

fn add_demo_files(playlist: &mut Playlist) {
    // Add some demo files (these don't need to exist for the TUI demo)
    let demo_files = vec![
        "demo_video1.mp4",
        "demo_video2.avi", 
        "demo_audio1.mp3",
        "demo_audio2.flac",
        "demo_video3.mkv",
    ];
    
    for file in demo_files {
        playlist.add_file(PathBuf::from(file));
    }
}

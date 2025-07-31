use crab_dlna::{
    Config, Error, MediaStreamingServer, Playlist, Render, RenderSpec, STREAMING_PORT_DEFAULT,
    get_local_ip, infer_subtitle_from_video, play,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logger
    simple_logger::SimpleLogger::new().init().unwrap();

    // Configuration
    let discover_timeout_secs = 5;

    // Create a playlist from a directory (replace with actual directory)
    let media_dir = PathBuf::from("./media"); // Replace with actual media directory

    println!("Creating playlist from directory: {}", media_dir.display());
    let mut playlist = match Playlist::from_directory(&media_dir) {
        Ok(playlist) => playlist,
        Err(_) => {
            // Fallback: create playlist from individual files
            println!("Directory not found, creating playlist from individual files...");
            let mut playlist = Playlist::new();

            // Add some example files (replace with actual files)
            let example_files = vec!["video1.mp4", "video2.avi", "audio1.mp3"];

            for file in example_files {
                let path = PathBuf::from(file);
                if path.exists() {
                    playlist.add_file(path);
                    println!("Added to playlist: {}", file);
                }
            }

            if playlist.is_empty() {
                eprintln!("No media files found. Please add some media files to test.");
                return Ok(());
            }

            playlist
        }
    };

    // Set playlist to loop
    playlist.set_loop(true);

    println!("Playlist created with {} files", playlist.len());
    for (i, file) in playlist.files().iter().enumerate() {
        println!("  {}: {}", i + 1, file.display());
    }

    // Discover render device
    println!("Discovering DLNA devices...");
    let render_spec = RenderSpec::First(discover_timeout_secs);
    let render = Render::new(render_spec).await?;
    println!("Found device: {}", render);

    // Setup streaming configuration
    let host_ip = get_local_ip().await?;
    let host_port = STREAMING_PORT_DEFAULT;
    let config = Config::default();

    // Play each file in the playlist
    let total_files = playlist.len();
    let mut file_count = 0;
    while let Some(current_file) = playlist.next() {
        file_count += 1;
        println!("\n=== Playing file {} of {} ===", file_count, total_files);
        println!("Now playing: {}", current_file.display());

        // Create streaming server for current file
        let inferred_subtitle_path = infer_subtitle_from_video(current_file);
        let media_streaming_server =
            MediaStreamingServer::new(current_file, &inferred_subtitle_path, &host_ip, &host_port)?;

        // Play the file
        match play(render.clone(), media_streaming_server, None, &config).await {
            Ok(_) => {
                println!("Successfully played: {}", current_file.display());
            }
            Err(e) => {
                eprintln!("Failed to play {}: {}", current_file.display(), e);
                // Continue with next file
            }
        }

        // Stop after playing 3 files for demo purposes
        if file_count >= 3 {
            println!("Demo completed after playing {} files", file_count);
            break;
        }
    }

    println!("Playlist demo finished!");
    Ok(())
}

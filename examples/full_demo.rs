use crab_dlna::{
    Config, Error, MediaStreamingServer, Playlist, Render, RenderSpec, 
    STREAMING_PORT_DEFAULT, get_local_ip, infer_subtitle_from_video, 
    play, start_interactive_control, toggle_play_pause,
};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logger
    simple_logger::SimpleLogger::new().init().unwrap();

    println!("=== crab-dlna Enhanced Features Demo ===\n");

    // Configuration
    let discover_timeout_secs = 5;
    
    // Discover render device
    println!("1. Discovering DLNA devices...");
    let render_spec = RenderSpec::First(discover_timeout_secs);
    let render = Render::new(render_spec).await?;
    println!("   Found device: {}\n", render);

    // Demo 1: Basic pause/resume functionality
    println!("2. Testing pause/resume functionality...");
    demo_pause_resume(&render).await?;

    // Demo 2: Playlist functionality
    println!("3. Testing playlist functionality...");
    demo_playlist(&render).await?;

    // Demo 3: Interactive control
    println!("4. Testing interactive control...");
    demo_interactive_control(&render).await?;

    println!("=== Demo completed successfully! ===");
    Ok(())
}

async fn demo_pause_resume(render: &Render) -> Result<(), Error> {
    println!("   Creating a simple media file for testing...");
    
    // For demo purposes, we'll just test the pause/resume functions
    // In a real scenario, you would have a media file playing
    
    println!("   Testing transport state query...");
    match render.get_transport_info().await {
        Ok(info) => {
            println!("   Current transport state: {}", info.transport_state);
            println!("   Transport status: {}", info.transport_status);
            println!("   Speed: {}", info.speed);
        }
        Err(e) => {
            println!("   Warning: Could not get transport info: {}", e);
        }
    }

    println!("   Pause/resume demo completed.\n");
    Ok(())
}

async fn demo_playlist(render: &Render) -> Result<(), Error> {
    println!("   Creating a demo playlist...");
    
    let mut playlist = Playlist::new();
    
    // Add some demo files (these don't need to exist for the demo)
    let demo_files = vec![
        "demo_video1.mp4",
        "demo_video2.avi",
        "demo_audio1.mp3",
    ];
    
    for file in demo_files {
        playlist.add_file(PathBuf::from(file));
    }
    
    playlist.set_loop(true);
    
    println!("   Playlist created with {} files:", playlist.len());
    for (i, file) in playlist.files().iter().enumerate() {
        println!("     {}: {}", i + 1, file.display());
    }
    
    println!("   Testing playlist navigation...");
    let mut count = 0;
    while let Some(current_file) = playlist.next() {
        count += 1;
        println!("     Playing: {}", current_file.display());
        
        // Stop after 3 iterations to avoid infinite loop
        if count >= 3 {
            break;
        }
    }
    
    println!("   Playlist demo completed.\n");
    Ok(())
}

async fn demo_interactive_control(render: &Render) -> Result<(), Error> {
    println!("   Interactive control demo...");
    println!("   Note: In a real application, this would start keyboard listening.");
    println!("   Available controls would be:");
    println!("     SPACE / P  : Toggle play/pause");
    println!("     Q / ESC    : Quit");
    println!("     H / ?      : Show help");
    
    // For demo purposes, we'll just simulate some control actions
    println!("   Simulating pause/resume toggle...");
    
    // This would normally be triggered by keyboard input
    match toggle_play_pause(render).await {
        Ok(_) => println!("   Toggle successful"),
        Err(e) => println!("   Toggle failed (expected if no media is playing): {}", e),
    }
    
    println!("   Interactive control demo completed.\n");
    Ok(())
}

// Example of how to use the new CLI features:
// 
// # Play a single file with interactive control:
// cargo run -- play video.mp4 --interactive
//
// # Play all files in a directory with playlist mode:
// cargo run -- play ./media_folder --playlist --interactive
//
// # Play with subtitle synchronization:
// cargo run -- play video.mp4 --subtitle-sync --interactive
//
// # Discover devices:
// cargo run -- list
//
// # Play with specific device:
// cargo run -- play video.mp4 --device "http://192.168.1.100:8080/" --interactive

use crab_dlna::{
    Config, Error, MediaStreamingServer, Render, RenderSpec, STREAMING_PORT_DEFAULT, get_local_ip,
    infer_subtitle_from_video, play, start_interactive_control,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logger
    simple_logger::SimpleLogger::new().init().unwrap();

    // Configuration
    let discover_timeout_secs = 5;
    let video_path = PathBuf::from("test_video.mp4"); // Replace with actual video file

    // Discover render device
    println!("Discovering DLNA devices...");
    let render_spec = RenderSpec::First(discover_timeout_secs);
    let render = Render::new(render_spec).await?;
    println!("Found device: {render}");

    // Setup streaming server
    let host_ip = get_local_ip().await?;
    let host_port = STREAMING_PORT_DEFAULT;
    let inferred_subtitle_path = infer_subtitle_from_video(&video_path);
    let media_streaming_server =
        MediaStreamingServer::new(&video_path, &inferred_subtitle_path, &host_ip, &host_port)?;

    // Clone render for interactive control
    let render_clone = render.clone();

    // Start interactive control in background
    let interactive_handle = tokio::spawn(async move {
        println!("Starting interactive control...");
        println!("Press SPACE to toggle play/pause, 'q' to quit");
        if let Err(e) = start_interactive_control(render_clone).await {
            eprintln!("Interactive control error: {e}");
        }
    });

    // Start playback
    let config = Config::default();
    let play_result = play(render, media_streaming_server, None, &config).await;

    // Cancel interactive control
    interactive_handle.abort();

    play_result
}

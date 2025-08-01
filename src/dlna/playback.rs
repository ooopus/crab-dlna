//! DLNA playback management for crab-dlna
//!
//! This module handles the main playback functionality including
//! media streaming, subtitle synchronization, and transport control.

use crate::{
    config::{
        Config, DLNA_ACTION_SET_AV_TRANSPORT_URI, LOG_MSG_PLAYING_VIDEO, LOG_MSG_SETTING_VIDEO_URI,
        MEDIA_PLAYBACK_FAILED_MSG,
    },
    devices::Render,
    error::{Error, Result},
    media::{MediaStreamingServer, SubtitleSyncer},
    utils::retry_with_backoff,
};
use log::{debug, info};
use std::time::Duration;
use tokio::time::interval;

use super::metadata::{build_metadata, build_setavtransporturi_payload};

/// Builds a DLNA play payload with configurable parameters
fn build_play_payload(instance_id: u32, speed: u32) -> String {
    format!(
        r#"
    <InstanceID>{instance_id}</InstanceID>
    <Speed>{speed}</Speed>
"#
    )
}

/// Plays a media file in a DLNA compatible device render, according to the render and media streaming server provided
pub async fn play(
    render: Render,
    streaming_server: MediaStreamingServer,
    subtitle_syncer: Option<SubtitleSyncer>,
    config: &Config,
) -> Result<()> {
    let metadata = build_metadata(&streaming_server)?;
    debug!("Metadata: '{metadata}'");

    let setavtransporturi_payload = build_setavtransporturi_payload(&streaming_server, &metadata)?;
    debug!("SetAVTransportURI payload: '{setavtransporturi_payload}'");

    // Get the video URI before moving streaming_server
    let video_uri = streaming_server.video_uri();

    info!("Starting media streaming server...");
    let streaming_server_handle = tokio::spawn(async move { streaming_server.run().await });

    info!("{LOG_MSG_SETTING_VIDEO_URI}");
    retry_with_backoff(
        || async {
            render
                .service
                .action(
                    render.device.url(),
                    DLNA_ACTION_SET_AV_TRANSPORT_URI,
                    setavtransporturi_payload.as_str(),
                )
                .await
        },
        "SetAVTransportURI",
    )
    .await
    .map_err(|err| Error::DlnaSetTransportUriFailed {
        source: err,
        uri: video_uri.clone(),
    })?;

    info!("{LOG_MSG_PLAYING_VIDEO}");
    let play_payload = build_play_payload(
        crate::config::DLNA_INSTANCE_ID,
        crate::config::DLNA_DEFAULT_SPEED,
    );
    retry_with_backoff(
        || async {
            render
                .service
                .action(
                    render.device.url(),
                    crate::config::DLNA_ACTION_PLAY,
                    &play_payload,
                )
                .await
        },
        "Play",
    )
    .await
    .map_err(|err| Error::DlnaPlaybackFailed {
        source: err,
        context: MEDIA_PLAYBACK_FAILED_MSG.to_string(),
    })?;

    // Start subtitle synchronization task if enabled
    let subtitle_sync_handle = if let Some(mut syncer) = subtitle_syncer {
        info!("Starting subtitle synchronization...");
        let render_clone = render.clone();
        let sync_interval_ms = config.subtitle_sync_interval_ms;
        Some(tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(sync_interval_ms));
            loop {
                interval.tick().await;

                // Get playback position
                match render_clone.get_position_info().await {
                    Ok(position_info) => {
                        // Convert time format to milliseconds
                        let position_ms =
                            crate::utils::time_str_to_milliseconds(&position_info.rel_time);

                        // Update subtitle content in clipboard
                        if let Err(e) = syncer.update_clipboard(position_ms) {
                            eprintln!("Failed to update clipboard: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get position info: {e}");
                    }
                }
            }
        }))
    } else {
        None
    };

    streaming_server_handle
        .await
        .map_err(|err| Error::StreamingServerError {
            source: err,
            context: "Media streaming server encountered an error".to_string(),
        })?;

    // Cancel subtitle synchronization task
    if let Some(handle) = subtitle_sync_handle {
        handle.abort();
    }

    Ok(())
}

//! DLNA action implementations for crab-dlna
//!
//! This module contains implementations of various DLNA actions
//! such as play, pause, resume, and transport control.

use crate::{
    config::{
        DLNA_ACTION_PAUSE, DLNA_ACTION_PLAY, DLNA_DEFAULT_SPEED, DLNA_INSTANCE_ID,
    },
    devices::Render,
    error::{Error, Result},
    utils::retry_with_backoff,
};
use log::info;

/// Builds a DLNA play payload with configurable parameters
fn build_play_payload(instance_id: u32, speed: u32) -> String {
    format!(
        r#"
    <InstanceID>{instance_id}</InstanceID>
    <Speed>{speed}</Speed>
"#
    )
}

/// Builds a DLNA pause payload
fn build_pause_payload(instance_id: u32) -> String {
    format!(
        r#"
    <InstanceID>{instance_id}</InstanceID>
"#
    )
}

/// Pauses playback on a DLNA device
pub async fn pause(render: &Render) -> Result<()> {
    let pause_payload = build_pause_payload(DLNA_INSTANCE_ID);
    retry_with_backoff(
        || async {
            render
                .service
                .action(render.device.url(), DLNA_ACTION_PAUSE, &pause_payload)
                .await
        },
        "Pause",
    )
    .await
    .map_err(|err| Error::DlnaPlaybackFailed {
        source: err,
        context: "Failed to pause media playback on render device".to_string(),
    })?;

    info!("Media playback paused");
    Ok(())
}

/// Resumes playback on a DLNA device
pub async fn resume(render: &Render) -> Result<()> {
    let play_payload = build_play_payload(DLNA_INSTANCE_ID, DLNA_DEFAULT_SPEED);
    retry_with_backoff(
        || async {
            render
                .service
                .action(render.device.url(), DLNA_ACTION_PLAY, &play_payload)
                .await
        },
        "Resume",
    )
    .await
    .map_err(|err| Error::DlnaPlaybackFailed {
        source: err,
        context: "Failed to resume media playback on render device".to_string(),
    })?;

    info!("Media playback resumed");
    Ok(())
}

/// Toggles play/pause state based on current transport state
pub async fn toggle_play_pause(render: &Render) -> Result<()> {
    let transport_info = render.get_transport_info().await?;

    match transport_info.transport_state.as_str() {
        "PLAYING" => {
            info!("Currently playing, pausing...");
            pause(render).await
        }
        "PAUSED_PLAYBACK" | "STOPPED" => {
            info!("Currently paused/stopped, resuming...");
            resume(render).await
        }
        state => {
            info!("Unknown transport state: {state}, attempting to resume...");
            resume(render).await
        }
    }
}

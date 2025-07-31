use crate::{
    config::{
        Config, DEFAULT_DLNA_VIDEO_TITLE, DLNA_ACTION_PLAY, DLNA_ACTION_SET_AV_TRANSPORT_URI,
        DLNA_DEFAULT_SPEED, DLNA_INSTANCE_ID, LOG_MSG_PLAYING_VIDEO, LOG_MSG_SETTING_VIDEO_URI,
        MEDIA_PLAYBACK_FAILED_MSG,
    },
    devices::Render,
    error::{Error, Result},
    streaming::MediaStreamingServer,
    subtitle_sync::SubtitleSyncer,
    utils::retry_with_backoff,
};
use log::{debug, info};
use std::time::Duration;
use tokio::time::interval;
use xml::escape::escape_str_attribute;

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

    let setavtransporturi_payload = build_setavtransporturi_payload(&streaming_server, &metadata);
    debug!("SetAVTransportURI payload: '{setavtransporturi_payload}'");

    // Get the video URI before moving streaming_server
    let video_uri = streaming_server.video_uri();

    info!("Starting media streaming server...");
    let streaming_server_handle = tokio::spawn(async move { streaming_server.run().await });

    info!("{}", LOG_MSG_SETTING_VIDEO_URI);
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

    info!("{}", LOG_MSG_PLAYING_VIDEO);
    let play_payload = build_play_payload(DLNA_INSTANCE_ID, DLNA_DEFAULT_SPEED);
    retry_with_backoff(
        || async {
            render
                .service
                .action(render.device.url(), DLNA_ACTION_PLAY, &play_payload)
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
                            eprintln!("Failed to update clipboard: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get position info: {}", e);
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

/// Builds the metadata XML for the media content
fn build_metadata(streaming_server: &MediaStreamingServer) -> Result<String> {
    let subtitle_uri = streaming_server.subtitle_uri();

    match subtitle_uri {
        Some(subtitle_uri) => {
            let metadata = format!(
                r###"
                    <DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/"
                                xmlns:dc="http://purl.org/dc/elements/1.1/" 
                                xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" 
                                xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/" 
                                xmlns:sec="http://www.sec.co.kr/" 
                                xmlns:xbmc="urn:schemas-xbmc-org:metadata-1-0/">
                        <item id="0" parentID="-1" restricted="1">
                            <dc:title>{}</dc:title>
                            <res protocolInfo="http-get:*:video/{type_video}:" xmlns:pv="http://www.pv.com/pvns/" pv:subtitleFileUri="{uri_sub}" pv:subtitleFileType="{type_sub}">{uri_video}</res>
                            <res protocolInfo="http-get:*:text/srt:*">{uri_sub}</res>
                            <res protocolInfo="http-get:*:smi/caption:*">{uri_sub}</res>
                            <sec:CaptionInfoEx sec:type="{type_sub}">{uri_sub}</sec:CaptionInfoEx>
                            <sec:CaptionInfo sec:type="{type_sub}">{uri_sub}</sec:CaptionInfo>
                            <upnp:class>object.item.videoItem.movie</upnp:class>
                        </item>
                    </DIDL-Lite>
                    "###,
                DEFAULT_DLNA_VIDEO_TITLE,
                uri_video = streaming_server.video_uri(),
                type_video = streaming_server.video_type(),
                uri_sub = subtitle_uri,
                type_sub = streaming_server
                    .subtitle_type()
                    .unwrap_or_else(|| "unknown".to_string())
            );
            Ok(escape_str_attribute(metadata.as_str()).to_string())
        }
        None => Ok("".to_string()),
    }
}

/// Builds the SetAVTransportURI payload
fn build_setavtransporturi_payload(
    streaming_server: &MediaStreamingServer,
    metadata: &str,
) -> String {
    format!(
        r#"
        <InstanceID>{}</InstanceID>
        <CurrentURI>{}</CurrentURI>
        <CurrentURIMetaData>{}</CurrentURIMetaData>
        "#,
        DLNA_INSTANCE_ID,
        streaming_server.video_uri(),
        metadata
    )
}

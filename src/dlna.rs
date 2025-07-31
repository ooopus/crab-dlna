use crate::{
    devices::Render,
    error::{Error, Result},
    streaming::MediaStreamingServer,
    subtitle_sync::SubtitleSyncer,
};
use log::{debug, info};
use std::time::Duration;
use tokio::time::interval;
use xml::escape::escape_str_attribute;

const PAYLOAD_PLAY: &str = r#"
    <InstanceID>0</InstanceID>
    <Speed>1</Speed>
"#;

/// Plays a media file in a DLNA compatible device render, according to the render and media streaming server provided
pub async fn play(
    render: Render,
    streaming_server: MediaStreamingServer,
    subtitle_syncer: Option<SubtitleSyncer>,
) -> Result<()> {
    let metadata = build_metadata(&streaming_server)?;
    debug!("Metadata: '{metadata}'");

    let setavtransporturi_payload = build_setavtransporturi_payload(&streaming_server, &metadata);
    debug!("SetAVTransportURI payload: '{setavtransporturi_payload}'");

    info!("Starting media streaming server...");
    let streaming_server_handle = tokio::spawn(async move { streaming_server.run().await });

    info!("Setting Video URI");
    render
        .service
        .action(
            render.device.url(),
            "SetAVTransportURI",
            setavtransporturi_payload.as_str(),
        )
        .await
        .map_err(Error::DLNASetAVTransportURIError)?;

    info!("Playing video");
    render
        .service
        .action(render.device.url(), "Play", PAYLOAD_PLAY)
        .await
        .map_err(Error::DLNAPlayError)?;

    // 如果启用了字幕同步功能，则启动字幕同步任务
    let subtitle_sync_handle = if let Some(mut syncer) = subtitle_syncer {
        info!("Starting subtitle synchronization...");
        let render_clone = render.clone();
        Some(tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(500)); // 每500毫秒检查一次播放位置
            loop {
                interval.tick().await;

                // 获取播放位置
                match render_clone.get_position_info().await {
                    Ok(position_info) => {
                        // 将时间格式转换为毫秒
                        let position_ms = time_str_to_milliseconds(&position_info.rel_time);

                        // 更新剪贴板中的字幕内容
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
        .map_err(Error::DLNAStreamingError)?;

    // 取消字幕同步任务
    if let Some(handle) = subtitle_sync_handle {
        handle.abort();
    }

    Ok(())
}

/// 将时间字符串转换为毫秒
///
/// # 参数
/// * `time_str` - 时间字符串（格式：HH:MM:SS）
///
/// # 返回值
/// 返回时间对应的毫秒数
fn time_str_to_milliseconds(time_str: &str) -> u64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return 0;
    }

    let hours: u64 = parts[0].parse().unwrap_or(0);
    let minutes: u64 = parts[1].parse().unwrap_or(0);
    let seconds: f64 = parts[2].parse().unwrap_or(0.0);

    ((hours as f64) * 3600.0 + (minutes as f64) * 60.0 + seconds) as u64 * 1000
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
                            <dc:title>crab-dlna Video</dc:title>
                            <res protocolInfo="http-get:*:video/{type_video}:" xmlns:pv="http://www.pv.com/pvns/" pv:subtitleFileUri="{uri_sub}" pv:subtitleFileType="{type_sub}">{uri_video}</res>
                            <res protocolInfo="http-get:*:text/srt:*">{uri_sub}</res>
                            <res protocolInfo="http-get:*:smi/caption:*">{uri_sub}</res>
                            <sec:CaptionInfoEx sec:type="{type_sub}">{uri_sub}</sec:CaptionInfoEx>
                            <sec:CaptionInfo sec:type="{type_sub}">{uri_sub}</sec:CaptionInfo>
                            <upnp:class>object.item.videoItem.movie</upnp:class>
                        </item>
                    </DIDL-Lite>
                    "###,
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
        <InstanceID>0</InstanceID>
        <CurrentURI>{}</CurrentURI>
        <CurrentURIMetaData>{}</CurrentURIMetaData>
        "#,
        streaming_server.video_uri(),
        metadata
    )
}

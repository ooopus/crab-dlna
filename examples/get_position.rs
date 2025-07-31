use crab_dlna::{Error, MediaStreamingServer, Render, RenderSpec, STREAMING_PORT_DEFAULT, play};
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 设置视频文件路径
    let video_path = PathBuf::from("test_video.mp4");

    // 发现DLNA设备
    let discover_timeout_secs = 5;
    let render_spec = RenderSpec::First(discover_timeout_secs);
    let render = Render::new(render_spec).await?;

    // 设置流媒体服务器
    let host_ip = "192.168.1.100".parse().unwrap(); // 应该使用实际的本地IP
    let host_port = STREAMING_PORT_DEFAULT;
    let media_streaming_server =
        MediaStreamingServer::new(&video_path, &None, &host_ip, &host_port)?;

    // 启动播放任务
    let render_clone = render.clone();
    let streaming_server = media_streaming_server.clone();
    tokio::spawn(async move {
        if let Err(e) = play(render_clone, streaming_server, None).await {
            eprintln!("播放错误: {}", e);
        }
    });

    // 等待播放开始
    sleep(Duration::from_secs(5)).await;

    // 定期获取播放进度
    loop {
        match render.get_position_info().await {
            Ok(position_info) => {
                println!("当前播放位置: {}", position_info.rel_time);
                println!("总时长: {}", position_info.track_duration);
            }
            Err(e) => {
                eprintln!("获取播放位置信息失败: {}", e);
            }
        }

        match render.get_transport_info().await {
            Ok(transport_info) => {
                println!("播放状态: {}", transport_info.transport_state);
            }
            Err(e) => {
                eprintln!("获取传输信息失败: {}", e);
            }
        }

        // 每秒获取一次进度
        sleep(Duration::from_secs(1)).await;
    }
}

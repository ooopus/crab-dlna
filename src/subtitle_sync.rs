//! 字幕同步模块
//!
//! 此模块提供字幕同步功能，包括解析字幕文件、根据播放时间获取当前字幕内容，
//! 以及将字幕内容复制到剪贴板等功能。

use crate::error::{Error, Result};
use arboard::Clipboard;
use std::path::Path;

/// 字幕条目
#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    /// 开始时间（毫秒）
    pub start_time: u64,
    /// 结束时间（毫秒）
    pub end_time: u64,
    /// 字幕文本
    pub text: String,
}

/// 字幕同步器
pub struct SubtitleSyncer {
    /// 解析后的字幕条目列表
    entries: Vec<SubtitleEntry>,
    /// 剪贴板实例
    clipboard: Option<Clipboard>,
}

impl SubtitleSyncer {
    /// 创建新的字幕同步器
    ///
    /// # 参数
    /// * `subtitle_path` - 字幕文件路径
    ///
    /// # 返回值
    /// 返回创建的字幕同步器实例
    pub fn new(subtitle_path: &Path) -> Result<Self> {
        // 解析字幕文件
        let entries = parse_subtitle_file(subtitle_path)?;

        // 初始化剪贴板
        let clipboard = match Clipboard::new() {
            Ok(clipboard) => Some(clipboard),
            Err(e) => {
                eprintln!("警告: 无法初始化剪贴板: {}", e);
                None
            }
        };

        Ok(SubtitleSyncer { entries, clipboard })
    }

    /// 根据播放时间获取当前字幕内容
    ///
    /// # 参数
    /// * `position_ms` - 播放位置（毫秒）
    ///
    /// # 返回值
    /// 返回当前应该显示的字幕文本，如果没有字幕则返回空字符串
    pub fn get_current_subtitle(&self, position_ms: u64) -> String {
        for entry in &self.entries {
            if position_ms >= entry.start_time && position_ms <= entry.end_time {
                return entry.text.clone();
            }
        }
        String::new()
    }

    /// 将字幕内容复制到剪贴板
    ///
    /// # 参数
    /// * `subtitle_text` - 要复制到剪贴板的字幕文本
    ///
    /// # 返回值
    /// 如果成功复制到剪贴板则返回Ok，否则返回Err
    pub fn copy_to_clipboard(&mut self, subtitle_text: &str) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard {
            clipboard
                .set_text(subtitle_text)
                .map_err(|e| Error::SubtitleSyncError(format!("无法复制到剪贴板: {}", e)))?;
        }
        Ok(())
    }

    /// 根据播放位置更新剪贴板中的字幕内容
    ///
    /// # 参数
    /// * `position_ms` - 播放位置（毫秒）
    ///
    /// # 返回值
    /// 如果成功更新剪贴板则返回Ok，否则返回Err
    pub fn update_clipboard(&mut self, position_ms: u64) -> Result<()> {
        let subtitle_text = self.get_current_subtitle(position_ms);
        self.copy_to_clipboard(&subtitle_text)
    }
}

/// 解析字幕文件
///
/// # 参数
/// * `subtitle_path` - 字幕文件路径
///
/// # 返回值
/// 返回解析后的字幕条目列表
fn parse_subtitle_file(subtitle_path: &Path) -> Result<Vec<SubtitleEntry>> {
    // 读取字幕文件内容
    let content = std::fs::read(subtitle_path)
        .map_err(|e| Error::SubtitleSyncError(format!("无法读取字幕文件: {}", e)))?;

    // 获取文件扩展名
    let extension = subtitle_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 根据扩展名确定字幕格式
    let format = subparse::get_subtitle_format_by_extension(Some(std::ffi::OsStr::new(&extension)))
        .ok_or_else(|| Error::SubtitleSyncError("无法确定字幕格式".to_string()))?;

    // 解析字幕文件
    let subtitle_file = subparse::parse_bytes(format, &content, None, 0.0)
        .map_err(|e| Error::SubtitleSyncError(format!("无法解析字幕文件: {}", e)))?;

    // 转换为统一的字幕条目格式
    let mut entries = Vec::new();

    // 获取所有字幕条目
    let subtitle_entries = subtitle_file
        .get_subtitle_entries()
        .map_err(|e| Error::SubtitleSyncError(format!("无法获取字幕条目: {}", e)))?;

    // 转换每个条目
    for entry in subtitle_entries {
        // 将时间转换为毫秒
        let start_time = entry.timespan.start.msecs() as u64;
        let end_time = entry.timespan.end.msecs() as u64;

        // 清理字幕文本
        let text = clean_subtitle_text(&entry.line.unwrap_or_default());

        entries.push(SubtitleEntry {
            start_time,
            end_time,
            text,
        });
    }

    Ok(entries)
}

/// 清理字幕文本
///
/// # 参数
/// * `text` - 原始字幕文本
///
/// # 返回值
/// 返回清理后的字幕文本
fn clean_subtitle_text(text: &str) -> String {
    // 移除字幕格式标记（如HTML标签）
    let cleaned = text.replace("<i>", "").replace("</i>", "");
    // 移除多余的空白字符
    cleaned.trim().to_string()
}

/// 将时间字符串转换为毫秒
///
/// # 参数
/// * `time_str` - 时间字符串（格式：HH:MM:SS,mmm）
///
/// # 返回值
/// 返回时间对应的毫秒数
#[allow(dead_code)]
fn time_str_to_milliseconds(time_str: &str) -> Result<u64> {
    let parts: Vec<&str> = time_str.split(&[',', ':']).collect();
    if parts.len() != 4 {
        return Err(Error::SubtitleSyncError("无效的时间格式".to_string()));
    }

    let hours: u64 = parts[0]
        .parse()
        .map_err(|_| Error::SubtitleSyncError("无效的小时数".to_string()))?;
    let minutes: u64 = parts[1]
        .parse()
        .map_err(|_| Error::SubtitleSyncError("无效的分钟数".to_string()))?;
    let seconds: u64 = parts[2]
        .parse()
        .map_err(|_| Error::SubtitleSyncError("无效的秒数".to_string()))?;
    let milliseconds: u64 = parts[3]
        .parse()
        .map_err(|_| Error::SubtitleSyncError("无效的毫秒数".to_string()))?;

    Ok(hours * 3600000 + minutes * 60000 + seconds * 1000 + milliseconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_parse_srt_subtitle() {
        // 创建临时SRT文件用于测试
        let mut file = File::create("test.srt").unwrap();
        writeln!(file, "1").unwrap();
        writeln!(file, "00:00:01,000 --> 00:00:03,000").unwrap();
        writeln!(file, "Hello, world!").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "2").unwrap();
        writeln!(file, "00:00:04,000 --> 00:00:06,000").unwrap();
        writeln!(file, "This is a test.").unwrap();

        // 解析字幕文件
        let entries = parse_subtitle_file(Path::new("test.srt")).unwrap();

        // 验证解析结果
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].start_time, 1000);
        assert_eq!(entries[0].end_time, 3000);
        assert_eq!(entries[0].text, "Hello, world!");
        assert_eq!(entries[1].start_time, 4000);
        assert_eq!(entries[1].end_time, 6000);
        assert_eq!(entries[1].text, "This is a test.");

        // 清理临时文件
        std::fs::remove_file("test.srt").unwrap();
    }

    #[test]
    fn test_get_current_subtitle() {
        let entries = vec![
            SubtitleEntry {
                start_time: 1000,
                end_time: 3000,
                text: "Hello, world!".to_string(),
            },
            SubtitleEntry {
                start_time: 4000,
                end_time: 6000,
                text: "This is a test.".to_string(),
            },
        ];

        let syncer = SubtitleSyncer {
            entries,
            clipboard: None,
        };

        // 测试在字幕时间段内的情况
        assert_eq!(syncer.get_current_subtitle(2000), "Hello, world!");
        assert_eq!(syncer.get_current_subtitle(5000), "This is a test.");

        // 测试在字幕时间段外的情况
        assert_eq!(syncer.get_current_subtitle(0), "");
        assert_eq!(syncer.get_current_subtitle(3500), "");
        assert_eq!(syncer.get_current_subtitle(7000), "");
    }
}

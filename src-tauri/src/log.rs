use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// 日志级别
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Error => "ERROR",
        }
    }
}

/// 日志文件路径
fn log_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_default();
    path.push(".token-proxy");
    path
}

fn log_file_path() -> PathBuf {
    let mut path = log_path();
    path.push("token-proxy.log");
    path
}

/// 确保日志目录存在
fn ensure_log_dir() {
    let path = log_path();
    let _ = fs::create_dir_all(&path);
}

/// 写入日志
///
/// 日志文件位置: ~/.token-proxy/token-proxy.log
/// 格式: [时间] [级别] [标签] 消息
pub fn log(tag: &str, level: LogLevel, message: &str) {
    ensure_log_dir();
    let path = log_file_path();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let now = chrono_or_fallback();
        let line = format!("[{}] [{}] [{}] {}\n", now, level.as_str(), tag, message);
        let _ = file.write_all(line.as_bytes());
    }
}

/// 获取日志内容（最新 N 行）
pub fn read_logs(max_lines: usize) -> Vec<String> {
    let path = log_file_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let total = lines.len();
            let start = if total > max_lines { total - max_lines } else { 0 };
            lines[start..].iter().map(|s| s.to_string()).collect()
        }
        Err(_) => vec![],
    }
}

/// 清空日志文件
pub fn clear_logs() {
    let path = log_file_path();
    let _ = fs::write(&path, "");
}

/// 在没有 chrono crate 的情况下获取时间字符串
fn chrono_or_fallback() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    // 简单计算 YYYY-MM-DD HH:MM:SS
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // 从 Unix epoch (1970-01-01) 开始计算年份
    let mut year = 1970i64;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let month_days = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1usize;
    for &md in month_days.iter() {
        if remaining_days < md {
            break;
        }
        remaining_days -= md;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_write_and_read() {
        clear_logs();
        log("test", LogLevel::Info, "hello world");
        let logs = read_logs(10);
        assert!(!logs.is_empty());
        assert!(logs[0].contains("hello world"));
        clear_logs();
    }

    #[test]
    fn test_chrono_format() {
        let s = chrono_or_fallback();
        // 格式应为 YYYY-MM-DD HH:MM:SS
        assert!(s.len() == 19, "时间格式错误: {s}");
    }
}
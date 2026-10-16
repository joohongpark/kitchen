use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::FixedOffset;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    /// 설정하면 로그인 필요
    pub password: Option<String>,
    pub ntfy_url: String,
    /// 없으면 알림 비활성화
    pub ntfy_topic: Option<String>,
    pub ntfy_token: Option<String>,
    /// 매일 알림 보낼 시각 (현지 시각, 0-23)
    pub notify_hour: u32,
    /// 유통기한 며칠 전부터 알릴지
    pub expiry_warn_days: i64,
    pub timezone: FixedOffset,
    /// 알림 클릭 시 열 주소 (예: http://172.30.1.25:8080)
    pub public_url: Option<String>,
}

fn var(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn parse<T: std::str::FromStr>(key: &str, default: T) -> Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match var(key) {
        Some(v) => v.parse().with_context(|| format!("{key} 값이 올바르지 않습니다: {v}")),
        None => Ok(default),
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let tz_hours: i32 = parse("KITCHEN_TZ_OFFSET_HOURS", 9)?;
        let notify_hour: u32 = parse("NOTIFY_HOUR", 9)?;
        anyhow::ensure!(notify_hour < 24, "NOTIFY_HOUR 는 0-23 이어야 합니다");
        Ok(Self {
            bind: parse("KITCHEN_BIND", "0.0.0.0:8080".parse()?)?,
            data_dir: PathBuf::from(var("KITCHEN_DATA_DIR").unwrap_or_else(|| "./data".into())),
            password: var("KITCHEN_PASSWORD"),
            ntfy_url: var("NTFY_URL").unwrap_or_else(|| "https://ntfy.sh".into()),
            ntfy_topic: var("NTFY_TOPIC"),
            ntfy_token: var("NTFY_TOKEN"),
            notify_hour,
            expiry_warn_days: parse("EXPIRY_WARN_DAYS", 3)?,
            timezone: FixedOffset::east_opt(tz_hours * 3600).context("KITCHEN_TZ_OFFSET_HOURS 범위 오류")?,
            public_url: var("KITCHEN_PUBLIC_URL"),
        })
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("kitchen.db")
    }

    pub fn photos_dir(&self) -> PathBuf {
        self.data_dir.join("photos")
    }
}

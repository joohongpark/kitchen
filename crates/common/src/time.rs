use chrono::{DateTime, Utc};

/// 저장용 타임스탬프. 고정 형식이라 문자열 비교가 시간 비교와 같다.
pub fn format(t: DateTime<Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn now() -> String {
    format(Utc::now())
}

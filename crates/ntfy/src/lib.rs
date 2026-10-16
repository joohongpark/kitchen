//! ntfy.sh 클라이언트. JSON 발행 API를 사용한다 (헤더에 한글을 넣지 않기 위해).

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum NtfyError {
    #[error("ntfy 요청 실패: {0}")]
    Http(#[from] reqwest::Error),
    #[error("ntfy 응답 오류: {0}")]
    Status(reqwest::StatusCode),
}

#[derive(Debug, Clone)]
pub struct NtfyClient {
    http: reqwest::Client,
    base_url: String,
    topic: String,
    token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Message {
    pub title: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// 1(min) ~ 5(max), 기본 3
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    /// 알림 클릭 시 열 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click: Option<String>,
}

#[derive(Serialize)]
struct Publish<'a> {
    topic: &'a str,
    #[serde(flatten)]
    msg: &'a Message,
}

impl NtfyClient {
    pub fn new(http: reqwest::Client, base_url: &str, topic: &str, token: Option<String>) -> Self {
        Self { http, base_url: base_url.trim_end_matches('/').to_string(), topic: topic.to_string(), token }
    }

    pub async fn send(&self, msg: &Message) -> Result<(), NtfyError> {
        let mut req = self.http.post(&self.base_url).json(&Publish { topic: &self.topic, msg });
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await?;
        if !res.status().is_success() {
            return Err(NtfyError::Status(res.status()));
        }
        Ok(())
    }
}

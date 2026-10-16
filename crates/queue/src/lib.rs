//! SQLite 기반 작업 큐 + 간단한 key-value 저장소.
//! 단일 인스턴스 전제라 `UPDATE … RETURNING` 하나로 작업을 가져온다.

use chrono::{Duration, Utc};
use common::id::new_id;
use common::{Db, Result, time};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Job {
    pub id: String,
    pub kind: String,
    pub payload: String,
    pub status: String,
    pub run_at: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Job {
    pub fn payload_json(&self) -> serde_json::Value {
        serde_json::from_str(&self.payload).unwrap_or(serde_json::Value::Null)
    }
}

#[derive(Debug, Clone)]
pub struct Queue {
    db: Db,
}

impl Queue {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn enqueue(&self, kind: &str, payload: serde_json::Value) -> Result<String> {
        let id = new_id();
        let now = time::now();
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload, status, run_at, created_at, updated_at) VALUES (?, ?, ?, 'queued', ?, ?, ?)",
        )
        .bind(&id)
        .bind(kind)
        .bind(payload.to_string())
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;
        Ok(id)
    }

    /// 실행할 작업 하나를 가져와 running 으로 표시
    pub async fn claim(&self) -> Result<Option<Job>> {
        let now = time::now();
        Ok(sqlx::query_as(
            "UPDATE jobs SET status = 'running', attempts = attempts + 1, updated_at = ?1 \
             WHERE id = (SELECT id FROM jobs WHERE status = 'queued' AND run_at <= ?1 ORDER BY run_at LIMIT 1) \
             RETURNING *",
        )
        .bind(&now)
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn complete(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE jobs SET status = 'done', last_error = NULL, updated_at = ? WHERE id = ?")
            .bind(time::now())
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// 실패 처리. 재시도 횟수가 남았으면 지수 백오프(30s, 60s, 120s…)로 다시 큐에 넣는다.
    pub async fn fail(&self, job: &Job, error: &str) -> Result<()> {
        let now = Utc::now();
        if job.attempts >= job.max_attempts {
            sqlx::query("UPDATE jobs SET status = 'failed', last_error = ?, updated_at = ? WHERE id = ?")
                .bind(error)
                .bind(time::format(now))
                .bind(&job.id)
                .execute(&self.db)
                .await?;
        } else {
            let delay = Duration::seconds(30 * (1i64 << (job.attempts - 1).clamp(0, 10)));
            sqlx::query("UPDATE jobs SET status = 'queued', last_error = ?, run_at = ?, updated_at = ? WHERE id = ?")
                .bind(error)
                .bind(time::format(now + delay))
                .bind(time::format(now))
                .bind(&job.id)
                .execute(&self.db)
                .await?;
        }
        Ok(())
    }

    /// 비정상 종료로 running 에 멈춘 작업을 되돌린다 (시작 시 호출)
    pub async fn requeue_running(&self) -> Result<u64> {
        Ok(sqlx::query("UPDATE jobs SET status = 'queued', updated_at = ? WHERE status = 'running'")
            .bind(time::now())
            .execute(&self.db)
            .await?
            .rows_affected())
    }

    /// 완료된 지 오래된 작업 정리
    pub async fn prune(&self, older_than: Duration) -> Result<u64> {
        Ok(sqlx::query("DELETE FROM jobs WHERE status IN ('done', 'failed') AND updated_at < ?")
            .bind(time::format(Utc::now() - older_than))
            .execute(&self.db)
            .await?
            .rows_affected())
    }
}

/// 내부 상태 저장 (예: 마지막 알림 날짜)
#[derive(Debug, Clone)]
pub struct Kv {
    db: Db,
}

impl Kv {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(sqlx::query_scalar("SELECT value FROM kv WHERE key = ?").bind(key).fetch_optional(&self.db).await?)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO kv (key, value, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(time::now())
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

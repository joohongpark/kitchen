//! 주기 작업을 큐에 넣는다. 단일 인스턴스라 리더 선출 없이 kv 로 하루 한 번만 실행되게 한다.

use std::time::Duration;

use chrono::{Timelike, Utc};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

const DIGEST_KEY: &str = "scheduler.expiry_digest.date";
const PRUNE_KEY: &str = "scheduler.prune.date";

pub async fn run(state: AppState, shutdown: CancellationToken) {
    let mut tick = tokio::time::interval(Duration::from_secs(60));
    loop {
        tokio::select! {
            _ = tick.tick() => {}
            _ = shutdown.cancelled() => break,
        }
        if let Err(err) = tick_once(&state).await {
            tracing::error!(error = %err, "scheduler tick failed");
        }
    }
}

async fn tick_once(state: &AppState) -> common::Result<()> {
    let local = Utc::now().with_timezone(&state.config.timezone);
    let today = local.date_naive().to_string();

    if local.hour() >= state.config.notify_hour && state.kv.get(DIGEST_KEY).await?.as_deref() != Some(&today) {
        state.queue.enqueue("expiry_digest", json!({ "date": today })).await?;
        state.kv.set(DIGEST_KEY, &today).await?;
        tracing::info!(%today, "enqueued expiry digest");
    }

    if state.kv.get(PRUNE_KEY).await?.as_deref() != Some(&today) {
        let jobs = state.queue.prune(chrono::Duration::days(14)).await?;
        let sessions = state.auth.prune_expired().await?;
        state.kv.set(PRUNE_KEY, &today).await?;
        tracing::info!(jobs, sessions, "pruned old rows");
    }
    Ok(())
}

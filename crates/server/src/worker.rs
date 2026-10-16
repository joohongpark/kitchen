//! 작업 큐 소비자

use std::time::Duration;

use chrono::Utc;
use common::Error;
use ntfy::Message;
use pantry::model::StockStatus;
use queue::Job;
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

const IDLE_WAIT: Duration = Duration::from_secs(3);

pub async fn run(state: AppState, shutdown: CancellationToken) {
    match state.queue.requeue_running().await {
        Ok(n) if n > 0 => tracing::warn!(n, "requeued jobs left running"),
        Err(err) => tracing::error!(error = %err, "requeue failed"),
        _ => {}
    }
    loop {
        if shutdown.is_cancelled() {
            break;
        }
        let job = match state.queue.claim().await {
            Ok(Some(job)) => job,
            Ok(None) => {
                tokio::select! {
                    _ = tokio::time::sleep(IDLE_WAIT) => {}
                    _ = shutdown.cancelled() => break,
                }
                continue;
            }
            Err(err) => {
                tracing::error!(error = %err, "claim failed");
                tokio::time::sleep(IDLE_WAIT).await;
                continue;
            }
        };

        let result = handle(&state, &job).await;
        let done = match &result {
            Ok(()) => state.queue.complete(&job.id).await,
            Err(err) => {
                tracing::warn!(job = %job.id, kind = %job.kind, attempt = job.attempts, error = %err, "job failed");
                state.queue.fail(&job, &err.to_string()).await
            }
        };
        if let Err(err) = done {
            tracing::error!(error = %err, "could not record job result");
        }
    }
}

async fn handle(state: &AppState, job: &Job) -> common::Result<()> {
    match job.kind.as_str() {
        "notify" => {
            let p = job.payload_json();
            let msg = Message {
                title: p["title"].as_str().unwrap_or("부엌").to_string(),
                message: p["message"].as_str().unwrap_or_default().to_string(),
                tags: vec!["bell".into()],
                click: state.config.public_url.clone(),
                ..Default::default()
            };
            send(state, &msg).await
        }
        "expiry_digest" => expiry_digest(state).await,
        other => Err(Error::internal(format!("알 수 없는 작업 종류: {other}"))),
    }
}

async fn send(state: &AppState, msg: &Message) -> common::Result<()> {
    match &state.ntfy {
        Some(client) => client.send(msg).await.map_err(Error::internal),
        None => {
            tracing::info!(title = %msg.title, "ntfy disabled, skipping notification");
            Ok(())
        }
    }
}

/// 유통기한 임박·지남 + 재고 부족 요약
async fn expiry_digest(state: &AppState) -> common::Result<()> {
    let today = Utc::now().with_timezone(&state.config.timezone).date_naive();
    let until = today + chrono::Duration::days(state.config.expiry_warn_days);
    let expiring = state.pantry.list_expiring(until).await?;
    let low: Vec<String> = state
        .pantry
        .list_ingredients()
        .await?
        .into_iter()
        .filter(|i| i.status == StockStatus::Low)
        .map(|i| match i.stock_summary {
            Some(s) => format!("{}({s})", i.ingredient.name),
            None => i.ingredient.name,
        })
        .collect();

    let (expired, soon): (Vec<_>, Vec<_>) = expiring.iter().partition(|it| it.expires_at.is_some_and(|d| d < today));
    let fmt = |it: &&pantry::model::PantryItem| {
        let d = it.expires_at.map(|d| d.format("%m/%d").to_string()).unwrap_or_default();
        format!("{} {} ({}, {d})", it.ingredient_name, it.amount_label(), it.storage.label())
    };

    let mut sections = Vec::new();
    if !expired.is_empty() {
        sections.push(format!("❌ 유통기한 지남\n{}", expired.iter().map(fmt).collect::<Vec<_>>().join("\n")));
    }
    if !soon.is_empty() {
        sections.push(format!(
            "⏰ {}일 이내\n{}",
            state.config.expiry_warn_days,
            soon.iter().map(fmt).collect::<Vec<_>>().join("\n")
        ));
    }
    if !low.is_empty() {
        sections.push(format!("🛒 부족\n{}", low.join(", ")));
    }
    if sections.is_empty() {
        tracing::info!("expiry digest: nothing to report");
        return Ok(());
    }

    let msg = Message {
        title: "오늘의 식재료 알림".into(),
        message: sections.join("\n\n"),
        tags: vec!["shallow_pan_of_food".into()],
        priority: Some(if expired.is_empty() { 3 } else { 4 }),
        click: state.config.public_url.clone(),
    };
    send(state, &msg).await
}

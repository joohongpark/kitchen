mod auth;
mod config;
mod http;
mod scheduler;
mod seed;
mod state;
mod webapp;
mod worker;

use anyhow::Context;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::state::AppState;

const USAGE: &str = "사용법: kitchen [serve | seed [파일.json]]";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn,tower_http=info".into()))
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = Config::from_env()?;
    std::fs::create_dir_all(&config.data_dir).context("데이터 디렉터리를 만들 수 없습니다")?;

    let db = common::db::connect(&config.db_path()).await?;
    common::db::migrate(&db).await?;
    let state = AppState::new(config, db).await?;

    match args.first().map(String::as_str) {
        None | Some("serve") => serve(state).await,
        Some("seed") => seed::run(&state, args.get(1).map(String::as_str)).await,
        Some(_) => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}

async fn serve(state: AppState) -> anyhow::Result<()> {
    let shutdown = CancellationToken::new();
    let bind = state.config.bind;
    if state.auth.enabled() {
        tracing::info!("password login enabled");
    }
    if state.ntfy.is_none() {
        tracing::info!("NTFY_TOPIC not set, notifications disabled");
    }

    let scheduler = tokio::spawn(scheduler::run(state.clone(), shutdown.clone()));
    let worker = tokio::spawn(worker::run(state.clone(), shutdown.clone()));

    let listener =
        tokio::net::TcpListener::bind(bind).await.with_context(|| format!("{bind} 에 바인드할 수 없습니다"))?;
    tracing::info!("listening on http://{bind}");
    let token = shutdown.clone();
    axum::serve(listener, http::router(state))
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            token.cancel();
        })
        .await?;

    shutdown.cancel();
    let _ = tokio::join!(scheduler, worker);
    tracing::info!("bye");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let term = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("signal handler").recv().await;
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
}

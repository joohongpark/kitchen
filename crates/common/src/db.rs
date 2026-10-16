use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Sqlite, SqliteConnection, SqlitePool};

use crate::Result;

pub type Db = SqlitePool;
pub type Transaction<'a> = sqlx::Transaction<'a, Sqlite>;

/// 저장소 메서드가 받는 실행기. 풀(`&Db`)과 트랜잭션(`&mut *tx`) 모두에서 같은 코드가 동작한다.
pub trait Queryer<'c>: sqlx::Executor<'c, Database = Sqlite> {}

impl<'c> Queryer<'c> for &SqlitePool {}
impl<'c> Queryer<'c> for &'c mut SqliteConnection {}

pub async fn connect(path: &Path) -> Result<Db> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .map_err(crate::Error::internal)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new().max_connections(8).connect_with(options).await?;
    Ok(pool)
}

pub async fn migrate(db: &Db) -> Result<()> {
    sqlx::migrate!("../../migrations").run(db).await?;
    Ok(())
}

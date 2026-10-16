/// 도메인 오류. HTTP 상태 코드로의 변환은 server 크레이트가 담당한다.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("찾을 수 없습니다: {0}")]
    NotFound(&'static str),
    #[error("이미 존재합니다: {0}")]
    AlreadyExists(String),
    #[error("잘못된 입력: {0}")]
    InvalidArgument(String),
    #[error("다른 곳에서 사용 중입니다: {0}")]
    InUse(String),
    #[error("인증이 필요합니다")]
    Unauthorized,
    #[error("내부 오류: {0}")]
    Internal(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    pub fn internal(err: impl std::fmt::Display) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "database error");
        Error::Internal(err.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Error::Internal(err.to_string())
    }
}

/// UNIQUE 제약 위반인지 확인한다. 저장소에서 도메인 오류로 매핑할 때 쓴다.
pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.is_unique_violation())
}

/// FOREIGN KEY 제약 위반인지 확인한다.
/// SQLite 는 ON DELETE RESTRICT 위반을 SQLITE_CONSTRAINT_TRIGGER(1811)로 보고해서 sqlx 가 분류하지 못한다.
pub fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db)
        if db.is_foreign_key_violation() || db.message().contains("FOREIGN KEY constraint failed"))
}

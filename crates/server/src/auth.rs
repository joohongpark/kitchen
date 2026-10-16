//! 단일 비밀번호 + 세션 쿠키 인증. `KITCHEN_PASSWORD` 가 없으면 비활성화.

use chrono::{Duration, Utc};
use common::{Db, Error, Result, time};
use rand::Rng;
use sha2::{Digest, Sha256};

pub const SESSION_COOKIE: &str = "kitchen_session";
pub const SESSION_DAYS: i64 = 30;

#[derive(Clone)]
pub struct AuthService {
    db: Db,
    password_digest: Option<[u8; 32]>,
}

fn digest(s: &str) -> [u8; 32] {
    Sha256::digest(s.as_bytes()).into()
}

impl AuthService {
    pub fn new(db: Db, password: Option<String>) -> Self {
        Self { db, password_digest: password.as_deref().map(digest) }
    }

    pub fn enabled(&self) -> bool {
        self.password_digest.is_some()
    }

    /// 성공하면 세션 토큰(쿠키 값)을 돌려준다
    pub async fn login(&self, password: &str) -> Result<String> {
        let expected = self.password_digest.ok_or(Error::invalid("로그인이 비활성화되어 있습니다"))?;
        // 다이제스트끼리 비교해 길이·내용에 따른 타이밍 차이를 줄인다
        let given = digest(password);
        let diff = expected.iter().zip(given.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b));
        if diff != 0 {
            return Err(Error::Unauthorized);
        }

        let mut raw = [0u8; 32];
        rand::rng().fill(&mut raw);
        let token = hex::encode(raw);
        let now = Utc::now();
        sqlx::query("INSERT INTO sessions (token_hash, created_at, expires_at) VALUES (?, ?, ?)")
            .bind(hex::encode(digest(&token)))
            .bind(time::format(now))
            .bind(time::format(now + Duration::days(SESSION_DAYS)))
            .execute(&self.db)
            .await?;
        Ok(token)
    }

    pub async fn verify(&self, token: &str) -> Result<bool> {
        let found: Option<String> =
            sqlx::query_scalar("SELECT token_hash FROM sessions WHERE token_hash = ? AND expires_at > ?")
                .bind(hex::encode(digest(token)))
                .bind(time::now())
                .fetch_optional(&self.db)
                .await?;
        Ok(found.is_some())
    }

    pub async fn logout(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(hex::encode(digest(token)))
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn prune_expired(&self) -> Result<u64> {
        Ok(sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
            .bind(time::now())
            .execute(&self.db)
            .await?
            .rows_affected())
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Storage {
    Fridge,
    Freezer,
    Room,
}

impl Storage {
    pub fn label(self) -> &'static str {
        match self {
            Storage::Fridge => "냉장",
            Storage::Freezer => "냉동",
            Storage::Room => "실온",
        }
    }
}

/// 재고 추적 방식
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Tracking {
    /// 수량 + 단위
    Quantity,
    /// 넉넉/보통/조금/없음
    Level,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Level {
    Out,
    Low,
    Medium,
    Plenty,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Level::Plenty => "넉넉",
            Level::Medium => "보통",
            Level::Low => "조금",
            Level::Out => "없음",
        }
    }
}

use chrono::NaiveDate;
use common::kinds::{Level, Storage, Tracking};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Ingredient {
    pub id: String,
    pub name: String,
    pub category: String,
    pub default_unit: Option<String>,
    pub default_storage: Storage,
    /// quantity 재고가 이 값 이하이면 "부족" (default_unit 기준)
    pub low_threshold: Option<f64>,
    pub photo_id: Option<String>,
    pub note: Option<String>,
    /// 묶음(상위 재료). 예: 청양 고춧가루 → 고춧가루
    pub parent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngredientInput {
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub default_unit: Option<String>,
    #[serde(default)]
    pub default_storage: Option<Storage>,
    #[serde(default)]
    pub low_threshold: Option<f64>,
    #[serde(default)]
    pub photo_id: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

/// 식재료별 재고 상태
/// 순서: None < Low < Ok
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StockStatus {
    /// 재고 없음
    None,
    /// 있지만 부족
    Low,
    Ok,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngredientWithStock {
    #[serde(flatten)]
    pub ingredient: Ingredient,
    pub status: StockStatus,
    /// 사람이 읽을 요약. 예: "2모", "보통", "300g + 넉넉"
    pub stock_summary: Option<String>,
    pub nearest_expiry: Option<NaiveDate>,
    /// 이 재료가 묶음이면 하위 재료 id. 묶음의 재고 상태는 하위 재료를 합친 것
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PantryItem {
    pub id: String,
    pub ingredient_id: String,
    pub ingredient_name: String,
    pub category: String,
    pub tracking: Tracking,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub level: Option<Level>,
    pub storage: Storage,
    pub purchased_at: Option<NaiveDate>,
    pub expires_at: Option<NaiveDate>,
    pub photo_id: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PantryItem {
    /// 다 써서 없는 상태인지
    pub fn is_depleted(&self) -> bool {
        match self.tracking {
            Tracking::Quantity => self.quantity.unwrap_or(0.0) <= 0.0,
            Tracking::Level => self.level.unwrap_or(Level::Out) == Level::Out,
        }
    }

    pub fn amount_label(&self) -> String {
        match self.tracking {
            Tracking::Quantity => {
                format!("{}{}", format_qty(self.quantity.unwrap_or(0.0)), self.unit.as_deref().unwrap_or(""))
            }
            Tracking::Level => self.level.unwrap_or(Level::Out).label().to_string(),
        }
    }
}

pub fn format_qty(q: f64) -> String {
    if (q - q.round()).abs() < 1e-9 {
        format!("{}", q.round() as i64)
    } else {
        format!("{:.2}", q).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PantryItemInput {
    pub ingredient_id: String,
    pub tracking: Tracking,
    #[serde(default)]
    pub quantity: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub level: Option<Level>,
    #[serde(default)]
    pub storage: Option<Storage>,
    #[serde(default)]
    pub purchased_at: Option<NaiveDate>,
    #[serde(default)]
    pub expires_at: Option<NaiveDate>,
    #[serde(default)]
    pub photo_id: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// 재고 수량만 바꾸는 입력 (요리 후 차감, 빠른 조정)
#[derive(Debug, Clone, Deserialize)]
pub struct StockAdjustment {
    pub pantry_item_id: String,
    #[serde(default)]
    pub quantity: Option<f64>,
    #[serde(default)]
    pub level: Option<Level>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ShoppingItem {
    pub id: String,
    pub ingredient_id: Option<String>,
    pub name: String,
    pub amount: Option<String>,
    pub done: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShoppingItemInput {
    #[serde(default)]
    pub ingredient_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub done: Option<bool>,
}

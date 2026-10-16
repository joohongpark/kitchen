use chrono::NaiveDate;
use common::kinds::{Level, Storage, Tracking};
use pantry::model::StockStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecipeRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub servings: Option<i64>,
    pub cook_minutes: Option<i64>,
    pub tags: String,
    pub photo_id: Option<String>,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub servings: Option<i64>,
    pub cook_minutes: Option<i64>,
    pub tags: Vec<String>,
    pub photo_id: Option<String>,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RecipeRow> for Recipe {
    fn from(r: RecipeRow) -> Self {
        Recipe {
            tags: serde_json::from_str(&r.tags).unwrap_or_default(),
            id: r.id,
            title: r.title,
            description: r.description,
            servings: r.servings,
            cook_minutes: r.cook_minutes,
            photo_id: r.photo_id,
            source: r.source,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RecipeIngredient {
    pub recipe_id: String,
    pub ingredient_id: String,
    pub ingredient_name: String,
    pub category: String,
    pub position: i64,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub amount_text: Option<String>,
    pub optional: bool,
    pub note: Option<String>,
}

impl RecipeIngredient {
    pub fn amount_label(&self) -> String {
        match (&self.quantity, &self.unit, &self.amount_text) {
            (Some(q), u, _) => format!("{}{}", pantry::model::format_qty(*q), u.as_deref().unwrap_or("")),
            (None, _, Some(t)) => t.clone(),
            _ => String::new(),
        }
    }
}

/// 레시피 재료 줄의 대체 재료
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RecipeAlternative {
    pub recipe_id: String,
    pub ingredient_id: String,
    pub alt_ingredient_id: String,
    pub alt_name: String,
    pub position: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeStep {
    pub position: i64,
    pub body: String,
    pub photo_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngredientLine {
    #[serde(flatten)]
    pub item: RecipeIngredient,
    pub amount_label: String,
    /// 원래 재료의 재고 상태 (묶음이면 하위 재료 포함)
    pub status: StockStatus,
    pub stock_summary: Option<String>,
    pub alternatives: Vec<AlternativeLine>,
    /// 원래 재료가 없고 대체 재료가 있을 때, 쓸 수 있는 대체 재료 이름
    pub substitute: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlternativeLine {
    pub ingredient_id: String,
    pub name: String,
    pub note: Option<String>,
    pub status: StockStatus,
    pub stock_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecipeDetail {
    #[serde(flatten)]
    pub recipe: Recipe,
    pub ingredients: Vec<IngredientLine>,
    pub steps: Vec<RecipeStep>,
    pub cook_count: i64,
    pub last_cooked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecipeSummary {
    #[serde(flatten)]
    pub recipe: Recipe,
    pub required_count: usize,
    pub missing: Vec<MissingIngredient>,
    /// 대체 재료로 채운 필수 재료
    pub substitutions: Vec<Substitution>,
    pub cook_count: i64,
    pub last_cooked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissingIngredient {
    pub ingredient_id: String,
    pub name: String,
    pub amount_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Substitution {
    /// 원래 재료
    pub original: String,
    /// 대신 쓸 재료
    pub substitute: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecipeInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub servings: Option<i64>,
    #[serde(default)]
    pub cook_minutes: Option<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub photo_id: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub ingredients: Vec<RecipeIngredientInput>,
    #[serde(default)]
    pub steps: Vec<RecipeStepInput>,
}

/// `ingredient_id` 또는 `name` 중 하나. 이름만 주면 식재료를 자동으로 만든다.
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeIngredientInput {
    #[serde(default)]
    pub ingredient_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub quantity: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub amount_text: Option<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub alternatives: Vec<AlternativeInput>,
}

/// 대체 재료. `ingredient_id` 또는 `name` (이름만 주면 식재료 자동 생성)
#[derive(Debug, Clone, Deserialize)]
pub struct AlternativeInput {
    #[serde(default)]
    pub ingredient_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecipeStepInput {
    pub body: String,
    #[serde(default)]
    pub photo_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendRequest {
    /// 비어 있으면 현재 재고 전체 기준
    #[serde(default)]
    pub ingredient_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    #[serde(flatten)]
    pub summary: RecipeSummary,
    /// 선택한 재료 중 이 레시피에 쓰이는 것
    pub matched: Vec<String>,
    pub have_count: usize,
    pub coverage: f64,
}

// ---------- 요리 / 재고 차감 ----------

#[derive(Debug, Clone, Serialize)]
pub struct CookPlan {
    pub recipe_id: String,
    pub scale: f64,
    pub lines: Vec<CookPlanLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CookPlanLine {
    pub ingredient_id: String,
    pub ingredient_name: String,
    pub amount_label: String,
    pub optional: bool,
    pub missing: bool,
    pub lots: Vec<CookPlanLot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CookPlanLot {
    pub pantry_item_id: String,
    /// 실제 재고 재료 (묶음의 하위 재료나 대체 재료일 수 있음)
    pub ingredient_id: String,
    pub ingredient_name: String,
    /// 원래 재료가 아닌 대체 재료인지
    pub is_substitute: bool,
    pub tracking: Tracking,
    pub storage: Storage,
    pub expires_at: Option<NaiveDate>,
    pub unit: Option<String>,
    pub current_quantity: Option<f64>,
    pub current_level: Option<Level>,
    /// 차감 후 제안값 (사용자가 수정 가능)
    pub suggested_quantity: Option<f64>,
    pub suggested_level: Option<Level>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CookRequest {
    #[serde(default)]
    pub adjustments: Vec<pantry::model::StockAdjustment>,
    #[serde(default)]
    pub note: Option<String>,
    /// 부족한 필수 재료를 장보기 목록에 추가
    #[serde(default)]
    pub add_missing_to_shopping: bool,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CookLog {
    pub id: String,
    pub recipe_id: String,
    pub cooked_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CookStats {
    pub recipe_id: String,
    pub cook_count: i64,
    pub last_cooked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CookResult {
    pub log: CookLog,
    /// 장보기 목록에 추가된 재료 이름
    pub added_to_shopping: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetPhotoInput {
    pub photo_id: Option<String>,
}

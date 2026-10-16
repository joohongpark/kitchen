use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use common::id::new_id;
use common::kinds::Tracking;
use common::{Db, Error, Result, time, units};
use pantry::PantryService;
use pantry::model::{PantryItem, ShoppingItemInput, StockStatus};
use sqlx::SqliteConnection;

use crate::model::*;
use crate::repository::RecipesRepository;

#[derive(Debug, Clone)]
pub struct RecipesService {
    db: Db,
    repo: RecipesRepository,
    pantry: PantryService,
}

/// 식재료 id → (재고 상태, 요약)
type StockMap = HashMap<String, (StockStatus, Option<String>)>;

impl RecipesService {
    pub fn new(db: Db, pantry: PantryService) -> Self {
        Self { db, repo: RecipesRepository, pantry }
    }

    async fn stock_map(&self) -> Result<StockMap> {
        Ok(self
            .pantry
            .list_ingredients()
            .await?
            .into_iter()
            .map(|i| (i.ingredient.id, (i.status, i.stock_summary)))
            .collect())
    }

    async fn stats_map(&self) -> Result<HashMap<String, CookStats>> {
        Ok(self.repo.cook_stats(&self.db).await?.into_iter().map(|s| (s.recipe_id.clone(), s)).collect())
    }

    pub async fn list(&self, query: Option<&str>, tag: Option<&str>) -> Result<Vec<RecipeSummary>> {
        let rows = self.repo.list_recipes(&self.db).await?;
        let mut lines_by_recipe = group_lines(self.repo.list_all_ingredients(&self.db).await?);
        let stock = self.stock_map().await?;
        let stats = self.stats_map().await?;
        let q = query.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty());

        let mut out = Vec::new();
        for row in rows {
            let recipe = Recipe::from(row);
            let lines = lines_by_recipe.remove(&recipe.id).unwrap_or_default();
            if let Some(tag) = tag
                && !recipe.tags.iter().any(|t| t == tag)
            {
                continue;
            }
            if let Some(q) = &q {
                let hit = recipe.title.to_lowercase().contains(q)
                    || lines.iter().any(|l| l.ingredient_name.to_lowercase().contains(q));
                if !hit {
                    continue;
                }
            }
            out.push(summarize(recipe, &lines, &stock, &HashSet::new(), &stats));
        }
        Ok(out)
    }

    pub async fn get(&self, id: &str) -> Result<RecipeDetail> {
        let recipe = Recipe::from(self.repo.find_recipe(&self.db, id).await?);
        let lines = self.repo.list_ingredients(&self.db, id).await?;
        let steps = self.repo.list_steps(&self.db, id).await?;
        let stock = self.stock_map().await?;
        let stats = self.stats_map().await?;
        let ingredients = lines
            .into_iter()
            .map(|item| {
                let (status, stock_summary) =
                    stock.get(&item.ingredient_id).cloned().unwrap_or((StockStatus::None, None));
                IngredientLine { amount_label: item.amount_label(), item, status, stock_summary }
            })
            .collect();
        let st = stats.get(id);
        Ok(RecipeDetail {
            recipe,
            ingredients,
            steps,
            cook_count: st.map_or(0, |s| s.cook_count),
            last_cooked_at: st.and_then(|s| s.last_cooked_at.clone()),
        })
    }

    pub async fn create(&self, input: RecipeInput) -> Result<RecipeDetail> {
        let id = new_id();
        let now = time::now();
        let mut tx = self.db.begin().await?;
        let row = build_row(id.clone(), &input, now.clone(), now)?;
        self.repo.insert_recipe(&mut *tx, &row).await?;
        self.write_children(&mut tx, &id, &input).await?;
        tx.commit().await?;
        self.get(&id).await
    }

    pub async fn update(&self, id: &str, input: RecipeInput) -> Result<RecipeDetail> {
        let mut tx = self.db.begin().await?;
        let existing = self.repo.find_recipe(&mut *tx, id).await?;
        let row = build_row(existing.id, &input, existing.created_at, time::now())?;
        self.repo.update_recipe(&mut *tx, &row).await?;
        self.repo.delete_ingredients(&mut *tx, id).await?;
        self.repo.delete_steps(&mut *tx, id).await?;
        self.write_children(&mut tx, id, &input).await?;
        tx.commit().await?;
        self.get(id).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.repo.delete_recipe(&self.db, id).await
    }

    async fn write_children(&self, conn: &mut SqliteConnection, recipe_id: &str, input: &RecipeInput) -> Result<()> {
        for (pos, ing) in input.ingredients.iter().enumerate() {
            let (ingredient_id, ingredient_name) = match (&ing.ingredient_id, &ing.name) {
                (Some(id), _) if !id.trim().is_empty() => (id.clone(), ing.name.clone().unwrap_or_default()),
                (_, Some(name)) if !name.trim().is_empty() => {
                    let found = self
                        .pantry
                        .ensure_ingredient(&mut *conn, name, ing.category.as_deref(), ing.unit.as_deref())
                        .await?;
                    (found.id, found.name)
                }
                _ => return Err(Error::invalid(format!("{}번째 재료: 식재료를 지정하세요", pos + 1))),
            };
            if ing.quantity.is_some_and(|q| q < 0.0) {
                return Err(Error::invalid(format!("{ingredient_name}: 양은 0 이상이어야 합니다")));
            }
            let line = RecipeIngredient {
                recipe_id: recipe_id.to_string(),
                ingredient_id,
                ingredient_name,
                category: String::new(),
                position: pos as i64,
                quantity: ing.quantity,
                unit: clean(&ing.unit),
                amount_text: clean(&ing.amount_text),
                optional: ing.optional,
                note: clean(&ing.note),
            };
            self.repo.insert_ingredient(&mut *conn, &line).await?;
        }
        let mut pos = 0;
        for step in &input.steps {
            let body = step.body.trim();
            if body.is_empty() {
                continue;
            }
            let step = RecipeStep { position: pos, body: body.to_string(), photo_id: clean(&step.photo_id) };
            self.repo.insert_step(&mut *conn, recipe_id, &step).await?;
            pos += 1;
        }
        Ok(())
    }

    /// 선택한 재료(없으면 재고 전체)로 만들 수 있는 레시피 추천
    pub async fn recommend(&self, req: RecommendRequest) -> Result<Vec<Recommendation>> {
        let rows = self.repo.list_recipes(&self.db).await?;
        let mut lines_by_recipe = group_lines(self.repo.list_all_ingredients(&self.db).await?);
        let stock = self.stock_map().await?;
        let stats = self.stats_map().await?;
        let selected: HashSet<String> = req.ingredient_ids.into_iter().collect();

        let mut out = Vec::new();
        for row in rows {
            let recipe = Recipe::from(row);
            let lines = lines_by_recipe.remove(&recipe.id).unwrap_or_default();
            let matched: Vec<String> = lines
                .iter()
                .filter(|l| selected.contains(&l.ingredient_id))
                .map(|l| l.ingredient_name.clone())
                .collect();
            if !selected.is_empty() && matched.is_empty() {
                continue;
            }
            let summary = summarize(recipe, &lines, &stock, &selected, &stats);
            let have_count = summary.required_count - summary.missing.len();
            if selected.is_empty() && have_count == 0 && summary.required_count > 0 {
                continue;
            }
            let coverage =
                if summary.required_count == 0 { 1.0 } else { have_count as f64 / summary.required_count as f64 };
            out.push(Recommendation { summary, matched, have_count, coverage });
        }
        out.sort_by(|a, b| {
            a.summary
                .missing
                .len()
                .cmp(&b.summary.missing.len())
                .then(b.matched.len().cmp(&a.matched.len()))
                .then(b.coverage.partial_cmp(&a.coverage).unwrap_or(Ordering::Equal))
                .then(a.summary.recipe.title.cmp(&b.summary.recipe.title))
        });
        Ok(out)
    }

    /// 요리 시 재고 차감안. `servings`가 있으면 레시피 인분 대비로 양을 조정한다.
    pub async fn cook_plan(&self, id: &str, servings: Option<f64>) -> Result<CookPlan> {
        let recipe = self.repo.find_recipe(&self.db, id).await?;
        let lines = self.repo.list_ingredients(&self.db, id).await?;
        let items = self.pantry.list_items().await?;
        let scale = match (servings, recipe.servings) {
            (Some(s), Some(base)) if s > 0.0 && base > 0 => s / base as f64,
            _ => 1.0,
        };

        let plan_lines = lines
            .iter()
            .map(|line| {
                let lots: Vec<&PantryItem> =
                    items.iter().filter(|it| it.ingredient_id == line.ingredient_id && !it.is_depleted()).collect();
                // 남은 필요량 (레시피 단위)
                let mut remaining = match (line.quantity, &line.unit) {
                    (Some(q), Some(_)) => Some(q * scale),
                    _ => None,
                };
                let plan_lots = lots
                    .iter()
                    .map(|lot| {
                        let mut suggested_quantity = lot.quantity;
                        if lot.tracking == Tracking::Quantity
                            && let (Some(need), Some(rec_unit), Some(cur), Some(lot_unit)) =
                                (remaining, &line.unit, lot.quantity, &lot.unit)
                            && let Some(need_in_lot) = units::convert(need, rec_unit, lot_unit)
                        {
                            let take = need_in_lot.min(cur);
                            suggested_quantity = Some(round2(cur - take));
                            remaining = units::convert(need_in_lot - take, lot_unit, rec_unit);
                        }
                        CookPlanLot {
                            pantry_item_id: lot.id.clone(),
                            tracking: lot.tracking,
                            storage: lot.storage,
                            expires_at: lot.expires_at,
                            unit: lot.unit.clone(),
                            current_quantity: lot.quantity,
                            current_level: lot.level,
                            suggested_quantity,
                            suggested_level: lot.level,
                        }
                    })
                    .collect::<Vec<_>>();
                let amount_label = match (line.quantity, &line.unit) {
                    (Some(q), u) if scale != 1.0 => {
                        format!("{}{}", pantry::model::format_qty(round2(q * scale)), u.as_deref().unwrap_or(""))
                    }
                    _ => line.amount_label(),
                };
                CookPlanLine {
                    ingredient_id: line.ingredient_id.clone(),
                    ingredient_name: line.ingredient_name.clone(),
                    amount_label,
                    optional: line.optional,
                    missing: plan_lots.is_empty(),
                    lots: plan_lots,
                }
            })
            .collect();

        Ok(CookPlan { recipe_id: recipe.id, scale, lines: plan_lines })
    }

    /// 요리 완료: 재고 변경 + 기록 + (선택) 부족 재료 장보기 추가를 한 트랜잭션으로
    pub async fn cook(&self, id: &str, req: CookRequest) -> Result<CookResult> {
        let mut tx = self.db.begin().await?;
        let recipe = self.repo.find_recipe(&mut *tx, id).await?;
        self.pantry.adjust_in(&mut tx, &req.adjustments).await?;

        let log =
            CookLog { id: new_id(), recipe_id: recipe.id.clone(), cooked_at: time::now(), note: clean(&req.note) };
        self.repo.insert_cook_log(&mut *tx, &log).await?;

        let mut added = Vec::new();
        if req.add_missing_to_shopping {
            let lines = self.repo.list_ingredients(&mut *tx, id).await?;
            let items = self.pantry.list_items_in(&mut tx).await?;
            for line in lines.iter().filter(|l| !l.optional) {
                let in_stock = items.iter().any(|it| it.ingredient_id == line.ingredient_id && !it.is_depleted());
                if !in_stock {
                    let input = ShoppingItemInput {
                        ingredient_id: Some(line.ingredient_id.clone()),
                        name: None,
                        amount: None,
                        done: None,
                    };
                    self.pantry.add_shopping_in(&mut tx, input).await?;
                    added.push(line.ingredient_name.clone());
                }
            }
        }
        tx.commit().await?;
        Ok(CookResult { log, added_to_shopping: added })
    }

    pub async fn cook_logs(&self, id: &str) -> Result<Vec<CookLog>> {
        self.repo.list_cook_logs(&self.db, id).await
    }
}

fn clean(s: &Option<String>) -> Option<String> {
    s.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn build_row(id: String, input: &RecipeInput, created_at: String, updated_at: String) -> Result<RecipeRow> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(Error::invalid("레시피 이름이 필요합니다"));
    }
    if input.servings.is_some_and(|s| s <= 0) {
        return Err(Error::invalid("인분은 1 이상이어야 합니다"));
    }
    let mut tags: Vec<String> = input.tags.iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    tags.dedup();
    Ok(RecipeRow {
        id,
        title,
        description: clean(&input.description),
        servings: input.servings,
        cook_minutes: input.cook_minutes,
        tags: serde_json::to_string(&tags).map_err(Error::internal)?,
        photo_id: clean(&input.photo_id),
        source: clean(&input.source),
        created_at,
        updated_at,
    })
}

fn group_lines(lines: Vec<RecipeIngredient>) -> HashMap<String, Vec<RecipeIngredient>> {
    let mut map: HashMap<String, Vec<RecipeIngredient>> = HashMap::new();
    for l in lines {
        map.entry(l.recipe_id.clone()).or_default().push(l);
    }
    map
}

/// `assume_have`: 재고와 상관없이 있다고 보는 재료 (추천에서 사용자가 고른 재료)
fn summarize(
    recipe: Recipe,
    lines: &[RecipeIngredient],
    stock: &StockMap,
    assume_have: &HashSet<String>,
    stats: &HashMap<String, CookStats>,
) -> RecipeSummary {
    let required: Vec<&RecipeIngredient> = lines.iter().filter(|l| !l.optional).collect();
    let missing = required
        .iter()
        .filter(|l| {
            !assume_have.contains(&l.ingredient_id)
                && stock.get(&l.ingredient_id).is_none_or(|(s, _)| *s == StockStatus::None)
        })
        .map(|l| MissingIngredient {
            ingredient_id: l.ingredient_id.clone(),
            name: l.ingredient_name.clone(),
            amount_label: l.amount_label(),
        })
        .collect();
    let st = stats.get(&recipe.id);
    RecipeSummary {
        required_count: required.len(),
        missing,
        cook_count: st.map_or(0, |s| s.cook_count),
        last_cooked_at: st.and_then(|s| s.last_cooked_at.clone()),
        recipe,
    }
}

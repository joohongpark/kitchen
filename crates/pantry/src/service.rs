use chrono::NaiveDate;
use common::db::Transaction;
use common::id::new_id;
use common::kinds::{Level, Storage, Tracking};
use common::{Db, Error, Result, time, units};
use sqlx::SqliteConnection;

use crate::model::*;
use crate::repository::PantryRepository;

#[derive(Debug, Clone)]
pub struct PantryService {
    db: Db,
    repo: PantryRepository,
}

impl PantryService {
    pub fn new(db: Db) -> Self {
        Self { db, repo: PantryRepository }
    }

    // ---------- ingredients ----------

    /// 묶음 재료는 하위 재료 재고를 합친 상태로 돌려준다
    pub async fn list_ingredients(&self) -> Result<Vec<IngredientWithStock>> {
        let ingredients = self.repo.list_ingredients(&self.db).await?;
        let items = self.repo.list_items(&self.db).await?;
        Ok(aggregate(ingredients, &items))
    }

    pub async fn get_ingredient(&self, id: &str) -> Result<IngredientWithStock> {
        self.list_ingredients().await?.into_iter().find(|i| i.ingredient.id == id).ok_or(Error::NotFound("식재료"))
    }

    pub async fn create_ingredient(&self, input: IngredientInput) -> Result<Ingredient> {
        let now = time::now();
        let ing = build_ingredient(new_id(), input, now.clone(), now)?;
        self.check_parent(&ing).await?;
        self.repo.insert_ingredient(&self.db, &ing).await?;
        Ok(ing)
    }

    pub async fn update_ingredient(&self, id: &str, input: IngredientInput) -> Result<Ingredient> {
        let existing = self.repo.find_ingredient(&self.db, id).await?;
        let ing = build_ingredient(existing.id, input, existing.created_at, time::now())?;
        self.check_parent(&ing).await?;
        self.repo.update_ingredient(&self.db, &ing).await?;
        Ok(ing)
    }

    /// 묶음은 한 단계만: 묶음의 묶음, 자기 자신, 하위가 있는 재료를 다른 묶음에 넣기는 금지
    async fn check_parent(&self, ing: &Ingredient) -> Result<()> {
        let Some(pid) = &ing.parent_id else { return Ok(()) };
        if *pid == ing.id {
            return Err(Error::invalid("자기 자신을 묶음으로 지정할 수 없습니다"));
        }
        let parent = match self.repo.find_ingredient(&self.db, pid).await {
            Err(Error::NotFound(_)) => return Err(Error::invalid("묶음 재료가 존재하지 않습니다")),
            other => other?,
        };
        if parent.parent_id.is_some() {
            return Err(Error::invalid(format!(
                "'{}'은(는) 다른 묶음에 속해 있어 묶음으로 쓸 수 없습니다",
                parent.name
            )));
        }
        if self.repo.count_children(&self.db, &ing.id).await? > 0 {
            return Err(Error::invalid("하위 재료가 있는 묶음은 다른 묶음에 넣을 수 없습니다"));
        }
        Ok(())
    }

    pub async fn delete_ingredient(&self, id: &str) -> Result<()> {
        self.repo.delete_ingredient(&self.db, id).await
    }

    /// 이름으로 찾고 없으면 만든다 (레시피 작성, 시드 데이터에서 사용)
    pub async fn ensure_ingredient(
        &self,
        conn: &mut SqliteConnection,
        name: &str,
        category: Option<&str>,
        unit: Option<&str>,
    ) -> Result<Ingredient> {
        let name = name.trim();
        if let Some(found) = self.repo.find_ingredient_by_name(&mut *conn, name).await? {
            return Ok(found);
        }
        let now = time::now();
        let ing = build_ingredient(
            new_id(),
            IngredientInput {
                name: name.to_string(),
                category: category.map(str::to_string),
                default_unit: unit.map(str::to_string),
                default_storage: None,
                low_threshold: None,
                photo_id: None,
                note: None,
                parent_id: None,
            },
            now.clone(),
            now,
        )?;
        self.repo.insert_ingredient(&mut *conn, &ing).await?;
        Ok(ing)
    }

    // ---------- pantry items ----------

    pub async fn list_items(&self) -> Result<Vec<PantryItem>> {
        self.repo.list_items(&self.db).await
    }

    /// 레시피 서비스가 추천·차감 계산에 쓰는 전체 재고 (트랜잭션 안에서도 사용)
    pub async fn list_items_in(&self, conn: &mut SqliteConnection) -> Result<Vec<PantryItem>> {
        self.repo.list_items(conn).await
    }

    pub async fn get_item(&self, id: &str) -> Result<PantryItem> {
        self.repo.find_item(&self.db, id).await
    }

    pub async fn create_item(&self, input: PantryItemInput) -> Result<PantryItem> {
        let ing = self.repo.find_ingredient(&self.db, &input.ingredient_id).await?;
        let now = time::now();
        let id = new_id();
        let item = build_item(id.clone(), &ing, input, now.clone(), now)?;
        self.repo.insert_item(&self.db, &item).await?;
        self.repo.find_item(&self.db, &id).await
    }

    pub async fn update_item(&self, id: &str, input: PantryItemInput) -> Result<PantryItem> {
        let existing = self.repo.find_item(&self.db, id).await?;
        let ing = self.repo.find_ingredient(&self.db, &input.ingredient_id).await?;
        let item = build_item(existing.id, &ing, input, existing.created_at, time::now())?;
        self.repo.update_item(&self.db, &item).await?;
        self.repo.find_item(&self.db, id).await
    }

    pub async fn delete_item(&self, id: &str) -> Result<()> {
        self.repo.delete_item(&self.db, id).await
    }

    pub async fn adjust(&self, adjustments: Vec<StockAdjustment>) -> Result<()> {
        let mut tx: Transaction<'_> = self.db.begin().await?;
        self.adjust_in(&mut tx, &adjustments).await?;
        tx.commit().await?;
        Ok(())
    }

    /// 재고 수량/단계 변경. 호출자의 트랜잭션 안에서 실행된다.
    pub async fn adjust_in(&self, conn: &mut SqliteConnection, adjustments: &[StockAdjustment]) -> Result<()> {
        let now = time::now();
        for adj in adjustments {
            let item = self.repo.find_item(&mut *conn, &adj.pantry_item_id).await?;
            let (quantity, level) = match item.tracking {
                Tracking::Quantity => {
                    let q = adj
                        .quantity
                        .ok_or_else(|| Error::invalid(format!("{}: 수량이 필요합니다", item.ingredient_name)))?;
                    (Some(q.max(0.0)), None)
                }
                Tracking::Level => {
                    let l = adj
                        .level
                        .ok_or_else(|| Error::invalid(format!("{}: 단계가 필요합니다", item.ingredient_name)))?;
                    (None, Some(l))
                }
            };
            self.repo.set_item_amount(&mut *conn, &item.id, quantity, level, &now).await?;
        }
        Ok(())
    }

    pub async fn list_expiring(&self, until: NaiveDate) -> Result<Vec<PantryItem>> {
        self.repo.list_expiring(&self.db, until).await
    }

    // ---------- shopping ----------

    pub async fn list_shopping(&self) -> Result<Vec<ShoppingItem>> {
        self.repo.list_shopping(&self.db).await
    }

    pub async fn add_shopping(&self, input: ShoppingItemInput) -> Result<ShoppingItem> {
        let mut conn = self.db.acquire().await?;
        self.add_shopping_in(&mut conn, input).await
    }

    /// 같은 식재료가 이미 (미완료로) 목록에 있으면 새로 추가하지 않는다.
    pub async fn add_shopping_in(&self, conn: &mut SqliteConnection, input: ShoppingItemInput) -> Result<ShoppingItem> {
        let mut name = input.name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());
        if let Some(ing_id) = &input.ingredient_id {
            if let Some(existing) = self.repo.find_open_shopping_for_ingredient(&mut *conn, ing_id).await? {
                return Ok(existing);
            }
            let ing = self.repo.find_ingredient(&mut *conn, ing_id).await?;
            name.get_or_insert(ing.name);
        }
        let name = name.ok_or_else(|| Error::invalid("이름이 필요합니다"))?;
        let now = time::now();
        let item = ShoppingItem {
            id: new_id(),
            ingredient_id: input.ingredient_id,
            name,
            amount: clean(input.amount),
            done: input.done.unwrap_or(false),
            created_at: now.clone(),
            updated_at: now,
        };
        self.repo.insert_shopping(&mut *conn, &item).await?;
        Ok(item)
    }

    pub async fn update_shopping(&self, id: &str, input: ShoppingItemInput) -> Result<ShoppingItem> {
        let mut item = self.repo.find_shopping(&self.db, id).await?;
        if let Some(name) = clean(input.name) {
            item.name = name;
        }
        if input.amount.is_some() {
            item.amount = clean(input.amount);
        }
        if let Some(done) = input.done {
            item.done = done;
        }
        item.updated_at = time::now();
        self.repo.update_shopping(&self.db, &item).await?;
        Ok(item)
    }

    pub async fn delete_shopping(&self, id: &str) -> Result<()> {
        self.repo.delete_shopping(&self.db, id).await
    }

    pub async fn clear_done_shopping(&self) -> Result<u64> {
        self.repo.delete_done_shopping(&self.db).await
    }
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn build_ingredient(id: String, input: IngredientInput, created_at: String, updated_at: String) -> Result<Ingredient> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(Error::invalid("식재료 이름이 필요합니다"));
    }
    if input.low_threshold.is_some_and(|t| t < 0.0) {
        return Err(Error::invalid("부족 기준은 0 이상이어야 합니다"));
    }
    Ok(Ingredient {
        id,
        name,
        category: clean(input.category).unwrap_or_else(|| "기타".into()),
        default_unit: clean(input.default_unit),
        default_storage: input.default_storage.unwrap_or(Storage::Fridge),
        low_threshold: input.low_threshold,
        photo_id: clean(input.photo_id),
        note: clean(input.note),
        parent_id: clean(input.parent_id),
        created_at,
        updated_at,
    })
}

fn build_item(
    id: String,
    ing: &Ingredient,
    input: PantryItemInput,
    created_at: String,
    updated_at: String,
) -> Result<PantryItem> {
    let (quantity, unit, level) = match input.tracking {
        Tracking::Quantity => {
            let q = input.quantity.ok_or_else(|| Error::invalid("수량이 필요합니다"))?;
            if q < 0.0 {
                return Err(Error::invalid("수량은 0 이상이어야 합니다"));
            }
            let unit = clean(input.unit)
                .or_else(|| ing.default_unit.clone())
                .ok_or_else(|| Error::invalid("단위가 필요합니다"))?;
            (Some(q), Some(unit), None)
        }
        Tracking::Level => {
            let level = input.level.ok_or_else(|| Error::invalid("남은 정도가 필요합니다"))?;
            (None, None, Some(level))
        }
    };
    Ok(PantryItem {
        id,
        ingredient_id: ing.id.clone(),
        ingredient_name: ing.name.clone(),
        category: ing.category.clone(),
        tracking: input.tracking,
        quantity,
        unit,
        level,
        storage: input.storage.unwrap_or(ing.default_storage),
        purchased_at: input.purchased_at,
        expires_at: input.expires_at,
        photo_id: clean(input.photo_id),
        note: clean(input.note),
        created_at,
        updated_at,
    })
}

/// 식재료의 재고 상태 계산
pub fn with_stock(ingredient: Ingredient, items: &[&PantryItem]) -> IngredientWithStock {
    let live: Vec<&&PantryItem> = items.iter().filter(|it| !it.is_depleted()).collect();
    if live.is_empty() {
        return IngredientWithStock {
            ingredient,
            status: StockStatus::None,
            stock_summary: None,
            nearest_expiry: None,
            children: Vec::new(),
        };
    }

    let nearest_expiry = live.iter().filter_map(|it| it.expires_at).min();

    // 수량 재고는 기준 단위로 합산 (변환 불가한 단위는 따로 표시)
    let base_unit = ingredient.default_unit.clone().or_else(|| live.iter().find_map(|it| it.unit.clone()));
    let mut total: Option<f64> = None;
    let mut parts: Vec<String> = Vec::new();
    let mut best_level: Option<Level> = None;
    for it in &live {
        match it.tracking {
            Tracking::Quantity => {
                let (q, u) = (it.quantity.unwrap_or(0.0), it.unit.clone().unwrap_or_default());
                match base_unit.as_deref().and_then(|b| units::convert(q, &u, b)) {
                    Some(v) => *total.get_or_insert(0.0) += v,
                    None => parts.push(it.amount_label()),
                }
            }
            Tracking::Level => best_level = best_level.max(it.level),
        }
    }

    let mut summary = Vec::new();
    if let (Some(t), Some(u)) = (total, &base_unit) {
        summary.push(format!("{}{}", format_qty(t), u));
    }
    summary.extend(parts);
    if let Some(l) = best_level {
        summary.push(l.label().to_string());
    }

    let quantity_low = match (total, ingredient.low_threshold) {
        (Some(t), Some(th)) => t <= th,
        _ => false,
    };
    let has_quantity = total.is_some() || live.iter().any(|it| it.tracking == Tracking::Quantity);
    let status = match best_level {
        Some(l) if l >= Level::Medium => StockStatus::Ok,
        Some(_) if !has_quantity || quantity_low => StockStatus::Low,
        _ if quantity_low => StockStatus::Low,
        _ => StockStatus::Ok,
    };

    IngredientWithStock {
        ingredient,
        status,
        stock_summary: Some(summary.join(" + ")),
        nearest_expiry,
        children: Vec::new(),
    }
}

/// 재료별 재고를 계산한 뒤, 묶음 재료에 하위 재료 재고를 합친다.
/// 묶음 상태 = 자신과 하위 중 가장 좋은 상태, 요약 = "청양 고춧가루 보통, 고춧가루 조금"
pub fn aggregate(ingredients: Vec<Ingredient>, items: &[PantryItem]) -> Vec<IngredientWithStock> {
    let mut all: Vec<IngredientWithStock> = ingredients
        .into_iter()
        .map(|ing| {
            let mine: Vec<&PantryItem> = items.iter().filter(|it| it.ingredient_id == ing.id).collect();
            with_stock(ing, &mine)
        })
        .collect();

    let mut kids: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (idx, w) in all.iter().enumerate() {
        if let Some(p) = &w.ingredient.parent_id {
            kids.entry(p.clone()).or_default().push(idx);
        }
    }

    let mut updates = Vec::new();
    for (idx, w) in all.iter().enumerate() {
        let Some(children) = kids.get(&w.ingredient.id) else { continue };
        let mut status = w.status;
        let mut nearest = w.nearest_expiry;
        let mut parts: Vec<String> = w.stock_summary.iter().cloned().collect();
        for &c in children {
            let child = &all[c];
            status = status.max(child.status);
            nearest = match (nearest, child.nearest_expiry) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            if let Some(s) = &child.stock_summary {
                parts.push(format!("{} {s}", child.ingredient.name));
            }
        }
        let ids = children.iter().map(|&c| all[c].ingredient.id.clone()).collect();
        updates.push((idx, status, (!parts.is_empty()).then(|| parts.join(", ")), nearest, ids));
    }
    for (idx, status, summary, nearest, ids) in updates {
        let w = &mut all[idx];
        w.status = status;
        w.stock_summary = summary;
        w.nearest_expiry = nearest;
        w.children = ids;
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ing(id: &str, name: &str, parent: Option<&str>) -> Ingredient {
        Ingredient {
            id: id.into(),
            name: name.into(),
            category: "양념".into(),
            default_unit: None,
            default_storage: Storage::Room,
            low_threshold: None,
            photo_id: None,
            note: None,
            parent_id: parent.map(Into::into),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn level_item(ing: &Ingredient, level: Level) -> PantryItem {
        PantryItem {
            id: format!("item-{}", ing.id),
            ingredient_id: ing.id.clone(),
            ingredient_name: ing.name.clone(),
            category: ing.category.clone(),
            tracking: Tracking::Level,
            quantity: None,
            unit: None,
            level: Some(level),
            storage: Storage::Room,
            purchased_at: None,
            expires_at: None,
            photo_id: None,
            note: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn group_takes_best_child_status() {
        let group = ing("g", "고춧가루", None);
        let plain = ing("a", "고운 고춧가루", Some("g"));
        let hot = ing("b", "청양 고춧가루", Some("g"));
        let items = vec![level_item(&plain, Level::Low), level_item(&hot, Level::Medium)];
        let out = aggregate(vec![group, plain, hot], &items);

        let g = out.iter().find(|i| i.ingredient.id == "g").unwrap();
        assert_eq!(g.status, StockStatus::Ok);
        assert_eq!(g.children, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(g.stock_summary.as_deref(), Some("고운 고춧가루 조금, 청양 고춧가루 보통"));

        let a = out.iter().find(|i| i.ingredient.id == "a").unwrap();
        assert_eq!(a.status, StockStatus::Low);
        assert!(a.children.is_empty());
    }

    #[test]
    fn empty_group_has_no_stock() {
        let group = ing("g", "간장", None);
        let child = ing("c", "국간장", Some("g"));
        let out = aggregate(vec![group, child], &[]);
        assert_eq!(out[0].status, StockStatus::None);
        assert_eq!(out[0].stock_summary, None);
    }
}

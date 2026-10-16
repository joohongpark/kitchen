use chrono::NaiveDate;
use common::error::{is_foreign_key_violation, is_unique_violation};
use common::kinds::Level;
use common::{Error, Queryer, Result};

use crate::model::{Ingredient, PantryItem, ShoppingItem};

#[derive(Debug, Clone, Default)]
pub struct PantryRepository;

const PANTRY_SELECT: &str = "SELECT p.id, p.ingredient_id, i.name AS ingredient_name, i.category, p.tracking, \
     p.quantity, p.unit, p.level, p.storage, p.purchased_at, p.expires_at, p.photo_id, p.note, \
     p.created_at, p.updated_at \
     FROM pantry_items p JOIN ingredients i ON i.id = p.ingredient_id";

impl PantryRepository {
    // ---------- ingredients ----------

    pub async fn insert_ingredient<'c>(&self, db: impl Queryer<'c>, ing: &Ingredient) -> Result<()> {
        sqlx::query(
            "INSERT INTO ingredients (id, name, category, default_unit, default_storage, low_threshold, \
             photo_id, note, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&ing.id)
        .bind(&ing.name)
        .bind(&ing.category)
        .bind(&ing.default_unit)
        .bind(ing.default_storage)
        .bind(ing.low_threshold)
        .bind(&ing.photo_id)
        .bind(&ing.note)
        .bind(&ing.parent_id)
        .bind(&ing.created_at)
        .bind(&ing.updated_at)
        .execute(db)
        .await
        .map_err(|e| map_ingredient_err(e, &ing.name))?;
        Ok(())
    }

    pub async fn update_ingredient<'c>(&self, db: impl Queryer<'c>, ing: &Ingredient) -> Result<()> {
        let res = sqlx::query(
            "UPDATE ingredients SET name = ?, category = ?, default_unit = ?, default_storage = ?, \
             low_threshold = ?, photo_id = ?, note = ?, parent_id = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&ing.name)
        .bind(&ing.category)
        .bind(&ing.default_unit)
        .bind(ing.default_storage)
        .bind(ing.low_threshold)
        .bind(&ing.photo_id)
        .bind(&ing.note)
        .bind(&ing.parent_id)
        .bind(&ing.updated_at)
        .bind(&ing.id)
        .execute(db)
        .await
        .map_err(|e| map_ingredient_err(e, &ing.name))?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("식재료"));
        }
        Ok(())
    }

    pub async fn delete_ingredient<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<()> {
        let res = sqlx::query("DELETE FROM ingredients WHERE id = ?").bind(id).execute(db).await.map_err(|e| {
            if is_foreign_key_violation(&e) {
                Error::InUse("레시피에서 사용 중인 식재료입니다".into())
            } else {
                e.into()
            }
        })?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("식재료"));
        }
        Ok(())
    }

    pub async fn find_ingredient<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<Ingredient> {
        sqlx::query_as("SELECT * FROM ingredients WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::NotFound("식재료"))
    }

    pub async fn find_ingredient_by_name<'c>(&self, db: impl Queryer<'c>, name: &str) -> Result<Option<Ingredient>> {
        Ok(sqlx::query_as("SELECT * FROM ingredients WHERE name = ?").bind(name).fetch_optional(db).await?)
    }

    pub async fn list_ingredients<'c>(&self, db: impl Queryer<'c>) -> Result<Vec<Ingredient>> {
        Ok(sqlx::query_as("SELECT * FROM ingredients ORDER BY category, name").fetch_all(db).await?)
    }

    pub async fn count_children<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE parent_id = ?").bind(id).fetch_one(db).await?)
    }

    // ---------- pantry items ----------

    pub async fn insert_item<'c>(&self, db: impl Queryer<'c>, item: &PantryItem) -> Result<()> {
        sqlx::query(
            "INSERT INTO pantry_items (id, ingredient_id, tracking, quantity, unit, level, storage, \
             purchased_at, expires_at, photo_id, note, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&item.id)
        .bind(&item.ingredient_id)
        .bind(item.tracking)
        .bind(item.quantity)
        .bind(&item.unit)
        .bind(item.level)
        .bind(item.storage)
        .bind(item.purchased_at)
        .bind(item.expires_at)
        .bind(&item.photo_id)
        .bind(&item.note)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .execute(db)
        .await
        .map_err(map_item_err)?;
        Ok(())
    }

    pub async fn update_item<'c>(&self, db: impl Queryer<'c>, item: &PantryItem) -> Result<()> {
        let res = sqlx::query(
            "UPDATE pantry_items SET ingredient_id = ?, tracking = ?, quantity = ?, unit = ?, level = ?, \
             storage = ?, purchased_at = ?, expires_at = ?, photo_id = ?, note = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&item.ingredient_id)
        .bind(item.tracking)
        .bind(item.quantity)
        .bind(&item.unit)
        .bind(item.level)
        .bind(item.storage)
        .bind(item.purchased_at)
        .bind(item.expires_at)
        .bind(&item.photo_id)
        .bind(&item.note)
        .bind(&item.updated_at)
        .bind(&item.id)
        .execute(db)
        .await
        .map_err(map_item_err)?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("재고"));
        }
        Ok(())
    }

    pub async fn set_item_amount<'c>(
        &self,
        db: impl Queryer<'c>,
        id: &str,
        quantity: Option<f64>,
        level: Option<Level>,
        updated_at: &str,
    ) -> Result<()> {
        let res = sqlx::query("UPDATE pantry_items SET quantity = ?, level = ?, updated_at = ? WHERE id = ?")
            .bind(quantity)
            .bind(level)
            .bind(updated_at)
            .bind(id)
            .execute(db)
            .await?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("재고"));
        }
        Ok(())
    }

    pub async fn delete_item<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<()> {
        let res = sqlx::query("DELETE FROM pantry_items WHERE id = ?").bind(id).execute(db).await?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("재고"));
        }
        Ok(())
    }

    pub async fn find_item<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<PantryItem> {
        sqlx::query_as(&format!("{PANTRY_SELECT} WHERE p.id = ?"))
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::NotFound("재고"))
    }

    /// 전체 재고. 유통기한 빠른 순(없으면 뒤로)
    pub async fn list_items<'c>(&self, db: impl Queryer<'c>) -> Result<Vec<PantryItem>> {
        Ok(sqlx::query_as(&format!("{PANTRY_SELECT} ORDER BY p.expires_at IS NULL, p.expires_at, i.name"))
            .fetch_all(db)
            .await?)
    }

    pub async fn list_items_for_ingredient<'c>(
        &self,
        db: impl Queryer<'c>,
        ingredient_id: &str,
    ) -> Result<Vec<PantryItem>> {
        Ok(sqlx::query_as(&format!(
            "{PANTRY_SELECT} WHERE p.ingredient_id = ? ORDER BY p.expires_at IS NULL, p.expires_at, p.created_at"
        ))
        .bind(ingredient_id)
        .fetch_all(db)
        .await?)
    }

    /// `until` 이전에 유통기한이 끝나는 (다 쓰지 않은) 재고
    pub async fn list_expiring<'c>(&self, db: impl Queryer<'c>, until: NaiveDate) -> Result<Vec<PantryItem>> {
        Ok(sqlx::query_as(&format!(
            "{PANTRY_SELECT} WHERE p.expires_at IS NOT NULL AND p.expires_at <= ? \
             AND NOT ((p.tracking = 'quantity' AND p.quantity <= 0) OR (p.tracking = 'level' AND p.level = 'out')) \
             ORDER BY p.expires_at"
        ))
        .bind(until)
        .fetch_all(db)
        .await?)
    }

    // ---------- shopping ----------

    pub async fn insert_shopping<'c>(&self, db: impl Queryer<'c>, item: &ShoppingItem) -> Result<()> {
        sqlx::query(
            "INSERT INTO shopping_items (id, ingredient_id, name, amount, done, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&item.id)
        .bind(&item.ingredient_id)
        .bind(&item.name)
        .bind(&item.amount)
        .bind(item.done)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn update_shopping<'c>(&self, db: impl Queryer<'c>, item: &ShoppingItem) -> Result<()> {
        let res = sqlx::query("UPDATE shopping_items SET name = ?, amount = ?, done = ?, updated_at = ? WHERE id = ?")
            .bind(&item.name)
            .bind(&item.amount)
            .bind(item.done)
            .bind(&item.updated_at)
            .bind(&item.id)
            .execute(db)
            .await?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("장보기 항목"));
        }
        Ok(())
    }

    pub async fn find_shopping<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<ShoppingItem> {
        sqlx::query_as("SELECT * FROM shopping_items WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::NotFound("장보기 항목"))
    }

    pub async fn find_open_shopping_for_ingredient<'c>(
        &self,
        db: impl Queryer<'c>,
        ingredient_id: &str,
    ) -> Result<Option<ShoppingItem>> {
        Ok(sqlx::query_as("SELECT * FROM shopping_items WHERE ingredient_id = ? AND done = 0")
            .bind(ingredient_id)
            .fetch_optional(db)
            .await?)
    }

    pub async fn list_shopping<'c>(&self, db: impl Queryer<'c>) -> Result<Vec<ShoppingItem>> {
        Ok(sqlx::query_as("SELECT * FROM shopping_items ORDER BY done, created_at").fetch_all(db).await?)
    }

    pub async fn delete_shopping<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<()> {
        let res = sqlx::query("DELETE FROM shopping_items WHERE id = ?").bind(id).execute(db).await?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("장보기 항목"));
        }
        Ok(())
    }

    pub async fn delete_done_shopping<'c>(&self, db: impl Queryer<'c>) -> Result<u64> {
        Ok(sqlx::query("DELETE FROM shopping_items WHERE done = 1").execute(db).await?.rows_affected())
    }
}

fn map_ingredient_err(e: sqlx::Error, name: &str) -> Error {
    if is_unique_violation(&e) {
        Error::AlreadyExists(format!("식재료 '{name}'"))
    } else if is_foreign_key_violation(&e) {
        Error::invalid("사진 또는 묶음 재료가 존재하지 않습니다")
    } else {
        e.into()
    }
}

fn map_item_err(e: sqlx::Error) -> Error {
    if is_foreign_key_violation(&e) {
        Error::invalid("식재료 또는 사진이 존재하지 않습니다")
    } else {
        e.into()
    }
}

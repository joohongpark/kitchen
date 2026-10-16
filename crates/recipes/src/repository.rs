use common::error::{is_foreign_key_violation, is_unique_violation};
use common::{Error, Queryer, Result};

use crate::model::*;

#[derive(Debug, Clone, Default)]
pub struct RecipesRepository;

const INGREDIENT_SELECT: &str = "SELECT ri.recipe_id, ri.ingredient_id, i.name AS ingredient_name, i.category, \
     ri.position, ri.quantity, ri.unit, ri.amount_text, ri.optional, ri.note \
     FROM recipe_ingredients ri JOIN ingredients i ON i.id = ri.ingredient_id";

impl RecipesRepository {
    pub async fn insert_recipe<'c>(&self, db: impl Queryer<'c>, r: &RecipeRow) -> Result<()> {
        sqlx::query(
            "INSERT INTO recipes (id, title, description, servings, cook_minutes, tags, photo_id, source, \
             created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&r.id)
        .bind(&r.title)
        .bind(&r.description)
        .bind(r.servings)
        .bind(r.cook_minutes)
        .bind(&r.tags)
        .bind(&r.photo_id)
        .bind(&r.source)
        .bind(&r.created_at)
        .bind(&r.updated_at)
        .execute(db)
        .await
        .map_err(|e| map_recipe_err(e, &r.title))?;
        Ok(())
    }

    pub async fn update_recipe<'c>(&self, db: impl Queryer<'c>, r: &RecipeRow) -> Result<()> {
        let res = sqlx::query(
            "UPDATE recipes SET title = ?, description = ?, servings = ?, cook_minutes = ?, tags = ?, \
             photo_id = ?, source = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&r.title)
        .bind(&r.description)
        .bind(r.servings)
        .bind(r.cook_minutes)
        .bind(&r.tags)
        .bind(&r.photo_id)
        .bind(&r.source)
        .bind(&r.updated_at)
        .bind(&r.id)
        .execute(db)
        .await
        .map_err(|e| map_recipe_err(e, &r.title))?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("레시피"));
        }
        Ok(())
    }

    pub async fn delete_recipe<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<()> {
        let res = sqlx::query("DELETE FROM recipes WHERE id = ?").bind(id).execute(db).await?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound("레시피"));
        }
        Ok(())
    }

    pub async fn find_recipe<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<RecipeRow> {
        sqlx::query_as("SELECT * FROM recipes WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::NotFound("레시피"))
    }

    pub async fn list_recipes<'c>(&self, db: impl Queryer<'c>) -> Result<Vec<RecipeRow>> {
        Ok(sqlx::query_as("SELECT * FROM recipes ORDER BY title").fetch_all(db).await?)
    }

    pub async fn delete_ingredients<'c>(&self, db: impl Queryer<'c>, recipe_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM recipe_ingredients WHERE recipe_id = ?").bind(recipe_id).execute(db).await?;
        Ok(())
    }

    pub async fn insert_ingredient<'c>(&self, db: impl Queryer<'c>, ri: &RecipeIngredient) -> Result<()> {
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, position, quantity, unit, amount_text, \
             optional, note) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&ri.recipe_id)
        .bind(&ri.ingredient_id)
        .bind(ri.position)
        .bind(ri.quantity)
        .bind(&ri.unit)
        .bind(&ri.amount_text)
        .bind(ri.optional)
        .bind(&ri.note)
        .execute(db)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                Error::invalid(format!("재료 '{}'가 중복되었습니다", ri.ingredient_name))
            } else if is_foreign_key_violation(&e) {
                Error::invalid("존재하지 않는 식재료입니다")
            } else {
                e.into()
            }
        })?;
        Ok(())
    }

    pub async fn list_ingredients<'c>(&self, db: impl Queryer<'c>, recipe_id: &str) -> Result<Vec<RecipeIngredient>> {
        Ok(sqlx::query_as(&format!("{INGREDIENT_SELECT} WHERE ri.recipe_id = ? ORDER BY ri.position"))
            .bind(recipe_id)
            .fetch_all(db)
            .await?)
    }

    pub async fn list_all_ingredients<'c>(&self, db: impl Queryer<'c>) -> Result<Vec<RecipeIngredient>> {
        Ok(sqlx::query_as(&format!("{INGREDIENT_SELECT} ORDER BY ri.recipe_id, ri.position")).fetch_all(db).await?)
    }

    pub async fn delete_steps<'c>(&self, db: impl Queryer<'c>, recipe_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM recipe_steps WHERE recipe_id = ?").bind(recipe_id).execute(db).await?;
        Ok(())
    }

    pub async fn insert_step<'c>(&self, db: impl Queryer<'c>, recipe_id: &str, step: &RecipeStep) -> Result<()> {
        sqlx::query("INSERT INTO recipe_steps (recipe_id, position, body, photo_id) VALUES (?, ?, ?, ?)")
            .bind(recipe_id)
            .bind(step.position)
            .bind(&step.body)
            .bind(&step.photo_id)
            .execute(db)
            .await
            .map_err(|e| {
                if is_foreign_key_violation(&e) {
                    Error::invalid("사진이 존재하지 않습니다")
                } else {
                    e.into()
                }
            })?;
        Ok(())
    }

    pub async fn list_steps<'c>(&self, db: impl Queryer<'c>, recipe_id: &str) -> Result<Vec<RecipeStep>> {
        Ok(sqlx::query_as("SELECT position, body, photo_id FROM recipe_steps WHERE recipe_id = ? ORDER BY position")
            .bind(recipe_id)
            .fetch_all(db)
            .await?)
    }

    pub async fn insert_cook_log<'c>(&self, db: impl Queryer<'c>, log: &CookLog) -> Result<()> {
        sqlx::query("INSERT INTO cook_logs (id, recipe_id, cooked_at, note) VALUES (?, ?, ?, ?)")
            .bind(&log.id)
            .bind(&log.recipe_id)
            .bind(&log.cooked_at)
            .bind(&log.note)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn cook_stats<'c>(&self, db: impl Queryer<'c>) -> Result<Vec<CookStats>> {
        Ok(sqlx::query_as(
            "SELECT recipe_id, COUNT(*) AS cook_count, MAX(cooked_at) AS last_cooked_at FROM cook_logs GROUP BY recipe_id",
        )
        .fetch_all(db)
        .await?)
    }

    pub async fn list_cook_logs<'c>(&self, db: impl Queryer<'c>, recipe_id: &str) -> Result<Vec<CookLog>> {
        Ok(sqlx::query_as("SELECT * FROM cook_logs WHERE recipe_id = ? ORDER BY cooked_at DESC")
            .bind(recipe_id)
            .fetch_all(db)
            .await?)
    }
}

fn map_recipe_err(e: sqlx::Error, title: &str) -> Error {
    if is_unique_violation(&e) {
        Error::AlreadyExists(format!("레시피 '{title}'"))
    } else if is_foreign_key_violation(&e) {
        Error::invalid("사진이 존재하지 않습니다")
    } else {
        e.into()
    }
}

-- 식재료 묶음: 고춧가루(묶음) ← 고춧가루, 청양 고춧가루. 한 단계만 허용 (서비스에서 검증)
ALTER TABLE ingredients ADD COLUMN parent_id TEXT REFERENCES ingredients(id) ON DELETE SET NULL;
CREATE INDEX ingredients_parent ON ingredients(parent_id);

-- 레시피 재료별 대체 재료: 차슈의 삼겹살 → 목살, 앞다리살
CREATE TABLE recipe_ingredient_alternatives (
    recipe_id          TEXT NOT NULL,
    ingredient_id      TEXT NOT NULL,
    alt_ingredient_id  TEXT NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
    position           INTEGER NOT NULL,
    note               TEXT,
    PRIMARY KEY (recipe_id, ingredient_id, alt_ingredient_id),
    FOREIGN KEY (recipe_id, ingredient_id) REFERENCES recipe_ingredients(recipe_id, ingredient_id) ON DELETE CASCADE
);
CREATE INDEX recipe_alternatives_alt ON recipe_ingredient_alternatives(alt_ingredient_id);

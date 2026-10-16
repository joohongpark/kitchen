-- 사진 메타데이터 (파일은 data/photos/{id}.{ext})
CREATE TABLE photos (
    id          TEXT PRIMARY KEY,
    file_name   TEXT NOT NULL,
    mime        TEXT NOT NULL,
    width       INTEGER NOT NULL,
    height      INTEGER NOT NULL,
    size_bytes  INTEGER NOT NULL,
    created_at  TEXT NOT NULL
);

-- 식재료 마스터
CREATE TABLE ingredients (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL UNIQUE,
    category         TEXT NOT NULL DEFAULT '기타',
    default_unit     TEXT,
    default_storage  TEXT NOT NULL DEFAULT 'fridge' CHECK (default_storage IN ('fridge', 'freezer', 'room')),
    low_threshold    REAL,
    photo_id         TEXT REFERENCES photos(id) ON DELETE SET NULL,
    note             TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

-- 재고 (구매분 단위). tracking 에 따라 quantity+unit 또는 level 사용
CREATE TABLE pantry_items (
    id             TEXT PRIMARY KEY,
    ingredient_id  TEXT NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
    tracking       TEXT NOT NULL CHECK (tracking IN ('quantity', 'level')),
    quantity       REAL,
    unit           TEXT,
    level          TEXT CHECK (level IN ('plenty', 'medium', 'low', 'out')),
    storage        TEXT NOT NULL CHECK (storage IN ('fridge', 'freezer', 'room')),
    purchased_at   TEXT,
    expires_at     TEXT,
    photo_id       TEXT REFERENCES photos(id) ON DELETE SET NULL,
    note           TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    CHECK (
        (tracking = 'quantity' AND quantity IS NOT NULL AND unit IS NOT NULL)
        OR (tracking = 'level' AND level IS NOT NULL)
    )
);
CREATE INDEX pantry_items_ingredient ON pantry_items(ingredient_id);
CREATE INDEX pantry_items_expires ON pantry_items(expires_at);

CREATE TABLE recipes (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL UNIQUE,
    description   TEXT,
    servings      INTEGER,
    cook_minutes  INTEGER,
    tags          TEXT NOT NULL DEFAULT '[]', -- JSON 배열
    photo_id      TEXT REFERENCES photos(id) ON DELETE SET NULL,
    source        TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE recipe_ingredients (
    recipe_id      TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    ingredient_id  TEXT NOT NULL REFERENCES ingredients(id) ON DELETE RESTRICT,
    position       INTEGER NOT NULL,
    quantity       REAL,
    unit           TEXT,
    amount_text    TEXT,
    optional       INTEGER NOT NULL DEFAULT 0,
    note           TEXT,
    PRIMARY KEY (recipe_id, ingredient_id)
);
CREATE INDEX recipe_ingredients_ingredient ON recipe_ingredients(ingredient_id);

CREATE TABLE recipe_steps (
    recipe_id  TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    position   INTEGER NOT NULL,
    body       TEXT NOT NULL,
    photo_id   TEXT REFERENCES photos(id) ON DELETE SET NULL,
    PRIMARY KEY (recipe_id, position)
);

CREATE TABLE cook_logs (
    id         TEXT PRIMARY KEY,
    recipe_id  TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    cooked_at  TEXT NOT NULL,
    note       TEXT
);
CREATE INDEX cook_logs_recipe ON cook_logs(recipe_id, cooked_at);

CREATE TABLE shopping_items (
    id             TEXT PRIMARY KEY,
    ingredient_id  TEXT REFERENCES ingredients(id) ON DELETE SET NULL,
    name           TEXT NOT NULL,
    amount         TEXT,
    done           INTEGER NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE TABLE jobs (
    id            TEXT PRIMARY KEY,
    kind          TEXT NOT NULL,
    payload       TEXT NOT NULL DEFAULT '{}',
    status        TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'done', 'failed')),
    run_at        TEXT NOT NULL,
    attempts      INTEGER NOT NULL DEFAULT 0,
    max_attempts  INTEGER NOT NULL DEFAULT 5,
    last_error    TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);
CREATE INDEX jobs_pending ON jobs(status, run_at);

CREATE TABLE kv (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE sessions (
    token_hash  TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL,
    expires_at  TEXT NOT NULL
);

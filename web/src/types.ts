export type Storage = "fridge" | "freezer" | "room";
export type Tracking = "quantity" | "level";
export type Level = "plenty" | "medium" | "low" | "out";
export type StockStatus = "none" | "low" | "ok";

export const STORAGE_LABEL: Record<Storage, string> = { fridge: "냉장", freezer: "냉동", room: "실온" };
export const LEVEL_LABEL: Record<Level, string> = { plenty: "넉넉", medium: "보통", low: "조금", out: "없음" };
export const LEVELS: Level[] = ["plenty", "medium", "low", "out"];
export const STORAGES: Storage[] = ["fridge", "freezer", "room"];
export const CATEGORIES = ["채소", "과일", "육류", "해산물", "건어물", "두부·콩", "달걀·유제품", "장류", "양념", "곡물·면", "가공식품", "기타"];
export const UNITS = ["g", "kg", "ml", "l", "큰술", "작은술", "컵", "개", "모", "단", "대", "봉", "포기", "마리", "팩", "캔"];

export interface Ingredient {
  id: string;
  name: string;
  category: string;
  default_unit: string | null;
  default_storage: Storage;
  low_threshold: number | null;
  photo_id: string | null;
  note: string | null;
  created_at: string;
  updated_at: string;
}

export interface IngredientWithStock extends Ingredient {
  status: StockStatus;
  stock_summary: string | null;
  nearest_expiry: string | null;
}

export interface IngredientInput {
  name: string;
  category?: string | null;
  default_unit?: string | null;
  default_storage?: Storage | null;
  low_threshold?: number | null;
  photo_id?: string | null;
  note?: string | null;
}

export interface PantryItem {
  id: string;
  ingredient_id: string;
  ingredient_name: string;
  category: string;
  tracking: Tracking;
  quantity: number | null;
  unit: string | null;
  level: Level | null;
  storage: Storage;
  purchased_at: string | null;
  expires_at: string | null;
  photo_id: string | null;
  note: string | null;
  created_at: string;
  updated_at: string;
}

export interface PantryItemInput {
  ingredient_id: string;
  tracking: Tracking;
  quantity?: number | null;
  unit?: string | null;
  level?: Level | null;
  storage?: Storage | null;
  purchased_at?: string | null;
  expires_at?: string | null;
  photo_id?: string | null;
  note?: string | null;
}

export interface StockAdjustment {
  pantry_item_id: string;
  quantity?: number | null;
  level?: Level | null;
}

export interface Recipe {
  id: string;
  title: string;
  description: string | null;
  servings: number | null;
  cook_minutes: number | null;
  tags: string[];
  photo_id: string | null;
  source: string | null;
  created_at: string;
  updated_at: string;
}

export interface MissingIngredient {
  ingredient_id: string;
  name: string;
  amount_label: string;
}

export interface RecipeSummary extends Recipe {
  required_count: number;
  missing: MissingIngredient[];
  cook_count: number;
  last_cooked_at: string | null;
}

export interface Recommendation extends RecipeSummary {
  matched: string[];
  have_count: number;
  coverage: number;
}

export interface IngredientLine {
  recipe_id: string;
  ingredient_id: string;
  ingredient_name: string;
  category: string;
  position: number;
  quantity: number | null;
  unit: string | null;
  amount_text: string | null;
  optional: boolean;
  note: string | null;
  amount_label: string;
  status: StockStatus;
  stock_summary: string | null;
}

export interface RecipeStep {
  position: number;
  body: string;
  photo_id: string | null;
}

export interface RecipeDetail extends Recipe {
  ingredients: IngredientLine[];
  steps: RecipeStep[];
  cook_count: number;
  last_cooked_at: string | null;
}

export interface RecipeIngredientInput {
  ingredient_id?: string | null;
  name?: string | null;
  category?: string | null;
  quantity?: number | null;
  unit?: string | null;
  amount_text?: string | null;
  optional?: boolean;
  note?: string | null;
}

export interface RecipeInput {
  title: string;
  description?: string | null;
  servings?: number | null;
  cook_minutes?: number | null;
  tags: string[];
  photo_id?: string | null;
  source?: string | null;
  ingredients: RecipeIngredientInput[];
  steps: { body: string; photo_id?: string | null }[];
}

export interface CookPlanLot {
  pantry_item_id: string;
  tracking: Tracking;
  storage: Storage;
  expires_at: string | null;
  unit: string | null;
  current_quantity: number | null;
  current_level: Level | null;
  suggested_quantity: number | null;
  suggested_level: Level | null;
}

export interface CookPlanLine {
  ingredient_id: string;
  ingredient_name: string;
  amount_label: string;
  optional: boolean;
  missing: boolean;
  lots: CookPlanLot[];
}

export interface CookPlan {
  recipe_id: string;
  scale: number;
  lines: CookPlanLine[];
}

export interface CookResult {
  log: { id: string; recipe_id: string; cooked_at: string; note: string | null };
  added_to_shopping: string[];
}

export interface ShoppingItem {
  id: string;
  ingredient_id: string | null;
  name: string;
  amount: string | null;
  done: boolean;
  created_at: string;
  updated_at: string;
}

export interface Photo {
  id: string;
  width: number;
  height: number;
}

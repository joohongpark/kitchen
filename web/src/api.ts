import type * as T from "./types";

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

async function request<R>(method: string, path: string, body?: unknown): Promise<R> {
  const init: RequestInit = { method, credentials: "same-origin", headers: {} };
  if (body instanceof FormData) {
    init.body = body;
  } else if (body !== undefined) {
    init.body = JSON.stringify(body);
    (init.headers as Record<string, string>)["Content-Type"] = "application/json";
  }
  const res = await fetch(`/api${path}`, init);
  if (!res.ok) {
    let message = `요청 실패 (${res.status})`;
    try {
      message = (await res.json()).error ?? message;
    } catch {
      /* JSON 이 아닌 응답 */
    }
    if (res.status === 401) window.dispatchEvent(new Event("kitchen:unauthorized"));
    throw new ApiError(res.status, message);
  }
  if (res.status === 204) return undefined as R;
  return res.json() as Promise<R>;
}

const get = <R>(p: string) => request<R>("GET", p);
const post = <R>(p: string, b?: unknown) => request<R>("POST", p, b);
const put = <R>(p: string, b?: unknown) => request<R>("PUT", p, b);
const del = (p: string) => request<void>("DELETE", p);

export const photoUrl = (id: string) => `/api/photos/${id}`;

export const api = {
  me: () => get<{ auth_required: boolean; authenticated: boolean }>("/auth/me"),
  login: (password: string) => post<void>("/auth/login", { password }),
  logout: () => post<void>("/auth/logout"),
  info: () => get<{ notifications: boolean; notify_hour: number; expiry_warn_days: number }>("/info"),
  notifyTest: () => post<{ job_id: string }>("/notifications/test"),

  ingredients: () => get<T.IngredientWithStock[]>("/ingredients"),
  ingredient: (id: string) => get<T.IngredientWithStock>(`/ingredients/${id}`),
  createIngredient: (b: T.IngredientInput) => post<T.Ingredient>("/ingredients", b),
  updateIngredient: (id: string, b: T.IngredientInput) => put<T.Ingredient>(`/ingredients/${id}`, b),
  deleteIngredient: (id: string) => del(`/ingredients/${id}`),

  pantry: () => get<T.PantryItem[]>("/pantry"),
  createPantryItem: (b: T.PantryItemInput) => post<T.PantryItem>("/pantry", b),
  updatePantryItem: (id: string, b: T.PantryItemInput) => put<T.PantryItem>(`/pantry/${id}`, b),
  deletePantryItem: (id: string) => del(`/pantry/${id}`),
  adjustPantry: (b: T.StockAdjustment[]) => post<void>("/pantry/adjust", b),

  recipes: (q?: string) => get<T.RecipeSummary[]>(`/recipes${q ? `?q=${encodeURIComponent(q)}` : ""}`),
  recipe: (id: string) => get<T.RecipeDetail>(`/recipes/${id}`),
  createRecipe: (b: T.RecipeInput) => post<T.RecipeDetail>("/recipes", b),
  updateRecipe: (id: string, b: T.RecipeInput) => put<T.RecipeDetail>(`/recipes/${id}`, b),
  deleteRecipe: (id: string) => del(`/recipes/${id}`),
  setRecipePhoto: (id: string, photo_id: string | null) => put<T.RecipeDetail>(`/recipes/${id}/photo`, { photo_id }),
  recommend: (ingredient_ids: string[]) => post<T.Recommendation[]>("/recipes/recommend", { ingredient_ids }),
  cookPlan: (id: string, servings?: number) =>
    get<T.CookPlan>(`/recipes/${id}/cook-plan${servings ? `?servings=${servings}` : ""}`),
  cook: (id: string, b: { adjustments: T.StockAdjustment[]; note?: string; add_missing_to_shopping: boolean }) =>
    post<T.CookResult>(`/recipes/${id}/cook`, b),

  shopping: () => get<T.ShoppingItem[]>("/shopping"),
  addShopping: (b: { ingredient_id?: string; name?: string; amount?: string }) => post<T.ShoppingItem>("/shopping", b),
  updateShopping: (id: string, b: { name?: string; amount?: string; done?: boolean }) =>
    put<T.ShoppingItem>(`/shopping/${id}`, b),
  deleteShopping: (id: string) => del(`/shopping/${id}`),
  clearDoneShopping: () => post<{ removed: number }>("/shopping/clear-done"),

  uploadPhoto: (file: File) => {
    const fd = new FormData();
    fd.append("file", file);
    return post<T.Photo>("/photos", fd);
  },
};

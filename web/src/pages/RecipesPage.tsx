import { useDeferredValue, useState, type ReactNode } from "react";
import { Link, useNavigate } from "react-router";
import { useQuery } from "@tanstack/react-query";
import { api } from "../api";
import { Empty, ErrorText, Fab, PageHeader, Thumb } from "../components/ui";
import type { RecipeSummary } from "../types";
import { relativeDate } from "../util";

export function RecipesPage() {
  const [q, setQ] = useState("");
  const query = useDeferredValue(q.trim());
  const { data = [], isLoading, error } = useQuery({ queryKey: ["recipes", query], queryFn: () => api.recipes(query) });
  const navigate = useNavigate();

  return (
    <div>
      <PageHeader title="레시피" />
      <input className="field mb-4" placeholder="요리 이름이나 재료로 검색" value={q} onChange={(e) => setQ(e.target.value)} />
      <ErrorText error={error} />
      {isLoading ? (
        <p className="text-muted">불러오는 중…</p>
      ) : data.length === 0 ? (
        <Empty>{query ? "검색 결과가 없어요." : "레시피를 추가해 보세요."}</Empty>
      ) : (
        <ul className="flex flex-col gap-3">
          {data.map((r) => (
            <li key={r.id}>
              <RecipeCard recipe={r} />
            </li>
          ))}
        </ul>
      )}
      <Fab label="레시피" onClick={() => navigate("/recipes/new")} />
    </div>
  );
}

export function RecipeCard({ recipe, extra }: { recipe: RecipeSummary; extra?: ReactNode }) {
  const ready = recipe.missing.length === 0;
  return (
    <Link to={`/recipes/${recipe.id}`} className="card flex gap-3 overflow-hidden p-2.5 transition hover:border-accent/40">
      <Thumb id={recipe.photo_id} className="size-20 shrink-0 rounded-xl" />
      <div className="min-w-0 flex-1 py-0.5">
        <div className="flex items-start justify-between gap-2">
          <h3 className="truncate font-bold">{recipe.title}</h3>
          {recipe.cook_minutes && <span className="shrink-0 text-xs text-muted">{recipe.cook_minutes}분</span>}
        </div>
        <p className={ready ? "text-sm font-semibold text-ok" : "truncate text-sm text-muted"}>
          {ready ? "✓ 바로 만들 수 있어요" : `없음 ${recipe.missing.length}: ${recipe.missing.map((m) => m.name).join(", ")}`}
        </p>
        {recipe.substitutions.length > 0 && (
          <p className="truncate text-xs text-muted">
            대체: {recipe.substitutions.map((s) => `${s.original}→${s.substitute}`).join(", ")}
          </p>
        )}
        <div className="mt-1 flex flex-wrap items-center gap-1">
          {recipe.tags.map((t) => (
            <span key={t} className="rounded-full bg-accent-soft px-2 py-0.5 text-xs text-accent">
              {t}
            </span>
          ))}
          {recipe.last_cooked_at && <span className="text-xs text-muted">· {relativeDate(recipe.last_cooked_at)} 요리</span>}
        </div>
        {extra}
      </div>
    </Link>
  );
}

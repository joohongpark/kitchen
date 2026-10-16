import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../api";
import { Empty, ErrorText, PageHeader, StatusDot } from "../components/ui";
import type { IngredientWithStock } from "../types";
import { cx } from "../util";
import { RecipeCard } from "./RecipesPage";

export function RecommendPage() {
  const { data: ingredients = [] } = useQuery({ queryKey: ["ingredients"], queryFn: api.ingredients });
  const [selected, setSelected] = useState<string[]>([]);
  const [onlyStock, setOnlyStock] = useState(true);
  const [filter, setFilter] = useState("");

  const sortedIds = [...selected].sort();
  const { data: recs = [], isFetching, error } = useQuery({
    queryKey: ["recommend", sortedIds],
    queryFn: () => api.recommend(sortedIds),
  });

  const grouped = useMemo(() => {
    const list = ingredients.filter(
      (i) => (!onlyStock || i.status !== "none" || selected.includes(i.id)) && (!filter || i.name.includes(filter.trim())),
    );
    const g = new Map<string, IngredientWithStock[]>();
    for (const i of list) {
      if (!g.has(i.category)) g.set(i.category, []);
      g.get(i.category)!.push(i);
    }
    return [...g.entries()];
  }, [ingredients, onlyStock, filter, selected]);

  const toggle = (id: string) => setSelected((s) => (s.includes(id) ? s.filter((x) => x !== id) : [...s, id]));

  return (
    <div>
      <PageHeader title="뭐 해먹지?" />
      <p className="mb-3 text-sm text-muted">
        쓰고 싶은 재료를 고르세요. 아무것도 고르지 않으면 <b>지금 재고</b>로 만들 수 있는 요리를 보여 줘요.
      </p>

      <div className="mb-2 flex items-center gap-2">
        <input className="field flex-1 py-2" placeholder="재료 찾기" value={filter} onChange={(e) => setFilter(e.target.value)} />
        <label className="flex shrink-0 items-center gap-1 text-sm text-muted">
          <input type="checkbox" className="accent-accent" checked={onlyStock} onChange={(e) => setOnlyStock(e.target.checked)} />
          재고만
        </label>
      </div>

      <div className="card mb-5 max-h-64 overflow-y-auto p-3">
        {grouped.length === 0 && <p className="text-sm text-muted">표시할 재료가 없어요.</p>}
        {grouped.map(([cat, list]) => (
          <div key={cat} className="mb-2 last:mb-0">
            <h3 className="mb-1 text-xs font-bold text-muted">{cat}</h3>
            <div className="flex flex-wrap gap-1.5">
              {list.map((i) => {
                const on = selected.includes(i.id);
                return (
                  <button
                    key={i.id}
                    onClick={() => toggle(i.id)}
                    className={cx("chip", on ? "border-accent bg-accent text-white" : "border-line bg-card")}
                  >
                    {!on && <StatusDot status={i.status} />}
                    {i.name}
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>

      {selected.length > 0 && (
        <div className="mb-3 flex items-center justify-between">
          <span className="text-sm font-semibold">{selected.length}개 선택</span>
          <button className="text-sm text-muted underline" onClick={() => setSelected([])}>
            선택 해제
          </button>
        </div>
      )}

      <ErrorText error={error} />
      {isFetching && recs.length === 0 ? (
        <p className="text-muted">찾는 중…</p>
      ) : recs.length === 0 ? (
        <Empty>맞는 레시피가 없어요.</Empty>
      ) : (
        <ul className="flex flex-col gap-3">
          {recs.map((r) => (
            <li key={r.id}>
              <RecipeCard
                recipe={r}
                extra={
                  <div className="mt-1.5">
                    <div className="h-1.5 overflow-hidden rounded-full bg-line">
                      <div className="h-full rounded-full bg-ok" style={{ width: `${Math.round(r.coverage * 100)}%` }} />
                    </div>
                    {r.matched.length > 0 && <p className="mt-1 text-xs text-accent">고른 재료: {r.matched.join(", ")}</p>}
                  </div>
                }
              />
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

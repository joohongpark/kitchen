import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import type { Level, RecipeDetail, StockAdjustment } from "../types";
import { LEVEL_LABEL, LEVELS, STORAGE_LABEL } from "../types";
import { formatQty } from "../util";
import { ErrorText, ExpiryBadge, Sheet } from "./ui";

/** 요리 완료: 서버가 제안한 차감안을 확인·수정한 뒤 적용 */
export function CookSheet({
  open,
  recipe,
  onClose,
  onDone,
}: {
  open: boolean;
  recipe: RecipeDetail;
  onClose: () => void;
  onDone: (message: string) => void;
}) {
  const qc = useQueryClient();
  const [servings, setServings] = useState<number | undefined>(recipe.servings ?? undefined);
  const { data: plan, isFetching, error } = useQuery({
    queryKey: ["cook-plan", recipe.id, servings],
    queryFn: () => api.cookPlan(recipe.id, servings),
    enabled: open,
  });

  // pantry_item_id → 편집 중인 값
  const [values, setValues] = useState<Record<string, { quantity?: string; level?: Level }>>({});
  const [addMissing, setAddMissing] = useState(true);
  const [note, setNote] = useState("");

  useEffect(() => {
    if (!plan) return;
    const v: typeof values = {};
    for (const line of plan.lines)
      for (const lot of line.lots)
        v[lot.pantry_item_id] =
          lot.tracking === "quantity" ? { quantity: formatQty(lot.suggested_quantity) } : { level: lot.suggested_level ?? "out" };
    setValues(v);
  }, [plan]);

  const cook = useMutation({
    mutationFn: () => {
      const adjustments: StockAdjustment[] = [];
      for (const line of plan?.lines ?? [])
        for (const lot of line.lots) {
          const v = values[lot.pantry_item_id];
          if (!v) continue;
          if (lot.tracking === "quantity") {
            const q = Number(v.quantity);
            if (!Number.isNaN(q) && q !== lot.current_quantity) adjustments.push({ pantry_item_id: lot.pantry_item_id, quantity: q });
          } else if (v.level && v.level !== lot.current_level) {
            adjustments.push({ pantry_item_id: lot.pantry_item_id, level: v.level });
          }
        }
      return api.cook(recipe.id, { adjustments, note: note || undefined, add_missing_to_shopping: addMissing });
    },
    onSuccess: (res) => {
      qc.invalidateQueries();
      const extra = res.added_to_shopping.length ? ` 장보기에 ${res.added_to_shopping.join(", ")} 추가.` : "";
      onDone(`요리 기록 완료!${extra}`);
    },
  });

  const missingRequired = plan?.lines.filter((l) => l.missing && !l.optional) ?? [];

  return (
    <Sheet open={open} onClose={onClose} title={`${recipe.title} 요리 완료`}>
      <div className="flex flex-col gap-4">
        {recipe.servings && (
          <label className="flex items-center gap-3">
            <span className="text-sm font-semibold text-muted">몇 인분 만들었나요?</span>
            <input
              type="number"
              min="1"
              className="field w-20"
              value={servings ?? ""}
              onChange={(e) => setServings(e.target.value ? Number(e.target.value) : undefined)}
            />
          </label>
        )}

        <p className="text-sm text-muted">
          사용 후 남은 양을 확인하세요. 원래 재료 → 대체 재료 순으로, 유통기한이 빠른 것부터 차감했어요.
        </p>
        <ErrorText error={error} />
        {isFetching && !plan && <p className="text-muted">계산 중…</p>}

        <ul className="card divide-y divide-line">
          {plan?.lines.map((line) => (
            <li key={line.ingredient_id} className="px-3 py-2.5">
              <div className="flex items-baseline justify-between">
                <span className="font-semibold">
                  {line.ingredient_name}
                  {line.optional && <span className="ml-1 text-xs font-normal text-muted">(선택)</span>}
                </span>
                <span className="text-sm text-muted">{line.amount_label}</span>
              </div>
              {line.missing ? (
                <p className="mt-1 text-sm text-muted">재고 없음</p>
              ) : (
                <div className="mt-1.5 flex flex-col gap-1.5">
                  {line.lots.map((lot) => {
                    const v = values[lot.pantry_item_id] ?? {};
                    return (
                      <div key={lot.pantry_item_id} className="flex items-center gap-2 text-sm">
                        {lot.ingredient_id !== line.ingredient_id && (
                          <span className={lot.is_substitute ? "font-semibold text-accent" : "font-semibold"}>
                            {lot.ingredient_name}
                            {lot.is_substitute && " (대체)"}
                          </span>
                        )}
                        <span className="text-muted">{STORAGE_LABEL[lot.storage]}</span>
                        <ExpiryBadge date={lot.expires_at} />
                        <span className="flex-1 text-right text-muted">
                          {lot.tracking === "quantity" ? `${formatQty(lot.current_quantity)}${lot.unit} →` : `${LEVEL_LABEL[lot.current_level ?? "out"]} →`}
                        </span>
                        {lot.tracking === "quantity" ? (
                          <span className="flex items-center gap-1">
                            <input
                              type="number"
                              inputMode="decimal"
                              step="any"
                              min="0"
                              className="w-20 rounded-lg border border-line bg-card px-2 py-1 text-right"
                              value={v.quantity ?? ""}
                              onChange={(e) => setValues((s) => ({ ...s, [lot.pantry_item_id]: { quantity: e.target.value } }))}
                            />
                            <span>{lot.unit}</span>
                          </span>
                        ) : (
                          <select
                            className="rounded-lg border border-line bg-card px-2 py-1"
                            value={v.level ?? "out"}
                            onChange={(e) => setValues((s) => ({ ...s, [lot.pantry_item_id]: { level: e.target.value as Level } }))}
                          >
                            {LEVELS.map((l) => (
                              <option key={l} value={l}>
                                {LEVEL_LABEL[l]}
                              </option>
                            ))}
                          </select>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </li>
          ))}
        </ul>

        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" className="size-4 accent-accent" checked={addMissing} onChange={(e) => setAddMissing(e.target.checked)} />
          다 떨어진 재료는 장보기에 추가
          {missingRequired.length > 0 && <span className="text-muted">(지금 없는 것 {missingRequired.length}개)</span>}
        </label>

        <input className="field" placeholder="메모 (예: 된장 조금 더 넣으니 좋았음)" value={note} onChange={(e) => setNote(e.target.value)} />

        <ErrorText error={cook.error} />
        <button className="btn-primary py-3" disabled={!plan || cook.isPending} onClick={() => cook.mutate()}>
          {cook.isPending ? "저장 중…" : "재고 반영하고 기록"}
        </button>
      </div>
    </Sheet>
  );
}

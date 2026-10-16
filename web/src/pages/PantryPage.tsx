import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, photoUrl } from "../api";
import { PantryItemSheet } from "../components/PantryItemSheet";
import { Empty, ErrorText, ExpiryBadge, Fab, PageHeader, Segmented } from "../components/ui";
import type { Level, PantryItem, Storage } from "../types";
import { LEVEL_LABEL, LEVELS, STORAGE_LABEL, STORAGES } from "../types";
import { cx, daysUntil, formatQty } from "../util";

type Filter = "all" | Storage;

export function PantryPage() {
  const { data: items = [], isLoading, error } = useQuery({ queryKey: ["pantry"], queryFn: api.pantry });
  const [filter, setFilter] = useState<Filter>("all");
  const [showEmpty, setShowEmpty] = useState(false);
  const [editing, setEditing] = useState<PantryItem | null>(null);
  const [adding, setAdding] = useState(false);

  const isDepleted = (it: PantryItem) =>
    it.tracking === "quantity" ? (it.quantity ?? 0) <= 0 : it.level === "out";

  const visible = items.filter((it) => (filter === "all" || it.storage === filter) && (showEmpty || !isDepleted(it)));
  const urgent = items.filter((it) => !isDepleted(it) && (daysUntil(it.expires_at) ?? 99) <= 3);
  const depletedCount = items.filter(isDepleted).length;

  const groups = useMemo(() => {
    const g = new Map<Storage, PantryItem[]>();
    for (const s of STORAGES) g.set(s, []);
    for (const it of visible) g.get(it.storage)!.push(it);
    return [...g.entries()].filter(([, v]) => v.length > 0);
  }, [visible]);

  return (
    <div>
      <PageHeader title="재고" />

      {urgent.length > 0 && (
        <section className="mb-4 rounded-2xl bg-warn-soft p-3">
          <h2 className="mb-2 text-sm font-bold text-warn">먼저 써야 할 재료</h2>
          <div className="flex flex-wrap gap-1.5">
            {urgent.map((it) => (
              <button key={it.id} onClick={() => setEditing(it)} className="chip border-warn/30 bg-card">
                {it.ingredient_name}
                <ExpiryBadge date={it.expires_at} />
              </button>
            ))}
          </div>
        </section>
      )}

      <div className="mb-4">
        <Segmented
          value={filter}
          onChange={setFilter}
          options={[{ value: "all" as Filter, label: "전체" }, ...STORAGES.map((s) => ({ value: s as Filter, label: STORAGE_LABEL[s] }))]}
        />
      </div>

      <ErrorText error={error} />
      {isLoading ? (
        <p className="text-muted">불러오는 중…</p>
      ) : visible.length === 0 ? (
        <Empty>
          아직 재고가 없어요.
          <br />
          아래 <b>+ 재고</b> 버튼으로 추가해 보세요.
        </Empty>
      ) : (
        <div className="flex flex-col gap-5">
          {groups.map(([storage, list]) => (
            <section key={storage}>
              <h2 className="mb-2 text-sm font-bold text-muted">
                {STORAGE_LABEL[storage]} · {list.length}
              </h2>
              <ul className="card divide-y divide-line">
                {list.map((it) => (
                  <PantryRow key={it.id} item={it} onEdit={() => setEditing(it)} depleted={isDepleted(it)} />
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}

      {depletedCount > 0 && (
        <button className="mt-4 text-sm text-muted underline" onClick={() => setShowEmpty((v) => !v)}>
          {showEmpty ? "다 쓴 재고 숨기기" : `다 쓴 재고 ${depletedCount}개 보기`}
        </button>
      )}

      <Fab label="재고" onClick={() => setAdding(true)} />
      <PantryItemSheet open={adding} onClose={() => setAdding(false)} />
      <PantryItemSheet open={!!editing} item={editing} onClose={() => setEditing(null)} />
    </div>
  );
}

function PantryRow({ item, onEdit, depleted }: { item: PantryItem; onEdit: () => void; depleted: boolean }) {
  const qc = useQueryClient();
  const adjust = useMutation({
    mutationFn: (b: { quantity?: number; level?: Level }) => api.adjustPantry([{ pantry_item_id: item.id, ...b }]),
    onSuccess: () => qc.invalidateQueries(),
  });

  const step = item.unit && ["g", "ml"].includes(item.unit) ? 50 : 1;

  return (
    <li className={cx("flex items-center gap-3 px-3 py-2.5", depleted && "opacity-50")}>
      <button className="flex min-w-0 flex-1 items-center gap-3 text-left" onClick={onEdit}>
        {item.photo_id ? (
          <img src={photoUrl(item.photo_id)} alt="" className="size-10 shrink-0 rounded-lg object-cover" />
        ) : (
          <div className="grid size-10 shrink-0 place-items-center rounded-lg bg-accent-soft text-sm font-bold text-accent">
            {item.ingredient_name.slice(0, 1)}
          </div>
        )}
        <div className="min-w-0">
          <div className="flex items-center gap-1.5">
            <span className="truncate font-semibold">{item.ingredient_name}</span>
            <ExpiryBadge date={item.expires_at} />
          </div>
          {item.note && <p className="truncate text-xs text-muted">{item.note}</p>}
        </div>
      </button>

      {item.tracking === "quantity" ? (
        <div className="flex shrink-0 items-center gap-1">
          <button
            className="grid size-8 place-items-center rounded-full border border-line text-lg leading-none"
            disabled={adjust.isPending}
            onClick={() => adjust.mutate({ quantity: Math.max(0, (item.quantity ?? 0) - step) })}
            aria-label="줄이기"
          >
            −
          </button>
          <span className="min-w-14 text-center text-sm font-semibold tabular-nums">
            {formatQty(item.quantity)}
            <span className="text-muted">{item.unit}</span>
          </span>
          <button
            className="grid size-8 place-items-center rounded-full border border-line text-lg leading-none"
            disabled={adjust.isPending}
            onClick={() => adjust.mutate({ quantity: (item.quantity ?? 0) + step })}
            aria-label="늘리기"
          >
            +
          </button>
        </div>
      ) : (
        <select
          className="shrink-0 rounded-lg border border-line bg-card px-2 py-1.5 text-sm font-semibold"
          value={item.level ?? "out"}
          onChange={(e) => adjust.mutate({ level: e.target.value as Level })}
        >
          {LEVELS.map((l) => (
            <option key={l} value={l}>
              {LEVEL_LABEL[l]}
            </option>
          ))}
        </select>
      )}
    </li>
  );
}

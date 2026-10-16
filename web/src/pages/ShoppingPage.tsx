import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import { PantryItemSheet, type PantryPreset } from "../components/PantryItemSheet";
import { Empty, ErrorText, PageHeader } from "../components/ui";
import type { ShoppingItem } from "../types";
import { cx } from "../util";

export function ShoppingPage() {
  const qc = useQueryClient();
  const { data: items = [], error } = useQuery({ queryKey: ["shopping"], queryFn: api.shopping });
  const [name, setName] = useState("");
  const [stockPreset, setStockPreset] = useState<PantryPreset | null>(null);

  const add = useMutation({
    mutationFn: () => api.addShopping({ name }),
    onSuccess: () => {
      setName("");
      qc.invalidateQueries({ queryKey: ["shopping"] });
    },
  });
  const toggle = useMutation({
    mutationFn: (it: ShoppingItem) => api.updateShopping(it.id, { done: !it.done }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["shopping"] }),
  });
  const remove = useMutation({
    mutationFn: (id: string) => api.deleteShopping(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["shopping"] }),
  });
  const clear = useMutation({
    mutationFn: api.clearDoneShopping,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["shopping"] }),
  });

  const open = items.filter((i) => !i.done);
  const done = items.filter((i) => i.done);

  return (
    <div>
      <PageHeader title="장보기" />
      <form
        className="mb-4 flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          if (name.trim()) add.mutate();
        }}
      >
        <input className="field" placeholder="살 것 추가" value={name} onChange={(e) => setName(e.target.value)} />
        <button className="btn-primary shrink-0" disabled={add.isPending}>
          추가
        </button>
      </form>
      <ErrorText error={error ?? add.error} />

      {items.length === 0 ? (
        <Empty>장볼 것이 없어요.</Empty>
      ) : (
        <>
          <ul className="card divide-y divide-line">
            {open.map((it) => (
              <Row key={it.id} item={it} onToggle={() => toggle.mutate(it)} onDelete={() => remove.mutate(it.id)} />
            ))}
            {open.length === 0 && <li className="px-3 py-4 text-center text-sm text-muted">다 샀어요! 🎉</li>}
          </ul>

          {done.length > 0 && (
            <section className="mt-6">
              <div className="mb-2 flex items-center justify-between">
                <h2 className="text-sm font-bold text-muted">산 것 · {done.length}</h2>
                <button className="text-sm text-muted underline" onClick={() => clear.mutate()}>
                  목록에서 지우기
                </button>
              </div>
              <p className="mb-2 text-xs text-muted">'재고에 넣기'를 누르면 바로 재고로 등록할 수 있어요.</p>
              <ul className="card divide-y divide-line">
                {done.map((it) => (
                  <Row
                    key={it.id}
                    item={it}
                    onToggle={() => toggle.mutate(it)}
                    onDelete={() => remove.mutate(it.id)}
                    onStock={() => setStockPreset({ ingredient_id: it.ingredient_id, name: it.name })}
                  />
                ))}
              </ul>
            </section>
          )}
        </>
      )}

      <PantryItemSheet open={!!stockPreset} preset={stockPreset} onClose={() => setStockPreset(null)} />
    </div>
  );
}

function Row({ item, onToggle, onDelete, onStock }: { item: ShoppingItem; onToggle: () => void; onDelete: () => void; onStock?: () => void }) {
  return (
    <li className="flex items-center gap-3 px-3 py-2.5">
      <input type="checkbox" className="size-5 shrink-0 accent-accent" checked={item.done} onChange={onToggle} />
      <span className={cx("flex-1", item.done && "text-muted line-through")}>
        {item.name}
        {item.amount && <span className="ml-1.5 text-sm text-muted">{item.amount}</span>}
      </span>
      {onStock && (
        <button className="text-sm font-semibold text-accent" onClick={onStock}>
          재고에 넣기
        </button>
      )}
      <button className="px-1 text-muted" onClick={onDelete} aria-label="삭제">
        ✕
      </button>
    </li>
  );
}

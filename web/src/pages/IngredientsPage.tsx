import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import { PantryItemSheet, type PantryPreset } from "../components/PantryItemSheet";
import { Empty, ErrorText, PageHeader, PhotoField, Segmented, Sheet, StatusDot } from "../components/ui";
import type { IngredientWithStock, Storage } from "../types";
import { CATEGORIES, STORAGE_LABEL, STORAGES, UNITS } from "../types";

export function IngredientsPage() {
  const navigate = useNavigate();
  const { data = [], error } = useQuery({ queryKey: ["ingredients"], queryFn: api.ingredients });
  const [q, setQ] = useState("");
  const [editing, setEditing] = useState<IngredientWithStock | "new" | null>(null);

  const grouped = useMemo(() => {
    const g = new Map<string, IngredientWithStock[]>();
    for (const i of data.filter((i) => !q || i.name.includes(q.trim()))) {
      if (!g.has(i.category)) g.set(i.category, []);
      g.get(i.category)!.push(i);
    }
    return [...g.entries()];
  }, [data, q]);

  return (
    <div>
      <PageHeader
        title="식재료"
        right={
          <button className="text-muted" onClick={() => navigate(-1)}>
            닫기
          </button>
        }
      />
      <div className="mb-4 flex gap-2">
        <input className="field" placeholder="검색" value={q} onChange={(e) => setQ(e.target.value)} />
        <button className="btn-primary shrink-0" onClick={() => setEditing("new")}>
          + 새로
        </button>
      </div>
      <ErrorText error={error} />
      {grouped.length === 0 ? (
        <Empty>식재료가 없어요.</Empty>
      ) : (
        <div className="flex flex-col gap-4">
          {grouped.map(([cat, list]) => (
            <section key={cat}>
              <h2 className="mb-1.5 text-sm font-bold text-muted">{cat}</h2>
              <ul className="card divide-y divide-line">
                {list.map((i) => (
                  <li key={i.id}>
                    <button className="flex w-full items-center gap-2.5 px-3 py-2.5 text-left" onClick={() => setEditing(i)}>
                      <StatusDot status={i.status} />
                      <span className="flex-1 font-semibold">{i.name}</span>
                      <span className="text-sm text-muted">{i.stock_summary ?? "없음"}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}
      <IngredientSheet value={editing} onClose={() => setEditing(null)} />
    </div>
  );
}

function IngredientSheet({ value, onClose }: { value: IngredientWithStock | "new" | null; onClose: () => void }) {
  const qc = useQueryClient();
  const ing = value && value !== "new" ? value : null;
  const [name, setName] = useState("");
  const [category, setCategory] = useState("기타");
  const [unit, setUnit] = useState("");
  const [storage, setStorage] = useState<Storage>("fridge");
  const [low, setLow] = useState("");
  const [photoId, setPhotoId] = useState<string | null>(null);
  const [note, setNote] = useState("");
  const [stockPreset, setStockPreset] = useState<PantryPreset | null>(null);

  useEffect(() => {
    if (!value) return;
    setName(ing?.name ?? "");
    setCategory(ing?.category ?? "기타");
    setUnit(ing?.default_unit ?? "");
    setStorage(ing?.default_storage ?? "fridge");
    setLow(ing?.low_threshold != null ? String(ing.low_threshold) : "");
    setPhotoId(ing?.photo_id ?? null);
    setNote(ing?.note ?? "");
  }, [value, ing]);

  const save = useMutation({
    mutationFn: () => {
      const body = {
        name,
        category,
        default_unit: unit || null,
        default_storage: storage,
        low_threshold: low ? Number(low) : null,
        photo_id: photoId,
        note: note || null,
      };
      return ing ? api.updateIngredient(ing.id, body) : api.createIngredient(body);
    },
    onSuccess: () => {
      qc.invalidateQueries();
      onClose();
    },
  });
  const remove = useMutation({
    mutationFn: () => api.deleteIngredient(ing!.id),
    onSuccess: () => {
      qc.invalidateQueries();
      onClose();
    },
  });

  return (
    <>
      <Sheet open={!!value} onClose={onClose} title={ing ? ing.name : "새 식재료"}>
        <form
          className="flex flex-col gap-4"
          onSubmit={(e) => {
            e.preventDefault();
            save.mutate();
          }}
        >
          <label>
            <span className="label">이름</span>
            <input className="field" value={name} onChange={(e) => setName(e.target.value)} required />
          </label>
          <div className="grid grid-cols-2 gap-3">
            <label>
              <span className="label">분류</span>
              <select className="field" value={category} onChange={(e) => setCategory(e.target.value)}>
                {[...new Set([...CATEGORIES, category])].map((c) => (
                  <option key={c}>{c}</option>
                ))}
              </select>
            </label>
            <label>
              <span className="label">기본 단위</span>
              <input className="field" list="ing-units" value={unit} onChange={(e) => setUnit(e.target.value)} placeholder="예: 모, g" />
              <datalist id="ing-units">
                {UNITS.map((u) => (
                  <option key={u} value={u} />
                ))}
              </datalist>
            </label>
          </div>
          <div>
            <span className="label">기본 보관</span>
            <Segmented value={storage} onChange={setStorage} options={STORAGES.map((s) => ({ value: s, label: STORAGE_LABEL[s] }))} />
          </div>
          <label>
            <span className="label">부족 알림 기준 ({unit || "기본 단위"} 이하)</span>
            <input className="field" type="number" step="any" min="0" value={low} onChange={(e) => setLow(e.target.value)} placeholder="비우면 알림 안 함" />
          </label>
          <PhotoField value={photoId} onChange={setPhotoId} />
          <label>
            <span className="label">메모</span>
            <input className="field" value={note} onChange={(e) => setNote(e.target.value)} placeholder="손질법, 보관 팁 등" />
          </label>
          <ErrorText error={save.error ?? remove.error} />
          <div className="flex gap-2">
            {ing && (
              <button type="button" className="btn-danger" onClick={() => window.confirm(`${ing.name}을(를) 삭제할까요? 재고도 함께 삭제됩니다.`) && remove.mutate()}>
                삭제
              </button>
            )}
            {ing && (
              <button type="button" className="btn-ghost" onClick={() => setStockPreset({ ingredient_id: ing.id, name: ing.name })}>
                재고 추가
              </button>
            )}
            <button className="btn-primary flex-1" disabled={save.isPending}>
              저장
            </button>
          </div>
        </form>
      </Sheet>
      <PantryItemSheet open={!!stockPreset} preset={stockPreset} onClose={() => setStockPreset(null)} />
    </>
  );
}

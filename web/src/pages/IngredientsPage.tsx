import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import { IngredientPicker, type Picked } from "../components/IngredientPicker";
import { PantryItemSheet, type PantryPreset } from "../components/PantryItemSheet";
import { Empty, ErrorText, PageHeader, PhotoField, Segmented, Sheet, StatusDot } from "../components/ui";
import type { IngredientWithStock, Storage } from "../types";
import { CATEGORIES, STORAGE_LABEL, STORAGES, UNITS } from "../types";

export function IngredientsPage() {
  const navigate = useNavigate();
  const { data = [], error } = useQuery({ queryKey: ["ingredients"], queryFn: api.ingredients });
  const [q, setQ] = useState("");
  const [editing, setEditing] = useState<IngredientWithStock | "new" | null>(null);

  const byId = useMemo(() => new Map(data.map((i) => [i.id, i])), [data]);

  // 분류별로 최상위 재료만 나열하고, 묶음의 하위 재료는 그 아래에 들여 쓴다
  const grouped = useMemo(() => {
    const needle = q.trim();
    const hit = (i: IngredientWithStock) => !needle || i.name.includes(needle);
    const g = new Map<string, { item: IngredientWithStock; kids: IngredientWithStock[] }[]>();
    for (const i of data) {
      if (i.parent_id && byId.has(i.parent_id)) continue;
      const kids = i.children.map((id) => byId.get(id)!).filter(Boolean);
      if (!hit(i) && !kids.some(hit)) continue;
      if (!g.has(i.category)) g.set(i.category, []);
      g.get(i.category)!.push({ item: i, kids: needle && !hit(i) ? kids.filter(hit) : kids });
    }
    return [...g.entries()];
  }, [data, byId, q]);

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
      <div className="mb-2 flex gap-2">
        <input className="field" placeholder="검색" value={q} onChange={(e) => setQ(e.target.value)} />
        <button className="btn-primary shrink-0" onClick={() => setEditing("new")}>
          + 새로
        </button>
      </div>
      <p className="mb-4 text-xs text-muted">
        <b>묶음</b>: 고춧가루 ← 청양 고춧가루처럼 종류를 묶어요. 레시피에 묶음을 쓰면 하위 재료 중 아무거나 있으면 돼요.
      </p>
      <ErrorText error={error} />
      {grouped.length === 0 ? (
        <Empty>식재료가 없어요.</Empty>
      ) : (
        <div className="flex flex-col gap-4">
          {grouped.map(([cat, list]) => (
            <section key={cat}>
              <h2 className="mb-1.5 text-sm font-bold text-muted">{cat}</h2>
              <ul className="card divide-y divide-line">
                {list.map(({ item, kids }) => (
                  <li key={item.id}>
                    <Row item={item} onClick={() => setEditing(item)} />
                    {kids.map((k) => (
                      <Row key={k.id} item={k} child onClick={() => setEditing(k)} />
                    ))}
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}
      <IngredientSheet value={editing} all={data} onClose={() => setEditing(null)} />
    </div>
  );
}

function Row({ item, child, onClick }: { item: IngredientWithStock; child?: boolean; onClick: () => void }) {
  const isGroup = item.children.length > 0;
  return (
    <button className={`flex w-full items-center gap-2.5 py-2.5 pr-3 text-left ${child ? "pl-8" : "pl-3"}`} onClick={onClick}>
      {child && <span className="-ml-4 text-muted">↳</span>}
      <StatusDot status={item.status} />
      <span className="flex-1 font-semibold">
        {item.name}
        {isGroup && <span className="ml-1.5 rounded bg-accent-soft px-1.5 py-0.5 text-[11px] text-accent">묶음 · {item.children.length}종</span>}
      </span>
      {!isGroup && <span className="text-sm text-muted">{item.stock_summary ?? "없음"}</span>}
    </button>
  );
}

function IngredientSheet({
  value,
  all,
  onClose,
}: {
  value: IngredientWithStock | "new" | null;
  all: IngredientWithStock[];
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const ing = value && value !== "new" ? value : null;
  const [name, setName] = useState("");
  const [category, setCategory] = useState("기타");
  const [unit, setUnit] = useState("");
  const [storage, setStorage] = useState<Storage>("fridge");
  const [low, setLow] = useState("");
  const [photoId, setPhotoId] = useState<string | null>(null);
  const [note, setNote] = useState("");
  const [parent, setParent] = useState<Picked>({ id: null, name: "" });
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
    const p = ing?.parent_id ? all.find((i) => i.id === ing.parent_id) : null;
    setParent(p ? { id: p.id, name: p.name } : { id: null, name: "" });
  }, [value]);

  const children = ing ? ing.children.map((id) => all.find((i) => i.id === id)?.name).filter(Boolean) : [];
  // 묶음 후보: 다른 묶음에 속하지 않은 재료 (자기 자신 제외)
  const parentCandidates = all.filter((i) => !i.parent_id && i.id !== ing?.id);

  const save = useMutation({
    mutationFn: async () => {
      let parentId = parent.id;
      if (!parentId && parent.name.trim()) {
        // 새 묶음 이름이면 묶음 재료를 먼저 만든다
        const created = await api.createIngredient({ name: parent.name.trim(), category, default_storage: storage });
        parentId = created.id;
      }
      const body = {
        name,
        category,
        default_unit: unit || null,
        default_storage: storage,
        low_threshold: low ? Number(low) : null,
        photo_id: photoId,
        note: note || null,
        parent_id: parentId,
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

          <div>
            <span className="label">묶음 (선택)</span>
            {children.length > 0 ? (
              <p className="rounded-xl bg-accent-soft/60 px-3 py-2 text-sm">
                이 재료는 묶음이에요: <b>{children.join(", ")}</b>
              </p>
            ) : (
              <>
                <div className="flex gap-2">
                  <div className="flex-1">
                    <IngredientPicker
                      ingredients={parentCandidates}
                      value={parent.name}
                      placeholder="예: 고춧가루 (비우면 묶음 없음)"
                      onChange={setParent}
                    />
                  </div>
                  {parent.name && (
                    <button type="button" className="shrink-0 px-2 text-sm text-muted" onClick={() => setParent({ id: null, name: "" })}>
                      해제
                    </button>
                  )}
                </div>
                {!parent.id && parent.name.trim() && <p className="mt-1 text-xs text-accent">'{parent.name.trim()}' 묶음을 새로 만들어요.</p>}
              </>
            )}
          </div>

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
              <button
                type="button"
                className="btn-danger"
                onClick={() => window.confirm(`${ing.name}을(를) 삭제할까요? 재고도 함께 삭제됩니다.`) && remove.mutate()}
              >
                삭제
              </button>
            )}
            {ing && children.length === 0 && (
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

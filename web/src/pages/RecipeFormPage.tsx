import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import { IngredientPicker, type Picked } from "../components/IngredientPicker";
import { ErrorText, PageHeader, PhotoField } from "../components/ui";
import type { IngredientWithStock, RecipeInput } from "../types";
import { UNITS } from "../types";

interface IngRow {
  key: number;
  ingredient_id: string | null;
  name: string;
  quantity: string;
  unit: string;
  amount_text: string;
  optional: boolean;
  note: string;
  alts: AltRow[];
  /** 대체 재료 입력 중인 값 (null 이면 입력창 닫힘) */
  altDraft: Picked | null;
}

interface AltRow {
  key: number;
  ingredient_id: string | null;
  name: string;
  note: string;
}

interface StepRow {
  key: number;
  body: string;
  photo_id: string | null;
}

let seq = 0;
const newIng = (): IngRow => ({
  key: ++seq,
  ingredient_id: null,
  name: "",
  quantity: "",
  unit: "",
  amount_text: "",
  optional: false,
  note: "",
  alts: [],
  altDraft: null,
});
const newStep = (): StepRow => ({ key: ++seq, body: "", photo_id: null });

export function RecipeFormPage() {
  const { id } = useParams();
  const editing = !!id;
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { data: existing } = useQuery({ queryKey: ["recipe", id], queryFn: () => api.recipe(id!), enabled: editing });
  const { data: ingredients = [] } = useQuery({ queryKey: ["ingredients"], queryFn: api.ingredients });

  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [servings, setServings] = useState("2");
  const [minutes, setMinutes] = useState("");
  const [tags, setTags] = useState("");
  const [photoId, setPhotoId] = useState<string | null>(null);
  const [source, setSource] = useState("");
  const [ings, setIngs] = useState<IngRow[]>([newIng()]);
  const [steps, setSteps] = useState<StepRow[]>([newStep()]);

  useEffect(() => {
    if (!existing) return;
    setTitle(existing.title);
    setDescription(existing.description ?? "");
    setServings(existing.servings ? String(existing.servings) : "");
    setMinutes(existing.cook_minutes ? String(existing.cook_minutes) : "");
    setTags(existing.tags.join(", "));
    setPhotoId(existing.photo_id);
    setSource(existing.source ?? "");
    setIngs(
      existing.ingredients.map((i) => ({
        key: ++seq,
        ingredient_id: i.ingredient_id,
        name: i.ingredient_name,
        quantity: i.quantity != null ? String(i.quantity) : "",
        unit: i.unit ?? "",
        amount_text: i.amount_text ?? "",
        optional: i.optional,
        note: i.note ?? "",
        alts: i.alternatives.map((a) => ({ key: ++seq, ingredient_id: a.ingredient_id, name: a.name, note: a.note ?? "" })),
        altDraft: null,
      })),
    );
    setSteps(existing.steps.length ? existing.steps.map((s) => ({ key: ++seq, body: s.body, photo_id: s.photo_id })) : [newStep()]);
  }, [existing]);

  const save = useMutation({
    mutationFn: () => {
      const body: RecipeInput = {
        title,
        description: description || null,
        servings: servings ? Number(servings) : null,
        cook_minutes: minutes ? Number(minutes) : null,
        tags: tags.split(",").map((t) => t.trim()).filter(Boolean),
        photo_id: photoId,
        source: source || null,
        ingredients: ings
          .filter((i) => i.name.trim())
          .map((i) => ({
            ingredient_id: i.ingredient_id,
            name: i.name.trim(),
            quantity: i.quantity ? Number(i.quantity) : null,
            unit: i.quantity ? i.unit || null : null,
            amount_text: i.quantity ? null : i.amount_text || null,
            optional: i.optional,
            note: i.note || null,
            alternatives: i.alts.map((a) => ({ ingredient_id: a.ingredient_id, name: a.name, note: a.note || null })),
          })),
        steps: steps.filter((s) => s.body.trim()).map((s) => ({ body: s.body, photo_id: s.photo_id })),
      };
      return editing ? api.updateRecipe(id!, body) : api.createRecipe(body);
    },
    onSuccess: (r) => {
      qc.invalidateQueries();
      navigate(`/recipes/${r.id}`, { replace: true });
    },
  });

  const updIng = (key: number, patch: Partial<IngRow>) => setIngs((rows) => rows.map((r) => (r.key === key ? { ...r, ...patch } : r)));
  const updStep = (key: number, patch: Partial<StepRow>) => setSteps((rows) => rows.map((r) => (r.key === key ? { ...r, ...patch } : r)));
  const move = <T,>(arr: T[], i: number, d: number) => {
    const j = i + d;
    if (j < 0 || j >= arr.length) return arr;
    const next = [...arr];
    [next[i], next[j]] = [next[j], next[i]];
    return next;
  };

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate();
      }}
      className="flex flex-col gap-5 pb-8"
    >
      <PageHeader
        title={editing ? "레시피 수정" : "새 레시피"}
        right={
          <button type="button" className="text-muted" onClick={() => navigate(-1)}>
            취소
          </button>
        }
      />

      <label>
        <span className="label">요리 이름</span>
        <input className="field text-lg font-bold" value={title} onChange={(e) => setTitle(e.target.value)} required placeholder="예: 아욱된장국" />
      </label>
      <PhotoField value={photoId} onChange={setPhotoId} label="대표 사진" />
      <label>
        <span className="label">설명</span>
        <textarea className="field" rows={2} value={description} onChange={(e) => setDescription(e.target.value)} />
      </label>
      <div className="grid grid-cols-2 gap-3">
        <label>
          <span className="label">인분</span>
          <input className="field" type="number" min="1" value={servings} onChange={(e) => setServings(e.target.value)} />
        </label>
        <label>
          <span className="label">조리 시간(분)</span>
          <input className="field" type="number" min="0" value={minutes} onChange={(e) => setMinutes(e.target.value)} />
        </label>
      </div>
      <label>
        <span className="label">태그 (쉼표로 구분)</span>
        <input className="field" value={tags} onChange={(e) => setTags(e.target.value)} placeholder="국, 된장" />
      </label>

      <section>
        <h2 className="mb-2 text-lg font-bold">재료</h2>
        <p className="mb-3 text-xs text-muted">수량을 적으면 요리 후 재고에서 자동으로 빼요. "약간"처럼 적고 싶으면 수량을 비우세요.</p>
        <datalist id="recipe-units">
          {UNITS.map((u) => (
            <option key={u} value={u} />
          ))}
        </datalist>
        <ul className="flex flex-col gap-2">
          {ings.map((row, idx) => (
            <li key={row.key} className="card flex flex-col gap-2 p-2.5">
              <div className="flex gap-2">
                <div className="flex-1">
                  <IngredientPicker
                    ingredients={ingredients}
                    value={row.name}
                    placeholder="재료"
                    onChange={(p) => {
                      const ing = p.id ? ingredients.find((i) => i.id === p.id) : null;
                      updIng(row.key, { ingredient_id: p.id, name: p.name, ...(ing?.default_unit && !row.unit ? { unit: ing.default_unit } : {}) });
                    }}
                  />
                </div>
                <button type="button" className="px-1 text-muted" onClick={() => setIngs((r) => move(r, idx, -1))} aria-label="위로">
                  ↑
                </button>
                <button type="button" className="px-1 text-muted" onClick={() => setIngs((r) => r.filter((x) => x.key !== row.key))} aria-label="삭제">
                  ✕
                </button>
              </div>
              <div className="flex gap-2">
                <input
                  className="field w-20"
                  type="number"
                  inputMode="decimal"
                  step="any"
                  min="0"
                  placeholder="양"
                  value={row.quantity}
                  onChange={(e) => updIng(row.key, { quantity: e.target.value })}
                />
                {row.quantity ? (
                  <input className="field w-24" list="recipe-units" placeholder="단위" value={row.unit} onChange={(e) => updIng(row.key, { unit: e.target.value })} />
                ) : (
                  <input className="field flex-1" placeholder='또는 "약간", "1/2모"' value={row.amount_text} onChange={(e) => updIng(row.key, { amount_text: e.target.value })} />
                )}
                <label className="flex shrink-0 items-center gap-1 text-sm text-muted">
                  <input type="checkbox" className="accent-accent" checked={row.optional} onChange={(e) => updIng(row.key, { optional: e.target.checked })} />
                  선택
                </label>
              </div>
              <AltEditor
                row={row}
                ingredients={ingredients}
                onChange={(patch) => updIng(row.key, patch)}
              />
            </li>
          ))}
        </ul>
        <button type="button" className="btn-ghost mt-2 w-full" onClick={() => setIngs((r) => [...r, newIng()])}>
          + 재료 추가
        </button>
      </section>

      <section>
        <h2 className="mb-2 text-lg font-bold">만드는 법</h2>
        <ol className="flex flex-col gap-2">
          {steps.map((s, idx) => (
            <li key={s.key} className="card flex flex-col gap-2 p-2.5">
              <div className="flex items-start gap-2">
                <span className="mt-2 grid size-6 shrink-0 place-items-center rounded-full bg-accent text-xs font-bold text-white">{idx + 1}</span>
                <textarea className="field flex-1" rows={2} value={s.body} onChange={(e) => updStep(s.key, { body: e.target.value })} />
                <div className="flex flex-col">
                  <button type="button" className="px-1 text-muted" onClick={() => setSteps((r) => move(r, idx, -1))} aria-label="위로">
                    ↑
                  </button>
                  <button type="button" className="px-1 text-muted" onClick={() => setSteps((r) => r.filter((x) => x.key !== s.key))} aria-label="삭제">
                    ✕
                  </button>
                </div>
              </div>
              <PhotoField value={s.photo_id} onChange={(pid) => updStep(s.key, { photo_id: pid })} label="단계 사진 (선택)" />
            </li>
          ))}
        </ol>
        <button type="button" className="btn-ghost mt-2 w-full" onClick={() => setSteps((r) => [...r, newStep()])}>
          + 단계 추가
        </button>
      </section>

      <label>
        <span className="label">출처 (선택)</span>
        <input className="field" value={source} onChange={(e) => setSource(e.target.value)} placeholder="URL 이나 메모" />
      </label>

      <ErrorText error={save.error} />
      <button className="btn-primary py-3.5 text-base" disabled={save.isPending}>
        {save.isPending ? "저장 중…" : "저장"}
      </button>
    </form>
  );
}

/** 레시피 재료 줄의 대체 재료 편집 (예: 삼겹살 → 목살, 앞다리살) */
function AltEditor({
  row,
  ingredients,
  onChange,
}: {
  row: IngRow;
  ingredients: IngredientWithStock[];
  onChange: (patch: Partial<IngRow>) => void;
}) {
  const draft = row.altDraft;
  const commit = () => {
    if (!draft || !draft.name.trim()) return;
    const exists = row.alts.some((a) => a.name === draft.name.trim()) || draft.name.trim() === row.name.trim();
    onChange({
      alts: exists ? row.alts : [...row.alts, { key: ++seq, ingredient_id: draft.id, name: draft.name.trim(), note: "" }],
      altDraft: null,
    });
  };

  return (
    <div className="flex flex-col gap-1.5">
      {row.alts.length > 0 && (
        <ul className="flex flex-col gap-1.5">
          {row.alts.map((a) => (
            <li key={a.key} className="flex items-center gap-2 text-sm">
              <span className="shrink-0 text-muted">↳ 대체</span>
              <span className="shrink-0 font-semibold">{a.name}</span>
              <input
                className="min-w-0 flex-1 rounded-lg border border-line bg-card px-2 py-1 text-sm"
                placeholder="메모 (예: 기름기 적어 퍽퍽함)"
                value={a.note}
                onChange={(e) => onChange({ alts: row.alts.map((x) => (x.key === a.key ? { ...x, note: e.target.value } : x)) })}
              />
              <button
                type="button"
                className="px-1 text-muted"
                aria-label={`${a.name} 대체 재료 삭제`}
                onClick={() => onChange({ alts: row.alts.filter((x) => x.key !== a.key) })}
              >
                ✕
              </button>
            </li>
          ))}
        </ul>
      )}
      {draft ? (
        <div className="flex gap-2">
          <div className="flex-1">
            <IngredientPicker
              ingredients={ingredients}
              value={draft.name}
              placeholder="대체할 수 있는 재료"
              autoFocus
              onChange={(p) => onChange({ altDraft: p })}
            />
          </div>
          <button type="button" className="btn-ghost shrink-0 px-3 py-1.5 text-sm" onClick={commit}>
            추가
          </button>
          <button type="button" className="shrink-0 px-1 text-muted" onClick={() => onChange({ altDraft: null })}>
            취소
          </button>
        </div>
      ) : (
        <button
          type="button"
          className="self-start text-sm font-semibold text-accent"
          disabled={!row.name.trim()}
          onClick={() => onChange({ altDraft: { id: null, name: "" } })}
        >
          + 대체 재료
        </button>
      )}
    </div>
  );
}

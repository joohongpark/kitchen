import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import type { Level, PantryItem, Storage, Tracking } from "../types";
import { CATEGORIES, STORAGE_LABEL, STORAGES, UNITS } from "../types";
import { todayStr } from "../util";
import { IngredientPicker, type Picked } from "./IngredientPicker";
import { ErrorText, LevelPicker, PhotoField, Segmented, Sheet } from "./ui";

export interface PantryPreset {
  ingredient_id: string | null;
  name: string;
}

/** 재고 추가/수정 시트. `item` 이 있으면 수정 */
export function PantryItemSheet({
  open,
  onClose,
  item,
  preset,
}: {
  open: boolean;
  onClose: () => void;
  item?: PantryItem | null;
  preset?: PantryPreset | null;
}) {
  const qc = useQueryClient();
  const { data: ingredients = [] } = useQuery({ queryKey: ["ingredients"], queryFn: api.ingredients, enabled: open });

  const [picked, setPicked] = useState<Picked>({ id: null, name: "" });
  const [category, setCategory] = useState("기타");
  const [tracking, setTracking] = useState<Tracking>("quantity");
  const [quantity, setQuantity] = useState("1");
  const [unit, setUnit] = useState("개");
  const [level, setLevel] = useState<Level>("plenty");
  const [storage, setStorage] = useState<Storage>("fridge");
  const [purchasedAt, setPurchasedAt] = useState("");
  const [expiresAt, setExpiresAt] = useState("");
  const [photoId, setPhotoId] = useState<string | null>(null);
  const [note, setNote] = useState("");

  useEffect(() => {
    if (!open) return;
    if (item) {
      setPicked({ id: item.ingredient_id, name: item.ingredient_name });
      setTracking(item.tracking);
      setQuantity(item.quantity != null ? String(item.quantity) : "1");
      setUnit(item.unit ?? "개");
      setLevel(item.level ?? "plenty");
      setStorage(item.storage);
      setPurchasedAt(item.purchased_at ?? "");
      setExpiresAt(item.expires_at ?? "");
      setPhotoId(item.photo_id);
      setNote(item.note ?? "");
    } else {
      setPicked(preset ? { id: preset.ingredient_id, name: preset.name } as Picked : { id: null, name: "" });
      setTracking("quantity");
      setQuantity("1");
      setUnit("개");
      setLevel("plenty");
      setStorage("fridge");
      setPurchasedAt(todayStr());
      setExpiresAt("");
      setPhotoId(null);
      setNote("");
    }
  }, [open, item, preset]);

  // 기존 식재료를 고르면 기본 단위·보관 위치를 따른다 (새로 추가할 때만)
  useEffect(() => {
    if (item || !picked.id) return;
    const ing = ingredients.find((i) => i.id === picked.id);
    if (!ing) return;
    if (ing.default_unit) setUnit(ing.default_unit);
    setStorage(ing.default_storage);
  }, [picked.id, ingredients, item]);

  const save = useMutation({
    mutationFn: async () => {
      let ingredientId = picked.id;
      if (!ingredientId) {
        if (!picked.name.trim()) throw new Error("식재료를 입력하세요");
        const created = await api.createIngredient({
          name: picked.name.trim(),
          category,
          default_unit: tracking === "quantity" ? unit : null,
          default_storage: storage,
        });
        ingredientId = created.id;
      }
      const body = {
        ingredient_id: ingredientId,
        tracking,
        quantity: tracking === "quantity" ? Number(quantity) : null,
        unit: tracking === "quantity" ? unit : null,
        level: tracking === "level" ? level : null,
        storage,
        purchased_at: purchasedAt || null,
        expires_at: expiresAt || null,
        photo_id: photoId,
        note: note || null,
      };
      return item ? api.updatePantryItem(item.id, body) : api.createPantryItem(body);
    },
    onSuccess: () => {
      qc.invalidateQueries();
      onClose();
    },
  });

  const remove = useMutation({
    mutationFn: () => api.deletePantryItem(item!.id),
    onSuccess: () => {
      qc.invalidateQueries();
      onClose();
    },
  });

  const quickExpiry = (days: number) => {
    const d = new Date();
    d.setDate(d.getDate() + days);
    const p = (n: number) => String(n).padStart(2, "0");
    setExpiresAt(`${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`);
  };

  return (
    <Sheet open={open} onClose={onClose} title={item ? "재고 수정" : "재고 추가"}>
      <form
        className="flex flex-col gap-4"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div>
          <span className="label">식재료</span>
          <IngredientPicker ingredients={ingredients} value={picked.name} onChange={setPicked} autoFocus={!item && !preset} />
          {!picked.id && picked.name.trim() && (
            <div className="mt-2">
              <span className="label">새 식재료 분류</span>
              <select className="field" value={category} onChange={(e) => setCategory(e.target.value)}>
                {CATEGORIES.map((c) => (
                  <option key={c}>{c}</option>
                ))}
              </select>
            </div>
          )}
        </div>

        <div>
          <span className="label">얼마나 있나요?</span>
          <Segmented
            value={tracking}
            onChange={setTracking}
            options={[
              { value: "quantity", label: "수량으로" },
              { value: "level", label: "대략으로" },
            ]}
          />
          <div className="mt-2">
            {tracking === "quantity" ? (
              <div className="flex gap-2">
                <input
                  className="field"
                  type="number"
                  inputMode="decimal"
                  step="any"
                  min="0"
                  value={quantity}
                  onChange={(e) => setQuantity(e.target.value)}
                  required
                />
                <input className="field max-w-28" list="units" value={unit} onChange={(e) => setUnit(e.target.value)} required />
                <datalist id="units">
                  {UNITS.map((u) => (
                    <option key={u} value={u} />
                  ))}
                </datalist>
              </div>
            ) : (
              <LevelPicker value={level} onChange={setLevel} />
            )}
          </div>
        </div>

        <div>
          <span className="label">보관</span>
          <Segmented value={storage} onChange={setStorage} options={STORAGES.map((s) => ({ value: s, label: STORAGE_LABEL[s] }))} />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <label>
            <span className="label">구매일</span>
            <input className="field" type="date" value={purchasedAt} onChange={(e) => setPurchasedAt(e.target.value)} />
          </label>
          <label>
            <span className="label">유통기한</span>
            <input className="field" type="date" value={expiresAt} onChange={(e) => setExpiresAt(e.target.value)} />
          </label>
        </div>
        <div className="-mt-2 flex flex-wrap gap-1.5">
          {[3, 7, 14, 30].map((d) => (
            <button key={d} type="button" className="chip border-line text-xs text-muted" onClick={() => quickExpiry(d)}>
              +{d}일
            </button>
          ))}
          {expiresAt && (
            <button type="button" className="chip border-line text-xs text-muted" onClick={() => setExpiresAt("")}>
              기한 없음
            </button>
          )}
        </div>

        <PhotoField value={photoId} onChange={setPhotoId} />

        <label>
          <span className="label">메모</span>
          <input className="field" value={note} onChange={(e) => setNote(e.target.value)} placeholder="예: 반 개는 손질해서 냉동" />
        </label>

        <ErrorText error={save.error ?? remove.error} />
        <div className="flex gap-2">
          {item && (
            <button
              type="button"
              className="btn-danger"
              disabled={remove.isPending}
              onClick={() => {
                if (window.confirm(`${item.ingredient_name} 재고를 삭제할까요?`)) remove.mutate();
              }}
            >
              삭제
            </button>
          )}
          <button className="btn-primary flex-1" disabled={save.isPending}>
            {save.isPending ? "저장 중…" : "저장"}
          </button>
        </div>
      </form>
    </Sheet>
  );
}

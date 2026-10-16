import { useMemo, useState } from "react";
import type { IngredientWithStock } from "../types";
import { StatusDot } from "./ui";

export type Picked = { id: string; name: string } | { id: null; name: string };

/** 식재료 검색 + 없으면 새 이름으로 선택 */
export function IngredientPicker({
  ingredients,
  value,
  onChange,
  placeholder = "식재료 검색 또는 새로 입력",
  autoFocus,
}: {
  ingredients: IngredientWithStock[];
  value: string;
  onChange: (p: Picked) => void;
  placeholder?: string;
  autoFocus?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const q = value.trim();
  const matches = useMemo(() => {
    if (!q) return ingredients.slice(0, 8);
    return ingredients.filter((i) => i.name.includes(q)).slice(0, 8);
  }, [ingredients, q]);
  const exact = ingredients.find((i) => i.name === q);
  const parentName = (id: string | null) => (id ? ingredients.find((i) => i.id === id)?.name : undefined);

  return (
    <div className="relative">
      <input
        className="field"
        value={value}
        placeholder={placeholder}
        autoFocus={autoFocus}
        onFocus={() => setOpen(true)}
        onBlur={() => setTimeout(() => setOpen(false), 150)}
        onChange={(e) => {
          const name = e.target.value;
          const hit = ingredients.find((i) => i.name === name.trim());
          onChange(hit ? { id: hit.id, name } : { id: null, name });
          setOpen(true);
        }}
      />
      {open && (matches.length > 0 || (q && !exact)) && (
        <ul className="absolute z-10 mt-1 max-h-64 w-full overflow-y-auto rounded-xl border border-line bg-card py-1 shadow-lg">
          {matches.map((i) => (
            <li key={i.id}>
              <button
                type="button"
                className="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-accent-soft/60"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  onChange({ id: i.id, name: i.name });
                  setOpen(false);
                }}
              >
                <StatusDot status={i.status} />
                <span className="flex-1">
                  {i.name}
                  {i.children.length > 0 && <span className="ml-1.5 rounded bg-accent-soft px-1 text-[11px] text-accent">묶음</span>}
                </span>
                <span className="text-xs text-muted">{parentName(i.parent_id) ?? i.category}</span>
              </button>
            </li>
          ))}
          {q && !exact && (
            <li>
              <button
                type="button"
                className="w-full px-3 py-2 text-left font-semibold text-accent hover:bg-accent-soft/60"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  onChange({ id: null, name: q });
                  setOpen(false);
                }}
              >
                + '{q}' 새 식재료
              </button>
            </li>
          )}
        </ul>
      )}
    </div>
  );
}

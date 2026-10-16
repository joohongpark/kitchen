import { useEffect, useRef, useState, type ReactNode } from "react";
import { api, photoUrl } from "../api";
import type { Level, StockStatus } from "../types";
import { LEVEL_LABEL, LEVELS } from "../types";
import { cx, daysUntil, ddayLabel } from "../util";

export function Sheet({ open, onClose, title, children }: { open: boolean; onClose: () => void; title: string; children: ReactNode }) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    document.addEventListener("keydown", onKey);
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", onKey);
      document.body.style.overflow = "";
    };
  }, [open, onClose]);
  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-ink/40 sm:items-center" onClick={onClose}>
      <div
        role="dialog"
        aria-label={title}
        className="max-h-[92dvh] w-full overflow-y-auto rounded-t-3xl bg-paper px-4 pt-3 pb-[calc(1.25rem+env(safe-area-inset-bottom))] sm:max-w-lg sm:rounded-3xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mx-auto mb-3 h-1 w-10 rounded-full bg-line sm:hidden" />
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-bold">{title}</h2>
          <button className="rounded-full p-2 text-muted hover:bg-line/50" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex rounded-xl border border-line bg-card p-1">
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          onClick={() => onChange(o.value)}
          className={cx(
            "flex-1 rounded-lg px-2 py-1.5 text-sm font-semibold transition",
            value === o.value ? "bg-accent text-white" : "text-muted hover:text-ink",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function LevelPicker({ value, onChange }: { value: Level; onChange: (l: Level) => void }) {
  return <Segmented value={value} onChange={onChange} options={LEVELS.map((l) => ({ value: l, label: LEVEL_LABEL[l] }))} />;
}

const STATUS_STYLE: Record<StockStatus, string> = {
  ok: "bg-ok",
  low: "bg-warn",
  none: "bg-line",
};

export function StatusDot({ status }: { status: StockStatus }) {
  const label = { ok: "있음", low: "부족", none: "없음" }[status];
  return <span title={label} aria-label={label} className={cx("inline-block size-2.5 shrink-0 rounded-full", STATUS_STYLE[status])} />;
}

export function ExpiryBadge({ date }: { date: string | null }) {
  const days = daysUntil(date);
  if (days == null) return null;
  const tone = days < 0 ? "bg-bad-soft text-bad" : days <= 3 ? "bg-warn-soft text-warn" : "bg-line/60 text-muted";
  return <span className={cx("rounded-full px-2 py-0.5 text-xs font-semibold", tone)}>{ddayLabel(days)}</span>;
}

export function Fab({ onClick, label }: { onClick: () => void; label: string }) {
  return (
    <button
      onClick={onClick}
      className="fixed right-4 bottom-[calc(5rem+env(safe-area-inset-bottom))] z-30 rounded-full bg-accent px-5 py-3.5 font-bold text-white shadow-lg shadow-accent/30 active:scale-95"
    >
      + {label}
    </button>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return <div className="card px-4 py-10 text-center text-muted">{children}</div>;
}

export function ErrorText({ error }: { error: unknown }) {
  if (!error) return null;
  return <p className="rounded-xl bg-bad-soft px-3 py-2 text-sm text-bad">{error instanceof Error ? error.message : String(error)}</p>;
}

/** 사진 선택/촬영 → 업로드 → photo_id 반환 */
export function PhotoField({ value, onChange, label = "사진" }: { value: string | null; onChange: (id: string | null) => void; label?: string }) {
  const input = useRef<HTMLInputElement>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<unknown>(null);

  async function pick(file: File | undefined) {
    if (!file) return;
    setBusy(true);
    setErr(null);
    try {
      onChange((await api.uploadPhoto(file)).id);
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
      if (input.current) input.current.value = "";
    }
  }

  return (
    <div>
      <span className="label">{label}</span>
      <div className="flex items-center gap-3">
        {value ? (
          <img src={photoUrl(value)} alt="" className="size-20 rounded-xl border border-line object-cover" />
        ) : (
          <div className="grid size-20 place-items-center rounded-xl border border-dashed border-line text-2xl text-muted">📷</div>
        )}
        <div className="flex flex-col gap-1.5">
          <button type="button" className="btn-ghost py-1.5 text-sm" disabled={busy} onClick={() => input.current?.click()}>
            {busy ? "올리는 중…" : value ? "바꾸기" : "사진 추가"}
          </button>
          {value && (
            <button type="button" className="text-sm text-muted underline" onClick={() => onChange(null)}>
              지우기
            </button>
          )}
        </div>
        <input ref={input} type="file" accept="image/*" hidden onChange={(e) => pick(e.target.files?.[0])} />
      </div>
      <ErrorText error={err} />
    </div>
  );
}

export function Thumb({ id, className }: { id: string | null; className?: string }) {
  if (!id) return <div className={cx("grid place-items-center bg-accent-soft text-2xl", className)}>🍲</div>;
  return <img src={photoUrl(id)} alt="" loading="lazy" className={cx("object-cover", className)} />;
}

export function PageHeader({ title, right }: { title: string; right?: ReactNode }) {
  return (
    <header className="sticky top-0 z-20 -mx-4 mb-4 flex items-center justify-between bg-paper/90 px-4 pt-[calc(0.75rem+env(safe-area-inset-top))] pb-3 backdrop-blur">
      <h1 className="text-2xl font-extrabold tracking-tight">{title}</h1>
      {right}
    </header>
  );
}

/** 오늘 기준 남은 일수. 날짜가 없으면 null */
export function daysUntil(date: string | null): number | null {
  if (!date) return null;
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const d = new Date(`${date}T00:00:00`);
  return Math.round((d.getTime() - today.getTime()) / 86_400_000);
}

export function ddayLabel(days: number): string {
  if (days < 0) return `${-days}일 지남`;
  if (days === 0) return "오늘까지";
  return `D-${days}`;
}

export function formatQty(q: number | null | undefined): string {
  if (q == null) return "";
  return Number.isInteger(q) ? String(q) : String(Math.round(q * 100) / 100);
}

export function todayStr(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

export function relativeDate(iso: string | null): string {
  if (!iso) return "";
  const d = new Date(iso);
  const days = Math.floor((Date.now() - d.getTime()) / 86_400_000);
  if (days <= 0) return "오늘";
  if (days === 1) return "어제";
  if (days < 30) return `${days}일 전`;
  return d.toLocaleDateString("ko-KR", { month: "short", day: "numeric" });
}

export function cx(...c: (string | false | null | undefined)[]): string {
  return c.filter(Boolean).join(" ");
}

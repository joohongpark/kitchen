import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, photoUrl } from "../api";
import { CookSheet } from "../components/CookSheet";
import { ErrorText, StatusDot, Thumb, usePhotoPicker } from "../components/ui";
import { relativeDate } from "../util";

export function RecipeDetailPage() {
  const { id = "" } = useParams();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { data: r, isLoading, error } = useQuery({ queryKey: ["recipe", id], queryFn: () => api.recipe(id) });
  const [cooking, setCooking] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  const setPhoto = useMutation({
    mutationFn: (photoId: string) => api.setRecipePhoto(id, photoId),
    onSuccess: () => qc.invalidateQueries(),
  });
  const photo = usePhotoPicker((photoId) => setPhoto.mutate(photoId));

  // 대체 재료로 채울 수 있는 줄은 사지 않아도 된다
  const missing = r?.ingredients.filter((i) => !i.optional && i.status === "none" && !i.substitute) ?? [];

  const addMissing = useMutation({
    mutationFn: async () => {
      for (const m of missing) await api.addShopping({ ingredient_id: m.ingredient_id, amount: m.amount_label || undefined });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["shopping"] });
      setToast(`장보기에 ${missing.length}개 추가했어요`);
    },
  });

  const remove = useMutation({
    mutationFn: () => api.deleteRecipe(id),
    onSuccess: () => {
      qc.invalidateQueries();
      navigate("/recipes", { replace: true });
    },
  });

  if (isLoading) return <p className="pt-8 text-muted">불러오는 중…</p>;
  if (error || !r) return <ErrorText error={error ?? "레시피가 없습니다"} />;

  return (
    <div className="-mx-4">
      <div className="relative">
        <button type="button" className="block w-full" onClick={photo.open} disabled={photo.busy || setPhoto.isPending}>
          <Thumb id={r.photo_id} className="aspect-[4/3] max-h-80 w-full text-6xl" />
          <span className="absolute right-3 bottom-3 rounded-full bg-card/90 px-3 py-1.5 text-sm font-semibold shadow">
            {photo.busy || setPhoto.isPending ? "올리는 중…" : r.photo_id ? "📷 사진 바꾸기" : "📷 사진 추가"}
          </span>
        </button>
        {photo.element}
        <button
          onClick={() => navigate(-1)}
          className="absolute top-[calc(0.75rem+env(safe-area-inset-top))] left-3 grid size-10 place-items-center rounded-full bg-card/90 text-lg shadow"
          aria-label="뒤로"
        >
          ←
        </button>
      </div>

      <div className="px-4 pt-4">
        <div className="flex items-start justify-between gap-3">
          <h1 className="text-2xl font-extrabold">{r.title}</h1>
          <Link to={`/recipes/${r.id}/edit`} className="btn-ghost shrink-0 px-3 py-1.5 text-sm">
            수정
          </Link>
        </div>
        <p className="mt-1 text-sm text-muted">
          {[r.servings && `${r.servings}인분`, r.cook_minutes && `${r.cook_minutes}분`, r.cook_count > 0 && `${r.cook_count}번 요리 · 마지막 ${relativeDate(r.last_cooked_at)}`]
            .filter(Boolean)
            .join(" · ")}
        </p>
        {r.description && <p className="mt-3 leading-relaxed">{r.description}</p>}
        {r.tags.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-1">
            {r.tags.map((t) => (
              <span key={t} className="rounded-full bg-accent-soft px-2.5 py-0.5 text-sm text-accent">
                {t}
              </span>
            ))}
          </div>
        )}

        <section className="mt-6">
          <div className="mb-2 flex items-center justify-between">
            <h2 className="text-lg font-bold">재료</h2>
            {missing.length > 0 && (
              <button className="text-sm font-semibold text-accent" disabled={addMissing.isPending} onClick={() => addMissing.mutate()}>
                없는 재료 {missing.length}개 장보기에 추가
              </button>
            )}
          </div>
          {toast && <p className="mb-2 rounded-xl bg-ok-soft px-3 py-2 text-sm text-ok">{toast}</p>}
          <ul className="card divide-y divide-line">
            {r.ingredients.map((i) => (
              <li key={i.ingredient_id} className="px-3 py-2.5">
                <div className="flex items-center gap-2.5">
                  <StatusDot status={i.status} />
                  <span className="flex-1">
                    {i.ingredient_name}
                    {i.optional && <span className="ml-1 text-xs text-muted">(선택)</span>}
                    {i.note && <span className="ml-1 text-xs text-muted">· {i.note}</span>}
                  </span>
                  <span className="text-right text-sm">
                    <span className="font-semibold">{i.amount_label}</span>
                    {i.stock_summary && <span className="block text-xs text-muted">재고 {i.stock_summary}</span>}
                  </span>
                </div>
                {i.alternatives.length > 0 && (
                  <div className="mt-1 ml-5 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted">
                    {i.substitute ? (
                      <span className="font-semibold text-ok">→ {i.substitute}(으)로 대체 가능</span>
                    ) : (
                      <span>대체:</span>
                    )}
                    {i.alternatives
                      .filter((a) => a.name !== i.substitute)
                      .map((a) => (
                        <span key={a.ingredient_id} className="inline-flex items-center gap-1" title={a.note ?? undefined}>
                          <StatusDot status={a.status} />
                          {a.name}
                          {a.note && <span className="text-muted">({a.note})</span>}
                        </span>
                      ))}
                  </div>
                )}
              </li>
            ))}
          </ul>
        </section>

        <section className="mt-6">
          <h2 className="mb-2 text-lg font-bold">만드는 법</h2>
          <ol className="flex flex-col gap-3">
            {r.steps.map((s, idx) => (
              <li key={s.position} className="flex gap-3">
                <span className="grid size-7 shrink-0 place-items-center rounded-full bg-accent text-sm font-bold text-white">{idx + 1}</span>
                <div className="flex-1 pt-0.5">
                  <p className="leading-relaxed">{s.body}</p>
                  {s.photo_id && <img src={photoUrl(s.photo_id)} alt="" className="mt-2 max-h-60 rounded-xl object-cover" />}
                </div>
              </li>
            ))}
          </ol>
        </section>

        <ErrorText error={addMissing.error ?? remove.error ?? setPhoto.error ?? photo.error} />
        <button
          className="mt-8 mb-4 text-sm text-bad underline"
          onClick={() => window.confirm(`'${r.title}' 레시피를 삭제할까요?`) && remove.mutate()}
        >
          레시피 삭제
        </button>
      </div>

      <div className="sticky bottom-[calc(4.5rem+env(safe-area-inset-bottom))] px-4 pb-2">
        <button className="btn-primary w-full py-3.5 text-base shadow-lg shadow-accent/25" onClick={() => setCooking(true)}>
          🍳 요리했어요
        </button>
      </div>

      <CookSheet
        open={cooking}
        recipe={r}
        onClose={() => setCooking(false)}
        onDone={(msg) => {
          setCooking(false);
          setToast(msg);
        }}
      />
    </div>
  );
}

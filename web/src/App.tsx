import { useEffect } from "react";
import { BrowserRouter, NavLink, Route, Routes } from "react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./api";
import { cx } from "./util";
import { IngredientsPage } from "./pages/IngredientsPage";
import { LoginPage } from "./pages/LoginPage";
import { MorePage } from "./pages/MorePage";
import { PantryPage } from "./pages/PantryPage";
import { RecipeDetailPage } from "./pages/RecipeDetailPage";
import { RecipeFormPage } from "./pages/RecipeFormPage";
import { RecipesPage } from "./pages/RecipesPage";
import { RecommendPage } from "./pages/RecommendPage";
import { ShoppingPage } from "./pages/ShoppingPage";

const TABS = [
  { to: "/", label: "재고", icon: "🧊", end: true },
  { to: "/recipes", label: "레시피", icon: "📖" },
  { to: "/cook", label: "뭐 해먹지", icon: "🍳" },
  { to: "/shopping", label: "장보기", icon: "🛒" },
  { to: "/more", label: "더보기", icon: "⋯" },
];

export function App() {
  const qc = useQueryClient();
  const { data: me, isLoading } = useQuery({ queryKey: ["me"], queryFn: api.me, retry: false });

  useEffect(() => {
    const onUnauthorized = () => qc.invalidateQueries({ queryKey: ["me"] });
    window.addEventListener("kitchen:unauthorized", onUnauthorized);
    return () => window.removeEventListener("kitchen:unauthorized", onUnauthorized);
  }, [qc]);

  if (isLoading) return null;
  if (me && !me.authenticated) return <LoginPage />;

  return (
    <BrowserRouter>
      <main className="mx-auto max-w-2xl px-4 pb-[calc(6rem+env(safe-area-inset-bottom))]">
        <Routes>
          <Route path="/" element={<PantryPage />} />
          <Route path="/recipes" element={<RecipesPage />} />
          <Route path="/recipes/new" element={<RecipeFormPage />} />
          <Route path="/recipes/:id" element={<RecipeDetailPage />} />
          <Route path="/recipes/:id/edit" element={<RecipeFormPage />} />
          <Route path="/cook" element={<RecommendPage />} />
          <Route path="/shopping" element={<ShoppingPage />} />
          <Route path="/more" element={<MorePage />} />
          <Route path="/ingredients" element={<IngredientsPage />} />
          <Route path="*" element={<p className="pt-10 text-center text-muted">페이지가 없어요.</p>} />
        </Routes>
      </main>
      <nav className="fixed inset-x-0 bottom-0 z-40 border-t border-line bg-paper/95 pb-[env(safe-area-inset-bottom)] backdrop-blur">
        <ul className="mx-auto flex max-w-2xl">
          {TABS.map((t) => (
            <li key={t.to} className="flex-1">
              <NavLink
                to={t.to}
                end={t.end}
                className={({ isActive }) =>
                  cx("flex flex-col items-center gap-0.5 py-2 text-[11px] font-semibold", isActive ? "text-accent" : "text-muted")
                }
              >
                <span className="text-xl leading-none">{t.icon}</span>
                {t.label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>
    </BrowserRouter>
  );
}

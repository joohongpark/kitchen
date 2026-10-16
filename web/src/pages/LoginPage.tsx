import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import { ErrorText } from "../components/ui";

export function LoginPage() {
  const qc = useQueryClient();
  const [password, setPassword] = useState("");
  const login = useMutation({ mutationFn: () => api.login(password), onSuccess: () => qc.invalidateQueries() });

  return (
    <div className="grid min-h-dvh place-items-center px-6">
      <form
        className="w-full max-w-xs text-center"
        onSubmit={(e) => {
          e.preventDefault();
          login.mutate();
        }}
      >
        <img src="/icon.svg" alt="" className="mx-auto mb-4 size-16" />
        <h1 className="mb-6 text-2xl font-extrabold">부엌</h1>
        <input
          className="field mb-3 text-center"
          type="password"
          placeholder="비밀번호"
          autoFocus
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <ErrorText error={login.error} />
        <button className="btn-primary mt-3 w-full" disabled={login.isPending}>
          들어가기
        </button>
      </form>
    </div>
  );
}

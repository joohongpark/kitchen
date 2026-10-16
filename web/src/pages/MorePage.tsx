import { Link } from "react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import { ErrorText, PageHeader } from "../components/ui";

export function MorePage() {
  const qc = useQueryClient();
  const { data: info } = useQuery({ queryKey: ["info"], queryFn: api.info });
  const { data: me } = useQuery({ queryKey: ["me"], queryFn: api.me });
  const test = useMutation({ mutationFn: api.notifyTest });
  const logout = useMutation({ mutationFn: api.logout, onSuccess: () => qc.invalidateQueries() });

  return (
    <div>
      <PageHeader title="더보기" />
      <ul className="card divide-y divide-line">
        <li>
          <Link to="/ingredients" className="flex items-center justify-between px-4 py-3.5">
            <span className="font-semibold">🥬 식재료 관리</span>
            <span className="text-muted">›</span>
          </Link>
        </li>
      </ul>

      <section className="card mt-4 p-4">
        <h2 className="mb-1 font-bold">🔔 알림 (ntfy)</h2>
        {info?.notifications ? (
          <>
            <p className="mb-3 text-sm text-muted">
              매일 {info.notify_hour}시에 유통기한 {info.expiry_warn_days}일 이내 재료와 부족한 재료를 알려 드려요.
            </p>
            <button className="btn-ghost" disabled={test.isPending} onClick={() => test.mutate()}>
              테스트 알림 보내기
            </button>
            {test.isSuccess && <p className="mt-2 text-sm text-ok">보냈어요. 몇 초 안에 도착해요.</p>}
          </>
        ) : (
          <p className="text-sm text-muted">
            서버의 <code>.env</code> 에 <code>NTFY_TOPIC</code> 을 설정하면 켜져요.
          </p>
        )}
        <ErrorText error={test.error} />
      </section>

      {me?.auth_required && (
        <button className="btn-ghost mt-4 w-full" onClick={() => logout.mutate()}>
          로그아웃
        </button>
      )}
    </div>
  );
}

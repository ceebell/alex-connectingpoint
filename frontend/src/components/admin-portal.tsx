"use client";
import { useCallback, useEffect, useRef, useState } from "react";
import { api, ApiError, errorText } from "@/lib/api";
import { AdminSession, canManageTeam, canUseWorkspace, invitationToken, lineLoginErrors, TeamInvitation } from "@/lib/team-api";
import AdminWorkspace from "./admin-workspace";
import TeamWorkspace from "./team-workspace";
import TeamWelcome from "./team-welcome";

export default function AdminPortal() {
  const [session, setSession] = useState<AdminSession | null>(null);
  const [invitation, setInvitation] = useState<TeamInvitation | null>(null);
  const [inviteError, setInviteError] = useState("");
  const [checking, setChecking] = useState(true);
  const [teamView, setTeamView] = useState(false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [initialHash] = useState(() => typeof window === "undefined" ? "" : window.location.hash);
  const [loginError] = useState(() => { const code = typeof window === "undefined" ? null : new URLSearchParams(window.location.search).get("login_error"); return code ? lineLoginErrors[code] || "เข้าสู่ระบบไม่สำเร็จ กรุณาลองใหม่" : ""; });
  const sessionRequest = useRef(0);
  const refreshSession = useCallback(async () => {
    const request = ++sessionRequest.current;
    try { const value = await api<AdminSession>("/api/admin/session"); if (request === sessionRequest.current) { setSession(value); setError(""); } }
    catch (cause) {
      if (request !== sessionRequest.current) return;
      if (cause instanceof ApiError && [401, 403].includes(cause.status)) setSession(null);
      else setError(errorText(cause));
    }
  }, []);
  useEffect(() => {
    let cancelled = false;
    async function start() {
      const rawHash = initialHash;
      const token = invitationToken(rawHash);
      setTeamView(rawHash === "#team");
      // Invitation secrets stay in the fragment, never in HTTP URLs, query logs or referrers.
      if (rawHash.includes("invite=")) window.history.replaceState(null, "", window.location.pathname + window.location.search);
      if (rawHash.includes("invite=") && !token) {
        setInviteError("ลิงก์คำเชิญไม่ถูกต้อง กรุณาขอลิงก์ใหม่จากเจ้าของร้าน");
        await refreshSession();
        if (!cancelled) setChecking(false);
        return;
      }
      try {
        const value = await api<{ invitation: TeamInvitation | null }>(token ? "/api/admin/invite/open" : "/api/admin/invite", token ? { method: "POST", body: JSON.stringify({ token }) } : undefined);
        if (!cancelled) setInvitation(value.invitation);
      } catch (cause) { if (!cancelled) setInviteError(errorText(cause)); }
      if (cancelled) return;
      await refreshSession();
      if (!cancelled) setChecking(false);
    }
    void start();
    const navigate = () => {
      const hash = window.location.hash;
      setTeamView(hash === "#team");
      if (hash.includes("invite=")) {
        const token = invitationToken(hash);
        window.history.replaceState(null, "", window.location.pathname + window.location.search);
        if (!token) { setInviteError("ลิงก์คำเชิญไม่ถูกต้อง กรุณาขอลิงก์ใหม่จากเจ้าของร้าน"); return; }
        setChecking(true);
        void api<{ invitation: TeamInvitation | null }>("/api/admin/invite/open", { method: "POST", body: JSON.stringify({ token }) }).then(value => { if (!cancelled) { setInvitation(value.invitation); setInviteError(""); } }).catch(cause => { if (!cancelled) setInviteError(errorText(cause)); }).finally(() => { if (!cancelled) setChecking(false); });
      }
    };
    window.addEventListener("hashchange", navigate);
    return () => { cancelled = true; window.removeEventListener("hashchange", navigate); };
  }, [refreshSession, initialHash]);
  useEffect(() => {
    let running = false;
    const resume = () => {
      if (document.hidden || running) return;
      running = true;
      void refreshSession().finally(() => { running = false; });
    };
    const timer = setInterval(resume, 10000);
    window.addEventListener("focus", resume);
    document.addEventListener("visibilitychange", resume);
    return () => { clearInterval(timer); window.removeEventListener("focus", resume); document.removeEventListener("visibilitychange", resume); };
  }, [refreshSession]);
  async function clearInvitation() {
    setBusy(true); setError("");
    try { await api("/api/admin/invite/clear", { method: "POST", body: "{}" }); setInvitation(null); setInviteError(""); }
    catch (cause) { setError(errorText(cause)); }
    finally { setBusy(false); }
  }
  async function accept() {
    if (!invitation) return;
    setBusy(true); setError("");
    try { await api("/api/admin/invite/accept", { method: "POST", body: JSON.stringify({ invitation_id: invitation.id, expected_version: invitation.version }) }); setInvitation(null); await refreshSession(); }
    catch (cause) { setError(errorText(cause)); }
    finally { setBusy(false); }
  }
  async function logout(switchAccount = false) {
    setBusy(true); setError("");
    try {
      await api("/api/admin/session", { method: "DELETE" }); setSession(null);
      // OAuth starts on the backend and requires a top-level redirect through LINE.
      // eslint-disable-next-line @next/next/no-location-assign-relative-destination
      if (switchAccount) window.location.assign("/api/admin/line/start?switch_account=true");
    } catch (cause) { setError(errorText(cause)); }
    finally { setBusy(false); }
  }
  if (checking) return <main className="login-page"><p role="status">กำลังตรวจสอบบัญชี…</p></main>;
  if (invitation || inviteError || (session?.team_enabled && !canUseWorkspace(session))) return <TeamWelcome session={session} invitation={invitation} error={error || inviteError || loginError} busy={busy} onAccept={accept} onClear={clearInvitation} onRefresh={refreshSession} onLogout={logout} />;
  if (teamView && session) {
    if (!canManageTeam(session)) return <main className="team-access-message"><h1>บัญชีนี้ไม่มีสิทธิ์จัดการทีม</h1><a className="button secondary" href="#">กลับพื้นที่ทำงาน</a></main>;
    return <TeamWorkspace session={session} onLogout={() => logout()} onSessionChange={refreshSession} />;
  }
  return <AdminWorkspace key={session?.membership ? `${session.membership.id}:${session.membership.version}` : "legacy"} access={session} />;
}

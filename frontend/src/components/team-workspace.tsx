"use client";
/* eslint-disable @next/next/no-img-element -- LINE avatars and locally generated invitation QR images. */
import { FormEvent, ReactNode, useEffect, useRef, useState } from "react";
import { api, ApiError, errorText } from "@/lib/api";
import { AdminSession, invitationUrl, TeamBranch, TeamMember, TeamRole, TeamStatus, teamRoles, teamStatuses } from "@/lib/team-api";
import { formatDateTime } from "@/lib/dates";
import WorkspaceSidebar from "./workspace-sidebar";
import { useChatNotifications } from "@/lib/use-chat-notifications";
import type { Inquiry } from "@/lib/api";

type Props = { session: AdminSession; onLogout: () => Promise<void>; onSessionChange: () => Promise<void> };
type Audit = { action: string; actor_name: string; actor_id: string; at: string; version: number; changes: { name?: string; role?: TeamRole; status?: TeamStatus; branches?: TeamBranch[]; note?: string } };
const actionNames: Record<string, string> = { INVITED: "สร้างคำเชิญ", ACCEPTED: "ยืนยัน LINE และรับคำเชิญ", APPROVED: "อนุมัติให้เข้าร้าน", REJECTED: "ปฏิเสธคำขอ", CANCELLED: "ยกเลิกคำเชิญ", UPDATED: "แก้ไขข้อมูลหรือสิทธิ์", MIGRATED_OWNER: "ย้ายเจ้าของร้านจากสิทธิ์เดิม" };
const merge = (current: TeamMember[], incoming: TeamMember[]) => [...new Map([...current, ...incoming].map(member => [member.id, member])).values()].sort((a, b) => b.id.localeCompare(a.id));

function useTeamViewport() {
  useEffect(() => {
    const viewport = window.visualViewport;
    if (!viewport) return;
    const root = document.documentElement;
    const mobile = window.matchMedia("(max-width: 767px), (max-width: 1024px) and (max-height: 600px)");
    let frame = 0;
    let blurFrame = 0;
    const editable = 'input:not([type="checkbox"]):not([type="radio"]), textarea, select';
    const dismiss = (event: PointerEvent) => {
      if (!mobile.matches || !(event.target instanceof Element) || event.target.closest(editable) || event.target.closest("label")?.querySelector(editable)) return;
      const control = document.activeElement;
      if (!(control instanceof HTMLElement) || !control.matches(editable)) return;
      cancelAnimationFrame(blurFrame);
      blurFrame = requestAnimationFrame(() => { if (document.activeElement === control) control.blur(); });
    };
    const update = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        if (mobile.matches && Math.abs(viewport.scale - 1) < 0.01) {
          root.style.setProperty("--app-height", `${viewport.height}px`);
          root.style.setProperty("--app-top", `${viewport.offsetTop}px`);
        } else if (!mobile.matches) { root.style.removeProperty("--app-height"); root.style.removeProperty("--app-top"); }
      });
    };
    update(); viewport.addEventListener("resize", update); viewport.addEventListener("scroll", update); window.addEventListener("resize", update);
    document.addEventListener("pointerup", dismiss);
    return () => { cancelAnimationFrame(frame); cancelAnimationFrame(blurFrame); document.removeEventListener("pointerup", dismiss); viewport.removeEventListener("resize", update); viewport.removeEventListener("scroll", update); window.removeEventListener("resize", update); root.style.removeProperty("--app-height"); root.style.removeProperty("--app-top"); };
  }, []);
}

function Dialog({ title, busy, onClose, children }: { title: string; busy?: boolean; onClose: () => void; children: ReactNode }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => { ref.current?.showModal(); }, []);
  return <dialog ref={ref} className="team-dialog" aria-label={title} onCancel={event => { if (busy) event.preventDefault(); else { event.preventDefault(); onClose(); } }}><div className="team-dialog-heading"><h2>{title}</h2><button type="button" className="text-button" disabled={busy} onClick={onClose}>ปิด</button></div><div className="team-dialog-body">{children}</div></dialog>;
}
function MemberEditor({ member, branches, branchesError, onSaved, onClose }: { member: TeamMember | null; branches: TeamBranch[]; branchesError: string; onSaved: (member: TeamMember, token?: string) => void; onClose: () => void }) {
  const [name, setName] = useState(member?.name ?? "");
  const [role, setRole] = useState<TeamRole>(member?.role ?? "STAFF");
  const [selected, setSelected] = useState(member?.branch_ids ?? (branches.length === 1 ? [branches[0].id] : []));
  const [note, setNote] = useState(member?.note ?? "");
  const [status, setStatus] = useState(member?.status ?? "INVITED");
  const [version, setVersion] = useState(member?.version ?? 0);
  const [requestId] = useState(() => crypto.randomUUID());
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  async function save(event: FormEvent) {
    event.preventDefault(); setBusy(true); setError("");
    try {
      const value = await api<{ member: TeamMember; token?: string }>(member ? `/api/admin/team/${member.id}/update` : "/api/admin/team/invitations", { method: "POST", body: JSON.stringify({ name: name.trim(), role, branch_ids: selected, note: note.trim(), ...(member ? { expected_version: version, status } : { request_id: requestId }) }) });
      onSaved(value.member, value.token);
    } catch (cause) {
      if (member && cause instanceof ApiError && cause.code === "RECORD_CONFLICT") {
        try { const latest = await api<{ member: TeamMember }>(`/api/admin/team/${member.id}`); setVersion(latest.member.version); setError("ข้อมูลคนนี้ถูกแก้แล้ว ข้อความที่คุณกรอกยังอยู่ กรุณาตรวจข้อมูลและบันทึกอีกครั้ง"); }
        catch { setError(errorText(cause)); }
      } else setError(errorText(cause));
    } finally { setBusy(false); }
  }
  const unknown = selected.filter(id => !branches.some(branch => branch.id === id));
  return <Dialog title={member ? `แก้ไขข้อมูล ${member.name}` : "เพิ่มพนักงาน"} busy={busy} onClose={onClose}><form className="team-member-form" autoComplete="off" onSubmit={save}>
    <div className="team-shop-label">ร้าน Arisa Rental</div>
    <label>ชื่อที่ใช้ทำงาน<input autoComplete="off" autoFocus required maxLength={100} value={name} disabled={busy} placeholder="เช่น พิม" onChange={event => setName(event.target.value)} /></label>
    <label>บทบาท<select autoComplete="off" value={role} disabled={busy} onChange={event => setRole(event.target.value as TeamRole)}><option value="STAFF">พนักงาน</option><option value="MANAGER">ผู้จัดการ</option></select></label>
    <p className="team-role-description">{role === "STAFF" ? "อ่านและตอบแชท เพิ่ม Note เตรียมร่างและเช็คคิว แล้วให้ผู้จัดการยืนยันออเดอร์" : "อ่านและตอบแชท จัดการ Note เตรียมร่าง เช็คคิว และยืนยันส่งออเดอร์เข้า Alex"}</p>
    <fieldset className="team-branch-picker" disabled={busy}><legend>สาขาที่ทำงาน</legend>{branches.length > 1 && <p className="muted small">เลือกอย่างน้อย 1 สาขา</p>}{branchesError ? <p className="notice warning" role="alert">{branchesError}</p> : branches.length ? branches.map(branch => <label className="checkbox-row" key={branch.id}><input type="checkbox" checked={selected.includes(branch.id)} onChange={event => setSelected(current => event.target.checked ? [...current, branch.id] : current.filter(id => id !== branch.id))} />{branch.name}</label>) : <p className="muted small">ยังไม่มีสาขาสินค้าที่เปิดใช้งาน สามารถเพิ่มพนักงานเพื่อจัดการแชทก่อนได้</p>}{unknown.length > 0 && <p className="notice warning">มีสาขาเดิมที่ร้านไม่มีสิทธิ์แล้ว กรุณาเลือกสาขาปัจจุบันก่อนบันทึก<button type="button" className="text-button" onClick={() => setSelected(current => current.filter(id => !unknown.includes(id)))}>นำสาขาที่ใช้ไม่ได้ออก</button></p>}</fieldset>
    {member && member.status !== "PENDING" && <label>สถานะ<select autoComplete="off" disabled={busy} value={status} onChange={event => setStatus(event.target.value as TeamStatus)}><option value="ACTIVE">ใช้งานอยู่</option><option value="SUSPENDED">ระงับสิทธิ์</option></select></label>}
    {status === "SUSPENDED" && <p className="notice warning">บัญชีนี้จะเข้าใช้ข้อมูลร้านไม่ได้ ประวัติงานเดิมยังคงอยู่</p>}
    <label>หมายเหตุ <span className="muted small">ไม่บังคับ</span><textarea autoComplete="off" rows={2} maxLength={1000} disabled={busy} value={note} onChange={event => setNote(event.target.value)} placeholder="เช่น แอดมินกะบ่าย" /></label>
    {!member && <p className="muted small">คำเชิญมีอายุ 24 ชั่วโมง พนักงานต้องยืนยัน LINE และรอเจ้าของร้านอนุมัติ</p>}
    {error && <p className="notice warning" role="alert">{error}</p>}
    <div className="team-form-actions"><button type="button" className="button secondary" disabled={busy} onClick={onClose}>ยกเลิก</button><button className="button primary" disabled={busy || !name.trim() || !!branchesError || !!unknown.length || (branches.length > 0 && !selected.length)}>{busy ? "กำลังบันทึก…" : member ? "บันทึกการเปลี่ยนแปลง" : "สร้างคำเชิญ"}</button></div>
  </form></Dialog>;
}
function InvitationLink({ member, token, onClose }: { member: TeamMember; token: string; onClose: () => void }) {
  const link = invitationUrl(window.location.origin, token);
  const [qr, setQr] = useState("");
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const textarea = useRef<HTMLTextAreaElement>(null);
  async function copy() {
    try { await navigator.clipboard.writeText(link); setMessage("คัดลอกลิงก์แล้ว ส่งให้พนักงานที่ต้องการเชิญได้เลย"); }
    catch { textarea.current?.focus(); textarea.current?.select(); setMessage("กรุณาคัดลอกลิงก์ที่เลือกไว้แล้วส่งให้พนักงาน"); }
  }
  async function showQr() {
    if (qr) { setQr(""); return; }
    setLoading(true); setMessage("");
    try { const generator = await import("qrcode"); setQr(await generator.toDataURL(link, { width: 280, margin: 2, errorCorrectionLevel: "M" })); }
    catch { setMessage("สร้าง QR ไม่สำเร็จ กรุณาคัดลอกลิงก์แทน"); }
    finally { setLoading(false); }
  }
  return <Dialog title="คำเชิญพร้อมส่ง" onClose={onClose}><p>ส่งคำเชิญนี้ให้ <strong>{member.name}</strong> เพื่อยืนยันบัญชี LINE</p><dl className="team-invite-summary"><div><dt>บทบาท</dt><dd>{teamRoles[member.role]}</dd></div><div><dt>รับคำเชิญภายใน</dt><dd>{formatDateTime(member.expires_at)}</dd></div></dl><label className="team-link-field">ลิงก์คำเชิญ<textarea autoComplete="off" ref={textarea} readOnly rows={3} value={link} /></label><div className="team-form-actions"><button type="button" className="button primary" onClick={() => void copy()}>คัดลอกลิงก์</button><button type="button" className="button secondary" disabled={loading} onClick={() => void showQr()}>{loading ? "กำลังสร้าง QR…" : qr ? "ซ่อน QR" : "แสดง QR"}</button></div>{qr && <img className="team-invitation-qr" src={qr} alt="QR สำหรับรับคำเชิญเข้าทีม Arisa Rental" />}{message && <p className="team-feedback" role="status">{message}</p>}<p className="muted small">เจ้าของร้านจะตรวจบัญชีและอนุมัติอีกครั้งก่อนพนักงานเข้าถึงข้อมูลร้าน</p></Dialog>;
}
function History({ member, onClose }: { member: TeamMember; onClose: () => void }) {
  const [events, setEvents] = useState<Audit[]>([]);
  const [next, setNext] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  useEffect(() => {
    let cancelled = false;
    api<{ events: Audit[]; next: number | null }>(`/api/admin/team/${member.id}/history`).then(value => { if (!cancelled) { setEvents(value.events); setNext(value.next); } }).catch(cause => { if (!cancelled) setError(errorText(cause)); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [member.id]);
  async function more() {
    setLoading(true); setError("");
    try { const value = await api<{ events: Audit[]; next: number | null }>(`/api/admin/team/${member.id}/history${next ? `?before_version=${next}` : ""}`); setEvents(current => [...current, ...value.events]); setNext(value.next); }
    catch (cause) { setError(errorText(cause)); }
    finally { setLoading(false); }
  }
  return <Dialog title={`ประวัติ ${member.name}`} onClose={onClose}>{error && <p className="notice warning" role="alert">{error}</p>}<ol className="team-history">{events.map(event => <li key={event.version}><strong>{actionNames[event.action] || event.action}</strong><p>{event.actor_name} · {formatDateTime(event.at)}</p>{event.changes.name && <p className="muted small">ชื่อที่ใช้ทำงาน: {event.changes.name}</p>}{event.changes.role && <p className="muted small">บทบาท: {teamRoles[event.changes.role]}</p>}{event.changes.status && <p className="muted small">สถานะ: {teamStatuses[event.changes.status]}</p>}{event.changes.branches?.length ? <p className="muted small">สาขา: {event.changes.branches.map(branch => branch.name).join(", ")}</p> : null}{event.changes.note && <p className="muted small">หมายเหตุ: {event.changes.note}</p>}</li>)}</ol>{loading && <p className="muted" role="status">กำลังโหลดประวัติ…</p>}{next && <button className="button secondary full" disabled={loading} onClick={() => void more()}>โหลดประวัติก่อนหน้า</button>}</Dialog>;
}

export default function TeamWorkspace({ session, onLogout, onSessionChange }: Props) {
  useTeamViewport();
  const { observeInbox, pollInBackground } = useChatNotifications(true, id => {
    const url = new URL(window.location.href); url.hash = ""; url.searchParams.set("inquiry", id);
    window.history.replaceState(null, "", url.pathname + url.search);
    window.dispatchEvent(new HashChangeEvent("hashchange"));
  }, () => false);
  useEffect(() => {
    let cancelled = false, polling = false;
    const resume = () => {
      if (polling || !pollInBackground()) return;
      polling = true;
      void api<Inquiry[]>("/api/inquiries").then(inquiries => { if (!cancelled) observeInbox(inquiries); }).catch(() => {}).finally(() => { polling = false; });
    };
    resume(); const timer = setInterval(resume, 10000);
    window.addEventListener("focus", resume);
    return () => { cancelled = true; clearInterval(timer); window.removeEventListener("focus", resume); };
  }, [observeInbox, pollInBackground]);
  const [members, setMembers] = useState<TeamMember[]>([]);
  const [next, setNext] = useState<string | null>(null);
  const hasPaginated = useRef(false);
  const [branches, setBranches] = useState<TeamBranch[]>([]);
  const [branchesError, setBranchesError] = useState("");
  const [branchesReady, setBranchesReady] = useState(false);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState("");
  const [filter, setFilter] = useState<TeamStatus | "ALL">("ALL");
  const [query, setQuery] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [editor, setEditor] = useState<TeamMember | "new" | null>(null);
  const [invitation, setInvitation] = useState<{ member: TeamMember; token: string } | null>(null);
  const [history, setHistory] = useState<TeamMember | null>(null);
  const [confirmation, setConfirmation] = useState<{ member: TeamMember; action: "approve" | "reject" | "cancel" } | null>(null);
  useEffect(() => {
    let cancelled = false;
    let polling = false;
    const resume = () => {
      if (polling || document.hidden) return;
      polling = true;
      void api<{ members: TeamMember[]; next: string | null }>("/api/admin/team").then(value => { if (!cancelled) { setMembers(current => merge(current, value.members)); if (!hasPaginated.current) setNext(value.next); } }).catch(cause => { if (!cancelled) setError(errorText(cause)); }).finally(() => { polling = false; if (!cancelled) setLoading(false); });
    };
    resume();
    const timer = setInterval(resume, 15000);
    window.addEventListener("focus", resume);
    document.addEventListener("visibilitychange", resume);
    api<{ branches: TeamBranch[] }>("/api/admin/team/branches").then(value => { if (!cancelled) { setBranches(value.branches); setBranchesError(""); } }).catch(cause => { if (!cancelled) setBranchesError(errorText(cause)); }).finally(() => { if (!cancelled) setBranchesReady(true); });
    return () => { cancelled = true; clearInterval(timer); window.removeEventListener("focus", resume); document.removeEventListener("visibilitychange", resume); };
  }, []);
  async function reload(more = false) {
    setLoading(true); setError("");
    try {
      const value = await api<{ members: TeamMember[]; next: string | null }>(`/api/admin/team${more && next ? `?before=${next}` : ""}`);
      setMembers(current => merge(current, value.members));
      if (more || !hasPaginated.current) setNext(value.next);
      if (more) hasPaginated.current = true;
      if (!more) {
        try { const options = await api<{ branches: TeamBranch[] }>("/api/admin/team/branches"); setBranches(options.branches); setBranchesError(""); }
        catch (cause) { setBranchesError(errorText(cause)); }
      }
    }
    catch (cause) { setError(errorText(cause)); if (cause instanceof ApiError && [401,403].includes(cause.status)) void onSessionChange(); }
    finally { setLoading(false); }
  }
  async function link(member: TeamMember) {
    setBusy(member.id); setError("");
    try { const value = await api<{ token: string }>(`/api/admin/team/${member.id}/link`, { method: "POST", body: "{}" }); setInvitation({ member, token: value.token }); }
    catch (cause) { setError(errorText(cause)); }
    finally { setBusy(""); }
  }
  async function transition() {
    if (!confirmation) return;
    const { member, action } = confirmation;
    setBusy(member.id); setError("");
    try {
      const value = await api<{ member: TeamMember }>(`/api/admin/team/${member.id}/${action}`, { method: "POST", body: JSON.stringify({ expected_version: member.version }) });
      setMembers(current => merge(current, [value.member])); setConfirmation(null); setNotice(`${value.member.name} · ${teamStatuses[value.member.status]}`);
    } catch (cause) { setError(errorText(cause)); setConfirmation(null); if (cause instanceof ApiError && cause.code === "RECORD_CONFLICT") void reload(); }
    finally { setBusy(""); }
  }
  const shown = members.filter(member => (filter === "ALL" || member.status === filter) && `${member.name} ${member.line_name || ""} ${member.branches.map(branch => branch.name).join(" ")}`.toLowerCase().includes(query.toLowerCase()));
  const navigate = () => { window.location.hash = ""; };
  return <div className="app-shell team-shell"><WorkspaceSidebar username={session.username} login={session.login} activePanel="inquiries" onNavigate={navigate} onLogout={onLogout} onManageTeam={() => {}} teamActive />
    <main className="main-content"><header className="page-header"><div><p className="eyebrow">Arisa Rental</p><h1>จัดการทีม</h1><p className="muted">เพิ่มพนักงาน ตรวจคำขอ และกำหนดสิทธิ์ของแต่ละคน</p></div><div className="header-actions"><button className="button secondary" onClick={navigate}>กลับแชท</button><button className="button primary" disabled={!branchesReady || !!busy} onClick={() => { setNotice(""); setEditor("new"); }}>＋ เพิ่มพนักงาน</button></div></header>
      <section className="team-content" aria-label="รายชื่อทีมงาน">
        {error && <p className="notice warning" role="alert">{error}</p>}{notice && <p className="team-feedback" role="status">{notice}</p>}
        <div className="team-toolbar"><label className="team-search">ค้นหาพนักงาน<input autoComplete="off" type="search" value={query} placeholder="ชื่อพนักงานหรือชื่อ LINE" onChange={event => setQuery(event.target.value)} /></label><button className="button secondary" disabled={loading} onClick={() => void reload()}>{loading ? "กำลังโหลด…" : "โหลดข้อมูลล่าสุด"}</button></div>
        <div className="team-filters" aria-label="กรองสถานะ">{(["ALL","INVITED","PENDING","ACTIVE","SUSPENDED","EXPIRED","REJECTED","CANCELLED"] as const).map(status => <button type="button" key={status} aria-pressed={filter === status} onClick={() => setFilter(status)}>{status === "ALL" ? "ทั้งหมด" : teamStatuses[status]}{status === "PENDING" && members.some(member => member.status === status) && <span>{members.filter(member => member.status === status).length}</span>}</button>)}</div>
        <div className="team-members">{shown.map(member => <article className="team-member" key={member.id}>
          <div className="team-member-identity">{member.picture_url ? <img className="team-avatar" src={member.picture_url} alt="" referrerPolicy="no-referrer" /> : <span className="team-avatar" aria-hidden="true">{member.name.slice(0,1)}</span>}<div><h2>{member.name}{member.id === session.membership?.id && <span className="muted small"> (คุณ)</span>}</h2><p className="muted small">{member.line_name ? `LINE: ${member.line_name}` : "ยังไม่ได้ยืนยันบัญชี LINE"}</p></div></div>
          <div className="team-member-detail"><span className={`team-status status-${member.status.toLowerCase()}`}>{teamStatuses[member.status]}</span><p>{teamRoles[member.role]}</p><p className="muted small">{member.role === "OWNER" ? "ทุกสาขาที่ร้านมีสิทธิ์" : member.branches.map(branch => branch.name).join(", ") || "ยังไม่มีสาขาสินค้า"}</p>{member.status === "INVITED" || member.status === "EXPIRED" ? <p className="muted small">รับคำเชิญภายใน {formatDateTime(member.expires_at)}</p> : member.last_login_at ? <p className="muted small">เข้าใช้ล่าสุด {formatDateTime(member.last_login_at)}</p> : null}</div>
          <div className="team-member-actions">{member.status === "INVITED" && <><button type="button" className="button secondary" disabled={!!busy} onClick={() => void link(member)}>ดูคำเชิญ</button><button type="button" className="text-button team-danger" disabled={!!busy} onClick={() => setConfirmation({ member, action: "cancel" })}>ยกเลิกคำเชิญ</button></>}{member.status === "PENDING" && <><button type="button" className="button primary" disabled={!!busy} onClick={() => setConfirmation({ member, action: "approve" })}>ตรวจและอนุมัติ</button><button type="button" className="text-button team-danger" disabled={!!busy} onClick={() => setConfirmation({ member, action: "reject" })}>ปฏิเสธ</button></>}{member.role !== "OWNER" && ["PENDING","ACTIVE","SUSPENDED"].includes(member.status) && <button type="button" className="button secondary" disabled={!!busy || !branchesReady} onClick={() => setEditor(member)}>แก้ไขสิทธิ์</button>}<button type="button" className="text-button" onClick={() => setHistory(member)}>ประวัติ</button></div>
        </article>)}</div>
        {!shown.length && !loading && <div className="empty compact"><h2>ไม่พบพนักงานในรายการนี้</h2><p>เปลี่ยนตัวกรองหรือเพิ่มพนักงานใหม่</p></div>}{loading && <p className="muted" role="status">กำลังโหลดรายชื่อ…</p>}{next && <button className="button secondary full" disabled={loading} onClick={() => void reload(true)}>โหลดรายชื่อก่อนหน้า</button>}
      </section>
    </main>
    {editor && <MemberEditor member={editor === "new" ? null : editor} branches={branches} branchesError={branchesError} onClose={() => setEditor(null)} onSaved={(member, token) => { setMembers(current => merge(current,[member])); setEditor(null); if (token) setInvitation({ member, token }); else setNotice(`บันทึกข้อมูล ${member.name} แล้ว`); }} />}
    {invitation && <InvitationLink {...invitation} onClose={() => setInvitation(null)} />}
    {history && <History member={history} onClose={() => setHistory(null)} />}
    {confirmation && <Dialog title={confirmation.action === "approve" ? "ตรวจบัญชีก่อนอนุมัติ" : confirmation.action === "reject" ? "ปฏิเสธคำขอเข้าร้าน" : "ยกเลิกคำเชิญ"} busy={!!busy} onClose={() => setConfirmation(null)}><div className="team-identity">{confirmation.member.picture_url && <img src={confirmation.member.picture_url} alt="" referrerPolicy="no-referrer" />}<div><strong>{confirmation.member.name}</strong><p>LINE: {confirmation.member.line_name || "ยังไม่ยืนยัน"}</p></div></div><p>บทบาท {teamRoles[confirmation.member.role]}</p><p className="muted">สาขา: {confirmation.member.branches.map(branch => branch.name).join(", ") || "ยังไม่มีสาขาสินค้า"}</p><p>{confirmation.action === "approve" ? "ตรวจว่าบัญชี LINE นี้เป็นของพนักงานที่คุณต้องการ ก่อนอนุญาตให้เข้าถึงข้อมูลร้าน" : confirmation.action === "reject" ? "บัญชีนี้จะไม่ได้รับสิทธิ์เข้าร้าน สามารถสร้างคำเชิญใหม่ภายหลังได้" : "ลิงก์และ QR ของคำเชิญนี้จะใช้ต่อไม่ได้"}</p><div className="team-form-actions"><button className="button secondary" disabled={!!busy} onClick={() => setConfirmation(null)}>กลับไปตรวจ</button><button className={`button ${confirmation.action === "approve" ? "primary" : "team-danger-button"}`} disabled={!!busy} onClick={() => void transition()}>{busy ? "กำลังบันทึก…" : confirmation.action === "approve" ? "อนุมัติให้เข้าร้าน" : confirmation.action === "reject" ? "ยืนยันปฏิเสธ" : "ยืนยันยกเลิกคำเชิญ"}</button></div></Dialog>}
  </div>;
}

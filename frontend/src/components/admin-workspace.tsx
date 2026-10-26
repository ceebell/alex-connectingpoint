"use client";
import { FormEvent, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { api, ApiError, Draft, DraftList, emptyFields, errorText, Evidence, fieldLabel, Fields, filledKeys, Inquiry, Job, LineEvent, Order } from "@/lib/api";
import DraftEditor from "./draft-editor";
import ChatMessage, { CustomerAvatar } from "./chat-message";
import CustomerProfile from "./customer-profile";

type Session = { username: string; login: "line" | "password" | "token"; hermes_configured: boolean };
type LoginOptions = { line: boolean; password: boolean };
// The login screen renders only after the session check, so reading the URL here cannot cause a hydration mismatch.
const loginParams = () => new URLSearchParams(typeof window === "undefined" ? "" : window.location.search);
const loginErrors: Record<string, string> = { LINE_CANCELLED: "ยกเลิกการเข้าสู่ระบบด้วย LINE", LINE_STATE_INVALID: "การเข้าสู่ระบบหมดเวลาหรือไม่ถูกต้อง กรุณาลองใหม่", LINE_LOGIN_FAILED: "เข้าสู่ระบบด้วย LINE ไม่สำเร็จ กรุณาลองใหม่", LINE_NOT_CONFIGURED: "ยังไม่ได้ตั้งค่า LINE Login กรุณาติดต่อผู้ดูแลระบบ", LINE_NOT_ALLOWED: "บัญชี LINE นี้ยังไม่มีสิทธิ์เข้าหลังบ้าน" };
type OaStatus = { state: string; synced?: number; pending?: number };
const oaStates: Record<string, string> = { NOT_CONFIGURED: "รอเปิดตัวเชื่อม OA บน Ubuntu", NEEDS_LOGIN: "ตัวเชื่อมรอล็อกอิน LINE กรุณาติดต่อผู้ดูแลระบบ", READY: "อ่าน OA ได้ ยังไม่มีข้อความแอดมินใหม่ที่จับคู่ได้", SYNCING: "กำลังรับข้อความแอดมินจาก OA อัตโนมัติ", UNLINKED: "มีข้อความ OA รอจับคู่กับบทสนทนาลูกค้า", UNSUPPORTED_SCHEMA: "รูปแบบข้อมูล OA ยังไม่รองรับ กรุณาติดต่อผู้พัฒนา", OFFLINE: "ตัวเชื่อม OA ขาดการติดต่อ", LINE_UNAVAILABLE: "ตัวเชื่อมติดต่อ LINE ไม่สำเร็จ", BACKEND_UNAVAILABLE: "backend ของตัวเชื่อมยังไม่พร้อม", BACKEND_UPDATE_REQUIRED: "ต้องอัปเดต backend เพื่อเปิดตัวเชื่อม OA", STOPPED: "ตัวเชื่อม OA หยุดทำงาน" };
function mergeMessages(previous: LineEvent[], incoming: LineEvent[]) {
  return [...new Map([...previous, ...incoming].map(message => [message._id, message])).values()].sort((a, b) => new Date(a.event_at).getTime() - new Date(b.event_at).getTime() || (a.event_id ?? a._id).localeCompare(b.event_id ?? b._id));
}
const time = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", timeZone: "Asia/Bangkok" }).format(new Date(value));
const statusLabels: Record<string, string> = { RECEIVED: "รับเรื่องแล้ว", READY_FOR_REVIEW: "รอตรวจร่าง", NEEDS_INFORMATION: "รอข้อมูลเพิ่ม", NEEDS_REVIEW: "มีข้อมูลขัดกัน", CLOSED: "ปิดเรื่อง", PENDING: "รอประมวลผล", PROCESSING: "AI กำลังเตรียมร่าง", DONE: "เสร็จแล้ว", STALE: "บริบทเปลี่ยนแล้ว", FAILED: "เตรียมร่างไม่สำเร็จ" };

function EvidenceList({ evidence }: { evidence: Evidence[] }) {
  return <details className="evidence"><summary>ดูหลักฐานอ้างอิง ({evidence.length})</summary>{evidence.map((item, i) => <div className="evidence-item" key={i}><strong>{fieldLabel(item.field)}</strong><span className="muted small">{({ admin_summary: "สรุปแอดมิน", oa_message: "ข้อความแอดมินจากตัวเชื่อม OA", line_event: "ข้อความหรือรูปจาก LINE", previous_draft: "ร่างก่อนหน้า", admin_edit: "แอดมินแก้ไข", admin_message: "ข้อความที่แอดมินส่งจาก Alex" })[item.source_type] ?? item.source_type}</span><p>{item.quote}</p><code>{item.source_ref}</code></div>)}</details>;
}

function DraftReadOnly({ draft }: { draft: Draft }) {
  if (draft.redacted || !draft.result) return <div className="notice warning">ร่างนี้อ้างอิงข้อความที่ถูกยกเลิก จึงซ่อนเนื้อหา กรุณาเตรียมร่างใหม่</div>;
  const fields = draft.result.fields;
  return <div className="draft-preview"><dl>{filledKeys(fields).map(key => {
    const value = key.split(".").reduce<unknown>((node, part) => node && typeof node === "object" ? (node as Record<string, unknown>)[part] : null, fields);
    return <div key={key}><dt>{fieldLabel(key)}</dt><dd>{key.startsWith("amounts.") ? `${Number(value) / 100} บาท` : String(value)}</dd></div>;
  })}</dl><EvidenceList evidence={draft.result.evidence} /></div>;
}

export default function AdminWorkspace() {
  const [session, setSession] = useState<Session | null>(null);
  const [checking, setChecking] = useState(true);
  const [loginError, setLoginError] = useState(() => { const code = loginParams().get("login_error"); return code ? loginErrors[code] ?? "เข้าสู่ระบบไม่สำเร็จ กรุณาลองใหม่" : ""; });
  const [loginBusy, setLoginBusy] = useState(false);
  const [loginOptions, setLoginOptions] = useState<LoginOptions>({ line: false, password: true });
  const [deniedLineId] = useState(() => loginParams().get("login_error") === "LINE_NOT_ALLOWED" ? loginParams().get("line_user_id")?.replace(/[^A-Za-z0-9]/g, "") ?? "" : "");
  const [replyText, setReplyText] = useState("");
  const [replyId, setReplyId] = useState<string | null>(null);
  const [replyBusy, setReplyBusy] = useState(false);
  const [replyError, setReplyError] = useState("");
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");
  const [inquiries, setInquiries] = useState<Inquiry[]>([]);
  const [counts, setCounts] = useState<Record<string, number>>({});
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const [messages, setMessages] = useState<LineEvent[]>([]);
  const messagesRef = useRef<LineEvent[]>([]);
  const [oaStatus, setOaStatus] = useState<OaStatus>({ state: "NOT_CONFIGURED" });
  const [nextMessage, setNextMessage] = useState<string | null>(null);
  const [drafts, setDrafts] = useState<Draft[]>([]);
  const [version, setVersion] = useState(0);
  const [contextHash,setContextHash]=useState("");
  const [summary, setSummary] = useState("");
  const [images, setImages] = useState<string[]>([]);
  const [tab, setTab] = useState<"summary" | "draft" | "history" | "customer">("summary");
  const [order, setOrder] = useState<Order | null>(null);
  const [approveChecked, setApproveChecked] = useState(false);
  const [job, setJob] = useState<Job | null>(null);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [refresh, setRefresh] = useState(0);
  const latest = drafts[0];
  const working = busy || job?.status === "PENDING" || job?.status === "PROCESSING";
  const current = inquiries.find(inquiry => inquiry._id === selected);
  const canApprove = !order && !!latest?.result && latest.workflow_status === "READY_FOR_REVIEW" && !latest.stale && !latest.redacted && latest.draft_version === version;
  useEffect(() => { messagesRef.current = messages; }, [messages]);

  useEffect(() => {
    // Show the LINE Login result once, then drop it from the address bar.
    if (window.location.search) window.history.replaceState(null, "", window.location.pathname);
    api<LoginOptions>("/api/admin/login-options").then(setLoginOptions).catch(() => {});
  }, []);

  useEffect(() => { let cancelled = false; api<Session>("/api/admin/session").then(value => { if (!cancelled) setSession(value); }).catch(() => {}).finally(() => { if (!cancelled) setChecking(false); }); return () => { cancelled = true; }; }, []);

  useEffect(() => {
    if (!session) return;
    let cancelled = false;
    Promise.all([api<Inquiry[]>("/api/inquiries"), api<{ events: Record<string, number> }>("/api/line/status")]).then(([items, status]) => {
      if (cancelled) return;
      setInquiries(items); setCounts(status.events);
    }).catch(cause => { if (!cancelled) { if (cause instanceof ApiError && cause.status === 401) setSession(null); else setError(errorText(cause)); } });
    return () => { cancelled = true; };
  }, [session, refresh]);

  useEffect(() => {
    if (!selected || !session) return;
    let cancelled = false;
    Promise.all([api<{ batches: LineEvent[][]; next: string | null }>(`/api/inquiries/${selected}/context`), api<DraftList>(`/api/inquiries/${selected}/drafts`), api<{ order: Order | null }>(`/api/inquiries/${selected}/order`)]).then(([context, result, approved]) => {
      if (cancelled) return;
      setOrder(approved.order);
      setMessages(context.batches.flat()); setNextMessage(context.next);
      setDrafts(result.drafts); setVersion(result.version);
      setContextHash(result.context_hash);
      setSummary(result.drafts[0]?.admin_summary ?? "");
      setJob(result.latest_request ?? null);
      setLoading(false);
    }).catch(cause => { if (!cancelled) { setLoading(false); setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) setSession(null); } });
    return () => { cancelled = true; };
  }, [selected, session, refresh]);

  useEffect(() => {
    if (!job || !["PENDING", "PROCESSING"].includes(job.status)) return;
    let cancelled = false;
    const timer = setInterval(() => {
      api<Job>(`/api/draft-requests/${job.id}`).then(value => {
        if (cancelled) return;
        setJob(value);
        if (value.status === "DONE") { setTab("draft"); setRefresh(n => n + 1); }
        else if (["FAILED", "STALE"].includes(value.status)) setError(errorText(new ApiError(value.error_code ?? "HERMES_FAILED", 400)));
      }).catch(cause => { if (!cancelled) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) setSession(null); } });
    }, 1500);
    return () => { cancelled = true; clearInterval(timer); };
  }, [job?.id, job?.status]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!session) return;
    let cancelled = false;
    let polling = false;
    async function poll() {
      if (polling) return;
      polling = true;
      try {
        const status = await api<OaStatus>("/api/line/oa-sync/status");
        if (cancelled) return;
        setOaStatus(status);
        if (selected) {
          let cursor: string | null = null;
          const records: LineEvent[] = [];
          const pageCount = Math.max(1, Math.ceil(messagesRef.current.length / 200));
          for (let n = 0; n < pageCount; n++) {
            const result: { batches: LineEvent[][]; next: string | null } = await api(`/api/inquiries/${selected}/context${cursor ? `?after=${cursor}` : ""}`);
            if (cancelled) return;
            records.push(...result.batches.flat()); cursor = result.next;
            if (!cursor) break;
          }
          setMessages(records); setNextMessage(cursor);
          const [result, approved] = await Promise.all([api<DraftList>(`/api/inquiries/${selected}/drafts`), api<{ order: Order | null }>(`/api/inquiries/${selected}/order`)]);
          if (!cancelled) { setDrafts(result.drafts); setVersion(result.version); setContextHash(result.context_hash); setOrder(approved.order); }
        }
      } catch (cause) {
        if (!cancelled) {
          if (cause instanceof ApiError && cause.status === 401) setSession(null);
          else setOaStatus({ state: cause instanceof ApiError && cause.status === 404 ? "BACKEND_UPDATE_REQUIRED" : "BACKEND_UNAVAILABLE" });
        }
      } finally { polling = false; }
    }
    const timer = setInterval(poll, 10000); void poll();
    return () => { cancelled = true; clearInterval(timer); };
  }, [selected, session]);

  async function login(event: FormEvent) {
    event.preventDefault(); setLoginBusy(true); setLoginError("");
    try {
      await api("/api/admin/login", { method: "POST", body: JSON.stringify({ username, password }) });
      setPassword(""); setSession(await api<Session>("/api/admin/session"));
    } catch (cause) { setLoginError(errorText(cause)); } finally { setLoginBusy(false); }
  }

  function selectInquiry(id: string) {
    setReplyText(""); setReplyId(null); setReplyError("");
    setSelected(id); setSummary(""); setImages([]); setMessages([]); setDrafts([]); setVersion(0); setJob(null); setOrder(null); setApproveChecked(false); setError(""); setLoading(true); setTab("summary");
  }

  async function submit(manual?: Fields, confirm = false) {
    if (!selected) return;
    setBusy(true); setError("");
    try {
      const result = await api<Job>(`/api/inquiries/${selected}/${manual ? "draft" : "prepare"}`, { method: "POST", body: JSON.stringify({ request_id: crypto.randomUUID(), expected_version: version, context_hash: contextHash, summary, image_event_ids: manual ? [] : images, fields: manual ?? null, confirmed_fields: manual && confirm ? filledKeys(manual) : [] }) });
      setJob(result);
    } catch (cause) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) setSession(null); } finally { setBusy(false); }
  }

  async function approveOrder() {
    if (!selected || !latest || !canApprove || !approveChecked) return;
    setBusy(true); setError("");
    try {
      const result = await api<{ order: Order }>(`/api/inquiries/${selected}/order`, { method: "POST", body: JSON.stringify({ draft_id: latest._id, draft_version: latest.draft_version, context_hash: contextHash }) });
      setOrder(result.order); setApproveChecked(false); setInquiries(previous => previous.map(inquiry => inquiry._id === selected ? { ...inquiry, order_id: result.order._id } : inquiry));
    } catch (cause) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) setSession(null); else setRefresh(n => n + 1); } finally { setBusy(false); }
  }

  async function sendReply() {
    if (!selected || !replyText.trim() || replyBusy) return;
    // Keep the ID until LINE accepts it: retrying with it never sends twice.
    const requestId = replyId ?? crypto.randomUUID();
    setReplyId(requestId); setReplyBusy(true); setReplyError("");
    try {
      const result = await api<{ message: LineEvent }>(`/api/inquiries/${selected}/messages`, { method: "POST", body: JSON.stringify({ request_id: requestId, text: replyText }) });
      setMessages(previous => mergeMessages(previous, [result.message])); setReplyText(""); setReplyId(null);
    } catch (cause) { if (cause instanceof ApiError && cause.status === 401) setSession(null); else setReplyError(errorText(cause)); } finally { setReplyBusy(false); }
  }

  async function moreMessages() {
    if (!selected || !nextMessage) return;
    setBusy(true);
    try {
      const result = await api<{ batches: LineEvent[][]; next: string | null }>(`/api/inquiries/${selected}/context?after=${nextMessage}`);
      setMessages(previous => mergeMessages(previous, result.batches.flat())); setNextMessage(result.next);
    } catch (cause) { setError(errorText(cause)); } finally { setBusy(false); }
  }

  async function moreInquiries() {
    if (!inquiries.length) return;
    setBusy(true);
    try { const more = await api<Inquiry[]>(`/api/inquiries?before=${inquiries[inquiries.length - 1]._id}`); setInquiries(previous => [...previous, ...more]); }
    catch (cause) { setError(errorText(cause)); } finally { setBusy(false); }
  }

  async function logout() {
    try { await api("/api/admin/session", { method: "DELETE" }); setSession(null); setSelected(null); setMessages([]); setDrafts([]); setInquiries([]); setSummary(""); setJob(null); setOrder(null); }
    catch (cause) { setError(errorText(cause)); }
  }

  if (checking) return <main className="login-page"><p role="status">กำลังตรวจสอบการเข้าสู่ระบบ…</p></main>;
  if (!session) return <main className="login-page"><div className="login-intro"><div className="brand-mark">a<span>•</span></div><p className="eyebrow">ALEX CONNECTING POINT</p><h1>ทุกบทสนทนา<br />สู่ร่างออเดอร์ที่ชัดเจน</h1><p className="muted">พื้นที่ทำงานของแอดมินสำหรับอ่านแชท สรุปความต้องการ<br />และตรวจร่างออเดอร์ก่อนดำเนินการต่อ</p></div><form className="login-card" onSubmit={login}><span className="pill">สำหรับทีมแอดมิน</span><h2>เข้าสู่ระบบหลังบ้าน</h2><p className="muted">{loginOptions.line ? "ใช้บัญชี LINE ที่ผู้ดูแลระบบเพิ่มสิทธิ์ไว้" : "ใช้บัญชีที่ผู้ดูแลระบบกำหนดให้"}</p>
    {loginError && <div className="notice warning" role="alert">{loginError}{deniedLineId && <p>ส่ง LINE user ID นี้ให้ผู้ดูแลเพิ่มใน ADMIN_LINE_USER_IDS:<br /><code className="line-id">{deniedLineId}</code></p>}</div>}
    {loginOptions.line && <a className="button primary full" href="/api/admin/line/start"><span aria-hidden="true">💬</span> เข้าสู่ระบบด้วย LINE</a>}
    {loginOptions.line && loginOptions.password && <p className="login-divider muted small">หรือใช้รหัสผ่านสำรอง</p>}
    {loginOptions.password && <><label>ชื่อผู้ใช้<input autoComplete="username" value={username} onChange={e => setUsername(e.target.value)} required /></label><label>รหัสผ่าน<input type="password" autoComplete="current-password" value={password} onChange={e => setPassword(e.target.value)} required /></label><button className={`button ${loginOptions.line ? "secondary" : "primary"} full`} disabled={loginBusy}>{loginBusy ? "กำลังเข้าสู่ระบบ…" : "เข้าสู่พื้นที่ทำงาน →"}</button></>}
    {!loginOptions.line && !loginOptions.password && <p className="notice warning">ยังไม่ได้ตั้งค่าการเข้าสู่ระบบ กรุณาติดต่อผู้ดูแลระบบ</p>}
  </form></main>;

  return <div className="app-shell">
    <aside className="sidebar"><div className="brand"><div className="brand-mark">a<span>•</span></div><div><strong>alex</strong><small>connecting point</small></div></div><span className="nav-heading">พื้นที่ทำงาน</span><Link className="nav-item active" href="/" aria-current="page"><span aria-hidden="true">▤</span> แชทและ inquiry <span className="nav-count">{inquiries.length}</span></Link><div className="sidebar-note"><span className="online-dot" /> รับเรื่องจาก LINE OA<p>อ่านแชท เตรียมร่าง<br />แล้วตรวจสอบก่อนส่งต่อ</p></div><div className="sidebar-account"><div className="avatar">{session.username.charAt(0).toUpperCase()}</div><div><strong>{session.username}</strong><small>{session.login === "line" ? "เข้าด้วย LINE" : "ผู้ดูแลหลังบ้าน"}</small></div><button title="ออกจากระบบ" aria-label="ออกจากระบบ" className="icon-button" onClick={logout}>↪</button></div></aside>
    <main className="main-content"><header className="page-header"><div><p className="eyebrow">จัดการความต้องการลูกค้า</p><h1>แชทและร่างออเดอร์</h1><p className="muted">เลือกเรื่องที่รับเข้ามา แล้วเปลี่ยนรายละเอียดเป็นร่างที่ตรวจสอบได้</p></div><div className="header-actions"><button className="mobile-logout text-button" onClick={logout}>ออกจากระบบ</button><button className="button secondary" disabled={working} onClick={() => { setError(""); setRefresh(n => n + 1); }}><span aria-hidden="true">↻</span> โหลดข้อมูลล่าสุด</button></div></header>
      <div className="stats"><div><span className="stat-icon green" aria-hidden="true">▤</span><div><span>inquiry ที่โหลดมา</span><strong>{inquiries.length}</strong></div></div><div><span className="stat-icon sky" aria-hidden="true">✓</span><div><span>ข้อความประมวลผลแล้ว</span><strong>{counts.DONE ?? 0}</strong></div></div><div><span className="stat-icon orange" aria-hidden="true">!</span><div><span>เหตุการณ์รอแอดมินตรวจ</span><strong>{(counts.NEEDS_ADMIN ?? 0) + (counts.FAILED ?? 0)}</strong></div></div></div>
      {error && <div className="notice warning page-notice" role="alert">{error}<button className="text-button" aria-label="ปิดข้อความแจ้งเตือน" onClick={() => setError("")}>ปิด</button></div>}
      <div className="workspace"><section className="inquiry-panel"><div className="panel-heading"><h2>เรื่องที่รับเข้า</h2><span className="small muted">ล่าสุดก่อน</span></div><div className="search-box"><span aria-hidden="true">⌕</span><input aria-label="ค้นหา inquiry หรือ LINE user ID" placeholder="ค้นหา inquiry / ผู้ใช้…" value={query} onChange={e => setQuery(e.target.value)} /></div><div className="inquiry-list">
        {inquiries.filter(inquiry => `${inquiry._id} ${inquiry.line_user_id ?? ""} ${inquiry.customer?.display_name ?? ""}`.toLowerCase().includes(query.toLowerCase())).map(inquiry => <button key={inquiry._id} className={`inquiry-card ${selected === inquiry._id ? "selected" : ""}`} disabled={working} onClick={() => selectInquiry(inquiry._id)}><div className="section-caption"><span className="line-badge">LINE</span><time>{time(inquiry.updated_at)}</time></div><div className="customer-row"><CustomerAvatar customer={inquiry.customer} /><div><strong>{inquiry.customer?.display_name || `เรื่อง #${inquiry._id.slice(-6).toUpperCase()}`}</strong><p className="user-ref">{inquiry.customer?.display_name ? `เรื่อง #${inquiry._id.slice(-6).toUpperCase()}` : inquiry.line_user_id ?? "ยังไม่มีรหัสผู้ใช้"}</p></div></div><div className="section-caption"><span className="pill">{statusLabels[inquiry.status] ?? inquiry.status}</span>{inquiry.order_id ? <span className="pill success">ออเดอร์แล้ว</span> : (inquiry.draft_version ?? 0) > 0 && <span className="small muted">ร่าง v{inquiry.draft_version}</span>}</div></button>)}
        {!inquiries.length && <div className="empty compact"><span aria-hidden="true">▤</span><h3>ยังไม่มี inquiry</h3><p>เมื่อมีข้อความเข้า LINE OA<br />เรื่องใหม่จะแสดงที่นี่</p></div>}
        {inquiries.length >= 100 && <button className="text-button full" disabled={busy} onClick={moreInquiries}>โหลดเรื่องก่อนหน้า</button>}
      </div></section>
      <section className="chat-panel"><div className="panel-heading"><div><h2>{current ? current.customer?.display_name || `เรื่อง #${current._id.slice(-6).toUpperCase()}` : "บทสนทนาจาก LINE"}</h2><span className="small muted">{current ? `${current.customer?.display_name ? `เรื่อง #${current._id.slice(-6).toUpperCase()} · ` : ""}ลูกค้าและแอดมิน · เวลาไทย` : "เลือก inquiry เพื่ออ่านรายละเอียด"}</span></div>{current && <span className="line-badge">LINE OA</span>}</div>
        <div className="chat-stream" aria-busy={loading}>
          {!selected ? <div className="empty"><div className="empty-symbol" aria-hidden="true">☰</div><h3>เริ่มจากเลือกบทสนทนา</h3><p>อ่านแชทและรูปจากลูกค้า<br />แล้วใส่สรุปเพื่อเตรียมร่างออเดอร์</p></div> : loading ? <p className="muted" role="status">กำลังโหลดบทสนทนา…</p> : <><div className="chat-notice">ลูกค้าอยู่ด้านซ้าย · แอดมินอยู่ด้านขวา{oaStatus.state !== "NOT_CONFIGURED" && <><br /><span>{oaStates[oaStatus.state] ?? "กำลังตรวจตัวเชื่อม OA"}</span></>}</div>{messages.map(message => <ChatMessage key={message._id} message={message} customer={current?.customer} selectedImages={images} disabled={working} onToggleImage={(id, checked) => setImages(previous => checked ? [...previous, id] : previous.filter(value => value !== id))} />)}{nextMessage && <button className="button secondary full" disabled={working} onClick={moreMessages}>โหลดข้อความถัดไป</button>}{!messages.length && <p className="muted">ยังไม่มีข้อความที่ประมวลผลเสร็จ กรุณาโหลดข้อมูลอีกครั้ง</p>}</>}
        </div>
        {current && !loading && <form className="reply-box" onSubmit={e => { e.preventDefault(); void sendReply(); }}>
          {replyError && <div className="notice warning" role="alert">{replyError}</div>}
          <div className="reply-row"><label className="sr-only" htmlFor="reply-text">ข้อความตอบลูกค้า</label><textarea id="reply-text" rows={2} maxLength={5000} value={replyText} disabled={replyBusy || !current.line_user_id} placeholder={current.line_user_id ? "พิมพ์ตอบลูกค้า… (Ctrl+Enter เพื่อส่ง)" : "เรื่องนี้ไม่มี LINE user ID จึงตอบจาก Alex ไม่ได้"} onChange={e => { setReplyText(e.target.value); setReplyId(null); setReplyError(""); }} onKeyDown={e => { if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); void sendReply(); } }} /><button className="button primary" disabled={replyBusy || !replyText.trim() || !current.line_user_id}>{replyBusy ? "กำลังส่ง…" : replyId ? "ส่งอีกครั้ง" : "ส่ง"}</button></div>
          <p className="muted small">ส่งถึงลูกค้าจริงทาง LINE OA และนับโควตาข้อความ · ข้อความที่ส่งจะช่วยเตรียมร่างรอบถัดไป</p>
        </form>}
        <div className="chat-footer"><span className="online-dot" /> เก็บประวัติแยกตามผู้ใช้และบทสนทนา</div>
      </section>
      <section className="draft-panel"><div className="panel-heading"><div><h2>เตรียมร่างออเดอร์</h2><span className="small muted">แอดมินตรวจทุกครั้งก่อนดำเนินการต่อ</span></div>{version > 0 && <span className="version-badge">v{version}</span>}</div><div className="tabs" role="tablist" aria-label="ส่วนจัดการร่าง">{([["summary", "สรุปแอดมิน"], ["draft", "ร่างล่าสุด"], ["history", "เวอร์ชันก่อนหน้า"], ["customer", "ลูกค้า"]] as const).map(([key, label]) => <button key={key} role="tab" aria-selected={tab === key} onClick={() => setTab(key)}>{label}</button>)}</div>
        <div className="draft-content">{!selected ? <div className="empty compact"><span aria-hidden="true">✎</span><h3>รอเลือก inquiry</h3><p>สรุปแอดมินและร่างออเดอร์<br />จะผูกกับเรื่องที่เลือก</p></div> : loading ? <p role="status" className="muted">กำลังโหลดร่าง…</p> : <>
          {working && <div className="notice info" role="status">{job?.status === "PROCESSING" ? "กำลังเตรียมและตรวจร่าง…" : "รับงานแล้ว กำลังรอประมวลผล…"} คุณกลับมาเปิดเรื่องนี้ใหม่เพื่อตรวจผลได้</div>}
          {tab === "summary" && <><div className="step-caption"><span>01</span><div><h3>สรุปสิ่งที่ตกลงกับลูกค้า</h3><p className="muted small">เพิ่มรายละเอียดที่ตกลงกับลูกค้าไว้ที่นี่</p></div></div><label className="summary-label">สรุปสำหรับ inquiry นี้<textarea rows={9} maxLength={20000} value={summary} disabled={working} onChange={e => setSummary(e.target.value)} placeholder="เช่น คุณเอ เช่าชุด DR-012 สีแดง ไซซ์ M จำนวน 1 ชุด รับ 15 พ.ย. 2569 คืน 18 พ.ย. 2569 ค่าเช่า 1,500 บาท มัดจำ 2,000 บาท รับสาขา A" /></label><div className="section-caption small muted"><span>แนบรูปที่เลือก {images.length}/4 รูป</span><span>{summary.length.toLocaleString()}/20,000</span></div><div className="notice soft"><strong>AI ช่วยจัดข้อมูล แอดมินช่วยตรวจความถูกต้อง</strong><p>ร่างจะแสดงข้อมูลที่ยังขาด ข้อขัดแย้ง และหลักฐานอ้างอิง โดยยังไม่สร้างออเดอร์จริง</p></div>{!session.hermes_configured && <div className="notice warning">AI ยังไม่พร้อม กรุณาติดต่อผู้ดูแลระบบ หรือไปแท็บร่างล่าสุดเพื่อกรอกด้วยมือ</div>}<button className="button primary full" disabled={working || !summary.trim() || !session.hermes_configured} onClick={() => submit()}><span aria-hidden="true">✦</span> เตรียมออเดอร์ด้วย AI</button><button className="text-button full" onClick={() => setTab("draft")}>หรือกรอกร่างด้วยมือ →</button></>}
          {tab === "draft" && <>{latest?.stale && <div className="notice warning">มีการเปลี่ยนแปลงในแชท ร่างนี้ต้องได้รับการตรวจและเตรียมใหม่</div>}{latest && <div className="section-caption draft-state"><span className="pill">{statusLabels[latest.workflow_status] ?? latest.workflow_status}</span><span className="small muted">{time(latest.created_at)}</span></div>}{latest?.redacted && <div className="notice warning">ซ่อนร่างเดิมเพราะมีข้อความถูกยกเลิก กรุณาเตรียมร่างใหม่จากสรุปและบริบทล่าสุด</div>}{latest?.result && <>{latest.result.conflicts.length > 0 && <div className="notice warning"><strong>ข้อมูลขัดกันที่ต้องตรวจ</strong><ul>{latest.result.conflicts.map((value, i) => <li key={i}>{value}</li>)}</ul></div>}{latest.missing_fields.length > 0 && <div className="notice soft"><strong>ข้อมูลที่ยังขาด</strong><p>{latest.missing_fields.map(fieldLabel).join(" · ")}</p></div>}{latest.result.questions.length > 0 && <div className="questions"><h3>คำถามที่ AI แนะนำ</h3><ul>{latest.result.questions.map((value, i) => <li key={i}>{value}</li>)}</ul><p className="muted small">ใช้ประกอบการคุยกับลูกค้า ยังไม่ได้ส่งข้อความ</p></div>}</>}
          {order ? <div className="notice success" role="status"><strong><span aria-hidden="true">✅</span> อนุมัติเป็นออเดอร์แล้ว</strong><p>จากร่าง v{order.draft_version} · โดย {order.approved_by} · {time(order.approved_at)}<br />บันทึกในฐานข้อมูลบน server แล้ว ยังไม่ได้ตรวจสต็อก คิว และราคากับระบบ Alex</p></div>
            : latest?.result && <div className="approve-card">{canApprove ? <><label className="checkbox-row"><input type="checkbox" checked={approveChecked} disabled={working} onChange={e => setApproveChecked(e.target.checked)} />ตรวจร่าง v{latest.draft_version} กับลูกค้าแล้ว ข้อมูลถูกต้อง</label><button type="button" className="button primary full" disabled={working || !approveChecked} onClick={approveOrder}>อนุมัติเป็นออเดอร์</button><p className="muted small">บันทึกร่างนี้เป็นออเดอร์ในฐานข้อมูล แก้ร่างภายหลังได้ แต่ออเดอร์ที่อนุมัติแล้วจะไม่เปลี่ยนตาม</p></> : <p className="muted small">อนุมัติเป็นออเดอร์ได้เมื่อร่างล่าสุดข้อมูลครบ ไม่มีข้อขัดแย้ง และแชทไม่มีข้อความใหม่หลังทำร่าง</p>}</div>}
          <DraftEditor key={`${selected}-${version}-${latest?.redacted}`} initial={latest?.result?.fields ?? emptyFields()} busy={!!working} onSave={(fields, confirm) => submit(fields, confirm)} />{latest?.result && <EvidenceList evidence={latest.result.evidence} />}</>}
          {tab === "customer" && (current?.line_user_id ? <CustomerProfile key={current.line_user_id} userId={current.line_user_id} line={current.customer} onUnauthorized={() => setSession(null)} /> : <div className="notice warning">เรื่องนี้ไม่มี LINE user ID จึงยังบันทึกข้อมูลลูกค้าไม่ได้</div>)}
          {tab === "history" && <>{!drafts.length && <p className="muted">ยังไม่มีเวอร์ชันที่บันทึก</p>}{drafts.map(draft => <details className="history-card" key={draft._id}><summary><span>ร่างเวอร์ชัน {draft.draft_version}</span><span className="small muted">{time(draft.created_at)}</span></summary><DraftReadOnly draft={draft} /></details>)}</>}
        </>}</div><div className="draft-footer">ออเดอร์ที่อนุมัติบันทึกในฐานข้อมูลบน server · ยังไม่เช็คคิวหรือสต็อกกับ Alex</div>
      </section></div>
    </main>
  </div>;
}

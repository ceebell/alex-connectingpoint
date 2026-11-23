"use client";
import ImagePreview from "./image-preview";
import { useCallback, useEffect, useRef, useState } from "react";
import { api, ApiError, errorText } from "@/lib/api";
import { canManageTeam, type AdminSession } from "@/lib/team-api";
import { formatDate, formatDateTime } from "@/lib/dates";
import { normalizeTracking, prepareReceipt, suggestedChatOrder, trackingMessage, validTracking, type Parcel, type Receipt, type ReceiptResult, type ShippingCustomerMatch, type ShippingOrder, type ShippingSearchResult, type Tracking } from "@/lib/shipping";
import { navigateWorkspace } from "@/lib/workspace-navigation";
import { CustomerAvatar } from "./chat-message";
import WorkspaceNavigation, { WorkspaceIcon } from "./workspace-navigation";
import WorkspaceSidebar from "./workspace-sidebar";
import { useTeamViewport } from "./team-workspace";

const statuses = { QUEUED: "รออ่านใบเสร็จ", READING: "กำลังอ่านใบเสร็จ", READY: "พร้อมตรวจสอบ", FAILED: "อ่านไม่สำเร็จ" };
const money = (satang: number | null | undefined) => satang == null ? "อ่านยอดไม่ได้" : `${(satang / 100).toLocaleString("th-TH", { minimumFractionDigits: 2 })} บาท`;
const endpoint = "/api/admin/shipping";
const matchSources = { alex_submission: "ชื่อในประวัติบิล", customer_profile: "ชื่อในโปรไฟล์ลูกค้า", chat_alias: "ชื่อที่แอดมินตั้งในแชท", line_profile: "ชื่อโปรไฟล์ LINE", oa_name: "ชื่อใน LINE OA", customer_note: "ข้อความในโน้ตแชท", profile_note: "โน้ตในโปรไฟล์ลูกค้า" };
const fromNote = (source: ShippingCustomerMatch["match_source"]) => source === "customer_note" || source === "profile_note";
async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const controller = new AbortController(); const timer = setTimeout(() => controller.abort(), 90000);
  try { return await api<T>(url, { ...options, signal: controller.signal }); } finally { clearTimeout(timer); }
}

function ParcelCard({ parcel, receiptId, saved, onSaved, batchBusy }: { parcel: Parcel; receiptId: string; saved?: Tracking; onSaved: () => Promise<void>; batchBusy: boolean }) {
  const name = saved?.recipient_name ?? parcel.recipient_name ?? "";
  const [number, setNumber] = useState(saved?.tracking_number ?? parcel.tracking_number ?? "");
  const fee = saved ? saved.shipping_fee_satang : parcel.shipping_fee_satang;
  const [postcode, setPostcode] = useState(saved?.postal_code ?? parcel.postal_code ?? "");
  const service = saved?.service ?? parcel.service ?? "EMS";
  const [message, setMessage] = useState(saved?.message ?? trackingMessage(number));
  const [editedMessage, setEditedMessage] = useState(!!saved);
  const [query, setQuery] = useState(saved?.chat_customer?.display_name ?? saved?.customer.name ?? parcel.recipient_name ?? "");
  const [orders, setOrders] = useState<ShippingOrder[]>([]);
  const [customers, setCustomers] = useState<ShippingCustomerMatch[]>([]);
  const [customersMore, setCustomersMore] = useState(false);
  const [selected, setSelected] = useState(saved?.submission_id ?? "");
  const [selectedInquiry, setSelectedInquiry] = useState(saved && !saved.submission_id ? saved.inquiry_id : "");
  const [selectedCustomer, setSelectedCustomer] = useState<ShippingCustomerMatch | null>(null);
  const [searching, setSearching] = useState(true);
  const [more, setMore] = useState(false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [sending, setSending] = useState(false);
  const [dirty, setDirty] = useState(false);
  const locked = !!saved?.delivery_started_at;
  const chosen = orders.find(order => order.submission_id === selected);
  const suggested = !selected && !searching ? suggestedChatOrder(orders, more || customersMore, customers) : undefined;
  const visibleCustomers = customers.filter(customer => fromNote(customer.match_source) || !orders.some(order => order.line_user_id === customer.line_user_id));
  const chatName = chosen?.chat_customer.display_name || selectedCustomer?.chat_customer.display_name || saved?.chat_customer?.display_name;
  const recipientChat = chosen?.chat_customer ?? selectedCustomer?.chat_customer ?? saved?.chat_customer ?? { display_name: saved?.customer.name };
  const recipientInquiry = chosen?.inquiry_id || selectedInquiry || saved?.inquiry_id;
  const orderNumber = selected ? chosen?.alex_order_number || saved?.alex_order_number : null;
  const recipientName = name.trim() || chosen?.customer.name || selectedCustomer?.matched_name || saved?.customer.name || "";
  const hasRecipient = !!selected || !!selectedInquiry;
  function chooseCustomer(customer: ShippingCustomerMatch) {
    setSelected(""); setSelectedInquiry(customer.inquiry_id); setSelectedCustomer(customer); setDirty(true); setError("");
  }
  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(() => {
      setSearching(true);
      void api<ShippingSearchResult>(`${endpoint}/orders?query=${encodeURIComponent(query)}`).then(value => {
        if (!cancelled) { setOrders(value.orders); setMore(value.more); setCustomers(value.customer_matches ?? []); setCustomersMore(!!value.customers_more); setError(""); }
      }).catch(cause => { if (!cancelled) setError(errorText(cause)); }).finally(() => { if (!cancelled) setSearching(false); });
    }, 350);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [query]);
  const changeMessage = (tracking: string) => { if (!editedMessage) setMessage(trackingMessage(tracking)); };
  async function saveTracking(): Promise<Tracking | undefined> {
    if (!hasRecipient || !recipientName || !validTracking(number)) { setError("เลือกลูกค้าที่จะส่งและตรวจเลข Tracking 13 ตัวให้ถูกต้อง"); return; }
    if (!message.trim() || !message.includes(normalizeTracking(number))) { setError("ข้อความที่จะส่งต้องมีเลข Tracking ที่ตรงกับช่องด้านบน"); return; }
    const result = await request<{ tracking: Tracking }>(`${endpoint}/receipts/${receiptId}/rows/${parcel.row}`, { method: "POST", body: JSON.stringify({ expected_version: saved?.version ?? 0, ...(selected ? { submission_id: selected } : { inquiry_id: selectedInquiry }), recipient_name: recipientName, tracking_number: normalizeTracking(number), postal_code: postcode, service, shipping_fee_satang: fee, message }) });
    return result.tracking;
  }
  async function save() {
    if (batchBusy || busy) return;
    setBusy(true); setError("");
    try { if (await saveTracking()) await onSaved(); }
    catch (cause) { setError(errorText(cause)); }
    finally { setBusy(false); }
  }
  async function send() {
    if (busy || batchBusy || saved?.status === "SENT") return;
    setBusy(true); setSending(true); setError("");
    let deliveryStarted = false;
    try {
      const tracking = !saved || dirty ? await saveTracking() : saved;
      if (!tracking) return;
      deliveryStarted = true;
      const result = await request<{ tracking: Tracking; accepted: boolean }>(`${endpoint}/trackings/${tracking._id}/send`, { method: "POST", body: JSON.stringify({ expected_version: tracking.version }) });
      if (!result.accepted && result.tracking.status !== "SENT") setError("LINE ยังไม่ยืนยันการส่ง ตรวจแชทและผลล่าสุดก่อนลองคำขอเดิม"); await onSaved();
    }
    catch (cause) { setError(`${errorText(cause)} ${deliveryStarted ? "กดตรวจผลล่าสุดก่อนส่งซ้ำ" : "ยังไม่ได้ส่งข้อความ กรุณาตรวจข้อมูลแล้วลองอีกครั้ง"}`); try { await onSaved(); } catch { /* Keep the last known result. */ } }
    finally { setBusy(false); setSending(false); }
  }
  const pending = saved?.status === "SENDING";
  const [clock, setClock] = useState(() => Date.now());
  useEffect(() => { if (!pending) return; const timer = setInterval(() => setClock(Date.now()), 5000); return () => clearInterval(timer); }, [pending]);
  const stale = pending && clock - new Date(saved.updated_at).getTime() > 65000;
  const unknown = saved?.status === "SEND_FAILED";
  const retryExpired = saved?.delivery_started_at && clock - new Date(saved.delivery_started_at).getTime() >= 23 * 3600000;
  return <article className="shipping-parcel" aria-label={`พัสดุรายการ ${parcel.row}`} aria-busy={busy}>
    <header className="shipping-parcel-heading"><span className="shipping-row-number">{String(parcel.row).padStart(2, "0")}</span><div><div className="shipping-parcel-title"><h3>{chatName || name || "ผู้รับพัสดุ"}</h3><span className="shipping-service">{/ems/i.test(service) ? "EMS" : service}</span></div>{chatName && name && <p>ชื่อบนใบเสร็จ: {name}</p>}</div>{saved && <span className={`shipping-status shipping-status-${saved.status.toLowerCase()}`}>{saved.status === "SENT" ? "ส่งแล้ว" : pending ? "กำลังส่ง" : unknown ? "ยังไม่ยืนยัน" : "พร้อมส่ง"}</span>}</header>
    <section className="shipping-order-search" aria-label={`เลือกผู้รับรายการ ${parcel.row}`}>
      <fieldset className="shipping-parcel-fields" disabled={busy || locked || batchBusy}>
        <label>เลข Tracking<input autoComplete="off" spellCheck={false} maxLength={30} value={number} className="shipping-tracking-number" onChange={e => { setNumber(e.target.value); changeMessage(e.target.value); setDirty(true); }} aria-invalid={!!number && !validTracking(number)} /></label>
        <label>รหัสไปรษณีย์<input autoComplete="off" inputMode="numeric" maxLength={5} value={postcode} onChange={e => { setPostcode(e.target.value); setDirty(true); }} /></label>
      </fieldset>
      {!!number && !validTracking(number) && <p className="shipping-description">ตรวจเลข Tracking ให้ครบ 13 ตัว เช่น JD333458790TH</p>}
      {!locked && <>
        <label>ค้นหาชื่อลูกค้า<input autoComplete="off" value={query} disabled={busy || batchBusy} maxLength={200} onChange={e => { setQuery(e.target.value); setSearching(true); setSelected(""); setSelectedInquiry(""); setSelectedCustomer(null); }} placeholder="ชื่อผู้รับ ชื่อในโน้ต หรือเลขออเดอร์" /></label>
        {searching && <p role="status">กำลังค้นหาลูกค้า…</p>}
        {!searching && !orders.length && !customers.length && <p className="shipping-description">ไม่พบลูกค้าที่ตรงกัน ลองค้นหาด้วยชื่อในแชทหรือโน้ต</p>}
        {suggested && !hasRecipient && <div className="shipping-recipient-check shipping-chat-match"><CustomerAvatar customer={suggested.chat_customer} /><div><strong>{suggested.chat_customer.display_name}</strong><p>ชื่อในบิล: {suggested.customer.name}</p><button type="button" className="text-button" disabled={busy || batchBusy} onClick={() => { setSelected(suggested.submission_id); setSelectedInquiry(""); setSelectedCustomer(null); setDirty(true); }}>เลือกออเดอร์นี้</button></div><a className="text-button" href={`#chat/${suggested.inquiry_id}`} target="_blank" rel="noopener noreferrer" title="เปิดแชทในแท็บใหม่">เปิดแชท</a></div>}
        {!searching && visibleCustomers.length > 0 && <section className="shipping-customer-matches" aria-label={`แชทที่ตรงกับชื่อผู้รับรายการ ${parcel.row}`}><h4>เลือกลูกค้าที่จะส่ง Tracking</h4>{visibleCustomers.map(customer => <div key={customer.line_user_id} className={`shipping-recipient-check ${selectedInquiry === customer.inquiry_id ? "is-selected" : ""}`}><CustomerAvatar customer={customer.chat_customer} /><div><strong>{customer.chat_customer.display_name || "ลูกค้าในแชท"}</strong><p>{matchSources[customer.match_source]}: {customer.matched_name}</p><p className="shipping-description">{fromNote(customer.match_source) ? "พบชื่อในโน้ต โปรดตรวจผู้รับก่อนเลือก" : customer.match_score === 100 ? "ชื่อตรงกัน" : "ชื่อใกล้เคียง โปรดตรวจสอบ"}</p><button type="button" className="button secondary shipping-select-customer" aria-pressed={selectedInquiry === customer.inquiry_id} disabled={busy || batchBusy} onClick={() => chooseCustomer(customer)}>{selectedInquiry === customer.inquiry_id ? "เลือกลูกค้าคนนี้แล้ว" : "เลือกลูกค้าคนนี้"}</button></div><a className="text-button" href={`#chat/${customer.inquiry_id}`} target="_blank" rel="noopener noreferrer" title="เปิดแชทในแท็บใหม่">เปิดแชท</a></div>)}</section>}
        {customersMore && <p className="shipping-description">ยังมีลูกค้าที่อาจตรงกันเพิ่มเติม กรุณาพิมพ์ชื่อเต็ม</p>}
        <div className="shipping-order-options" role="radiogroup" aria-label={`ออเดอร์ของผู้รับรายการ ${parcel.row}`}>{orders.map(order => <label key={order.submission_id} className={`shipping-customer-option ${selected === order.submission_id ? "is-selected" : ""}`}><input type="radio" name={`shipping-order-${parcel.row}`} checked={selected === order.submission_id} disabled={busy || batchBusy} onChange={() => { setSelected(order.submission_id); setSelectedInquiry(""); setSelectedCustomer(null); changeMessage(number); setDirty(true); }} /><CustomerAvatar customer={order.chat_customer} /><span><strong>{order.chat_customer.display_name || order.customer.name}</strong><span>ชื่อในบิล: {order.customer.name}</span><span>{order.alex_order_number || order.alex_order_id}{order.draft_version ? ` · ร่าง v${order.draft_version}` : ""}{order.pickup_date ? ` · รับชุด ${formatDate(order.pickup_date)}` : ""}</span><span>{order.match_score === 100 ? "ชื่อตรงกัน" : order.match_score >= 60 && order.match_score <= 100 ? "ชื่อใกล้เคียง โปรดตรวจสอบ" : "ผลการค้นหา"}</span></span></label>)}</div>
        {orders.length > 1 && <p className="shipping-description">พบหลายออเดอร์ กรุณาเลือกออเดอร์ที่จะใช้เลขพัสดุนี้</p>}{more && <p className="shipping-description">มีผลเพิ่มเติม พิมพ์ชื่อหรือเลขออเดอร์ให้เจาะจงขึ้น</p>}
      </>}
    </section>
    {hasRecipient && (chosen || saved) && <div className="shipping-recipient-check"><CustomerAvatar customer={recipientChat} /><div><strong>ส่งให้: {chatName || saved?.customer.name || chosen?.customer.name}</strong>{orderNumber && <p>เลขออเดอร์: {orderNumber}</p>}</div><a className="text-button" href={`#chat/${recipientInquiry}`} target="_blank" rel="noopener noreferrer" title="เปิดแชทในแท็บใหม่">เปิดแชท</a></div>}
    <label className="shipping-message-label">ข้อความที่จะส่งให้ลูกค้า<textarea autoComplete="off" rows={6} maxLength={5000} value={message} disabled={busy || locked || batchBusy} onChange={e => { setMessage(e.target.value); setEditedMessage(true); setDirty(true); }} /></label>
    <div className="shipping-parcel-actions">
      {saved?.status !== "SENT" && <button type="button" className="button primary" disabled={busy || batchBusy || !hasRecipient || !recipientName || !validTracking(number) || (pending && !stale) || !!retryExpired} onClick={() => void send()}>{sending ? "กำลังส่งข้อความ…" : unknown ? "ลองส่งคำขอเดิม" : "ส่งข้อความให้ลูกค้าคนนี้"}</button>}
      {!locked && <button type="button" className="button secondary" disabled={busy || batchBusy || !hasRecipient || !recipientName || !validTracking(number)} onClick={() => void save()}>{busy && !sending ? "กำลังบันทึก…" : "บันทึก Tracking ไว้ก่อน"}</button>}
      {!locked && <button type="button" className="text-button" disabled={busy || batchBusy} onClick={() => { setMessage(trackingMessage(number)); setEditedMessage(false); setDirty(true); }}>ใช้ข้อความมาตรฐาน</button>}
      {saved && <button type="button" className="text-button" disabled={busy || batchBusy} onClick={() => void onSaved().catch(cause => setError(errorText(cause)))}>ตรวจผลล่าสุด</button>}
    </div>
    {!hasRecipient && !locked && <p className="shipping-description">เลือกลูกค้าหรือออเดอร์ด้านบนก่อนส่งข้อความ</p>}
    <details className="shipping-evidence"><summary>ข้อความที่อ่านจากใบเสร็จ</summary><p>{parcel.raw_text}</p></details>
    {saved?.status === "SENT" && <p className="shipping-sent" role="status">ส่ง Tracking {saved.tracking_number} ให้ลูกค้าแล้ว{saved.sent_at ? ` · ${formatDateTime(saved.sent_at)}` : ""}{saved.sent_by ? ` · ${saved.sent_by}` : ""}</p>}
    {pending && <p role="status" className="shipping-description">กำลังติดตามผลคำขอเดิม หากการเชื่อมต่อขาดช่วง ให้กดตรวจผลล่าสุด</p>}
    {retryExpired && saved?.status !== "SENT" && <p className="notice warning">คำขอนี้เกินช่วงเวลาติดตามอัตโนมัติแล้ว กรุณาตรวจข้อความใน LINE ก่อนดำเนินการต่อ</p>}
    {error && <p className="notice warning" role="alert">{error}</p>}
  </article>;
}

export default function ShippingWorkspace({ session, onLogout }: { session: AdminSession; onLogout: () => Promise<void> }) {
  useTeamViewport();
  const [receipts, setReceipts] = useState<Receipt[]>([]); const [next, setNext] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null); const [result, setResult] = useState<ReceiptResult | null>(null);
  const [busy, setBusy] = useState(false); const [loading, setLoading] = useState(true); const [error, setError] = useState("");
  const [hasPendingUpload, setHasPendingUpload] = useState(false);
  const [notice, setNotice] = useState("");
  const deleted = useRef(new Set<string>());
  const file = useRef<HTMLInputElement>(null); const pendingUpload = useRef<{ request_id: string; name: string; image_base64: string } | null>(null);
  const sequence = useRef(0);
  const remember = useCallback((value: ReceiptResult) => { if (deleted.current.has(value.receipt._id)) return; setResult(value); setReceipts(current => [value.receipt, ...current.filter(r => r._id !== value.receipt._id)].sort((a, b) => b._id.localeCompare(a._id))); }, []);
  const reload = useCallback(async () => { if (selected) { const seq = sequence.current; try { const value = await api<ReceiptResult>(`${endpoint}/receipts/${selected}`); if (seq === sequence.current) remember(value); } catch (cause) { if (seq === sequence.current && !deleted.current.has(selected)) throw cause; } } }, [selected, remember]);
  useEffect(() => { let cancelled = false; void api<{ receipts: Receipt[]; next: string | null }>(`${endpoint}/receipts`).then(value => { if (!cancelled) { setReceipts(value.receipts.filter(r => !deleted.current.has(r._id))); setNext(value.next); setSelected(current => current ?? value.receipts[0]?._id ?? null); } }).catch(cause => { if (!cancelled) setError(errorText(cause)); }).finally(() => { if (!cancelled) setLoading(false); }); return () => { cancelled = true; }; }, []);
  useEffect(() => { const current = ++sequence.current; if (!selected) return; void api<ReceiptResult>(`${endpoint}/receipts/${selected}`).then(value => { if (current === sequence.current) remember(value); }).catch(cause => { if (current === sequence.current) setError(errorText(cause)); }); return () => { sequence.current = current + 1; }; }, [selected, remember]);
  const processing = result?.receipt.status === "QUEUED" || result?.receipt.status === "READING" || result?.trackings.some(t => t.status === "SENDING");
  useEffect(() => { if (!processing) return; const timer = setInterval(() => void reload().catch(cause => setError(errorText(cause))), 2500); return () => clearInterval(timer); }, [processing, reload]);
  async function upload(selectedFile?: File) {
    if (busy) return; setBusy(true); setError("");
    try {
      if (selectedFile) { pendingUpload.current = { ...await prepareReceipt(selectedFile), request_id: crypto.randomUUID() }; setHasPendingUpload(true); }
      if (!pendingUpload.current) return;
      const value = await request<ReceiptResult>(`${endpoint}/receipts`, { method: "POST", body: JSON.stringify(pendingUpload.current) });
      pendingUpload.current = null; setHasPendingUpload(false); setSelected(value.receipt._id); remember(value);
    } catch (cause) { setError(cause instanceof ApiError ? errorText(cause) : cause instanceof Error && cause.name !== "AbortError" ? cause.message : "การเชื่อมต่อขัดข้อง ลองติดตามการอัปโหลดด้วยคำขอเดิมได้"); }
    finally { setBusy(false); if (file.current) file.current.value = ""; }
  }
  async function more() { try { const value = await api<{ receipts: Receipt[]; next: string | null }>(`${endpoint}/receipts?before=${next}`); setReceipts(current => [...new Map([...current, ...value.receipts].filter(r => !deleted.current.has(r._id)).map(r => [r._id, r])).values()]); setNext(value.next); } catch (cause) { setError(errorText(cause)); } }
  async function extract() { if (!selected || busy) return; setBusy(true); setError(""); try { remember(await api<ReceiptResult>(`${endpoint}/receipts/${selected}/extract`, { method: "POST" })); } catch (cause) { setError(errorText(cause)); } finally { setBusy(false); } }
  async function deleteBatch() {
    if (!selected || busy || !result || result.receipt._id !== selected) return;
    const target = selected;
    if (!window.confirm(`ลบทั้ง batch "${result.receipt.name}" รวมพัสดุทุกรายการออกจากหน้าพัสดุหรือไม่?\nออเดอร์และข้อความที่ส่งในแชทแล้วจะยังอยู่ ระบบเก็บประวัติการส่งเพื่อป้องกันส่งซ้ำ`)) return;
    setBusy(true); setError(""); setNotice("");
    try {
      await request(`${endpoint}/receipts/${target}/delete`, { method: "POST" });
      deleted.current.add(target); sequence.current += 1;
      setReceipts(current => current.filter(receipt => receipt._id !== target));
      setResult(null); setSelected(receipts.find(receipt => receipt._id !== target)?._id ?? null);
      setNotice("ลบทั้ง batch ออกจากหน้าพัสดุแล้ว");
    } catch (cause) { setError(cause instanceof ApiError && cause.code === "RECORD_CONFLICT" ? "batch นี้กำลังบันทึกหรือส่ง Tracking อยู่ กรุณารอแล้วลองลบอีกครั้ง" : errorText(cause)); }
    finally { setBusy(false); }
  }
  return <div className="app-shell team-shell shipping-shell"><WorkspaceSidebar username={session.username} login={session.login} activePanel="inquiries" shippingActive onNavigate={() => navigateWorkspace("chats")} onHome={() => navigateWorkspace("home")} onManageTeam={canManageTeam(session) ? () => navigateWorkspace("team") : undefined} onLogout={onLogout} /><main className="main-content">
    <header className="page-header"><div><p className="eyebrow">Arisa Rental</p><h1>พัสดุและ Tracking</h1><p className="shipping-description">อ่านใบเสร็จ แล้วเลือกลูกค้าเพื่อส่ง Tracking</p></div><div className="header-actions"><button className="button primary" disabled={busy || !session.hermes_configured} onClick={() => file.current?.click()}><WorkspaceIcon name="shipping" />{busy ? "กำลังอัปโหลด…" : "อัปโหลดใบเสร็จ"}</button><input ref={file} type="file" accept="image/jpeg,image/png,image/webp,image/heic,image/heif" hidden aria-label="อัปโหลดรูปใบเสร็จขนส่ง" disabled={busy} onChange={event => void upload(event.target.files?.[0])} /></div></header>
    <div className="shipping-content">{notice && <p className="notice" role="status">{notice}</p>}{!session.hermes_configured && <p className="notice warning">ยังไม่ได้ตั้งค่า AI อ่านรูป กรุณาติดต่อผู้ดูแลระบบ</p>}{error && <div className="notice warning" role="alert">{error}{hasPendingUpload && <button type="button" className="text-button" disabled={busy} onClick={() => void upload()}>ติดตามอัปโหลดด้วยคำขอเดิม</button>}</div>}
      <div className="shipping-workflow" aria-label="ขั้นตอนพัสดุ"><span>1. อัปโหลดใบเสร็จ</span><span>2. ตรวจชื่อและเลือกลูกค้า</span><span>3. บันทึกและส่ง Tracking</span></div>
      <nav className="shipping-receipts" aria-label="ใบเสร็จที่อัปโหลด">{receipts.map(receipt => <button type="button" key={receipt._id} aria-current={selected === receipt._id ? "page" : undefined} disabled={busy} onClick={() => { setSelected(receipt._id); setError(""); }}><strong>{receipt.name}</strong><span>{statuses[receipt.status]} · {formatDateTime(receipt.created_at)}</span></button>)}{next && <button type="button" disabled={busy} onClick={() => void more()}>โหลดใบเสร็จเก่า</button>}</nav>
      {loading && <p role="status">กำลังโหลดใบเสร็จ…</p>}{!loading && !receipts.length && <section className="shipping-empty"><WorkspaceIcon name="shipping" /><h2>ส่งเลขพัสดุให้ลูกค้าได้จากใบเสร็จเดียว</h2><p>อัปโหลดภาพไปรษณีย์ไทยที่เห็นชื่อผู้รับ เลข Tracking และค่าขนส่งครบ ระบบจะแยกพัสดุแต่ละรายให้ตรวจสอบ</p><button type="button" className="button primary" disabled={busy || !session.hermes_configured} onClick={() => file.current?.click()}>เลือกรูปใบเสร็จ</button></section>}
      {result && result.receipt._id === selected && <section className="shipping-review" aria-label="ตรวจใบเสร็จขนส่ง"><div className="shipping-receipt-summary"><div><h2>{result.receipt.extraction?.receipt_number ? `ใบเสร็จ ${result.receipt.extraction.receipt_number}` : result.receipt.name}</h2><p className="shipping-description">{result.receipt.extraction?.receipt_date && `${formatDate(result.receipt.extraction.receipt_date)} · `}อัปโหลดโดย {result.receipt.created_by}</p>{result.receipt.extraction && <p>{result.receipt.extraction.parcels.length} พัสดุ · รวมทั้งใบเสร็จ {money(result.receipt.extraction.total_satang)}</p>}</div><div className="shipping-batch-actions">{result.receipt.image_url && <ImagePreview src={result.receipt.image_url} alt={`รูปใบเสร็จ ${result.receipt.name}`} label="ดูรูปใบเสร็จต้นฉบับ" className="button secondary" />}<button type="button" className="button secondary shipping-delete-batch" disabled={busy} onClick={() => void deleteBatch()}>ลบทั้ง batch</button></div></div>
        {["QUEUED", "READING"].includes(result.receipt.status) && <p className="shipping-reading" role="status"><span className="bill-feedback-spinner" aria-hidden="true" />{statuses[result.receipt.status]} เก็บภาพแล้ว สามารถกลับมาติดตามในเมนูนี้ได้</p>}
        {result.receipt.status === "FAILED" && <div className="notice warning" role="alert"><p>อ่านใบเสร็จไม่สำเร็จ ({result.receipt.error_code}) ภาพยังอยู่ ลองอ่านอีกครั้งหรืออัปโหลดภาพที่ชัดขึ้น</p><button className="button secondary" type="button" disabled={busy} onClick={() => void extract()}>อ่านรูปนี้อีกครั้ง</button></div>}
        {result.receipt.total_mismatch && <p className="notice warning">ยอดค่าขนส่งรายพัสดุไม่ตรงยอดรวมบนใบเสร็จ โปรดตรวจค่าบริการเพิ่มเติมจากภาพต้นฉบับ</p>}
        {result.receipt.status === "READY" && <><p className="shipping-description">ตรวจผู้รับและเลข Tracking จากใบเสร็จก่อนส่งข้อความ</p>{result.receipt.extraction?.parcels.map(parcel => { const saved = result.trackings.find(t => t.row === parcel.row); return <ParcelCard key={`${result.receipt._id}-${parcel.row}-${saved?.version ?? 0}`} parcel={parcel} receiptId={result.receipt._id} saved={saved} onSaved={reload} batchBusy={busy} />; })}</>}
      </section>}
    </div><WorkspaceNavigation activePage="shipping" manageTeam={canManageTeam(session)} /></main></div>;
}

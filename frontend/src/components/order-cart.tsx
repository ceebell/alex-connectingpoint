"use client";
/* eslint-disable @next/next/no-img-element -- Catalog images are external Alex links; next/image would need a fixed remote host list. */
import { KeyboardEvent, useEffect, useState } from "react";
import { api, ApiError, Attachment, Availability, Candidate, CartItem, Draft, DraftItem, errorText, LineEvent } from "@/lib/api";

type Search = { search_id: string; branch_id: string; item_index?: number; page: number; has_more: boolean; method: string; candidates: Candidate[]; mode?: Mode; query?: string | null; color?: string | null; size?: string | null; category?: string | null };
type Category = { code: string; name: string };
type Suggestion = { item_index: number; status: string; selected_product_item_id?: string; search?: Search };
type Settings = { configured: boolean; branches: { id: string; name: string }[]; image_search_configured: boolean; product_url_configured: boolean };
type Mode = "code" | "text" | "product_url" | "image";
// Alex shows catalog prices as plain baht, e.g. 600 or 1,500.
export const price = (value: number | null) => value === null ? "ไม่ระบุ" : (value / 100).toLocaleString("th-TH", { maximumFractionDigits: 2 });
const quantityLimit = (value: number) => Math.min(999, Math.max(1, Math.round(value) || 1));
const date = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", timeZone: "Asia/Bangkok" }).format(new Date(value));
const lookupStates: Record<string, string> = { MATCHED: "พบตรงหนึ่งรายการ", NEEDS_SELECTION: "พบหลายตัวเลือก", NOT_FOUND: "ยังไม่พบใน Alex", BRANCH_REQUIRED: "ต้องเลือกสาขา", MISSING_QUERY: "ไม่มีชื่อหรือรหัส", NOT_CONFIGURED: "ยังไม่เชื่อม Alex", FAILED: "ค้นอัตโนมัติไม่สำเร็จ", EXPIRED: "ผลค้นหาหมดอายุ กรุณาค้นใหม่" };
const queueStates: Record<string, string> = { AVAILABLE: "ว่าง ณ เวลาตรวจ", UNAVAILABLE: "ไม่ว่าง", UNKNOWN: "ยังตรวจไม่ได้" };

function CartCard({ item, index, disabled, onQuantity, onRemove }: { item: CartItem; index: number; disabled: boolean; onQuantity: (quantity: number) => void; onRemove: () => void }) {
  const [text, setText] = useState(String(item.quantity));
  const { candidate } = item;
  function commit() { const next = quantityLimit(Number(text)); setText(String(next)); if (next !== item.quantity) onQuantity(next); }
  function step(next: number) { setText(String(next)); onQuantity(next); }
  return <article className="match-card" aria-label={`สินค้าในตะกร้า ${index + 1} ${candidate.name}`}>
    <span className="match-number" aria-hidden="true">{index + 1}</span>
    <div className="match-side">
      <div className="match-image">{candidate.image_url ? <img src={candidate.image_url} alt={`รูปสินค้า ${candidate.name}`} loading="lazy" referrerPolicy="no-referrer" onError={e => { e.currentTarget.hidden = true; }} /> : <span aria-hidden="true">👗</span>}</div>
      <p>สี: {candidate.color || "ไม่ระบุ"}</p><p>ไซซ์: {candidate.size || "ไม่ระบุ"}</p>
    </div>
    <div className="match-main">
      <div className="match-title"><strong>{candidate.name}</strong><span>({candidate.code})</span></div>
      <dl className="match-price"><dt>ราคา:</dt><dd>{price(candidate.rental_fee_satang)}</dd><dt>ค่าประกัน:</dt><dd>{price(candidate.deposit_satang)}</dd></dl>
    </div>
    <div className="quantity-stepper" role="group" aria-label={`จำนวน ${candidate.name}`}>
      <button type="button" aria-label="ลดจำนวน" disabled={disabled || item.quantity <= 1} onClick={() => step(item.quantity - 1)}>−</button>
      <input type="number" min={1} max={999} aria-label="จำนวน" value={text} disabled={disabled} onChange={e => setText(e.target.value)} onBlur={commit} onKeyDown={e => { if (e.key === "Enter") { e.preventDefault(); commit(); } }} />
      <button type="button" aria-label="เพิ่มจำนวน" disabled={disabled || item.quantity >= 999} onClick={() => step(item.quantity + 1)}>+</button>
    </div>
    <button type="button" className="match-remove" disabled={disabled} onClick={onRemove}><span aria-hidden="true">✕</span> เอาออก</button>
  </article>;
}

function CandidateTile({ candidate, inCart, disabled, onAdd }: { candidate: Candidate; inCart: number; disabled: boolean; onAdd: () => void }) {
  return <article className="candidate-tile">
    <div className="candidate-image">{candidate.image_url ? <img src={candidate.image_url} alt={`รูปสินค้า ${candidate.name}`} loading="lazy" referrerPolicy="no-referrer" onError={e => { e.currentTarget.hidden = true; }} /> : <span aria-hidden="true">👗</span>}</div>
    <strong>{candidate.name}</strong>
    <span className="candidate-code">({candidate.code})</span>
    <p>สี: {candidate.color || "ไม่ระบุ"} · ไซซ์: {candidate.size || "ไม่ระบุ"}</p>
    <p className="candidate-price">ค่าเช่า {price(candidate.rental_fee_satang)} · ค่าประกัน {price(candidate.deposit_satang)}</p>
    {candidate.score !== null && <p className="muted small">ความคล้าย {Math.round(candidate.score * 100)}%</p>}
    <button type="button" className="button secondary full" disabled={disabled} onClick={onAdd}>{inCart ? `+ เพิ่มอีก (มี ${inCart})` : "+ เพิ่มลงตะกร้า"}</button>
  </article>;
}

// Searches Alex and keeps the inquiry cart; the cart is independent of draft versions.
export default function OrderCart({ inquiryId, draft, items, messages, attachments, disabled, onCart, onUnauthorized }: { inquiryId: string; draft: Draft | undefined; items: DraftItem[]; messages: LineEvent[]; attachments: Attachment[]; disabled: boolean; onCart: (items: CartItem[]) => void; onUnauthorized: () => void }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [cart, setCart] = useState<CartItem[]>([]);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [branch, setBranch] = useState("");
  const [mode, setMode] = useState<Mode>(items[0]?.product_code ? "code" : "text");
  const [query, setQuery] = useState(items[0]?.product_code ?? items[0]?.product_name ?? "");
  const [color, setColor] = useState(items[0]?.color ?? "");
  const [size, setSize] = useState(items[0]?.size ?? "");
  // Alex category code; a poster title like "Haatiya" is a category, not a product name.
  const [category, setCategory] = useState("");
  const [categories, setCategories] = useState<Category[]>([]);
  const [texture, setTexture] = useState("");
  const [image, setImage] = useState("");
  const [result, setResult] = useState<Search | null>(null);
  const [choices, setChoices] = useState<(Candidate & { search_id: string })[]>([]);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const working = disabled || busy || loading || !settings?.configured;
  const images = [...messages.filter(message => !message.withdrawn && message.payload?.message?.type === "image" && message.media_id), ...attachments.map(attachment => ({ _id: attachment.id, event_at: attachment.created_at }))];
  const rentalTotal = cart.reduce((sum, item) => sum + (item.candidate.rental_fee_satang ?? 0) * item.quantity, 0);
  const depositTotal = cart.reduce((sum, item) => sum + (item.candidate.deposit_satang ?? 0) * item.quantity, 0);

  function fail(cause: unknown) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); }
  function keep(value: { items: CartItem[] }) { setCart(value.items); onCart(value.items); }
  function show(search: Search) { setResult(search); setCategory(search.category ?? ""); setChoices(search.candidates.map(candidate => ({ ...candidate, search_id: search.search_id }))); }
  const categoryCode = (name: string | null | undefined, list = categories) => { const wanted = name?.trim().toLowerCase(); return wanted ? list.find(value => value.name.toLowerCase() === wanted || value.code.toLowerCase() === wanted)?.code ?? "" : ""; };

  useEffect(() => {
    let cancelled = false;
    Promise.all([
      api<Settings>("/api/admin/alex"),
      api<{ items: CartItem[] }>(`/api/inquiries/${inquiryId}/cart`),
      draft ? api<{ suggestions?: Suggestion[] }>(`/api/inquiries/${inquiryId}/products`).catch(() => ({ suggestions: [] })) : Promise.resolve({ suggestions: [] }),
      api<{ categories: Category[] }>("/api/admin/alex/categories").catch(() => ({ categories: [] })),
    ]).then(([settings, cart, found, listed]) => {
      if (cancelled) return;
      setSettings(settings); setCart(cart.items); onCart(cart.items); setCategories(listed.categories);
      setCategory(categoryCode(items[0]?.category, listed.categories));
      const usable = (found.suggestions ?? []).filter(suggestion => suggestion.search?.candidates.length);
      setSuggestions(found.suggestions ?? []);
      const first = usable[0]?.search;
      setBranch(first?.branch_id ?? (settings.branches.length === 1 ? settings.branches[0].id : ""));
      if (first) show(first);
    }).catch(cause => { if (!cancelled) fail(cause); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [inquiryId, draft?._id]); // eslint-disable-line react-hooks/exhaustive-deps

  function pickRequest(item: DraftItem, index: number) {
    const suggestion = suggestions.find(value => value.item_index === index && value.search?.candidates.length);
    setMode(item.product_code ? "code" : "text"); setQuery(item.product_code ?? item.product_name ?? ""); setCategory(categoryCode(item.category)); setColor(item.color ?? ""); setSize(item.size ?? ""); setError("");
    if (suggestion?.search) { setBranch(suggestion.search.branch_id); show(suggestion.search); } else { setResult(null); setChoices([]); }
  }

  async function search(next = false) {
    if (working || !branch || (mode === "image" && !image) || ((mode === "code" || mode === "product_url") && !query.trim())) return;
    setBusy(true); setError("");
    try {
      const value = await api<Search>(`/api/inquiries/${inquiryId}/products/search`, { method: "POST", body: JSON.stringify({
        branch_id: branch, mode, query: query.trim() || null, color: color.trim() || null, size: size.trim() || null,
        category: category.trim() || null, texture: texture.trim() || null, image_event_id: mode === "image" ? image : null,
        page: next && result ? result.page + 1 : 1,
      }) });
      setResult(value);
      const incoming = value.candidates.map(candidate => ({ ...candidate, search_id: value.search_id }));
      setChoices(previous => [...new Map([...(next ? previous : []), ...incoming].map(candidate => [candidate.product_item_id, candidate])).values()]);
    } catch (cause) { fail(cause); } finally { setBusy(false); }
  }

  async function add(candidate: Candidate & { search_id: string }) {
    if (working) return;
    setBusy(true); setError("");
    try { keep(await api<{ items: CartItem[] }>(`/api/inquiries/${inquiryId}/cart`, { method: "POST", body: JSON.stringify({ search_id: candidate.search_id, product_item_id: candidate.product_item_id, quantity: 1 }) })); }
    catch (cause) { fail(cause); } finally { setBusy(false); }
  }

  async function update(item: CartItem, quantity: number | null) {
    if (working) return;
    setBusy(true); setError("");
    try { keep(await api<{ items: CartItem[] }>(`/api/inquiries/${inquiryId}/cart/item`, { method: "POST", body: JSON.stringify({ item_id: item._id, expected_revision: item.revision, quantity }) })); }
    catch (cause) { fail(cause); await api<{ items: CartItem[] }>(`/api/inquiries/${inquiryId}/cart`).then(keep).catch(() => {}); }
    finally { setBusy(false); }
  }

  const enter = (e: KeyboardEvent) => { if (e.key === "Enter") { e.preventDefault(); void search(); } };
  return <div className="order-cart">
    <section className="order-section">
      <h3 className="form-heading"><span aria-hidden="true">🔎</span> ค้นหาชุดจาก Alex</h3>
      {loading && <p role="status" className="muted small">กำลังโหลดตะกร้าและสาขา…</p>}
      {settings && !settings.configured && <div className="notice warning">ยังไม่เชื่อมบัญชี Alex กรุณาให้ผู้ดูแลระบบตั้งค่าการเชื่อมต่อก่อนค้นสินค้า</div>}
      {error && <div className="notice warning" role="alert">{error}</div>}
      {items.length > 0 && <div className="request-list"><span className="muted small">สิ่งที่ลูกค้าขอจากรูป/สรุป (กดเพื่อค้น):</span>{items.map((item, index) => { const lookup = suggestions.find(value => value.item_index === index); return <button type="button" key={index} className="request-chip" disabled={working} onClick={() => pickRequest(item, index)}>{[item.category && `ประเภท ${item.category}`, item.product_code || item.product_name, item.color, item.size].filter(Boolean).join(" · ") || "ไม่มีชื่อ"}{lookup && <span> · {lookupStates[lookup.status] ?? lookup.status}</span>}</button>; })}</div>}
      <div className="field-grid search-grid">
        {(settings?.branches.length ?? 0) > 1 && <label className="span-2">สาขา Alex<select value={branch} disabled={working} onChange={e => setBranch(e.target.value)}><option value="">เลือกสาขา</option>{settings?.branches.map(branch => <option key={branch.id} value={branch.id}>{branch.name}</option>)}</select></label>}
        <label>ค้นหาจาก<select value={mode} disabled={working} onChange={e => setMode(e.target.value as Mode)}><option value="text">ชื่อหรือคำบรรยาย</option><option value="code">รหัสสินค้าแบบตรงตัว</option><option value="product_url" disabled={!settings?.product_url_configured}>ลิงก์สินค้า</option><option value="image" disabled={!settings?.image_search_configured}>รูปจากแชท/รูปแนบ</option></select></label>
        {mode === "image" ? <label>รูปอ้างอิง<select value={image} disabled={working} onChange={e => setImage(e.target.value)}><option value="">เลือกรูป</option>{images.map((image, i) => <option key={image._id} value={image._id}>รูปที่ {i + 1} · {date(image.event_at)}</option>)}</select></label>
          : <label>{mode === "product_url" ? "ลิงก์สินค้า" : mode === "code" ? "รหัสสินค้า" : "ชื่อชุด"}<input maxLength={500} value={query} disabled={working} onKeyDown={enter} onChange={e => setQuery(e.target.value)} placeholder={mode === "text" ? "เว้นว่างเพื่อดูทั้งหมดในประเภท/สาขา" : "เช่น 0291FRLDBRM"} /></label>}
        <label className="span-2">ประเภท<select value={category} disabled={working} onChange={e => setCategory(e.target.value)}><option value="">ทุกประเภท</option>{categories.map(value => <option key={value.code} value={value.code}>{value.name} ({value.code})</option>)}{category && !categories.some(value => value.code === category) && <option value={category}>{category}</option>}</select></label>
        <label>สี<input maxLength={200} value={color} disabled={working} onKeyDown={enter} onChange={e => setColor(e.target.value)} placeholder="เช่น น้ำตาล" /></label>
        <label>ไซซ์<input maxLength={200} value={size} disabled={working} onKeyDown={enter} onChange={e => setSize(e.target.value)} placeholder="เช่น M" /></label>
      </div>
      <details className="product-filters"><summary>ตัวกรองเพิ่มเติม</summary><div className="field-grid"><label>ลาย/เนื้อผ้า (รหัส)<input maxLength={200} value={texture} disabled={working} onKeyDown={enter} onChange={e => setTexture(e.target.value)} /></label></div></details>
      <button type="button" className="button secondary full" disabled={working || !branch || (mode === "image" && !image)} onClick={() => search()}>{busy ? "กำลังค้นหา…" : "ค้นหาสินค้าใน Alex"}</button>
      {result && <p className="muted small">พบ {choices.length} ตัวเลือก · หน้าที่ {result.page}{result.method === "admin_input" ? " · ผลค้นอัตโนมัติจากรูป/สรุป" : result.method === "image_code" ? " · อ่านรหัสจากรูปแล้วยืนยันใน Alex" : result.method === "image_similarity" ? " · เรียงตามความคล้ายของรูป" : ""}</p>}
      {result && !choices.length && <div className="notice info">ยังไม่พบตัวเลือก ลองปรับชื่อ สี หรือไซซ์{result.has_more ? " หรือโหลดหน้าถัดไป" : ""}</div>}
      {choices.length > 0 && <div className="candidate-grid">{choices.map(candidate => <CandidateTile key={`${candidate.search_id}:${candidate.product_item_id}`} candidate={candidate} inCart={cart.find(item => item.product_item_id === candidate.product_item_id)?.quantity ?? 0} disabled={working} onAdd={() => add(candidate)} />)}</div>}
      {result?.has_more && <button type="button" className="text-button full" disabled={working} onClick={() => search(true)}>โหลดหน้าถัดไป</button>}
    </section>
    <section className="order-section">
      <h3 className="form-heading"><span aria-hidden="true">🛒</span> ตะกร้าสินค้า ({cart.reduce((sum, item) => sum + item.quantity, 0)} ชิ้น)</h3>
      {!cart.length ? <p className="muted small">ยังไม่มีสินค้าในตะกร้า ค้นหาแล้วกด “เพิ่มลงตะกร้า”</p> : <>
        <div className="cart-list">{cart.map((item, index) => <CartCard key={`${item._id}-${item.revision}`} item={item} index={index} disabled={working} onQuantity={quantity => update(item, quantity)} onRemove={() => update(item, null)} />)}</div>
        <p className="cart-total">ค่าเช่ารวม {price(rentalTotal)} บาท · ค่าประกันรวม {price(depositTotal)} บาท</p>
      </>}
    </section>
  </div>;
}

// Checks the cart against Alex for the dates on screen; it reads only and reserves nothing.
export function QueueCheck({ inquiryId, pickup, returned, count, disabled, onUnauthorized }: { inquiryId: string; pickup: string | null; returned: string | null; count: number; disabled: boolean; onUnauthorized: () => void }) {
  const [result, setResult] = useState<Availability | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  async function check() {
    setBusy(true); setError("");
    try { setResult(await api<Availability>(`/api/inquiries/${inquiryId}/cart/availability`, { method: "POST", body: JSON.stringify({ pickup_date: pickup, return_date: returned }) })); }
    catch (cause) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); } finally { setBusy(false); }
  }
  const ready = !!pickup && !!returned && count > 0;
  return <div className="queue-check">
    <button type="button" className="button secondary full" disabled={disabled || busy || !ready} onClick={check}><span aria-hidden="true">📅</span> {busy ? "กำลังเช็คคิว…" : "เช็คคิวจาก Alex"}</button>
    {!ready && <p className="muted small">ใส่วันรับชุด วันคืนชุด และเพิ่มสินค้าลงตะกร้าก่อนเช็คคิว</p>}
    {error && <div className="notice warning" role="alert">{error}</div>}
    {result && <div className={`notice ${result.state === "AVAILABLE" ? "success" : "warning"}`} role="status"><strong>{queueStates[result.state] ?? result.state}</strong>{result.items.map(item => <p key={item.item_index}>{item.code} × {item.quantity}: {queueStates[item.state] ?? item.state}</p>)}<p className="small">ผลตรวจยังไม่กันคิว ระบบจะตรวจซ้ำก่อนส่งออเดอร์</p></div>}
  </div>;
}

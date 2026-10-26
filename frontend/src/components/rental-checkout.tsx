"use client";
/* eslint-disable @next/next/no-img-element -- Catalog images are external Alex links; next/image would need a fixed remote host list. */
import { useEffect, useState } from "react";
import { api, ApiError, cleanPhone, codeLabel, CodeName, errorText, PAYMENT_METHODS, Profile, RECEIVING_METHODS, STORE_PICKUP, UNPAID } from "@/lib/api";
import CustomerExtractButton from "./customer-extract-button";

type Line = { item_index: number; product_item_id: string; code: string; name: string; color: string | null; size: string | null; quantity: number; rental_fee_satang: number; deposit_satang: number; image_url?: string | null };
type Group = { key: string; branch_id: string; pickup_date: string; return_date: string; fulfillment_method: string; delivery_provider: string | null; lines: Line[]; notes: string; quoted_rental_fee_satang: number | null; quoted_deposit_satang: number | null; quoted_shipping_fee_satang: number | null; rental_discount_satang: number; deposit_discount_satang: number };
type Submission = { _id: string; status: string; alex_order_id?: string; group: Group; submitted_by: string; created_at: string; verification?: string };
type Data = { source_hash: string | null; draft_version?: number; groups: Group[]; missing: { field: string; code?: string }[]; has_conflicts: boolean; context_error?: string; submissions: Submission[]; customer: { name: string | null; phone: string | null; address: string | null }; notes: string | null; order_response_configured: boolean };
type Availability = { state: string; checked_at: string; items: { item_index: number; product_item_id: string; code: string; quantity: number; state: string }[] };
type Review = { review_id: string; group: Group; customer: { name: string; phone: string; address: string }; catalog_rental_satang: number; catalog_deposit_satang: number; rental_fee_satang: number; deposit_satang: number; shipping_fee_satang: number; rental_discount_satang: number; deposit_discount_satang: number; receiving_method: string; rental_payment_method: string; bail_payment_method: string; notes: string; expires_at: string };
const baht = (value: number) => (value / 100).toLocaleString("th-TH", { maximumFractionDigits: 2 });
const nights = (pickup: string, returned: string) => Math.max(1, Math.round((Date.parse(returned) - Date.parse(pickup)) / 86_400_000));
const badges: Record<string, [string, string]> = { AVAILABLE: ["ว่าง", "free"], UNAVAILABLE: ["ติดคิว", "blocked"], UNKNOWN: ["ตรวจคิวไม่ได้", "unknown"] };
const day = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", year: "numeric", timeZone: "Asia/Bangkok" }).format(new Date(`${value}T00:00:00+07:00`));
const states: Record<string, string> = { AVAILABLE: "ว่าง ณ เวลาตรวจ", UNAVAILABLE: "ไม่ว่าง", UNKNOWN: "ยังตรวจไม่ได้", SUBMITTING: "กำลังส่ง / รอตรวจผล", SUBMIT_UNKNOWN: "ต้องตรวจผลใน Alex", CONFIRMED: "มีออเดอร์ใน Alex แล้ว", NOT_CREATED: "ยังไม่ได้สร้างออเดอร์" };
const missingLabels: Record<string, string> = { cart: "เพิ่มสินค้าลงตะกร้าในแท็บร่างล่าสุด", mixed_branches: "สินค้าในตะกร้าต้องอยู่สาขาเดียวกัน", pickup_date: "วันรับชุด", return_date: "วันคืนชุด", fulfillment_method: "วิธีมารับ", catalog_price: "ราคาแค็ตตาล็อก", overlapping_product: "Product ID ซ้ำและวันเช่าทับกัน กรุณารวมจำนวนหรือเลือกชิ้นอื่น" };

// The bill as an invoice: one order from the cart and the latest draft, checked and reviewed before it is sent to Alex.
export default function RentalCheckout({ inquiryId, userId, disabled, titleId, onUnauthorized }: { inquiryId: string; userId?: string; disabled: boolean; titleId?: string; onUnauthorized: () => void }) {
  const [data, setData] = useState<Data | null>(null);
  const [groupKey, setGroupKey] = useState("");
  const [name, setName] = useState(""); const [phone, setPhone] = useState(""); const [address, setAddress] = useState("");
  const [shipping, setShipping] = useState("");
  const [receiving, setReceiving] = useState(""); const [rentalPayment, setRentalPayment] = useState(UNPAID); const [bailPayment, setBailPayment] = useState(UNPAID);
  const [birth, setBirth] = useState(""); const [notes, setNotes] = useState("");
  const [freePromotion, setFreePromotion] = useState(false); const [discountPromotion, setDiscountPromotion] = useState(false);
  const [availability, setAvailability] = useState<Availability | null>(null);
  const [review, setReview] = useState<Review | null>(null); const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState(false); const [extracting, setExtracting] = useState(false); const [loading, setLoading] = useState(true); const [error, setError] = useState("");
  const [now, setNow] = useState(Date.now);
  // Lines carry Alex color codes (P); show the shop's color names next to them.
  const [colors, setColors] = useState<CodeName[]>([]);
  const [resolveId, setResolveId] = useState(""); const [resolveOrder, setResolveOrder] = useState(""); const [resolveChecked, setResolveChecked] = useState(false);
  const working = disabled || busy || extracting || loading;
  const group = data?.groups.find(group => group.key === groupKey);
  const pending = data?.submissions.some(submission => ["SUBMITTING", "SUBMIT_UNKNOWN"].includes(submission.status));
  const ready = !!group && !!data?.source_hash && !data.has_conflicts;
  const catalogRental = group?.lines.reduce((sum, line) => sum + line.rental_fee_satang * line.quantity, 0) ?? 0;
  const catalogDeposit = group?.lines.reduce((sum, line) => sum + line.deposit_satang * line.quantity, 0) ?? 0;
  const satang = (value: string) => Math.round(Number(value || 0) * 100);
  const endpoint = `/api/inquiries/${inquiryId}/checkout`;

  function invalidate() { setReview(null); setConfirmed(false); }
  function changeGroup(group: Group, commonNotes = data?.notes ?? "") {
    setGroupKey(group.key); setAvailability(null); invalidate();
    setShipping(group.quoted_shipping_fee_satang === null ? group.fulfillment_method === "store_pickup" ? "0" : "" : String(group.quoted_shipping_fee_satang / 100));
    setReceiving(group.fulfillment_method === "store_pickup" ? STORE_PICKUP : group.delivery_provider && RECEIVING_METHODS.includes(group.delivery_provider) ? group.delivery_provider : "Flash");
    setNotes([commonNotes, group.notes].filter(Boolean).join("\n"));
  }
  function fail(cause: unknown) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); }
  async function refresh() { const value = await api<Data>(endpoint); setData(value); return value; }

  useEffect(() => {
    let cancelled = false;
    Promise.all([api<Data>(endpoint), userId ? api<{ profile: Profile | null }>(`/api/line/users/${encodeURIComponent(userId)}/profile`).catch(() => null) : Promise.resolve(null), api<{ colors: CodeName[] }>("/api/admin/alex/colors").catch(() => ({ colors: [] }))]).then(([value, profile, shopColors]) => {
      if (cancelled) return;
      setColors(shopColors.colors);
      setData(value); setName(value.customer.name ?? profile?.profile?.display_name ?? ""); setPhone(cleanPhone(value.customer.phone ?? profile?.profile?.phone ?? "")); setAddress(value.customer.address ?? profile?.profile?.address ?? "");
      if (value.groups.length === 1) changeGroup(value.groups[0], value.notes ?? "");
    }).catch(cause => { if (!cancelled) fail(cause); }).finally(() => { if (!cancelled) setLoading(false); });
    const timer = setInterval(() => { api<Data>(endpoint).then(value => { if (!cancelled) { setData(value); setNow(Date.now()); } }).catch(() => {}); }, 10000);
    return () => { cancelled = true; clearInterval(timer); };
  }, [inquiryId]); // eslint-disable-line react-hooks/exhaustive-deps

  async function check() {
    if (!ready || working) return;
    setBusy(true); setError(""); invalidate();
    try { setAvailability(await api<Availability>(`${endpoint}/availability`, { method: "POST", body: JSON.stringify({ source_hash: data!.source_hash, group_key: groupKey }) })); }
    catch (cause) { fail(cause); } finally { setBusy(false); }
  }
  async function prepare() {
    if (!ready || working) return;
    setBusy(true); setError(""); invalidate();
    try { setReview(await api<Review>(`${endpoint}/review`, { method: "POST", body: JSON.stringify({
      source_hash: data!.source_hash, group_key: groupKey, customer_name: name, customer_phone: phone, customer_address: address,
      shipping_fee_satang: satang(shipping),
      receiving_method: receiving, rental_payment_method: rentalPayment, bail_payment_method: bailPayment, birth_date: birth || null, notes,
      use_free_item_promotion: freePromotion, use_discount_promotion: discountPromotion,
    }) })); }
    catch (cause) { fail(cause); } finally { setBusy(false); }
  }
  async function submit() {
    if (!review || !confirmed || working) return;
    setBusy(true); setError("");
    try { await api<Submission>(`${endpoint}/submit`, { method: "POST", body: JSON.stringify({ review_id: review.review_id, confirmed: true }) }); await refresh(); invalidate(); }
    catch (cause) { fail(cause); } finally { setBusy(false); }
  }
  async function resolve(outcome: "created" | "not_created") {
    if (!resolveId || !resolveChecked || working) return;
    setBusy(true); setError("");
    try { await api(`${endpoint}/resolve`, { method: "POST", body: JSON.stringify({ submission_id: resolveId, outcome, order_id: resolveOrder || null, verified_in_alex: true }) }); await refresh(); setResolveId(""); setResolveOrder(""); setResolveChecked(false); }
    catch (cause) { fail(cause); } finally { setBusy(false); }
  }

  const totals = review ? { rent: review.catalog_rental_satang, rentDiscount: review.rental_discount_satang, rentNet: review.rental_fee_satang, deposit: review.catalog_deposit_satang, depositDiscount: review.deposit_discount_satang, depositNet: review.deposit_satang, shipping: review.shipping_fee_satang }
    : group ? { rent: catalogRental, rentDiscount: group.rental_discount_satang, rentNet: catalogRental - group.rental_discount_satang, deposit: catalogDeposit, depositDiscount: group.deposit_discount_satang, depositNet: catalogDeposit - group.deposit_discount_satang, shipping: satang(shipping) } : null;
  const pieces = group?.lines.reduce((sum, line) => sum + line.quantity, 0) ?? 0;
  const expired = !!review && now > new Date(review.expires_at).getTime();

  return <div className="rental-checkout invoice">
    <header className="invoice-head">
      <div><p className="invoice-eyebrow">ใบสรุปออเดอร์</p><h3 id={titleId}>สรุปก่อนส่งเข้า Alex</h3><p className="muted small">เรื่อง #{inquiryId.slice(-6).toUpperCase()}{data?.draft_version ? ` · จากร่าง v${data.draft_version}` : ""}</p></div>
      {group && <dl className="invoice-period"><div><dt>วันรับชุด</dt><dd>{day(group.pickup_date)}</dd></div><div><dt>วันคืนชุด</dt><dd>{day(group.return_date)}</dd></div><div><dt>ระยะเช่า</dt><dd>{nights(group.pickup_date, group.return_date)} วัน</dd></div></dl>}
    </header>
    {loading && <p role="status" className="muted">กำลังโหลดรายการและประวัติส่งบิล…</p>}
    {error && <div className="notice warning" role="alert">{error}<button type="button" className="text-button" disabled={working} onClick={() => refresh().catch(fail)}>โหลดผลล่าสุด</button></div>}
    {data?.submissions.map(submission => <div key={submission._id} className={`notice ${submission.status === "CONFIRMED" ? "success" : submission.status === "NOT_CREATED" ? "info" : "warning"}`}><strong>{submission.status === "CONFIRMED" && <span aria-hidden="true">✅ </span>}{states[submission.status] ?? submission.status}</strong><p>{submission.group.lines.map(line => line.code).join(", ")} · {day(submission.group.pickup_date)} – {day(submission.group.return_date)}</p>{submission.alex_order_id && <p>เลขออเดอร์ Alex: <code>{submission.alex_order_id}</code></p>}
      {["SUBMITTING", "SUBMIT_UNKNOWN"].includes(submission.status) && <><p>ตรวจผลใน Alex ก่อน ระบบกันการสร้างบิลใหม่ไว้เพื่อป้องกันบิลซ้ำ</p><button type="button" className="text-button" onClick={() => { setResolveId(submission._id); setResolveChecked(false); setResolveOrder(""); }}>บันทึกผลที่ตรวจจาก Alex</button>
        {resolveId === submission._id && <div className="bill-resolve"><label>เลขออเดอร์ที่พบใน Alex<input value={resolveOrder} disabled={working} onChange={e => setResolveOrder(e.target.value)} /></label><label className="checkbox-row"><input type="checkbox" checked={resolveChecked} onChange={e => setResolveChecked(e.target.checked)} disabled={working} />ตรวจสถานะใน Alex ด้วยตัวเองแล้ว</label><button type="button" className="button secondary full" disabled={working || !resolveChecked || !resolveOrder.trim()} onClick={() => resolve("created")}>ยืนยันว่ามีออเดอร์นี้แล้ว</button><button type="button" className="text-button full" disabled={working || !resolveChecked || now - new Date(submission.created_at).getTime() < 120000} onClick={() => resolve("not_created")}>ตรวจแล้วว่าไม่ได้สร้างบิล</button><p className="muted small">กรณีไม่พบบิล ต้องรออย่างน้อย 2 นาทีแล้วตรวจ Alex ก่อนเปิดให้ส่งใหม่</p></div>}
      </>}
    </div>)}
    {data?.context_error && <div className="notice warning">ร่างหรือแชทเปลี่ยน กรุณาตรวจและบันทึกร่างล่าสุดก่อนเตรียมบิลใหม่</div>}
    {!!data?.missing.length && <div className="notice warning"><strong>ข้อมูลที่ยังต้องเติมในแท็บร่างล่าสุด</strong><ul>{data.missing.map((missing, i) => <li key={i}>{missingLabels[missing.field] ?? missing.field}{missing.code && ` (${missing.code})`}</li>)}</ul></div>}
    {data?.has_conflicts && !data.context_error && <div className="notice warning">มีข้อมูลขัดกันในร่าง กรุณาตรวจในแท็บร่างล่าสุดก่อน</div>}
    {group && totals && <form className="invoice-body" onSubmit={e => { e.preventDefault(); void prepare(); }}>
      <fieldset disabled={working || !ready || !!pending}>
        <section className="invoice-block">
          <h4><span aria-hidden="true">👤</span> ลูกค้า</h4>
          <div className="field-grid">
            <label>ชื่อลูกค้า<input required value={name} maxLength={200} onChange={e => { setName(e.target.value); invalidate(); }} /></label><label>เบอร์โทร<input required value={phone} maxLength={40} inputMode="tel" onChange={e => { setPhone(cleanPhone(e.target.value)); invalidate(); }} /></label>
            <label className="span-2">ที่อยู่<textarea rows={2} required={group.fulfillment_method === "delivery"} value={address} maxLength={1000} onChange={e => { setAddress(e.target.value); invalidate(); }} /></label>
          </div>
          <CustomerExtractButton inquiryId={inquiryId} disabled={working} onBusyChange={setExtracting} onUnauthorized={onUnauthorized} onFill={value => { setName(current => value.display_name ?? current); setPhone(current => value.phone ? cleanPhone(value.phone) : current); setAddress(current => value.address ?? current); invalidate(); }} />
        </section>
        <section className="invoice-block">
          <h4><span aria-hidden="true">👗</span> รายการเช่า <span className="muted small">{pieces} ชิ้น</span></h4>
          <div className="invoice-items">{group.lines.map(line => { const state = availability?.items.find(item => item.product_item_id === line.product_item_id)?.state; const badge = state ? badges[state] : undefined; return <article key={line.item_index} className={`invoice-item ${badge ? `queue-${badge[1]}` : ""}`}>
            <div className="invoice-thumb">{line.image_url ? <img src={line.image_url} alt={`รูปสินค้า ${line.name}`} loading="lazy" referrerPolicy="no-referrer" onError={e => { e.currentTarget.hidden = true; }} /> : <span aria-hidden="true">👗</span>}</div>
            <div className="invoice-item-body"><strong>{line.name}</strong><span className="invoice-code">{line.code}</span><div className="invoice-chips"><span>สี {codeLabel(line.color, colors)}</span><span>ไซซ์ {line.size || "-"}</span><span>× {line.quantity}</span>{badge && <span className={`queue-badge ${badge[1]}`}>{badge[0]}</span>}</div></div>
            <dl className="invoice-item-amounts"><div><dt>ค่าเช่า</dt><dd>{baht(line.rental_fee_satang * line.quantity)}</dd></div><div><dt>ค่าประกัน</dt><dd>{baht(line.deposit_satang * line.quantity)}</dd></div>{line.quantity > 1 && <p className="muted small">ชิ้นละ {baht(line.rental_fee_satang)} / {baht(line.deposit_satang)}</p>}</dl>
          </article>; })}</div>
          <button type="button" className="button info full" disabled={working || !ready} onClick={check}><span aria-hidden="true">📅</span> {busy ? "กำลังตรวจ…" : "เช็คคิวจาก Alex"}</button>
          {availability && <p className={`invoice-queue ${availability.state === "AVAILABLE" ? "free" : "blocked"}`} role="status">{states[availability.state]} · ผลตรวจยังไม่กันคิว ระบบตรวจซ้ำก่อนส่งออเดอร์</p>}
        </section>
        <section className="invoice-block">
          <h4><span aria-hidden="true">💳</span> การรับและชำระเงิน</h4>
          <div className="field-grid">
            <label>วิธีมารับ<select required value={receiving} onChange={e => { setReceiving(e.target.value); invalidate(); }}>{RECEIVING_METHODS.map(method => <option key={method} value={method}>{method}</option>)}</select></label>
            <label>ค่าจัดส่ง (บาท)<input required type="number" min="0" step="0.01" value={shipping} onChange={e => { setShipping(e.target.value); invalidate(); }} /></label>
            {([["วิธีชำระค่าเช่า", rentalPayment, setRentalPayment], ["วิธีชำระค่าประกัน", bailPayment, setBailPayment]] as const).map(([label, value, set]) => <label key={label}>{label}<select value={value} onChange={e => { set(e.target.value); invalidate(); }}>{PAYMENT_METHODS.map(method => <option key={method} value={method}>{method}</option>)}</select></label>)}
            <label>วันเกิด (ถ้าทราบ)<input type="date" value={birth} onChange={e => { setBirth(e.target.value); invalidate(); }} /></label>
          </div>
          {Number(shipping) > 0 && <p className="notice info">ค่าจัดส่ง {baht(satang(shipping))} บาท จะบันทึกต่อท้ายหมายเหตุของออเดอร์ใน Alex (Alex ไม่มีช่องค่าส่งแยก)</p>}
          <label>หมายเหตุ / อุปกรณ์<textarea rows={2} maxLength={2000} value={notes} onChange={e => { setNotes(e.target.value); invalidate(); }} /></label>
          <div className="invoice-options"><label className="checkbox-row"><input type="checkbox" checked={freePromotion} onChange={e => { setFreePromotion(e.target.checked); invalidate(); }} />ใช้โปรโมชั่นของแถม</label><label className="checkbox-row"><input type="checkbox" checked={discountPromotion} onChange={e => { setDiscountPromotion(e.target.checked); invalidate(); }} />ใช้โปรโมชั่นส่วนลด</label></div>
        </section>
        <section className="invoice-totals" aria-label="สรุปยอด">
          <dl>
            <div><dt>ยอดเช่า</dt><dd>{baht(totals.rent)}</dd></div>
            <div className="discount"><dt>ส่วนลดค่าเช่า</dt><dd>−{baht(totals.rentDiscount)}</dd></div>
            <div className="subtotal"><dt>ค่าเช่าสุทธิ</dt><dd>{baht(totals.rentNet)}</dd></div>
            <div><dt>ยอดรวมค่าประกัน</dt><dd>{baht(totals.deposit)}</dd></div>
            <div className="discount"><dt>ส่วนลดค่าประกัน</dt><dd>−{baht(totals.depositDiscount)}</dd></div>
            <div className="subtotal"><dt>ค่าประกันสุทธิ</dt><dd>{baht(totals.depositNet)}</dd></div>
            {totals.shipping > 0 && <div><dt>ค่าจัดส่ง <span className="muted small">(หมายเหตุใน Alex)</span></dt><dd>{baht(totals.shipping)}</dd></div>}
          </dl>
          <div className="invoice-grand"><span>ยอดรวมทั้งหมด</span><strong>{baht(totals.rentNet + totals.depositNet + totals.shipping)} <small>บาท</small></strong></div>
          {(group.quoted_rental_fee_satang !== null || group.quoted_deposit_satang !== null) && <p className="muted small">ราคาที่แจ้งลูกค้า:{group.quoted_rental_fee_satang !== null && ` ค่าเช่า ${baht(group.quoted_rental_fee_satang)}`}{group.quoted_deposit_satang !== null && ` · ค่าประกัน ${baht(group.quoted_deposit_satang)}`} · ส่วนลดที่ไม่ได้กรอกคิดจากส่วนต่างกับราคาใน Alex</p>}
          {((group.quoted_rental_fee_satang ?? 0) > catalogRental || (group.quoted_deposit_satang ?? 0) > catalogDeposit) && <p className="notice warning">ราคาที่แจ้งลูกค้าสูงกว่าราคาใน Alex — Alex ไม่มีช่องบวกเพิ่ม บิลจะใช้ราคาใน Alex</p>}
          <p className="muted small">ส่วนลดแก้ได้ที่แท็บร่างล่าสุด{review ? " · ยอดนี้ตรวจกับราคาและคิวใน Alex แล้ว" : " · ยอดจะตรวจกับ Alex อีกครั้งเมื่อเตรียมบิล"}</p>
        </section>
        {!review && <button className="button primary full" disabled={working || !ready || !!pending}><span aria-hidden="true">🧾</span> ตรวจคิวและเตรียมบิล</button>}
      </fieldset>
    </form>}
    {review && <section className="invoice-confirm">
      <p><strong>พร้อมส่ง:</strong> วิธีมารับ {review.receiving_method} · ชำระค่าเช่า {review.rental_payment_method} · ค่าประกัน {review.bail_payment_method}</p>
      <p className="muted small">{expired ? "ใบตรวจหมดอายุแล้ว กรุณากดตรวจคิวและเตรียมบิลใหม่" : `ใบตรวจใช้ได้ถึง ${new Intl.DateTimeFormat("th-TH", { hour: "2-digit", minute: "2-digit", timeZone: "Asia/Bangkok" }).format(new Date(review.expires_at))} น.`}</p>
      {!data?.order_response_configured && <p className="muted small">หากยังอ่านเลขบิลจากผลตอบกลับไม่ได้ ระบบจะให้ตรวจผลใน Alex และบันทึกเลขออเดอร์ก่อนส่งรายการอื่น</p>}
      <label className="checkbox-row"><input type="checkbox" checked={confirmed} disabled={working} onChange={e => setConfirmed(e.target.checked)} />ตรวจลูกค้า รายการ วันที่ ยอด และส่วนลดแล้ว ยืนยันสร้างออเดอร์จริงใน Alex</label>
      <div className="invoice-actions"><button type="button" className="button secondary" disabled={working} onClick={invalidate}>แก้ไขข้อมูล</button><button type="button" className="button primary" disabled={working || !confirmed || !!pending || expired} onClick={submit}><span aria-hidden="true">🚀</span> {busy ? "กำลังส่งออเดอร์…" : "ยืนยันส่งออเดอร์เข้า Alex"}</button></div>
    </section>}
  </div>;
}

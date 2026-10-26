"use client";
import { useEffect, useState } from "react";
import { api, ApiError, cleanPhone, errorText, Profile } from "@/lib/api";
import CustomerExtractButton from "./customer-extract-button";

type Line = { item_index: number; product_item_id: string; code: string; name: string; color: string | null; size: string | null; quantity: number; rental_fee_satang: number; deposit_satang: number };
type Group = { key: string; branch_id: string; pickup_date: string; return_date: string; fulfillment_method: string; delivery_provider: string | null; lines: Line[]; notes: string; quoted_rental_fee_satang: number | null; quoted_deposit_satang: number | null; quoted_shipping_fee_satang: number | null };
type Submission = { _id: string; status: string; alex_order_id?: string; group: Group; submitted_by: string; created_at: string; verification?: string };
type Data = { source_hash: string | null; groups: Group[]; missing: { item_index: number; fields: string[] }[]; has_conflicts: boolean; context_error?: string; submissions: Submission[]; customer: { name: string | null; phone: string | null; address: string | null }; notes: string | null; order_response_configured: boolean };
type Availability = { state: string; checked_at: string; items: { item_index: number; code: string; quantity: number; state: string }[] };
type Review = { review_id: string; group: Group; customer: { name: string; phone: string; address: string }; catalog_rental_satang: number; catalog_deposit_satang: number; rental_fee_satang: number; deposit_satang: number; shipping_fee_satang: number; shipping_note_only: boolean; rental_discount_satang: number; deposit_discount_satang: number; receiving_method: string; rental_payment_method: string; bail_payment_method: string; notes: string; expires_at: string };
const money = (value: number) => `${(value / 100).toLocaleString("th-TH", { minimumFractionDigits: 2 })} บาท`;
const day = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", year: "numeric", timeZone: "Asia/Bangkok" }).format(new Date(`${value}T00:00:00+07:00`));
const states: Record<string, string> = { AVAILABLE: "ว่าง ณ เวลาตรวจ", UNAVAILABLE: "ไม่ว่าง", UNKNOWN: "ยังตรวจไม่ได้", SUBMITTING: "กำลังส่ง / รอตรวจผล", SUBMIT_UNKNOWN: "ต้องตรวจผลใน Alex", CONFIRMED: "มีออเดอร์ใน Alex แล้ว", NOT_CREATED: "ยังไม่ได้สร้างออเดอร์" };
const missingLabels: Record<string, string> = { product_match: "ยืนยันชุดในแท็บเลือกชุด", pickup_date: "วันรับชุดพร้อมปี", return_date: "วันคืนชุดพร้อมปี", fulfillment_method: "วิธีรับสินค้า", catalog_price: "ราคาแค็ตตาล็อก", overlapping_product: "Product ID ซ้ำและวันเช่าทับกัน กรุณารวมจำนวนหรือเลือกชิ้นอื่น" };

export default function RentalCheckout({ inquiryId, userId, disabled, onUnauthorized }: { inquiryId: string; userId?: string; disabled: boolean; onUnauthorized: () => void }) {
  const [data, setData] = useState<Data | null>(null);
  const [groupKey, setGroupKey] = useState("");
  const [name, setName] = useState(""); const [phone, setPhone] = useState(""); const [address, setAddress] = useState("");
  const [rentalDiscount, setRentalDiscount] = useState("0"); const [depositDiscount, setDepositDiscount] = useState("0"); const [shipping, setShipping] = useState("");
  const [receiving, setReceiving] = useState(""); const [rentalPayment, setRentalPayment] = useState(""); const [bailPayment, setBailPayment] = useState("");
  const [birth, setBirth] = useState(""); const [notes, setNotes] = useState("");
  const [shippingNoteOnly, setShippingNoteOnly] = useState(false); const [freePromotion, setFreePromotion] = useState(false); const [discountPromotion, setDiscountPromotion] = useState(false);
  const [availability, setAvailability] = useState<Availability | null>(null);
  const [review, setReview] = useState<Review | null>(null); const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState(false); const [extracting, setExtracting] = useState(false); const [loading, setLoading] = useState(true); const [error, setError] = useState("");
  const [now, setNow] = useState(Date.now);
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
    setRentalDiscount("0"); setDepositDiscount("0");
    setShipping(group.quoted_shipping_fee_satang === null ? group.fulfillment_method === "store_pickup" ? "0" : "" : String(group.quoted_shipping_fee_satang / 100));
    setReceiving(group.delivery_provider ?? (group.fulfillment_method === "store_pickup" ? "รับหน้าร้าน" : ""));
    setNotes([commonNotes, group.notes].filter(Boolean).join("\n")); setShippingNoteOnly(false);
  }
  function fail(cause: unknown) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); }
  async function refresh() { const value = await api<Data>(endpoint); setData(value); return value; }

  useEffect(() => {
    let cancelled = false;
    Promise.all([api<Data>(endpoint), userId ? api<{ profile: Profile | null }>(`/api/line/users/${encodeURIComponent(userId)}/profile`).catch(() => null) : Promise.resolve(null)]).then(([value, profile]) => {
      if (cancelled) return;
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
      rental_discount_satang: satang(rentalDiscount), deposit_discount_satang: satang(depositDiscount), shipping_fee_satang: satang(shipping), shipping_note_only: shippingNoteOnly,
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

  return <div className="rental-checkout">
    <div className="step-caption"><span>03</span><div><h3>เช็คคิวและเตรียมบิลใน Alex</h3><p className="muted small">ตรวจข้อมูล รายการ และยอดก่อนกดส่งออเดอร์จริง</p></div></div>
    {loading && <p role="status">กำลังโหลดรายการและประวัติส่งบิล…</p>}
    {error && <div className="notice warning" role="alert">{error}<button type="button" className="text-button" disabled={working} onClick={() => refresh().catch(fail)}>โหลดผลล่าสุด</button></div>}
    {data?.submissions.map(submission => <div key={submission._id} className={`notice ${submission.status === "CONFIRMED" ? "success" : submission.status === "NOT_CREATED" ? "info" : "warning"}`}><strong>{states[submission.status] ?? submission.status}</strong><p>{submission.group.lines.map(line => line.code).join(", ")} · {day(submission.group.pickup_date)} – {day(submission.group.return_date)}</p>{submission.alex_order_id && <p>เลขออเดอร์ Alex: <code>{submission.alex_order_id}</code></p>}
      {["SUBMITTING", "SUBMIT_UNKNOWN"].includes(submission.status) && <><p>ตรวจผลใน Alex ก่อน ระบบกันการสร้างบิลใหม่ไว้เพื่อป้องกันบิลซ้ำ</p><button type="button" className="text-button" onClick={() => { setResolveId(submission._id); setResolveChecked(false); setResolveOrder(""); }}>บันทึกผลที่ตรวจจาก Alex</button>
        {resolveId === submission._id && <div className="bill-resolve"><label>เลขออเดอร์ที่พบใน Alex<input value={resolveOrder} disabled={working} onChange={e => setResolveOrder(e.target.value)} /></label><label className="checkbox-row"><input type="checkbox" checked={resolveChecked} onChange={e => setResolveChecked(e.target.checked)} disabled={working} />ตรวจสถานะใน Alex ด้วยตัวเองแล้ว</label><button type="button" className="button secondary full" disabled={working || !resolveChecked || !resolveOrder.trim()} onClick={() => resolve("created")}>ยืนยันว่ามีออเดอร์นี้แล้ว</button><button type="button" className="text-button full" disabled={working || !resolveChecked || now - new Date(submission.created_at).getTime() < 120000} onClick={() => resolve("not_created")}>ตรวจแล้วว่าไม่ได้สร้างบิล</button><p className="muted small">กรณีไม่พบบิล ต้องรออย่างน้อย 2 นาทีแล้วตรวจ Alex ก่อนเปิดให้ส่งใหม่</p></div>}
      </>}
    </div>)}
    {data?.context_error && <div className="notice warning">ร่างหรือแชทเปลี่ยน กรุณาตรวจและบันทึกร่างล่าสุดก่อนเตรียมบิลใหม่</div>}
    {!!data?.missing.length && <div className="notice warning"><strong>ข้อมูลที่ยังต้องเติมก่อนเช็คคิว</strong><ul>{data.missing.map((missing, i) => <li key={i}>รายการ {missing.item_index + 1}: {missing.fields.map(field => missingLabels[field] ?? field).join(" · ")}</li>)}</ul></div>}
    {data?.has_conflicts && !data.context_error && <div className="notice warning">มีข้อมูลขัดกันในร่าง กรุณาตรวจในแท็บร่างล่าสุดก่อน</div>}
    {!!data?.groups.length && <>
      {data.groups.length > 1 && <div className="notice info">พบ {data.groups.length} กลุ่มที่มีช่วงเช่า สาขา หรือวิธีรับต่างกัน เลือกเตรียมบิลแยกทีละกลุ่ม</div>}
      <label>กลุ่มรายการที่จะออกบิล<select value={groupKey} disabled={working} onChange={e => { const group = data.groups.find(group => group.key === e.target.value); if (group) changeGroup(group); else { setGroupKey(""); invalidate(); } }}><option value="">เลือกช่วงเช่าและรายการ</option>{data.groups.map(group => <option key={group.key} value={group.key}>{group.lines.map(line => line.code).join(", ")} · {day(group.pickup_date)} – {day(group.return_date)}</option>)}</select></label>
      {group && <><div className="bill-lines">{group.lines.map(line => <div key={line.item_index}><strong>{line.code} · {line.name}</strong><p>{line.color || "ไม่ระบุสี"} · {line.size || "ไม่ระบุไซซ์"} · จำนวน {line.quantity}</p></div>)}</div><button type="button" className="button secondary full" disabled={working || !ready} onClick={check}>{busy ? "กำลังตรวจ…" : "เช็คคิวจาก Alex"}</button>
        {availability && <div className={`notice ${availability.state === "AVAILABLE" ? "success" : "warning"}`} role="status"><strong>{states[availability.state]}</strong>{availability.items.map(item => <p key={item.item_index}>{item.code} × {item.quantity}: {states[item.state]}</p>)}<p className="small">ผลตรวจยังไม่กันคิว ระบบจะตรวจซ้ำก่อนส่งออเดอร์</p></div>}
        <form onSubmit={e => { e.preventDefault(); void prepare(); }}>
          <CustomerExtractButton inquiryId={inquiryId} disabled={working} onBusyChange={setExtracting} onUnauthorized={onUnauthorized} onFill={value => { setName(current => value.display_name ?? current); setPhone(current => value.phone ? cleanPhone(value.phone) : current); setAddress(current => value.address ?? current); invalidate(); }} />
          <fieldset disabled={working || !ready || !!pending}><div className="field-grid">
            <label>ชื่อลูกค้า<input required value={name} maxLength={200} onChange={e => { setName(e.target.value); invalidate(); }} /></label><label>เบอร์โทร<input required value={phone} maxLength={40} inputMode="tel" onChange={e => { setPhone(cleanPhone(e.target.value)); invalidate(); }} /></label>
            <label className="span-2">ที่อยู่<textarea rows={3} required={group.fulfillment_method === "delivery"} value={address} maxLength={1000} onChange={e => { setAddress(e.target.value); invalidate(); }} /></label>
            <label>วิธีรับ / ชื่อขนส่ง<input required value={receiving} maxLength={100} onChange={e => { setReceiving(e.target.value); invalidate(); }} /></label><label>วันเกิด (ถ้าทราบ)<input type="date" value={birth} onChange={e => { setBirth(e.target.value); invalidate(); }} /></label>
            <p className="span-2 muted small">ค่าเช่าแค็ตตาล็อก {money(catalogRental)} · ค่าประกัน {money(catalogDeposit)}{group.quoted_rental_fee_satang !== null && group.quoted_rental_fee_satang !== catalogRental && <><br />ราคาที่แจ้งลูกค้าในร่าง: ค่าเช่า {money(group.quoted_rental_fee_satang)}{group.quoted_deposit_satang !== null && ` · ค่าประกัน ${money(group.quoted_deposit_satang)}`}</>}</p>
            {([["ส่วนลดค่าเช่า (บาท)", rentalDiscount, setRentalDiscount, catalogRental], ["ส่วนลดค่าประกัน (บาท)", depositDiscount, setDepositDiscount, catalogDeposit], ["ค่าจัดส่ง (บาท)", shipping, setShipping, null]] as const).map(([label, value, set, max]) => <label key={label}>{label}<input required type="number" min="0" max={max === null ? undefined : max / 100} step="0.01" value={value} onChange={e => { set(e.target.value); invalidate(); }} /></label>)}
            <p className="span-2 muted small">ยอดหลังหักส่วนลด: ค่าเช่า {money(catalogRental - satang(rentalDiscount))} · ค่าประกัน {money(catalogDeposit - satang(depositDiscount))}</p>
            {([["วิธีชำระค่าเช่า", rentalPayment, setRentalPayment], ["วิธีชำระค่าประกัน", bailPayment, setBailPayment]] as const).map(([label, value, set]) => <label key={label}>{label}<select required value={value} onChange={e => { set(e.target.value); invalidate(); }}><option value="">เลือกวิธีชำระ</option><option>โอนเงิน</option><option>เงินสด</option><option>บัตรเครดิต</option></select></label>)}
          </div>
          {Number(shipping) > 0 && <div className="notice warning"><p>รูปแบบสร้างบิลที่ได้รับยังไม่มีช่องค่าส่งแยก การส่งรอบนี้บันทึกค่าส่งในหมายเหตุ โดยยอดค่าเช่าใน Alex จะไม่รวมค่าส่ง</p><label className="checkbox-row"><input type="checkbox" required checked={shippingNoteOnly} onChange={e => { setShippingNoteOnly(e.target.checked); invalidate(); }} />รับทราบและให้บันทึกค่าส่งไว้ในหมายเหตุ</label></div>}
          <label>หมายเหตุ / อุปกรณ์<textarea rows={3} maxLength={2000} value={notes} onChange={e => { setNotes(e.target.value); invalidate(); }} /></label>
          <label className="checkbox-row"><input type="checkbox" checked={freePromotion} onChange={e => { setFreePromotion(e.target.checked); invalidate(); }} />ใช้โปรโมชั่นของแถม</label><label className="checkbox-row"><input type="checkbox" checked={discountPromotion} onChange={e => { setDiscountPromotion(e.target.checked); invalidate(); }} />ใช้โปรโมชั่นส่วนลด</label>
          <button className="button secondary full" disabled={working || !ready || !!pending}>ตรวจคิวและเตรียมบิลให้ตรวจ</button>
          </fieldset>
        </form>
      </>}
    </>}
    {review && <section className="bill-review"><h3>ตรวจบิลก่อนส่งเข้า Alex</h3><p>{review.customer.name} · {review.customer.phone}</p><p>{review.customer.address}</p><p>{review.group.lines.map(line => `${line.code} × ${line.quantity}`).join(", ")}<br />{day(review.group.pickup_date)} – {day(review.group.return_date)}</p><dl><div><dt>ค่าเช่าแค็ตตาล็อก</dt><dd>{money(review.catalog_rental_satang)}</dd></div><div><dt>ส่วนลดค่าเช่า</dt><dd>{money(review.rental_discount_satang)}</dd></div><div><dt>ค่าเช่าสุทธิ</dt><dd>{money(review.rental_fee_satang)}</dd></div><div><dt>ค่าประกันแค็ตตาล็อก</dt><dd>{money(review.catalog_deposit_satang)}</dd></div><div><dt>ส่วนลดค่าประกัน</dt><dd>{money(review.deposit_discount_satang)}</dd></div><div><dt>ค่าประกันสุทธิ</dt><dd>{money(review.deposit_satang)}</dd></div><div><dt>ค่าส่ง (บันทึกหมายเหตุ)</dt><dd>{money(review.shipping_fee_satang)}</dd></div></dl><p>รับ/ส่ง: {review.receiving_method}<br />ชำระค่าเช่า: {review.rental_payment_method} · ค่าประกัน: {review.bail_payment_method}</p><p>{review.notes}</p>
      {!data?.order_response_configured && <p className="muted small">หากยังอ่านเลขบิลจากผลตอบกลับไม่ได้ ระบบจะให้ตรวจผลใน Alex และบันทึกเลขออเดอร์ก่อนส่งรายการอื่น</p>}
      <label className="checkbox-row"><input type="checkbox" checked={confirmed} disabled={working} onChange={e => setConfirmed(e.target.checked)} />ตรวจลูกค้า รายการ วันที่ ยอด และส่วนลดแล้ว ยืนยันสร้างออเดอร์จริงใน Alex</label>
      <button type="button" className="button primary full" disabled={working || !confirmed || !!pending || now > new Date(review.expires_at).getTime()} onClick={submit}>{busy ? "กำลังส่งออเดอร์…" : "ส่งสร้างบิล / ออเดอร์ใน Alex"}</button>
    </section>}
  </div>;
}

"use client";
/* eslint-disable @next/next/no-img-element -- Catalog images are external Alex links; next/image would need a fixed remote host list. */
import { useEffect, useRef, useState } from "react";
import { api, ApiError, cleanPhone, codeLabel, CodeName, errorText, PAYMENT_METHODS, Profile, RECEIVING_METHODS, STORE_PICKUP, UNPAID } from "@/lib/api";
import CustomerExtractButton from "./customer-extract-button";
import { formatDate as day } from "@/lib/dates";

type Line = { item_index: number; product_item_id: string; code: string; name: string; color: string | null; size: string | null; quantity: number; rental_fee_satang: number; deposit_satang: number; is_shipping?: boolean; image_url?: string | null };
type Group = { key: string; branch_id: string; pickup_date: string; return_date: string; fulfillment_method: string; delivery_provider: string | null; lines: Line[]; notes: string; quoted_rental_fee_satang: number | null; quoted_deposit_satang: number | null; quoted_shipping_fee_satang: number | null; rental_discount_satang: number; deposit_discount_satang: number; amounts_from_summary?: boolean };
type Submission = { _id: string; status: string; alex_order_id?: string; alex_order_number?: string; group: Group; submitted_by: string; created_at: string; verification?: string; draft_version?: number | null; queue_accepted?: boolean };
type Data = { source_hash: string | null; draft_version?: number; closed?: "SENT" | "CANCELLED" | null; groups: Group[]; missing: { field: string; code?: string }[]; has_conflicts: boolean; context_error?: string; submissions: Submission[]; customer: { name: string | null; phone: string | null; address: string | null }; notes: string | null; order_response_configured: boolean };
type Availability = { state: string; checked_at: string; items: { item_index: number; product_item_id: string; code: string; quantity: number; state: string }[] };
type Review = { review_id: string; group: Group; queue: Omit<Availability, "checked_at">; customer: { name: string; phone: string; address: string }; catalog_rental_satang: number; catalog_deposit_satang: number; rental_fee_satang: number; deposit_satang: number; shipping_fee_satang: number; rental_discount_satang: number; deposit_discount_satang: number; receiving_method: string; rental_payment_method: string; bail_payment_method: string; notes: string; expires_at: string };
const baht = (value: number) => (value / 100).toLocaleString("th-TH", { maximumFractionDigits: 2 });
const moneyInput = (value: string) => {
  if (!/^\d*(?:\.\d{0,2})?$/.test(value) || value === ".") return null;
  const satang = Math.round(Number(value || "0") * 100);
  return Number.isSafeInteger(satang) && satang >= 0 ? satang : null;
};
const nights = (pickup: string, returned: string) => Math.max(1, Math.round((Date.parse(returned) - Date.parse(pickup)) / 86_400_000));
const badges: Record<string, [string, string]> = { AVAILABLE: ["ว่าง", "free"], UNAVAILABLE: ["ติดคิว", "blocked"], UNKNOWN: ["ตรวจคิวไม่ได้", "unknown"] };
const states: Record<string, string> = { AVAILABLE: "ว่าง ณ เวลาตรวจ", UNAVAILABLE: "ไม่ว่าง", UNKNOWN: "ยังตรวจไม่ได้", SUBMITTING: "กำลังส่งเข้า Alex", SUBMIT_UNKNOWN: "ส่งแล้ว · ยังไม่ได้เลขออเดอร์", CONFIRMED: "มีออเดอร์ใน Alex แล้ว", NOT_CREATED: "ยังไม่ได้สร้างออเดอร์" };
const missingLabels: Record<string, string> = { cart: "เพิ่มสินค้าลงตะกร้าในแท็บร่างล่าสุด", mixed_branches: "สินค้าในตะกร้าต้องอยู่สาขาเดียวกัน", pickup_date: "วันรับชุด", return_date: "วันคืนชุด", fulfillment_method: "วิธีมารับ", catalog_price: "ราคาแค็ตตาล็อก", overlapping_product: "Product ID ซ้ำและวันเช่าทับกัน กรุณารวมจำนวนหรือเลือกชิ้นอื่น",
  summary_rental: "ค่าเช่าในสรุปบิล — ร้านตั้งให้ใช้ยอดจากสรุปบิลเป็นหลัก ใส่ค่าเช่าในสรุปบิลแล้วเตรียมออเดอร์ด้วย AI ใหม่",
  summary_deposit: "ค่าประกันในสรุปบิล — ร้านตั้งให้ใช้ยอดจากสรุปบิลเป็นหลัก ใส่ค่าประกันในสรุปบิลแล้วเตรียมออเดอร์ด้วย AI ใหม่",
  summary_rental_above_catalog: "ค่าเช่าในสรุปบิลสูงกว่าราคาใน Alex ของตะกร้า — Alex ไม่มีช่องบวกเพิ่ม กรุณาตรวจสรุปบิลหรือสินค้าในตะกร้า",
  summary_deposit_above_catalog: "ค่าประกันในสรุปบิลสูงกว่าราคาใน Alex ของตะกร้า — Alex ไม่มีช่องบวกเพิ่ม กรุณาตรวจสรุปบิลหรือสินค้าในตะกร้า" };

// The bill as an invoice: one order from the cart and the latest draft, checked and reviewed before it is sent to Alex.
export default function RentalCheckout({ inquiryId, userId, disabled, titleId, onUnauthorized, canSubmit = true }: { inquiryId: string; userId?: string; disabled: boolean; titleId?: string; onUnauthorized: () => void; canSubmit?: boolean }) {
  const [data, setData] = useState<Data | null>(null);
  const [groupKey, setGroupKey] = useState("");
  const [name, setName] = useState(""); const [phone, setPhone] = useState(""); const [address, setAddress] = useState("");
  const [receiving, setReceiving] = useState(""); const [rentalPayment, setRentalPayment] = useState(UNPAID); const [bailPayment, setBailPayment] = useState(UNPAID);
  const [notes, setNotes] = useState("");
  const [freePromotion, setFreePromotion] = useState(false); const [discountPromotion, setDiscountPromotion] = useState(false);
  const [availability, setAvailability] = useState<Availability | null>(null);
  const [storedReview, setReview] = useState<(Review & { sourceHash: string }) | null>(null);
  const review = storedReview?.sourceHash === data?.source_hash ? storedReview : null;
  const [amountEdit, setAmountEdit] = useState<{ key: string; rental: string; deposit: string } | null>(null);
  const [amountPreview, setAmountPreview] = useState<{ key: string; group?: Group; amounts_hash?: string; error?: string } | null>(null);
  const [amountRetry, setAmountRetry] = useState(0);
  const [amountReset, setAmountReset] = useState(0);
  const sending = useRef(false);
  const attemptedReview = useRef<string | null>(null);
  const [busy, setBusy] = useState(false); const [extracting, setExtracting] = useState(false); const [loading, setLoading] = useState(true); const [error, setError] = useState("");
  const [now, setNow] = useState(Date.now);
  // Lines carry Alex color codes (P); show the shop's color names next to them.
  const [colors, setColors] = useState<CodeName[]>([]);
  const [resolveId, setResolveId] = useState(""); const [resolveOrder, setResolveOrder] = useState(""); const [resolveChecked, setResolveChecked] = useState(false);
  const [resolveNumber, setResolveNumber] = useState("");
  const working = disabled || busy || extracting || loading;
  const sourceGroup = data?.groups.find(group => group.key === groupKey);
  const edited = amountEdit?.key === groupKey;
  const rentalInput = edited ? amountEdit.rental : String((sourceGroup?.rental_discount_satang ?? 0) / 100);
  const depositInput = edited ? amountEdit.deposit : String((sourceGroup?.deposit_discount_satang ?? 0) / 100);
  const rentalDiscount = moneyInput(rentalInput), depositDiscount = moneyInput(depositInput);
  const validAmounts = rentalDiscount !== null && depositDiscount !== null;
  const amountRequestKey = edited && validAmounts && data?.source_hash && sourceGroup ? JSON.stringify({ source_hash: data.source_hash, group_key: groupKey, rental_discount_satang: rentalDiscount, deposit_discount_satang: depositDiscount }) : "";
  const matchingPreview = amountRequestKey && amountPreview?.key === amountRequestKey ? amountPreview : null;
  const amountPending = !!amountRequestKey && !matchingPreview;
  const amountError = matchingPreview?.error;
  const group = matchingPreview?.group ?? sourceGroup;
  const shipping = 0;
  // Only a send in progress blocks a new bill; bills of other versions do not, the queue check covers stock.
  const pending = data?.submissions.some(submission => submission.status === "SUBMITTING");
  const ready = !!group && !!data?.source_hash && !data.has_conflicts && !data.closed;
  // The form locks for these reasons; their notices sit at the top, so the one that applies is repeated by the button.
  const locked = data?.closed ? `ร่างเวอร์ชันนี้${data.closed === "SENT" ? "ส่งเข้า Alex แล้ว" : "ถูกยกเลิกแล้ว"} — ถ้าจะส่งอีกครั้ง ให้โคลนเป็นเวอร์ชันใหม่ในแท็บเวอร์ชันก่อนหน้า`
    : pending ? "มีบิลกำลังส่งเข้า Alex อยู่ รอสักครู่แล้วกดโหลดผลล่าสุด"
    : data?.has_conflicts ? "ร่างล่าสุดมีข้อมูลขัดกัน — ปิดหน้านี้ ตรวจร่าง ติ๊ก “ยืนยันข้อมูลที่กรอกครบในร่างนี้” แล้วกดบันทึกร่างเป็นเวอร์ชันใหม่"
    : disabled ? "รอให้งานที่ค้างอยู่เสร็จก่อน (AI กำลังอ่านสรุป/บันทึกร่าง หรือมีสรุปที่ยังไม่ได้บันทึก)" : "";
  const queueBlocked = !!review && review.queue.state !== "AVAILABLE";
  const catalogRental = group?.lines.reduce((sum, line) => sum + line.rental_fee_satang * line.quantity, 0) ?? 0;
  const catalogDeposit = group?.lines.reduce((sum, line) => sum + line.deposit_satang * line.quantity, 0) ?? 0;
  const endpoint = `/api/inquiries/${inquiryId}/checkout`;

  function invalidate() { setReview(null); attemptedReview.current = null; }
  function changeGroup(group: Group, commonNotes = data?.notes ?? "") {
    setGroupKey(group.key); setAvailability(null); invalidate(); setAmountEdit(null); setAmountPreview(null);
    setReceiving(group.fulfillment_method === "store_pickup" ? STORE_PICKUP : group.delivery_provider && RECEIVING_METHODS.includes(group.delivery_provider) ? group.delivery_provider : "Flash");
    setNotes(commonNotes);
  }
  function changeDiscount(field: "rental" | "deposit", value: string) {
    setDiscountPromotion(false);
    setAmountEdit(current => ({ ...(current?.key === groupKey ? current : { key: groupKey, rental: String((sourceGroup?.rental_discount_satang ?? 0) / 100), deposit: String((sourceGroup?.deposit_discount_satang ?? 0) / 100) }), [field]: value }));
    // A preview is usable only for its exact source and amounts; formatting changes can reuse it.
    setAvailability(null); invalidate();
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

  useEffect(() => {
    if (!amountRequestKey) return;
    let cancelled = false;
    const timer = setTimeout(() => {
      api<{ group: Group; amounts_hash: string }>(`${endpoint}/amounts`, { method: "POST", body: amountRequestKey }).then(value => {
        if (!cancelled) setAmountPreview({ key: amountRequestKey, ...value });
      }).catch(cause => {
        if (!cancelled) {
          setAmountPreview({ key: amountRequestKey, error: cause instanceof ApiError && cause.code === "VALIDATION_ERROR" ? "ส่วนลดต้องตั้งแต่ 0 บาท และไม่เกินยอดค่าเช่าหรือค่าประกันของสินค้าใน Alex" : errorText(cause) });
          if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
        }
      });
    }, 250);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [amountRequestKey, endpoint, amountRetry]); // eslint-disable-line react-hooks/exhaustive-deps

  async function check() {
    if (!ready || working || !validAmounts || amountPending || amountError) return;
    setBusy(true); setError(""); invalidate();
    try { setAvailability(await api<Availability>(`${endpoint}/availability`, { method: "POST", body: JSON.stringify({ source_hash: data!.source_hash, group_key: groupKey, ...(edited ? { rental_discount_satang: rentalDiscount, deposit_discount_satang: depositDiscount } : {}) }) })); }
    catch (cause) { fail(cause); } finally { setBusy(false); }
  }
  async function submit(acceptQueue = false) {
    if (!canSubmit || !ready || working || pending || sending.current || !validAmounts || amountPending || amountError) return;
    sending.current = true;
    setBusy(true); setError("");
    try {
      // Keep the same ID on a retry after an ambiguous network failure.
      const value = review && (!acceptQueue || attemptedReview.current === review.review_id) ? review : await api<Review>(`${endpoint}/review`, { method: "POST", body: JSON.stringify({
        source_hash: data!.source_hash, group_key: groupKey, customer_name: name, customer_phone: phone, customer_address: address,
        shipping_fee_satang: shipping,
        ...(edited ? { rental_discount_satang: rentalDiscount, deposit_discount_satang: depositDiscount, amounts_hash: matchingPreview?.amounts_hash } : {}),
        receiving_method: receiving, rental_payment_method: rentalPayment, bail_payment_method: bailPayment, birth_date: null, notes,
        use_free_item_promotion: freePromotion, use_discount_promotion: !edited && discountPromotion,
      }) });
      // The review checks the queue too; show it on the items.
      setReview({ ...value, sourceHash: data!.source_hash! }); setAvailability({ ...value.queue, checked_at: new Date().toISOString() });
      if (value.queue.state !== "AVAILABLE" && !acceptQueue) return;
      attemptedReview.current = value.review_id;
      await api<Submission>(`${endpoint}/submit`, { method: "POST", body: JSON.stringify({ review_id: value.review_id, confirmed: true, accept_queue: value.queue.state !== "AVAILABLE" && acceptQueue }) });
      await refresh(); invalidate();
    }
    catch (cause) {
      fail(cause);
      if (cause instanceof ApiError && ["QUEUE_BLOCKED", "CONTEXT_CHANGED", "BILL_NOT_READY"].includes(cause.code)) { invalidate(); refresh().catch(() => {}); }
    } finally { sending.current = false; setBusy(false); }
  }
  async function resolve(outcome: "created" | "not_created") {
    if (!canSubmit) return;
    if (!resolveId || !resolveChecked || working) return;
    setBusy(true); setError("");
    try { await api(`${endpoint}/resolve`, { method: "POST", body: JSON.stringify({ submission_id: resolveId, outcome, order_id: resolveOrder || null, order_number: resolveNumber.trim() || null, verified_in_alex: true }) }); await refresh(); setResolveId(""); setResolveOrder(""); setResolveNumber(""); setResolveChecked(false); }
    catch (cause) { fail(cause); } finally { setBusy(false); }
  }

  const totals = review ? { rent: review.catalog_rental_satang, rentDiscount: review.rental_discount_satang, rentNet: review.rental_fee_satang, deposit: review.catalog_deposit_satang, depositDiscount: review.deposit_discount_satang, depositNet: review.deposit_satang, shipping: review.shipping_fee_satang }
    : group ? { rent: catalogRental, rentDiscount: rentalDiscount ?? 0, rentNet: catalogRental - (rentalDiscount ?? 0), deposit: catalogDeposit, depositDiscount: depositDiscount ?? 0, depositNet: catalogDeposit - (depositDiscount ?? 0), shipping } : null;
  const pieces = group?.lines.reduce((sum, line) => sum + line.quantity, 0) ?? 0;

  return <div className="rental-checkout invoice">
    <header className="invoice-head">
      <div><p className="invoice-eyebrow">ใบสรุปออเดอร์</p><h3 id={titleId}>สรุปก่อนส่งเข้า Alex</h3><p className="muted small">เรื่อง #{inquiryId.slice(-6).toUpperCase()}{data?.draft_version ? ` · จากร่าง v${data.draft_version}` : ""}</p></div>
      {group && <dl className="invoice-period"><div><dt>วันรับชุด</dt><dd>{day(group.pickup_date)}</dd></div><div><dt>วันคืนชุด</dt><dd>{day(group.return_date)}</dd></div><div><dt>ระยะเช่า</dt><dd>{nights(group.pickup_date, group.return_date)} วัน</dd></div></dl>}
    </header>
    {loading && <p role="status" className="muted">กำลังโหลดรายการและประวัติส่งบิล…</p>}
    {error && <div className="notice warning" role="alert">{error}<button type="button" className="text-button" disabled={working} onClick={() => refresh().catch(fail)}>โหลดผลล่าสุด</button></div>}
    {data?.closed && <div className={`notice ${data.closed === "SENT" ? "success" : "warning"}`}><strong>{data.closed === "SENT" ? <><span aria-hidden="true">✅ </span>ร่างเวอร์ชันนี้ส่งเข้า Alex แล้ว</> : "ร่างเวอร์ชันนี้ถูกยกเลิกแล้ว"}</strong><p>ส่งซ้ำไม่ได้ ถ้าต้องการส่งอีกครั้งให้โคลนเป็นเวอร์ชันใหม่ในแท็บเวอร์ชันก่อนหน้า</p></div>}
    {data?.submissions.map(submission => <div key={submission._id} className={`notice ${submission.status === "CONFIRMED" ? "success" : submission.status === "SUBMITTING" ? "warning" : "info"}`}><strong>{submission.status === "CONFIRMED" && <span aria-hidden="true">✅ </span>}{submission.draft_version ? `ร่าง v${submission.draft_version} · ` : ""}{states[submission.status] ?? submission.status}</strong><p>{submission.group.lines.map(line => line.code).join(", ")} · {day(submission.group.pickup_date)} – {day(submission.group.return_date)}{submission.queue_accepted && " · ส่งทั้งที่ติดคิว"}</p>{submission.alex_order_number && <p>Order Number: <code>{submission.alex_order_number}</code></p>}{submission.alex_order_id && <p>Order ID: <code>{submission.alex_order_id}</code></p>}
      {["SUBMITTING", "SUBMIT_UNKNOWN"].includes(submission.status) && <><p>{submission.status === "SUBMITTING" ? "กำลังรอ Alex ตอบกลับ ถ้าค้างนานให้ตรวจใน Alex แล้วบันทึกผล" : "ระบบอ่านเลขออเดอร์จากผลตอบกลับไม่ได้ ตรวจใน Alex แล้วบันทึกเลขออเดอร์ไว้ได้ (ไม่ล็อกการส่งเวอร์ชันอื่น)"}</p><button type="button" className="text-button" disabled={!canSubmit} onClick={() => { setResolveId(submission._id); setResolveChecked(false); setResolveOrder(submission.alex_order_id ?? ""); setResolveNumber(submission.alex_order_number ?? ""); }}>บันทึกผลที่ตรวจจาก Alex</button>
        {resolveId === submission._id && <div className="bill-resolve"><label>Order ID ที่พบใน Alex<input autoComplete="off" value={resolveOrder} disabled={working} onChange={e => setResolveOrder(e.target.value)} /></label><label>Order Number ที่พบใน Alex<input autoComplete="off" value={resolveNumber} disabled={working} onChange={e => setResolveNumber(e.target.value)} /></label><label className="checkbox-row"><input type="checkbox" checked={resolveChecked} onChange={e => setResolveChecked(e.target.checked)} disabled={working} />ตรวจสถานะใน Alex ด้วยตัวเองแล้ว</label><button type="button" className="button secondary full" disabled={!canSubmit || working || !resolveChecked || !resolveOrder.trim()} onClick={() => resolve("created")}>ยืนยันว่ามีออเดอร์นี้แล้ว</button><button type="button" className="text-button full" disabled={!canSubmit || working || !resolveChecked || now - new Date(submission.created_at).getTime() < 120000} onClick={() => resolve("not_created")}>ตรวจแล้วว่าไม่ได้สร้างบิล</button><p className="muted small">กรณีไม่พบบิล ต้องรออย่างน้อย 2 นาทีแล้วตรวจ Alex ก่อนเปิดให้ส่งใหม่</p></div>}
      </>}
    </div>)}
    {data?.context_error && <div className="notice warning">ร่างหรือแชทเปลี่ยน กรุณาตรวจและบันทึกร่างล่าสุดก่อนเตรียมบิลใหม่</div>}
    {!!data?.missing.length && <div className="notice warning"><strong>ข้อมูลที่ยังต้องเติมในแท็บร่างล่าสุด</strong><ul>{data.missing.map((missing, i) => <li key={i}>{missingLabels[missing.field] ?? missing.field}{missing.code && ` (${missing.code})`}</li>)}</ul></div>}
    {data?.has_conflicts && !data.context_error && <div className="notice warning">มีข้อมูลขัดกันในร่าง กรุณาตรวจในแท็บร่างล่าสุดก่อน</div>}
    {group && totals && <form autoComplete="off" className="invoice-body" onSubmit={e => { e.preventDefault(); void submit(queueBlocked); }}>
      <fieldset disabled={working || !ready || !!pending}>
        <section className="invoice-block">
          <h4><span aria-hidden="true">👤</span> ลูกค้า</h4>
          <div className="field-grid">
            <label>ชื่อลูกค้า<input autoComplete="off" required value={name} maxLength={200} onChange={e => { setName(e.target.value); invalidate(); }} /></label><label>เบอร์โทร<input autoComplete="off" required value={phone} maxLength={40} inputMode="tel" onChange={e => { setPhone(cleanPhone(e.target.value)); invalidate(); }} /></label>
            <label className="span-2">ที่อยู่<textarea autoComplete="off" rows={2} required={group.fulfillment_method === "delivery"} value={address} maxLength={1000} onChange={e => { setAddress(e.target.value); invalidate(); }} /></label>
          </div>
          <CustomerExtractButton inquiryId={inquiryId} disabled={working} onBusyChange={setExtracting} onUnauthorized={onUnauthorized} onFill={value => { setName(current => value.display_name ?? current); setPhone(current => value.phone ? cleanPhone(value.phone) : current); setAddress(current => value.address ?? current); invalidate(); }} />
        </section>
        <section className="invoice-block">
          <h4><span aria-hidden="true">👗</span> รายการเช่า <span className="muted small">{pieces} ชิ้น</span></h4>
          <div className="invoice-items">{group.lines.map(line => { const state = availability?.items.find(item => item.product_item_id === line.product_item_id)?.state; const badge = state ? badges[state] : undefined; return <article key={line.item_index} className={`invoice-item ${badge ? `queue-${badge[1]}` : ""}`}>
            <div className="invoice-thumb">{line.image_url ? <img src={line.image_url} alt={`รูปสินค้า ${line.name}`} loading="lazy" referrerPolicy="no-referrer" onError={e => { e.currentTarget.hidden = true; }} /> : <span aria-hidden="true">👗</span>}</div>
            <div className="invoice-item-body"><strong>{line.name}</strong><span className="invoice-code">{line.code}</span><div className="invoice-chips"><span>สี {codeLabel(line.color, colors)}</span><span>ไซซ์ {line.size || "-"}</span><span>× {line.quantity}</span>{badge && <span className={`queue-badge ${badge[1]}`}>{badge[0]}</span>}</div></div>
            <dl className="invoice-item-amounts"><div><dt>{line.is_shipping ? "ราคา" : "ค่าเช่า"}</dt><dd>{baht(line.rental_fee_satang * line.quantity)}</dd></div>{!line.is_shipping && <div><dt>ค่าประกัน</dt><dd>{baht(line.deposit_satang * line.quantity)}</dd></div>}</dl>
          </article>; })}</div>
          <button type="button" className="button info full" disabled={working || !ready || !validAmounts || amountPending || !!amountError} onClick={check}><span aria-hidden="true">📅</span> {busy ? "กำลังตรวจ…" : "เช็คคิวจาก Alex"}</button>
          {availability && <p className={`invoice-queue ${availability.state === "AVAILABLE" ? "free" : "blocked"}`} role="status">{states[availability.state]} · ผลตรวจยังไม่กันคิว ระบบตรวจซ้ำก่อนส่งออเดอร์</p>}
        </section>
        <section className="invoice-block">
          <h4><span aria-hidden="true">💳</span> การรับและชำระเงิน</h4>
          <div className="field-grid">
            <label>วิธีมารับ<select autoComplete="off" required value={receiving} onChange={e => { setReceiving(e.target.value); invalidate(); }}>{RECEIVING_METHODS.map(method => <option key={method} value={method}>{method}</option>)}</select></label>
            {([["วิธีชำระค่าเช่า", rentalPayment, setRentalPayment], ["วิธีชำระค่าประกัน", bailPayment, setBailPayment]] as const).map(([label, value, set]) => <label key={label}>{label}<select autoComplete="off" value={value} onChange={e => { set(e.target.value); invalidate(); }}>{PAYMENT_METHODS.map(method => <option key={method} value={method}>{method}</option>)}</select></label>)}
          </div>
          <label>หมายเหตุ / อุปกรณ์<textarea autoComplete="off" rows={2} maxLength={2000} value={notes} onChange={e => { setNotes(e.target.value); invalidate(); }} /></label>
          <div className="invoice-options"><label className="checkbox-row"><input type="checkbox" checked={freePromotion} onChange={e => { setFreePromotion(e.target.checked); invalidate(); }} />ใช้โปรโมชั่นของแถม</label><label className="checkbox-row"><input type="checkbox" checked={!edited && discountPromotion} disabled={edited} onChange={e => { setDiscountPromotion(e.target.checked); invalidate(); }} />ใช้โปรโมชั่นส่วนลด</label></div>
        </section>
        <section className="invoice-totals" aria-label="สรุปยอด" aria-busy={amountPending}>
          <dl>
            <div><dt>ยอดเช่า</dt><dd>{baht(totals.rent)}</dd></div>
            <div className="discount invoice-discount"><dt><label htmlFor={`${inquiryId}-rental-discount`}>ส่วนลดค่าเช่า (บาท)</label></dt><dd><input key={`rental-${amountReset}`} id={`${inquiryId}-rental-discount`} type="number" inputMode="decimal" autoComplete="off" min="0" step="0.01" value={rentalInput} aria-invalid={rentalDiscount === null || !!amountError} aria-describedby={`${inquiryId}-amount-status`} onChange={e => changeDiscount("rental", e.target.value)} /></dd></div>
            <div className="subtotal"><dt>ค่าเช่าสุทธิ</dt><dd>{baht(totals.rentNet)}</dd></div>
            <div><dt>ยอดรวมค่าประกัน</dt><dd>{baht(totals.deposit)}</dd></div>
            <div className="discount invoice-discount"><dt><label htmlFor={`${inquiryId}-deposit-discount`}>ส่วนลดค่าประกัน (บาท)</label></dt><dd><input key={`deposit-${amountReset}`} id={`${inquiryId}-deposit-discount`} type="number" inputMode="decimal" autoComplete="off" min="0" step="0.01" value={depositInput} aria-invalid={depositDiscount === null || !!amountError} aria-describedby={`${inquiryId}-amount-status`} onChange={e => changeDiscount("deposit", e.target.value)} /></dd></div>
            <div className="subtotal"><dt>ค่าประกันสุทธิ</dt><dd>{baht(totals.depositNet)}</dd></div>
          </dl>
          <div className="invoice-grand" aria-live="polite"><span>ยอดรวมทั้งหมด</span><strong>{!validAmounts || amountPending || amountError ? "…" : baht(totals.rentNet + totals.depositNet + totals.shipping)} <small>บาท</small></strong></div>
          {edited && <p className="notice info">ใช้ส่วนลดที่แอดมินกรอกเป็นยอดสุดท้าย ไม่บวกส่วนลดเดิมหรือโปรโมชั่นส่วนลดอัตโนมัติเพิ่ม</p>}
          <p id={`${inquiryId}-amount-status`} role={amountError || !validAmounts ? "alert" : "status"} className={amountError || !validAmounts ? "notice warning" : "muted small"}>{!validAmounts ? "กรอกส่วนลดเป็นจำนวนตั้งแต่ 0 บาท และทศนิยมไม่เกิน 2 ตำแหน่ง" : amountError || (amountPending ? "กำลังตรวจยอดและค่าขนส่งตามเงื่อนไขร้าน…" : "แก้ส่วนลดได้ก่อนส่ง ยอดที่แก้จะใช้กับบิลครั้งนี้")}</p>
          {amountError && <button type="button" className="text-button" onClick={() => { setAmountPreview(null); setAmountRetry(value => value + 1); }}>คำนวณยอดอีกครั้ง</button>}
          {edited && <button type="button" className="text-button" onClick={() => { setAmountEdit(null); setAmountPreview(null); setAmountReset(value => value + 1); setAvailability(null); invalidate(); }}>ใช้ส่วนลดจากร่าง</button>}
          {group.amounts_from_summary ? <p className="muted small">ใช้ยอดจากสรุปบิลเป็นค่าเริ่มต้น แก้ส่วนลดด้านบนก่อนส่งได้</p>
            : <>
              {((group.quoted_rental_fee_satang ?? 0) > catalogRental || (group.quoted_deposit_satang ?? 0) > catalogDeposit) && <p className="notice warning">ราคาที่แจ้งลูกค้าสูงกว่าราคาใน Alex — Alex ไม่มีช่องบวกเพิ่ม บิลจะใช้ราคาใน Alex</p>}
            </>}
          <p className="muted small">ระบบตรวจราคาและคิวใน Alex ก่อนส่งออเดอร์</p>
        </section>
        {queueBlocked && <div className="notice warning" role="alert"><strong>{review.queue.state === "UNAVAILABLE" ? "Alex แจ้งว่าสินค้าติดคิวในช่วงวันนี้" : "ตรวจคิวจาก Alex ไม่ได้"}</strong><ul>{review.queue.items.filter(item => item.state !== "AVAILABLE").map(item => <li key={item.product_item_id}>{item.code} · {states[item.state] ?? item.state}</li>)}</ul><p>หากต้องการส่งต่อ ให้กดปุ่มส่งออเดอร์ตามสถานะที่แสดงด้านล่าง</p></div>}
        {locked && <p className="notice warning" role="status">{locked}</p>}
        {!canSubmit && <p className="notice info">เตรียมข้อมูลและเช็คคิวได้ ให้ผู้จัดการหรือเจ้าของร้านส่งออเดอร์</p>}
        <button className="button primary full" aria-busy={busy || amountPending} disabled={!canSubmit || working || !ready || !!pending || !validAmounts || amountPending || !!amountError}>{busy ? "กำลังส่งออเดอร์…" : queueBlocked ? review.queue.state === "UNAVAILABLE" ? "ส่งออเดอร์เข้า Alex แม้ติดคิว" : "ส่งออเดอร์เข้า Alex แม้ยังตรวจคิวไม่ได้" : "ส่งออเดอร์เข้า Alex"}</button>
      </fieldset>
    </form>}
  </div>;
}

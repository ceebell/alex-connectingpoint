"use client";
import { CSSProperties, PointerEvent, ReactNode, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Attachment, Amounts, Availability, CartItem, cleanPhone, Draft, emptyAmounts, emptyRental, Fields, labels, LineEvent, RECEIVING_METHODS, receivingMethod, Rental, STORE_PICKUP } from "@/lib/api";
import CustomerExtractButton from "./customer-extract-button";
import OrderCart, { price, QueueCheck } from "./order-cart";
import RentalCheckout from "./rental-checkout";

const rentalKeys = Object.keys(emptyRental()) as (keyof Rental)[];
const day = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", year: "numeric", timeZone: "Asia/Bangkok" }).format(new Date(`${value}T00:00:00+07:00`));
const quotedKeys = ["rental_fee_satang", "deposit_satang", "shipping_fee_satang"] as const;

// Older drafts kept dates, fees and notes per item; lift them to the order, which is what Alex books.
function orderLevel(initial: Fields): Fields {
  const fields = structuredClone(initial);
  const rental = emptyRental();
  for (const key of rentalKeys) rental[key] = fields.rental?.[key] ?? fields.items.find(item => item.rental?.[key])?.rental?.[key] ?? null;
  const extra: string[] = [];
  // A return-by time used to have its own field; it now lives in the notes.
  const deadline = (fields.rental as { return_shipping_deadline?: string | null } | undefined)?.return_shipping_deadline;
  if (deadline) extra.push(`ส่งคืนก่อน ${deadline}`);
  // The shop ships by Flash unless the order says otherwise; carriers use Alex's choices (same rule as the backend).
  rental.fulfillment_method ??= "delivery";
  if (rental.fulfillment_method === "delivery") {
    const written = rental.delivery_provider;
    rental.delivery_provider = written ? receivingMethod(written) ?? "ขนส่งอื่น" : "Flash";
    if (written && !receivingMethod(written)) extra.push(`ขนส่ง: ${written}`);
  } else rental.delivery_provider = null;
  const amounts: Amounts = { ...emptyAmounts(), ...fields.amounts };
  // No discount is shown as 0; the cart fills in the quoted-price difference when there is one.
  amounts.rental_discount_satang ??= 0;
  amounts.deposit_discount_satang ??= 0;
  for (const key of quotedKeys) {
    const perItem = fields.items.map(item => item.amounts?.[key]).filter((value): value is number => typeof value === "number");
    if (amounts[key] == null && perItem.length) amounts[key] = perItem.reduce((sum, value) => sum + value, 0);
  }
  const notes = [fields.notes, ...fields.items.map((item, i) => item.notes ? (fields.items.length > 1 ? `รายการ ${i + 1}: ${item.notes}` : item.notes) : null), ...extra.filter(note => !fields.notes?.includes(note))].filter(Boolean);
  return { ...fields, rental, amounts, notes: notes.join("\n") || null, items: fields.items.map(item => ({ product_code: item.product_code, product_name: item.product_name ?? null, category: item.category ?? null, color: item.color, size: item.size, quantity: item.quantity })) };
}

// The invoice opens as a modal page; the native dialog handles focus and the Escape key.
// It is portalled to <body> and scrolls in its own body so wheel/touch never reach the panels behind.
// It opens centred; dragging the top bar moves it, and a double-click puts it back in the centre.
function InvoiceDialog({ onClose, children }: { onClose: () => void; children: ReactNode }) {
  const ref = useRef<HTMLDialogElement>(null);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  function startDrag(event: PointerEvent<HTMLDivElement>) {
    const box = ref.current?.getBoundingClientRect();
    if (!box || event.button !== 0 || (event.target as HTMLElement).closest("button")) return;
    event.preventDefault();
    const bar = event.currentTarget;
    // Centred position without the current offset, so the clamp below works in the same coordinates as the offset.
    const left = box.left - offset.x, top = box.top - offset.y, grabX = event.clientX - offset.x, grabY = event.clientY - offset.y;
    bar.setPointerCapture(event.pointerId);
    // Keep part of the bar on screen so the dialog can always be grabbed again.
    const move = (moved: globalThis.PointerEvent) => setOffset({
      x: Math.min(Math.max(moved.clientX - grabX, 120 - left - box.width), window.innerWidth - 120 - left),
      y: Math.min(Math.max(moved.clientY - grabY, -top), window.innerHeight - 48 - top),
    });
    const stop = () => { bar.removeEventListener("pointermove", move); bar.removeEventListener("pointerup", stop); bar.removeEventListener("pointercancel", stop); };
    bar.addEventListener("pointermove", move); bar.addEventListener("pointerup", stop); bar.addEventListener("pointercancel", stop);
  }
  useEffect(() => {
    const dialog = ref.current;
    if (dialog && !dialog.open) dialog.showModal();
    const overflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => { document.body.style.overflow = overflow; };
  }, []);
  return createPortal(<dialog ref={ref} className="invoice-dialog" aria-labelledby="invoice-title" onClose={onClose} style={{ "--drag-x": `${offset.x}px`, "--drag-y": `${offset.y}px` } as CSSProperties}>
    <div className="invoice-dialog-bar" onPointerDown={startDrag} onDoubleClick={() => setOffset({ x: 0, y: 0 })}>
      <span className="invoice-dialog-grip" aria-hidden="true" title="ดับเบิลคลิกเพื่อกลับกลางจอ">⠿ ลากแถบนี้เพื่อย้าย</span>
      <button type="button" className="text-button" onClick={() => ref.current?.close()}><span aria-hidden="true">✕</span> ปิดหน้าสรุป</button>
    </div>
    <div className="invoice-dialog-body">{children}</div>
  </dialog>, document.body);
}

export default function DraftEditor({ inquiryId, userId, draft, initial, busy, messages, attachments, onSave, onUnauthorized }: { inquiryId: string; userId?: string; draft: Draft | undefined; initial: Fields; busy: boolean; messages: LineEvent[]; attachments: Attachment[]; onSave: (fields: Fields, confirm: boolean) => Promise<void>; onUnauthorized: () => void }) {
  const [fields, setFields] = useState<Fields>(() => orderLevel(initial));
  // The bill is built from the saved draft, so the invoice waits until the form is saved.
  const [baseline] = useState(() => JSON.stringify(orderLevel(initial)));
  const dirty = JSON.stringify(fields) !== baseline;
  const [invoice, setInvoice] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [cart, setCart] = useState<CartItem[]>([]);
  // A queue result belongs to the dates and cart it was checked with; it is hidden once either changes.
  const [queue, setQueue] = useState<{ key: string; result: Availability } | null>(null);
  const queueKey = `${fields.rental.pickup_date}|${fields.rental.return_date}|${cart.map(item => `${item._id}:${item.quantity}`).join(",")}`;
  const availability = queue?.key === queueKey ? queue.result : null;
  const receiving = fields.rental.fulfillment_method === "store_pickup" ? STORE_PICKUP : fields.rental.delivery_provider ?? "Flash";
  const rentalTotal = cart.reduce((sum, item) => sum + (item.candidate.rental_fee_satang ?? 0) * item.quantity, 0);
  const depositTotal = cart.reduce((sum, item) => sum + (item.candidate.deposit_satang ?? 0) * item.quantity, 0);
  const amount = (key: keyof Amounts) => fields.amounts[key] ?? null;
  const setRental = (key: keyof Rental, value: string) => setFields(current => ({ ...current, rental: { ...current.rental, [key]: value || null } }));
  const setReceiving = (method: string) => setFields(current => ({ ...current, rental: { ...current.rental, fulfillment_method: method === STORE_PICKUP ? "store_pickup" : "delivery", delivery_provider: method === STORE_PICKUP ? null : method } }));
  const setAmount = (key: keyof Amounts, value: string) => setFields(current => ({ ...current, amounts: { ...current.amounts, [key]: value ? Math.round(Number(value) * 100) : null } }));
  // Discounts are filled with the quoted-price difference (Alex price in the cart − quoted fee, 0 when none)
  // and follow cart changes while they still hold the previous automatic value; a value the admin typed is kept.
  // When the saved cart first loads, only a discount the saved draft left empty is filled.
  const cartLoaded = useRef(false);
  function cartChanged(items: CartItem[]) {
    const previous = cart;
    const first = !cartLoaded.current;
    cartLoaded.current = true;
    setCart(items);
    setFields(current => {
      const amounts = { ...current.amounts };
      for (const [key, fee] of [["rental_discount_satang", "rental_fee_satang"], ["deposit_discount_satang", "deposit_satang"]] as const) {
        const quoted = current.amounts[fee] ?? null;
        const auto = (list: CartItem[]) => !list.length || quoted === null ? 0 : Math.max(0, list.reduce((sum, item) => sum + (item.candidate[fee] ?? 0) * item.quantity, 0) - quoted);
        const now = current.amounts[key] ?? null;
        if (now === null || (first ? initial.amounts?.[key] == null : now === auto(previous))) amounts[key] = auto(items);
      }
      return JSON.stringify(amounts) === JSON.stringify(current.amounts) ? current : { ...current, amounts };
    });
  }
  const text = (key: "intent" | "customer_name" | "customer_phone" | "customer_address" | "branch") => <label className={key === "customer_address" ? "span-2" : ""}>{labels[key]}{key === "customer_address" ? <textarea rows={3} value={fields[key] ?? ""} maxLength={1000} disabled={busy} onChange={e => setFields({ ...fields, [key]: e.target.value || null })} /> : <input value={fields[key] ?? ""} maxLength={200} disabled={busy} inputMode={key === "customer_phone" ? "tel" : undefined} onChange={e => setFields({ ...fields, [key]: (key === "customer_phone" ? cleanPhone(e.target.value) : e.target.value) || null })} />}</label>;
  const money = (key: keyof Amounts, label: string, max?: number, auto?: number) => <label>{label} (บาท)<input type="number" min="0" max={max === undefined ? undefined : max / 100} step="0.01" disabled={busy} placeholder={auto ? `อัตโนมัติ ${price(auto)}` : undefined} value={amount(key) == null ? "" : amount(key)! / 100} onChange={e => setAmount(key, e.target.value)} /></label>;
  // Same rule as the bill on the server: with no written discount, a fee quoted below the Alex prices in the cart is the discount.
  const fromQuote = (quoted: number | null, catalog: number) => !cart.length || quoted == null ? 0 : Math.max(0, catalog - quoted);
  const autoRental = fromQuote(amount("rental_fee_satang"), rentalTotal);
  const autoDeposit = fromQuote(amount("deposit_satang"), depositTotal);
  const rentalDiscount = amount("rental_discount_satang") ?? autoRental;
  const depositDiscount = amount("deposit_discount_satang") ?? autoDeposit;
  const usesAuto = (autoRental > 0 && rentalDiscount === autoRental) || (autoDeposit > 0 && depositDiscount === autoDeposit);
  const aboveCatalog = !!cart.length && ((amount("rental_fee_satang") ?? 0) > rentalTotal || (amount("deposit_satang") ?? 0) > depositTotal);
  const tooMuch = !!cart.length && (rentalDiscount > rentalTotal || depositDiscount > depositTotal);

  return <div className="draft-form">
    <section className="order-section section-customer">
      <h3 className="form-heading"><span aria-hidden="true">👤</span> ลูกค้า</h3>
      <div className="field-grid">{text("customer_name")}{text("customer_phone")}{text("customer_address")}</div>
      <CustomerExtractButton inquiryId={inquiryId} disabled={busy} onUnauthorized={onUnauthorized} onFill={value => setFields(current => ({ ...current, customer_name: value.display_name ?? current.customer_name, customer_phone: value.phone ? cleanPhone(value.phone) : current.customer_phone, customer_address: value.address ?? current.customer_address }))} />
    </section>
    <OrderCart inquiryId={inquiryId} draft={draft} items={fields.items} messages={messages} attachments={attachments} availability={availability} disabled={busy} onCart={cartChanged} onUnauthorized={onUnauthorized} />
    <section className="order-section section-dates">
      <h3 className="form-heading"><span aria-hidden="true">📅</span> วันรับ–คืนชุด <span className="muted small">ใช้กับทั้งออเดอร์</span></h3>
      {!!draft?.result?.date_options?.length && <div className="request-list date-options"><span className="muted small">วันที่อ่านได้จากรูป/สรุป (กดเพื่อใช้):</span>{draft.result.date_options.map((option, i) => {
        const chosen = option.pickup_date === fields.rental.pickup_date && option.return_date === fields.rental.return_date;
        return <button type="button" key={i} className={`request-chip ${chosen ? "selected" : ""}`} aria-pressed={chosen} disabled={busy} onClick={() => setFields(current => ({ ...current, rental: { ...current.rental, pickup_date: option.pickup_date ?? current.rental.pickup_date, return_date: option.return_date ?? current.rental.return_date } }))}>{option.label}<span> · รับ {option.pickup_date ? day(option.pickup_date) : "ยังไม่ระบุ"} – คืน {option.return_date ? day(option.return_date) : "ยังไม่ระบุ"}</span></button>;
      })}</div>}
      <div className="field-grid">
        {(["pickup_date", "return_date"] as const).map(key => <label key={key}>{labels[`rental.${key}`]}<input type="date" value={fields.rental[key] ?? ""} disabled={busy} onChange={e => setRental(key, e.target.value)} /></label>)}
        <label>วิธีมารับ<select value={receiving} disabled={busy} onChange={e => setReceiving(e.target.value)}>{RECEIVING_METHODS.map(method => <option key={method} value={method}>{method}</option>)}</select></label>
      </div>
      <QueueCheck inquiryId={inquiryId} pickup={fields.rental.pickup_date} returned={fields.rental.return_date} count={cart.length} result={availability} disabled={busy} onResult={result => setQueue({ key: queueKey, result })} onUnauthorized={onUnauthorized} />
    </section>
    <section className="order-section section-amounts">
      <h3 className="form-heading"><span aria-hidden="true">🏷️</span> ส่วนลดและค่าจัดส่ง <span className="muted small">ใช้กับทั้งออเดอร์</span></h3>
      <p className="muted small">ตามแค็ตตาล็อก: ค่าเช่า {price(rentalTotal)} บาท · ค่าประกัน {price(depositTotal)} บาท{amount("rental_fee_satang") != null && <><br />ราคาที่แจ้งลูกค้า: ค่าเช่า {price(amount("rental_fee_satang"))} บาท{amount("deposit_satang") != null && ` · ค่าประกัน ${price(amount("deposit_satang"))} บาท`}</>}</p>
      <div className="field-grid">
        {money("rental_discount_satang", "ส่วนลดค่าเช่า", cart.length ? rentalTotal : undefined, autoRental)}
        {money("deposit_discount_satang", "ส่วนลดค่าประกัน", cart.length ? depositTotal : undefined, autoDeposit)}
        {money("shipping_fee_satang", "ค่าจัดส่ง")}
      </div>
      {usesAuto && <p className="notice info">ใส่ส่วนลดอัตโนมัติจากราคาที่แจ้งลูกค้าเทียบกับราคาใน Alex{autoRental > 0 && rentalDiscount === autoRental && ` · ค่าเช่า ${price(autoRental)} บาท`}{autoDeposit > 0 && depositDiscount === autoDeposit && ` · ค่าประกัน ${price(autoDeposit)} บาท`} — แก้ตัวเลขได้ หรือกรอก 0 ถ้าไม่ให้ส่วนลด แล้วกดบันทึกร่าง</p>}
      {aboveCatalog && <p className="notice warning">ราคาที่แจ้งลูกค้าสูงกว่าราคาใน Alex — Alex ไม่มีช่องบวกเพิ่ม บิลจะใช้ราคาใน Alex กรุณาตรวจสินค้าในตะกร้า</p>}
      {!!cart.length && <p className={tooMuch ? "notice warning" : "cart-total"}>{tooMuch ? "ส่วนลดมากกว่ายอดตามแค็ตตาล็อก" : `ยอดหลังหักส่วนลด: ค่าเช่า ${price(rentalTotal - rentalDiscount)} บาท · ค่าประกัน ${price(depositTotal - depositDiscount)} บาท`}</p>}
      <label className="order-notes">หมายเหตุ / อุปกรณ์<textarea rows={3} value={fields.notes ?? ""} maxLength={3000} disabled={busy} onChange={e => setFields({ ...fields, notes: e.target.value || null })} /></label>
    </section>
    <details className="product-filters"><summary>ข้อมูลอื่นในร่าง</summary><div className="field-grid">{text("intent")}{text("branch")}</div></details>
    <label className="checkbox-row"><input type="checkbox" checked={confirm} onChange={e => setConfirm(e.target.checked)} />ยืนยันข้อมูลที่กรอกครบในร่างนี้</label>
    <p className="muted small">ตะกร้าบันทึกทันทีเมื่อเพิ่มหรือแก้จำนวน ส่วนลูกค้า วันที่ ส่วนลด และหมายเหตุบันทึกเมื่อกดปุ่มด้านล่าง</p>
    <button type="button" className={`button ${dirty || !draft ? "primary" : "secondary"} full`} disabled={busy || tooMuch} onClick={() => onSave(fields, confirm)}>{busy ? "กำลังบันทึก…" : "บันทึกร่างเป็นเวอร์ชันใหม่"}</button>
    <div className="order-submit">
      <button type="button" className={`button ${dirty || !draft ? "secondary" : "primary"} full`} disabled={busy || dirty || !draft || !cart.length} onClick={() => setInvoice(true)}><span aria-hidden="true">🧾</span> สรุปออเดอร์และส่งเข้า Alex</button>
      <p className="muted small">{!cart.length ? "เพิ่มสินค้าลงตะกร้าก่อนสร้างออเดอร์" : dirty || !draft ? "มีข้อมูลที่ยังไม่บันทึก กรุณาบันทึกร่างก่อนสร้างออเดอร์" : "เปิดใบสรุปเพื่อตรวจยอด เช็คคิว และยืนยันส่งออเดอร์เข้า Alex"}</p>
    </div>
    {invoice && <InvoiceDialog onClose={() => setInvoice(false)}><RentalCheckout inquiryId={inquiryId} userId={userId} disabled={busy} titleId="invoice-title" onUnauthorized={onUnauthorized} /></InvoiceDialog>}
  </div>;
}

"use client";
import { useState } from "react";
import { Attachment, Amounts, CartItem, cleanPhone, Draft, emptyAmounts, emptyRental, Fields, labels, LineEvent, Rental } from "@/lib/api";
import CustomerExtractButton from "./customer-extract-button";
import OrderCart, { price, QueueCheck } from "./order-cart";

const rentalKeys = Object.keys(emptyRental()) as (keyof Rental)[];
const day = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", year: "numeric", timeZone: "Asia/Bangkok" }).format(new Date(`${value}T00:00:00+07:00`));
const quotedKeys = ["rental_fee_satang", "deposit_satang", "shipping_fee_satang"] as const;

// Older drafts kept dates, fees and notes per item; lift them to the order, which is what Alex books.
function orderLevel(initial: Fields): Fields {
  const fields = structuredClone(initial);
  const rental = emptyRental();
  for (const key of rentalKeys) rental[key] = fields.rental?.[key] ?? fields.items.find(item => item.rental?.[key])?.rental?.[key] ?? null;
  // The shop ships by Flash unless the order says otherwise (same rule as the backend).
  rental.fulfillment_method ??= "delivery";
  if (rental.fulfillment_method === "delivery") rental.delivery_provider ??= "Flash";
  const amounts: Amounts = { ...emptyAmounts(), ...fields.amounts };
  for (const key of quotedKeys) {
    const perItem = fields.items.map(item => item.amounts?.[key]).filter((value): value is number => typeof value === "number");
    if (amounts[key] == null && perItem.length) amounts[key] = perItem.reduce((sum, value) => sum + value, 0);
  }
  const notes = [fields.notes, ...fields.items.map((item, i) => item.notes ? (fields.items.length > 1 ? `รายการ ${i + 1}: ${item.notes}` : item.notes) : null)].filter(Boolean);
  return { ...fields, rental, amounts, notes: notes.join("\n") || null, items: fields.items.map(item => ({ product_code: item.product_code, product_name: item.product_name ?? null, category: item.category ?? null, color: item.color, size: item.size, quantity: item.quantity })) };
}

export default function DraftEditor({ inquiryId, draft, initial, busy, messages, attachments, onSave, onUnauthorized }: { inquiryId: string; draft: Draft | undefined; initial: Fields; busy: boolean; messages: LineEvent[]; attachments: Attachment[]; onSave: (fields: Fields, confirm: boolean) => Promise<void>; onUnauthorized: () => void }) {
  const [fields, setFields] = useState<Fields>(() => orderLevel(initial));
  const [confirm, setConfirm] = useState(false);
  const [cart, setCart] = useState<CartItem[]>([]);
  const rentalTotal = cart.reduce((sum, item) => sum + (item.candidate.rental_fee_satang ?? 0) * item.quantity, 0);
  const depositTotal = cart.reduce((sum, item) => sum + (item.candidate.deposit_satang ?? 0) * item.quantity, 0);
  const amount = (key: keyof Amounts) => fields.amounts[key] ?? null;
  const setRental = (key: keyof Rental, value: string) => setFields(current => ({ ...current, rental: { ...current.rental, [key]: value || null } }));
  const setAmount = (key: keyof Amounts, value: string) => setFields(current => ({ ...current, amounts: { ...current.amounts, [key]: value ? Math.round(Number(value) * 100) : null } }));
  const text = (key: "intent" | "customer_name" | "customer_phone" | "customer_address" | "branch") => <label className={key === "customer_address" ? "span-2" : ""}>{labels[key]}{key === "customer_address" ? <textarea rows={3} value={fields[key] ?? ""} maxLength={1000} disabled={busy} onChange={e => setFields({ ...fields, [key]: e.target.value || null })} /> : <input value={fields[key] ?? ""} maxLength={200} disabled={busy} inputMode={key === "customer_phone" ? "tel" : undefined} onChange={e => setFields({ ...fields, [key]: (key === "customer_phone" ? cleanPhone(e.target.value) : e.target.value) || null })} />}</label>;
  const money = (key: keyof Amounts, label: string, max?: number) => <label>{label} (บาท)<input type="number" min="0" max={max === undefined ? undefined : max / 100} step="0.01" disabled={busy} value={amount(key) == null ? "" : amount(key)! / 100} onChange={e => setAmount(key, e.target.value)} /></label>;
  const rentalDiscount = amount("rental_discount_satang") ?? 0;
  const depositDiscount = amount("deposit_discount_satang") ?? 0;
  const tooMuch = !!cart.length && (rentalDiscount > rentalTotal || depositDiscount > depositTotal);

  return <div className="draft-form">
    <section className="order-section">
      <h3 className="form-heading"><span aria-hidden="true">👤</span> ลูกค้า</h3>
      <CustomerExtractButton inquiryId={inquiryId} disabled={busy} onUnauthorized={onUnauthorized} onFill={value => setFields(current => ({ ...current, customer_name: value.display_name ?? current.customer_name, customer_phone: value.phone ? cleanPhone(value.phone) : current.customer_phone, customer_address: value.address ?? current.customer_address }))} />
      <div className="field-grid">{text("customer_name")}{text("customer_phone")}{text("customer_address")}</div>
    </section>
    <OrderCart inquiryId={inquiryId} draft={draft} items={fields.items} messages={messages} attachments={attachments} disabled={busy} onCart={setCart} onUnauthorized={onUnauthorized} />
    <section className="order-section">
      <h3 className="form-heading"><span aria-hidden="true">📅</span> วันรับ–คืนชุด <span className="muted small">ใช้กับทั้งออเดอร์</span></h3>
      {!!draft?.result?.date_options?.length && <div className="request-list date-options"><span className="muted small">วันที่อ่านได้จากรูป/สรุป (กดเพื่อใช้):</span>{draft.result.date_options.map((option, i) => {
        const chosen = option.pickup_date === fields.rental.pickup_date && option.return_date === fields.rental.return_date;
        return <button type="button" key={i} className={`request-chip ${chosen ? "selected" : ""}`} aria-pressed={chosen} disabled={busy} onClick={() => setFields(current => ({ ...current, rental: { ...current.rental, pickup_date: option.pickup_date ?? current.rental.pickup_date, return_date: option.return_date ?? current.rental.return_date } }))}>{option.label}<span> · รับ {option.pickup_date ? day(option.pickup_date) : "ยังไม่ระบุ"} – คืน {option.return_date ? day(option.return_date) : "ยังไม่ระบุ"}</span></button>;
      })}</div>}
      <div className="field-grid">
        {(["pickup_date", "return_date"] as const).map(key => <label key={key}>{labels[`rental.${key}`]}<input type="date" value={fields.rental[key] ?? ""} disabled={busy} onChange={e => setRental(key, e.target.value)} /></label>)}
        <label>วิธีรับสินค้า<select value={fields.rental.fulfillment_method ?? ""} disabled={busy} onChange={e => { const method = e.target.value; setFields(current => ({ ...current, rental: { ...current.rental, fulfillment_method: method || null, delivery_provider: method === "delivery" ? current.rental.delivery_provider ?? "Flash" : current.rental.delivery_provider } })); }}><option value="delivery">จัดส่ง</option><option value="store_pickup">รับหน้าร้าน</option><option value="">ยังไม่ระบุ</option></select></label>
        <label>ขนส่ง<input value={fields.rental.delivery_provider ?? ""} disabled={busy} maxLength={100} onChange={e => setRental("delivery_provider", e.target.value)} /></label>
        <label>ส่งคืนก่อนเวลา<input type="time" value={fields.rental.return_shipping_deadline ?? ""} disabled={busy} onChange={e => setRental("return_shipping_deadline", e.target.value)} /></label>
      </div>
      <QueueCheck key={`${fields.rental.pickup_date}-${fields.rental.return_date}-${cart.map(item => `${item._id}:${item.quantity}`).join(",")}`} inquiryId={inquiryId} pickup={fields.rental.pickup_date} returned={fields.rental.return_date} count={cart.length} disabled={busy} onUnauthorized={onUnauthorized} />
    </section>
    <section className="order-section">
      <h3 className="form-heading"><span aria-hidden="true">🏷️</span> ส่วนลดและค่าจัดส่ง <span className="muted small">ใช้กับทั้งออเดอร์</span></h3>
      <p className="muted small">ตามแค็ตตาล็อก: ค่าเช่า {price(rentalTotal)} บาท · ค่าประกัน {price(depositTotal)} บาท{amount("rental_fee_satang") != null && <><br />ราคาที่แจ้งลูกค้า: ค่าเช่า {price(amount("rental_fee_satang"))} บาท{amount("deposit_satang") != null && ` · ค่าประกัน ${price(amount("deposit_satang"))} บาท`}</>}</p>
      <div className="field-grid">
        {money("rental_discount_satang", "ส่วนลดค่าเช่า", cart.length ? rentalTotal : undefined)}
        {money("deposit_discount_satang", "ส่วนลดค่าประกัน", cart.length ? depositTotal : undefined)}
        {money("shipping_fee_satang", "ค่าจัดส่ง")}
      </div>
      {!!cart.length && <p className={tooMuch ? "notice warning" : "cart-total"}>{tooMuch ? "ส่วนลดมากกว่ายอดตามแค็ตตาล็อก" : `ยอดหลังหักส่วนลด: ค่าเช่า ${price(rentalTotal - rentalDiscount)} บาท · ค่าประกัน ${price(depositTotal - depositDiscount)} บาท`}</p>}
      <label className="order-notes">หมายเหตุ / อุปกรณ์<textarea rows={3} value={fields.notes ?? ""} maxLength={3000} disabled={busy} onChange={e => setFields({ ...fields, notes: e.target.value || null })} /></label>
    </section>
    <details className="product-filters"><summary>ข้อมูลอื่นในร่าง</summary><div className="field-grid">{text("intent")}{text("branch")}</div></details>
    <label className="checkbox-row"><input type="checkbox" checked={confirm} onChange={e => setConfirm(e.target.checked)} />ยืนยันข้อมูลที่กรอกครบในร่างนี้</label>
    <p className="muted small">ตะกร้าบันทึกทันทีเมื่อเพิ่มหรือแก้จำนวน ส่วนลูกค้า วันที่ ส่วนลด และหมายเหตุบันทึกเมื่อกดปุ่มด้านล่าง</p>
    <button type="button" className="button primary full" disabled={busy || tooMuch} onClick={() => onSave(fields, confirm)}>{busy ? "กำลังบันทึก…" : "บันทึกร่างเป็นเวอร์ชันใหม่"}</button>
  </div>;
}

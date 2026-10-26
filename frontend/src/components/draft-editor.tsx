"use client";
import { useState } from "react";
import { cleanPhone, Fields, labels } from "@/lib/api";
import DraftItemEditor from "./draft-item-editor";
import CustomerExtractButton from "./customer-extract-button";

export default function DraftEditor({ inquiryId, initial, busy, onSave }: { inquiryId: string; initial: Fields; busy: boolean; onSave: (fields: Fields, confirm: boolean) => Promise<void> }) {
  const [fields, setFields] = useState<Fields>(structuredClone(initial));
  const [confirm, setConfirm] = useState(false);
  const text = (key: "intent" | "customer_name" | "customer_phone" | "customer_address" | "branch" | "notes") => <label className={key === "notes" || key === "customer_address" ? "span-2" : ""}>{labels[key]}{key === "customer_address" || key === "notes" ? <textarea rows={3} value={fields[key] ?? ""} maxLength={key === "notes" ? 3000 : 1000} disabled={busy} onChange={e => setFields({ ...fields, [key]: e.target.value || null })} /> : <input value={fields[key] ?? ""} maxLength={200} disabled={busy} inputMode={key === "customer_phone" ? "tel" : undefined} onChange={e => setFields({ ...fields, [key]: (key === "customer_phone" ? cleanPhone(e.target.value) : e.target.value) || null })} />}</label>;
  return <form onSubmit={async e => { e.preventDefault(); await onSave(fields, confirm); }} className="draft-form">
    <CustomerExtractButton inquiryId={inquiryId} disabled={busy} onFill={value => setFields(current => ({ ...current, customer_name: value.display_name ?? current.customer_name, customer_phone: value.phone ? cleanPhone(value.phone) : current.customer_phone, customer_address: value.address ?? current.customer_address }))} />
    <div className="field-grid">{text("customer_name")}{text("customer_phone")}{text("customer_address")}{text("branch")}{text("intent")}</div>
    <div className="section-caption"><h3>รายการสินค้า</h3><button type="button" className="text-button" onClick={() => setFields({ ...fields, items: [...fields.items, { product_code: null, color: null, size: null, quantity: 1 }] })} disabled={fields.items.length >= 20}>+ เพิ่มรายการ</button></div>
    {fields.items.map((item, i) => <DraftItemEditor key={i} item={item} index={i} busy={busy} onRemove={() => setFields({ ...fields, items: fields.items.filter((_, n) => n !== i) })} onChange={item => setFields(current => ({ ...current, items: current.items.map((old, n) => n === i ? item : old) }))} />)}
    <h3 className="form-heading">วันและวิธีรับสินค้า</h3><div className="field-grid">
      {(["pickup_date", "return_date"] as const).map(key => <label key={key}>{labels[`rental.${key}`]}<input type="date" value={fields.rental[key] ?? ""} disabled={busy} onChange={e => setFields({ ...fields, rental: { ...fields.rental, [key]: e.target.value || null } })} /></label>)}
      <label>วิธีรับสินค้า<select value={fields.rental.fulfillment_method ?? ""} onChange={e => setFields({ ...fields, rental: { ...fields.rental, fulfillment_method: e.target.value || null } })}><option value="">ยังไม่ระบุ</option><option value="store_pickup">รับหน้าร้าน</option><option value="delivery">จัดส่ง</option></select></label>
      <label>ขนส่ง<input value={fields.rental.delivery_provider ?? ""} disabled={busy} maxLength={100} onChange={e => setFields({ ...fields, rental: { ...fields.rental, delivery_provider: e.target.value || null } })} /></label>
      <label>ส่งคืนก่อนเวลา<input type="time" value={fields.rental.return_shipping_deadline ?? ""} disabled={busy} onChange={e => setFields({ ...fields, rental: { ...fields.rental, return_shipping_deadline: e.target.value || null } })} /></label>
    </div>
    <h3 className="form-heading">ยอดที่แจ้งไว้ <span className="muted small">ยังไม่ตรวจสอบกับ Alex</span></h3><div className="field-grid">
      {(["rental_fee_satang", "deposit_satang", "shipping_fee_satang"] as const).map(key => <label key={key}>{labels[`amounts.${key}`]} (บาท)<input type="number" step="0.01" min="0" disabled={busy} value={fields.amounts[key] == null ? "" : fields.amounts[key]! / 100} onChange={e => setFields({ ...fields, amounts: { ...fields.amounts, [key]: e.target.value ? Math.round(Number(e.target.value) * 100) : null } })} /></label>)}
      {text("notes")}
    </div>
    <label className="checkbox-row"><input type="checkbox" checked={confirm} onChange={e => setConfirm(e.target.checked)} />ยืนยันข้อมูลที่กรอกครบในร่างนี้</label>
    <p className="muted small">การยืนยันนี้ใช้เป็นข้อมูลอ้างอิงสำหรับร่างครั้งต่อไป ยังไม่ใช่การอนุมัติหรือส่งออเดอร์</p>
    <button className="button secondary full" disabled={busy}>{busy ? "กำลังบันทึก…" : "บันทึกร่างเป็นเวอร์ชันใหม่"}</button>
  </form>;
}

"use client";
import { useState } from "react";
import { Fields, labels } from "@/lib/api";

export default function DraftEditor({ initial, busy, onSave }: { initial: Fields; busy: boolean; onSave: (fields: Fields, confirm: boolean) => Promise<void> }) {
  const [fields, setFields] = useState<Fields>(structuredClone(initial));
  const [confirm, setConfirm] = useState(false);
  const text = (key: "intent" | "customer_name" | "customer_phone" | "branch" | "notes") => <label className={key === "notes" ? "span-2" : ""}>{labels[key]}<input value={fields[key] ?? ""} maxLength={key === "notes" ? 3000 : 200} onChange={e => setFields({ ...fields, [key]: e.target.value || null })} /></label>;
  return <form onSubmit={async e => { e.preventDefault(); await onSave(fields, confirm); }} className="draft-form">
    <div className="field-grid">{text("customer_name")}{text("customer_phone")}{text("branch")}{text("intent")}</div>
    <div className="section-caption"><h3>รายการสินค้า</h3><button type="button" className="text-button" onClick={() => setFields({ ...fields, items: [...fields.items, { product_code: null, color: null, size: null, quantity: null }] })} disabled={fields.items.length >= 20}>+ เพิ่มรายการ</button></div>
    {fields.items.map((item, i) => <div className="item-card" key={i}><div className="section-caption"><span>รายการ {i + 1}</span><button type="button" className="text-button danger" onClick={() => setFields({ ...fields, items: fields.items.filter((_, n) => n !== i) })}>ลบ</button></div><div className="field-grid">
      {(["product_code", "color", "size", "quantity"] as const).map(key => <label key={key}>{labels[key]}<input type={key === "quantity" ? "number" : "text"} min={key === "quantity" ? 1 : undefined} max={key === "quantity" ? 999 : undefined} step={key === "quantity" ? 1 : undefined} value={item[key] ?? ""} onChange={e => setFields({ ...fields, items: fields.items.map((value, n) => n === i ? { ...value, [key]: e.target.value ? key === "quantity" ? Number(e.target.value) : e.target.value : null } : value) })} /></label>)}
    </div></div>)}
    <h3 className="form-heading">วันและวิธีรับสินค้า</h3><div className="field-grid">
      {(["usage_date", "pickup_date", "return_date"] as const).map(key => <label key={key}>{labels[`rental.${key}`]}<input type="date" value={fields.rental[key] ?? ""} onChange={e => setFields({ ...fields, rental: { ...fields.rental, [key]: e.target.value || null } })} /></label>)}
      <label>วิธีรับสินค้า<select value={fields.rental.fulfillment_method ?? ""} onChange={e => setFields({ ...fields, rental: { ...fields.rental, fulfillment_method: e.target.value || null } })}><option value="">ยังไม่ระบุ</option><option value="store_pickup">รับหน้าร้าน</option><option value="delivery">จัดส่ง</option></select></label>
    </div>
    <h3 className="form-heading">ยอดที่แจ้งไว้ <span className="muted small">ยังไม่ตรวจสอบกับ Alex</span></h3><div className="field-grid">
      {(["rental_fee_satang", "deposit_satang"] as const).map(key => <label key={key}>{key === "rental_fee_satang" ? "ค่าเช่า (บาท)" : "มัดจำ (บาท)"}<input type="number" step="0.01" min="0" value={fields.amounts[key] === null ? "" : fields.amounts[key]! / 100} onChange={e => setFields({ ...fields, amounts: { ...fields.amounts, [key]: e.target.value ? Math.round(Number(e.target.value) * 100) : null } })} /></label>)}
      {text("notes")}
    </div>
    <label className="checkbox-row"><input type="checkbox" checked={confirm} onChange={e => setConfirm(e.target.checked)} />ยืนยันข้อมูลที่กรอกครบในร่างนี้</label>
    <p className="muted small">การยืนยันนี้ใช้เป็นข้อมูลอ้างอิงสำหรับร่างครั้งต่อไป ยังไม่ใช่การอนุมัติหรือส่งออเดอร์</p>
    <button className="button secondary full" disabled={busy}>{busy ? "กำลังบันทึก…" : "บันทึกร่างเป็นเวอร์ชันใหม่"}</button>
  </form>;
}

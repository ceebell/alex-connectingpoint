"use client";
import { DraftItem, emptyAmounts, emptyRental, labels } from "@/lib/api";

export default function DraftItemEditor({ item, index, busy, onChange, onRemove }: { item: DraftItem; index: number; busy: boolean; onChange: (item: DraftItem) => void; onRemove: () => void }) {
  const rental = item.rental ?? emptyRental();
  const amounts = item.amounts ?? emptyAmounts();
  return <div className="item-card"><div className="section-caption"><span>รายการ {index + 1}</span><button type="button" className="text-button danger" onClick={onRemove} disabled={busy}>ลบ</button></div>
    <div className="field-grid">{(["product_name", "product_code", "color", "size", "quantity"] as const).map(key => <label key={key} className={key === "product_name" ? "span-2" : ""}>{labels[key]}<input value={item[key] ?? ""} type={key === "quantity" ? "number" : "text"} min={key === "quantity" ? 1 : undefined} max={key === "quantity" ? 999 : undefined} maxLength={200} disabled={busy} onChange={e => onChange({ ...item, [key]: e.target.value ? key === "quantity" ? Number(e.target.value) : e.target.value : null })} /></label>)}</div>
    <details className="item-rental" open={!!item.rental || !!item.amounts || !!item.notes}><summary>วัน ราคา และหมายเหตุเฉพาะรายการนี้</summary><p className="muted small">เหมาะกับรูปแต่ละชุดที่มีวันเช่าหรือราคาต่างกัน ช่องวันที่ว่างจะใช้ข้อมูลร่วมด้านล่าง</p><div className="field-grid">
      {(["dispatch_date", "pickup_date", "usage_date", "return_date"] as const).map(key => <label key={key}>{labels[`rental.${key}`]}<input type="date" value={rental[key] ?? ""} disabled={busy} onChange={e => onChange({ ...item, rental: { ...rental, [key]: e.target.value || null } })} /></label>)}
      <label>วิธีรับสินค้า<select value={rental.fulfillment_method ?? ""} disabled={busy} onChange={e => onChange({ ...item, rental: { ...rental, fulfillment_method: e.target.value || null } })}><option value="">ใช้ข้อมูลร่วม / ยังไม่ระบุ</option><option value="store_pickup">รับหน้าร้าน</option><option value="delivery">จัดส่ง</option></select></label>
      <label>ขนส่ง<input value={rental.delivery_provider ?? ""} disabled={busy} onChange={e => onChange({ ...item, rental: { ...rental, delivery_provider: e.target.value || null } })} /></label>
      <label>ส่งคืนก่อนเวลา<input type="time" value={rental.return_shipping_deadline ?? ""} disabled={busy} onChange={e => onChange({ ...item, rental: { ...rental, return_shipping_deadline: e.target.value || null } })} /></label>
      {(["rental_fee_satang", "deposit_satang", "shipping_fee_satang"] as const).map(key => <label key={key}>{labels[`amounts.${key}`]} (บาท)<input type="number" min="0" step="0.01" value={amounts[key] == null ? "" : amounts[key]! / 100} disabled={busy} onChange={e => onChange({ ...item, amounts: { ...amounts, [key]: e.target.value ? Math.round(Number(e.target.value) * 100) : null } })} /></label>)}
      <label className="span-2">หมายเหตุ / อุปกรณ์<textarea rows={2} value={item.notes ?? ""} maxLength={1000} disabled={busy} onChange={e => onChange({ ...item, notes: e.target.value || null })} /></label>
    </div></details>
  </div>;
}

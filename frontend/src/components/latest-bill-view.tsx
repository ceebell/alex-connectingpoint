import { formatDate, formatDateTime } from "@/lib/dates";
import type { BillSnapshot } from "./rental-checkout";
import PaymentSlipNotice from "./payment-slip-notice";
import BillFeedbackCard from "./bill-feedback";
import { submissionFeedback } from "@/lib/bill-feedback";

const baht = (value: number) => (value / 100).toLocaleString("th-TH", { maximumFractionDigits: 2 });

export default function LatestBillView({ bill }: { bill: BillSnapshot | null }) {
  if (!bill) return <div className="empty-state"><h3>ยังไม่มีสรุปบิลล่าสุด</h3><p>เปิดแท็บร่างล่าสุด แล้วกด “สรุปออเดอร์และส่งเข้า Alex” เพื่อเตรียมข้อมูลบิล</p></div>;
  return <article className="invoice-body" aria-label="สรุปบิลล่าสุด">
    {bill.status !== "PREPARED" && <BillFeedbackCard value={submissionFeedback({ _id: bill.id, status: bill.status, alex_order_id: bill.alex_order_id, alex_order_number: bill.alex_order_number })} />}
    <header className="invoice-head"><div><h3>สรุปบิลล่าสุด{bill.draft_version ? ` · ร่าง v${bill.draft_version}` : ""}</h3>{bill.status === "PREPARED" && <p role="status">บันทึกสรุปแล้ว · ยังไม่ได้ส่งเข้า Alex</p>}</div></header>
    {bill.payment_slip && <PaymentSlipNotice match={bill.payment_slip} />}
    <section className="invoice-block"><h4>ลูกค้า</h4><dl className="bill-details"><div><dt>ชื่อ</dt><dd>{bill.customer.name || "—"}</dd></div><div><dt>โทรศัพท์</dt><dd>{bill.customer.phone || "—"}</dd></div><div><dt>ที่อยู่</dt><dd>{bill.customer.address || "—"}</dd></div></dl></section>
    <section className="invoice-block"><h4>รายการสินค้า</h4><dl className="bill-details"><div><dt>วันรับชุด</dt><dd>{formatDate(bill.group.pickup_date)}</dd></div><div><dt>วันคืนชุด</dt><dd>{formatDate(bill.group.return_date)}</dd></div></dl>{bill.group.lines.map((line, i) => <div className="bill-line" key={`${line.product_item_id}-${i}`}><strong>{line.name}</strong><p>{line.code}{line.size && ` · ${line.size}`}{line.is_shipping && " · ค่าขนส่ง"} · {line.quantity} รายการ</p><p>ค่าเช่า {baht(line.rental_fee_satang * line.quantity)} บาท · ค่าประกัน {baht(line.deposit_satang * line.quantity)} บาท</p></div>)}</section>
    <section className="invoice-block"><h4>การรับและชำระเงิน</h4><dl className="bill-details"><div><dt>วิธีมารับ</dt><dd>{bill.receiving_method || "—"}</dd></div><div><dt>วิธีชำระค่าเช่า</dt><dd>{bill.rental_payment_method || "—"}</dd></div><div><dt>วิธีชำระค่าประกัน</dt><dd>{bill.bail_payment_method || "—"}</dd></div><div><dt>หมายเหตุ / อุปกรณ์</dt><dd>{bill.notes || "—"}</dd></div><div><dt>โปรโมชั่น</dt><dd>{[bill.use_free_item_promotion && "ของแถม", bill.use_discount_promotion && "ส่วนลด"].filter(Boolean).join(" · ") || "ไม่ใช้โปรโมชั่น"}</dd></div></dl></section>
    <section className="invoice-totals" aria-label="ยอดบิลล่าสุด"><dl><div><dt>ยอดเช่า</dt><dd>{baht(bill.catalog_rental_satang)}</dd></div><div className="discount"><dt>ส่วนลดค่าเช่า</dt><dd>{baht(bill.rental_discount_satang)}</dd></div><div className="subtotal"><dt>ค่าเช่าสุทธิ</dt><dd>{baht(bill.rental_fee_satang)}</dd></div><div><dt>ยอดรวมค่าประกัน</dt><dd>{baht(bill.catalog_deposit_satang)}</dd></div><div className="discount"><dt>ส่วนลดค่าประกัน</dt><dd>{baht(bill.deposit_discount_satang)}</dd></div><div className="subtotal"><dt>ค่าประกันสุทธิ</dt><dd>{baht(bill.deposit_satang)}</dd></div></dl><div className="invoice-grand"><span>ยอดรวมทั้งหมด</span><strong>{baht(bill.rental_fee_satang + bill.deposit_satang)} <small>บาท</small></strong></div></section>
    {(bill.updated_by || bill.updated_at) && <p className="muted small">บันทึกโดย {bill.updated_by || "—"}{bill.updated_at && ` · ${formatDateTime(bill.updated_at)}`}</p>}
  </article>;
}

import type { PaymentSlipMatch } from "@/lib/payment-slip";

const baht = (value: number) => (value / 100).toLocaleString("th-TH", { maximumFractionDigits: 2 });

export default function PaymentSlipNotice({ match }: { match: PaymentSlipMatch }) {
  return <div className="notice info" aria-label="ข้อมูลชำระจากสลิป"><strong>โอนเงินทั้งค่าเช่าและค่าประกันตามสลิปที่แนบ</strong><p>ยอดสลิป {baht(match.slip.amount_satang)} บาท · ยอดสุทธิ {baht(match.net_satang)} บาท · ต่างกัน {baht(Math.abs(match.slip.amount_satang - match.net_satang))} บาท (ไม่เกิน {baht(match.tolerance_satang)} บาท)</p><p className="small muted">อ่านจากภาพ · {match.slip.bank} · อ้างอิง {match.slip.transaction_ref} · ยังไม่ได้ตรวจยืนยันกับธนาคาร</p></div>;
}

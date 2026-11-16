"use client";
import { useEffect, useState, type ReactNode } from "react";
import type { BillFeedback } from "@/lib/bill-feedback";

const titles = { saving: "กำลังบันทึกข้อมูลล่าสุด", checking: "กำลังตรวจราคาและคิวใน Alex", sending: "กำลังส่งออเดอร์เข้า Alex", checking_result: "กำลังตรวจผลคำขอเดิม", success: "ส่งออเดอร์เข้า Alex สำเร็จ", failed: "ส่งออเดอร์ไม่สำเร็จ", unknown: "ยังยืนยันผลการส่งไม่ได้", waiting: "Alex ยังประมวลผลคำขอนี้อยู่", blocked: "ยังไม่ได้ส่งออเดอร์เข้า Alex" };

export default function BillFeedbackCard({ value, children, onDismiss }: { value: BillFeedback; children?: ReactNode; onDismiss?: () => void }) {
  const [seconds, setSeconds] = useState(0);
  const progress = ["saving", "checking", "sending", "checking_result", "waiting"].includes(value.state);
  useEffect(() => {
    if (!progress) return;
    const started = Date.now();
    const timer = setInterval(() => setSeconds(Math.floor((Date.now() - started) / 1000)), 1000);
    return () => clearInterval(timer);
  }, [progress, value.state]);
  const urgent = ["failed", "unknown", "blocked"].includes(value.state);
  return <section className={`bill-feedback bill-feedback-${value.state}`} role={urgent ? "alert" : "status"} aria-live={urgent ? "assertive" : "polite"} aria-atomic="true" aria-label="ผลการส่งออเดอร์">
    <div className="bill-feedback-title">{progress && <span className="bill-feedback-spinner" aria-hidden="true" />}<strong>{titles[value.state]}</strong>{onDismiss && <button type="button" className="text-button" aria-label="ปิดข้อความผลการส่งออเดอร์" onClick={onDismiss}>✕</button>}</div>
    <p>{value.message}</p>
    {progress && seconds >= 20 && <p>ใช้เวลานานกว่าปกติ กำลังรอผลจากระบบ ข้อมูลที่กรอกยังอยู่ในหน้านี้</p>}
    {value.orderNumber && <p>Order Number: <strong>{value.orderNumber}</strong></p>}{value.orderId && <p>Order ID: <code>{value.orderId}</code></p>}
    {value.requestId && <p className="small">รหัสติดตามคำขอ: <code>{value.requestId}</code></p>}
    {children && <div className="bill-feedback-actions">{children}</div>}
  </section>;
}

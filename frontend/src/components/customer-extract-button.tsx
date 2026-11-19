"use client";
import { useState } from "react";
import { api, ApiError, errorText, Evidence } from "@/lib/api";

export type CustomerFields = { display_name: string | null; phone: string | null; address: string | null };
type Result = { fields: CustomerFields; evidence: Evidence[]; conflicts: { field: string; message: string }[]; questions: string[] };
const labels: Record<string, string> = { display_name: "ชื่อลูกค้า", phone: "เบอร์โทร", address: "ที่อยู่" };

export default function CustomerExtractButton({ inquiryId, disabled = false, onFill, onUnauthorized, onBusyChange }: { inquiryId: string; disabled?: boolean; onFill: (fields: CustomerFields) => void; onUnauthorized?: () => void; onBusyChange?: (busy: boolean) => void }) {
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Result | null>(null);
  const [error, setError] = useState("");
  async function extract() {
    setBusy(true); onBusyChange?.(true); setError(""); setResult(null);
    try {
      const result = await api<Result>(`/api/inquiries/${inquiryId}/customer-extract`, { method: "POST" });
      setResult(result); onFill(result.fields);
    } catch (cause) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized?.(); }
    finally { setBusy(false); onBusyChange?.(false); }
  }
  return <div className="customer-extract">
    <button type="button" className="button info full" disabled={busy || disabled} onClick={extract}><span aria-hidden="true">💬</span> {busy ? "กำลังค้นข้อมูลลูกค้า…" : "ค้นข้อมูลลูกค้า"}</button>
    <p className="muted small">ชื่อและเบอร์โทร: Note → ชื่อที่แสดง → แชท · ที่อยู่อ่านจากแชท</p>
    {error && <div className="notice warning" role="alert">{error}</div>}
    {result && <div className="notice info" role="status"><p>{Object.values(result.fields).some(Boolean) ? "เติมข้อมูลที่พบแล้ว กรุณาตรวจและกดบันทึก ช่องที่ไม่พบข้อมูลจะคงค่าเดิม" : "ยังไม่พบข้อมูลลูกค้าที่ชัดเจนใน Note ชื่อที่แสดง หรือแชทของเรื่องนี้"}</p>
      {result.conflicts.map((conflict, i) => <p key={i}>{labels[conflict.field] ?? conflict.field}: {conflict.message}</p>)}
      {result.questions.map((question, i) => <p key={i}>{question}</p>)}
      {result.evidence.length > 0 && <details className="customer-evidence"><summary>แหล่งข้อมูลอ้างอิง ({result.evidence.length})</summary>{result.evidence.map((evidence, i) => <div key={i}><strong>{labels[evidence.field] ?? evidence.field}</strong><p>{evidence.quote}</p>{evidence.source_type === "line_event" ? <a href={`#chat-message-${evidence.source_ref}`}>ดูข้อความในแชท</a> : <span className="muted small">{evidence.source_type === "customer_note" ? "จาก Note ของลูกค้า" : "จากชื่อที่แสดงของลูกค้า"}</span>}</div>)}</details>}
    </div>}
  </div>;
}

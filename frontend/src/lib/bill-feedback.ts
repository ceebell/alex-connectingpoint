export type BillFeedbackState = "saving" | "checking" | "sending" | "checking_result" | "success" | "failed" | "unknown" | "waiting" | "blocked";
export type BillFeedback = { state: BillFeedbackState; message: string; requestId?: string; orderId?: string | null; orderNumber?: string | null };
export type BillAttempt = { inquiryId: string; reviewId: string; draftVersion: number; acceptQueue: boolean };
export type SubmissionResult = { _id: string; status: string; alex_order_id?: string | null; alex_order_number?: string | null; http_status?: number };

export function submissionFeedback(result: SubmissionResult): BillFeedback {
  const base = { requestId: result._id, orderId: result.alex_order_id, orderNumber: result.alex_order_number };
  if (result.status === "CONFIRMED" && result.alex_order_id) return { ...base, state: "success", message: result.alex_order_number ? "Alex ยืนยันการสร้างออเดอร์แล้ว บันทึก Order ID และ Order Number ผูกกับลูกค้าและร่างที่ส่งไว้ในสรุปบิลล่าสุด" : "Alex ยืนยันการสร้างออเดอร์แล้วและบันทึก Order ID ไว้ แต่ยังไม่ได้รับ Order Number กรุณาตรวจเลขบิลใน Alex โดยไม่ส่งออเดอร์ซ้ำ" };
  if (result.status === "NOT_CREATED") return { ...base, state: "failed", message: "คำขอนี้ยังไม่ได้สร้างออเดอร์ ข้อมูลที่กรอกยังอยู่ ตรวจข้อมูลแล้วส่งใหม่ได้" };
  if (result.status === "SUBMITTING") return { ...base, state: "waiting", message: "คำขอเดิมยังอยู่ระหว่างดำเนินการ ระบบกำลังติดตามผลโดยไม่สร้างคำขอใหม่" };
  return { ...base, state: "unknown", message: result.http_status && result.http_status >= 400 ? `Alex ตอบข้อผิดพลาด HTTP ${result.http_status} และยังไม่มีเลขออเดอร์ยืนยัน กรุณาตรวจผลใน Alex ก่อนสร้างคำขอใหม่` : "ส่งคำขอแล้ว แต่ยังไม่ได้ผลยืนยันจาก Alex ออเดอร์อาจถูกสร้างแล้ว กรุณาตรวจผลก่อนสร้างคำขอใหม่" };
}

export function isPreflightRejection(code: string): boolean {
  return ["QUEUE_BLOCKED", "CONTEXT_CHANGED", "BILL_NOT_READY", "VALIDATION_ERROR", "DRAFT_CLOSED", "ALEX_NOT_CONFIGURED", "ALEX_SCOPE_DENIED"].includes(code);
}

const attemptKey = (inquiryId: string) => `alex:bill-attempt:${inquiryId}`;
export function rememberBillAttempt(attempt: BillAttempt): void {
  try { sessionStorage.setItem(attemptKey(attempt.inquiryId), JSON.stringify(attempt)); } catch { /* Storage may be unavailable in private browsing. */ }
}
export function forgetBillAttempt(inquiryId: string): void {
  try { sessionStorage.removeItem(attemptKey(inquiryId)); } catch { /* Server history still records accepted requests. */ }
}
export function restoreBillAttempt(inquiryId: string): BillAttempt | null {
  try {
    const value = JSON.parse(sessionStorage.getItem(attemptKey(inquiryId)) ?? "null") as BillAttempt | null;
    return value && value.inquiryId === inquiryId && typeof value.reviewId === "string" && value.reviewId.length > 0 && value.reviewId.length <= 200 && Number.isSafeInteger(value.draftVersion) && value.draftVersion > 0 && typeof value.acceptQueue === "boolean" ? value : null;
  } catch { return null; }
}

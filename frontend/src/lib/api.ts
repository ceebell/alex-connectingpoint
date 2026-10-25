export class ApiError extends Error {
  constructor(public code: string, public status: number) { super(code); }
}

const errors: Record<string, string> = {
  UNAUTHORIZED: "ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง หรือหมดเวลาเข้าสู่ระบบ",
  ADMIN_NOT_CONFIGURED: "ยังไม่ได้ตั้งค่าบัญชีแอดมิน กรุณาติดต่อผู้ดูแลระบบ",
  LOGIN_RATE_LIMITED: "ลองเข้าสู่ระบบหลายครั้ง กรุณารอ 1 นาที",
  HERMES_NOT_CONFIGURED: "AI ยังไม่พร้อมใช้งาน กรุณาติดต่อผู้ดูแลระบบ คุณยังแก้ร่างด้วยมือได้",
  HERMES_FAILED: "AI ทำงานไม่สำเร็จ กรุณาลองใหม่หรือตรวจการเชื่อมต่อกับผู้ดูแลระบบ",
  HERMES_TOOLS_ENABLED: "กรุณาให้ผู้ดูแลระบบตั้งค่า Hermes สำหรับการสกัดร่างก่อนใช้งาน",
  CONTEXT_CHANGED: "มีข้อความใหม่หรือข้อความถูกยกเลิก กรุณารอประมวลผลแล้วเตรียมร่างอีกครั้ง",
  DRAFT_CONFLICT: "ร่างมีการเปลี่ยนแปลงหรือมีผู้ดูแลกำลังทำงาน กรุณาโหลดข้อมูลล่าสุด",
  DATABASE_UNAVAILABLE: "ฐานข้อมูลไม่พร้อมใช้งาน กรุณาลองใหม่",
  VALIDATION_ERROR: "ข้อมูลไม่ผ่านการตรวจสอบ ตรวจวันที่ จำนวน เงิน หรือหลักฐาน แล้วลองใหม่",
  NOT_FOUND: "ไม่พบข้อมูล หรือรูปหมดอายุหรือถูกยกเลิกแล้ว",
  RECORD_CONFLICT: "ข้อมูลถูกแก้จากหน้าต่างอื่นก่อนหน้า ระบบโหลดข้อมูลล่าสุดแล้ว กรุณาตรวจแล้วบันทึกอีกครั้ง",
  ORDER_EXISTS: "เรื่องนี้อนุมัติเป็นออเดอร์ไปแล้ว",
  ORDER_NOT_READY: "อนุมัติได้เฉพาะร่างล่าสุดที่ข้อมูลครบและไม่มีข้อขัดแย้ง กรุณาโหลดข้อมูลล่าสุด",
};
export function errorText(error: unknown) {
  return error instanceof ApiError ? errors[error.code] ?? "ทำรายการไม่สำเร็จ กรุณาลองใหม่" : "เชื่อมต่อระบบไม่ได้ กรุณาลองใหม่";
}

// Normalize MongoDB Extended JSON at the API boundary.
function normalize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(normalize);
  if (value && typeof value === "object") {
    const record = value as Record<string, unknown>;
    if (typeof record.$oid === "string") return record.$oid;
    if (record.$date !== undefined) {
      const date = record.$date;
      const raw = date && typeof date === "object" ? (date as Record<string, string>).$numberLong : date;
      return new Date(typeof raw === "string" && /^\d+$/.test(raw) ? Number(raw) : raw as string | number).toISOString();
    }
    return Object.fromEntries(Object.entries(record).map(([key, child]) => [key, normalize(child)]));
  }
  return value;
}

export async function api<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(path, { ...options, cache: "no-store", credentials: "same-origin", headers: { "Content-Type": "application/json", "X-Admin-Request": "1", ...options.headers } });
  let value;
  try { value = await response.json(); } catch { throw new ApiError("CONNECTION_ERROR", response.status); }
  if (!response.ok) throw new ApiError(value.error?.code ?? "CONNECTION_ERROR", response.status);
  return normalize(value) as T;
}

export type Inquiry = { _id: string; line_user_id?: string; status: string; created_at: string; updated_at: string; draft_version?: number; order_id?: string };
export type LineEvent = { _id: string; event_at: string; event_id?: string; status: string; sender_role?: "customer" | "admin"; direction?: "inbound" | "outbound"; sender_name?: string; source?: string; delivery_status?: string; withdrawn?: boolean; media_id?: string; media_status?: string; payload?: { type: string; message?: { type: string; text?: string }; postback?: { data: string } } };
export type Fields = {
  intent: string | null; customer_name: string | null; customer_phone: string | null; branch: string | null;
  items: { product_code: string | null; color: string | null; size: string | null; quantity: number | null }[];
  rental: { usage_date: string | null; pickup_date: string | null; return_date: string | null; fulfillment_method: string | null };
  amounts: { rental_fee_satang: number | null; deposit_satang: number | null }; notes: string | null;
};
export type Evidence = { field: string; source_type: string; source_ref: string; quote: string };
export type Draft = { _id: string; draft_version: number; workflow_status: string; admin_summary?: string; stale: boolean; redacted: boolean; missing_fields: string[]; confirmed_fields?: string[]; created_at: string; result?: { fields: Fields; evidence: Evidence[]; conflicts: string[]; questions: string[] } };
export type Job = { id: string; status: string; error_code: string | null };
export type Profile = { _id: string; line_user_id: string; version: number; display_name: string | null; phone: string | null; address: string | null; notes: string | null; updated_by: string; updated_at: string };
export type Order = { _id: string; inquiry_id: string; draft_id: string; draft_version: number; status: string; approved_by: string; approved_at: string; fields: Fields };
export type DraftList = { version: number; drafts: Draft[]; latest_request?: Job | null; context_hash:string };
export const emptyFields = (): Fields => ({ intent: null, customer_name: null, customer_phone: null, branch: null, items: [], rental: { usage_date: null, pickup_date: null, return_date: null, fulfillment_method: null }, amounts: { rental_fee_satang: null, deposit_satang: null }, notes: null });
export const labels: Record<string, string> = { intent: "ความต้องการ", customer_name: "ชื่อลูกค้า", customer_phone: "เบอร์โทร", branch: "สาขา", items: "รายการสินค้า", product_code: "รหัสสินค้า", color: "สี", size: "ไซซ์", quantity: "จำนวน", "rental.usage_date": "วันใช้งาน", "rental.pickup_date": "วันรับ", "rental.return_date": "วันคืน", "rental.fulfillment_method": "วิธีรับสินค้า", "amounts.rental_fee_satang": "ค่าเช่า", "amounts.deposit_satang": "มัดจำ", notes: "หมายเหตุ" };
export function fieldLabel(key: string) { const match = key.match(/^items\.(\d+)\.(.+)$/); return match ? `สินค้ารายการ ${Number(match[1]) + 1} · ${labels[match[2]] ?? match[2]}` : labels[key] ?? key; }
export function filledKeys(fields: Fields) {
  const keys: string[] = [];
  const walk = (value: unknown, prefix: string) => {
    if (value === null || value === undefined || value === "") return;
    if (typeof value === "object") Object.entries(value).forEach(([key, child]) => walk(child, prefix ? `${prefix}.${key}` : key));
    else keys.push(prefix);
  };
  walk(fields, ""); return keys;
}

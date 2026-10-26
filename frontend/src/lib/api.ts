export class ApiError extends Error {
  constructor(public code: string, public status: number) { super(code); }
}

const errors: Record<string, string> = {
  BILL_NOT_READY: "บิลยังไม่พร้อม ตรวจข้อมูลลูกค้า วันเช่า รายการสินค้า และผลเช็คคิวล่าสุด",
  BILL_PENDING: "มีบิลหรือคำขอส่งไป Alex ที่ต้องตรวจผลก่อน กรุณาตรวจรายการบิลด้านบน",
  ALEX_NOT_CONFIGURED: "ยังไม่ได้ตั้งค่าบัญชี Alex กรุณาติดต่อผู้ดูแลระบบ",
  ALEX_FAILED: "อ่านแค็ตตาล็อก Alex ไม่สำเร็จ กรุณาลองใหม่หรือตรวจบัญชีกับผู้ดูแลระบบ",
  ALEX_SCOPE_DENIED: "บัญชี Alex ไม่มีสิทธิ์ในร้านหรือสาขาที่เลือก",
  PRODUCT_CONFLICT: "รายการสินค้าหรือร่างเปลี่ยนไป หรือผลค้นหาหมดอายุ กรุณาโหลดผลล่าสุดแล้วค้นหาใหม่",
  CATALOG_IMAGES_UNAVAILABLE: "อ่านรหัสจากรูปไม่ได้ และยังไม่มีรูปแค็ตตาล็อกที่ใช้เปรียบเทียบ กรุณาค้นด้วยข้อความ หรือเพิ่มรูปสินค้าใน Alex",
  UNAUTHORIZED: "ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง หรือหมดเวลาเข้าสู่ระบบ",
  ADMIN_NOT_CONFIGURED: "ยังไม่ได้ตั้งค่าบัญชีแอดมิน กรุณาติดต่อผู้ดูแลระบบ",
  LOGIN_RATE_LIMITED: "ลองเข้าสู่ระบบหลายครั้ง กรุณารอ 1 นาที",
  HERMES_NOT_CONFIGURED: "เซิร์ฟเวอร์ยังไม่ได้ตั้งค่าการเชื่อมต่อ Hermes คุณยังบันทึกสรุปและรูปไว้ก่อนได้",
  HERMES_FAILED: "AI ทำงานไม่สำเร็จ กรุณาลองใหม่หรือตรวจการเชื่อมต่อกับผู้ดูแลระบบ",
  HERMES_TOOLS_ENABLED: "กรุณาให้ผู้ดูแลระบบตั้งค่า Hermes สำหรับการสกัดร่างก่อนใช้งาน",
  CONTEXT_CHANGED: "มีข้อความใหม่หรือข้อความถูกยกเลิก กรุณารอประมวลผลแล้วเตรียมร่างอีกครั้ง",
  DRAFT_CONFLICT: "ร่างมีการเปลี่ยนแปลงหรือมีผู้ดูแลกำลังทำงาน กรุณาโหลดข้อมูลล่าสุด",
  DRAFT_WAITING: "ข้อความยังประมวลผลไม่เสร็จ กรุณารอสักครู่แล้วลองใหม่",
  SUMMARY_CONFLICT: "สรุปถูกแก้จากหน้าต่างอื่นแล้ว ข้อความที่กำลังแก้ยังอยู่ กรุณาโหลดสรุปล่าสุดก่อนบันทึก",
  DATABASE_UNAVAILABLE: "ฐานข้อมูลไม่พร้อมใช้งาน กรุณาลองใหม่",
  VALIDATION_ERROR: "ข้อมูลไม่ผ่านการตรวจสอบ ตรวจวันที่ จำนวน เงิน หรือหลักฐาน แล้วลองใหม่",
  NOT_FOUND: "ไม่พบข้อมูล หรือรูปหมดอายุหรือถูกยกเลิกแล้ว",
  RECORD_CONFLICT: "ข้อมูลถูกแก้จากหน้าต่างอื่นก่อนหน้า ระบบโหลดข้อมูลล่าสุดแล้ว กรุณาตรวจแล้วบันทึกอีกครั้ง",
  ORDER_EXISTS: "เรื่องนี้อนุมัติเป็นออเดอร์ไปแล้ว",
  IMAGE_UNAVAILABLE: "ยังส่งรูปไม่ได้ ผู้ดูแลระบบต้องตั้ง PUBLIC_BASE_URL (HTTPS) บน server",
  REPLY_UNAVAILABLE: "ส่งข้อความได้เฉพาะแชท 1:1 กับลูกค้าที่ทักเข้ามาทาง LINE OA",
  LINE_UNAVAILABLE: "ติดต่อ LINE ไม่สำเร็จ ข้อความยังไม่ถูกส่ง กดส่งอีกครั้งได้โดยไม่ส่งซ้ำ",
  LINE_RATE_LIMITED: "LINE ปฏิเสธเพราะส่งถี่เกินไปหรือโควตาข้อความของ OA เดือนนี้หมด",
  LINE_SEND_FAILED: "LINE ไม่รับข้อความนี้ ตรวจว่าลูกค้ายังเป็นเพื่อนกับ OA และ token ของช่องทางถูกต้อง",
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

export type Inquiry = { _id: string; line_user_id?: string; status: string; created_at: string; updated_at: string; draft_version?: number; order_id?: string; customer?: Customer };
export type Customer = { display_name?: string | null; picture_url?: string | null };
export type LineEvent = { _id: string; event_at: string; event_id?: string; status: string; sender_role?: "customer" | "admin"; direction?: "inbound" | "outbound"; sender_name?: string; source?: string; delivery_status?: string; withdrawn?: boolean; media_id?: string; media_status?: string; payload?: { type: string; message?: { type: string; text?: string; stickerId?: string; packageId?: string; keywords?: string[]; preview_url?: string; original_url?: string }; postback?: { data: string } } };
// delivery_provider holds one of RECEIVING_METHODS when fulfillment is delivery.
export type Rental = { pickup_date: string | null; return_date: string | null; fulfillment_method: string | null; delivery_provider?: string | null };
// Alex's own receiving-method choices; the shop ships by Flash unless told otherwise.
export const STORE_PICKUP = "มารับเอง";
export const RECEIVING_METHODS = [STORE_PICKUP, "Line Man", "Grab", "Kerry", "Flash", "ขนส่งอื่น"];
// Alex's own payment choices for rent and deposit; "ยังไม่ชำระ" is the default.
export const UNPAID = "ยังไม่ชำระ";
export const PAYMENT_METHODS = [UNPAID, "เงินสด", "โอนเงิน", "Alipay/wechat", "บัตรเครดิต", "ช่องทางอื่น"];
const carriers: Record<string, string> = { flash: "Flash", flashexpress: "Flash", "แฟลช": "Flash", kerry: "Kerry", kerryexpress: "Kerry", "เคอรี่": "Kerry", grab: "Grab", grabexpress: "Grab", "แกร็บ": "Grab", lineman: "Line Man", "ไลน์แมน": "Line Man", "ขนส่งอื่น": "ขนส่งอื่น" };
export const receivingMethod = (carrier: string): string | undefined => carriers[carrier.replace(/\s/g, "").toLowerCase()];
export type Amounts = { rental_fee_satang: number | null; deposit_satang: number | null; shipping_fee_satang?: number | null; rental_discount_satang?: number | null; deposit_discount_satang?: number | null };
// rental/amounts/notes per item are retired; older drafts may still return them and the editor lifts them to the order.
export type DraftItem = { product_code: string | null; product_name?: string | null; category?: string | null; color: string | null; size: string | null; quantity: number | null; rental?: Partial<Rental> | null; amounts?: Partial<Amounts> | null; notes?: string | null };
export type Candidate = { product_item_id: string; code: string; name: string; color: string | null; size: string | null; rental_fee_satang: number | null; deposit_satang: number | null; image_url: string | null; score: number | null };
// An Alex code list entry: category "Haatiya" (HT) or color "ชมพู" (P). Search filters use the code.
export type CodeName = { code: string; name: string };
export const findCode = (written: string | null | undefined, list: CodeName[]) => { const wanted = written?.trim().toLowerCase(); return wanted ? list.find(value => value.name.toLowerCase() === wanted || value.code.toLowerCase() === wanted)?.code ?? "" : ""; };
export const codeLabel = (code: string | null | undefined, list: CodeName[]) => { if (!code) return "ไม่ระบุ"; const found = list.find(value => value.code === code); return found ? `${found.name} (${code})` : code; };
export type CartItem = { _id: string; product_item_id: string; branch_id: string; candidate: Candidate; quantity: number; revision: number; added_by: string; created_at: string };
export type Availability = { state: string; checked_at: string; items: { item_index: number; product_item_id: string; code: string; quantity: number; state: string }[] };
export type Fields = { intent: string | null; customer_name: string | null; customer_phone: string | null; customer_address?: string | null; branch: string | null; items: DraftItem[]; rental: Rental; amounts: Amounts; notes: string | null };
export type Evidence = { field: string; source_type: string; source_ref: string; quote: string };
export type Draft = { _id: string; draft_version: number; workflow_status: string; admin_summary?: string; stale: boolean; redacted: boolean; missing_fields: string[]; confirmed_fields?: string[]; catalog_lookup?: { item_index: number; status: string; search_id?: string; selected_product_item_id?: string }[]; created_at: string; result?: { fields: Fields; evidence: Evidence[]; conflicts: string[]; questions: string[]; date_options?: DateOption[] } };
// Pickup/return pairs the AI read from images; the admin picks one for the order.
export type DateOption = { pickup_date: string | null; return_date: string | null; label: string };
export type Job = { id: string; status: string; error_code: string | null };
export type Profile = { _id: string; line_user_id: string; version: number; display_name: string | null; phone: string | null; address: string | null; notes: string | null; updated_by: string; updated_at: string };
export type Order = { _id: string; inquiry_id: string; draft_id: string; draft_version: number; status: string; approved_by: string; approved_at: string; fields: Fields };
export type AdminSummary = { revision: number; text?: string; image_event_ids?: string[]; updated_by: string; updated_at: string; stale: boolean; redacted: boolean };
export type Attachment = { id: string; name: string; media_id: string; url: string; created_at: string };
export type DraftList = { version: number; drafts: Draft[]; latest_request?: Job | null; context_hash:string; summary: AdminSummary | null; attachments: Attachment[] };
export const emptyRental = (): Rental => ({ pickup_date: null, return_date: null, fulfillment_method: null, delivery_provider: null });
export const emptyAmounts = (): Amounts => ({ rental_fee_satang: null, deposit_satang: null, shipping_fee_satang: null, rental_discount_satang: null, deposit_discount_satang: null });
export const emptyFields = (): Fields => ({ intent: null, customer_name: null, customer_phone: null, customer_address: null, branch: null, items: [], rental: emptyRental(), amounts: emptyAmounts(), notes: null });
export const labels: Record<string, string> = { intent: "ความต้องการ", customer_name: "ชื่อลูกค้า", customer_phone: "เบอร์โทร", customer_address: "ที่อยู่ลูกค้า", branch: "สาขา", items: "รายการสินค้า", product_code: "รหัสสินค้า", product_name: "ชื่อชุด", category: "ประเภท", color: "สี", size: "ไซซ์", quantity: "จำนวน", "rental.pickup_date": "วันรับชุด", "rental.return_date": "วันคืนชุด", "rental.delivery_provider": "วิธีมารับ", "rental.fulfillment_method": "วิธีมารับ", "amounts.rental_fee_satang": "ค่าเช่าที่แจ้ง", "amounts.deposit_satang": "ค่าประกันที่แจ้ง", "amounts.shipping_fee_satang": "ค่าจัดส่ง", "amounts.rental_discount_satang": "ส่วนลดค่าเช่า", "amounts.deposit_discount_satang": "ส่วนลดค่าประกัน", notes: "หมายเหตุ" };
export function fieldLabel(key: string) { const match = key.match(/^items\.(\d+)\.(.+)$/); return match ? `สินค้ารายการ ${Number(match[1]) + 1} · ${labels[match[2]] ?? match[2]}` : labels[key] ?? key; }
// Alex books only pickup and return dates; older drafts may still carry these retired keys.
const retiredKeys = ["usage_date", "dispatch_date", "return_shipping_deadline"];
const retiredItemKeys = ["rental", "amounts", "notes"];
// Phone numbers are stored without spaces or dashes, matching the backend rule.
export const cleanPhone = (value: string) => value.replace(/[\s-]/g, "");
export function filledKeys(fields: Fields) {
  const keys: string[] = [];
  const walk = (value: unknown, prefix: string) => {
    const parts = prefix.split(".");
    if (value === null || value === undefined || value === "" || retiredKeys.includes(parts[parts.length - 1]) || (parts[0] === "items" && retiredItemKeys.includes(parts[2]))) return;
    if (typeof value === "object") Object.entries(value).forEach(([key, child]) => walk(child, prefix ? `${prefix}.${key}` : key));
    else keys.push(prefix);
  };
  walk(fields, ""); return keys;
}

import type { Customer } from "./api";
export type Parcel = { row: number; recipient_name: string | null; tracking_number: string | null; postal_code: string | null; service: string | null; shipping_fee_satang: number | null; raw_text: string };
export type ShippingOrder = { submission_id: string; inquiry_id: string; line_user_id: string; customer: { name: string; phone: string; address: string }; chat_customer: Customer; alex_order_id: string; alex_order_number: string | null; draft_version: number | null; pickup_date: string | null; created_at: string; match_score: number };
export type Tracking = { chat_customer?: Customer; _id: string; row: number; version: number; status: "READY" | "SENDING" | "SEND_FAILED" | "SENT"; submission_id: string | null; inquiry_id: string; customer: ShippingOrder["customer"]; alex_order_id: string | null; alex_order_number: string | null; recipient_name: string; tracking_number: string; postal_code: string; service: string; shipping_fee_satang: number | null; message: string; delivery_started_at?: string; sent_at?: string; sent_by?: string; updated_at: string; updated_by?: string; error_code?: string };
export type Receipt = { _id: string; name: string; status: "QUEUED" | "READING" | "READY" | "FAILED"; created_at: string; created_by: string; error_code?: string; total_mismatch?: boolean; image_url?: string; extraction?: { carrier: string; receipt_number: string | null; receipt_date: string | null; total_satang: number | null; parcels: Parcel[] } };
export type ReceiptResult = { receipt: Receipt; trackings: Tracking[] };
export type ShippingCustomerMatch = { line_user_id: string; inquiry_id: string; chat_customer: Customer; matched_name: string; match_source: "alex_submission" | "customer_profile" | "chat_alias" | "line_profile" | "oa_name" | "customer_note" | "profile_note"; match_score: number; submission_status: string | null };
export type ShippingSearchResult = { orders: ShippingOrder[]; more: boolean; customer_matches?: ShippingCustomerMatch[]; customers_more?: boolean };
export function suggestedChatOrder(orders: ShippingOrder[], more: boolean, customers: ShippingCustomerMatch[] = []): ShippingOrder | undefined {
  if (more || !orders.length) return;
  const score = Math.max(...orders.map(order => order.match_score));
  if (score < 85 || score > 100) return;
  const matches = orders.filter(order => order.match_score === score);
  if (new Set(matches.map(order => order.line_user_id)).size !== 1) return;
  if (customers.some(customer => customer.match_score >= score && customer.line_user_id !== matches[0].line_user_id)) return;
  return matches.find(order => !!order.chat_customer.display_name);
}
export const normalizeTracking = (value: string) => value.replace(/[\s-]/g, "").toUpperCase();
export const validTracking = (value: string) => /^[A-Z]{2}\d{9}TH$/.test(normalizeTracking(value));
export const trackingMessage = (number: string) => `ส่งของเรียบร้อยค่ะ ✨ (ems)\nเลขที่ Tracking: ${normalizeTracking(number)}\n\nสามารถเช็คสถานะได้ที่ https://track.thailandpost.co.th`;
export function feeSatang(value: string): number | null {
  if (!value.trim()) return null;
  if (!/^\d{1,7}(?:\.\d{1,2})?$/.test(value.trim())) return NaN;
  const [whole, fraction = ""] = value.trim().split(".");
  return Number(whole) * 100 + Number(fraction.padEnd(2, "0"));
}
export async function prepareReceipt(file: File): Promise<{ name: string; image_base64: string }> {
  if (!/^image\/(jpeg|png|webp|heic|heif)$/.test(file.type) || file.size > 40 * 1024 * 1024) throw new Error("รองรับรูป JPG, PNG หรือ WEBP ขนาดไม่เกิน 40 MB");
  const bitmap = await createImageBitmap(file);
  try {
    const scale = Math.min(1, 6000 / Math.max(bitmap.width, bitmap.height));
    const canvas = document.createElement("canvas"); canvas.width = Math.max(1, Math.round(bitmap.width * scale)); canvas.height = Math.max(1, Math.round(bitmap.height * scale));
    const context = canvas.getContext("2d"); if (!context) throw new Error("เตรียมรูปไม่สำเร็จ");
    context.fillStyle = "white"; context.fillRect(0, 0, canvas.width, canvas.height); context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
    let blob: Blob | null = null;
    for (const quality of [.88, .75, .6]) { blob = await new Promise(resolve => canvas.toBlob(resolve, "image/jpeg", quality)); if (blob && blob.size <= 8 * 1024 * 1024) break; }
    if (!blob || blob.size > 8 * 1024 * 1024) throw new Error("รูปใบเสร็จใหญ่เกินไป กรุณาแบ่งถ่ายเป็นหลายรูป");
    const image_base64 = await new Promise<string>((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result).split(",", 2)[1]); reader.onerror = () => reject(new Error("อ่านรูปไม่สำเร็จ")); reader.readAsDataURL(blob); });
    return { name: file.name.slice(0, 190) || "ใบเสร็จขนส่ง.jpg", image_base64 };
  } finally { bitmap.close(); }
}

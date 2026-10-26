"use client";
/* eslint-disable @next/next/no-img-element -- Private attachments use same-origin session cookies. */
import { useRef, useState } from "react";
import { api, ApiError, Attachment, errorText } from "@/lib/api";

async function encode(file: File): Promise<string> {
  if (!/^image\/(jpeg|png|webp|gif)$/.test(file.type) || file.size > 40 * 1024 * 1024) throw new Error("unsupported");
  const bitmap = await createImageBitmap(file);
  try {
    const scale = Math.min(1, 2000 / Math.max(bitmap.width, bitmap.height));
    const canvas = document.createElement("canvas"); canvas.width = Math.round(bitmap.width * scale); canvas.height = Math.round(bitmap.height * scale);
    const context = canvas.getContext("2d"); if (!context) throw new Error("canvas");
    context.fillStyle = "#ffffff"; context.fillRect(0, 0, canvas.width, canvas.height); context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
    let blob: Blob | null = null;
    for (const quality of [0.9, 0.8, 0.65, 0.5]) {
      blob = await new Promise<Blob | null>(resolve => canvas.toBlob(resolve, "image/jpeg", quality));
      if (blob && blob.size <= 1400 * 1024) break;
    }
    if (!blob || blob.size > 1400 * 1024) throw new Error("large");
    return await new Promise((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result).split(",", 2)[1]); reader.onerror = reject; reader.readAsDataURL(blob!); });
  } finally { bitmap.close(); }
}

export default function SummaryImages({ inquiryId, attachments, selected, disabled, onUploaded, onToggle, onBusy, onUnauthorized }: { inquiryId: string; attachments: Attachment[]; selected: string[]; disabled: boolean; onUploaded: (attachment: Attachment, contextHash: string) => void; onToggle: (id: string, checked: boolean) => void; onBusy: (busy: boolean) => void; onUnauthorized: () => void }) {
  const fileRef = useRef<HTMLInputElement>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  async function upload(files: FileList | null) {
    const pending = files ? Array.from(files) : [];
    if (fileRef.current) fileRef.current.value = "";
    if (!pending.length) return;
    const remaining = 5 - selected.length;
    if (pending.length > remaining) { setError(`แนบเพิ่มได้อีก ${remaining} รูป รวมรูปจากแชทสูงสุด 5 รูป`); return; }
    setBusy(true); onBusy(true); setError("");
    try {
      for (const file of pending) {
        const image_base64 = await encode(file);
        const value = await api<{ attachment: Attachment; context_hash: string }>(`/api/inquiries/${inquiryId}/draft-images`, { method: "POST", body: JSON.stringify({ name: file.name.slice(0, 200), image_base64 }) });
        onUploaded(value.attachment, value.context_hash);
      }
    } catch (cause) { setError(cause instanceof ApiError ? errorText(cause) : "แนบรูปไม่สำเร็จ รองรับ JPG, PNG, WEBP หรือ GIF ขนาดไม่เกิน 40 MB ต่อรูป"); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); }
    finally { setBusy(false); onBusy(false); }
  }
  return <div className="summary-images">
    <div className="section-caption"><strong>รูปสรุปสำหรับ AI ({selected.length}/5)</strong><button type="button" className="button secondary" disabled={disabled || busy || selected.length >= 5} onClick={() => fileRef.current?.click()}>{busy ? "กำลังแนบรูป…" : "+ แนบรูป"}</button></div>
    <input className="sr-only" ref={fileRef} type="file" accept="image/jpeg,image/png,image/webp,image/gif" multiple aria-label="แนบรูปสรุปให้ AI สูงสุด 5 รูป" onChange={e => upload(e.target.files)} disabled={disabled || busy || selected.length >= 5} />
    {error && <div className="notice warning" role="alert">{error}</div>}
    {attachments.length > 0 && <div className="summary-image-grid">{attachments.map(attachment => <label key={attachment.id} className={selected.includes(attachment.id) ? "selected" : ""}><img src={attachment.url} alt={attachment.name || "รูปสรุปที่แนบ"} /><span><input type="checkbox" checked={selected.includes(attachment.id)} disabled={disabled || busy || (!selected.includes(attachment.id) && selected.length >= 5)} onChange={e => onToggle(attachment.id, e.target.checked)} />ใช้รูปนี้</span></label>)}</div>}
    <p className="muted small">แนบภาพที่มีชื่อชุด สี ไซซ์ และวันที่ได้ หรือเลือกรูปจากแชท รวมสูงสุด 5 รูป รูปที่แนบส่วนนี้ใช้เตรียมร่างภายใน</p>
  </div>;
}

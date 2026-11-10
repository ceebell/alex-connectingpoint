"use client";
/* eslint-disable @next/next/no-img-element -- Sticker CDN and uploaded previews are plain images. */
import { useEffect, useRef, useState } from "react";
import { api, ApiError, errorText, LineEvent } from "@/lib/api";
import { stickerIds, stickerPackages, stickerUrl } from "@/lib/stickers";

type Body = { text: string } | { sticker: { package_id: string; sticker_id: string } } | { image_id: string };
type Failed = { requestId: string; body: Body; label: string };
type Upload = { id: string; preview: string; requestId: string };
type Props = { inquiryId: string; canReply: boolean; onSent: (message: LineEvent) => void; onUnauthorized: () => void };

// LINE limits: original JPEG/PNG up to 10 MB, preview up to 1 MB.
const ORIGINAL_MAX = 10 * 1024 * 1024;
async function encode(bitmap: ImageBitmap, maxEdge: number, quality: number): Promise<Blob> {
  const scale = Math.min(1, maxEdge / Math.max(bitmap.width, bitmap.height));
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(bitmap.width * scale));
  canvas.height = Math.max(1, Math.round(bitmap.height * scale));
  const context = canvas.getContext("2d");
  if (!context) throw new Error("canvas");
  context.fillStyle = "#ffffff"; // JPEG has no transparency.
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  return new Promise((resolve, reject) => canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error("encode")), "image/jpeg", quality));
}
const base64 = (blob: Blob) => new Promise<string>((resolve, reject) => {
  const reader = new FileReader();
  reader.onload = () => resolve(String(reader.result).split(",", 2)[1] ?? "");
  reader.onerror = () => reject(reader.error);
  reader.readAsDataURL(blob);
});

export default function ReplyComposer({ inquiryId, canReply, onSent, onUnauthorized }: Props) {
  const [text, setText] = useState("");
  const [textId, setTextId] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [failed, setFailed] = useState<Failed | null>(null);
  const [stickersOpen, setStickersOpen] = useState(false);
  const [toolsOpen, setToolsOpen] = useState(false);
  const [imageOpen, setImageOpen] = useState(false);
  const [focused, setFocused] = useState(false);
  const [pack, setPack] = useState(stickerPackages[0]);
  const [image, setImage] = useState<Upload | null>(null);
  const [preparing, setPreparing] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const formRef = useRef<HTMLFormElement>(null);
  const textRef = useRef<HTMLTextAreaElement>(null);
  const pointerInside = useRef(false);
  const working = busy || preparing;
  const attachmentsOpen = toolsOpen || stickersOpen || imageOpen;
  const active = focused || attachmentsOpen;

  // Dismiss only the presentation: unsent text and a prepared photo remain available.
  function collapse() {
    textRef.current?.blur();
    setFocused(false); setToolsOpen(false); setStickersOpen(false); setImageOpen(false);
  }
  useEffect(() => {
    if (!active) return;
    const outside = (event: PointerEvent) => {
      if (event.target instanceof Node && !formRef.current?.contains(event.target)) collapse();
    };
    const escape = (event: KeyboardEvent) => { if (event.key === "Escape") collapse(); };
    document.addEventListener("pointerdown", outside);
    document.addEventListener("keydown", escape);
    return () => { document.removeEventListener("pointerdown", outside); document.removeEventListener("keydown", escape); };
  }, [active]);

  function toggleTools() {
    const open = !attachmentsOpen;
    collapse();
    setToolsOpen(open);
  }
  function chooseImage() {
    collapse();
    if (image) setImageOpen(true);
    else fileRef.current?.click();
  }

  // The request ID doubles as LINE's retry key, so a retry never sends twice.
  async function deliver(requestId: string, body: Body, label: string) {
    setBusy(true); setError(""); setFailed(null);
    try {
      const result = await api<{ message: LineEvent }>(`/api/inquiries/${inquiryId}/messages`, { method: "POST", body: JSON.stringify({ request_id: requestId, ...body }) });
      onSent(result.message);
      return true;
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
      else { setError(errorText(cause)); if (!("text" in body)) setFailed({ requestId, body, label }); }
      return false;
    } finally { setBusy(false); }
  }

  async function sendText() {
    if (!text.trim() || working) return;
    const requestId = textId ?? crypto.randomUUID();
    setTextId(requestId);
    if (await deliver(requestId, { text }, "ข้อความ")) { setText(""); setTextId(null); }
  }

  async function sendSticker(stickerId: string) {
    if (working) return;
    setStickersOpen(false);
    setToolsOpen(false);
    await deliver(crypto.randomUUID(), { sticker: { package_id: pack.id, sticker_id: stickerId } }, "สติกเกอร์");
  }

  async function sendImage() {
    if (!image || working) return;
    if (await deliver(image.requestId, { image_id: image.id }, "รูป")) { setImage(null); setImageOpen(false); }
  }

  async function pickImage(file: File | undefined) {
    if (fileRef.current) fileRef.current.value = "";
    if (!file) return;
    if (!/^image\/(jpeg|png|webp|gif)$/.test(file.type) || file.size > 40 * 1024 * 1024) { setError("รองรับไฟล์ JPG, PNG, WEBP หรือ GIF ขนาดไม่เกิน 40 MB"); return; }
    setPreparing(true); setImageOpen(true); setError(""); setFailed(null); setStickersOpen(false); setToolsOpen(false);
    try {
      const bitmap = await createImageBitmap(file);
      let original = await encode(bitmap, 2048, 0.85);
      if (original.size > ORIGINAL_MAX) original = await encode(bitmap, 1600, 0.7);
      const preview = await encode(bitmap, 240, 0.8);
      bitmap.close();
      if (original.size > ORIGINAL_MAX) throw new Error("too large");
      const result = await api<{ media: { _id: string; preview_url: string } }>(`/api/inquiries/${inquiryId}/media`, { method: "POST", body: JSON.stringify({ original: await base64(original), preview: await base64(preview) }) });
      setImage({ id: result.media._id, preview: result.media.preview_url, requestId: crypto.randomUUID() });
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
      else setError(cause instanceof ApiError ? errorText(cause) : "เตรียมรูปไม่สำเร็จ กรุณาเลือกรูปอื่น");
      setImageOpen(false);
    } finally { setPreparing(false); }
  }

  return <form ref={formRef} className="reply-box" data-active={active} autoComplete="off" onPointerDownCapture={() => { pointerInside.current = true; }} onPointerUpCapture={() => { window.requestAnimationFrame(() => { pointerInside.current = false; }); }} onPointerCancelCapture={() => { pointerInside.current = false; }} onBlur={event => { if (!pointerInside.current && (!(event.relatedTarget instanceof Node) || !event.currentTarget.contains(event.relatedTarget))) setFocused(false); }} onSubmit={e => { e.preventDefault(); void sendText(); }}>
    {error && <div className="notice warning" role="alert">{error}{failed && <button type="button" className="text-button retry" disabled={working} onClick={() => void deliver(failed.requestId, failed.body, failed.label)}>ส่ง{failed.label}อีกครั้ง</button>}</div>}
    {attachmentsOpen && <div className="reply-attachments" id={`reply-attachments-${inquiryId}`}>
    {imageOpen && preparing && <p className="muted small" role="status">กำลังเตรียมรูป…</p>}
    {imageOpen && !preparing && image && <div className="image-draft"><img src={image.preview} alt="ตัวอย่างรูปที่จะส่ง" /><div><strong>รูปพร้อมส่ง</strong><p className="muted small">ตรวจรูปก่อนส่ง ลูกค้าจะเห็นทันที</p></div><button type="button" className="text-button" disabled={working} onClick={() => { collapse(); fileRef.current?.click(); }}>เปลี่ยนรูป</button><button type="button" className="text-button" disabled={working} onClick={() => { setImage(null); setImageOpen(false); }}>ยกเลิก</button><button type="button" className="button primary" disabled={working} onClick={() => void sendImage()}>ส่งรูป</button></div>}
    {stickersOpen && <div className="sticker-panel" role="group" aria-label="เลือกสติกเกอร์">
      <div className="sticker-tabs" role="tablist" aria-label="ชุดสติกเกอร์">{stickerPackages.map(item => <button key={item.id} type="button" role="tab" aria-selected={item.id === pack.id} title={item.title} aria-label={item.title} onClick={() => setPack(item)}><img src={stickerUrl(item.first)} alt="" loading="lazy" referrerPolicy="no-referrer" /></button>)}</div>
      <p className="muted small sticker-title">{pack.title} · กดเพื่อส่งทันที</p>
      <div className="sticker-grid">{stickerIds(pack).map(id => <button key={id} type="button" disabled={working} aria-label={`ส่งสติกเกอร์ ${id}`} onClick={() => void sendSticker(id)}><img src={stickerUrl(id)} alt="" loading="lazy" referrerPolicy="no-referrer" /></button>)}</div>
    </div>}
    {toolsOpen && <div className="reply-tools" role="group" aria-label="เครื่องมือแนบข้อความ">
        <button type="button" className="tool-button" aria-label="สติกเกอร์" disabled={!canReply || working} onClick={() => { collapse(); setStickersOpen(true); }}><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M8 14s1.5 3 4 3 4-3 4-3M8 9h.01M16 9h.01" /></svg><span>สติกเกอร์</span></button>
        <button type="button" className="tool-button" aria-label="แนบรูป" disabled={!canReply || working} onClick={chooseImage}><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="3" /><circle cx="8" cy="8" r="1.5" /><path d="m21 15-5-5L5 21" /></svg><span>{image ? "รูปที่เลือก" : "รูปภาพ"}</span></button>
    </div>}
    </div>}
    <input ref={fileRef} type="file" accept="image/jpeg,image/png,image/webp,image/gif" hidden onChange={e => void pickImage(e.target.files?.[0])} />
    <div className="reply-row">
      <button type="button" className="reply-add" aria-label={attachmentsOpen ? "ปิดเครื่องมือแนบข้อความ" : "เปิดเครื่องมือแนบข้อความ"} aria-expanded={attachmentsOpen} aria-controls={`reply-attachments-${inquiryId}`} disabled={!canReply || working} onClick={toggleTools}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 4v16M4 12h16" /></svg></button>
      <label className="sr-only" htmlFor="reply-text">ข้อความตอบลูกค้า</label>
      <textarea ref={textRef} id="reply-text" name="chat-message" autoComplete="off" inputMode="text" rows={1} maxLength={5000} value={text} disabled={busy || !canReply} placeholder={canReply ? "พิมพ์ตอบลูกค้า…" : "เรื่องนี้ไม่มี LINE user ID จึงตอบจาก Alex ไม่ได้"} onFocus={() => { setFocused(true); setToolsOpen(false); setStickersOpen(false); setImageOpen(false); }} onChange={e => { setText(e.target.value); setTextId(null); setError(""); setFailed(null); }} onKeyDown={e => { if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); void sendText(); } }} />
      <button className="button primary reply-send" aria-label={busy ? "กำลังส่งข้อความ" : textId ? "ส่งอีกครั้ง" : "ส่งข้อความ"} disabled={working || !text.trim() || !canReply}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 7-7 7 7M12 5v15" /></svg></button>
    </div>
  </form>;
}

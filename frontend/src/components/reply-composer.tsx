"use client";
/* eslint-disable @next/next/no-img-element -- Sticker CDN and uploaded previews are plain images. */
import { useEffect, useRef, useState } from "react";
import { api, ApiError, errorText, LineEvent } from "@/lib/api";
import { stickerIds, stickerPackages, stickerUrl } from "@/lib/stickers";
import { matchingQuickReplies, slashQuery, type QuickReply } from "@/lib/quick-replies";
import { prepareReplyImage } from "@/lib/reply-images";
import QuickRepliesDialog from "./quick-replies-dialog";

type Body = { text: string } | { sticker: { package_id: string; sticker_id: string } } | { image_id: string };
type Failed = { requestId: string; body: Body; label: string };
type Upload = { id: string; preview: string; requestId: string };
type Props = { inquiryId: string; canReply: boolean; onSent: (message: LineEvent) => void; onUnauthorized: () => void };

// Same breakpoint as mobile.css, plus touch-first devices: there Enter inserts a new line
// (on-screen keyboards have no Shift+Enter) and the send button submits.
const COMPACT_INPUT = "(max-width: 767px), (max-width: 1024px) and (max-height: 600px), (pointer: coarse)";
const sendOnEnter = () => !window.matchMedia(COMPACT_INPUT).matches;

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
  const [quickItems, setQuickItems] = useState<QuickReply[]>([]);
  const [quickLoading, setQuickLoading] = useState(false);
  const [quickError, setQuickError] = useState("");
  const [quickLibrary, setQuickLibrary] = useState(false);
  const [quickDismissed, setQuickDismissed] = useState(false);
  const [quickActive, setQuickActive] = useState(0);
  const fileRef = useRef<HTMLInputElement>(null);
  const formRef = useRef<HTMLFormElement>(null);
  const textRef = useRef<HTMLTextAreaElement>(null);
  const pointerInside = useRef(false);
  const working = busy || preparing;
  const attachmentsOpen = toolsOpen || stickersOpen || imageOpen;
  const active = focused || attachmentsOpen;
  const query = slashQuery(text);
  const quickMatches = matchingQuickReplies(quickItems, query ?? "", "text").slice(0, 8);
  const quickVisible = focused && query !== null && !quickDismissed && !working && !quickLibrary;
  const quickIndex = Math.max(0, Math.min(quickActive, quickMatches.length - 1));
  const quickListId = `quick-replies-${inquiryId}`;

  useEffect(() => {
    if (quickVisible) document.getElementById(`${quickListId}-${quickIndex}`)?.scrollIntoView({ block: "nearest" });
  }, [quickVisible, quickIndex, quickListId, query]);

  async function loadQuickReplies() {
    if (quickLoading) return;
    setQuickLoading(true); setQuickError("");
    try {
      const items = new Map<string, QuickReply>(); let before: string | null = null;
      do {
        const page: { items: QuickReply[]; next: string | null } = await api(`/api/admin/quick-replies${before ? `?before=${encodeURIComponent(before)}` : ""}`);
        for (const item of page.items) items.set(item._id, item);
        if (page.next === before) break;
        before = page.next;
      } while (before);
      setQuickItems([...items.values()]);
    } catch (cause) { setQuickError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); }
    finally { setQuickLoading(false); }
  }
  function openQuickLibrary() { collapse(); setQuickLibrary(true); void loadQuickReplies(); }
  function chooseQuickText(item: QuickReply) {
    setText(item.text); setTextId(null); setError(""); setFailed(null); setQuickLibrary(false); setQuickDismissed(true); setQuickActive(0);
    setToolsOpen(false); setStickersOpen(false); setImageOpen(false); setFocused(true);
    requestAnimationFrame(() => textRef.current?.focus({ preventScroll: true }));
  }
  async function chooseQuickReply(item: QuickReply) {
    if (working) return;
    if (item.kind === "text") { chooseQuickText(item); return; }
    collapse(); setQuickLibrary(false); setPreparing(true); setImageOpen(true); setError(""); setFailed(null);
    try {
      const result = await api<{ media: { _id: string; preview_url: string } }>(`/api/inquiries/${inquiryId}/quick-replies/${item._id}/media`, { method: "POST", body: JSON.stringify({ expected_version: item.version }) });
      setImage({ id: result.media._id, preview: result.media.preview_url, requestId: crypto.randomUUID() });
    } catch (cause) {
      setImageOpen(false); setError(cause instanceof ApiError && cause.code === "RECORD_CONFLICT" ? "การ์ดนี้ถูกแก้แล้ว กรุณาเปิดคลังแล้วเลือกรูปล่าสุด" : errorText(cause));
      if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
    } finally { setPreparing(false); }
  }

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
    if (quickVisible) { if (quickMatches[quickIndex]) chooseQuickText(quickMatches[quickIndex]); return; }
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
      const prepared = await prepareReplyImage(file);
      const result = await api<{ media: { _id: string; preview_url: string } }>(`/api/inquiries/${inquiryId}/media`, { method: "POST", body: JSON.stringify(prepared) });
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
        <button type="button" className="tool-button" aria-label="คลังข้อความตอบกลับ" disabled={!canReply || working} onClick={openQuickLibrary}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 3h16v15H9l-5 4Z" /><path d="M8 7h8M8 11h6" /></svg><span>ข้อความด่วน / การ์ด</span></button>
    </div>}
    </div>}
    <input ref={fileRef} type="file" accept="image/jpeg,image/png,image/webp,image/gif" hidden onChange={e => void pickImage(e.target.files?.[0])} />
    <div className="reply-row">
      <button type="button" className="reply-add" aria-label={attachmentsOpen ? "ปิดเครื่องมือแนบข้อความ" : "เปิดเครื่องมือแนบข้อความ"} aria-expanded={attachmentsOpen} aria-controls={`reply-attachments-${inquiryId}`} disabled={!canReply || working} onClick={toggleTools}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 4v16M4 12h16" /></svg></button>
      <label className="sr-only" htmlFor="reply-text">ข้อความตอบลูกค้า</label>
      <textarea ref={textRef} id="reply-text" name="chat-message" role="combobox" aria-autocomplete="list" aria-expanded={quickVisible} aria-controls={quickVisible ? quickListId : undefined} aria-activedescendant={quickVisible && quickMatches.length ? `${quickListId}-${quickIndex}` : undefined} autoComplete="off" inputMode="text" rows={1} maxLength={5000} value={text} disabled={busy || !canReply} placeholder={canReply ? "พิมพ์ตอบลูกค้า… หรือ / ข้อความด่วน" : "เรื่องนี้ไม่มี LINE user ID จึงตอบจาก Alex ไม่ได้"} onFocus={() => { setFocused(true); setToolsOpen(false); setStickersOpen(false); setImageOpen(false); if (query !== null) void loadQuickReplies(); }} onChange={e => { if (e.target.value.startsWith("/") && !text.startsWith("/")) void loadQuickReplies(); setText(e.target.value); setTextId(null); setQuickDismissed(false); setQuickActive(0); setError(""); setFailed(null); }} onKeyDown={e => {
        if (e.nativeEvent.isComposing) return;
        if (quickVisible && ["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(e.key)) {
          e.preventDefault(); e.stopPropagation();
          if (e.key === "Escape") setQuickDismissed(true);
          else if (e.key === "Enter") { if (quickMatches[quickIndex]) chooseQuickText(quickMatches[quickIndex]); }
          else setQuickActive(e.key === "ArrowDown" ? (quickIndex + 1) % Math.max(1, quickMatches.length) : (quickIndex - 1 + quickMatches.length) % Math.max(1, quickMatches.length));
          return;
        }
        if (e.key !== "Enter" || e.shiftKey || e.altKey) return;
        if (e.ctrlKey || e.metaKey || sendOnEnter()) { e.preventDefault(); void sendText(); }
      }} />
      <button className="button primary reply-send" aria-label={busy ? "กำลังส่งข้อความ" : textId ? "ส่งอีกครั้ง" : "ส่งข้อความ"} disabled={working || !text.trim() || !canReply}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 7-7 7 7M12 5v15" /></svg></button>
    </div>
    {quickVisible && <div className="reply-quick-suggestions" data-editor-popup>
      <div className="reply-quick-heading"><strong>ข้อความด่วน</strong><button type="button" className="text-button" onPointerDown={event => event.preventDefault()} onClick={openQuickLibrary}>จัดการข้อความ</button></div>
      {quickLoading && <p role="status">กำลังโหลดข้อความด่วน…</p>}
      {quickError && <p role="alert">{quickError}<button type="button" className="text-button" onPointerDown={event => event.preventDefault()} onClick={() => void loadQuickReplies()}>ลองใหม่</button></p>}
      <div id={quickListId} role="listbox" aria-label="ข้อความด่วนที่ตรงคำค้น">{quickMatches.map((item, index) => <button key={item._id} id={`${quickListId}-${index}`} type="button" role="option" aria-selected={index === quickIndex} className={`reply-quick-option ${index === quickIndex ? "is-active" : ""}`} onPointerDown={event => event.preventDefault()} onClick={() => chooseQuickText(item)}><strong>/{item.title}</strong><span>{item.text}</span></button>)}</div>
      {!quickLoading && !quickError && !quickMatches.length && <p role="status">ไม่พบข้อความด่วนที่ตรงคำค้น กดจัดการข้อความเพื่อเพิ่มรายการ</p>}
      <p>เลือกเพื่อเติมข้อความก่อนส่ง · Enter เลือก · Esc ปิด</p>
    </div>}
    {quickLibrary && <QuickRepliesDialog items={quickItems} loading={quickLoading} loadError={quickError} initialText={text} onReload={loadQuickReplies} onSaved={item => { setQuickItems(previous => [item, ...previous.filter(row => row._id !== item._id)]); setQuickError(""); }} onDeleted={id => setQuickItems(previous => previous.filter(item => item._id !== id))} onSelect={item => void chooseQuickReply(item)} onClose={() => setQuickLibrary(false)} onUnauthorized={onUnauthorized} />}
  </form>;
}

"use client";
/* eslint-disable @next/next/no-img-element -- Reusable cards use protected image URLs or local previews. */
import { useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { api, ApiError, errorText } from "@/lib/api";
import { formatDateTime } from "@/lib/dates";
import { matchingQuickReplies, type QuickReply } from "@/lib/quick-replies";
import { prepareReplyImage, type ReplyImageInput } from "@/lib/reply-images";

type Props = { items: QuickReply[]; loading: boolean; loadError: string; initialText: string; onReload: () => Promise<void>; onSaved: (item: QuickReply) => void; onDeleted: (id: string) => void; onSelect: (item: QuickReply) => void; onClose: () => void; onUnauthorized: () => void };
type Editor = { id?: string; version?: number; kind: QuickReply["kind"]; title: string; text: string; preview?: string; image?: ReplyImageInput; requestId: string };

export default function QuickRepliesDialog({ items, loading, loadError, initialText, onReload, onSaved, onDeleted, onSelect, onClose, onUnauthorized }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const file = useRef<HTMLInputElement>(null);
  const heading = useId();
  const [tab, setTab] = useState<QuickReply["kind"]>("text");
  const [query, setQuery] = useState("");
  const [editor, setEditor] = useState<Editor | null>(null);
  const [busy, setBusy] = useState(false);
  const [preparing, setPreparing] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [deleting, setDeleting] = useState<string | null>(null);
  const working = busy || preparing;
  const matches = matchingQuickReplies(items, query, tab);
  useEffect(() => {
    const opener = document.activeElement; const element = dialog.current; element?.showModal();
    return () => { element?.close(); if (opener instanceof HTMLElement && opener.isConnected) opener.focus({ preventScroll: true }); };
  }, []);
  function failed(cause: unknown) {
    if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
    if (cause instanceof ApiError && cause.code === "RECORD_CONFLICT") { setError("รายการนี้ถูกแก้จากอีกหน้าต่าง ข้อมูลที่กรอกยังอยู่ กดยกเลิกแล้วเปิดรายการล่าสุดเพื่อตรวจอีกครั้ง"); void onReload(); }
    else setError(errorText(cause));
  }
  function start(item?: QuickReply, currentText = "") {
    setError(""); setNotice(""); setDeleting(null);
    setEditor(item ? { id: item._id, version: item.version, kind: item.kind, title: item.title, text: item.text, preview: item.preview_url ? `${item.preview_url}?v=${item.version}` : undefined, requestId: crypto.randomUUID() }
      : { kind: tab, title: currentText.slice(0, 40).replace(/\s+/g, " "), text: currentText, requestId: crypto.randomUUID() });
  }
  async function attach(selected: File | undefined) {
    if (file.current) file.current.value = ""; if (!selected) return;
    setPreparing(true); setError("");
    try { const image = await prepareReplyImage(selected); setEditor(previous => previous && { ...previous, image, preview: `data:image/jpeg;base64,${image.preview}`, requestId: crypto.randomUUID() }); }
    catch { setError("เตรียมรูปไม่สำเร็จ รองรับ JPG, PNG, WEBP หรือ GIF ขนาดไม่เกิน 40 MB"); }
    finally { setPreparing(false); }
  }
  async function save() {
    if (!editor || working) return;
    setBusy(true); setError(""); setNotice("");
    try {
      const identity = editor.id ? { expected_version: editor.version } : { request_id: editor.requestId };
      const value = await api<{ item: QuickReply }>(`/api/admin/quick-replies${editor.id ? `/${editor.id}` : ""}`, { method: "POST", body: JSON.stringify({ ...identity, title: editor.title, kind: editor.kind, text: editor.kind === "text" ? editor.text : "", image: editor.image }) });
      onSaved(value.item); setEditor(null); setNotice("บันทึกไว้ในคลังของร้านแล้ว");
    } catch (cause) { failed(cause); }
    finally { setBusy(false); }
  }
  async function remove(item: QuickReply) {
    if (working) return; setBusy(true); setError("");
    try { await api(`/api/admin/quick-replies/${item._id}/delete`, { method: "POST", body: JSON.stringify({ expected_version: item.version }) }); onDeleted(item._id); setDeleting(null); setNotice("ลบรายการแล้ว"); }
    catch (cause) { failed(cause); } finally { setBusy(false); }
  }

  return createPortal(<dialog ref={dialog} className="quick-replies-dialog" aria-labelledby={heading} onCancel={event => { event.preventDefault(); if (!working) onClose(); }} data-editor-popup>
    <div className="quick-dialog-heading"><div><h2 id={heading}>คลังข้อความตอบกลับ</h2><p>ข้อความด่วนและการ์ดรูปภาพ ใช้ร่วมกันในร้าน</p></div><button type="button" className="text-button" disabled={working} onClick={onClose}>ปิด</button></div>
    <div className="quick-dialog-body">
      <div className="quick-kind-tabs" role="tablist" aria-label="ประเภทข้อความตอบกลับ">{([['text', 'ข้อความด่วน'], ['image', 'การ์ดรูปภาพ']] as const).map(([kind, label]) => <button key={kind} type="button" role="tab" aria-selected={tab === kind} disabled={working || !!editor} onClick={() => { setTab(kind); setQuery(""); setError(""); setNotice(""); setDeleting(null); }}>{label}</button>)}</div>
      {(error || loadError) && <p className="notice warning" role="alert">{error || loadError}</p>}
      {notice && <p className="quick-feedback" role="status">{notice}</p>}
      {editor ? <form className="quick-reply-editor" autoComplete="off" onSubmit={event => { event.preventDefault(); event.stopPropagation(); void save(); }}>
        <h3>{editor.id ? "แก้ไข" : "เพิ่ม"}{editor.kind === "text" ? "ข้อความด่วน" : "การ์ดรูปภาพ"}</h3>
        <label>ชื่อรายการ<input autoFocus autoComplete="off" value={editor.title} maxLength={80} required disabled={working} placeholder="เช่น สวัสดี หรือ ตารางไซซ์" onChange={event => { setEditor({ ...editor, title: event.target.value, requestId: crypto.randomUUID() }); setError(""); }} /></label>
        {editor.kind === "text" ? <label>ข้อความตอบกลับ<textarea autoComplete="off" rows={5} value={editor.text} maxLength={5000} required disabled={working} placeholder="ข้อความที่จะเติมลงช่องพิมพ์ก่อนส่ง" onChange={event => { setEditor({ ...editor, text: event.target.value, requestId: crypto.randomUUID() }); setError(""); }} /></label> : <div className="quick-image-editor">
          {editor.preview && <img src={editor.preview} alt="ตัวอย่างการ์ดรูปภาพ" />}
          <input ref={file} type="file" accept="image/jpeg,image/png,image/webp,image/gif" hidden onChange={event => void attach(event.target.files?.[0])} />
          <button type="button" className="button secondary" disabled={working} onClick={() => file.current?.click()}>{preparing ? "กำลังเตรียมรูป…" : editor.preview ? "เปลี่ยนรูป" : "เลือกรูปภาพ"}</button>
          <p>การ์ดนี้ส่งเป็นรูปภาพหนึ่งรูป</p>
        </div>}
        <div className="quick-editor-actions"><button type="button" className="button secondary" disabled={working} onClick={() => { setEditor(null); setError(""); }}>ยกเลิก</button><button className="button primary" disabled={working || !editor.title.trim() || (editor.kind === "text" ? !editor.text.trim() : !editor.preview)}>{busy ? "กำลังบันทึก…" : "บันทึก"}</button></div>
      </form> : <>
        <div className="quick-library-actions"><input aria-label="ค้นหาในคลังข้อความตอบกลับ" autoComplete="off" value={query} placeholder="ค้นหาชื่อหรือข้อความ" onChange={event => setQuery(event.target.value)} /><button type="button" className="button primary" disabled={working || loading} onClick={() => start()}>+ เพิ่ม{tab === "text" ? "ข้อความ" : "การ์ด"}</button></div>
        <div className="quick-library-secondary">{tab === "text" && initialText.trim() && !initialText.startsWith("/") && <button type="button" className="text-button" onClick={() => start(undefined, initialText)}>บันทึกข้อความในช่องพิมพ์</button>}<button type="button" className="text-button" disabled={loading || working} onClick={() => void onReload()}>โหลดรายการล่าสุด</button></div>
        {loading && <p role="status">กำลังโหลดรายการ…</p>}
        <div className={`quick-library-list ${tab === "image" ? "is-images" : ""}`} aria-label={tab === "text" ? "รายการข้อความด่วน" : "รายการการ์ดรูปภาพ"}>
          {matches.map(item => <article key={item._id} className="quick-library-item">
            {item.kind === "image" && item.preview_url && <img src={`${item.preview_url}?v=${item.version}`} alt={item.title} loading="lazy" />}
            <h3>{item.title}</h3>{item.kind === "text" && <p className="quick-library-text">{item.text}</p>}
            <p className="quick-library-meta">{item.updated_by} · {formatDateTime(item.updated_at)}</p>
            <div className="quick-item-actions"><button type="button" className="button secondary" disabled={working || loading} onClick={() => onSelect(item)}>ใช้{item.kind === "text" ? "ข้อความ" : "รูปนี้"}</button><button type="button" className="text-button" disabled={working || loading} onClick={() => start(item)}>แก้ไข</button><button type="button" className="text-button danger" disabled={working || loading} onClick={() => { setDeleting(item._id); setError(""); }}>ลบ</button></div>
            {deleting === item._id && <div className="quick-delete-confirm"><p>ลบ “{item.title}” จากคลังของร้าน?</p><button type="button" className="text-button" disabled={working} onClick={() => setDeleting(null)}>ยกเลิก</button><button type="button" className="text-button danger" disabled={working} onClick={() => void remove(item)}>ยืนยันลบ</button></div>}
          </article>)}
        </div>
        {!loading && !matches.length && <p className="quick-empty">{query ? "ไม่พบรายการที่ตรงคำค้น" : tab === "text" ? "ยังไม่มีข้อความด่วน กดเพิ่มข้อความเพื่อบันทึกไว้ใช้ซ้ำ" : "ยังไม่มีการ์ดรูปภาพ กดเพิ่มการ์ดแล้วเลือกรูป"}</p>}
      </>}
    </div>
  </dialog>, document.body);
}

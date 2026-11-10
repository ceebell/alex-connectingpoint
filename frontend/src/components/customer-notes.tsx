"use client";
/* eslint-disable @next/next/no-img-element -- Note photos use authenticated same-origin URLs or local upload previews. */
import { ReactNode, useEffect, useId, useRef, useState } from "react";
import { api, ApiError, CustomerNote, errorText, NoteImage, NoteRevision } from "@/lib/api";
import { formatDateTime } from "@/lib/dates";

type PendingImage = { key: string; name: string; image_base64: string; url: string };
type FormImage = NoteImage | PendingImage;
const pending = (image: FormImage): image is PendingImage => "image_base64" in image;
const imageBody = (image: FormImage) => pending(image) ? { name: image.name, image_base64: image.image_base64 } : { id: image.id };
function mergeNotes(previous: CustomerNote[], incoming: CustomerNote[]) {
  const merged = new Map(previous.map(note => [note._id, note]));
  for (const note of incoming) if ((merged.get(note._id)?.version ?? 0) <= note.version) merged.set(note._id, note);
  return [...merged.values()].sort((a, b) => b.created_at.localeCompare(a.created_at) || b._id.localeCompare(a._id));
}

async function encode(file: File): Promise<PendingImage> {
  if (!/^image\/(jpeg|png|webp|gif)$/.test(file.type) || file.size > 40 * 1024 * 1024) throw new Error("image");
  const bitmap = await createImageBitmap(file);
  try {
    const scale = Math.min(1, 1600 / Math.max(bitmap.width, bitmap.height));
    const canvas = document.createElement("canvas"); canvas.width = Math.max(1, Math.round(bitmap.width * scale)); canvas.height = Math.max(1, Math.round(bitmap.height * scale));
    const context = canvas.getContext("2d"); if (!context) throw new Error("canvas");
    context.fillStyle = "#fff"; context.fillRect(0, 0, canvas.width, canvas.height); context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
    const blob = await new Promise<Blob | null>(resolve => canvas.toBlob(resolve, "image/jpeg", .8));
    if (!blob || blob.size > 2 * 1024 * 1024) throw new Error("size");
    const url = await new Promise<string>((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result)); reader.onerror = reject; reader.readAsDataURL(blob); });
    return { key: crypto.randomUUID(), name: file.name.slice(0, 180) || "รูปโน้ต.jpg", image_base64: url.split(",", 2)[1], url };
  } finally { bitmap.close(); }
}

function NoteImages({ images }: { images: NoteImage[] }) {
  return !!images.length && <div className="note-images">{images.map(image => <a key={image.id} href={image.url} target="_blank" rel="noreferrer"><img src={image.url} alt={image.name} loading="lazy" /></a>)}</div>;
}

function History({ noteId, endpoint, initial, onUnauthorized }: { noteId: string; endpoint: string; initial?: NoteRevision[]; onUnauthorized: () => void }) {
  const [history, setHistory] = useState(initial);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  async function load(open: boolean) {
    if (!open || history || loading) return;
    setLoading(true); setError("");
    try { const value = await api<{ note: CustomerNote }>(`${endpoint}/${noteId}`); setHistory(value.note.history ?? []); }
    catch (cause) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); }
    finally { setLoading(false); }
  }
  return <details className="note-history" onToggle={event => void load(event.currentTarget.open)}><summary>ประวัติการแก้ไข</summary>
    {loading && <p className="muted small" role="status">กำลังโหลดประวัติ…</p>}
    {error && <p className="notice warning" role="alert">{error}<button type="button" className="text-button" onClick={() => void load(true)}>ลองอีกครั้ง</button></p>}
    {history && [...history].reverse().map(revision => <div className="note-revision" key={revision.version}><p className="muted small">เวอร์ชัน {revision.version} · {revision.updated_by} · {formatDateTime(revision.updated_at)}</p><p className="note-text">{revision.text}</p><NoteImages images={revision.images} /></div>)}
  </details>;
}

function Editor({ text, images, busy, onText, onImages, onError, onBusy, children }: { text: string; images: FormImage[]; busy: boolean; onText: (value: string) => void; onImages: (value: FormImage[]) => void; onError: (value: string) => void; onBusy: (value: boolean) => void; children: ReactNode }) {
  const file = useRef<HTMLInputElement>(null);
  const textId = useId();
  async function attach(files: FileList | null) {
    const list = Array.from(files ?? []); if (file.current) file.current.value = "";
    if (!list.length) return;
    if (images.length + list.length > 5) { onError("แนบได้สูงสุด 5 รูปต่อโน้ต"); return; }
    onBusy(true); onError("");
    try { const added: PendingImage[] = []; for (const image of list) added.push(await encode(image)); onImages([...images, ...added]); }
    catch { onError("แนบรูปไม่สำเร็จ รองรับ JPG, PNG, WEBP หรือ GIF ขนาดไม่เกิน 40 MB ต่อรูป"); }
    finally { onBusy(false); }
  }
  return <>
    <label className="sr-only" htmlFor={textId}>ข้อความโน้ต</label>
    <textarea autoComplete="off" id={textId} rows={3} maxLength={5000} placeholder="เขียนโน้ตเกี่ยวกับลูกค้า…" disabled={busy} value={text} onChange={event => onText(event.target.value)} />
    {!!images.length && <div className="note-images note-image-drafts">{images.map(image => <div key={pending(image) ? image.key : image.id}><img src={image.url} alt={image.name} /><button type="button" className="note-remove-image" aria-label={`นำรูป ${image.name} ออกจากโน้ต`} disabled={busy} onClick={() => onImages(images.filter(value => value !== image))}>×</button></div>)}</div>}
    <div className="note-editor-actions"><button type="button" className="text-button" disabled={busy || images.length >= 5} onClick={() => file.current?.click()}><span aria-hidden="true">＋</span> แนบรูป</button><input ref={file} type="file" accept="image/jpeg,image/png,image/webp,image/gif" multiple hidden aria-label="แนบรูปในโน้ต" disabled={busy} onChange={event => void attach(event.target.files)} />{children}</div>
  </>;
}

export default function CustomerNotes({ userId, active, onUnauthorized }: { userId: string; active: boolean; onUnauthorized: () => void }) {
  const [notes, setNotes] = useState<CustomerNote[]>([]);
  const [next, setNext] = useState<string | null>(null);
  const [text, setText] = useState(""); const [images, setImages] = useState<FormImage[]>([]);
  const [editing, setEditing] = useState<CustomerNote | null>(null);
  const [editText, setEditText] = useState(""); const [editImages, setEditImages] = useState<FormImage[]>([]);
  const [requestId, setRequestId] = useState<string | null>(null);
  const [busy, setBusy] = useState(false); const [encoding, setEncoding] = useState(false); const [loading, setLoading] = useState(true);
  const [error, setError] = useState(""); const [saveError, setSaveError] = useState("");
  const endpoint = `/api/line/users/${encodeURIComponent(userId)}/notes`;
  function fail(cause: unknown) { if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); return errorText(cause); }
  useEffect(() => {
    if (!active) return;
    let cancelled = false;
    api<{ notes: CustomerNote[]; next: string | null }>(endpoint).then(value => { if (!cancelled) { setNotes(current => mergeNotes(current, value.notes)); setNext(value.next); setError(""); } }).catch(cause => { if (!cancelled) setError(fail(cause)); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [active, endpoint]); // eslint-disable-line react-hooks/exhaustive-deps

  async function more() {
    setLoading(true); setError("");
    try { const value = await api<{ notes: CustomerNote[]; next: string | null }>(`${endpoint}${next ? `?before=${next}` : ""}`); setNotes(current => mergeNotes(current, value.notes)); setNext(value.next); }
    catch (cause) { setError(fail(cause)); } finally { setLoading(false); }
  }
  async function create() {
    if (busy || encoding || (!text.trim() && !images.length)) return;
    const request_id = requestId ?? crypto.randomUUID(); setRequestId(request_id); setBusy(true); setSaveError("");
    try {
      const value = await api<{ note: CustomerNote }>(endpoint, { method: "POST", body: JSON.stringify({ request_id, text, images: images.map(imageBody) }) });
      setNotes(current => mergeNotes(current, [value.note])); setText(""); setImages([]); setRequestId(null);
    } catch (cause) { setSaveError(fail(cause)); } finally { setBusy(false); }
  }
  async function save() {
    if (!editing || busy || encoding || (!editText.trim() && !editImages.length)) return;
    setBusy(true); setSaveError("");
    try {
      const value = await api<{ note: CustomerNote }>(`${endpoint}/${editing._id}`, { method: "POST", body: JSON.stringify({ expected_version: editing.version, text: editText, images: editImages.map(imageBody) }) });
      setNotes(current => mergeNotes(current, [value.note])); setEditing(null);
    } catch (cause) {
      if (cause instanceof ApiError && cause.code === "RECORD_CONFLICT") {
        setSaveError("โน้ตนี้มีแอดมินแก้ไขแล้ว ข้อความที่คุณกรอกยังอยู่ กดยกเลิกเพื่อดูเวอร์ชันล่าสุดก่อนแก้ไขใหม่");
        try { const latest = await api<{ note: CustomerNote }>(`${endpoint}/${editing._id}`); setNotes(current => mergeNotes(current, [latest.note])); } catch (cause) { fail(cause); /* Keep the unsaved edit if reloading fails. */ }
      } else setSaveError(fail(cause));
    } finally { setBusy(false); }
  }
  const working = busy || encoding;
  return <section className="customer-notes" aria-label="โน้ตลูกค้า">
    <div className="new-customer-note"><Editor text={text} images={images} busy={working || !!editing} onText={value => { setText(value); setRequestId(null); }} onImages={value => { setImages(value); setRequestId(null); }} onBusy={setEncoding} onError={setSaveError}><button type="button" className="button primary" disabled={working || !!editing || (!text.trim() && !images.length)} onClick={() => void create()}><span aria-hidden="true">＋</span> {busy && !editing ? "กำลังบันทึก…" : "เพิ่มโน้ต"}</button></Editor></div>
    {saveError && !editing && <div className="notice warning" role="alert">{saveError}</div>}
    {error && <div className="notice warning" role="alert">{error}<button type="button" className="text-button" disabled={loading} onClick={() => void more()}>ลองโหลดอีกครั้ง</button></div>}
    {loading && <p className="muted small" role="status">กำลังโหลดโน้ต…</p>}
    {!loading && !notes.length && !error && <p className="muted small">ยังไม่มีโน้ตของลูกค้าคนนี้</p>}
    {notes.map(note => <article className="customer-note" key={note._id}>
      <div className="note-meta"><span>เพิ่มโดย {note.created_by} · {formatDateTime(note.created_at)}</span><button type="button" className="text-button" disabled={working || !!editing} aria-label={`แก้ไขโน้ต ${formatDateTime(note.created_at)}`} onClick={() => { setEditing(note); setEditText(note.text); setEditImages(note.images); setSaveError(""); }}>แก้ไข</button></div>
      {editing?._id === note._id ? <><Editor text={editText} images={editImages} busy={working} onText={setEditText} onImages={setEditImages} onBusy={setEncoding} onError={setSaveError}><button type="button" className="text-button" disabled={working} onClick={() => { setEditing(null); setSaveError(""); }}>ยกเลิก</button><button type="button" className="button primary" disabled={working || (!editText.trim() && !editImages.length)} onClick={() => void save()}>{busy ? "กำลังบันทึก…" : "บันทึกการแก้ไข"}</button></Editor>{saveError && <div className="notice warning" role="alert">{saveError}</div>}</> : <><p className="note-text">{note.text}</p><NoteImages images={note.images} /></>}
      {note.version > 1 && <><p className="muted small">แก้ไขโดย {note.updated_by} · {formatDateTime(note.updated_at)}</p><History key={`${note._id}-${note.version}`} noteId={note._id} endpoint={endpoint} initial={note.history} onUnauthorized={onUnauthorized} /></>}
    </article>)}
    {next && <button type="button" className="button secondary full" disabled={loading || working} onClick={() => void more()}>โหลดโน้ตก่อนหน้า</button>}
  </section>;
}

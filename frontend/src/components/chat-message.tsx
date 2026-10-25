"use client";
/* eslint-disable @next/next/no-img-element -- Protected media uses same-origin session cookies. */
import type { LineEvent } from "@/lib/api";

type Props = {
  message: LineEvent;
  selectedImages: string[];
  disabled: boolean;
  onToggleImage: (id: string, selected: boolean) => void;
};

export default function ChatMessage({ message, selectedImages, disabled, onToggleImage }: Props) {
  const admin = message.sender_role === "admin" || message.direction === "outbound";
  const sender = admin ? message.sender_name ?? "แอดมิน" : "ลูกค้า";
  const timestamp = new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", timeZone: "Asia/Bangkok" }).format(new Date(message.event_at));
  return <article id={`chat-message-${message._id}`} className={`message ${admin ? "message-admin" : "message-customer"}`} aria-label={`ข้อความจาก${sender}`}>
    <div className={`avatar line-avatar ${admin ? "admin-avatar" : ""}`} aria-hidden="true">{admin ? "A" : "L"}</div>
    <div className="message-body">
      <span className="message-sender">{sender}</span>
      <div className="message-bubble">
        {message.withdrawn ? <p className="muted">ข้อความถูกยกเลิกแล้ว</p> : message.payload?.message?.type === "image" ? message.media_id ? <>
          <a href={`/api/media/${message.media_id}`} target="_blank" rel="noreferrer"><img src={`/api/media/${message.media_id}`} alt={`รูปจาก${sender}`} loading="lazy" /></a>
          <label className="checkbox-row small"><input type="checkbox" checked={selectedImages.includes(message._id)} disabled={disabled || (!selectedImages.includes(message._id) && selectedImages.length >= 4)} onChange={event => onToggleImage(message._id, event.target.checked)} />แนบรูปนี้ให้ AI</label>
        </> : <p className="muted">รูปยังไม่พร้อมหรือหมดอายุ</p> : <p>{message.payload?.message?.text ?? message.payload?.postback?.data ?? `ข้อความประเภท ${message.payload?.message?.type ?? message.payload?.type ?? "อื่น ๆ"}`}</p>}
      </div>
      <div className="message-meta"><time dateTime={message.event_at}>{timestamp}</time><span>{message.source === "oa_manager_browser" ? "จาก OA Manager" : message.source === "alex_push" ? "ส่งจาก Alex" : message._id.slice(-6)}</span></div>
    </div>
  </article>;
}

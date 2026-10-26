"use client";
/* eslint-disable @next/next/no-img-element -- Protected media uses same-origin session cookies; LINE CDN images are external. */
import { useState } from "react";
import type { Customer, LineEvent } from "@/lib/api";

type Props = {
  message: LineEvent;
  customer?: Customer;
  selectedImages: string[];
  disabled: boolean;
  onToggleImage: (id: string, selected: boolean) => void;
};

// Webhooks carry only sticker IDs; the image comes from LINE's sticker CDN.
function Sticker({ id, keywords }: { id: string; keywords?: string[] }) {
  const [failed, setFailed] = useState(false);
  const label = keywords?.length ? `สติกเกอร์: ${keywords.slice(0, 3).join(", ")}` : "สติกเกอร์";
  if (failed || !/^\d{1,20}$/.test(id)) return <p className="muted">{label}</p>;
  return <img className="sticker" src={`https://stickershop.line-scdn.net/stickershop/v1/sticker/${id}/android/sticker.png`} alt={label} title={label} loading="lazy" referrerPolicy="no-referrer" onError={() => setFailed(true)} />;
}

export function CustomerAvatar({ customer, className = "" }: { customer?: Customer; className?: string }) {
  const [failed, setFailed] = useState(false);
  const picture = customer?.picture_url;
  return <div className={`avatar line-avatar ${className}`} aria-hidden="true">{picture && !failed && picture.startsWith("https://") ? <img src={picture} alt="" loading="lazy" referrerPolicy="no-referrer" onError={() => setFailed(true)} /> : (customer?.display_name?.trim().charAt(0) || "L")}</div>;
}

export default function ChatMessage({ message, customer, selectedImages, disabled, onToggleImage }: Props) {
  const admin = message.sender_role === "admin" || message.direction === "outbound";
  const sender = admin ? message.sender_name ?? "แอดมิน" : customer?.display_name || "ลูกค้า";
  const content = message.payload?.message;
  const timestamp = new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", timeZone: "Asia/Bangkok" }).format(new Date(message.event_at));
  return <article id={`chat-message-${message._id}`} className={`message ${admin ? "message-admin" : "message-customer"}`} aria-label={`ข้อความจาก${sender}`}>
    {admin ? <div className="avatar line-avatar admin-avatar" aria-hidden="true">A</div> : <CustomerAvatar customer={customer} />}
    <div className="message-body">
      <span className="message-sender">{sender}</span>
      <div className={`message-bubble ${content?.type === "sticker" && !message.withdrawn ? "sticker-bubble" : ""}`}>
        {message.withdrawn ? <p className="muted">ข้อความถูกยกเลิกแล้ว</p> : content?.type === "image" ? message.media_id ? <>
          <a href={`/api/media/${message.media_id}`} target="_blank" rel="noreferrer"><img src={`/api/media/${message.media_id}`} alt={`รูปจาก${sender}`} loading="lazy" /></a>
          <label className="checkbox-row small"><input type="checkbox" checked={selectedImages.includes(message._id)} disabled={disabled || (!selectedImages.includes(message._id) && selectedImages.length >= 4)} onChange={event => onToggleImage(message._id, event.target.checked)} />แนบรูปนี้ให้ AI</label>
        </> : <p className="muted">รูปยังไม่พร้อมหรือหมดอายุ</p> : content?.type === "sticker" ? <Sticker id={String(content.stickerId ?? "")} keywords={content.keywords} /> : <p>{content?.text ?? message.payload?.postback?.data ?? `ข้อความประเภท ${content?.type ?? message.payload?.type ?? "อื่น ๆ"}`}</p>}
      </div>
      <div className="message-meta"><time dateTime={message.event_at}>{timestamp}</time><span>{message.source === "oa_manager_browser" ? "จาก OA Manager" : message.source === "alex_push" ? "ส่งจาก Alex" : message._id.slice(-6)}</span></div>
    </div>
  </article>;
}

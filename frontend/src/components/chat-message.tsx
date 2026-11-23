"use client";
/* eslint-disable @next/next/no-img-element -- Protected media uses same-origin session cookies; LINE CDN images are external. */
import { useState } from "react";
import type { Customer, LineEvent } from "@/lib/api";
import { formatDateTime } from "@/lib/dates";
import ChatLinkedContent from "./chat-linked-content";
import ChatSticker from "./chat-sticker";
import ImagePreview from "./image-preview";

type Props = {
  message: LineEvent;
  customer?: Customer;
  selectedImages: string[];
  disabled: boolean;
  onToggleImage: (id: string, selected: boolean) => void;
};

export function CustomerAvatar({ customer, className = "" }: { customer?: Customer; className?: string }) {
  const [failed, setFailed] = useState(false);
  const picture = customer?.picture_url;
  return <div className={`avatar line-avatar ${className}`} aria-hidden="true">{picture && !failed && picture.startsWith("https://") ? <img src={picture} alt="" loading="lazy" referrerPolicy="no-referrer" onError={() => setFailed(true)} /> : (customer?.display_name?.trim().charAt(0) || "L")}</div>;
}

export default function ChatMessage({ message, customer, selectedImages, disabled, onToggleImage }: Props) {
  const admin = message.sender_role === "admin" || message.direction === "outbound";
  const sender = admin ? message.sender_name ?? "แอดมิน" : customer?.display_name || "ลูกค้า";
  const content = message.payload?.message;
  const timestamp = formatDateTime(message.event_at);
  return <article id={`chat-message-${message._id}`} className={`message ${admin ? "message-admin" : "message-customer"}`} aria-label={`ข้อความจาก${sender}`}>
    {admin ? <div className="avatar line-avatar admin-avatar" aria-hidden="true">A</div> : <CustomerAvatar customer={customer} />}
    <div className="message-body">
      <span className="message-sender">{sender}</span>
      <div className={`message-bubble ${content?.type === "sticker" && !message.withdrawn ? "sticker-bubble" : ""}`}>
        {message.withdrawn ? <p className="muted">ข้อความถูกยกเลิกแล้ว</p> : content?.type === "image" && content.preview_url?.startsWith("/api/line/outbound-media/") ? <ImagePreview src={content.original_url?.startsWith("/api/line/outbound-media/") ? content.original_url : content.preview_url} previewSrc={content.preview_url} alt={`รูปที่${sender}ส่ง`} /> : content?.type === "image" ? message.media_id ? <>
          <ImagePreview src={`/api/media/${message.media_id}`} alt={`รูปจาก${sender}`} />
          <label className="checkbox-row small"><input type="checkbox" checked={selectedImages.includes(message._id)} disabled={disabled || (!selectedImages.includes(message._id) && selectedImages.length >= 5)} onChange={event => onToggleImage(message._id, event.target.checked)} />แนบรูปนี้ให้ AI</label>
        </> : <p className="muted">รูปยังไม่พร้อมหรือหมดอายุ</p> : content?.type === "sticker" ? <ChatSticker key={String(content.stickerId ?? "")} id={String(content.stickerId ?? "")} keywords={content.keywords} /> : content?.type === "location" ? <ChatLinkedContent location={content} /> : <ChatLinkedContent text={content?.text ?? message.payload?.postback?.data ?? `ข้อความประเภท ${content?.type ?? message.payload?.type ?? "อื่น ๆ"}`} emojis={content?.emojis} />}
      </div>
      <div className="message-meta"><time dateTime={message.event_at}>{timestamp}</time><span>{message.source === "oa_manager_browser" ? "จาก OA Manager" : message.source === "alex_push" ? "ส่งจาก Alex" : message._id.slice(-6)}</span></div>
    </div>
  </article>;
}

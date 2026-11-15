"use client";
import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { chatAddressTarget, chatTextParts, sharedChatLocation, type ChatLinkTarget, type SharedLocation } from "@/lib/chat-links";

function MapPin() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 10c0 6-8 12-8 12S4 16 4 10a8 8 0 1 1 16 0Z" /><circle cx="12" cy="10" r="2.5" /></svg>;
}

function ExternalLinkDialog({ target, opener, onClose }: { target: ChatLinkTarget; opener: HTMLButtonElement; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const descriptionId = useId();
  const map = target.kind === "map";
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    cancel.current?.focus({ preventScroll: true });
    return () => {
      element?.close();
      if (opener.isConnected) opener.focus({ preventScroll: true });
    };
  }, [opener]);

  return createPortal(<dialog ref={dialog} className="chat-link-dialog" aria-labelledby={titleId} aria-describedby={descriptionId} onCancel={event => { event.preventDefault(); onClose(); }} onClick={event => {
    if (event.target !== event.currentTarget) return;
    const rect = event.currentTarget.getBoundingClientRect();
    if (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom) onClose();
  }}>
    <h2 id={titleId}>{map ? "เปิด Google Maps?" : "เปิดลิงก์ภายนอก?"}</h2>
    <p id={descriptionId}>{map ? "กำลังจะเปิดตำแหน่งหรือค้นหาสถานที่ใน Google Maps" : "กำลังจะเปิดเว็บไซต์ภายนอกในเบราว์เซอร์"} แชทนี้จะยังเปิดอยู่</p>
    {target.query && <p className="chat-link-query">{target.query}</p>}
    <div className="chat-link-destination"><strong dir="ltr">{new URL(target.url).hostname}</strong><p dir="ltr">{target.url}</p></div>
    <div className="chat-link-dialog-actions">
      <button ref={cancel} type="button" className="button secondary" onClick={onClose}>ยกเลิก</button>
      <a className="button primary" href={target.url} target="_blank" rel="noopener noreferrer" referrerPolicy="no-referrer" onClick={() => {
        // Let the browser follow the anchor during this user gesture before removing it.
        requestAnimationFrame(onClose);
      }}>{map ? "ตกลง เปิด Google Maps" : "ตกลง เปิดลิงก์"}</a>
    </div>
  </dialog>, document.body);
}

export default function ChatLinkedContent({ text = "", location }: { text?: string; location?: SharedLocation }) {
  const [pending, setPending] = useState<{ source: string; target: ChatLinkTarget; opener: HTMLButtonElement } | null>(null);
  const close = useCallback(() => setPending(null), []);
  const parts = useMemo(() => chatTextParts(text), [text]);
  const address = useMemo(() => chatAddressTarget(text), [text]);
  const pin = location ? sharedChatLocation(location) : null;
  const source = location ? JSON.stringify(location) : text;
  const open = (target: ChatLinkTarget, opener: HTMLButtonElement) => setPending({ source, target, opener });

  return <>
    {pin ? <div className="chat-location">
      <div className="chat-location-heading"><MapPin /><strong>{pin.title}</strong></div>
      {pin.address && <p>{pin.address}</p>}
      {pin.coordinates && <p className="chat-location-coordinates" dir="ltr">{pin.coordinates}</p>}
      {pin.target ? <button type="button" className="chat-map-action" aria-haspopup="dialog" onClick={event => pin.target && open(pin.target, event.currentTarget)}>เปิดใน Google Maps</button> : <p className="muted">ไม่มีพิกัดหรือที่อยู่สำหรับเปิดแผนที่</p>}
    </div> : <>
      <p>{parts.map((part, index) => part.target ? <button key={index} type="button" className="chat-inline-link" aria-haspopup="dialog" onClick={event => part.target && open(part.target, event.currentTarget)}>{part.text}</button> : part.text)}</p>
      {address && <button type="button" className="chat-map-action" aria-haspopup="dialog" onClick={event => open(address, event.currentTarget)}><MapPin />ค้นหาที่อยู่ใน Google Maps</button>}
    </>}
    {pending?.source === source && <ExternalLinkDialog target={pending.target} opener={pending.opener} onClose={close} />}
  </>;
}

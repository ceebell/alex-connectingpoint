"use client";
/* eslint-disable @next/next/no-img-element -- Protected images use session cookies and existing media URLs. */
import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

function ImageDialog({ src, alt, opener, onClose }: { src: string; alt: string; opener: HTMLButtonElement; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const [status, setStatus] = useState<"loading" | "ready" | "error">("loading");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    closeButton.current?.focus({ preventScroll: true });
    return () => {
      element?.close();
      if (opener.isConnected) opener.focus({ preventScroll: true });
    };
  }, [opener]);

  return createPortal(<dialog ref={dialog} className="image-viewer-dialog" aria-label="ดูรูปภาพขยาย" onPointerDown={event => event.stopPropagation()} onKeyDown={event => {
    event.stopPropagation();
    if (event.key !== "Tab") return;
    const buttons = event.currentTarget.querySelectorAll<HTMLButtonElement>("button:not(:disabled)");
    const first = buttons[0], last = buttons[buttons.length - 1];
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
  }} onCancel={event => { event.preventDefault(); onClose(); }} onClick={event => {
    if (event.target !== event.currentTarget) return;
    const rect = event.currentTarget.getBoundingClientRect();
    if (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom) onClose();
  }}>
    <header className="image-viewer-heading"><h2>ดูรูปภาพ</h2><button ref={closeButton} type="button" aria-label="ปิดรูปภาพ" onClick={onClose}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18" /></svg></button></header>
    <div className="image-viewer-body" aria-busy={status === "loading"}>
      {status === "loading" && <p role="status">กำลังโหลดรูป…</p>}
      {status === "error" ? <div className="image-viewer-error" role="alert"><p>โหลดรูปไม่สำเร็จ รูปอาจไม่พร้อมหรือหมดอายุแล้ว</p><button type="button" className="button secondary" onClick={() => { setStatus("loading"); setAttempt(value => value + 1); }}>ลองโหลดรูปใหม่</button></div>
        : <img key={attempt} src={src} alt={alt} referrerPolicy="no-referrer" onLoad={() => setStatus("ready")} onError={() => setStatus("error")} />}
    </div>
  </dialog>, document.body);
}

export default function ImagePreview({ src, previewSrc, alt, className = "", label }: { src: string; previewSrc?: string; alt: string; className?: string; label?: string }) {
  const [pending, setPending] = useState<{ src: string; opener: HTMLButtonElement } | null>(null);
  const close = useCallback(() => setPending(null), []);
  return <>
    <button type="button" className={`image-preview-trigger ${label ? "image-preview-link" : ""} ${className}`} aria-label={label ?? `ดูภาพขยาย: ${alt}`} aria-haspopup="dialog" onClick={event => setPending({ src, opener: event.currentTarget })}>
      {label ?? <img src={previewSrc ?? src} alt={alt} loading="lazy" referrerPolicy="no-referrer" />}
    </button>
    {pending?.src === src && <ImageDialog key={src} src={src} alt={alt} opener={pending.opener} onClose={close} />}
  </>;
}

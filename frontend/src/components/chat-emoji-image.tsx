"use client";
/* eslint-disable @next/next/no-img-element -- Emoji CDN images have fixed inline dimensions and text fallback. */
import { useState } from "react";

export default function ChatEmojiImage({ src, label }: { src: string; label: string }) {
  const [failed, setFailed] = useState(false);
  if (failed) return <>{label}</>;
  return <img className="chat-inline-emoji" src={src} alt={label} title={label} width={32} height={32} loading="lazy" referrerPolicy="no-referrer"
    data-artwork={src.startsWith("https://cdn.jsdelivr.net/gh/jdecked/twemoji@17.0.3/") ? "Twemoji 17.0.3, Twitter and contributors, CC-BY-4.0; /emoji-notices.txt" : undefined}
    onError={() => setFailed(true)} />;
}

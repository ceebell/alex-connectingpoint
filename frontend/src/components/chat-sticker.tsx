"use client";
/* eslint-disable @next/next/no-img-element -- Webhooks provide sticker IDs; images come from LINE's CDN. */
import { useState } from "react";
import { stickerUrl } from "@/lib/stickers";

export default function ChatSticker({ id, keywords }: { id: string; keywords?: string[] }) {
  const [failed, setFailed] = useState(false);
  const label = keywords?.length ? `สติกเกอร์: ${keywords.slice(0, 3).join(", ")}` : "สติกเกอร์";
  if (failed || !/^\d{1,20}$/.test(id)) return <p className="muted">{label}</p>;
  return <img className="sticker" src={stickerUrl(id)} alt={label} title={label} width={120} height={120} loading="lazy" referrerPolicy="no-referrer" onError={() => setFailed(true)} />;
}

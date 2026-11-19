"use client";

import EmojiPicker from "emoji-picker-react";
import { emojiImageUrl } from "@/lib/emojis";

const labels = {
  searchPlaceholder: "ค้นหาอีโมจิ เช่น ยิ้ม หัวใจ ขอบคุณ",
  searchLabel: "ค้นหาอีโมจิ",
  searchClear: "ล้างคำค้นอีโมจิ",
  searchResultsNone: "ไม่พบอีโมจิ ลองค้นหาด้วยคำอื่น",
  searchResultsOne: "พบ 1 อีโมจิ",
  searchResultsMany: "พบ %n อีโมจิ",
  categoryNavigation: "หมวดอีโมจิ",
  reactions: "อีโมจิที่ใช้บ่อย",
  expandReactions: "ดูอีโมจิทั้งหมด",
  loading: "กำลังโหลดอีโมจิ…",
  loadingError: "โหลดอีโมจิไม่สำเร็จ",
  retryLoading: "ลองโหลดอีโมจิใหม่",
  skinToneNeutral: "สีผิวเริ่มต้น",
  skinToneLight: "สีผิวอ่อน",
  skinToneMediumLight: "สีผิวค่อนข้างอ่อน",
  skinToneMedium: "สีผิวปานกลาง",
  skinToneMediumDark: "สีผิวค่อนข้างเข้ม",
  skinToneDark: "สีผิวเข้ม",
};

export default function ChatEmojiPicker({ onSelect }: { onSelect: (emoji: string) => void }) {
  return <EmojiPicker
    aria-label="รายการอีโมจิ"
    lang="th"
    data-artwork="Twemoji 17.0.3, Twitter and contributors, CC-BY-4.0; /emoji-notices.txt"
    emojiData={() => import("emoji-picker-react/data/emojis-th")}
    labels={labels}
    emojiStyle="twitter"
    getEmojiUrl={emojiImageUrl}
    colorScheme="light"
    className="chat-emoji-picker"
    width="100%"
    height="100%"
    autoFocusSearch={!window.matchMedia("(pointer: coarse)").matches}
    keepSearchFocus
    suggestedEmojisMode="recent"
    previewConfig={{ showPreview: false }}
    lazyLoadEmojis
    onEmojiClick={item => onSelect(item.emoji)}
  />;
}

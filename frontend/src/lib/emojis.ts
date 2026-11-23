export function emojiImageUrl(unified: string): string {
  let parts = unified.toLowerCase().split("-").map(part => parseInt(part, 16).toString(16));
  if (!parts.includes("200d")) parts = parts.filter(part => part !== "fe0f");
  let code = parts.join("-");
  if (code === "1f441-fe0f-200d-1f5e8-fe0f") code = "1f441-200d-1f5e8";
  return `https://cdn.jsdelivr.net/gh/jdecked/twemoji@17.0.3/assets/svg/${code}.svg`;
}

export function insertEmojiText(text: string, emoji: string, start: number, end: number): { text: string; caret: number } | null {
  const from = Math.max(0, Math.min(start, text.length));
  const to = Math.max(from, Math.min(end, text.length));
  const next = text.slice(0, from) + emoji + text.slice(to);
  return next.length <= 5000 ? { text: next, caret: from + emoji.length } : null;
}

export type LineEmoji = { index: number; length?: number; productId: string; emojiId: string };
export type EmojiTextPart = { text: string; emojiUrl?: string };

export function lineEmojiImageUrl(productId: string, emojiId: string): string | null {
  if (typeof productId !== "string" || !/^[a-f\d]{24}$/i.test(productId)
    || typeof emojiId !== "string" || !/^\d{3}$/.test(emojiId)) return null;
  return `https://stickershop.line-scdn.net/sticonshop/v1/sticon/${productId.toLowerCase()}/android/${emojiId}.png`;
}

function utf16Boundary(text: string, index: number): boolean {
  return !(index > 0 && index < text.length && text.charCodeAt(index - 1) >= 0xd800
    && text.charCodeAt(index - 1) <= 0xdbff && text.charCodeAt(index) >= 0xdc00 && text.charCodeAt(index) <= 0xdfff);
}

export function lineEmojiParts(text: string, emojis?: readonly LineEmoji[]): EmojiTextPart[] {
  const ranges = (Array.isArray(emojis) ? emojis : []).flatMap(emoji => {
    if (!emoji || typeof emoji !== "object") return [];
    const { index, productId, emojiId } = emoji;
    if (!Number.isInteger(index) || index < 0 || index >= text.length) return [];
    const length = emoji.length ?? (text[index] === "$" ? 1 : 0);
    const url = lineEmojiImageUrl(productId, emojiId);
    if (!url || !Number.isInteger(length) || length <= 0
      || index + length > text.length || !utf16Boundary(text, index) || !utf16Boundary(text, index + length)) return [];
    return [{ start: index, end: index + length, url }];
  }).sort((a, b) => a.start - b.start);
  const parts: EmojiTextPart[] = [];
  let cursor = 0;
  for (const range of ranges) {
    if (range.start < cursor) continue;
    if (range.start > cursor) parts.push({ text: text.slice(cursor, range.start) });
    parts.push({ text: text.slice(range.start, range.end), emojiUrl: range.url });
    cursor = range.end;
  }
  if (cursor < text.length || !parts.length) parts.push({ text: text.slice(cursor) });
  return parts;
}

const unicodeEmoji = new RegExp("\\p{Emoji_Presentation}|\\p{Extended_Pictographic}\\uFE0F|[0-9#*]\\uFE0F?\\u20E3", "u");
const regionalIndicator = new RegExp("^\\p{Regional_Indicator}$", "u");
const graphemes = typeof Intl.Segmenter === "function" ? new Intl.Segmenter("th", { granularity: "grapheme" }) : null;

export function unicodeEmojiParts(text: string): EmojiTextPart[] {
  if (!graphemes || !unicodeEmoji.test(text)) return [{ text }];
  const parts: EmojiTextPart[] = [];
  let cursor = 0;
  for (const { segment, index } of graphemes.segment(text)) {
    if (!unicodeEmoji.test(segment) || regionalIndicator.test(segment)) continue;
    if (index > cursor) parts.push({ text: text.slice(cursor, index) });
    parts.push({ text: segment, emojiUrl: emojiImageUrl(Array.from(segment, char => char.codePointAt(0)?.toString(16) ?? "").join("-")) });
    cursor = index + segment.length;
  }
  if (cursor < text.length || !parts.length) parts.push({ text: text.slice(cursor) });
  return parts;
}

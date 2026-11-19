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

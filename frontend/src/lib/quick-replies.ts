export type QuickReply = { _id: string; version: number; title: string; kind: "text" | "image"; text: string; preview_url?: string; created_by: string; updated_by: string; created_at: string; updated_at: string };

const normalize = (value: string) => value.normalize("NFC").toLocaleLowerCase("th-TH").replace(/[\u200b-\u200f\ufeff]/g, "").replace(/\s+/g, " ").trim();
export function slashQuery(value: string): string | null {
  return value.startsWith("/") && !value.includes("\n") ? value.slice(1).trim() : null;
}
export function matchingQuickReplies(items: QuickReply[], query: string, kind?: QuickReply["kind"]): QuickReply[] {
  const search = normalize(query); const words = search.split(" ").filter(Boolean);
  const score = (item: QuickReply) => normalize(item.title) === search ? 0 : normalize(item.title).startsWith(search) ? 1 : normalize(item.text).startsWith(search) ? 2 : 3;
  return items.filter(item => (!kind || item.kind === kind) && words.every(word => normalize(`${item.title} ${item.text}`).includes(word)))
    .sort((a, b) => score(a) - score(b) || a.title.localeCompare(b.title, "th") || a._id.localeCompare(b._id));
}

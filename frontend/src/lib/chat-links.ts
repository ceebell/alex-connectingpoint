export type ChatLinkTarget = { url: string; kind: "website" | "map"; query?: string };
export type ChatTextPart = { text: string; target?: ChatLinkTarget };
export type SharedLocation = { title?: string; address?: string; latitude?: number; longitude?: number };

const forbiddenCharacters = /[\u0000-\u001f\u007f-\u009f\u200b-\u200f\u202a-\u202e\u2060-\u2069\ufeff]/u;

export function externalChatTarget(value: string): ChatLinkTarget | null {
  if (value.length > 4096 || forbiddenCharacters.test(value) || /\s|\\/u.test(value)) return null;
  try {
    const url = new URL(/^www\./i.test(value) ? `https://${value}` : value);
    if (!["https:", "http:"].includes(url.protocol) || !url.hostname || url.username || url.password) return null;
    const map = ["maps.app.goo.gl", "maps.google.com", "maps.google.co.th"].includes(url.hostname)
      || (["google.com", "www.google.com", "google.co.th", "www.google.co.th"].includes(url.hostname) && /^\/maps(?:\/|$)/.test(url.pathname))
      || (url.hostname === "goo.gl" && url.pathname.startsWith("/maps/"));
    return { url: url.href, kind: map ? "map" : "website" };
  } catch {
    return null;
  }
}

export function googleMapsTarget(query: string): ChatLinkTarget | null {
  const trimmed = query.replace(/[\r\n\t]+/g, " ").trim();
  if (!trimmed || forbiddenCharacters.test(trimmed)) return null;
  const url = new URL("https://www.google.com/maps/search/");
  url.search = new URLSearchParams({ api: "1", query: trimmed }).toString();
  return url.href.length <= 2048 ? { url: url.href, kind: "map", query: trimmed } : null;
}

function coordinates(latitude: unknown, longitude: unknown): string | null {
  return typeof latitude === "number" && typeof longitude === "number"
    && Number.isFinite(latitude) && Number.isFinite(longitude)
    && Math.abs(latitude) <= 90 && Math.abs(longitude) <= 180
    ? `${latitude}, ${longitude}` : null;
}

function locationText(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

export function sharedChatLocation(location: SharedLocation) {
  const title = locationText(location.title);
  const address = locationText(location.address);
  const coordinateText = coordinates(location.latitude, location.longitude);
  return {
    title: title || "ตำแหน่งที่ส่งมา",
    address,
    coordinates: coordinateText,
    target: googleMapsTarget(coordinateText || address || title),
  };
}

function trimUrlPunctuation(value: string): string {
  let result = value;
  while (result) {
    if (/[.,!?;:。！？、，；：]$/u.test(result)) { result = result.slice(0, -1); continue; }
    const pairs: Record<string, string> = { ")": "(", "]": "[", "}": "{" };
    const last = result.at(-1) ?? "";
    const opener = pairs[last];
    if (opener && result.split(last).length > result.split(opener).length) { result = result.slice(0, -1); continue; }
    break;
  }
  return result;
}

export function chatTextParts(text: string): ChatTextPart[] {
  const ranges: { start: number; end: number; target?: ChatLinkTarget }[] = [];
  // Reserve URL ranges even when invalid, so coordinates inside them cannot become links.
  for (const match of text.matchAll(/(?:https?:\/\/|www\.)[^\s<>"'`]+/giu)) {
    const start = match.index;
    const value = trimUrlPunctuation(match[0]);
    const before = text.slice(0, start).at(-1) ?? "";
    const boundary = !/[\p{L}\p{N}_@/:]/u.test(before);
    ranges.push({ start, end: start + value.length, target: boundary ? externalChatTarget(value) ?? undefined : undefined });
  }
  // Decimal precision or an explicit label distinguishes a pin from ordinary pairs of amounts.
  for (const match of text.matchAll(/([+-]?\d{1,3}(?:\.\d+)?)[ \t]*,[ \t]*([+-]?\d{1,3}(?:\.\d+)?)/g)) {
    const start = match.index;
    const end = start + match[0].length;
    if (ranges.some(range => start < range.end && end > range.start)) continue;
    if (/[\d.,/]/.test(text[start - 1] ?? "") || /[\d.,/]/.test(text[end] ?? "")) continue;
    const precise = /\.\d{3,}/.test(match[1]) || /\.\d{3,}/.test(match[2]);
    const labelled = /(?:พิกัด|gps|coordinates?|location)[ \t]*[:：]?[ \t]*$/i.test(text.slice(Math.max(0, start - 30), start));
    if (!precise && !labelled) continue;
    const query = coordinates(Number(match[1]), Number(match[2]));
    const target = query ? googleMapsTarget(query) : null;
    if (target) ranges.push({ start, end, target });
  }
  ranges.sort((a, b) => a.start - b.start);
  const parts: ChatTextPart[] = [];
  let cursor = 0;
  for (const range of ranges) {
    if (!range.target) continue;
    if (range.start > cursor) parts.push({ text: text.slice(cursor, range.start) });
    parts.push({ text: text.slice(range.start, range.end), target: range.target });
    cursor = range.end;
  }
  if (cursor < text.length || !parts.length) parts.push({ text: text.slice(cursor) });
  return parts;
}

const addressLabel = /^(?:ที่อยู่(?:ในการจัดส่ง|สำหรับจัดส่ง|จัดส่ง|รับสินค้า)?|สถานที่(?:จัดส่ง)?|สถานที่นัดรับ|address|location)(?:[ \t]*[:：][ \t]*|[ \t]+|$)/iu;
const contactOrOrder = /(?:โทร(?:ศัพท์)?|เบอร์(?:โทร)?\s*[:：]?\s*[\d(+]|\btel(?:ephone)?\b|\bphone\b|มือถือ|เลขบัญชี|บัญชี|พร้อมเพย์|ยอด(?:รวม|เช่า|โอน)|ค่าเช่า|ค่าประกัน|ขอบคุณ|ขอบใจ|https?:\/\/|www\.|[\w.+-]+@[\w.-]+\.)/iu;
const addressMarkers = [/หมู่(?:ที่)?\s*\d|ม\.\s*\d/u, /ซอย|ซ\./u, /ถนน|ถ\./u, /ตำบล|ต\./u, /แขวง/u, /อำเภอ|อ\./u, /เขต/u, /จังหวัด|จ\./u, /กรุงเทพ|กทม/u];

export function chatAddressTarget(text: string): ChatLinkTarget | null {
  const lines = text.split(/\r?\n/).map(line => line.trim());
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    const labelled = addressLabel.test(line);
    if (!labelled && !/^(?:บ้านเลขที่\s*)?\d{1,6}(?:[/-]\d{1,6})?(?:\s|[ก-๙])/u.test(line)) continue;
    const address: string[] = [];
    for (let next = index; next < Math.min(lines.length, index + 6); next++) {
      let part = next === index ? lines[next].replace(addressLabel, "") : lines[next];
      if (/^(?:\+66|0)[\d ()-]{8,18}$/.test(part)) break;
      const stop = part.search(contactOrOrder);
      if (stop >= 0) part = part.slice(0, stop).trim();
      if (part) address.push(part);
      if (stop >= 0 || (!part && next > index) || /(?:^|\D)[1-9]\d{4}$/.test(part)) break;
      if (labelled && part && !/\d/.test(part) && !addressMarkers.some(marker => marker.test(part))) break;
    }
    const query = address.join(" ").trim();
    const markerCount = addressMarkers.filter(marker => marker.test(query)).length;
    if (query.length >= 5 && (labelled || (markerCount >= 2 && /\d/.test(query)))) {
      const target = googleMapsTarget(query);
      if (target) return target;
    }
  }
  return null;
}

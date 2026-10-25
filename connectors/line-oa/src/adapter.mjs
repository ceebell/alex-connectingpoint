export class SchemaError extends Error {
  constructor(code = "UNSUPPORTED_SCHEMA") { super(code); this.code = code; }
}

export function id(value) {
  if (typeof value === "number" && Number.isSafeInteger(value)) value = String(value);
  if (typeof value !== "string" || !/^[A-Za-z0-9_.:-]{1,128}$/.test(value)) throw new SchemaError();
  return value;
}

export function listPage(value) {
  if (!value || !Array.isArray(value.list)) throw new SchemaError();
  const next = value.next ?? value.nextCursor ?? value.paging?.next ?? null;
  if (next !== null && (typeof next !== "string" || next.length > 1000)) throw new SchemaError();
  if (!next && (value.hasMore === true || value.list.length >= 100)) throw new SchemaError("UNSUPPORTED_PAGINATION");
  return { items: value.list, next };
}

function timestamp(value) {
  const at = typeof value === "string" ? (/^\d{13}$/.test(value) ? Number(value) : Date.parse(value)) : value;
  if (!Number.isSafeInteger(at) || at < 1_000_000_000_000 || at > Date.now() + 300_000) throw new SchemaError();
  return at;
}

// Only explicit roles or exact account/user IDs are accepted. Unknown schemas
// stop the cycle; visual alignment and display names are never role evidence.
export function decodeMessages(records, botId, userId) {
  return records.flatMap(record => {
    if (!record || typeof record !== "object") throw new SchemaError();
    const message = record.message ?? record;
    if (message.type !== "text") {
      if (["image", "video", "audio", "file", "sticker", "location"].includes(message.type)) return [];
      throw new SchemaError();
    }
    const sender = record.sender ?? record.source ?? message.sender;
    const actor = String(record.senderType ?? record.fromType ?? sender?.type ?? "").toLowerCase();
    const senderId = sender?.id ?? sender?.userId ?? sender?.mid ?? record.fromMid;
    let role;
    if (["user", "customer"].includes(actor) || (userId && senderId === userId)) role = "customer";
    else if (["operator", "admin", "bot", "official_account"].includes(actor) || senderId === botId) role = "admin";
    else throw new SchemaError();
    if (typeof message.text !== "string" || !message.text.trim() || [...message.text].length > 5000) throw new SchemaError();
    return [{ id: id(message.id ?? message.messageId ?? record.id), text: message.text, at: timestamp(record.timestamp ?? record.createdTime ?? record.createdAt ?? message.timestamp), role }];
  }).sort((a, b) => a.at - b.at || a.id.localeCompare(b.id));
}

export function makeBatches(messages, botId, chatId) {
  return messages.filter(m => m.role === "admin").map(message => {
    const anchor = messages.filter(m => m.role === "customer" && m.at <= message.at).at(-1);
    if (!anchor) return null;
    return { bot_id: botId, chat_id: chatId, anchor_message_ids: [anchor.id], messages: [{ id: message.id, text: message.text, sent_at_ms: message.at }] };
  });
}

// Return field names and types only. Diagnostics must not contain message text,
// account identifiers, session cookies, QR payloads or credential values.
export function shape(value, depth = 0) {
  if (depth > 4) return typeof value;
  if (Array.isArray(value)) return { array: value.slice(0, 1).map(v => shape(v, depth + 1)) };
  if (value && typeof value === "object") return Object.fromEntries(Object.entries(value).slice(0, 35).map(([key, child], index) => [/^[A-Za-z_][A-Za-z0-9_]{0,39}$/.test(key) && !/^U[0-9a-f]{32}$/i.test(key) ? key : `{field${index}}`, shape(child, depth + 1)]));
  return value === null ? "null" : typeof value;
}

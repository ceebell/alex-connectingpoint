// src/index.mjs
import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";

// src/adapter.mjs
var SchemaError = class extends Error {
  constructor(code = "UNSUPPORTED_SCHEMA") {
    super(code);
    this.code = code;
  }
};
function id(value) {
  if (typeof value === "number" && Number.isSafeInteger(value)) value = String(value);
  if (typeof value !== "string" || !/^[A-Za-z0-9_.:-]{1,128}$/.test(value)) throw new SchemaError();
  return value;
}
function listPage(value) {
  if (!value || !Array.isArray(value.list)) throw new SchemaError();
  const next = value.next ?? value.nextCursor ?? value.paging?.next ?? null;
  if (next !== null && (typeof next !== "string" || next.length > 1e3)) throw new SchemaError();
  if (!next && (value.hasMore === true || value.list.length >= 100)) throw new SchemaError("UNSUPPORTED_PAGINATION");
  return { items: value.list, next };
}
function timestamp(value) {
  const at = typeof value === "string" ? /^\d{13}$/.test(value) ? Number(value) : Date.parse(value) : value;
  if (!Number.isSafeInteger(at) || at < 1e12 || at > Date.now() + 3e5) throw new SchemaError();
  return at;
}
function decodeMessages(records, botId2, userId) {
  return records.flatMap((record) => {
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
    if (["user", "customer"].includes(actor) || userId && senderId === userId) role = "customer";
    else if (["operator", "admin", "bot", "official_account"].includes(actor) || senderId === botId2) role = "admin";
    else throw new SchemaError();
    if (typeof message.text !== "string" || !message.text.trim() || [...message.text].length > 5e3) throw new SchemaError();
    return [{ id: id(message.id ?? message.messageId ?? record.id), text: message.text, at: timestamp(record.timestamp ?? record.createdTime ?? record.createdAt ?? message.timestamp), role }];
  }).sort((a, b) => a.at - b.at || a.id.localeCompare(b.id));
}
function makeBatches(messages, botId2, chatId) {
  return messages.filter((m) => m.role === "admin").map((message) => {
    const anchor = messages.filter((m) => m.role === "customer" && m.at <= message.at).at(-1);
    if (!anchor) return null;
    return { bot_id: botId2, chat_id: chatId, anchor_message_ids: [anchor.id], messages: [{ id: message.id, text: message.text, sent_at_ms: message.at }] };
  });
}
function shape(value, depth = 0) {
  if (depth > 4) return typeof value;
  if (Array.isArray(value)) return { array: value.slice(0, 1).map((v) => shape(v, depth + 1)) };
  if (value && typeof value === "object") return Object.fromEntries(Object.entries(value).slice(0, 35).map(([key, child], index) => [/^[A-Za-z_][A-Za-z0-9_]{0,39}$/.test(key) && !/^U[0-9a-f]{32}$/i.test(key) ? key : `{field${index}}`, shape(child, depth + 1)]));
  return value === null ? "null" : typeof value;
}

// src/control.mjs
import http from "node:http";
import { createHash, timingSafeEqual } from "node:crypto";
import { readFile } from "node:fs/promises";
async function controlServer({ page: page2, getPage = () => page2, username, password: password2, status, diagnostics, onOpen, port = 8082, host = "0.0.0.0" }) {
  const html = await readFile(new URL("./control.html", import.meta.url));
  const digest = (value) => createHash("sha256").update(value).digest();
  const expected = digest("Basic " + Buffer.from(`${username}:${password2}`).toString("base64"));
  const server2 = http.createServer(async (req, res) => {
    res.setHeader("Cache-Control", "no-store");
    res.setHeader("X-Content-Type-Options", "nosniff");
    res.setHeader("X-Frame-Options", "DENY");
    const json = (code, value) => {
      res.writeHead(code, { "Content-Type": "application/json" });
      res.end(JSON.stringify(value));
    };
    if (req.method === "GET" && req.url === "/health") {
      json(200, { status: "ok" });
      return;
    }
    if (!timingSafeEqual(expected, digest(req.headers.authorization ?? ""))) {
      res.setHeader("WWW-Authenticate", 'Basic realm="Alex OA connector", charset="UTF-8"');
      json(401, { error: "UNAUTHORIZED" });
      return;
    }
    try {
      const page3 = getPage();
      if (req.method === "GET" && req.url === "/") {
        res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
        res.end(html);
        return;
      }
      if (req.method === "GET" && req.url === "/status") {
        json(200, status());
        return;
      }
      if (req.method === "GET" && req.url === "/schema") {
        json(200, diagnostics());
        return;
      }
      if (req.method === "GET" && req.url === "/screen") {
        const bytes = await page3.screenshot({ type: "jpeg", quality: 75 });
        res.writeHead(200, { "Content-Type": "image/jpeg" });
        res.end(bytes);
        return;
      }
      if (req.method !== "POST" || req.url !== "/action" || req.headers["x-oa-control"] !== "1" || req.headers.origin && req.headers.origin !== `http://${req.headers.host}`) {
        json(403, { error: "FORBIDDEN" });
        return;
      }
      let size = 0;
      const parts = [];
      for await (const chunk of req) {
        size += chunk.length;
        if (size > 16384) {
          json(413, { error: "TOO_LARGE" });
          return;
        }
        parts.push(chunk);
      }
      const action = JSON.parse(Buffer.concat(parts).toString("utf8"));
      if (action.type === "open") await onOpen();
      else if (action.type === "click" && Number.isFinite(action.x) && Number.isFinite(action.y) && action.x >= 0 && action.x < 1280 && action.y >= 0 && action.y < 900) await page3.mouse.click(action.x, action.y);
      else if (action.type === "text" && typeof action.text === "string" && action.text.length <= 2e3) await page3.keyboard.insertText(action.text);
      else if (action.type === "key" && ["Enter", "Tab", "Backspace", "Escape", "Control+A"].includes(action.key)) await page3.keyboard.press(action.key);
      else if (action.type === "scroll" && [-600, 600].includes(action.dy)) await page3.mouse.wheel(0, action.dy);
      else {
        json(400, { error: "INVALID_ACTION" });
        return;
      }
      json(200, { status: "ok" });
    } catch {
      json(503, { error: "CONTROL_UNAVAILABLE" });
    }
  });
  await new Promise((resolve, reject) => {
    server2.once("error", reject);
    server2.listen(port, host, resolve);
  });
  return server2;
}

// src/index.mjs
var required = (name) => {
  const value = process.env[name];
  if (!value?.trim()) throw new Error(`Missing ${name}`);
  return value;
};
var botId = id(required("LINE_DESTINATION_ID"));
var token = required("INQUIRY_ADMIN_TOKEN");
var password = required("ADMIN_PASSWORD");
if (password.length < 16) throw new Error("ADMIN_PASSWORD must have at least 16 characters");
var backend = new URL(process.env.OA_BACKEND_URL ?? "http://host.docker.internal:18080");
if (!["http:", "https:"].includes(backend.protocol) || backend.username || backend.password || backend.search || backend.hash || backend.pathname !== "/") throw new Error("Invalid OA_BACKEND_URL");
var pollMs = Number(process.env.OA_POLL_SECONDS ?? 30) * 1e3;
if (!Number.isInteger(pollMs) || pollMs < 15e3 || pollMs > 3e5) throw new Error("OA_POLL_SECONDS must be 15..300");
var profile = process.env.OA_PROFILE_PATH ?? "/profile";
await mkdir(profile, { recursive: true, mode: 448 });
var context = await chromium.launchPersistentContext(profile, { headless: true, viewport: { width: 1280, height: 900 }, locale: "th-TH", timezoneId: "Asia/Bangkok", acceptDownloads: false });
await context.route("https://chat.line.biz/api/v1/bots/**", async (route) => {
  if (!["GET", "HEAD"].includes(route.request().method())) await route.abort("blockedbyclient");
  else await route.continue();
});
var page = context.pages()[0] ?? await context.newPage();
context.on("page", (next) => {
  page = next;
  next.on("close", () => {
    page = context.pages().find((p) => !p.isClosed()) ?? page;
  });
});
var state = { state: "NEEDS_LOGIN", synced: 0, pending: 0, last_checked_at: null };
var diagnostic = {};
var stopping = false;
var accepted = /* @__PURE__ */ new Set();
var open = async () => {
  state.state = "NEEDS_LOGIN";
  await page.goto("https://chat.line.biz/", { waitUntil: "domcontentloaded", timeout: 3e4 });
};
try {
  await open();
} catch {
  state.state = "LINE_UNAVAILABLE";
}
async function lineGet(path, csrfToken = null) {
  if (new URL(page.url()).hostname !== "chat.line.biz") throw new SchemaError("NEEDS_LOGIN");
  const response = await page.evaluate(async ({ path: path2, csrfToken: csrfToken2 }) => {
    const r = await fetch(path2, { credentials: "include", cache: "no-store", headers: csrfToken2 ? { "x-xsrf-token": csrfToken2 } : {} });
    const json = await r.json().catch(() => null);
    return { status: r.status, json };
  }, { path, csrfToken });
  if ([401, 403].includes(response.status)) throw new SchemaError("NEEDS_LOGIN");
  if (response.status === 404) throw new SchemaError();
  if (response.status !== 200) throw new SchemaError("LINE_UNAVAILABLE");
  diagnostic[path.split("?")[0].replace(/U[0-9a-f]{32}/g, "{id}").replace(/\/messages\/[^/]+$/, "/messages/{chat}")] = shape(response.json);
  return response.json;
}
async function pages(path, csrfToken) {
  const records = [];
  const visited = /* @__PURE__ */ new Set();
  let cursor = null;
  for (let n = 0; n < 10; n++) {
    const url = path + (cursor ? `${path.includes("?") ? "&" : "?"}cursor=${encodeURIComponent(cursor)}` : "");
    const result = listPage(await lineGet(url, csrfToken));
    records.push(...result.items);
    if (!result.next) return records;
    if (visited.has(result.next)) throw new SchemaError("UNSUPPORTED_SCHEMA");
    visited.add(result.next);
    cursor = result.next;
  }
  throw new SchemaError("UNSUPPORTED_SCHEMA");
}
async function post(path, body) {
  const r = await fetch(new URL(path, backend), { method: "POST", headers: { "Authorization": `Bearer ${token}`, "Content-Type": "application/json" }, body: JSON.stringify(body), signal: AbortSignal.timeout(15e3) });
  const value = await r.json().catch(() => ({}));
  if (r.status === 409 && value.error?.code === "OA_UNLINKED") return null;
  if (r.status === 409 && value.error?.code === "OA_MESSAGE_CONFLICT") throw new SchemaError();
  if (!r.ok) throw new SchemaError("BACKEND_UNAVAILABLE");
  return value;
}
async function cycle() {
  state.pending = 0;
  try {
    const csrf = await lineGet("/api/v1/csrfToken");
    if (typeof csrf.token !== "string" || csrf.token.length > 4096) throw new SchemaError();
    const bots = listPage(await lineGet("/api/v1/bots?noFilter=true&limit=100", csrf.token)).items;
    if (bots.some((bot) => typeof bot.botId !== "string")) throw new SchemaError();
    if (!bots.some((bot) => bot.botId === botId)) throw new SchemaError("NEEDS_LOGIN");
    const chats = await pages(`/api/v1/bots/${botId}/chats?folderType=ALL&tagIds=&limit=100`, csrf.token);
    let ingested = 0;
    for (const chat of chats) {
      const chatId = id(chat.chatId ?? chat.id);
      const raw = await pages(`/api/v1/bots/${botId}/messages/${chatId}?limit=100`, csrf.token);
      const messages = decodeMessages(raw, botId, chat.userId ?? chat.user?.userId);
      for (const batch of makeBatches(messages, botId, chatId)) {
        if (!batch) {
          state.pending++;
          continue;
        }
        const key = `${chatId}:${batch.messages[0].id}`;
        if (accepted.has(key)) continue;
        const result = await post("/api/connectors/line-oa/messages", batch);
        if (!result) state.pending++;
        else {
          state.synced += result.inserted;
          ingested++;
          accepted.add(key);
        }
      }
    }
    state.state = state.pending ? "UNLINKED" : ingested || accepted.size ? "SYNCING" : "READY";
    state.last_checked_at = (/* @__PURE__ */ new Date()).toISOString();
  } catch (error) {
    state.state = error instanceof SchemaError ? error.code === "UNSUPPORTED_PAGINATION" ? "UNSUPPORTED_SCHEMA" : error.code : "LINE_UNAVAILABLE";
  }
  try {
    await post("/api/connectors/line-oa/status", { state: state.state, synced: state.synced, pending: state.pending });
  } catch {
    state.state = "BACKEND_UNAVAILABLE";
  }
}
var server = await controlServer({ getPage: () => page, username: process.env.ADMIN_USERNAME ?? "admin", password, status: () => state, diagnostics: () => diagnostic, onOpen: open, port: Number(process.env.OA_CONTROL_PORT ?? 8082) });
console.log("OA connector control is ready; login may be required.");
var loop = async () => {
  if (stopping) return;
  await cycle();
  if (!stopping) setTimeout(loop, pollMs);
};
void loop();
async function stop() {
  if (stopping) return;
  stopping = true;
  state.state = "STOPPED";
  server.close();
  await context.close();
  process.exit(0);
}
process.on("SIGTERM", stop);
process.on("SIGINT", stop);

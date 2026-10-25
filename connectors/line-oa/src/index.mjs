import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { id, listPage, decodeMessages, makeBatches, shape, SchemaError } from "./adapter.mjs";
import { controlServer } from "./control.mjs";

const required = name => { const value = process.env[name]; if (!value?.trim()) throw new Error(`Missing ${name}`); return value; };
const botId = id(required("LINE_DESTINATION_ID"));
const token = required("INQUIRY_ADMIN_TOKEN");
const password = required("ADMIN_PASSWORD");
if (password.length < 16) throw new Error("ADMIN_PASSWORD must have at least 16 characters");
const backend = new URL(process.env.OA_BACKEND_URL ?? "http://host.docker.internal:18080");
if (!["http:", "https:"].includes(backend.protocol) || backend.username || backend.password || backend.search || backend.hash || backend.pathname !== "/") throw new Error("Invalid OA_BACKEND_URL");
const pollMs = Number(process.env.OA_POLL_SECONDS ?? 30) * 1000;
if (!Number.isInteger(pollMs) || pollMs < 15000 || pollMs > 300000) throw new Error("OA_POLL_SECONDS must be 15..300");
const profile = process.env.OA_PROFILE_PATH ?? "/profile";
await mkdir(profile, { recursive: true, mode: 0o700 });
const context = await chromium.launchPersistentContext(profile, { headless: true, viewport: { width: 1280, height: 900 }, locale: "th-TH", timezoneId: "Asia/Bangkok", acceptDownloads: false });
// Login forms may submit to LINE's account service, but chat sends, read marks,
// deletes and setting mutations through the OA chat API are always blocked.
await context.route("https://chat.line.biz/api/v1/bots/**", async route => {
  if (!["GET", "HEAD"].includes(route.request().method())) await route.abort("blockedbyclient");
  else await route.continue();
});
let page = context.pages()[0] ?? await context.newPage();
context.on("page", next => { page = next; next.on("close", () => { page = context.pages().find(p => !p.isClosed()) ?? page; }); });
const state = { state: "NEEDS_LOGIN", synced: 0, pending: 0, last_checked_at: null };
let diagnostic = {};
let stopping = false;
const accepted = new Set();
const open = async () => { state.state = "NEEDS_LOGIN"; await page.goto("https://chat.line.biz/", { waitUntil: "domcontentloaded", timeout: 30000 }); };
try { await open(); } catch { state.state = "LINE_UNAVAILABLE"; }

async function lineGet(path, csrfToken = null) {
  if (new URL(page.url()).hostname !== "chat.line.biz") throw new SchemaError("NEEDS_LOGIN");
  const response = await page.evaluate(async ({ path, csrfToken }) => {
    const r = await fetch(path, { credentials: "include", cache: "no-store", headers: csrfToken ? { "x-xsrf-token": csrfToken } : {} });
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
  const records = []; const visited = new Set(); let cursor = null;
  for (let n = 0; n < 10; n++) {
    const url = path + (cursor ? `${path.includes("?") ? "&" : "?"}cursor=${encodeURIComponent(cursor)}` : "");
    const result = listPage(await lineGet(url, csrfToken)); records.push(...result.items);
    if (!result.next) return records;
    if (visited.has(result.next)) throw new SchemaError("UNSUPPORTED_SCHEMA");
    visited.add(result.next); cursor = result.next;
  }
  throw new SchemaError("UNSUPPORTED_SCHEMA");
}

async function post(path, body) {
  const r = await fetch(new URL(path, backend), { method: "POST", headers: { "Authorization": `Bearer ${token}`, "Content-Type": "application/json" }, body: JSON.stringify(body), signal: AbortSignal.timeout(15000) });
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
    if (bots.some(bot => typeof bot.botId !== "string")) throw new SchemaError();
    if (!bots.some(bot => bot.botId === botId)) throw new SchemaError("NEEDS_LOGIN");
    const chats = await pages(`/api/v1/bots/${botId}/chats?folderType=ALL&tagIds=&limit=100`, csrf.token);
    let ingested = 0;
    for (const chat of chats) {
      const chatId = id(chat.chatId ?? chat.id);
      const raw = await pages(`/api/v1/bots/${botId}/messages/${chatId}?limit=100`, csrf.token);
      const messages = decodeMessages(raw, botId, chat.userId ?? chat.user?.userId);
      for (const batch of makeBatches(messages, botId, chatId)) {
        if (!batch) { state.pending++; continue; }
        const key = `${chatId}:${batch.messages[0].id}`;
        if (accepted.has(key)) continue;
        const result = await post("/api/connectors/line-oa/messages", batch);
        if (!result) state.pending++;
        else { state.synced += result.inserted; ingested++; accepted.add(key); }
      }
    }
    state.state = state.pending ? "UNLINKED" : ingested || accepted.size ? "SYNCING" : "READY";
    state.last_checked_at = new Date().toISOString();
  } catch (error) { state.state = error instanceof SchemaError ? (error.code === "UNSUPPORTED_PAGINATION" ? "UNSUPPORTED_SCHEMA" : error.code) : "LINE_UNAVAILABLE"; }
  try { await post("/api/connectors/line-oa/status", { state: state.state, synced: state.synced, pending: state.pending }); }
  catch { state.state = "BACKEND_UNAVAILABLE"; }
}

const server = await controlServer({ getPage: () => page, username: process.env.ADMIN_USERNAME ?? "admin", password, status: () => state, diagnostics: () => diagnostic, onOpen: open, port: Number(process.env.OA_CONTROL_PORT ?? 8082) });
console.log("OA connector control is ready; login may be required.");
const loop = async () => { if (stopping) return; await cycle(); if (!stopping) setTimeout(loop, pollMs); };
void loop();
async function stop() { if (stopping) return; stopping = true; state.state = "STOPPED"; server.close(); await context.close(); process.exit(0); }
process.on("SIGTERM", stop); process.on("SIGINT", stop);

import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";

const code = readFileSync(new URL("../public/alex-notifications-sw.js", import.meta.url), "utf8");
function worker(windows = []) {
  const handlers = {};
  const opened = [];
  const self = {
    location: { origin: "https://alex.test" },
    addEventListener: (name, handler) => { handlers[name] = handler; },
    clients: { matchAll: async () => windows, openWindow: async url => { opened.push(url); } },
    skipWaiting: async () => {},
  };
  runInNewContext(code, { self, URL });
  return { handlers, opened };
}
function windowClient(id, url = "https://alex.test/") {
  const messages = [];
  return { id, url, messages, focused: false, async focus() { this.focused = true; }, postMessage(message) { messages.push(message); } };
}
async function click(handlers, data) {
  let pending;
  let closed = false;
  handlers.notificationclick({ notification: { data, close: () => { closed = true; } }, waitUntil: value => { pending = value; } });
  await pending;
  assert.equal(closed, true);
}
const inquiryId = "111111111111111111111111";
test("notification click opens exactly the originating Alex tab", async () => {
  const other = windowClient("other"), origin = windowClient("origin");
  const { handlers, opened } = worker([other, origin]);
  await click(handlers, { inquiryId, clientId: "origin" });
  assert.equal(origin.focused, true);
  assert.equal(other.focused, false);
  assert.equal(origin.messages[0].inquiryId, inquiryId);
  assert.equal(other.messages.length, 0);
  assert.equal(opened.length, 0);
});
test("closed tab falls back to another app tab or an authenticated deep link", async () => {
  const other = windowClient("other");
  const existing = worker([windowClient("foreign", "https://other.test/"), other]);
  await click(existing.handlers, { inquiryId, clientId: "closed" });
  assert.equal(other.focused, true);
  const noApp = worker([windowClient("guide", "https://alex.test/guide")]);
  await click(noApp.handlers, { inquiryId, clientId: "closed" });
  assert.deepEqual(noApp.opened, [`/?inquiry=${inquiryId}`]);
});
test("invalid notification data cannot open URLs or tabs", async () => {
  const { handlers, opened } = worker();
  for (const id of [undefined, null, "//evil.test", "javascript:alert(1)", {}]) await click(handlers, { inquiryId: id });
  assert.deepEqual(opened, []);
});
test("client handshake returns only a same-origin window ID", () => {
  const { handlers } = worker();
  const answers = [];
  const ports = [{ postMessage: value => answers.push(value) }];
  handlers.message({ data: { type: "ALEX_CLIENT_ID" }, source: { id: "client", url: "https://alex.test/" }, ports });
  handlers.message({ data: { type: "ALEX_CLIENT_ID" }, source: { id: "foreign", url: "https://other.test/" }, ports });
  assert.deepEqual(answers, ["client"]);
});

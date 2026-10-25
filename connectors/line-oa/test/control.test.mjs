import test from "node:test";
import assert from "node:assert/strict";
import { controlServer } from "../src/control.mjs";

test("login control requires auth and CSRF header, and never accepts arbitrary navigation", async () => {
  const calls = [];
  const page = { screenshot: async () => Buffer.from("fixture"), mouse: { click: async (...args) => calls.push(args), wheel: async () => {} }, keyboard: { insertText: async () => {}, press: async () => {} } };
  const server = await controlServer({ page, username: "fixture", password: "fixture-password-123", status: () => ({ state: "NEEDS_LOGIN" }), diagnostics: () => ({}), onOpen: async () => {}, host: "127.0.0.1", port: 0 });
  const base = `http://127.0.0.1:${server.address().port}`;
  const headers = { Authorization: "Basic " + Buffer.from("fixture:fixture-password-123").toString("base64"), "Content-Type": "application/json" };
  try {
    assert.equal((await fetch(base + "/health")).status, 200);
    assert.equal((await fetch(base + "/screen")).status, 401);
    assert.equal((await fetch(base + "/action", { method: "POST", headers, body: '{"type":"click","x":1,"y":1}' })).status, 403);
    assert.equal((await fetch(base + "/action", { method: "POST", headers: { ...headers, "X-OA-Control": "1", Origin: "https://untrusted.invalid" }, body: '{"type":"click","x":1,"y":1}' })).status, 403);
    assert.equal((await fetch(base + "/action", { method: "POST", headers: { ...headers, "X-OA-Control": "1" }, body: '{"type":"goto","url":"https://untrusted.invalid"}' })).status, 400);
    assert.equal((await fetch(base + "/action", { method: "POST", headers: { ...headers, "X-OA-Control": "1" }, body: '{"type":"click","x":20,"y":30}' })).status, 200);
    assert.deepEqual(calls, [[20, 30]]);
  } finally { await new Promise(resolve => server.close(resolve)); }
});

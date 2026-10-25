import test from "node:test";
import assert from "node:assert/strict";
import { decodeMessages, makeBatches, listPage, shape } from "../src/adapter.mjs";

const at = 1_700_000_000_000;
const raw = (id, role, timestamp, text) => ({ message: { id, type: "text", text }, sender: { type: role }, timestamp });
test("exact sender roles and inbound anchors normalize without display-name guessing", () => {
  const messages = decodeMessages([raw("a", "user", at, "customer"), raw("b", "operator", at + 1, "admin")], "bot");
  assert.deepEqual(makeBatches(messages, "bot", "chat"), [{ bot_id: "bot", chat_id: "chat", anchor_message_ids: ["a"], messages: [{ id: "b", text: "admin", sent_at_ms: at + 1 }] }]);
  assert.equal(makeBatches(decodeMessages([raw("b", "operator", at, "admin")], "bot"), "bot", "chat")[0], null);
  assert.throws(() => decodeMessages([raw("b", "unknown", at, "never guess")], "bot"));
});
test("unknown timestamps, unsafe message IDs and pagination fail closed", () => {
  assert.throws(() => decodeMessages([raw(999999999999999999, "operator", at, "bad id")], "bot"));
  assert.throws(() => decodeMessages([raw("b", "operator", 1_700_000_000, "seconds are unverified")], "bot"));
  assert.throws(() => listPage({ list: [], hasMore: true }));
  assert.throws(() => listPage({ list: Array(100).fill({}) }));
  assert.deepEqual(listPage({ list: [], next: null }), { items: [], next: null });
});
test("schema diagnostics contain types, never customer values", () => {
  const result = JSON.stringify(shape({ list: [{ text: "private message", cookie: "private cookie", id: "private id" }] }));
  assert(!result.includes("private")); assert(result.includes('"text":"string"'));
});

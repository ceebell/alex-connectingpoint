import test from "node:test";
import assert from "node:assert/strict";
import { IncomingMessageTracker, notificationCopy, notificationHistory, validInquiryId } from "../src/lib/chat-notifications.ts";

const inquiry = (overrides = {}) => ({ _id: "111111111111111111111111", created_at: "2026-11-10T00:00:00Z", updated_at: "2026-11-10T00:00:00Z", status: "RECEIVED", last_incoming_message_id: "a", last_incoming_message_at: "2026-11-10T01:00:00Z", last_incoming_message_preview: "สนใจเช่าชุดค่ะ", unread: true, customer: { display_name: "ชื่อใน Alex" }, ...overrides });

test("initial load and repeat polling never notify historical messages", () => {
  const tracker = new IncomingMessageTracker();
  assert.deepEqual(tracker.observe([inquiry()]), []);
  assert.deepEqual(tracker.observe([inquiry()]), []);
  assert.deepEqual(tracker.observe([inquiry({ last_message_preview: "คุณ: ตอบแล้ว", updated_at: "2026-11-10T02:00:00Z" })]), []);
});
test("new incoming messages notify once with customer name and incoming preview", () => {
  const tracker = new IncomingMessageTracker();
  tracker.observe([inquiry()]);
  const next = inquiry({ last_incoming_message_id: "b", last_incoming_message_at: "2026-11-10T02:00:00Z", last_message_preview: "คุณ: แอดมินตอบ" });
  assert.deepEqual(tracker.observe([next]), [next]);
  assert.deepEqual(notificationCopy(next), { title: "Alex · ชื่อใน Alex", body: "สนใจเช่าชุดค่ะ" });
  assert.deepEqual(tracker.observe([next]), []);
});
test("new customers notify, but pagination and stale or withdrawn snapshots do not", () => {
  const tracker = new IncomingMessageTracker();
  tracker.observe([inquiry()]);
  const old = inquiry({ _id: "222222222222222222222222", last_incoming_message_at: "2026-11-09T01:00:00Z" });
  assert.deepEqual(tracker.observe([old]), []);
  const fresh = inquiry({ _id: "333333333333333333333333", last_incoming_message_at: "2026-11-10T03:00:00Z" });
  assert.deepEqual(tracker.observe([fresh]), [fresh]);
  assert.deepEqual(tracker.observe([inquiry({ last_incoming_message_id: "rewind", last_incoming_message_at: "2026-11-09T23:00:00Z" })]), []);
  assert.deepEqual(tracker.observe([inquiry()]), []);
});
test("read messages and read/unread toggles are not new messages", () => {
  const tracker = new IncomingMessageTracker();
  tracker.observe([inquiry()]);
  const read = inquiry({ last_incoming_message_id: "b", last_incoming_message_at: "2026-11-10T02:00:00Z", unread: false });
  assert.deepEqual(tracker.observe([read]), []);
  assert.deepEqual(tracker.observe([{ ...read, unread: true }]), []);
});
test("empty inbox can receive its first new message; malformed metadata is ignored", () => {
  const tracker = new IncomingMessageTracker();
  tracker.observe([]);
  assert.deepEqual(tracker.observe([inquiry({ last_incoming_message_at: undefined }), inquiry({ _id: "bad" })]), []);
  assert.equal(tracker.observe([inquiry()]).length, 1);
  assert.equal(validInquiryId("javascript:alert(1)"), false);
  assert.equal(validInquiryId(null), false);
});
test("notification storage is bounded and tolerates corrupt values", () => {
  for (const value of [null, "not-json", "{}", "true"]) assert.deepEqual(notificationHistory(value), []);
  assert.deepEqual(notificationHistory('[null,42,"message-id"]'), ["message-id"]);
  assert.equal(notificationHistory(JSON.stringify(Array.from({ length: 1500 }, (_, i) => `id-${i}`))).length, 1000);
  assert.equal(notificationCopy(inquiry({ last_incoming_message_preview: "x".repeat(500) })).body.length, 240);
});

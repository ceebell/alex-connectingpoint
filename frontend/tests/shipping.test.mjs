import test from "node:test";
import assert from "node:assert/strict";
import { feeSatang, normalizeTracking, suggestedChatOrder, trackingMessage, validTracking } from "../src/lib/shipping.ts";

test("receipt numbers keep every printed digit without guessing O as zero", () => {
  assert.equal(normalizeTracking("JD 3334 5879 0 TH"), "JD333458790TH");
  assert.equal(validTracking("JD 3334 5879 0 TH"), true);
  assert.equal(validTracking("JD33345879OTH"), false);
  assert.equal(validTracking("JD333458790XX"), false);
});
test("shipping charges are optional exact satang and stay separate from the order bill", () => {
  assert.equal(feeSatang("24.50"), 2450);
  assert.equal(feeSatang("24.5"), 2450);
  assert.equal(feeSatang("0"), 0);
  assert.equal(feeSatang(""), null);
  assert.equal(Number.isNaN(feeSatang("24.999")), true);
  assert.equal(Number.isNaN(feeSatang("-1")), true);
});
test("the prepared EMS message uses the requested wording and the parcel's normalized tracking", () => {
  assert.equal(trackingMessage("JD 3334 5879 0 TH"), "ส่งของเรียบร้อยค่ะ ✨ (ems)\nเลขที่ Tracking: JD333458790TH\n\nสามารถเช็คสถานะได้ที่ https://track.thailandpost.co.th");
});

test("chat name suggestions distinguish customers from multiple orders and incomplete results", () => {
  const first = { submission_id: "order-a", line_user_id: "user-a", match_score: 100, chat_customer: { display_name: "Chat customer" } };
  const second = { ...first, submission_id: "order-b" };
  assert.equal(suggestedChatOrder([first, second], false), first);
  assert.equal(suggestedChatOrder([first, { ...second, line_user_id: "user-b" }], false), undefined);
  assert.equal(suggestedChatOrder([first], true), undefined);
  assert.equal(suggestedChatOrder([{ ...first, match_score: 80 }], false), undefined);
  assert.equal(suggestedChatOrder([{ ...first, match_score: 110 }], false), undefined);
  assert.equal(suggestedChatOrder([], false), undefined);
});

test("an eligible order is not recommended when an equally matching chat has an unconfirmed bill", () => {
  const order = { submission_id: "order-a", line_user_id: "user-a", match_score: 100, chat_customer: { display_name: "Chat A" } };
  const pending = { line_user_id: "user-b", inquiry_id: "inquiry-b", match_score: 100, chat_customer: { display_name: "Chat B" }, matched_name: "Example Recipient", match_source: "alex_submission", submission_status: "SUBMIT_UNKNOWN" };
  assert.equal(suggestedChatOrder([order], false, [pending]), undefined);
  assert.equal(suggestedChatOrder([order], false, [{ ...pending, line_user_id: "user-a" }]), order);
  assert.equal(suggestedChatOrder([], false, [pending]), undefined);
});

test("a note-only name match requires review rather than an automatic order recommendation", () => {
  const order = { submission_id: "order-a", line_user_id: "user-a", match_score: 80, chat_customer: { display_name: "Chat A" } };
  const note = { line_user_id: "user-a", inquiry_id: "inquiry-a", match_score: 80, chat_customer: order.chat_customer, matched_name: "Recipient Example Recipient", match_source: "customer_note", submission_status: null };
  assert.equal(suggestedChatOrder([order], false, [note]), undefined);
});

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
test("the prepared customer message contains the selected order and normalized tracking", () => {
  const message = trackingMessage("JD 3334 5879 0 TH", "EMS", "AR-0001");
  assert.match(message, /AR-0001/); assert.match(message, /JD333458790TH/);
  assert.match(message, /EMS/); assert.match(message, /https:\/\/track\.thailandpost\.co\.th\//);
  assert.doesNotMatch(message, /undefined|null/);
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

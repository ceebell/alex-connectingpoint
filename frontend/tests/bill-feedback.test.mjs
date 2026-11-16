import test from "node:test";
import assert from "node:assert/strict";
import { submissionFeedback, isPreflightRejection, rememberBillAttempt, restoreBillAttempt, forgetBillAttempt } from "../src/lib/bill-feedback.ts";

test("success requires a confirmed order ID, not merely a completed HTTP request", () => {
  assert.equal(submissionFeedback({ _id: "request-1", status: "CONFIRMED", alex_order_id: "id-1", alex_order_number: "AR-1" }).state, "success");
  assert.equal(submissionFeedback({ _id: "request-1", status: "CONFIRMED" }).state, "unknown");
  assert.equal(submissionFeedback({ _id: "request-1", status: "SUBMIT_UNKNOWN", alex_order_number: "AR-1" }).state, "unknown");
});
test("a definite non-creation is distinct from pending and ambiguous responses", () => {
  assert.equal(submissionFeedback({ _id: "request-1", status: "NOT_CREATED" }).state, "failed");
  assert.equal(submissionFeedback({ _id: "request-1", status: "SUBMITTING" }).state, "waiting");
  const rejected = submissionFeedback({ _id: "request-1", status: "SUBMIT_UNKNOWN", http_status: 422 });
  assert.equal(rejected.state, "unknown"); assert.match(rejected.message, /422/);
  assert.equal(isPreflightRejection("QUEUE_BLOCKED"), true);
  assert.equal(isPreflightRejection("DATABASE_UNAVAILABLE"), false);
});
test("both references survive feedback and a missing number never invites another creation", () => {
  const complete = submissionFeedback({ _id: "request-1", status: "CONFIRMED", alex_order_id: "id-1", alex_order_number: "AR-1" });
  assert.equal(complete.orderId, "id-1"); assert.equal(complete.orderNumber, "AR-1"); assert.equal(complete.requestId, "request-1");
  const partial = submissionFeedback({ _id: "request-1", status: "CONFIRMED", alex_order_id: "id-1", alex_order_number: null });
  assert.equal(partial.state, "success"); assert.equal(partial.orderId, "id-1"); assert.equal(partial.orderNumber, null);
  assert.match(partial.message, /ยังไม่ได้รับ Order Number/); assert.match(partial.message, /ไม่ส่งออเดอร์ซ้ำ/);
});
test("an interrupted request survives reload without storing customer data", () => {
  const storage = new Map(); globalThis.sessionStorage = { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, value), removeItem: key => storage.delete(key) };
  const attempt = { inquiryId: "customer-1", reviewId: "original-request", draftVersion: 2, acceptQueue: false };
  rememberBillAttempt(attempt);
  assert.deepEqual(restoreBillAttempt("customer-1"), attempt); assert.equal(restoreBillAttempt("customer-2"), null);
  forgetBillAttempt("customer-1"); assert.equal(restoreBillAttempt("customer-1"), null);
  storage.set("alex:bill-attempt:customer-1", '{"inquiryId":"another-customer"}'); assert.equal(restoreBillAttempt("customer-1"), null);
  globalThis.sessionStorage = { getItem: () => { throw Error("Storage denied"); } };
  assert.equal(restoreBillAttempt("customer-1"), null);
});

import test from "node:test";
import assert from "node:assert/strict";
import { matchingQuickReplies, slashQuery } from "../src/lib/quick-replies.ts";
const items = [
  { _id: "1", kind: "text", title: "ต้อนรับ", text: "สวัสดีค่ะ ยินดีต้อนรับสู่ Arisa Rental ค่ะ" },
  { _id: "2", kind: "text", title: "สวัสดี", text: "สวัสดีค่ะ ยินดีให้บริการค่ะ" },
  { _id: "3", kind: "image", title: "สวัสดีการ์ด", text: "" },
  { _id: "4", kind: "text", title: "จัดส่ง", text: "ร้านจัดส่งด้วย Flash ค่ะ" },
];
test("slash shortcuts only activate at the beginning of a single-line draft", () => {
  assert.equal(slashQuery("/"), ""); assert.equal(slashQuery("/สวัสดี "), "สวัสดี");
  for (const value of ["https://example.com/", "ขอ / จัดส่ง", " /สวัสดี", "สวัสดี", "/สวัสดี\nลูกค้า"]) assert.equal(slashQuery(value), null);
});
test("suggestions match Thai content as well as titles and prioritize a matching shortcut", () => {
  assert.deepEqual(matchingQuickReplies(items, "สวัสดี", "text").map(item => item._id), ["2", "1"]);
  assert.deepEqual(matchingQuickReplies(items, "ARISA rental", "text").map(item => item._id), ["1"]);
  assert.deepEqual(matchingQuickReplies(items, "ยินดี ต้อนรับ", "text").map(item => item._id), ["1"]);
  assert.deepEqual(matchingQuickReplies(items, "ไม่ตรง", "text"), []);
  assert.equal(matchingQuickReplies(items, "", "text").length, 3);
});
test("image cards stay out of slash suggestions and source text is never interpreted", () => {
  assert.equal(matchingQuickReplies(items, "สวัสดี", "image")[0]._id, "3");
  const literal = { _id: "5", kind: "text", title: "html", text: "<script>alert(1)</script>" };
  assert.equal(matchingQuickReplies([literal], "script", "text")[0].text, literal.text);
  assert.equal(items[0].title, "ต้อนรับ");
});

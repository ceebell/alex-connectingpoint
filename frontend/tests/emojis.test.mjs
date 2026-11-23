import test from "node:test";
import assert from "node:assert/strict";
import { emojiImageUrl, insertEmojiText, lineEmojiImageUrl, lineEmojiParts, unicodeEmojiParts } from "../src/lib/emojis.ts";

test("emoji insertion respects the caret and replaces only the selected text", () => {
  assert.deepEqual(insertEmojiText("สวัสดีค่ะ", "😊", 6, 6), { text: "สวัสดี😊ค่ะ", caret: 8 });
  assert.deepEqual(insertEmojiText("Hello old world", "❤️", 6, 9), { text: "Hello ❤️ world", caret: 8 });
  const family = "👩🏽‍👩🏻‍👧🏾";
  assert.deepEqual(insertEmojiText("AB", family, 1, 1), { text: `A${family}B`, caret: 1 + family.length });
});

const productId = "5ac1bfd5040ab15980c9b435";
test("LINE emoji replaces only the supplied UTF-16 spans and preserves all original text", () => {
  const text = "😊 สวัสดี (love)\n(hello) จบ";
  const emojis = [
    { index: text.indexOf("(hello)"), length: 7, productId, emojiId: "002" },
    { index: text.indexOf("(love)"), length: 6, productId, emojiId: "001" },
  ];
  const parts = lineEmojiParts(text, emojis);
  assert.equal(parts.map(part => part.text).join(""), text);
  assert.deepEqual(parts.filter(part => part.emojiUrl).map(part => part.text), ["(love)", "(hello)"]);
  assert.equal(parts[1].emojiUrl, `https://stickershop.line-scdn.net/sticonshop/v1/sticon/${productId}/android/001.png`);
  assert.equal(emojis[0].emojiId, "002", "Metadata order is not mutated");
  assert.deepEqual(lineEmojiParts("A$B", [{ index: 1, productId, emojiId: "001" }]).map(part => part.text), ["A", "$", "B"]);
});

test("malformed, overlapping or surrogate-splitting emoji metadata never removes text", () => {
  const text = "😊(love)";
  for (const metadata of [undefined, null, {}, [null], [1], [
    { index: 1, length: 1, productId, emojiId: "001" },
    { index: 0, length: 1, productId, emojiId: "001" },
    { index: -1, length: 1, productId, emojiId: "001" },
    { index: 2.5, length: 2, productId, emojiId: "001" },
    { index: { toString: null }, productId, emojiId: "001" },
    { index: 2, length: 500, productId, emojiId: "001" },
    { index: 2, length: "6", productId, emojiId: "001" },
    { index: 2, length: 0, productId, emojiId: "001" },
  ]]) assert.deepEqual(lineEmojiParts(text, metadata), [{ text }]);
  const overlap = lineEmojiParts(text, [
    { index: 2, length: 6, productId, emojiId: "001" },
    { index: 2, length: 6, productId, emojiId: "002" },
  ]);
  assert.equal(overlap.map(part => part.text).join(""), text);
  assert.equal(overlap.filter(part => part.emojiUrl).length, 1);
});

test("LINE emoji URLs accept only catalog IDs on the fixed LINE CDN origin", () => {
  assert(lineEmojiImageUrl(productId.toUpperCase(), "001").includes(productId));
  for (const [product, emoji] of [["../secret", "001"], [productId, "../1"], [productId, "1?x"], ["https://evil.test", "001"], [null, "001"], [productId, null]]) {
    assert.equal(lineEmojiImageUrl(product, emoji), null);
  }
});

test("Unicode emoji previews keep flags, keycaps, tones and joined families intact", () => {
  const samples = ["😊", "❤️", "🇹🇭", "1️⃣", "👍🏽", "👩🏽‍👩🏻‍👧🏾"];
  const text = `ข้อความ\n${samples.join(" ")} & <script>คำไทย</script>`;
  const parts = unicodeEmojiParts(text);
  assert.equal(parts.map(part => part.text).join(""), text);
  assert.deepEqual(parts.filter(part => part.emojiUrl).map(part => part.text), samples);
  assert.deepEqual(unicodeEmojiParts("ราคา 1,234 # © ® ™ กำลังทดสอบ"), [{ text: "ราคา 1,234 # © ® ™ กำลังทดสอบ" }]);
  assert.deepEqual(unicodeEmojiParts("🇹"), [{ text: "🇹" }]);
});

test("emoji insertion preserves the draft when the text limit would be exceeded", () => {
  assert.equal(insertEmojiText("a".repeat(4999), "😊", 4999, 4999), null);
  const result = insertEmojiText("a".repeat(5000), "😊", 4998, 5000);
  assert.equal(result.text.length, 5000);
  assert.equal(result.caret, 5000);
});

test("emoji image paths cover keycaps, variation selectors, flags and joined sequences", () => {
  for (const [unified, filename] of [["0031-fe0f-20e3", "31-20e3"], ["00a9-fe0f", "a9"], ["2764-fe0f", "2764"], ["1f1f9-1f1ed", "1f1f9-1f1ed"], ["1f3f3-fe0f-200d-1f308", "1f3f3-fe0f-200d-1f308"], ["1f441-fe0f-200d-1f5e8-fe0f", "1f441-200d-1f5e8"]]) {
    assert.equal(emojiImageUrl(unified), `https://cdn.jsdelivr.net/gh/jdecked/twemoji@17.0.3/assets/svg/${filename}.svg`);
  }
});

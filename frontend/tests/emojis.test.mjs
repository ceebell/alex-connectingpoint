import test from "node:test";
import assert from "node:assert/strict";
import { emojiImageUrl, insertEmojiText } from "../src/lib/emojis.ts";

test("emoji insertion respects the caret and replaces only the selected text", () => {
  assert.deepEqual(insertEmojiText("สวัสดีค่ะ", "😊", 6, 6), { text: "สวัสดี😊ค่ะ", caret: 8 });
  assert.deepEqual(insertEmojiText("Hello old world", "❤️", 6, 9), { text: "Hello ❤️ world", caret: 8 });
  const family = "👩🏽‍👩🏻‍👧🏾";
  assert.deepEqual(insertEmojiText("AB", family, 1, 1), { text: `A${family}B`, caret: 1 + family.length });
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

import test from "node:test";
import assert from "node:assert/strict";
import { clipboardImages } from "../src/lib/reply-images.ts";

const item = file => ({ kind: "file", type: file?.type ?? "", getAsFile: () => file });

test("image paste reads files without duplicating clipboard items or pasting accompanying HTML", () => {
  const image = new File(["image"], "image.png", { type: "image/png" });
  const text = { kind: "string", type: "text/html", getAsFile: () => { throw new Error("Text is not a file"); } };
  assert.deepEqual(clipboardImages({ files: [image], items: [item(image), text] }), [image]);
  assert.deepEqual(clipboardImages({ files: [], items: [text, item(image)] }), [image]);
});

test("ordinary text, HTML and non-image files retain native paste behavior", () => {
  const pdf = new File(["document"], "document.pdf", { type: "application/pdf" });
  assert.deepEqual(clipboardImages({ files: [pdf], items: [item(pdf), item(null)] }), []);
  assert.deepEqual(clipboardImages({ files: [], items: [{ kind: "string", type: "text/plain" }] }), []);
  assert.deepEqual(clipboardImages({ files: [], items: [{ kind: "string", type: "text/html" }] }), []);
});

test("multiple images and unsupported image types remain available for validation", () => {
  const png = new File(["png"], "image.png", { type: "image/png" });
  const svg = new File(["svg"], "image.svg", { type: "image/svg+xml" });
  assert.deepEqual(clipboardImages({ files: [png, svg], items: [] }), [png, svg]);
  assert.deepEqual(clipboardImages({ files: [], items: [item(png), item(svg)] }), [png, svg]);
});

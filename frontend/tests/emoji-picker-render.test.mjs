import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { runInNewContext } from "node:vm";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import ts from "typescript";

test("chat refreshes and emoji selections reuse the loaded emoji inventory", async () => {
  const require = createRequire(import.meta.url);
  const renders = [];
  const compiledModule = { exports: {} };
  const { outputText } = ts.transpileModule(
    readFileSync(new URL("../src/components/chat-emoji-picker.tsx", import.meta.url), "utf8"),
    { compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX } },
  );
  runInNewContext(outputText, {
    module: compiledModule,
    exports: compiledModule.exports,
    window: { matchMedia: () => ({ matches: false }) },
    require: name => {
      if (name === "emoji-picker-react") return { default: props => { renders.push(props); return null; } };
      if (name === "@/lib/emojis") return { emojiImageUrl: () => "" };
      return require(name);
    },
  });
  const selections = [];
  const Picker = compiledModule.exports.default;
  renderToStaticMarkup(createElement(Picker, { onSelect: emoji => selections.push(["initial", emoji]) }));
  renderToStaticMarkup(createElement(Picker, { onSelect: emoji => selections.push(["refresh", emoji]) }));
  renders[1].onEmojiClick({ emoji: "😊" });
  renderToStaticMarkup(createElement(Picker, { onSelect: emoji => selections.push(["after-selection", emoji]) }));

  // A changed loader makes emoji-picker-react replace its inventory with an empty
  // loading state, which collapses the viewport and resets scrollTop to zero.
  assert.equal(renders[1].emojiData, renders[0].emojiData);
  assert.equal(renders[2].emojiData, renders[0].emojiData);
  const dataset = await renders[0].emojiData();
  assert(Object.values(dataset.default.emojis).some(group => group.length > 0));
  renders[2].onEmojiClick({ emoji: "❤️" });
  assert.deepEqual(selections, [["refresh", "😊"], ["after-selection", "❤️"]]);
});

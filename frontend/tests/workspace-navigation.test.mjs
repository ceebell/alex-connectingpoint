import test from "node:test";
import assert from "node:assert/strict";
import { workspaceRoute } from "../src/lib/workspace-navigation.ts";

const id = "0123456789abcdef01234567";
test("top-level destinations do not retain a previous conversation", () => {
  for (const page of ["home", "chats", "team", "settings"]) {
    assert.deepEqual(workspaceRoute(`#${page}`, `?inquiry=${id}`), { page, inquiryId: null, panel: "inquiries" });
  }
  assert.equal(workspaceRoute("").page, "home");
});
test("conversation links identify both customer and chat or order panel", () => {
  assert.deepEqual(workspaceRoute(`#chat/${id}`), { page: "chats", inquiryId: id, panel: "chat" });
  assert.deepEqual(workspaceRoute(`#order/${id}`), { page: "chats", inquiryId: id, panel: "draft" });
  assert.equal(workspaceRoute(`#chat/${id.toUpperCase()}`).inquiryId, id);
});
test("existing notification links still open the intended customer", () => {
  assert.deepEqual(workspaceRoute("", `?inquiry=${id}`), { page: "chats", inquiryId: id, panel: "chat" });
  assert.equal(workspaceRoute("", "?inquiry=invalid").inquiryId, null);
});
test("malformed and invitation fragments cannot become a customer identifier", () => {
  for (const hash of ["#chat/invalid", `#chat/${id}/extra`, "#order/../../team", "#invite=private-invitation"]) {
    assert.deepEqual(workspaceRoute(hash), { page: "home", inquiryId: null, panel: "inquiries" });
  }
});

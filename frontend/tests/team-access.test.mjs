import test from "node:test";
import assert from "node:assert/strict";
import { canManageTeam, canSubmitOrder, canUseWorkspace, invitationToken, invitationUrl } from "../src/lib/team-api.ts";

const session = (role, status = "ACTIVE") => ({ username: "fixture", login: "line", hermes_configured: false, team_enabled: true, membership: { role, status }, capabilities: { manage_team: role === "OWNER", submit_order: role !== "STAFF" } });
test("active staff can prepare work but cannot manage people or submit orders", () => {
  const staff = session("STAFF");
  assert.equal(canUseWorkspace(staff), true);
  assert.equal(canManageTeam(staff), false);
  assert.equal(canSubmitOrder(staff), false);
});
test("managers submit orders and only owners see team management", () => {
  assert.equal(canSubmitOrder(session("MANAGER")), true);
  assert.equal(canManageTeam(session("MANAGER")), false);
  assert.equal(canManageTeam(session("OWNER")), true);
});
test("pending and suspended users cannot enter the workspace despite cached capabilities", () => {
  for (const role of ["OWNER", "MANAGER", "STAFF"]) for (const status of ["PENDING", "SUSPENDED", "REJECTED"]) {
    assert.equal(canUseWorkspace(session(role, status)), false);
    assert.equal(canManageTeam(session(role, status)), false);
    assert.equal(canSubmitOrder(session(role, status)), false);
  }
  assert.equal(canUseWorkspace({ ...session("STAFF"), membership: null }), false);
});
test("legacy deployments retain their existing workspace behavior", () => {
  assert.equal(canUseWorkspace({ username: "legacy" }), true);
  assert.equal(canSubmitOrder({ username: "legacy" }), true);
  assert.equal(canManageTeam({ username: "legacy" }), false);
});
test("invitation secret appears only in a URL fragment", () => {
  const token = "a".repeat(43);
  const link = new URL(invitationUrl("https://alex.example/path?old=true", token));
  assert.equal(link.origin, "https://alex.example");
  assert.equal(link.pathname, "/");
  assert.equal(link.search, "");
  assert.equal(invitationToken(link.hash), token);
  assert.equal(new URL(link.href.split("#")[0]).href.includes(token), false);
});
test("malformed fragments never become invitation contexts", () => {
  for (const token of ["bad", "javascript:alert(1)", "//evil.example", "a".repeat(44), "a".repeat(42) + "/"]) {
    assert.equal(invitationToken(`#invite=${encodeURIComponent(token)}`), null);
    assert.throws(() => invitationUrl("https://alex.example", token));
  }
  assert.equal(invitationToken("#team"), null);
});

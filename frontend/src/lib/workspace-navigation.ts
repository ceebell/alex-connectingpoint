export type WorkspacePage = "home" | "chats" | "team" | "settings" | "shipping";
export type ConversationPanel = "inquiries" | "chat" | "draft";
export type WorkspaceRoute = { page: WorkspacePage; inquiryId: string | null; panel: ConversationPanel };

export function workspaceRoute(hash: string, search = ""): WorkspaceRoute {
  const conversation = /^#(chat|order)\/([a-f0-9]{24})$/i.exec(hash);
  if (conversation) return { page: "chats", inquiryId: conversation[2].toLowerCase(), panel: conversation[1].toLowerCase() === "order" ? "draft" : "chat" };
  if (hash === "#team") return { page: "team", inquiryId: null, panel: "inquiries" };
  if (hash === "#settings") return { page: "settings", inquiryId: null, panel: "inquiries" };
  if (hash === "#shipping") return { page: "shipping", inquiryId: null, panel: "inquiries" };
  if (hash === "#chats") return { page: "chats", inquiryId: null, panel: "inquiries" };
  const notificationId = new URLSearchParams(search).get("inquiry");
  if (!hash && notificationId && /^[a-f0-9]{24}$/i.test(notificationId)) return { page: "chats", inquiryId: notificationId.toLowerCase(), panel: "chat" };
  return { page: "home", inquiryId: null, panel: "inquiries" };
}

export function navigateWorkspace(page: WorkspacePage, inquiryId?: string, panel: "chat" | "draft" = "chat") {
  window.location.hash = page === "chats" && inquiryId ? `${panel === "draft" ? "order" : "chat"}/${inquiryId}` : page;
}

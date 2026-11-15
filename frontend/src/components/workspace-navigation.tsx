"use client";
import { navigateWorkspace, type WorkspacePage } from "@/lib/workspace-navigation";

export function WorkspaceIcon({ name }: { name: "home" | "chats" | "team" | "order" | "back" | "settings" }) {
  return <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false">
    {name === "home" && <><path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1Z" /><path d="M9 21v-7h6v7" /></>}
    {name === "chats" && <><path d="M21 11.5a8.5 8.5 0 0 1-8.5 8.5 9 9 0 0 1-3.8-.9L3 21l1.9-5.7a8.5 8.5 0 1 1 16.1-3.8Z" /><path d="M8 11.5h.01M12 11.5h.01M16 11.5h.01" /></>}
    {name === "team" && <><circle cx="9" cy="8" r="3" /><path d="M3 21v-2a6 6 0 0 1 12 0v2M16 5a3 3 0 0 1 0 6m3 10v-2a6 6 0 0 0-3-5" /></>}
    {name === "order" && <><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z" /><path d="M14 2v6h6M8 13h8M8 17h6" /></>}
    {name === "back" && <path d="m14 5-7 7 7 7" />}
    {name === "settings" && <><path d="m9 3-1 3-3 1-2 5 2 5 3 1 1 3h6l1-3 3-1 2-5-2-5-3-1-1-3Z" /><circle cx="12" cy="12" r="3" /></>}
  </svg>;
}

export default function WorkspaceNavigation({ activePage, manageTeam }: { activePage: WorkspacePage; manageTeam: boolean }) {
  return <nav className="mobile-workspace-nav" aria-label="เมนูหลักบนมือถือ">
    <button type="button" aria-current={activePage === "home" ? "page" : undefined} onClick={() => navigateWorkspace("home")}><WorkspaceIcon name="home" /><span>หน้าหลัก</span></button>
    <button type="button" aria-current={activePage === "chats" ? "page" : undefined} onClick={() => navigateWorkspace("chats")}><WorkspaceIcon name="chats" /><span>รายการแชท</span></button>
    <button type="button" aria-current={activePage === "team" ? "page" : undefined} disabled={!manageTeam} title={manageTeam ? "จัดการทีม" : "เฉพาะเจ้าของร้าน"} aria-label={manageTeam ? "จัดการทีม" : "จัดการทีม (เฉพาะเจ้าของร้าน)"} onClick={() => navigateWorkspace("team")}><WorkspaceIcon name="team" /><span>ทีม</span></button>
  </nav>;
}

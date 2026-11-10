"use client";

import { useEffect, useRef, useState } from "react";
import type { FocusEvent, KeyboardEvent, PointerEvent as ReactPointerEvent } from "react";

type Panel = "inquiries" | "chat" | "draft";
type Menu = "chat" | "info" | "account" | "settings";
type Icon = Panel | "info" | "account" | "logout" | "settings";
type Props = {
  inquiryCount?: number;
  username: string;
  login: "line" | "password" | "token";
  activePanel: Panel;
  onNavigate: (panel: Panel) => void;
  onLogout: () => Promise<void>;
  onManageTeam?: () => void;
  teamActive?: boolean;
};

function RailIcon({ name }: { name: Icon }) {
  return <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false">
    {name === "chat" && <><path d="M21 11.5a8.3 8.3 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.3 8.3 0 0 1-3.8-.9L3 21l1.9-5.7a8.3 8.3 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.3 8.3 0 0 1 3.8-.9h.5a8.5 8.5 0 0 1 8 8v.5Z" /><path d="M8 9h8M8 13h5" /></>}
    {name === "inquiries" && <><rect x="4" y="3" width="16" height="18" rx="2" /><path d="M8 8h8M8 12h8M8 16h5" /></>}
    {name === "draft" && <><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z" /><path d="M14 2v6h6M8 13h8M8 17h5" /></>}
    {name === "info" && <><circle cx="12" cy="12" r="9" /><path d="M12 11v6M12 7h.01" /></>}
    {name === "account" && <><circle cx="12" cy="12" r="9" /><circle cx="12" cy="9" r="3" /><path d="M6 19a6 6 0 0 1 12 0" /></>}
    {name === "logout" && <><path d="M9 4H5a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h4M14 8l4 4-4 4M8 12h13" /></>}
    {name === "settings" && <><path d="m9 3-1 3-3 1-2 5 2 5 3 1 1 3h6l1-3 3-1 2-5-2-5-3-1-1-3Z" /><circle cx="12" cy="12" r="3" /></>}
  </svg>;
}

const destinations: { panel: Panel; label: string }[] = [
  { panel: "inquiries", label: "รายชื่อลูกค้า" },
  { panel: "chat", label: "บทสนทนา" },
  { panel: "draft", label: "ร่างออเดอร์" },
];

export default function WorkspaceSidebar({ inquiryCount, username, login, activePanel, onNavigate, onLogout, onManageTeam, teamActive }: Props) {
  const [open, setOpen] = useState<Menu | null>(null);
  const sidebar = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!open) return;
    const dismiss = (event: PointerEvent) => {
      if (event.target instanceof Node && !sidebar.current?.contains(event.target)) setOpen(null);
    };
    const resize = () => setOpen(null);
    document.addEventListener("pointerdown", dismiss);
    window.addEventListener("resize", resize);
    return () => {
      document.removeEventListener("pointerdown", dismiss);
      window.removeEventListener("resize", resize);
    };
  }, [open]);

  function groupEvents(menu: Menu) {
    return {
      onPointerEnter: (event: ReactPointerEvent<HTMLDivElement>) => {
        if (event.pointerType === "mouse") setOpen(menu);
      },
      onPointerLeave: (event: ReactPointerEvent<HTMLDivElement>) => {
        if (event.pointerType === "mouse" && !event.currentTarget.contains(document.activeElement)) setOpen(null);
      },
      onFocus: () => setOpen(menu),
      onBlur: (event: FocusEvent<HTMLDivElement>) => {
        if (!(event.relatedTarget instanceof Node) || !event.currentTarget.contains(event.relatedTarget)) setOpen(null);
      },
      onKeyDown: (event: KeyboardEvent<HTMLDivElement>) => {
        const group = event.currentTarget;
        if (event.key === "Escape") {
          event.preventDefault();
          event.stopPropagation();
          group.querySelector<HTMLButtonElement>(".rail-button")?.focus();
          setOpen(null);
        } else if (["ArrowRight", "ArrowDown"].includes(event.key) && (event.target as HTMLElement).classList.contains("rail-button") && menu !== "info") {
          event.preventDefault();
          setOpen(menu);
          requestAnimationFrame(() => group.querySelector<HTMLButtonElement>(".rail-menu-item")?.focus());
        }
      },
    };
  }

  return <aside className="sidebar sidebar-rail" ref={sidebar} aria-label="แถบเมนูหลัก">
    <div className="rail-brand" role="img" aria-label="Alex"><div className="brand-mark" aria-hidden="true">a<span>•</span></div></div>
    <nav className="rail-navigation" aria-label="พื้นที่ทำงาน">
      <div className="rail-group" {...groupEvents("chat")}>
        <button type="button" className={`rail-button ${teamActive ? "" : "is-active"}`} aria-label={inquiryCount === undefined ? "แชทและ inquiry" : `แชทและ inquiry ${inquiryCount} เรื่อง`} aria-expanded={open === "chat"} aria-controls="rail-chat-menu" onClick={() => setOpen("chat")}>
          <RailIcon name="chat" />{inquiryCount !== undefined && <span className="rail-count" aria-hidden="true">{inquiryCount > 99 ? "99+" : inquiryCount}</span>}
        </button>
        {open === "chat" && <div className="rail-flyout" id="rail-chat-menu" role="region" aria-labelledby="rail-chat-heading">
          <h2 id="rail-chat-heading">แชทและ inquiry</h2>{inquiryCount !== undefined && <p className="rail-meta">เรื่องที่รับเข้า {inquiryCount} รายการ</p>}
          <div className="rail-menu">{destinations.map(({ panel, label }) => <button key={panel} type="button" className="rail-menu-item" aria-current={activePanel === panel ? "location" : undefined} onClick={() => { setOpen(null); onNavigate(panel); }}><RailIcon name={panel} /><span>{label}</span></button>)}</div>
        </div>}
      </div>
    </nav>
    <div className="rail-utilities">
      {onManageTeam && <div className="rail-group rail-group-bottom" {...groupEvents("settings")}><button type="button" className={`rail-button ${teamActive ? "is-active" : ""}`} aria-label="ตั้งค่าร้าน" aria-expanded={open === "settings"} aria-controls="rail-settings-menu" onClick={() => setOpen("settings")}><RailIcon name="settings" /></button>{open === "settings" && <div className="rail-flyout" id="rail-settings-menu" role="region" aria-labelledby="rail-settings-heading"><h2 id="rail-settings-heading">ตั้งค่าร้าน</h2><button type="button" className="rail-menu-item" aria-current={teamActive ? "page" : undefined} onClick={() => { setOpen(null); onManageTeam(); }}><RailIcon name="account" /><span>จัดการทีม</span></button></div>}</div>}
      <div className="rail-group rail-group-bottom" {...groupEvents("info")}>
        <button type="button" className="rail-button" aria-label="ข้อมูลพื้นที่ทำงาน" aria-expanded={open === "info"} aria-controls="rail-info-menu" onClick={() => setOpen("info")}><RailIcon name="info" /></button>
        {open === "info" && <div className="rail-flyout" id="rail-info-menu" role="region" aria-labelledby="rail-info-heading"><h2 id="rail-info-heading">ข้อมูลพื้นที่ทำงาน</h2><p className="rail-meta">รับเรื่องจาก LINE OA</p><p className="rail-meta">อ่านแชท เตรียมร่าง แล้วตรวจสอบก่อนส่งต่อ</p></div>}
      </div>
      <div className="rail-group rail-group-bottom" {...groupEvents("account")}>
        <button type="button" className="rail-button" aria-label={`บัญชีผู้ใช้ ${username}`} aria-expanded={open === "account"} aria-controls="rail-account-menu" onClick={() => setOpen("account")}><RailIcon name="account" /></button>
        {open === "account" && <div className="rail-flyout" id="rail-account-menu" role="region" aria-labelledby="rail-account-heading"><h2 id="rail-account-heading">บัญชีผู้ใช้</h2><strong className="rail-account-name">{username}</strong><p className="rail-meta">{login === "line" ? "เข้าด้วย LINE" : "ผู้ดูแลหลังบ้าน"}</p><button type="button" className="rail-menu-item rail-logout" onClick={() => { setOpen(null); void onLogout(); }}><RailIcon name="logout" /><span>ออกจากระบบ</span></button></div>}
      </div>
    </div>
  </aside>;
}

"use client";
import type { Inquiry } from "@/lib/api";
import { AdminSession, canManageTeam, teamRoles } from "@/lib/team-api";
import { formatDateTime } from "@/lib/dates";
import { navigateWorkspace } from "@/lib/workspace-navigation";
import { CustomerAvatar } from "./chat-message";
import { WorkspaceIcon } from "./workspace-navigation";

type Props = { inquiries: Inquiry[]; session: AdminSession; loading: boolean; notifications: { enabled: boolean; busy: boolean; toggle: () => Promise<void> } };

export default function ShopDashboard({ inquiries, session, loading, notifications }: Props) {
  const unread = inquiries.filter(inquiry => inquiry.unread).length;
  const drafts = inquiries.filter(inquiry => (inquiry.draft_version ?? 0) > 0 && !inquiry.order_id).length;
  return <section className="shop-dashboard" aria-label="ภาพรวมร้าน Arisa Rental" aria-busy={loading}>
    <div className="dashboard-welcome"><span className="dashboard-shop-icon"><WorkspaceIcon name="home" /></span><div><p className="dashboard-eyebrow">ภาพรวมร้าน</p><h2>Arisa Rental</h2><p>สวัสดี {session.username}</p></div>{canManageTeam(session) && <button type="button" className="dashboard-settings" aria-label="ตั้งค่าร้าน" title="ตั้งค่าร้าน" onClick={() => navigateWorkspace("settings")}><WorkspaceIcon name="settings" /></button>}</div>
    <div className="dashboard-stats">
      <article><span>แชทที่โหลดแล้ว</span><strong>{inquiries.length.toLocaleString("th-TH")}</strong></article>
      <article><span>ยังไม่ได้อ่าน</span><strong>{unread.toLocaleString("th-TH")}</strong></article>
      <article><span>มีร่างออเดอร์</span><strong>{drafts.toLocaleString("th-TH")}</strong></article>
    </div>
    <p className="dashboard-description">สรุปจากรายการแชทที่โหลดอยู่ในขณะนี้</p>
    <button type="button" className="button secondary dashboard-shipping" onClick={() => navigateWorkspace("shipping")}><WorkspaceIcon name="shipping" />พัสดุและ Tracking · อัปโหลดใบเสร็จขนส่ง</button>
    <div className="dashboard-actions"><button className="button primary" onClick={() => navigateWorkspace("chats")}><WorkspaceIcon name="chats" />เปิดรายการแชท</button>{canManageTeam(session) && <button className="button secondary" onClick={() => navigateWorkspace("team")}><WorkspaceIcon name="team" />จัดการทีม</button>}</div>
    <section className="dashboard-recent" aria-label="แชทล่าสุด"><div className="dashboard-section-heading"><h3>แชทล่าสุด</h3><button type="button" className="text-button" onClick={() => navigateWorkspace("chats")}>ดูทั้งหมด</button></div>
      {!inquiries.length ? <p className="dashboard-description">{loading ? "กำลังโหลดรายการแชท…" : "ยังไม่มีรายการแชท"}</p> : inquiries.slice(0, 5).map(inquiry => <button type="button" className="dashboard-chat" key={inquiry._id} onClick={() => navigateWorkspace("chats", inquiry._id)}><CustomerAvatar customer={inquiry.customer} /><span><strong>{inquiry.customer?.display_name || "ลูกค้า LINE"}</strong><span className="dashboard-message">{inquiry.last_message_preview || "ยังไม่มีข้อความ"}</span><time>{formatDateTime(inquiry.last_activity_at ?? inquiry.created_at)}</time></span>{inquiry.unread && <span className="dashboard-unread"><span className="sr-only">ยังไม่ได้อ่าน</span></span>}</button>)}
    </section>
    <section className="dashboard-account" aria-label="บัญชีที่ใช้งาน"><div><h3>บัญชีที่ใช้งาน</h3><p>{session.username}{session.membership && ` · ${teamRoles[session.membership.role]}`}</p>{session.membership?.branches.length ? <p className="dashboard-description">{session.membership.branches.map(branch => branch.name).join(", ")}</p> : null}</div><button type="button" className="button secondary" disabled={notifications.busy} aria-pressed={notifications.enabled} onClick={() => void notifications.toggle()}>{notifications.enabled ? "ปิดแจ้งเตือนข้อความ" : "เปิดแจ้งเตือนข้อความ"}</button></section>
  </section>;
}

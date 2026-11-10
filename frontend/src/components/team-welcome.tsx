"use client";
/* eslint-disable @next/next/no-img-element -- LINE profile images are external identity avatars. */
import { AdminSession, canUseWorkspace, TeamInvitation, teamRoles } from "@/lib/team-api";
import { formatDateTime } from "@/lib/dates";

type Props = { session: AdminSession | null; invitation: TeamInvitation | null; error: string; busy: boolean; onAccept: () => Promise<void>; onClear: () => Promise<void>; onRefresh: () => Promise<void>; onLogout: (switchAccount?: boolean) => Promise<void> };
export default function TeamWelcome({ session, invitation, error, busy, onAccept, onClear, onRefresh, onLogout }: Props) {
  const member = session?.membership;
  const active = !!session && canUseWorkspace(session);
  const awaiting = member?.status === "PENDING";
  const title = invitation ? invitation.status === "INVITED" ? "คุณได้รับคำเชิญเข้าร่วมทีม" : "คำเชิญนี้ใช้ต่อไม่ได้แล้ว" : awaiting ? "รอเจ้าของร้านอนุมัติ" : member?.status === "SUSPENDED" ? "บัญชีนี้ถูกระงับสิทธิ์" : "บัญชียังไม่มีสิทธิ์เข้าร้าน";
  const role = invitation?.role ?? member?.role;
  const branches = invitation?.branches ?? member?.branches ?? [];
  const picture = session?.line_profile?.picture_url || member?.picture_url;
  const lineName = session?.line_profile?.name || member?.line_name || session?.username;
  return <main className="team-welcome"><section className="team-welcome-card" aria-labelledby="team-welcome-title">
    <div className="team-brand">alex<span>Arisa Rental</span></div>
    <h1 id="team-welcome-title">{title}</h1>
    <p className="muted">{invitation?.status === "INVITED" ? `คำเชิญสำหรับ ${invitation.name}` : awaiting ? "ยืนยันบัญชี LINE แล้ว คุณจะเริ่มใช้งานได้เมื่อเจ้าของร้านอนุมัติ" : "ติดต่อเจ้าของร้านเพื่อตรวจสถานะหรือขอคำเชิญใหม่"}</p>
    {session && <div className="team-identity">{picture ? <img src={picture} alt="" referrerPolicy="no-referrer" /> : <span className="team-avatar" aria-hidden="true">{lineName?.slice(0, 1)}</span>}<div><strong>{lineName}</strong><p className="muted small">บัญชี LINE ที่กำลังใช้</p></div></div>}
    {role && <dl className="team-invite-summary"><div><dt>ร้าน</dt><dd>{invitation?.shop_name || "Arisa Rental"}</dd></div><div><dt>ชื่อที่ใช้ทำงาน</dt><dd>{invitation?.name || member?.name}</dd></div><div><dt>บทบาท</dt><dd>{teamRoles[role]}</dd></div><div><dt>สาขา</dt><dd>{branches.map(branch => branch.name).join(", ") || (role === "OWNER" ? "ทุกสาขาที่ร้านมีสิทธิ์" : "ยังไม่มีสาขาสินค้า")}</dd></div>{invitation?.invited_by && <div><dt>ผู้เชิญ</dt><dd>{invitation.invited_by}</dd></div>}{invitation && <div><dt>รับคำเชิญภายใน</dt><dd>{formatDateTime(invitation.expires_at)}</dd></div>}</dl>}
    {error && <p className="notice warning" role="alert">{error}</p>}
    <div className="team-welcome-actions">
      {invitation?.status === "INVITED" && !session && <a className="button primary full" href="/api/admin/line/start">ยืนยันตัวตนด้วย LINE</a>}
      {invitation?.status === "INVITED" && session && !active && (!member || member.status === "REJECTED") && <button className="button primary full" disabled={busy} onClick={() => void onAccept()}>{busy ? "กำลังยอมรับ…" : "ยอมรับคำเชิญ"}</button>}
      {active && <button className="button primary full" disabled={busy} onClick={() => void onClear()}>เข้าสู่ร้าน Arisa Rental</button>}
      {awaiting && <p className="team-pending-status" role="status">รออนุมัติ · ยังเข้าถึงข้อมูลร้านไม่ได้</p>}
      {!active && session && <button className="button secondary full" disabled={busy} onClick={() => void onRefresh()}>ตรวจสถานะบัญชี</button>}
      {invitation && session && <button className="text-button" disabled={busy} onClick={() => void onLogout(true)}>ยืนยันด้วย LINE บัญชีอื่น</button>}
      {invitation && !session && <button className="text-button" disabled={busy} onClick={() => void onClear()}>กลับหน้าเข้าสู่ระบบ</button>}
      {!invitation && !session && <button className="button secondary full" disabled={busy} onClick={() => void onClear()}>กลับหน้าเข้าสู่ระบบ</button>}
      {session && <button className="text-button" disabled={busy} onClick={() => void onLogout()}>ออกจากระบบ</button>}
    </div>
  </section></main>;
}

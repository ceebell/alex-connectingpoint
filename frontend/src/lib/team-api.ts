export type TeamRole = "OWNER" | "MANAGER" | "STAFF";
export const lineLoginErrors: Record<string, string> = { LINE_CANCELLED: "ยกเลิกการเข้าสู่ระบบด้วย LINE", LINE_STATE_INVALID: "การเข้าสู่ระบบหมดเวลาหรือไม่ถูกต้อง กรุณาลองใหม่", LINE_LOGIN_FAILED: "เข้าสู่ระบบด้วย LINE ไม่สำเร็จ กรุณาลองใหม่", LINE_NOT_CONFIGURED: "ยังไม่ได้ตั้งค่า LINE Login กรุณาติดต่อผู้ดูแลระบบ", LINE_NOT_ALLOWED: "บัญชี LINE นี้ยังไม่มีสิทธิ์เข้าหลังบ้าน" };
export type TeamStatus = "INVITED" | "PENDING" | "ACTIVE" | "SUSPENDED" | "REJECTED" | "CANCELLED" | "EXPIRED";
export type TeamBranch = { id: string; name: string };
export type TeamMember = {
  id: string; name: string; role: TeamRole; status: TeamStatus; version: number;
  branch_ids: string[]; branches: TeamBranch[]; note: string; line_user_id?: string | null;
  line_name?: string | null; picture_url?: string | null; last_login_at?: string | null;
  created_by?: string | null; created_at?: string | null; updated_at?: string | null;
  expires_at?: string | null; accepted_at?: string | null; approved_at?: string | null;
};
export type AdminSession = {
  username: string; login: "line" | "password" | "token"; hermes_configured: boolean;
  team_enabled?: boolean; membership?: TeamMember | null;
  capabilities?: { manage_team: boolean; submit_order: boolean };
  line_profile?: { name: string; picture_url?: string | null };
};
export type TeamInvitation = { id: string; version: number; shop_name: string; name: string; role: TeamRole; branches: TeamBranch[]; invited_by?: string; expires_at: string; status: "INVITED" | "EXPIRED" | "CANCELLED" | "USED" };
export const teamRoles: Record<TeamRole, string> = { OWNER: "เจ้าของร้าน", MANAGER: "ผู้จัดการ", STAFF: "พนักงาน" };
export const teamStatuses: Record<TeamStatus, string> = { INVITED: "รอยืนยัน LINE", PENDING: "รออนุมัติ", ACTIVE: "ใช้งานอยู่", SUSPENDED: "ระงับสิทธิ์", REJECTED: "ปฏิเสธแล้ว", CANCELLED: "ยกเลิกคำเชิญแล้ว", EXPIRED: "คำเชิญหมดอายุ" };
export const canManageTeam = (session: AdminSession) => session.team_enabled === true && session.membership?.status === "ACTIVE" && session.capabilities?.manage_team === true;
export const canUseWorkspace = (session: AdminSession) => !session.team_enabled || session.membership?.status === "ACTIVE";
export const canSubmitOrder = (session: AdminSession) => !session.team_enabled || (canUseWorkspace(session) && session.capabilities?.submit_order === true);

export function invitationToken(hash: string) {
  const token = new URLSearchParams(hash.replace(/^#/, "")).get("invite");
  return token && /^[A-Za-z0-9_-]{43}$/.test(token) ? token : null;
}
export function invitationUrl(origin: string, token: string) {
  if (!/^[A-Za-z0-9_-]{43}$/.test(token)) throw new Error("Invalid invitation");
  const url = new URL("/", origin);
  url.hash = new URLSearchParams({ invite: token }).toString();
  return url.href;
}

"use client";
import { useEffect, useState } from "react";
import { api, ApiError, errorText } from "@/lib/api";
import type { AdminSession } from "@/lib/team-api";
import { formatDateTime } from "@/lib/dates";
import { navigateWorkspace } from "@/lib/workspace-navigation";
import WorkspaceNavigation from "./workspace-navigation";
import WorkspaceSidebar from "./workspace-sidebar";
import { useTeamViewport } from "./team-workspace";

type Settings = { version: number; instruction_text: string; updated_by: string | null; updated_at: string | null; rules: { customer_name_prefix: string | null; shipping_product: { rental_below_satang: number; fee_satang: number; product_code: string } | null; summary_amounts?: boolean } };
const example = "ชื่อลูกค้าทุกคนต้องมี K. นำหน้า\nหากค่าเช่าในออเดอร์นั้นต่ำกว่า 600 บาท จะมีค่าขนส่ง 40 บาท ให้เพิ่มค่าขนส่งจาก stock product code: 0053MNANANA\nแอดมินสรุปบิลให้เอาค่าเช่า ค่าประกันตรงหน้าสรุปบิลเป็นหลัก ไม่คำนวณจากตะกร้าเอง";

export default function ShopSettings({ session, onLogout }: { session: AdminSession; onLogout: () => Promise<void> }) {
  useTeamViewport();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [saved, setSaved] = useState(false);
  const [conflict, setConflict] = useState(false);
  useEffect(() => {
    let cancelled = false;
    void api<Settings>("/api/admin/shop-settings").then(value => { if (!cancelled) { setSettings(value); setText(value.instruction_text); } }).catch(cause => { if (!cancelled) setError(errorText(cause)); });
    return () => { cancelled = true; };
  }, []);
  async function save() {
    if (!settings || busy || conflict) return;
    setBusy(true); setError(""); setSaved(false);
    try { const value = await api<Settings>("/api/admin/shop-settings", { method: "POST", body: JSON.stringify({ instruction_text: text, expected_version: settings.version }) }); setSettings(value); setText(value.instruction_text); setSaved(true); }
    catch (cause) { setError(errorText(cause)); if (cause instanceof ApiError && cause.code === "RECORD_CONFLICT") setConflict(true); }
    finally { setBusy(false); }
  }
  async function reload() {
    setBusy(true); setError("");
    try { const value = await api<Settings>("/api/admin/shop-settings"); setSettings(value); setText(value.instruction_text); setConflict(false); setSaved(false); }
    catch (cause) { setError(errorText(cause)); }
    finally { setBusy(false); }
  }
  const rules = settings?.rules;
  return <div className="app-shell team-shell settings-shell"><WorkspaceSidebar username={session.username} login={session.login} activePanel="inquiries" homeActive onHome={() => navigateWorkspace("home")} onNavigate={() => navigateWorkspace("chats")} onManageTeam={() => navigateWorkspace("team")} onLogout={onLogout} />
    <main className="main-content"><header className="page-header"><div><p className="eyebrow">Arisa Rental</p><h1>ตั้งค่าร้าน</h1><p className="muted">เงื่อนไขการเตรียมออเดอร์ของร้าน</p></div><div className="header-actions"><button className="button secondary" onClick={() => navigateWorkspace("home")}>กลับหน้าหลัก</button></div></header>
      <section className="shop-settings-content" aria-label="เงื่อนไข AI สำหรับออกบิล"><form className="shop-settings-card" onSubmit={event => { event.preventDefault(); void save(); }} autoComplete="off">
        <h2>เงื่อนไขให้ AI เตรียมออเดอร์</h2><p className="settings-description">AI จะอ่านกฎที่บันทึกไว้ก่อนเตรียมออเดอร์ และระบบจะใช้กฎนี้ตรวจชื่อ ยอดค่าเช่า/ค่าประกัน และค่าขนส่งก่อนส่งบิล</p>
        <label htmlFor="billing-instructions">เงื่อนไขการออกบิล<textarea id="billing-instructions" autoComplete="off" rows={7} maxLength={4000} value={text} disabled={!settings || busy} placeholder={example} onChange={event => { setText(event.target.value); setSaved(false); }} aria-describedby="billing-instruction-help" /></label>
        <p id="billing-instruction-help" className="settings-description">เขียนหนึ่งเงื่อนไขต่อบรรทัด รองรับคำนำหน้าชื่อลูกค้า ค่าขนส่งจากรหัสสินค้าเมื่อค่าเช่าสุทธิต่ำกว่ายอดที่กำหนด และใช้ค่าเช่า/ค่าประกันจากสรุปบิลเป็นหลัก เงื่อนไขรูปแบบอื่นจะไม่ถูกบันทึก</p>
        <button type="button" className="text-button" disabled={!settings || busy} onClick={() => { setText(example); setSaved(false); }}>ใช้ตัวอย่างเงื่อนไข</button>
        <div className="notice info"><strong>ใช้เฉพาะการออกบิล</strong><p>ไม่รับสคริปต์ ลิงก์เรียกใช้งาน หรือคำสั่งเปลี่ยนสิทธิ์/แก้ข้อมูลอื่น รหัสค่าขนส่งต้องตรงกับสินค้าในสาขาของออเดอร์ ราคาใน Alex ต้องตรงกับค่าขนส่งและไม่มีค่าประกัน</p></div>
        {error && <div className="notice warning" role="alert">{error}{conflict && <button type="button" className="text-button" disabled={busy} onClick={() => void reload()}>โหลดค่าล่าสุด</button>}</div>}
        {saved && <p className="team-feedback" role="status">บันทึกเงื่อนไขร้านแล้ว</p>}
        {settings?.updated_at && <p className="settings-description">แก้ไขโดย {settings.updated_by} · {formatDateTime(settings.updated_at)}</p>}
        <div className="team-form-actions"><button type="button" className="button secondary" disabled={!settings || busy} onClick={() => { if (settings) setText(settings.instruction_text); setSaved(false); }}>ยกเลิกการแก้ไข</button><button className="button primary" disabled={!settings || busy || conflict || text === settings.instruction_text}>{busy ? "กำลังบันทึก…" : "บันทึกเงื่อนไข"}</button></div>
      </form>
      {rules && <section className="shop-settings-card" aria-label="กฎที่ใช้งานอยู่"><h2>กฎที่ใช้งานอยู่</h2>{!rules.customer_name_prefix && !rules.shipping_product && !rules.summary_amounts && <p className="settings-description">ยังไม่มีเงื่อนไขเพิ่มเติม</p>}{rules.customer_name_prefix && <p>เติม {rules.customer_name_prefix} นำหน้าชื่อลูกค้าในบิล โดยไม่เติมซ้ำ</p>}{rules.shipping_product && <p>ค่าเช่าสุทธิต่ำกว่า {rules.shipping_product.rental_below_satang / 100} บาท เพิ่ม {rules.shipping_product.fee_satang / 100} บาท จากรหัส {rules.shipping_product.product_code} จำนวน 1 รายการ ไม่คิดค่าส่งซ้ำในหมายเหตุ</p>}{rules.summary_amounts && <p>บิลใช้ค่าเช่าและค่าประกันตามสรุปบิลของแอดมิน ส่วนลดคิดจากส่วนต่างกับราคาใน Alex ไม่คำนวณยอดจากตะกร้าหรือช่องส่วนลดในร่าง ถ้าสรุปบิลไม่มียอดหรือยอดสูงกว่าราคาใน Alex จะหยุดให้แก้ก่อนส่ง</p>}</section>}
      </section><WorkspaceNavigation activePage="home" manageTeam />
    </main></div>;
}

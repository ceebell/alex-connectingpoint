"use client";
import { FormEvent, useEffect, useState } from "react";
import { api, ApiError, Customer, errorText, Profile } from "@/lib/api";
import { CustomerAvatar } from "./chat-message";
import CustomerExtractButton from "./customer-extract-button";

type Form = { display_name: string; phone: string; address: string; notes: string };
const toForm = (profile: Profile | null): Form => ({ display_name: profile?.display_name ?? "", phone: profile?.phone ?? "", address: profile?.address ?? "", notes: profile?.notes ?? "" });
const time = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit", timeZone: "Asia/Bangkok" }).format(new Date(value));

export default function CustomerProfile({ userId, inquiryId, line, onUnauthorized }: { userId: string; inquiryId: string; line?: Customer; onUnauthorized: () => void }) {
  const [profile, setProfile] = useState<Profile | null>(null);
  const [form, setForm] = useState<Form>(toForm(null));
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<{ kind: "warning" | "success"; text: string } | null>(null);

  useEffect(() => {
    let cancelled = false;
    api<{ profile: Profile | null }>(`/api/line/users/${encodeURIComponent(userId)}/profile`).then(result => {
      if (!cancelled) { setProfile(result.profile); setForm(toForm(result.profile)); }
    }).catch(cause => { if (!cancelled) { if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); else setNotice({ kind: "warning", text: errorText(cause) }); } }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [userId]); // eslint-disable-line react-hooks/exhaustive-deps

  async function save(event: FormEvent) {
    event.preventDefault(); setBusy(true); setNotice(null);
    try {
      const result = await api<{ profile: Profile }>(`/api/line/users/${encodeURIComponent(userId)}/profile`, { method: "POST", body: JSON.stringify({ expected_version: profile?.version ?? 0, ...Object.fromEntries(Object.entries(form).map(([key, value]) => [key, value.trim() || null])) }) });
      setProfile(result.profile); setForm(toForm(result.profile)); setNotice({ kind: "success", text: "บันทึกข้อมูลลูกค้าแล้ว" });
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) { onUnauthorized(); return; }
      if (cause instanceof ApiError && cause.code === "RECORD_CONFLICT") {
        // Another tab saved first: show the stored version instead of overwriting it.
        const latest = await api<{ profile: Profile | null }>(`/api/line/users/${encodeURIComponent(userId)}/profile`).catch(() => null);
        if (latest) { setProfile(latest.profile); setForm(toForm(latest.profile)); }
      }
      setNotice({ kind: "warning", text: errorText(cause) });
    } finally { setBusy(false); }
  }

  const field = (key: keyof Form, label: string, multiline = false, max = 200) => <label className={multiline ? "span-2" : ""}>{label}{multiline ? <textarea rows={key === "notes" ? 6 : 3} maxLength={max} value={form[key]} disabled={busy} onChange={e => setForm({ ...form, [key]: e.target.value })} /> : <input value={form[key]} maxLength={max} placeholder={key === "display_name" && line?.display_name ? `ชื่อใน LINE: ${line.display_name}` : undefined} inputMode={key === "phone" ? "tel" : undefined} autoComplete="off" disabled={busy} onChange={e => setForm({ ...form, [key]: e.target.value })} />}</label>;

  if (loading) return <p role="status" className="muted">กำลังโหลดข้อมูลลูกค้า…</p>;
  return <form className="draft-form" onSubmit={save}>
    <div className="step-caption"><span aria-hidden="true">👤</span><div><h3>ข้อมูลลูกค้า</h3><p className="muted small">ใช้ร่วมกันทุกเรื่องของลูกค้าคนนี้ · เก็บในฐานข้อมูลบน server ลูกค้าไม่เห็น</p></div></div>
    <div className="customer-row"><CustomerAvatar customer={line} /><div><strong>{line?.display_name || "ไม่ทราบชื่อใน LINE"}</strong><p className="user-ref small muted">LINE: {userId}</p></div></div>
    <CustomerExtractButton inquiryId={inquiryId} disabled={busy} onUnauthorized={onUnauthorized} onFill={value => setForm(current => ({ ...current, display_name: value.display_name ?? current.display_name, phone: value.phone ?? current.phone, address: value.address ?? current.address }))} />
    <div className="field-grid">{field("display_name", "ชื่อลูกค้า")}{field("phone", "เบอร์โทร", false, 40)}{field("address", "ที่อยู่", true, 1000)}{field("notes", "โน้ตภายใน", true, 5000)}</div>
    {notice && <div className={`notice ${notice.kind}`} role={notice.kind === "warning" ? "alert" : "status"}>{notice.text}</div>}
    {profile && <p className="muted small">บันทึกล่าสุดโดย {profile.updated_by} · {time(profile.updated_at)} · ครั้งที่ {profile.version}</p>}
    <button className="button primary full" disabled={busy}>{busy ? "กำลังบันทึก…" : "บันทึกข้อมูลลูกค้า"}</button>
  </form>;
}

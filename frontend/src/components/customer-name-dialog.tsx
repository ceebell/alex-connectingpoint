"use client";
import { useEffect, useRef, useState } from "react";
import { api, ApiError, Customer, CustomerAlias, errorText } from "@/lib/api";

export default function CustomerNameDialog({ userId, customer, onSaved, onClose, onUnauthorized }: { userId: string; customer?: Customer; onSaved: (alias: CustomerAlias, lineName: string | null) => void; onClose: () => void; onUnauthorized: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [name, setName] = useState(customer?.display_name ?? "");
  const [lineName, setLineName] = useState(customer?.line_display_name ?? customer?.display_name ?? null);
  const [version, setVersion] = useState(customer?.name_version ?? 0);
  const [custom, setCustom] = useState(customer?.custom_name ?? null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const endpoint = `/api/line/users/${encodeURIComponent(userId)}/alias`;
  useEffect(() => {
    dialog.current?.showModal();
    let cancelled = false;
    api<{ alias: CustomerAlias | null; line_display_name: string | null }>(endpoint).then(value => {
      if (cancelled) return;
      setVersion(value.alias?.version ?? 0); setCustom(value.alias?.custom_name ?? null); setLineName(value.line_display_name); setName(value.alias?.custom_name ?? value.line_display_name ?? "");
    }).catch(cause => { if (!cancelled) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); } }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [endpoint]); // eslint-disable-line react-hooks/exhaustive-deps

  async function save(custom_name: string | null) {
    setBusy(true); setError("");
    try {
      const value = await api<{ alias: CustomerAlias }>(endpoint, { method: "POST", body: JSON.stringify({ expected_version: version, custom_name }) });
      onSaved(value.alias, lineName); onClose();
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
      if (cause instanceof ApiError && cause.code === "RECORD_CONFLICT") {
        try {
          const latest = await api<{ alias: CustomerAlias | null; line_display_name: string | null }>(endpoint);
          setVersion(latest.alias?.version ?? 0); setCustom(latest.alias?.custom_name ?? null); setLineName(latest.line_display_name);
          setError("มีแอดมินเปลี่ยนชื่อแล้ว ข้อความที่คุณกรอกยังอยู่ ตรวจชื่อแล้วกดบันทึกอีกครั้ง");
        } catch { setError("โหลดชื่อล่าสุดไม่สำเร็จ กรุณาปิดแล้วเปิดหน้าต่างนี้อีกครั้ง"); }
      } else setError(errorText(cause));
    } finally { setBusy(false); }
  }

  return <dialog ref={dialog} className="customer-name-dialog" aria-labelledby="customer-name-title" onCancel={event => { if (busy) event.preventDefault(); }} onClose={onClose}>
    <form autoComplete="off" onSubmit={event => { event.preventDefault(); if (name.trim()) void save(name.trim()); }}>
      <div className="section-caption"><h2 id="customer-name-title">ตั้งชื่อลูกค้า</h2><button type="button" className="text-button" disabled={busy} onClick={onClose}>ปิด</button></div>
      <label>ชื่อที่ใช้ใน Alex<input autoComplete="off" autoFocus value={name} maxLength={200} disabled={busy || loading} onChange={event => setName(event.target.value)} /></label>
      <p className="muted small">ชื่อใน LINE: {lineName || "ยังไม่มีชื่อ"}</p>
      {error && <p className="notice warning" role="alert">{error}</p>}
      <div className="customer-name-actions"><button type="button" className="button secondary" disabled={busy || loading || !custom} onClick={() => void save(null)}>ใช้ชื่อจาก LINE</button><button className="button primary" disabled={busy || loading || !name.trim()}>{busy ? "กำลังบันทึก…" : "บันทึกชื่อ"}</button></div>
    </form>
  </dialog>;
}

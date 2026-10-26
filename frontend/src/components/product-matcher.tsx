"use client";
/* eslint-disable @next/next/no-img-element -- Catalog image proxy requires same-origin HttpOnly cookies. */
import { useEffect, useState } from "react";
import { api, ApiError, Draft, errorText, LineEvent } from "@/lib/api";

type Candidate = { product_item_id: string; code: string; name: string; color: string | null; size: string | null; rental_fee_satang: number | null; deposit_satang: number | null; image_url: string | null; score: number | null };
type Search = { search_id: string; branch_id: string; page: number; has_more: boolean; method: string; candidates: Candidate[] };
type Choice = Candidate & { search_id: string };
type Match = { item_index: number; branch_id: string; revision: number; candidate: Candidate; confirmed_by: string; confirmed_at: string; method: string; differences: string[] };
type Matches = { matches: Match[]; product_verified: boolean };
type Settings = { configured: boolean; branches: { id: string; name: string }[]; image_search_configured: boolean; product_url_configured: boolean };
type Mode = "code" | "text" | "product_url" | "image";
const money = (value: number | null) => value === null ? "ไม่ระบุ" : `${(value / 100).toLocaleString("th-TH", { minimumFractionDigits: 2 })} บาท`;
const date = (value: string) => new Intl.DateTimeFormat("th-TH", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", timeZone: "Asia/Bangkok" }).format(new Date(value));

export default function ProductMatcher({ inquiryId, draft, contextHash, messages, disabled, onUnauthorized }: { inquiryId: string; draft: Draft | undefined; contextHash: string; messages: LineEvent[]; disabled: boolean; onUnauthorized: () => void }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [matches, setMatches] = useState<Matches>({ matches: [], product_verified: false });
  const [branch, setBranch] = useState("");
  const [index, setIndex] = useState(0);
  const [mode, setMode] = useState<Mode>("code");
  const [query, setQuery] = useState(draft?.result?.fields.items[0]?.product_code ?? "");
  const [color, setColor] = useState("");
  const [size, setSize] = useState("");
  const [category, setCategory] = useState("");
  const [texture, setTexture] = useState("");
  const [image, setImage] = useState("");
  const [result, setResult] = useState<Search | null>(null);
  const [choices, setChoices] = useState<Choice[]>([]);
  const [selected, setSelected] = useState("");
  const [acknowledged, setAcknowledged] = useState(false);
  const [differenceAcknowledged, setDifferenceAcknowledged] = useState(false);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(!!draft?.result?.fields.items.length);
  const [error, setError] = useState("");
  const fields = draft?.result?.fields;
  const item = fields?.items[index];
  const choice = choices.find(value => `${value.search_id}:${value.product_item_id}` === selected);
  const differing = choice && item ? ([
    ["รหัส", item.product_code, choice.code], ["สี", item.color, choice.color], ["ไซซ์", item.size, choice.size],
  ] as const).filter(([, original, catalog]) => original && original.toLowerCase() !== catalog?.toLowerCase()) : [];
  const unavailable = disabled || busy || loading || !settings?.configured || !!draft?.stale || !!draft?.redacted;
  const images = messages.filter(message => !message.withdrawn && message.payload?.message?.type === "image" && message.media_id);

  useEffect(() => {
    if (!draft?.result?.fields.items.length) return;
    let cancelled = false;
    Promise.all([api<Settings>("/api/admin/alex"), api<Matches>(`/api/inquiries/${inquiryId}/products`)]).then(([settings, matches]) => {
      if (cancelled) return;
      setSettings(settings); setMatches(matches);
      if (settings.branches.length === 1) setBranch(settings.branches[0].id);
    }).catch(cause => { if (!cancelled) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); } }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [inquiryId, draft?.draft_version]); // eslint-disable-line react-hooks/exhaustive-deps

  function clearSearch() { setChoices([]); setResult(null); setSelected(""); setAcknowledged(false); setDifferenceAcknowledged(false); setError(""); }
  function fail(cause: unknown) { setError(errorText(cause)); if (cause instanceof ApiError && cause.status === 401) onUnauthorized(); }

  async function search(next = false) {
    if (unavailable || !draft) return;
    setBusy(true); setError(""); setSelected(""); setAcknowledged(false); setDifferenceAcknowledged(false);
    try {
      const value = await api<Search>(`/api/inquiries/${inquiryId}/products/search`, { method: "POST", body: JSON.stringify({
        draft_version: draft.draft_version, context_hash: contextHash, item_index: index, branch_id: branch, mode,
        query: query.trim() || null, color: color.trim() || null, size: size.trim() || null,
        category: category.trim() || null, texture: texture.trim() || null, image_event_id: mode === "image" ? image : null,
        page: next && result ? result.page + 1 : 1,
      }) });
      setResult(value);
      const incoming = value.candidates.map(candidate => ({ ...candidate, search_id: value.search_id }));
      setChoices(previous => [...new Map([...(next ? previous : []), ...incoming].map(candidate => [candidate.product_item_id, candidate])).values()]);
    } catch (cause) { fail(cause); } finally { setBusy(false); }
  }

  async function confirm() {
    if (!choice || !acknowledged || unavailable || (differing.length > 0 && !differenceAcknowledged)) return;
    setBusy(true); setError("");
    try {
      await api(`/api/inquiries/${inquiryId}/products/confirm`, { method: "POST", body: JSON.stringify({
        search_id: choice.search_id, product_item_id: choice.product_item_id,
        expected_revision: matches.matches.find(match => match.item_index === index)?.revision ?? 0,
        acknowledge_difference: differenceAcknowledged,
      }) });
      setMatches(await api<Matches>(`/api/inquiries/${inquiryId}/products`));
      setSelected(""); setAcknowledged(false); setDifferenceAcknowledged(false);
    } catch (cause) { fail(cause); } finally { setBusy(false); }
  }

  if (!fields?.items.length) return <div className="notice info">เตรียมร่างและบันทึกรายการสินค้าก่อน แล้วกลับมาเลือกชุดจาก Alex</div>;
  return <div className="product-matcher">
    <div className="step-caption"><span>02</span><div><h3>ระบุชุดที่จะเช่าจาก Alex</h3><p className="muted small">ใช้รายการในร่าง v{draft?.draft_version} แล้วตรวจรหัส สี ไซซ์ และสาขาก่อนยืนยัน</p></div></div>
    {loading && <p role="status">กำลังอ่านสิทธิ์สาขาและรายการที่ยืนยัน…</p>}
    {error && <div className="notice warning" role="alert">{error}</div>}
    {settings && !settings.configured && <div className="notice warning">ยังไม่เชื่อมบัญชี Alex กรุณาให้ผู้ดูแลระบบตั้งค่าการเชื่อมต่อก่อนค้นและยืนยันชุด</div>}
    {(draft?.stale || draft?.redacted) && <div className="notice warning">ร่างหรือบริบทเปลี่ยน กรุณาตรวจและบันทึกร่างใหม่ก่อนเลือกสินค้า</div>}
    {matches.product_verified && <div className="notice success" role="status">ยืนยัน Product ID ครบ {fields.items.length} รายการแล้ว พร้อมนำไปตรวจคิวในขั้นถัดไป</div>}
    {matches.matches.map(match => <div className="product-match" key={match.item_index}><strong>รายการ {match.item_index + 1} · {match.candidate.code}</strong><p>{match.candidate.color || "ไม่ระบุสี"} · {match.candidate.size || "ไม่ระบุไซซ์"} · {settings?.branches.find(branch => branch.id === match.branch_id)?.name ?? match.branch_id}</p><code>{match.candidate.product_item_id}</code><p className="muted small">ยืนยันโดย {match.confirmed_by} · {date(match.confirmed_at)}</p>{match.differences.length > 0 && <p className="muted small">แอดมินยืนยันค่าจาก Alex ที่ต่างจากข้อความในร่างแล้ว</p>}</div>)}
    <form onSubmit={e => { e.preventDefault(); void search(); }}>
      <fieldset disabled={unavailable}>
        <label>สินค้ารายการที่จะจับคู่<select value={index} onChange={e => { const next = Number(e.target.value); setIndex(next); setQuery(fields.items[next].product_code ?? ""); clearSearch(); }}>{fields.items.map((item, i) => <option key={i} value={i}>รายการ {i + 1} · {item.product_code || "ยังไม่มีรหัส"} · {item.color || "ไม่ระบุสี"} · {item.size || "ไม่ระบุไซซ์"}</option>)}</select></label>
        <label>สาขาที่ได้รับสิทธิ์<select value={branch} onChange={e => { setBranch(e.target.value); clearSearch(); }} required><option value="">เลือกสาขา</option>{settings?.branches.map(branch => <option key={branch.id} value={branch.id}>{branch.name}</option>)}</select></label>
        <label>ค้นหาจาก<select value={mode} onChange={e => { setMode(e.target.value as Mode); setQuery(""); clearSearch(); }}><option value="code">รหัสสินค้าแบบตรงตัว</option><option value="text">ชื่อหรือคำบรรยาย / ดูแค็ตตาล็อก</option><option value="product_url" disabled={!settings?.product_url_configured}>ลิงก์สินค้าที่อนุญาต</option><option value="image" disabled={!settings?.image_search_configured}>รูปจากบทสนทนา</option></select></label>
        {mode === "image" && <label>รูปอ้างอิง<select value={image} onChange={e => { setImage(e.target.value); clearSearch(); }} required><option value="">เลือกรูปจากบทสนทนา</option>{images.map((image, i) => <option key={image._id} value={image._id}>รูปที่ {i + 1} · {date(image.event_at)}</option>)}</select></label>}
        <label>{mode === "product_url" ? "ลิงก์สินค้า" : "รหัส ชื่อ หรือคำบรรยาย"}<input maxLength={500} value={query} onChange={e => { setQuery(e.target.value); clearSearch(); }} required={mode === "code" || mode === "product_url"} placeholder={mode === "text" ? "เว้นว่างเพื่อดูแค็ตตาล็อกในสาขานี้" : mode === "image" ? "ระบุคำค้นเพิ่มเพื่อจำกัดภาพที่เปรียบเทียบได้" : "เช่น DR-012"} /></label>
        <details className="product-filters"><summary>ตัวกรองเพิ่มเติม</summary><div className="field-grid">{([["สี", color, setColor], ["ไซซ์", size, setSize], ["หมวดหมู่ (รหัส)", category, setCategory], ["ลาย/เนื้อผ้า (รหัส)", texture, setTexture]] as const).map(([label, value, set]) => <label key={label}>{label}<input maxLength={200} value={value} onChange={e => { set(e.target.value); clearSearch(); }} /></label>)}</div></details>
        <button className="button secondary full" disabled={!branch || (mode === "image" && !image)}>{busy ? "กำลังค้นหาสินค้า…" : "ค้นหาสินค้าใน Alex"}</button>
      </fieldset>
    </form>
    {busy && <p className="muted small" role="status">กำลังอ่าน Alex และตรวจบริบทล่าสุด…</p>}
    {result && <p className="muted small">พบ {choices.length} ตัวเลือกที่โหลดมา · หน้าที่ {result.page}{result.method === "image_code" ? " · อ่านรหัสจากรูปแล้วค้นยืนยันใน Alex" : result.method === "image_similarity" ? " · เรียงตามความคล้ายของรูปในหน้าที่ค้น" : ""}</p>}
    {result && !choices.length && <div className="notice info">ยังไม่พบตัวเลือกที่ตรงในหน้าที่ค้น ลองปรับตัวกรอง{result.has_more ? " หรือโหลดหน้าถัดไป" : ""}</div>}
    <div className="product-candidates" role="radiogroup" aria-label="ตัวเลือกสินค้าจาก Alex">{choices.map(candidate => {
      const key = `${candidate.search_id}:${candidate.product_item_id}`;
      return <label key={key} className={`product-candidate ${selected === key ? "selected" : ""}`}><input type="radio" name="catalog-candidate" disabled={unavailable} checked={selected === key} onChange={() => { setSelected(key); setAcknowledged(false); setDifferenceAcknowledged(false); }} />{candidate.image_url && <img src={candidate.image_url} alt={`รูปสินค้า ${candidate.code}`} loading="lazy" onError={e => { e.currentTarget.hidden = true; }} />}<div><strong>{candidate.code} · {candidate.name}</strong><p>{candidate.color || "ไม่ระบุสี"} · {candidate.size || "ไม่ระบุไซซ์"}</p><code>{candidate.product_item_id}</code><p className="muted small">ค่าเช่า {money(candidate.rental_fee_satang)} · มัดจำ {money(candidate.deposit_satang)}</p>{candidate.score !== null && <p className="muted small">ความคล้าย {Math.round(candidate.score * 100)}% · ใช้จัดลำดับตัวเลือก</p>}</div></label>;
    })}</div>
    {result?.has_more && <button type="button" className="button secondary full" disabled={unavailable} onClick={() => search(true)}>โหลดหน้าถัดไป</button>}
    {choice && <div className="product-confirm">
      {differing.length > 0 && <div className="notice warning"><strong>ค่าจาก Alex ต่างจากร่าง</strong><ul>{differing.map(([label, original, catalog]) => <li key={label}>{label}: ในร่าง {original} → Alex {catalog || "ไม่ระบุ"}</li>)}</ul><label className="checkbox-row"><input type="checkbox" checked={differenceAcknowledged} onChange={e => setDifferenceAcknowledged(e.target.checked)} disabled={unavailable} />ตรวจแล้วและยืนยันเลือกค่าจาก Alex ที่ต่างตามรายการนี้</label></div>}
      <label className="checkbox-row"><input type="checkbox" checked={acknowledged} onChange={e => setAcknowledged(e.target.checked)} disabled={unavailable} />ตรวจชุด {choice.code} สี ไซซ์ และสาขาแล้ว เป็นรายการที่ต้องการเช่า</label>
      <button type="button" className="button primary full" disabled={unavailable || !acknowledged || (differing.length > 0 && !differenceAcknowledged)} onClick={confirm}>ยืนยันชุดสำหรับรายการ {index + 1}</button>
    </div>}
    <div className="notice soft"><p>การยืนยันนี้เก็บ Product ID ตามที่ Alex ส่งกลับ ราคาที่แสดงยังต้องตรวจเงื่อนไขราคาและคิวในขั้นถัดไป เมื่อบันทึกร่างเวอร์ชันใหม่ให้ตรวจและยืนยันชุดอีกครั้ง</p></div>
  </div>;
}

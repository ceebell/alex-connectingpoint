"use client";
/* eslint-disable @next/next/no-img-element -- Alex catalog images use the existing external image policy. */
import { useEffect, useRef, useState } from "react";
import { api, ApiError, Candidate, codeLabel, CodeName, errorText } from "@/lib/api";

export type ProductFavorite = { id: string; branch_id: string; candidate: Candidate; created_at: string };
type SavedCandidate = Candidate & { search_id: string };

export function StarIcon() {
  return <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round" aria-hidden="true"><path d="m12 3 2.8 5.7 6.3.9-4.55 4.43 1.07 6.27L12 17.34l-5.62 2.96 1.07-6.27L2.9 9.6l6.3-.9L12 3Z" /></svg>;
}

export function FavoriteButton({ name, selected, disabled, onClick }: { name: string; selected: boolean; disabled: boolean; onClick: () => void }) {
  const label = `${selected ? "นำออกจากรายการโปรด" : "เพิ่มรายการโปรด"} ${name}`;
  return <button type="button" className={`favorite-star ${selected ? "selected" : ""}`} aria-label={label} title={label} aria-pressed={selected} disabled={disabled} onClick={onClick}><StarIcon /></button>;
}

export function useProductFavorites(inquiryId: string, branch: string, onUnauthorized: () => void) {
  const key = `${inquiryId}:${branch}`;
  const [result, setResult] = useState<{ key: string; items: ProductFavorite[]; error: string; loading: boolean }>({ key: "", items: [], error: "", loading: false });
  const [retry, setRetry] = useState(0);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const generation = useRef<object | null>(null);
  const locked = useRef(false);
  const items = result.key === key ? result.items : [];
  const path = `/api/inquiries/${inquiryId}/favorites`;

  useEffect(() => {
    const current = {};
    generation.current = current;
    if (!branch) return;
    let cancelled = false;
    void Promise.resolve().then(async () => {
      if (cancelled) return;
      setResult(previous => ({ key, items: previous.key === key ? previous.items : [], error: "", loading: true })); setNotice("");
      try {
        const value = await api<{ items: ProductFavorite[] }>(`${path}?branch_id=${encodeURIComponent(branch)}`);
        if (!cancelled && generation.current === current) setResult({ key, items: value.items, loading: false, error: "" });
      } catch (cause) {
        if (!cancelled && generation.current === current) {
          setResult(previous => ({ ...previous, loading: false, error: errorText(cause) }));
          if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
        }
      }
    });
    return () => { cancelled = true; generation.current = null; };
  }, [key, retry]); // eslint-disable-line react-hooks/exhaustive-deps -- Key contains inquiry and branch; callbacks do not change the requested resource.

  async function toggle(candidate: SavedCandidate | ProductFavorite) {
    if (locked.current || !branch || result.key !== key || result.loading || result.error) return;
    locked.current = true; setBusy(true); setNotice("");
    const current = generation.current;
    const existing = "candidate" in candidate ? candidate : items.find(item => item.candidate.product_item_id === candidate.product_item_id);
    try {
      const saved = existing
        ? await api<{ removed: boolean }>(`${path}/${existing.id}`, { method: "DELETE" })
        : await api<ProductFavorite>(path, { method: "POST", body: JSON.stringify({ search_id: (candidate as SavedCandidate).search_id, product_item_id: (candidate as SavedCandidate).product_item_id }) });
      if (generation.current !== current) return;
      setResult(previous => ({ ...previous, items: existing ? previous.items.filter(item => item.id !== existing.id) : [saved as ProductFavorite, ...previous.items] }));
      setNotice(existing ? "นำสินค้าออกจากรายการโปรดแล้ว" : "บันทึกสินค้าในรายการโปรดแล้ว");
    } catch (cause) {
      if (generation.current === current) {
        setResult(previous => ({ ...previous, error: errorText(cause) }));
        if (cause instanceof ApiError && cause.status === 401) onUnauthorized();
      }
    } finally { locked.current = false; setBusy(false); }
  }

  return { items, busy, notice: result.key === key ? notice : "", error: result.key === key ? result.error : "", loading: !!branch && (result.key !== key || result.loading), reload: () => setRetry(value => value + 1), toggle };
}

export function ProductFavorites({ branch, controller, colors, disabled, adding, cartNotice, onAdd }: { branch: string; controller: ReturnType<typeof useProductFavorites>; colors: CodeName[]; disabled: boolean; adding: string; cartNotice: string; onAdd: (favorite: ProductFavorite) => void }) {
  const [filter, setFilter] = useState("");
  const query = filter.trim().toLocaleLowerCase();
  const items = controller.items.filter(item => [item.candidate.name, item.candidate.code, item.candidate.color, codeLabel(item.candidate.color, colors), item.candidate.size].some(value => value?.toLocaleLowerCase().includes(query)));
  const money = (value: number | null) => value === null ? "ไม่ระบุ" : (value / 100).toLocaleString("th-TH", { maximumFractionDigits: 2 });
  return <section className="product-favorites" aria-label="รายการโปรดของร้าน" aria-busy={controller.loading || controller.busy}>
    <h4><StarIcon /> รายการโปรดของร้าน <span>({controller.items.length})</span></h4>
    {!branch ? <p className="muted small">เลือกสาขาเพื่อดูรายการโปรด</p> : <>
      <p className="muted small">กดดาวบนผลค้นหาเพื่อเก็บไว้ใช้กับลูกค้าคนอื่นได้</p>
      {controller.loading && <p className="muted small" role="status">กำลังโหลดรายการโปรด…</p>}
      {controller.busy && <p className="muted small" role="status">กำลังบันทึกรายการโปรด…</p>}
      {controller.error && <div className="notice warning" role="alert">{controller.error}<button type="button" className="text-button" disabled={controller.busy} onClick={controller.reload}>โหลดรายการโปรดอีกครั้ง</button></div>}
      {!controller.loading && !controller.error && !controller.items.length && <p className="muted small">ยังไม่มีรายการโปรด ค้นหาสินค้าแล้วกดดาวเพื่อบันทึก</p>}
      {controller.items.length > 0 && <>
        <label className="favorite-filter">ค้นในรายการโปรด<input autoComplete="off" value={filter} onChange={event => setFilter(event.target.value)} placeholder="ชื่อสินค้า รหัส สี หรือไซซ์" /></label>
        <div className="favorite-list">{items.map(favorite => { const product = favorite.candidate; return <article className="favorite-row" key={favorite.id} aria-label={`สินค้าโปรด ${product.name}`}>
          <div className="favorite-photo">{product.image_url && <img src={product.image_url} alt={`รูปสินค้า ${product.name}`} loading="lazy" referrerPolicy="no-referrer" onError={event => { event.currentTarget.hidden = true; }} />}</div>
          <div className="favorite-description"><strong>{product.name}</strong><p>{product.code} · {codeLabel(product.color, colors)} · {product.size || "ไม่ระบุไซซ์"}</p><p>ค่าเช่า {money(product.rental_fee_satang)} · ประกัน {money(product.deposit_satang)}</p></div>
          <div className="favorite-actions"><FavoriteButton name={product.name} selected disabled={disabled || controller.busy || controller.loading} onClick={() => void controller.toggle(favorite)} /><button type="button" className="favorite-add" aria-label={`เพิ่มสินค้าโปรด ${product.name} ลงตะกร้า`} disabled={disabled || controller.busy || controller.loading} onClick={() => onAdd(favorite)}>{adding === favorite.id ? "…" : "+"}</button></div>
        </article>; })}</div>
        {!items.length && <p className="muted small">ไม่พบรายการโปรดที่ตรงกับคำค้น</p>}
        <p className="muted small">ระบบตรวจสินค้าและราคาล่าสุดจาก Alex ก่อนเพิ่มลงตะกร้า</p>
      </>}
      {controller.notice && <p className="favorite-notice small" role="status">{controller.notice}</p>}
      {cartNotice && <p className="favorite-notice small" role="status">{cartNotice}</p>}
    </>}
  </section>;
}

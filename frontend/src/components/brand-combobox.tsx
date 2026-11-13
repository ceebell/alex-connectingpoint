"use client";
import { CSSProperties, KeyboardEvent, useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { CodeName } from "@/lib/api";
import { brandLabel, similarBrands } from "@/lib/brand-matching";

type Props = { options: CodeName[]; value: string; query: string; disabled: boolean; onQuery: (query: string) => void; onSelect: (option?: CodeName) => void };
export default function BrandCombobox({ options, value, query, disabled, onQuery, onSelect }: Props) {
  const id = useId(), root = useRef<HTMLDivElement>(null), input = useRef<HTMLInputElement>(null), popup = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false), [active, setActive] = useState(0), [position, setPosition] = useState<CSSProperties | null>(null);
  const matches = similarBrands(query, options).map(match => match.option);
  const expanded = open && !disabled;
  const current = Math.min(active, Math.max(0, matches.length - 1));
  useEffect(() => {
    if (!expanded) return;
    let frame = 0;
    const place = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const box = root.current?.getBoundingClientRect();
        if (!box) return;
        const viewport = window.visualViewport;
        const top = viewport?.offsetTop ?? 0, bottom = top + (viewport?.height ?? window.innerHeight);
        const below = bottom - box.bottom - 8, above = box.top - top - 8;
        const flip = below < 180 && above > below;
        setPosition({ position: "fixed", left: Math.max(8, Math.min(box.left, window.innerWidth - box.width - 8)), width: Math.min(box.width, window.innerWidth - 16), maxHeight: Math.max(60, Math.min(260, flip ? above : below)), ...(flip ? { bottom: window.innerHeight - box.top + 4 } : { top: box.bottom + 4 }) });
      });
    };
    const outside = (event: globalThis.PointerEvent) => { if (event.target instanceof Node && !root.current?.contains(event.target) && !popup.current?.contains(event.target)) setOpen(false); };
    place(); window.addEventListener("resize", place); window.addEventListener("scroll", place, true);
    window.visualViewport?.addEventListener("resize", place); window.visualViewport?.addEventListener("scroll", place);
    document.addEventListener("pointerdown", outside, true);
    return () => { cancelAnimationFrame(frame); window.removeEventListener("resize", place); window.removeEventListener("scroll", place, true); window.visualViewport?.removeEventListener("resize", place); window.visualViewport?.removeEventListener("scroll", place); document.removeEventListener("pointerdown", outside, true); };
  }, [expanded]);
  useEffect(() => { if (expanded) document.getElementById(`${id}-option-${current}`)?.scrollIntoView({ block: "nearest" }); }, [expanded, current, id]);
  function choose(option?: CodeName) { onSelect(option); setOpen(false); }
  function key(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") { event.preventDefault(); setOpen(true); setActive(index => event.key === "ArrowDown" ? Math.min(matches.length - 1, expanded ? index + 1 : 0) : Math.max(0, index - 1)); }
    else if (event.key === "Enter" && expanded) { event.preventDefault(); if (matches[current]) choose(matches[current]); }
    else if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); setOpen(false); }
  }
  return <div className="brand-field span-2" data-editor-popup>
    <label htmlFor={id}>แบรนด์</label>
    <div ref={root} className="brand-combobox">
      <input id={id} ref={input} role="combobox" autoComplete="off" maxLength={200} value={query} disabled={disabled} aria-autocomplete="list" aria-expanded={expanded} aria-controls={`${id}-options`} aria-activedescendant={expanded && matches.length ? `${id}-option-${current}` : undefined} placeholder="พิมพ์ชื่อหรือรหัสแบรนด์" onFocus={() => setOpen(true)} onBlur={() => setOpen(false)} onChange={event => { onQuery(event.target.value); setActive(0); setOpen(true); }} onKeyDown={key} />
      {query && <button type="button" className="brand-clear" disabled={disabled} aria-label="ล้างแบรนด์" onPointerDown={event => event.preventDefault()} onClick={event => { event.stopPropagation(); onSelect(); setActive(0); setOpen(true); input.current?.focus(); }}>×</button>}
      <button type="button" className="brand-toggle" disabled={disabled} aria-label="แสดงตัวเลือกแบรนด์" aria-expanded={expanded} onPointerDown={event => event.preventDefault()} onClick={event => { event.stopPropagation(); input.current?.focus(); setOpen(!expanded); }}><svg viewBox="0 0 20 20" aria-hidden="true"><path d="m5 7 5 5 5-5" /></svg></button>
    </div>
    {query && !value && <span className="brand-hint">เลือกแบรนด์ที่ใกล้เคียงจากรายการ</span>}
    {expanded && position && createPortal(<div ref={popup} className="brand-options" data-editor-popup style={position} id={`${id}-options`} role="listbox" aria-label="แบรนด์ที่ใกล้เคียง">
      {!query && <button type="button" role="option" aria-selected={!value} onPointerDown={event => event.preventDefault()} onClick={event => { event.stopPropagation(); choose(); }}>ทุกแบรนด์</button>}
      {matches.map((option, index) => <button type="button" role="option" id={`${id}-option-${index}`} key={option.code} className={index === current ? "is-highlighted" : ""} aria-selected={value === option.code} onPointerDown={event => event.preventDefault()} onClick={event => { event.stopPropagation(); choose(option); }}>{brandLabel(option)}</button>)}
      {!matches.length && <p role="status">ไม่พบแบรนด์ที่ใกล้เคียง ลองปรับคำค้น</p>}
    </div>, document.body)}
  </div>;
}

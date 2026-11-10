"use client";
import { useEffect, useId, useRef, useState } from "react";
import { DATE_PLACEHOLDER, formatDate, parseDateInput } from "@/lib/dates";

type Props = { value: string; disabled?: boolean; onChange: (value: string) => void };
const validationMessage = "กรอกวันที่จริงในรูปแบบ dd/mm/yyyy โดยใช้ปี ค.ศ. 1900–2199";

// The text field fixes the display format; the adjacent native control keeps the calendar picker.
export default function DateInput({ value, disabled, onChange }: Props) {
  const [entry, setEntry] = useState(() => ({ source: value, text: value ? formatDate(value) : "" }));
  const [touched, setTouched] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const errorId = useId();
  if (entry.source !== value) setEntry({ source: value, text: value ? formatDate(value) : "" });
  const invalid = !!entry.text && !parseDateInput(entry.text);
  useEffect(() => { input.current?.setCustomValidity(invalid ? validationMessage : ""); }, [invalid]);

  function change(text: string) {
    const iso = parseDateInput(text) ?? "";
    setEntry({ source: iso, text });
    onChange(iso);
  }

  return <span className="date-input">
    <span className="date-input-control">
      <input ref={input} className="date-input-text" type="text" inputMode="numeric" autoComplete="off" maxLength={10} placeholder={DATE_PLACEHOLDER} value={entry.text} disabled={disabled} aria-invalid={invalid || undefined} aria-describedby={invalid && touched ? errorId : undefined} onBlur={() => setTouched(true)} onInvalid={() => setTouched(true)} onChange={event => change(event.target.value)} />
      <span className="date-picker-control">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><rect x="3" y="5" width="18" height="16" rx="2" /><path d="M16 3v4M8 3v4M3 11h18M8 15h2M14 15h2" /></svg>
        <input autoComplete="off" type="date" lang="en-GB" aria-label="เลือกวันที่จากปฏิทิน" title="เลือกวันที่จากปฏิทิน" min="1900-01-01" max="2199-12-31" value={value} disabled={disabled} onChange={event => change(event.target.value ? formatDate(event.target.value) : "")} />
      </span>
    </span>
    {invalid && touched && <span id={errorId} className="date-input-error" role="alert">{validationMessage}</span>}
  </span>;
}

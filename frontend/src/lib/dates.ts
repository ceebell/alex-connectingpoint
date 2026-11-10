// Display Gregorian dates consistently, regardless of the browser's locale or time zone.
export const DATE_PLACEHOLDER = "dd/mm/yyyy";
const timeZone = "Asia/Bangkok";
type DateValue = string | number | Date | null | undefined;
const dateFormatter = new Intl.DateTimeFormat("en-GB", { day: "2-digit", month: "2-digit", year: "numeric", calendar: "gregory", numberingSystem: "latn", timeZone });
const timeFormatter = new Intl.DateTimeFormat("en-GB", { hour: "2-digit", minute: "2-digit", hourCycle: "h23", numberingSystem: "latn", timeZone });

// Match the backend's supported Gregorian range and reject dates that JavaScript would roll forward.
function validDay(year: number, month: number, day: number) {
  if (year < 1900 || year > 2199) return false;
  const date = new Date(Date.UTC(year, month - 1, day));
  return date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
}

export function parseDateInput(value: string): string | null {
  const match = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(value);
  if (!match || !validDay(Number(match[3]), Number(match[2]), Number(match[1]))) return null;
  return `${match[3]}-${match[2]}-${match[1]}`;
}

function asDate(value: DateValue): Date | null {
  if (value === null || value === undefined || value === "") return null;
  let input = value;
  if (typeof input === "string" && /^\d{4}-\d{2}-\d{2}$/.test(input)) {
    const [year, month, day] = input.split("-").map(Number);
    if (!validDay(year, month, day)) return null;
    // A booking day belongs to Bangkok, rather than the viewer's local midnight.
    input = `${input}T00:00:00+07:00`;
  }
  const date = input instanceof Date ? input : new Date(input);
  return Number.isNaN(date.getTime()) ? null : date;
}

export function formatDate(value: DateValue): string {
  const date = asDate(value);
  return date ? dateFormatter.format(date) : "—";
}

export function formatDateTime(value: DateValue): string {
  const date = asDate(value);
  return date ? `${dateFormatter.format(date)} ${timeFormatter.format(date)}` : "—";
}

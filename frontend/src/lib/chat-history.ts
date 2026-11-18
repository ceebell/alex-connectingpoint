import type { LineEvent } from "./api";

export type ChatPage = { batches: LineEvent[][]; next: string | null; previous?: string | null };

export function compareMessages(a: LineEvent, b: LineEvent) {
  const compare = (left: string, right: string) => left < right ? -1 : left > right ? 1 : 0;
  return new Date(a.event_at).getTime() - new Date(b.event_at).getTime()
    || compare(a.event_id ?? a._id, b.event_id ?? b._id) || compare(a._id, b._id);
}

export function mergeMessages(previous: LineEvent[], incoming: LineEvent[]) {
  return [...new Map([...previous, ...incoming].map(message => [message._id, message])).values()].sort(compareMessages);
}

// Refresh every page the admin has opened, starting at the newest messages. Count-based
// page limits miss new pages when many messages arrive or old captures are inserted late.
export async function refreshChatWindow(load: (before: string | null) => Promise<ChatPage>, oldest?: LineEvent) {
  let before: string | null = null;
  let previous: string | null = null;
  const seen = new Set<string>();
  const records: LineEvent[] = [];
  do {
    const page = await load(before);
    const incoming = page.batches.flat();
    records.push(...incoming);
    previous = page.previous ?? null;
    if (!oldest || incoming.some(message => compareMessages(message, oldest) <= 0) || !previous) break;
    if (seen.has(previous)) throw new Error("Chat history cursor did not advance");
    seen.add(previous); before = previous;
  } while (previous);
  const merged = mergeMessages([], records);
  if (!oldest) return { records: merged, previous };
  const visible = merged.filter(message => compareMessages(message, oldest) >= 0);
  const hasOlder = !!previous || merged.some(message => compareMessages(message, oldest) < 0);
  return { records: visible, previous: hasOlder ? visible[0]?._id ?? oldest._id : null };
}

// Keep rows added by a concurrent send/history load, while removing withdrawn rows
// from the refreshed window that the server no longer returns.
export function reconcileMessages(previous: LineEvent[], snapshot: LineEvent[], incoming: LineEvent[]) {
  const ids = new Set(snapshot.map(message => message._id));
  return mergeMessages(incoming, previous.filter(message => !ids.has(message._id)));
}

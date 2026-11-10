import type { Inquiry } from "./api.ts";

export const NOTIFICATION_PREFERENCE = "alex.chat-notifications.enabled.v1";
export const NOTIFICATION_HISTORY = "alex.chat-notifications.seen.v1";
export const NOTIFICATION_TAG = "alex-chat:";
export const validInquiryId = (id: unknown): id is string => typeof id === "string" && /^[a-f0-9]{24}$/i.test(id);

// Track incoming messages independently of the inbox preview, which may be an admin reply.
// The first snapshot is a baseline; pagination, unsends and repeated polls must not replay history.
export class IncomingMessageTracker {
  private initialized = false;
  private watermark = 0;
  private cursors = new Map<string, { id: string; at: number }>();

  observe(inquiries: Inquiry[]): Inquiry[] {
    const fresh: Inquiry[] = [];
    let newest = this.watermark;
    for (const inquiry of inquiries) {
      const id = inquiry.last_incoming_message_id;
      const at = Date.parse(inquiry.last_incoming_message_at ?? "");
      if (!id || !validInquiryId(inquiry._id) || !Number.isFinite(at)) continue;
      const previous = this.cursors.get(inquiry._id);
      if (previous && at < previous.at) continue;
      const changed = previous ? previous.id !== id : at > this.watermark;
      if (this.initialized && changed && inquiry.unread !== false) fresh.push(inquiry);
      this.cursors.delete(inquiry._id);
      this.cursors.set(inquiry._id, { id, at });
      newest = Math.max(newest, at);
    }
    this.watermark = newest;
    this.initialized = true;
    while (this.cursors.size > 4000) this.cursors.delete(this.cursors.keys().next().value!);
    return fresh;
  }
}

export function notificationCopy(inquiry: Inquiry) {
  return {
    title: `Alex · ${inquiry.customer?.display_name || "ลูกค้า LINE"}`,
    body: (inquiry.last_incoming_message_preview || "มีข้อความใหม่").slice(0, 240),
  };
}

export function notificationHistory(raw: string | null): string[] {
  try {
    const parsed: unknown = JSON.parse(raw ?? "[]");
    return Array.isArray(parsed) ? parsed.filter((id): id is string => typeof id === "string" && id.length <= 200).slice(-1000) : [];
  } catch { return []; }
}

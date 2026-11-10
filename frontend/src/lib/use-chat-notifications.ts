"use client";
import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from "react";
import type { Inquiry } from "./api";
import { IncomingMessageTracker, notificationCopy, notificationHistory, NOTIFICATION_HISTORY, NOTIFICATION_PREFERENCE, NOTIFICATION_TAG, validInquiryId } from "./chat-notifications";

const preferenceEvent = "alex:notification-preference";
const workerPath = "/alex-notifications-sw.js";
const workerScope = "/alex-notifications/";
const supported = () => typeof window !== "undefined" && window.isSecureContext && typeof window.Notification === "function" && "serviceWorker" in navigator && typeof ServiceWorkerRegistration !== "undefined" && "showNotification" in ServiceWorkerRegistration.prototype;
function preference() { try { return localStorage.getItem(NOTIFICATION_PREFERENCE) === "true"; } catch { return false; } }
const enabled = () => supported() && Notification.permission === "granted" && preference();
const snapshot = () => supported() ? `${Notification.permission}:${preference()}` : "unsupported";
function subscribe(listener: () => void) {
  let permission: PermissionStatus | undefined;
  let cancelled = false;
  window.addEventListener("focus", listener);
  window.addEventListener("storage", listener);
  window.addEventListener(preferenceEvent, listener);
  void navigator.permissions?.query({ name: "notifications" }).then(value => {
    if (!cancelled) { permission = value; permission.addEventListener("change", listener); }
  }).catch(() => {});
  return () => {
    cancelled = true;
    permission?.removeEventListener("change", listener);
    window.removeEventListener("focus", listener);
    window.removeEventListener("storage", listener);
    window.removeEventListener(preferenceEvent, listener);
  };
}
function savePreference(value: boolean) {
  localStorage.setItem(NOTIFICATION_PREFERENCE, String(value));
  window.dispatchEvent(new Event(preferenceEvent));
}

let registrationPromise: Promise<ServiceWorkerRegistration> | null = null;
async function registration() {
  if (!registrationPromise) registrationPromise = (async () => {
    const result = await navigator.serviceWorker.register(workerPath, { scope: workerScope, updateViaCache: "none" });
    if (result.active) return result;
    const worker = result.installing ?? result.waiting;
    if (!worker) throw new Error("Notification worker unavailable");
    await new Promise<void>((resolve, reject) => {
      const finish = (error?: Error) => {
        clearTimeout(timer); worker.removeEventListener("statechange", changed);
        if (error) reject(error); else resolve();
      };
      const changed = () => {
        if (worker.state === "activated") finish();
        else if (worker.state === "redundant") finish(new Error("Notification worker unavailable"));
      };
      const timer = setTimeout(() => finish(new Error("Notification worker timed out")), 15000);
      worker.addEventListener("statechange", changed); changed();
    });
    return result;
  })().catch(cause => { registrationPromise = null; throw cause; });
  return registrationPromise;
}
function clientId(worker: ServiceWorker | null) {
  return new Promise<string>(resolve => {
    if (!worker) { resolve(""); return; }
    const channel = new MessageChannel();
    const finish = (value: string) => { clearTimeout(timer); channel.port1.close(); resolve(value); };
    const timer = setTimeout(() => finish(""), 1500);
    channel.port1.onmessage = event => finish(typeof event.data === "string" ? event.data : "");
    worker.postMessage({ type: "ALEX_CLIENT_ID" }, [channel.port2]);
  });
}
async function closeNotifications() {
  try {
    const registered = await navigator.serviceWorker.getRegistration(workerScope);
    if (registered?.scope !== new URL(workerScope, location.origin).href) return;
    for (const notification of await registered.getNotifications()) if (notification.tag.startsWith(NOTIFICATION_TAG)) notification.close();
  } catch { /* Closing a toast must never block logout or disabling notifications. */ }
}

export function useChatNotifications(active: boolean, onOpen: (id: string) => void | Promise<void>, isReading: (id: string) => boolean) {
  const state = useSyncExternalStore(subscribe, snapshot, () => "unsupported");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [initialInquiry] = useState(() => typeof window === "undefined" ? null : new URLSearchParams(window.location.search).get("inquiry"));
  const initialOpened = useRef(false);
  const hadSession = useRef(false);
  const options = useRef({ active, onOpen, isReading });
  const tracker = useRef(new IncomingMessageTracker());
  const queue = useRef(Promise.resolve());
  const seen = useRef(new Set<string>());
  useEffect(() => { options.current = { active, onOpen, isReading }; });
  useEffect(() => () => { options.current.active = false; if (hadSession.current) void closeNotifications(); }, []);
  useEffect(() => { tracker.current = new IncomingMessageTracker(); }, [active]);
  useEffect(() => {
    if (active && validInquiryId(initialInquiry) && !initialOpened.current) {
      initialOpened.current = true;
      void options.current.onOpen(initialInquiry);
    }
    if (!active && hadSession.current) void closeNotifications();
    hadSession.current = active;
  }, [active, initialInquiry]);
  useEffect(() => {
    if (!("serviceWorker" in navigator)) return;
    const open = (event: MessageEvent) => {
      const source = event.source as ServiceWorker | null;
      if (options.current.active && source?.scriptURL === new URL(workerPath, location.origin).href && event.data?.type === "ALEX_OPEN_INQUIRY" && validInquiryId(event.data.inquiryId)) {
        void options.current.onOpen(event.data.inquiryId);
      }
    };
    navigator.serviceWorker.addEventListener("message", open);
    return () => navigator.serviceWorker.removeEventListener("message", open);
  }, []);

  const observeInbox = useCallback((inquiries: Inquiry[]) => {
    const incoming = tracker.current.observe(inquiries);
    if (!options.current.active || !enabled()) return;
    queue.current = queue.current.then(async () => {
      for (const inquiry of incoming) {
        const show = async () => {
          if (!options.current.active || !enabled() || options.current.isReading(inquiry._id)) return;
          const key = `${inquiry._id}:${inquiry.last_incoming_message_id}`;
          let history: string[] = [];
          try { history = notificationHistory(localStorage.getItem(NOTIFICATION_HISTORY)); } catch { /* Use this tab's history if browser storage is unavailable. */ }
          if (seen.current.has(key) || history.includes(key)) return;
          const registered = await registration();
          const targetClient = await clientId(registered.active);
          if (!options.current.active || !enabled() || options.current.isReading(inquiry._id)) return;
          const { title, body } = notificationCopy(inquiry);
          const notificationOptions: NotificationOptions & { renotify: boolean } = { body, tag: `${NOTIFICATION_TAG}${inquiry._id}`, renotify: true, data: { inquiryId: inquiry._id, clientId: targetClient } };
          await registered.showNotification(title, notificationOptions);
          seen.current.add(key);
          if (seen.current.size > 1000) seen.current.delete(seen.current.values().next().value!);
          try { localStorage.setItem(NOTIFICATION_HISTORY, JSON.stringify([...history, key].slice(-1000))); } catch { /* Notification delivery still works without persistent history. */ }
        };
        // One browser notification per message even when several Alex tabs poll together.
        if (navigator.locks) await navigator.locks.request("alex-chat-notifications", show);
        else await show();
      }
    }).catch(() => { if (options.current.active) setMessage("แสดงการแจ้งเตือนไม่สำเร็จ ตรวจสิทธิ์การแจ้งเตือนของเบราว์เซอร์"); });
  }, []);
  const pollInBackground = useCallback(() => options.current.active && enabled(), []);
  async function toggle() {
    setMessage("");
    if (!supported()) { setMessage("เบราว์เซอร์นี้ยังไม่รองรับการแจ้งเตือน กรุณาเปิด Alex ด้วย Chrome ผ่าน HTTPS"); return; }
    if (enabled()) { savePreference(false); void closeNotifications(); return; }
    if (Notification.permission === "denied") { setMessage("การแจ้งเตือนถูกบล็อก เปิดสิทธิ์การแจ้งเตือนในการตั้งค่าเว็บไซต์ของเบราว์เซอร์ แล้วกดเปิดอีกครั้ง"); return; }
    setBusy(true);
    try {
      // Ask only on this explicit button click, before asynchronous worker setup.
      const permission = Notification.permission === "granted" ? "granted" : await Notification.requestPermission();
      window.dispatchEvent(new Event(preferenceEvent));
      if (permission !== "granted") { setMessage("ยังไม่ได้อนุญาตการแจ้งเตือน กดปุ่มกระดิ่งเพื่อตั้งค่าอีกครั้งได้"); return; }
      await registration();
      savePreference(true);
      setMessage("เปิดแจ้งเตือนข้อความใหม่แล้ว เปิดแท็บ Alex ค้างไว้เพื่อรับการแจ้งเตือน");
    } catch { setMessage("เปิดการแจ้งเตือนไม่สำเร็จ ตรวจสิทธิ์เบราว์เซอร์และการอนุญาตจัดเก็บข้อมูลของเว็บไซต์"); }
    finally { setBusy(false); }
  }
  return { observeInbox, pollInBackground, toggle, busy, message, dismissMessage: () => setMessage(""), enabled: state === "granted:true" };
}

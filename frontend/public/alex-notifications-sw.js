/* This worker only opens notifications. It does not cache pages or make authenticated API calls. */
self.addEventListener("install", event => event.waitUntil(self.skipWaiting()));
self.addEventListener("message", event => {
  if (event.data?.type === "ALEX_CLIENT_ID" && event.source?.url && new URL(event.source.url).origin === self.location.origin) {
    event.ports[0]?.postMessage(event.source.id);
  }
});
self.addEventListener("notificationclick", event => {
  event.notification.close();
  const { inquiryId, clientId } = event.notification.data ?? {};
  if (typeof inquiryId !== "string" || !/^[a-f0-9]{24}$/i.test(inquiryId)) return;
  event.waitUntil((async () => {
    const windows = await self.clients.matchAll({ type: "window", includeUncontrolled: true });
    const appWindows = windows.filter(client => {
      const url = new URL(client.url);
      return url.origin === self.location.origin && url.pathname === "/";
    });
    const target = appWindows.find(client => client.id === clientId) ?? appWindows[0];
    if (target) {
      await target.focus();
      target.postMessage({ type: "ALEX_OPEN_INQUIRY", inquiryId });
    } else {
      await self.clients.openWindow(`/?inquiry=${encodeURIComponent(inquiryId)}`);
    }
  })());
});

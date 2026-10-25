import http from "node:http";
import { createHash, timingSafeEqual } from "node:crypto";
import { readFile } from "node:fs/promises";

export async function controlServer({ page, getPage = () => page, username, password, status, diagnostics, onOpen, port = 8082, host = "0.0.0.0" }) {
  const html = await readFile(new URL("./control.html", import.meta.url));
  const digest = value => createHash("sha256").update(value).digest();
  const expected = digest("Basic " + Buffer.from(`${username}:${password}`).toString("base64"));
  const server = http.createServer(async (req, res) => {
    res.setHeader("Cache-Control", "no-store");
    res.setHeader("X-Content-Type-Options", "nosniff");
    res.setHeader("X-Frame-Options", "DENY");
    const json = (code, value) => { res.writeHead(code, { "Content-Type": "application/json" }); res.end(JSON.stringify(value)); };
    if (req.method === "GET" && req.url === "/health") { json(200, { status: "ok" }); return; }
    if (!timingSafeEqual(expected, digest(req.headers.authorization ?? ""))) {
      res.setHeader("WWW-Authenticate", 'Basic realm="Alex OA connector", charset="UTF-8"'); json(401, { error: "UNAUTHORIZED" }); return;
    }
    try {
      const page = getPage();
      if (req.method === "GET" && req.url === "/") { res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" }); res.end(html); return; }
      if (req.method === "GET" && req.url === "/status") { json(200, status()); return; }
      if (req.method === "GET" && req.url === "/schema") { json(200, diagnostics()); return; }
      if (req.method === "GET" && req.url === "/screen") {
        const bytes = await page.screenshot({ type: "jpeg", quality: 75 });
        res.writeHead(200, { "Content-Type": "image/jpeg" }); res.end(bytes); return;
      }
      if (req.method !== "POST" || req.url !== "/action" || req.headers["x-oa-control"] !== "1"
        || (req.headers.origin && req.headers.origin !== `http://${req.headers.host}`)) { json(403, { error: "FORBIDDEN" }); return; }
      let size = 0; const parts = [];
      for await (const chunk of req) { size += chunk.length; if (size > 16_384) { json(413, { error: "TOO_LARGE" }); return; } parts.push(chunk); }
      const action = JSON.parse(Buffer.concat(parts).toString("utf8"));
      if (action.type === "open") await onOpen();
      else if (action.type === "click" && Number.isFinite(action.x) && Number.isFinite(action.y) && action.x >= 0 && action.x < 1280 && action.y >= 0 && action.y < 900) await page.mouse.click(action.x, action.y);
      else if (action.type === "text" && typeof action.text === "string" && action.text.length <= 2000) await page.keyboard.insertText(action.text);
      else if (action.type === "key" && ["Enter", "Tab", "Backspace", "Escape", "Control+A"].includes(action.key)) await page.keyboard.press(action.key);
      else if (action.type === "scroll" && [-600, 600].includes(action.dy)) await page.mouse.wheel(0, action.dy);
      else { json(400, { error: "INVALID_ACTION" }); return; }
      json(200, { status: "ok" });
    } catch { json(503, { error: "CONTROL_UNAVAILABLE" }); }
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(port, host, resolve); });
  return server;
}

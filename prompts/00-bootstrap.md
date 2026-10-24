# 00 — Bootstrap: สร้าง skeleton ของโปรเจกต์

> ใช้ครั้งเดียวตอนเริ่มโปรเจกต์ (หลังวางไฟล์ชุดนี้ลงโฟลเดอร์ repo แล้ว)
> แก้ค่าใน `{{ }}` ก่อนส่ง

---

อ่าน `AGENTS.md` และ `revising-log.md` ให้ครบก่อน แล้วสร้าง skeleton ของโปรเจกต์ `{{myapp}}` ตามกฎใน `AGENTS.md` ทุกข้อ
**ห้ามแก้** `runall.bat`, `build.bat`, `rungit.bat`, `loadgit.sh`, `docker-*.sh`, `docker-stack.yml`, `docker/*.Dockerfile` — สคริปต์เหล่านี้ถูกเขียนไว้ให้ทำงานกับโครงสร้างที่ระบุไว้ด้านล่าง ถ้าพบว่าต้องแก้จริง ให้หยุดและบอกเหตุผลก่อน

## 1) Backend — `backend/`

- `cargo init --name backend` (package name ต้องเป็น `backend`)
- Dependencies: `axum`, `tokio` (features `full`), `serde` (derive), `serde_json`, `mongodb`, `futures-util`, `tower-http` (features `cors`, `trace`), `tracing`, `tracing-subscriber` (features `env-filter`), `dotenvy`, `thiserror` — ใช้เวอร์ชันล่าสุดที่เข้ากันได้ และตรวจ API ตาม docs ของเวอร์ชันนั้น
- โครงสร้าง: `main.rs`, `config.rs`, `error.rs`, `state.rs`, `routes/mod.rs`, `routes/health.rs`, `routes/items.rs`, `models/mod.rs`, `models/item.rs`, `db/mod.rs`, `db/indexes.rs`
- `config.rs` อ่าน env: `PORT` (default 8080), `MONGODB_URI` (default `mongodb://localhost:27017`), `MONGODB_DB` (default `myapp`), `CORS_ORIGIN` (ว่างหรือไม่ตั้ง = ไม่เปิด CORS layer)
- โหลด `.env` ด้วย `dotenvy` (ไฟล์อยู่ที่ root ของ repo — dotenvy ค้นหาจากโฟลเดอร์แม่ให้เอง)
- `AppError` (thiserror) → แปลงเป็น JSON `{"error":{"code","message"}}` พร้อม status code ที่ถูกต้อง
- Routes:
  - `GET /api/health` → `{"status":"ok"}` (Dockerfile ใช้เป็น healthcheck — ห้ามเปลี่ยน path)
  - ตัวอย่าง collection `items`: `GET /api/items`, `POST /api/items` (`name: String`) พร้อม `created_at`, `updated_at`
- Bind `0.0.0.0:{PORT}` · graceful shutdown (Ctrl+C / SIGTERM) · log ด้วย `tracing`
- สร้าง index ใน `db/indexes.rs` ตอน startup (idempotent)
- เพิ่ม unit/integration test อย่างน้อย 1 ตัวสำหรับ health

## 2) Frontend — `frontend/`

- สร้างด้วย `create-next-app` : TypeScript, Tailwind CSS, ESLint, App Router, `src/` directory, import alias `@/*`, ใช้ **npm**
- `next.config.ts`:
  - `output: "standalone"`
  - `rewrites()` : `/api/:path*` → `${BACKEND_URL}/api/:path*` โดย `BACKEND_URL = process.env.BACKEND_URL ?? "http://localhost:8080"`
- Font: **Mitr** ผ่าน `next/font/google` (subsets `thai`, `latin`; weights 300–700; variable `--font-mitr`) ตั้งเป็น font หลักของ Tailwind และ `<html lang="th">`
- Theme ใน `globals.css` (`@theme`) ตามตาราง Role ใน `AGENTS.md` ข้อ 6: `primary` เขียว · `accent` ส้ม · `info` ฟ้า
- `src/lib/api.ts` — fetch wrapper เรียก `/api/...` แบบ relative พร้อม type ของ error
- หน้าแรก `src/app/page.tsx` (ภาษาไทย) เป็นหน้าตัวอย่างที่แสดง design system:
  - Section สลับพื้น `bg-white` / `bg-gray-50` / `bg-white`
  - Hero + ปุ่ม primary (เขียว) และ accent (ส้ม) + badge/ลิงก์สีฟ้า
  - การ์ดแสดงสถานะ backend (เรียก `/api/health`) และรายการ `items` พร้อมฟอร์มเพิ่ม item
  - ใช้ emoji ประกอบหัวข้อ/ปุ่ม/empty state ตามสมควร (ใส่ `aria-hidden` เมื่อเป็นของตกแต่ง)
- สร้าง `frontend/public/.gitkeep` (Dockerfile ต้องใช้โฟลเดอร์ `public`)

## 3) ตรวจสอบ

รันและรายงานผลจริงของแต่ละข้อ:

1. `cd backend` → `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release`
2. `cd frontend` → `npm run lint`, `npm run build` และตรวจว่ามี `.next/standalone/server.js`
3. ถ้าเครื่องมี MongoDB รันอยู่: เปิด backend แล้ว `curl http://localhost:8080/api/health`
4. ตรวจว่าชื่อ binary คือ `backend` (`backend/target/release/backend[.exe]`)

ถ้ารันข้อไหนไม่ได้ ให้บอกว่าข้อไหนและเพราะอะไร — ห้ามรายงานว่าผ่านโดยไม่ได้รัน

## 4) ปิดงาน

- ดึงเวลาจริงจากเครื่อง แล้วเพิ่ม entry ใน `revising-log.md` (ครบทุกโฟลเดอร์/ไฟล์ที่สร้าง)
- ไม่ commit / push — ให้ผู้ใช้รัน `rungit.bat`
- สรุปสั้น ๆ: สร้างอะไร, เช็กอะไรผ่านแล้ว, ผู้ใช้ต้องรันอะไรต่อ (`runall.bat`)

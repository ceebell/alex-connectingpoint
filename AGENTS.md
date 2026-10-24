# AGENTS.md

คู่มือสำหรับ AI agent (Claude Code, Codex, Cursor ฯลฯ) — **อ่านไฟล์นี้ก่อนแก้โค้ดทุกครั้ง**

## 1. ภาพรวม

- Monorepo: `backend/` (Rust + Axum) · `frontend/` (Next.js) · MongoDB
- ชื่อแอป: `myapp` (ตั้งที่ `.env` → `APP_NAME`; ใช้เป็นชื่อ Docker image และ stack)
- Dev บน **Windows** · Deploy บน **Ubuntu server** ด้วย **Docker Swarm**
- ตอบผู้ใช้เป็น **ภาษาไทย** · โค้ด, ชื่อตัวแปร, คอมเมนต์ในโค้ด เป็นภาษาอังกฤษ · ข้อความบน UI เป็นภาษาไทย

## 2. Tech stack

| ส่วน | เทคโนโลยี |
|---|---|
| Backend | Rust (stable), axum, tokio, serde, tracing, tower-http, dotenvy, thiserror |
| Database | MongoDB (dev: ติดตั้งในเครื่อง) ผ่าน official crate `mongodb` |
| Frontend | Next.js (App Router) + TypeScript + Tailwind CSS · จัดการแพ็กเกจด้วย **npm** |
| Font | **Mitr** (Google Fonts) เป็นหลัก |
| Deploy | Docker + Docker Swarm (`docker-stack.yml`) |

## 3. โครงสร้างโฟลเดอร์

```
.
├─ backend/               Rust axum (package name = `backend`)
│  └─ src/  main.rs · config.rs · error.rs · state.rs · routes/ · models/ · db/
├─ frontend/              Next.js (src/app, src/components, src/lib)
├─ docker/                backend.Dockerfile · frontend.Dockerfile
├─ prompts/               prompt สำเร็จรูปสำหรับสั่งงาน AI
├─ data/db/               ข้อมูล MongoDB ตอน dev (อยู่ใน .gitignore)
├─ dist/                  ผลลัพธ์จาก build.bat (อยู่ใน .gitignore)
├─ runall.bat · build.bat · rungit.bat        (Windows)
├─ loadgit.sh · docker-build.sh · docker-deploy.sh   (Ubuntu)
├─ docker-stack.yml
├─ .env.example           ตัวอย่าง env (ห้าม commit .env จริง)
├─ AGENTS.md · CLAUDE.md · revising-log.md
```

## 4. คำสั่งหลัก

| งาน | คำสั่ง | ที่รัน |
|---|---|---|
| รัน dev ทั้งหมด (Mongo + backend + frontend) | `runall.bat` | Windows |
| Build เฉพาะ binary → `dist/` | `build.bat` | Windows |
| Commit + push | `rungit.bat "ข้อความ commit"` | Windows |
| Pull โค้ดลง server | `./loadgit.sh [branch]` | Ubuntu |
| Build Docker image | `./docker-build.sh` | Ubuntu |
| Deploy Swarm stack | `./docker-deploy.sh` | Ubuntu |

พอร์ต dev: frontend `3000` · backend `8080` · MongoDB `27017`

## 5. กฎ Backend (Rust + axum)

- Package/binary ชื่อ `backend` (`build.bat` และ Dockerfile อ้างชื่อนี้ — ห้ามเปลี่ยนโดยไม่แก้ทั้งสองที่)
- ทุก route ขึ้นต้น `/api` · health check ที่ `GET /api/health` → `{"status":"ok"}`
- Error response รูปแบบเดียวกัน: `{"error":{"code":"...","message":"..."}}` ผ่าน `AppError` ใน `error.rs`
- Config อ่านจาก env เท่านั้น: `PORT`, `MONGODB_URI`, `MONGODB_DB`, `RUST_LOG`, `CORS_ORIGIN`
- ห้าม `unwrap()` / `expect()` ใน handler — คืน `Result<_, AppError>`
- แยก layer: `routes/` (HTTP) → `models/` (struct + serde) → `db/` (query)
- ก่อนจบงานต้องผ่าน: `cargo fmt --check` · `cargo clippy -- -D warnings` · `cargo test`
- commit `Cargo.lock` เสมอ (Docker build ใช้ cache จากไฟล์นี้)

## 6. กฎ Frontend (Next.js) และ Design system

**การเรียก API:** เรียกแบบ relative `/api/...` เสมอ — `next.config.ts` ทำ `rewrites` ไปหา `BACKEND_URL`
(dev: `http://localhost:8080` · Docker: `http://backend:8080`) ห้าม hardcode URL backend ใน component

**next.config.ts:** ต้องมี `output: "standalone"` (ใช้ทั้ง `build.bat` และ Docker)

**โฟลเดอร์ `frontend/public/`:** ต้องมีอยู่เสมอ (เก็บ `.gitkeep` ไว้ถ้าไม่มีไฟล์อื่น) เพราะ Dockerfile `COPY` โฟลเดอร์นี้

**โครงสร้าง:** ใช้ `src/app` (App Router) · component ไว้ `src/components` · helper/fetch wrapper ไว้ `src/lib/api.ts`

**Font:** โหลด Mitr ด้วย `next/font/google` (subsets `thai` + `latin`, weights 300/400/500/600/700)
ผูกกับ `--font-mitr` และตั้งเป็น sans หลักของ Tailwind

**สีและพื้นหลัง:**

- พื้นหลังหลัก **ขาว** สลับกับ **เทาอ่อน** เป็นแถบ section (`bg-white` ↔ `bg-gray-50` / `bg-gray-100`)
- ธีมหลัก **เขียว · ส้ม · ฟ้า** — ตั้งชื่อ semantic ใน `@theme` ของ `globals.css`

| Role | ใช้กับ | พื้น/ขอบ/ไฮไลต์ | ปุ่มและตัวอักษรบนพื้นขาว |
|---|---|---|---|
| `primary` เขียว | CTA หลัก, สำเร็จ | `green-500` | `green-700` |
| `accent` ส้ม | ไฮไลต์, เตือน, ป้ายสำคัญ | `orange-500` | `orange-700` |
| `info` ฟ้า | ลิงก์, ข้อมูล, รอง | `sky-500` | `sky-700` |

- ตัวอักษรบนพื้นสีให้ใช้เฉดเข้ม (700) เพื่อให้ contrast ผ่าน WCAG AA — `*-500` ใช้กับพื้นผิว/ขอบ/ป้าย
- ข้อความหลัก `text-gray-900` · ข้อความรอง `text-gray-600`

**Emoji:** ใช้ประกอบหัวข้อ, ปุ่ม, empty state และ badge ตามสมควร (ไม่เกิน 1 ตัว/หัวข้อ)
emoji ที่เป็นแค่การตกแต่งให้ใส่ `aria-hidden="true"` · ห้ามใช้ emoji แทนข้อความที่จำเป็นต้องอ่านออก

**ก่อนจบงาน:** `npm run lint` · `npm run build` ต้องผ่าน

## 7. Database (MongoDB)

- Dev: `mongodb://localhost:27017` · database ตาม `MONGODB_DB` · data dir `data/db` (runall.bat สร้างให้)
- ชื่อ collection เป็น `snake_case` พหูพจน์ (`orders`, `order_items`)
- ทุก document มี `_id` (ObjectId), `created_at`, `updated_at` (BSON DateTime)
- สร้าง index ตอน startup แบบ idempotent ใน `db/indexes.rs`
- Production (Swarm): Mongo รันเป็น service ใน stack พร้อม user/password จาก `.env` บน server

## 8. Docker / Deploy

- **นโยบายผู้ใช้ล่าสุด:** compile/build โปรแกรมบน local เท่านั้น Ubuntu ใช้ precompiled Linux release และ Dockerfile runtime ที่ COPY binary/standalone เข้า image ห้าม compile Rust หรือ npm build บน server
- `build-line.bat` ใช้ Docker Desktop บน local เพื่อสร้าง Linux/amd64 release พร้อม archive; `deploy-line.sh` ตรวจ release และนำเข้า runtime image บน Ubuntu
- Build context = root ของ repo · ใช้ `-f docker/<name>.Dockerfile`
- Image tag = release tag จาก local (เวลา UTC + hash ของ binary; `dist/line-linux/release.tag`) — **ห้ามใช้แค่ `latest`** เพราะ Swarm จะไม่ rolling update ถ้า tag เดิม
- `.env` จริงอยู่บน server เท่านั้น ไม่เข้า git และไม่ถูก copy เข้า image
- `build.bat` สร้าง **Windows** (`.exe`) ส่วน `build-line.bat` compile **Linux** บนเครื่อง local; Dockerfile ที่ deploy บน Ubuntu ต้องไม่มี compiler หรือคำสั่ง build โปรแกรม

## 9. กฎการทำงานของ agent

1. อ่าน `AGENTS.md` และ entry ล่าสุดใน `revising-log.md` ก่อนเริ่ม
2. แก้เท่าที่จำเป็นตามที่ผู้ใช้ขอ — ไม่ refactor หรือเพิ่ม dependency นอกขอบเขตโดยไม่บอก
3. ใช้ Windows (cmd/PowerShell) เป็นสภาพแวดล้อม dev — ไม่ใช้คำสั่ง bash กับไฟล์ `.bat`
4. ไฟล์ `.bat` ใช้ CRLF และเขียนข้อความเป็น ASCII · ไฟล์ `.sh` ใช้ LF (บังคับด้วย `.gitattributes`)
5. ห้าม `git commit` / `git push` เอง จนกว่าผู้ใช้สั่ง (ผู้ใช้ใช้ `rungit.bat`)
6. ห้ามใส่ secret, password, token ลงในโค้ด, log หรือไฟล์ที่ commit
7. **หลังแก้ไฟล์ใดก็ตามใน repo ต้องบันทึก `revising-log.md` ทุกครั้ง** (ดูข้อ 10) — ถือเป็นขั้นตอนสุดท้ายของทุกงาน

## 10. revising-log.md

บันทึกทุกครั้งที่แก้โค้ด/config/script/Docker (ไม่ต้องบันทึกเมื่อแก้แค่ตัว `revising-log.md` เอง)

- เพิ่ม entry ใหม่ **ไว้บนสุด** ต่อจากบรรทัด `<!-- entries -->`
- 1 งาน = 1 entry (รวมหลายไฟล์ไว้ด้วยกัน) · ห้ามแก้หรือลบ entry เก่า
- วัน-เวลาให้ดึงจากเครื่องจริง ห้ามเดา (เขตเวลา Asia/Bangkok, UTC+7)
  - Windows: `powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-dd HH:mm'"`
  - Linux: `TZ=Asia/Bangkok date '+%Y-%m-%d %H:%M'`

รูปแบบ:

```markdown
## YYYY-MM-DD HH:mm — หัวข้อสั้น ๆ
- **คำอธิบาย:** แก้เพราะอะไร / ผลที่ได้
- **โฟลเดอร์ `path/to/folder/`**
  - `file.ext` — เพิ่ม | แก้ | ลบ: รายละเอียดว่าแก้อะไรไป
  - `other.ext` — ...
- **โฟลเดอร์ `another/folder/`**
  - `file.ext` — ...
```

## 11. Definition of Done

- [ ] โค้ดผ่านเช็กของ backend (ข้อ 5) และ frontend (ข้อ 6) ตามส่วนที่แก้
- [ ] `runall.bat` ยังรันขึ้นครบ 3 ส่วน (ถ้าแก้ส่วนที่กระทบ)
- [ ] ถ้าแก้ Dockerfile/stack: `docker build` ผ่าน
- [ ] อัปเดต `revising-log.md` แล้ว
- [ ] สรุปให้ผู้ใช้สั้น ๆ: แก้อะไร, ไฟล์ไหน, ต้องรันอะไรต่อ

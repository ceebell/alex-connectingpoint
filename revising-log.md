# revising-log

บันทึกการแก้ไขโค้ดทุกครั้ง — entry ใหม่อยู่บนสุด · รูปแบบและกฎดู `AGENTS.md` ข้อ 10

<!-- entries -->

## 2026-10-26 00:04 — ชื่อ/รูปโปรไฟล์ LINE ของลูกค้า, แสดงสติกเกอร์ และปิดตัวเชื่อม OA เป็นค่าเริ่มต้น
- **คำอธิบาย:** ผู้ใช้เข้า LINE Login ได้แล้ว ยืนยันว่าตอบลูกค้าจาก Alex อย่างเดียว (ข้อความที่ตอบใน OA Manager/แอป LINE OA ไม่มี API ทางการให้ดึง) ตรวจ server พบตัวเชื่อม OA ยังส่ง heartbeat NEEDS_LOGIN ทำให้หน้าแชทแสดงข้อความสับสน จึงเพิ่มสวิตช์ปิดตัวเชื่อม; ผู้ใช้ขอให้ดึงชื่อ/รูปลูกค้าและแสดงสติกเกอร์ด้วย
- **โฟลเดอร์ `backend/src/`**
  - `config.rs`, `routes/oa_sync.rs` — แก้: `OA_CONNECTOR_ENABLED` (ค่าเริ่มต้น false) เมื่อปิด status คืน NOT_CONFIGURED และ endpoint ingest/heartbeat ของตัวเชื่อมตอบ 404 ก่อนแตะ DB
  - `db/users.rs`, `worker.rs` — เพิ่ม/แก้: `refresh_profile` เรียก Messaging API Get profile เก็บ display_name/picture_url (เฉพาะ https)/profile_status ใน line_users ดึงซ้ำไม่เกินวันละครั้ง ไม่เก็บ statusMessage; 404 = UNAVAILABLE; ล้มเหลวไม่ทำให้ event ล้ม; `profiles` สำหรับ join
  - `routes/admin.rs` — แก้: `/api/inquiries` แนบ `customer {display_name, picture_url}` และดึงโปรไฟล์ลูกค้าเก่าเบื้องหลังครั้งละไม่เกิน 20 คน
  - `tests.rs`, `tests_oa_sync.rs`, `tests_line.rs` — เพิ่ม/แก้: test ตัวเชื่อมที่ปิดอยู่, mock Get profile, cache รายวัน, UNAVAILABLE และ customer ในรายการ inquiry
- **โฟลเดอร์ `frontend/src/`**
  - `components/chat-message.tsx` — แก้: avatar รูปโปรไฟล์ลูกค้า (fallback ตัวอักษร), ชื่อลูกค้าเป็นผู้ส่ง, สติกเกอร์จาก stickershop.line-scdn.net ตาม stickerId พร้อม alt จาก keywords และ fallback เป็นข้อความเมื่อโหลดไม่ได้
  - `components/admin-workspace.tsx`, `components/customer-profile.tsx`, `lib/api.ts`, `app/globals.css` — แก้: ชื่อ/รูปในรายการ (ค้นหาด้วยชื่อได้), หัวแชท, แท็บลูกค้า (placeholder ชื่อใน LINE), types Customer/sticker และสไตล์
- **โฟลเดอร์ `./` และ `docs/`**
  - `docker-stack.line.yml`, `.env.example` — แก้: `OA_CONNECTOR_ENABLED` (ตรวจด้วย docker stack config)
  - `docs/step-2-admin.md`, `docs/line-oa-auto-sync.md` — แก้: โปรไฟล์/สติกเกอร์ ข้อจำกัด CDN ที่ไม่ใช่ API ทางการ และการเปิดตัวเชื่อมกลับ
- **โฟลเดอร์ `dist/` (build จริงบน local)**
  - `line-linux/` — rel-20261025170323-3ccacb444b06
  - `frontend-linux/` — front-20261025170420-588f29abdf8d (checksum 2108 ไฟล์ผ่าน และไม่มีไฟล์ release ถูก gitignore)
- **ผลตรวจสอบ:** cargo fmt/clippy -D warnings ผ่าน; cargo test รวม MongoDB integration ผ่าน 21/21; npm lint/build ผ่าน; Chrome headless กับ backend local + mock LINE: Get profile เรียก 1 ครั้งต่อ 2 ข้อความ, avatar/ชื่อในรายการ หัวแชท ผู้ส่ง และแท็บลูกค้าถูกต้อง, สติกเกอร์ 52002734 โหลดจาก LINE CDN จริงได้, ไม่มีบรรทัดสถานะตัวเชื่อม, ไม่ล้นที่ 390px, ไม่มี page error
- **ขอบเขต:** ยังไม่ commit/push/deploy; ตัวเชื่อม OA container บน server ยังรันอยู่ (ผู้ใช้ปิดเองด้วย sudo); หยุดบริการทดสอบที่สร้างเองแล้ว

## 2026-10-25 23:36 — แก้ frontend release ขึ้น Git ไม่ครบ (checksum mismatch บน server)
- **คำอธิบาย:** `docker-build.sh --full-stack` บน server ล้มด้วย "Frontend checksum mismatch" เพราะ `.gitignore` มี `data/` ซึ่ง ignore โฟลเดอร์ชื่อ data ทุกระดับ ทำให้ `dist/frontend-linux/node_modules/caniuse-lite/data/` 588 ไฟล์ไม่เข้า Git (local 2011 ไฟล์ใน node_modules, Git 1423) จึงจำกัดให้ ignore เฉพาะ `/data/` ที่ root ตามเจตนาเดิม (ข้อมูล MongoDB dev)
- **โฟลเดอร์ `./`**
  - `.gitignore` — แก้: `data/` → `/data/`
- **ผลตรวจสอบ:** หลังแก้ไม่มีไฟล์ใน dist/frontend-linux, dist/oa-connector-linux, dist/line-linux ถูก ignore; root data/ ยังถูก ignore; ไม่มีไฟล์ใหม่นอก dist โผล่ (billing_wth_text/ ยัง ignore ทั้งโฟลเดอร์); `sha256sum -c frontend.sha256` บน local ผ่าน 2108 ไฟล์ และ git attr ของไฟล์เหล่านี้ไม่แปลง line ending; release เดิม front-20261025153210-4e1f9f6df1b1 ใช้ต่อได้ไม่ต้อง build ใหม่
- **สถานะ server (อ่านอย่างเดียว):** อยู่ที่ commit 29f1ee0, Caddyfile ยังเป็นรุ่นเดิม (ผู้ใช้รัน `caddy validate` โดยไม่ระบุ --config จึงไปตรวจ config JSON ว่าง); ยังไม่ commit/push

## 2026-10-25 22:49 — ใช้ hermes.salepie.online เป็น URL ของหน้า Alex และ LINE Login
- **คำอธิบาย:** ผู้ใช้ให้ใช้ hermes.salepie.online ตรวจ server แบบอ่านอย่างเดียวพบว่า domain นี้ส่ง /api/line/webhook ไป backend 18080 และ path อื่นไป Hermes agent (aiohttp, localhost:8646) จึงใช้ร่วมกันด้วย Caddy path routing: เฉพาะ path ของ Alex ไป admin frontend 18081, webhook คงเดิม, ที่เหลือไป Hermes ตามเดิม; Hermes ตอบ 404 ทุก path ของ Alex จึงไม่ชนกัน
- **โฟลเดอร์ `docker/`**
  - `Caddyfile.line.example` — แก้: เลิกใช้ admin.salepie.online เปลี่ยนเป็น named matcher `@alex` (/, /_next/*, /favicon.ico, /api/health, /api/admin/*, /api/inquiries*, /api/conversations/*, /api/line/* ยกเว้น webhook, /api/draft-requests/*, /api/media/*) → 127.0.0.1:18081
- **โฟลเดอร์ `./` และ `docs/`**
  - `.env.example` — แก้: LINE_LOGIN_REDIRECT_URI=https://hermes.salepie.online/api/admin/line/callback และหมายเหตุ ADMIN_COOKIE_SECURE=true
  - `docs/step-2-admin.md` — แก้: ขั้นตอนแก้ Caddyfile พร้อมสำรอง/validate/reload, callback URL ของ LINE Login, ข้อจำกัดว่า runall.bat บนเครื่องใช้ได้เฉพาะรหัสผ่านสำรอง และคำแนะนำปิดพอร์ต 18081 จากภายนอก
- **ผลตรวจสอบ:** `caddy validate` (caddy:2.10 ใน Docker local) ผ่านทั้ง Caddyfile ใน docs และ docker/Caddyfile.line.example; ทดสอบ routing ด้วย Caddy จริงที่แทน upstream ด้วย respond: path ของ Alex → alex, /api/line/webhook → webhook, /v1/chat/completions, /api/other, /health, /admin → hermes; release ใน dist ไม่อ้างอิง Caddyfile จึงไม่ต้อง build ใหม่ (rel-20261025153117-65c57753fb02, front-20261025153210-4e1f9f6df1b1)
- **ขอบเขต:** ไม่ได้แก้ Caddy, .env, firewall หรือบริการบน server (ต้องใช้ sudo โดยผู้ใช้) และยังไม่ commit/push

## 2026-10-25 22:32 — เข้า Alex ด้วย LINE Login และตอบลูกค้าจาก Alex ผ่าน Messaging API (แทน login-oa.bat)
- **คำอธิบาย:** ผู้ใช้ไม่ต้องการใช้ login-oa.bat และต้องการ LINE Login API จริง ตรวจเอกสารแล้วพบว่า LINE Login มีแค่ scope profile/openid/email อ่านแชท OA Manager ไม่ได้ ผู้ใช้จึงเลือกให้แอดมินเข้า Alex ด้วย LINE Login และตอบลูกค้าจาก Alex ด้วย Push API (ขั้น B) ทำให้ข้อความเข้า MongoDB ด้วย API ทางการทั้งหมดโดยไม่ต้องใช้ตัวเชื่อม OA
- **โฟลเดอร์ `backend/src/`**
  - `routes/session.rs` — แก้: session cookie v2 เก็บผู้ล็อกอิน (password/line) ผูกกับรหัสผ่านหรือ channel + allowlist ทำให้ถอดสิทธิ์/เปลี่ยนรหัสแล้ว session เดิมใช้ไม่ได้; เพิ่ม `/api/admin/login-options`, `/api/admin/line/start` (state ลงนาม + nonce, cookie Lax เฉพาะ path) และ `/api/admin/line/callback` (แลก code, verify ID token กับ LINE, ตรวจ aud/allowlist, แสดง LINE user ID เมื่อไม่มีสิทธิ์); `actor()` คืน ID + ชื่อ; ปิด password login อัตโนมัติเมื่อไม่ได้ตั้งรหัส
  - `config.rs` — แก้: LINE_LOGIN_CHANNEL_ID/SECRET/REDIRECT_URI, ADMIN_LINE_USER_IDS และ LINE_API_BASE (ค่าเริ่มต้น https://api.line.me ใช้ทดสอบกับ mock)
  - `models/outbound.rs`, `db/outbound.rs` — เพิ่ม: ส่งข้อความด้วย Push API เฉพาะแชท 1:1, X-Line-Retry-Key = request UUID, บันทึก `outbound_messages` (SENDING/DONE/FAILED, line_message_id, ผู้ส่ง), ส่งซ้ำ request เดิมไม่ส่งสองครั้ง, request เดิมแต่ข้อความต่างถูกปฏิเสธ, แยก error 429/5xx/4xx
  - `routes/customers.rs`, `routes/mod.rs`, `error.rs`, `db/indexes.rs`, `db/mod.rs`, `models/mod.rs`, `models/customer.rs`, `db/customers.rs`, `db/orders.rs` — แก้: route `POST /api/inquiries/{id}/messages`, error REPLY_UNAVAILABLE/LINE_*, index outbound_messages, Actor เก็บ updated_by_id/approved_by_id
  - `db/oa_sync.rs`, `db/drafts.rs`, `order_draft_prompt.txt` — แก้: รวม outbound_messages ในแชท/pagination และบริบท AI เป็น source_type `admin_message` (ข้อความร้าน ไม่ใช่การยืนยันจากลูกค้า)
  - `tests_line.rs`, `tests.rs` — เพิ่ม/แก้: mock LINE server ทดสอบ state/nonce, ผู้ไม่อยู่ใน allowlist, cancel, ถอดสิทธิ์, login-options, Push idempotent/retry/429/กลุ่ม และ context hash เปลี่ยนหลังส่ง
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx` — แก้: ปุ่ม "เข้าสู่ระบบด้วย LINE" (รหัสผ่านเป็นทางสำรองเมื่อเปิดไว้), แสดงเหตุผลและ LINE user ID เมื่อไม่มีสิทธิ์, ช่องตอบลูกค้าใต้แชท (Ctrl+Enter, ส่งอีกครั้งด้วย request ID เดิม), ซ่อนสถานะตัวเชื่อม OA เมื่อไม่ได้ติดตั้ง
  - `components/chat-message.tsx`, `lib/api.ts`, `app/globals.css` — แก้: ป้าย "ส่งจาก Alex", ข้อความ error ภาษาไทย, สไตล์ช่องตอบ/หน้าล็อกอิน
- **โฟลเดอร์ `./` และ `docs/`**
  - `docker-stack.line.yml`, `.env.example`, `AGENTS.md` — แก้: ส่ง env LINE Login เข้า backend service (ตรวจด้วย docker stack config แล้ว) และตัวอย่าง/กฎ env
  - `docs/step-2-admin.md`, `docs/line-oa-auto-sync.md` — แก้: ขั้นตอนสร้าง LINE Login channel, callback URL, Roles/Publish, allowlist, การตอบลูกค้าจาก Alex/โควตา และระบุว่าตัวเชื่อม OA ไม่จำเป็นแล้ว
- **โฟลเดอร์ `dist/` (build จริงบน local)**
  - `line-linux/` — rel-20261025153117-65c57753fb02
  - `frontend-linux/` — front-20261025153210-4e1f9f6df1b1
- **ผลตรวจสอบ:** cargo fmt/clippy -D warnings ผ่าน; cargo test รวม MongoDB integration (disposable 27018) ผ่าน 19/19; npm lint/build ผ่าน; Chrome headless ผ่าน backend local + standalone frontend + mock LINE: ผู้ไม่มีสิทธิ์เห็น LINE user ID, ผู้มีสิทธิ์เข้าได้และแสดงชื่อ LINE, ส่งข้อความด้วย Ctrl+Enter ขึ้นขวา "ส่งจาก Alex", 500 จาก LINE แสดงปุ่มส่งอีกครั้งและ retry ใช้ key เดิมส่งครั้งเดียว, reload แล้วข้อความยังอยู่, ไม่ล้นที่ 390/800px, ไม่มี page error; ระหว่างดีบักสคริปต์ browser เคยเปิดหน้า access.line.me จริงด้วย channel ID ปลอมและ LINE ปฏิเสธ (400) ไม่มีการล็อกอินหรือส่งข้อความจริง
- **ขอบเขต:** ยังไม่ commit/push/deploy และยังไม่ได้ทดสอบกับ LINE Login channel/OA จริง ผู้ใช้ต้องสร้าง LINE Login channel และตั้ง .env บน server; หยุด backend/frontend/mongod/mock ทดสอบที่สร้างเองแล้ว คง frontend 3000 ของผู้ใช้ไว้; ตัวเชื่อม OA ยังรันอยู่บน server (ไม่ได้แตะ)

## 2026-10-25 21:45 — ขั้น A: บันทึกข้อมูลลูกค้าและอนุมัติร่างเป็นออเดอร์ใน Alex
- **คำอธิบาย:** ผู้ใช้ต้องการใช้ Alex เป็น UI บันทึกข้อมูลลง MongoDB บน Ubuntu หลังล็อกอิน OA ให้ตัวเชื่อม จึงแบ่งงานเป็นขั้น A (ข้อมูลลูกค้า + อนุมัติออเดอร์) และขั้น B (ตอบลูกค้าจาก Alex ด้วย Push API ซึ่งผู้ใช้อนุมัติแล้วแต่ยังไม่ทำในรอบนี้)
- **โฟลเดอร์ `backend/src/`**
  - `models/customer.rs` — เพิ่ม: ProfileInput (trim, จำกัดความยาว, ตรวจเบอร์/อักขระควบคุม, deny_unknown_fields) และ ApproveInput พร้อม unit test
  - `db/customers.rs` — เพิ่ม: อ่าน/บันทึก `customer_profiles` ต่อ shop/channel/line_user_id เฉพาะผู้ใช้ที่มีใน line_users, optimistic `version` และ updated_by
  - `db/orders.rs` — เพิ่ม: อนุมัติได้เฉพาะร่าง active ล่าสุดที่เป็น READY_FOR_REVIEW, context_hash ตรงและไม่มีข้อความรอประมวลผล/ถูกยกเลิก; snapshot ฟิลด์ลง `orders` (CONFIRMED, approved_by/at, verified flags = false) 1 ออเดอร์ต่อ inquiry ด้วย unique index, กดซ้ำร่างเดิมคืนผลเดิม และเก็บ order_id ใน inquiry โดยไม่แตะ status ที่ worker ใช้
  - `routes/customers.rs`, `routes/mod.rs` — เพิ่ม/แก้: GET/POST `/api/line/users/{user_id}/profile`, GET/POST `/api/inquiries/{id}/order` ภายใต้ auth เดิม (cookie + X-Admin-Request หรือ Bearer)
  - `routes/session.rs` — แก้: เพิ่ม `actor()` ใช้ชื่อแอดมินจาก session หรือ internal-api สำหรับ Bearer token
  - `error.rs`, `db/indexes.rs`, `db/mod.rs`, `models/mod.rs` — แก้: error RECORD_CONFLICT/ORDER_EXISTS/ORDER_NOT_READY (409) และ unique index ของ customer_profiles/orders แบบ idempotent
  - `tests_customers.rs`, `tests.rs` — เพิ่ม/แก้: ตรวจ auth/DTO ก่อนแตะ DB และ MongoDB integration ของ profile version/unknown user และเงื่อนไขอนุมัติ (ไม่มีร่าง, ร่างไม่ครบ, hash ผิด, อนุมัติซ้ำ, ORDER_EXISTS)
- **โฟลเดอร์ `frontend/src/`**
  - `components/customer-profile.tsx` — เพิ่ม: แท็บข้อมูลลูกค้า (ชื่อ เบอร์ ที่อยู่ โน้ตภายใน) แสดงผู้บันทึกล่าสุดและโหลดข้อมูลล่าสุดเมื่อชนกัน
  - `components/admin-workspace.tsx` — แก้: แท็บ "ลูกค้า", การ์ดอนุมัติพร้อม checkbox ยืนยัน, แจ้งออเดอร์ที่อนุมัติแล้ว, ป้าย "ออเดอร์แล้ว" ในรายการ และ poll สถานะออเดอร์
  - `lib/api.ts`, `app/globals.css` — แก้: types Profile/Order, ข้อความ error ภาษาไทย, สไตล์ notice/pill success และ approve-card
- **โฟลเดอร์ `docs/`**
  - `step-2-admin.md` — แก้: อธิบายข้อมูลลูกค้า การอนุมัติออเดอร์ ข้อจำกัด และ API ใหม่
- **โฟลเดอร์ `dist/` (build จริงบน local)**
  - `line-linux/` — rel-20261025144406-05a2895ef532
  - `frontend-linux/` — front-20261025144514-4c469a4e318b
- **ผลตรวจสอบ:** cargo fmt/clippy -D warnings ผ่าน; cargo test 10 ผ่าน และ MongoDB integration (disposable mongod 27018) ทั้ง 5 ผ่าน; npm lint/build ผ่าน; ทดสอบ browser จริง (Chrome headless) ผ่าน backend local 8080 + standalone frontend 3010 + Mongo ทดสอบ: webhook ลงนาม → บันทึกโปรไฟล์ → ร่างด้วยมือ → ปุ่มอนุมัติ disabled จนกว่าจะติ๊ก → อนุมัติ → ป้ายออเดอร์แล้ว → reload แล้วข้อมูลยังอยู่ ไม่มี page error และไม่ล้นจอที่ 390/800px; git diff --check ผ่าน
- **ขอบเขต:** ยังไม่ commit/push/deploy; หยุด backend/frontend/mongod ทดสอบที่สร้างเองแล้ว คง frontend 3000 ของผู้ใช้ไว้ ขั้น B (ตอบลูกค้าผ่าน Push API + กันข้อความซ้ำกับตัวเชื่อม OA) ยังไม่เริ่ม

## 2026-10-25 20:55 — แก้ login-oa.bat ล้มเมื่อมี ssh.exe หลายตัวใน PATH
- **คำอธิบาย:** เครื่องผู้ใช้มี ssh.exe ทั้ง Windows OpenSSH และ Git usr/bin ทำให้ Get-Command คืน array และ Start-Process -FilePath ล้มด้วย "Cannot convert System.Object[]" จึงเลือกตัวแรกใน PATH แบบเดียวกับ run-frontend.ps1
- **โฟลเดอร์ `scripts/`**
  - `login-oa.ps1` — แก้: ใช้ `Select-Object -First 1` ก่อนอ่าน `.Source` ของ ssh.exe
- **ผลตรวจสอบ:** PowerShell parser ผ่าน; ssh ที่เลือกคือ C:\Windows\System32\OpenSSH\ssh.exe; ทดสอบ tunnel 127.0.0.1:18082 ไป Hermes ได้ /health = ok และหน้าควบคุมตอบ 401 (ต้อง Basic auth ตามที่ออกแบบ) แล้วปิด tunnel ที่สร้างเอง
- **สถานะ Ubuntu (อ่านอย่างเดียว):** server อยู่ที่ commit 91cf6e7, connector tag oa-20261025082952-9a6bfea570b9 listen 127.0.0.1:18082 และ backend 18080 health ok; ไม่ใช้ sudo ไม่แก้บริการบน server
- **หมายเหตุ:** `bash ./loadgit.sh` ที่ผู้ใช้รันบน Windows เรียก WSL ที่ไม่มี /bin/bash จึงล้ม — สคริปต์ .sh ต้องรันบน Ubuntu ผ่าน SSH ไม่ใช่ปัญหาของ repo

## 2026-10-25 20:32 — ให้ runall ใช้ frontend เดิมและเปิด SSH tunnel กลับมาได้
- **คำอธิบาย:** พบ Next.js ของโปรเจกต์ยังทำงานที่พอร์ต 3000 แต่ SSH tunnel 28080 หลุด ทำให้ API ตอบ 500 และ launcher เดิมปฏิเสธการรันซ้ำ จึงเพิ่มการตรวจและใช้บริการเดิม พร้อมเปิด tunnel ใหม่เมื่อขาดการเชื่อมต่อ
- **โฟลเดอร์ `scripts/`**
  - `run-frontend.ps1` — แก้: ตรวจเจ้าของพอร์ตจาก process และ API rewrite ใน dev manifest ก่อนใช้ frontend เดิม ตรวจ SSH executable/forward/host ให้ตรง config ก่อนใช้ tunnel เดิม ตรวจ health ผ่าน frontend จริง และคงหน้าต่างไว้เมื่อสร้าง tunnel ให้ frontend ที่เปิดอยู่ ไม่ปิด process อื่นหรือเปิด backend/MongoDB ในเครื่อง
- **โฟลเดอร์ `docs/`**
  - `step-2-admin.md` — แก้: อธิบายการรันซ้ำ การเปิด tunnel ที่หลุดกลับมา การใช้ ProbeOnly กับ tunnel เดิม และหน้าต่างที่ต้องเปิดค้างไว้
- **ผลตรวจสอบ:** PowerShell parser และ runall -Check ผ่าน; ทดสอบ runall ซ้ำและ ProbeOnly ใช้ frontend/SSH เดิมผ่าน; ปฏิเสธ frontend ที่ชี้ backend คนละตัว, SSH host/remote port คนละค่า และพอร์ตของโปรแกรมอื่นโดย listener เดิมยังทำงาน; frontend localhost:3000 และ tunnel 28080 ตอบ /api/health เป็น ok หน้าเว็บ HTTP 200; BAT ASCII/CRLF และ git diff --check ผ่าน
- **สถานะทดสอบจริง:** คง frontend PID 58692 ไว้และเปิด SSH tunnel ใหม่ให้เชื่อม backend Ubuntu ได้แล้ว ไม่แก้บริการหรือ build โปรแกรมบน Ubuntu และยังไม่ commit/push

## 2026-10-25 15:40 — เพิ่มตัวเชื่อม OA Manager บน Ubuntu พร้อม release สำหรับทดสอบบัญชีจริง
- **คำอธิบาย:** ผู้ใช้ขอให้ทำตัวเชื่อมอัตโนมัติต่อ จึงเพิ่ม browser companion บน Ubuntu และหน้าล็อกอินผ่าน SSH ให้ใช้ OA Manager ตามเดิมโดยไม่คัดลอกข้อความ ไม่ส่งข้อความลูกค้าซ้ำ ไม่แก้ LINE webhook/Caddy/DNS และไม่ให้เครื่องแอดมินต้องเปิด browser ค้างเมื่อ server เชื่อมแล้ว
- **โฟลเดอร์ `connectors/line-oa/`**
  - `package.json`, `package-lock.json`, `build.mjs` — เพิ่ม: npm package ที่ pin Playwright 1.64.1 และ esbuild 0.28.2; bundle โปรแกรมบน local แล้วส่งเฉพาะ compiled JavaScript และ production dependencies ไป server
  - `src/index.mjs`, `src/adapter.mjs` — เพิ่ม: persistent Chromium profile, read-only OA API requests, CSRF token ในหน่วยความจำ, poll ค่าเริ่มต้น 30 วินาที, ตรวจ bot ID/บทบาทผู้ส่ง/เวลา/ID/pagination, หยุดเมื่อ schema ไม่รองรับ, จับคู่ข้อความแอดมินกับ incoming message ID ที่ได้รับจาก webhook และ heartbeat ไม่มี log เนื้อหาแชท/cookie/password
  - `src/control.mjs`, `src/control.html` — เพิ่ม: หน้าควบคุมล็อกอิน LINE ใน browser บน Ubuntu ผ่าน SSH; Basic auth ใช้บัญชีหลังบ้านเดิม, ตรวจ custom header/origin สำหรับคำสั่ง, screenshot ในหน่วยความจำ, รองรับ popup login; เปิดเผย diagnostics เฉพาะชื่อฟิลด์/ชนิดข้อมูล
  - `test/adapter.test.mjs`, `test/control.test.mjs` — เพิ่ม: ตรวจ explicit roles/anchors, timestamp/ID/pagination ที่ไม่แน่ใจ, diagnostics ไม่มีค่าลูกค้า และ auth/CSRF/ห้าม arbitrary navigation
- **โฟลเดอร์ `backend/src/`**
  - `models/oa_sync.rs`, `db/oa_sync.rs`, `routes/oa_sync.rs` — เพิ่ม: connector ingestion ที่รับเฉพาะ Bearer token, ตรวจ destination และ anchors ในร้าน/แชนแนล/บทสนทนาเดียวกัน, แยก collection oa_messages, dedup และปฏิเสธ ID เดิมแต่ข้อมูลขัดกัน, heartbeat/stale status และ merge ข้อความแอดมินกับลูกค้าพร้อม pagination
  - `routes/admin.rs`, `routes/mod.rs`, `db/indexes.rs`, `db/mod.rs`, `models/mod.rs`, `error.rs` — แก้: แยก token verifier ที่ใช้ร่วมกัน เพิ่ม routes/errors/indexes แบบ idempotent โดยคง session/CSRF ของ browser เดิม
  - `db/drafts.rs`, `draft_worker.rs`, `order_draft_prompt.txt` — แก้: รวมแอดมินจากตัวเชื่อมใน snapshot/hash/evidence เป็น oa_message แยกจาก signed customer webhook และไม่ถือเป็นการยืนยันลูกค้า ข้อความใหม่ทำให้ร่างเก่าถูกตรวจว่าบริบทเปลี่ยน
  - `tests_oa_sync.rs`, `tests.rs` — เพิ่ม/แก้: auth/DTO ก่อน DB, isolation, anchors, dedup/conflict, context hash, mixed-source pagination และ status
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx`, `components/chat-message.tsx` — แก้: แอดมินขวาพื้นฟ้าอ่อนพร้อมป้ายจาก OA Manager, แสดงสถานะตัวเชื่อม, poll บริบทที่โหลดอยู่ทุก 10 วินาทีโดยไม่ล้างสรุปที่พิมพ์ค้างไว้ ไม่มีช่องบันทึกข้อความเอง
- **โฟลเดอร์ `./`, `scripts/`, `docker/`**
  - `build-oa-connector.bat`, `scripts/build-oa-connector.ps1`, `docker/oa-connector.build.Dockerfile` — เพิ่ม: ตรวจ local named-pipe Docker Desktop/builder, build/test Linux release บน local, checksum/tag และเก็บใน dist/oa-connector-linux
  - `docker/oa-connector.Dockerfile`, `deploy-oa-connector.sh` — เพิ่ม: runtime packaging จาก compiled objects, Playwright/browser รุ่นตรงกัน, profile volume แยก, non-root process, private bind 127.0.0.1:18082, restart unless-stopped, ตรวจ manifest/tag/config ก่อนแก้ container ไม่ย้าย stack หรือ Mongo volume เดิม
  - `login-oa.bat`, `scripts/login-oa.ps1` — เพิ่ม: เปิดหน้าควบคุมล็อกอินผ่าน SSH alias Hermes และปิดเฉพาะ tunnel ที่สร้างเอง ไม่ปิด browser service บน Ubuntu
  - `scripts/build-frontend.ps1` — แก้: บันทึก frontend-release.json เป็น LF เพื่อคง byte ของ Linux artifact และผ่าน diff/checksum checks
  - `.env.example`, `.gitignore`, `.dockerignore`, `.gitattributes`, `AGENTS.md` — แก้: config ตัวอย่าง, รวม compiled connector/runtime dependencies ใน Git/Docker context แบบไม่แปลง bytes, คง profile/data/env/secrets ไว้นอก Git และเพิ่มคำสั่ง/ข้อกำหนด deployment
- **โฟลเดอร์ `docs/` และ guideline**
  - `line-oa-auto-sync.md`, `step-2-admin.md`, `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: ขั้นตอนแยกผู้พัฒนา/ผู้ดูแล/เจ้าของ OA, local build → Git → Ubuntu package/deploy → login-oa.bat, states/ข้อจำกัด/schema calibration; คง embedded images เดิมทั้ง 4 ภาพครบ
- **โฟลเดอร์ `dist/` (build จริงบน local)**
  - `line-linux/` — rel-20261025081859-c575b8b9f490
  - `frontend-linux/` — front-20261025083658-8588f12fd6a4
  - `oa-connector-linux/` — oa-20261025082952-9a6bfea570b9 พร้อม compiled connector, control UI, production dependencies และ manifest
- **ผลตรวจสอบ:** cargo fmt/clippy ผ่าน, unit tests 8 ผ่าน และ MongoDB integration tests ทั้ง 4 ผ่าน; npm lint/build ผ่านทั้ง Windows และ Linux; connector tests ทั้ง 4 ผ่าน; runtime Docker images ของ connector/backend build ผ่าน; real runtime Chromium → backend → MongoDB → frontend ด้วย fixture ผ่านทั้งข้อความเข้าด้านขวา สีพื้นถูกต้อง auto-refresh ไม่ล้างสรุป dedup บล็อก LINE POST และ auth ของหน้าควบคุม; unknown schema/unlinked anchors ไม่เพิ่มข้อมูล; ทดสอบ browser runtime เปิดหน้า LINE Business ID จริงได้และแสดง NEEDS_LOGIN โดยยังไม่ล็อกอิน; deploy mock ตรวจ private bind/runtime-only/env-only secrets และ manifest/config guards ผ่าน; secret scan, Git inclusion, BAT ASCII/CRLF, Linux LF, frontend checksum, runall remote preflight และ git diff --check ผ่าน
- **ขอบเขต:** ยังไม่ commit/push, deploy หรือเข้า LINE ด้วยบัญชีจริง ผู้ใช้เลือกเป็นผู้รัน sudo Docker เอง ต้องส่ง Git แล้ว deploy backend/connector และล็อกอิน LINE ผ่าน login-oa.bat ก่อน จากนั้นต้องยืนยัน schema ของ OA ปัจจุบัน รุ่นแรกเก็บข้อความตัวอักษรและแชท 1:1 ที่มี webhook anchor ไม่รับรองประวัติเก่าหรือ media ขาออกครบทั้งหมด; ตรวจ server แบบอ่านอย่างเดียวพบ x86_64, RAM available ประมาณ 1.8 GB และ disk ว่างประมาณ 65 GB
- **ไฟล์ช่วยทดสอบ `data/` (ignored):** test-oa-runtime.py, oa-runtime-preload.mjs, test-oa-login.py, test-oa-deploy.sh และ screenshots ใช้ fixture หรือหน้า login สาธารณะที่ยังไม่เข้า account ไม่ใช้แชทจริง หยุด disposable MongoDB 27018 และ test containers/processes ที่สร้างเองแล้ว คง frontend 3000/SSH 28080 ของผู้ใช้ไว้

## 2026-10-25 14:52 — ยกเลิกช่องบันทึกเองและตรวจแนวทางซิงก์แอดมินจาก OA อัตโนมัติ
- **คำอธิบาย:** ผู้ใช้เปลี่ยนข้อกำหนดเป็นรับข้อความแอดมินจาก OA Manager อัตโนมัติ จึงเอาแบบฟอร์ม/API/manual-copy persistence ที่กำลังพัฒนาออกทั้งหมด โดยไม่สร้างข้อความจากภาพตัวอย่างหรือบันทึกข้อความใดลง production
- **โฟลเดอร์ `frontend/`**
  - `src/components/chat-message.tsx` — เพิ่ม: แยกลูกค้าด้านซ้าย/แอดมินด้านขวาจาก sender_role/direction, พื้นแอดมินสีฟ้าอ่อน, ป้ายผู้ส่งและเวลาส่ง, ยังใช้ authenticated media/withdrawn handling เดิม
  - `src/components/admin-workspace.tsx`, `src/lib/api.ts`, `src/app/globals.css` — แก้: ใช้ component แชทสองฝั่ง เพิ่มชนิดบทบาท/ทิศทางข้อความและป้องกันข้อความซ้ำเมื่อโหลดต่อ ไม่มีช่องคัดลอก/บันทึกข้อความเอง ไม่มีการอ้างว่าข้อความแอดมินเข้าแล้ว
  - `src/components/admin-message-import.tsx` — ลบ: ไฟล์ที่ agent เพิ่งสร้างในงานที่ยกเลิก ยังไม่เคย commit/deploy
- **Backend:** เอาโค้ด manual-copy ที่ agent เพิ่งแก้/เพิ่มออกทั้งหมด ปัจจุบัน backend ไม่มี diff จาก HEAD ของผู้ใช้ ไม่เพิ่ม endpoint หรือ collection สำหรับกรอกข้อความเอง ไม่แก้ compiled release และไม่แก้ฐาน production
- **โฟลเดอร์ `docs/`**
  - `line-oa-auto-sync.md` — เพิ่ม: ผลตรวจเอกสาร LINE webhook/module และตัวอย่าง API ภายใน OA Manager, เปรียบเทียบตัวอ่านข้อมูลบนเครื่องแอดมินกับ browser service บน Ubuntu, เงื่อนไข session และข้อกำหนดทดสอบกับบัญชีจริงก่อนยืนยันว่า sync ได้
  - `step-2-admin.md` — แก้: อ้างอิงแนวทาง auto-sync และระบุสถานะว่ายังไม่มีการซิงก์ข้อความแอดมินจาก OA Manager
- **ผลค้นคว้า:** webhook ปกติในเอกสาร LINE รับ message จากผู้ใช้ ไม่มี event แอดมินขาออกที่ตรวจพบ; module ไม่ใช่หลักฐานว่ารับแอดมินได้; พบตัวอย่างเก่าของ GET chat.line.biz/api/v1/bots/{botId}/messages/{chatId} ซึ่งใช้ web session แต่ยังไม่ได้ทดสอบกับ OA ปัจจุบัน ไม่ถือเป็น public API ที่ LINE รับรอง
- **ผลตรวจสอบ:** npm lint/build ผ่าน; cargo fmt --check, clippy -- -D warnings และ cargo test ผ่าน 7 tests (3 MongoDB integration tests ถูก ignore ตามเดิม); Chrome fixture ที่ใช้ ChatMessage จริงตรวจลูกค้าซ้าย/แอดมินขวา พื้นฟ้าอ่อน ข้อความ HTML ถูก escape และไม่ล้นที่ความกว้าง 390/800 ผ่าน; git diff --check ผ่าน
- **ขอบเขต/งานที่ยังไม่เสร็จ:** ยังไม่ได้สร้าง ติดตั้ง หรือล็อกอินตัวเชื่อมอัตโนมัติจริง ไม่ยืนยันว่ารับข้อความ OA ได้ครบ ผู้ใช้ถามความต่างของ Chrome/Ubuntu และต้องตัดสินใจตำแหน่งที่รัน/ทดสอบ session ต่อ; หยุด disposable mongod 27018 ที่สร้างเองในงาน manual-copy ที่ยกเลิกแล้ว ไม่ปิด frontend/SSH ของงานเดิม ไม่มี commit/push หรือแก้ Caddy/LINE webhook

## 2026-10-25 14:18 — เปิด frontend local เชื่อม backend Ubuntu สำเร็จ
- **คำอธิบาย:** ผู้ใช้ให้รันเฉพาะ frontend ที่เชื่อม backend บน Ubuntu จึงเรียก runall.bat -RemoteBackend และยืนยันว่า runall.bat -Check ปกติใช้ remote mode จาก .env เช่นกัน
- **บริการ local:** Next.js เปิด http://localhost:3000 และ SSH tunnel เปิด 127.0.0.1:28080 ไป backend Ubuntu 127.0.0.1:18080; ไม่เริ่ม Rust backend หรือ MongoDB ในเครื่อง ตรวจไม่พบ listener 8080/27017
- **ผลตรวจสอบจริง:** เรียก health/login/session ผ่าน frontend proxy ได้ HTTP 200; อ่าน inquiries จาก Ubuntu ได้ 1 รายการ, inquiry context HTTP 200 และ events DONE 7 รายการ ไม่แสดงรหัส/token/cookie หรือข้อความลูกค้าในผลตรวจ
- **ขอบเขต:** เปิด frontend และ tunnel ค้างไว้ให้ผู้ใช้ทดสอบ ไม่แก้ source/config, ไม่ build/commit/push และไม่แก้บริการ Ubuntu

## 2026-10-25 13:50 — เชื่อม frontend local dev กับ backend Ubuntu ผ่าน SSH
- **คำอธิบาย:** frontend อ่าน backend Ubuntu ตัวเดียวกับ LINE webhook ผ่าน SSH tunnel โดยไม่เปิด backend เป็น public
- **โฟลเดอร์ `./` และ `scripts/`**
  - `run-frontend.bat`, `scripts/run-frontend.ps1` — เพิ่ม: ตรวจ SSH/config/พอร์ต เปิด tunnel 127.0.0.1:28080 ไป Ubuntu 127.0.0.1:18080 ตั้ง BACKEND_URL ตรวจ health/admin API และ cleanup เฉพาะ SSH child ของ launcher; รองรับ -Check/-ProbeOnly และแจ้งให้ deploy เมื่อ backend รุ่นเก่าตอบ 404
  - `scripts/runall.ps1` — แก้: รองรับ DEV_BACKEND_MODE, -RemoteBackend และ -LocalBackend; remote mode ไม่เรียก cargo หรือเริ่ม MongoDB/backend ในเครื่อง
  - `.env` — แก้ local/ignored: remote mode, SSH alias Hermes และพอร์ต โดยไม่เปลี่ยน credentials
  - `.env.example` — แก้: เพิ่ม remote dev config และลบรหัสผ่านจริงที่ติดมาในตัวอย่าง ให้ ADMIN_PASSWORD ว่าง
  - `AGENTS.md` — แก้: เพิ่มคำสั่ง frontend remote
- **โฟลเดอร์ `frontend/` และ `docs/`**
  - `frontend/.env.development.local` — เพิ่ม/แก้ local/ignored: BACKEND_URL=http://127.0.0.1:28080 สำหรับ npm dev โดยไม่มี token/password; production build ไม่ใช้ไฟล์ development นี้
  - `docs/step-2-admin.md` — แก้: วิธี frontend local + Ubuntu backend, SSH/config, บัญชี server, หยุด tunnel, สลับกลับ local และข้อกำหนด backend Step 2
- **Ubuntu:** เรียก loadgit.sh main แล้ว server เป็น commit 608551c ซึ่งมี compiled release จาก local; ไม่ compile/build โปรแกรมบน server ไม่เปลี่ยน Caddy/webhook/ฐานข้อมูล และไม่เรียก Docker ด้วย sudo ตามที่ผู้ใช้เลือกเป็นผู้รัน sudo เอง
- **ผลตรวจสอบ:** npm lint/build, PowerShell syntax, remote/local preflight, invalid-port/busy-port guards และ BAT ASCII/CRLF ผ่าน; config ทั้งสองไฟล์ยัง gitignored; SSH tunnel จริงและ health ผ่าน แต่ admin/session ยังตอบ 404 จึงปฏิเสธรุ่นเก่าและ cleanup tunnel ได้ถูกต้อง; Chrome isolated fixture ทดสอบ Secure/HttpOnly cookie บน http://localhost ผ่าน
- **ข้อจำกัดปัจจุบัน:** ยังไม่ยืนยัน login/inquiry ผ่าน frontend กับ backend ใหม่ เพราะต้องให้ผู้ใช้รัน sudo bash ./docker-build.sh และ sudo bash ./docker-deploy.sh ก่อน ไม่มี commit/push โดย agent และไม่เปลี่ยน compiled Linux artifacts

## 2026-10-25 13:39 — ตรวจข้อความ LINE ไม่แสดงในหน้าหลังบ้าน local
- **คำอธิบาย:** หน้าเว็บ local อ่าน backend localhost:8080 และ MongoDB localhost/database myapp ซึ่งไม่มี events หรือ inquiries แต่ LINE webhook เปิดใช้งานและส่งเข้า https://hermes.salepie.online/api/line/webhook บน Ubuntu
- **ผลตรวจสอบ:** LINE bot ตรงกับ destination ที่ตั้งไว้; Ubuntu /api/health ผ่าน, line_events DONE 7 รายการ และ inquiries 1 รายการใน database alex_line จึงยืนยันว่าข้อความถูกเก็บบน server ไม่ใช่ฐาน local ไม่มีการแสดงข้อความลูกค้าหรือ credentials ในผลตรวจ
- **สถานะ deploy:** Ubuntu ยังเป็น commit d5a2d14; /api/admin/session ตอบ 404, frontend 18081 ยังไม่เปิด, ยังไม่มี admin.salepie.online ใน Caddy และ DNS ไม่ resolve ต้องส่ง compiled releases ขึ้น Git แล้ว pull/package runtime/deploy full-stack พร้อมตั้ง admin DNS/Caddy ตาม docs/step-2-admin.md เพื่อให้หน้าหลังบ้านอ่านข้อมูลชุดเดียวกับ webhook
- **ขอบเขต:** ตรวจแบบ read-only ไม่เปลี่ยน webhook/ฐานข้อมูล/config ไม่สร้างข้อมูลจำลอง ไม่ build/commit/push หรือแก้บริการ production

## 2026-10-25 13:35 — ยืนยันล็อกอินหลัง backend โหลดรหัสใหม่
- **คำอธิบาย:** ตรวจภาพ error แล้วพบช่วงแรก local backend ยังเป็น process เดิม 54568 และ login ด้วยค่าปัจจุบันใน .env ได้ 401 ไม่มี backend/.env ที่ทับ root config ระหว่างตรวจ process เดิมถูกหยุดและมี backend ใหม่ 8584 ฟังพอร์ต 8080
- **ผลตรวจสอบ:** เรียก POST /api/admin/login ผ่าน frontend http://localhost:3000 โดยอ่าน credentials จาก .env ในหน่วยความจำ ได้ HTTP 200 และตรวจ GET /api/admin/session ด้วย cookie jar เดียวกันได้ HTTP 200 จึงยืนยันว่า backend ใหม่โหลดรหัสล่าสุดและ session ผ่าน frontend proxy ทำงานแล้ว ไม่พิมพ์รหัส/token/cookie ในผลตรวจ
- **ขอบเขต:** ไม่แก้ source/config ไม่หยุดหรือเริ่ม process ด้วย agent ไม่สร้าง restart script เนื่องจาก backend ถูก restart และตรวจสำเร็จแล้ว ไม่มีการ build/commit/push หรือแก้บริการบน Ubuntu

## 2026-10-25 12:23 — ตรวจรหัสล็อกอินและปรับให้ตรงกับรหัสล่าสุด
- **คำอธิบาย:** พบว่ารหัสที่ผู้ใช้แจ้งล่าสุดต่างจากค่าที่บันทึกก่อนหน้าหนึ่งตัวอักษร ทดสอบรหัสเดิมจาก .env แล้วได้ HTTP 200 ทั้ง backend 8080 และ frontend proxy 3000 จึงยืนยันว่าเส้นทาง login ทำงาน และเปลี่ยนเฉพาะรหัสให้ตรงกับค่าล่าสุดโดยรับผ่าน hidden input
- **โฟลเดอร์ `./` (local)**
  - `.env` — แก้: ADMIN_PASSWORD เป็นค่าล่าสุด คง username/admin cookie/config อื่น; ไม่พิมพ์รหัสหรือ token ในผลตรวจ
- **โฟลเดอร์ `/home/ubuntu/alex-connectingpoint/` บน Ubuntu**
  - `.env` — แก้: อัปเดตรหัสเดียวกันและรักษาสิทธิ์ 600; ยังไม่ได้ redeploy Docker
- **ผลตรวจสอบ:** ไม่มี admin key ซ้ำหรือ ADMIN_PASSWORD override ใน environment ของ agent; local backend เป็น binary ของ workspace และ proxy ตอบ login ได้ด้วยค่าเดิม ก่อน restart การลองรหัสใหม่ยังได้ 401 เนื่องจาก process เก็บ config ตอน startup; Ubuntu ยังอยู่ commit d5a2d14 และ admin/session ตอบ 404 ไม่มี frontend 18081
- **ข้อจำกัด:** คำสั่งหยุด/เริ่ม local backend ถูก automatic approval review ปฏิเสธด้วยข้อความ blocked by policy ก่อนรัน จึงไม่ได้หยุด process หรือเริ่มบริการใหม่ ผู้ใช้ต้อง Ctrl+C ในหน้าต่าง backend แล้วเรียก runall.bat เพื่อโหลดรหัสใหม่ ไม่มีการแก้ source/build/commit/push

## 2026-10-25 11:58 — ตั้งบัญชีแอดมินสำหรับทดสอบหน้าหลังบ้าน
- **คำอธิบาย:** ใช้รหัสที่ผู้ใช้กำหนดตั้ง ADMIN_PASSWORD และชื่อผู้ใช้ admin โดยรับรหัสผ่านผ่าน hidden input ไม่ใส่ใน script, command arguments, log หรือไฟล์ที่เข้า Git
- **โฟลเดอร์ `./` (local)**
  - `.env` — แก้: ตั้ง ADMIN_USERNAME/ADMIN_PASSWORD และ ADMIN_COOKIE_SECURE=false สำหรับ local HTTP; คง config อื่นไว้
- **โฟลเดอร์ `/home/ubuntu/alex-connectingpoint/` บน Ubuntu**
  - `.env` — แก้: ตั้งบัญชีเดียวกันและ ADMIN_COOKIE_SECURE=true สำหรับ HTTPS พร้อมรักษาสิทธิ์ไฟล์ 600; ต้อง deploy/restart backend เพื่อโหลดค่าใหม่
- **ผลตรวจสอบ:** .env ถูก ignore ทั้ง local/server; server backend health ผ่าน แต่ยังอยู่ commit d5a2d14 ไม่มี admin stack/พอร์ต 18081 และ Hermes API 8642 ยังไม่เปิด; ยังไม่มี Caddy site/DNS ที่ตรวจพบสำหรับ admin.salepie.online จึงต้อง push/pull compiled Step 2 release และตั้ง HTTPS ก่อนทดสอบบน server
- **ขอบเขต:** ไม่แสดงหรือบันทึกรหัสจริงใน log นี้ ไม่มีการ commit/push, compile บน Ubuntu หรือเปลี่ยนบริการ Docker/Caddy ของ server

## 2026-10-25 00:20 — เพิ่มภาพ infographic Step 3–4 (Hermes เชื่อม Alex ผ่าน MCP)
- **คำอธิบาย:** ผู้ใช้ส่งภาพอธิบายตำแหน่ง MCP Client, Alex Connector / MCP Server และ Alex REST API สำหรับ Step 3–4 จึงแปลง PNG 2.1 MB เป็น JPEG คุณภาพ 88 (≈515 KB, 1024×1536) แล้วฝังแบบ base64 ตามรูปแบบภาพเดิมในไฟล์
- **โฟลเดอร์ `./` (root)**
  - `alex-line-hermes-mcp-workflow.html` — แก้: เพิ่ม `<figure id="step3-4-mcp-diagram">` พร้อม alt และ figcaption ภาษาไทย ใต้ Input/เจ้าของงานหลักของหัวข้อ 6 (Step 3) โดย caption ลิงก์ไปหัวข้อ 7 (Step 4) และหัวข้อ 9 (MCP)

## 2026-10-25 00:20 — สร้างหน้าหลังบ้านแอดมินและ Step 2 เตรียมร่างออเดอร์
- **คำอธิบาย:** สร้าง frontend และ backend ให้แอดมินเลือก inquiry อ่านแชท/รูป ใส่สรุป สั่ง Hermes สกัดร่าง ตรวจข้อมูล แก้ไขด้วยมือ ยืนยันข้อมูล และดูประวัติเวอร์ชัน โดยยังไม่เช็คคิว ส่งข้อความถึงลูกค้า หรือ submit order
- **โฟลเดอร์ `backend/`**
  - `Cargo.toml` — แก้: เปิด feature json ของ reqwest เดิมโดยไม่เพิ่ม crate ใหม่; Cargo.lock คงอยู่ใน repository
  - `src/config.rs`, `src/state.rs`, `src/error.rs`, `src/main.rs` — เพิ่ม/แก้: config บัญชีแอดมิน/Hermes, login throttling, error codes ที่ไม่แสดง secret และแยก draft worker ที่ใช้ MongoDB queue
  - `src/routes/session.rs`, `src/routes/mod.rs`, `src/routes/admin.rs` — เพิ่ม/แก้: login/logout/session ด้วย signed HttpOnly SameSite=Strict cookie อายุ 8 ชั่วโมงและ Secure สำหรับ production, X-Admin-Request สำหรับ mutation, คง Bearer auth สำหรับเครื่องมือภายใน และ JSON error รูปแบบเดียวกัน
  - `src/routes/drafts.rs`, `src/db/drafts.rs`, `src/db/indexes.rs`, `src/db/mod.rs` — เพิ่ม/แก้: API เตรียม/แก้/อ่านร่างและสถานะงาน, collections admin_summaries/draft_requests/order_drafts, idempotency ต่อ request ID, expected version/context hash, lease/fencing/recovery และ atomic active-draft pointer โดยไม่ใช้ replica-set transactions
  - `src/models/draft.rs`, `src/models/mod.rs`, `src/order_draft_prompt.txt`, `src/draft_worker.rs` — เพิ่ม: schema และ validators สำหรับ Gregorian date, quantity, integer-satang, evidence source/quote, missing/conflicts/questions, latest confirmed fields, ภาพจาก GridFS เป็น data URLs และ Hermes Chat Completions ที่ตรวจ finish reason; ตรวจ extraction profile ว่าปิด tools ก่อนส่งข้อมูลลูกค้า
  - `src/db/events.rs` — แก้: เก็บ conversation_key ตั้งแต่รับ webhook เพื่อตรวจข้อความใหม่ที่ยังไม่ assign; ไม่ใช้ข้อความ NEEDS_ADMIN ที่ยังไม่ทราบ inquiry เป็นบริบท AI
  - `src/tests.rs`, `src/tests_step2.rs` — เพิ่ม/แก้: session/CSRF/JSON rejection, idempotency/version recovery, source isolation, invalid dates/evidence, tools refusal, context race, ambiguous messages และการซ่อนร่างที่อ้างอิงข้อความ unsend พร้อมสร้างร่างใหม่ต่อได้
- **โฟลเดอร์ `frontend/`**
  - `package.json`, `package-lock.json`, `tsconfig.json`, `next.config.ts`, `postcss.config.mjs`, `eslint.config.mjs`, `public/.gitkeep` — เพิ่ม/แก้: Next.js App Router + TypeScript + Tailwind, npm lockfile, standalone output และ relative API proxy ที่ไม่ส่ง server credentials เข้า client
  - `src/app/layout.tsx`, `src/app/page.tsx`, `src/app/globals.css` — เพิ่ม: Mitr Thai/Latin, หน้าล็อกอินและ workspace สีขาว/เทา เขียว/ส้ม/ฟ้า รองรับ desktop/mobile
  - `src/components/admin-workspace.tsx`, `src/components/draft-editor.tsx`, `src/lib/api.ts` — เพิ่ม: รายการ inquiry/ค้นหา/โหลดต่อ, แชท/รูปที่มีสิทธิ์อ่าน, สรุปแอดมิน/เลือกรูป, polling งานที่กลับมาเปิดต่อได้, แก้ร่าง/ยืนยันข้อมูล/หลักฐาน/ประวัติ และ normalize MongoDB Extended JSON
  - `AGENTS.md` — เพิ่มโดย Next.js dev: คำแนะนำให้อ่านเอกสารของ Next.js รุ่นที่ติดตั้งก่อนแก้ frontend
- **โฟลเดอร์ `./` (root) และ `docker/`**
  - `.env.example`, `AGENTS.md` — แก้: config ใหม่และวิธีทำงานของ Step 2 โดยไม่ใส่ secret จริง
  - `docker-stack.admin.yml` — เพิ่ม: frontend overlay บน LINE stack เดิม ใช้ line_net และ host port 18081 พร้อม pin node และ rollback
  - `docker-stack.line.yml`, `deploy-line.sh`, `docker-deploy.sh` — แก้: ส่ง config backend ใหม่และ deploy --full-stack ด้วย LINE base + admin overlay เพื่อรักษา Mongo volume/network/พอร์ต webhook เดิม ตรวจ admin config/tags/image checksums และรอ frontend health
  - `docker/frontend.Dockerfile`, `docker/Caddyfile.line.example` — แก้: runtime packaging และตัวอย่าง admin.salepie.online ไป frontend 18081 โดยเก็บ Hermes/webhook site เดิม
- **โฟลเดอร์ `scripts/`, `docs/` และ guideline**
  - `scripts/build-line.ps1` — แก้: รวม runtime admin overlay และคู่มือ Step 2 ใน local release archive สำรอง
  - `docs/step-2-admin.md` — เพิ่ม: แยกหน้าที่ผู้พัฒนา/ผู้ดูแลระบบ/แอดมิน, config, Hermes extraction profile, HTTPS/Caddy, local build → Git → runtime Docker deploy, API และวิธีทดสอบ/ข้อจำกัด
  - `docs/build-local.md`, `docs/deploy-line-server.md`, `docs/step-1-line-inquiry.md` — แก้: อ้างอิง frontend ที่มีแล้วและการส่งต่องาน Step 2
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: สถานะ implementation และสิ่งที่แต่ละบทบาทต้อง config/run; คง embedded images เดิมทั้ง 3 ภาพ
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — build จริงบน local: rel-20261024171734-2cb9ca0679a0, SHA256 2cb9ca0679a09d3a96ad4a00064fd8fefe783f691d6f81702c73690476956522
  - `frontend-linux/` — build จริงบน local: front-20261024170737-dee53199121e พร้อม Linux standalone dependencies/static/font และ checksums สำหรับ runtime Docker
- **ผลตรวจสอบ:** cargo fmt --check และ clippy --all-targets -- -D warnings ผ่าน; unit tests 7 และ MongoDB integration tests ทั้ง 3 ผ่าน; npm lint/build ผ่าน; build-backend.bat และ build-frontend.bat ผ่านจริง; Chrome desktop/mobile ทดสอบ signed webhook → แชท/ภาพ → login → Hermes จำลอง → ร่าง v1 → manual/confirm v2 → history/reload ผ่าน; real runtime Docker ผ่าน authenticated MongoDB startup, login/cookie, Next proxy/static assets; real Swarm config rendering และ mock deploy ตรวจการคง volume/network/ports และปฏิเสธ config/image ที่ผิดผ่าน โดยไม่เปลี่ยน Swarm จริง; runall.bat -Check, HTML role/search/mobile/print regression, frontend secret scan, Git inclusion, line endings และ git diff --check ผ่าน
- **ขอบเขตและสิ่งที่ต้องตั้งค่าจริง:** ยังไม่ได้ตั้งบัญชี/Hermes API/HTTPS หรือ deploy บน Ubuntu และไม่ได้ commit/push; ทดสอบ AI กับ Hermes จำลอง ไม่ส่งข้อมูลลูกค้าจริงให้โมเดล ผู้ดูแลต้องตั้ง .env และ extraction profile ก่อนใช้ AI จริง รองรับ 500 events/4 ภาพรวม 8 MB ต่อการเตรียมร่าง และยังใช้ API assign ของ Step 1 สำหรับข้อความกำกวมหลาย inquiry
- **ไฟล์ช่วยใน `data/` (ignored):** test-step2-ui.py, test-step2-runtime.py, update-step2-guide.py และ fixtures/screenshots ใช้ข้อมูลสมมติ; ลบ local Mongo test container ที่สร้างสำหรับงานนี้แล้ว

## 2026-10-24 23:44 — เพิ่มภาพ infographic Step 2 ในเอกสาร workflow
- **คำอธิบาย:** ผู้ใช้ส่งภาพสรุป Step 2 (จากแชทและสรุปแอดมิน → ร่างออเดอร์) ให้วางในหัวข้อ 5 จึงแปลง PNG 2.5 MB เป็น JPEG คุณภาพ 88 (≈573 KB, 887×1774) แล้วฝังแบบ base64 ตามรูปแบบภาพเดิมในไฟล์ เพื่อให้ HTML ยังเป็นไฟล์เดียวเปิดได้ทันที
- **โฟลเดอร์ `./` (root)**
  - `alex-line-hermes-mcp-workflow.html` — แก้: เพิ่ม `<figure id="step2-overview-diagram">` พร้อม alt และ figcaption ภาษาไทย ใต้ Input/เจ้าของงานหลักของหัวข้อ 5 ก่อนหัวข้อ 5.1

## 2026-10-24 20:05 — แก้ server pull ติดไฟล์ diagnostic ที่ยังไม่ tracked
- **คำอธิบาย:** loadgit.sh รุ่นเก่าบน Ubuntu หยุดเมื่อพบ diagnose-line.sh และ scripts/diagnose-line.py ที่ agent เคยวางไว้ก่อนมีไฟล์ใน Git จึงย้ายเฉพาะสองไฟล์ที่ตรวจแล้วว่า untracked ไปสำรองนอก repository โดยไม่ใช้ FORCE/reset แล้วรัน pull ตามปกติ
- **โฟลเดอร์ `/home/ubuntu/alex-connectingpoint/` บน Ubuntu**
  - `diagnose-line.sh`, `scripts/diagnose-line.py` — สำรอง: เก็บไฟล์เดิมใน /home/ubuntu/alex-before-pull-ec1re0km โดยรักษาโครงสร้าง relative path จากนั้นใช้ไฟล์ tracked ที่ pull จาก Git
  - ไฟล์ tracked และ compiled release — อัปเดต: fast-forward จาก 7174d78 เป็น d5a2d14 ด้วย bash ./loadgit.sh main; ได้ loadgit.sh รุ่นใหม่และ Linux compiled backend ที่ผู้ใช้ push ไว้แล้ว
- **ผลตรวจสอบ:** รัน loadgit.sh ซ้ำสำเร็จและรายงาน Already up to date; git status บน server สะอาด; sha256sum -c backend.sha256 ผ่าน; ตรวจ hash .env ก่อน/หลังตรงกันโดยไม่พิมพ์ค่า config หรือ secret
- **ขอบเขต:** ไม่มีการ compile/build โปรแกรมบน Ubuntu ไม่มีการ build image/deploy/change service หรือ commit/push โดย agent ผู้ใช้รัน sudo bash ./docker-build.sh และ sudo bash ./docker-deploy.sh ต่อเองตามสิทธิ์ Docker ของ server

## 2026-10-24 18:54 — เพิ่ม BAT สำหรับ compile backend และ build frontend บน local
- **คำอธิบาย:** แยกคำสั่ง build ที่เรียกง่ายบน Windows ทั้งสองส่วน โดย compile เป็น Linux/amd64 บน Docker Desktop ของเครื่อง local เท่านั้น Ubuntu pull compiled objects จาก Git แล้วสร้าง runtime Docker image โดยไม่ compile โปรแกรม
- **โฟลเดอร์ `./` (root)**
  - `build-backend.bat` — เพิ่ม: เรียก build-line.bat เดิมและส่งต่อ arguments เพื่อสร้าง Rust release ใน dist/line-linux
  - `build-frontend.bat` — เพิ่ม: เรียก PowerShell local builder สำหรับ Next.js standalone ใน dist/frontend-linux
  - `.gitignore`, `.gitattributes`, `.dockerignore` — แก้: รองรับ compiled frontend รวม traced dependencies/static ใน Git และ Docker context โดยรักษา bytes เดิมและกัน .env ออก
  - `docker-build.sh`, `docker-deploy.sh`, `docker-stack.yml` — แก้: ตรวจ checksum frontend และใช้ frontend tag แยกจาก backend เพื่อรองรับการอัปเดต frontend อย่างเดียว; ตรวจ image SHA256 labels ก่อน full-stack deploy
  - `AGENTS.md`, `.env.example` — แก้: ระบุคำสั่ง BAT ใหม่และการตั้ง BackendUrl ตอน local frontend build
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: เพิ่มคำสั่ง build แยกส่วนและลิงก์คู่มือ local โดยคงภาพประกอบและตัวกรองบทบาท
- **โฟลเดอร์ `scripts/`**
  - `build-frontend.ps1` — เพิ่ม: ตรวจ frontend source/lockfile, บังคับ local Linux builder, สร้าง checksum/tag/manifest, กัน .env, เก็บ release เก่าเป็น backup และแทนที่ output ทั้งโฟลเดอร์เมื่อ build สำเร็จ
  - `verify-frontend-release.sh` — เพิ่ม: ตรวจไฟล์ release, platform, tag และ SHA256 ก่อนสร้าง image/deploy
  - `build-line.ps1` — แก้: รวม frontend verifier และคู่มือ local ใน release archive สำรอง
- **โฟลเดอร์ `docker/`**
  - `frontend.build.Dockerfile`, `frontend.build.Dockerfile.dockerignore` — เพิ่ม: npm ci/lint/build ภายใน local Linux container และ export standalone พร้อม public/static โดยไม่ส่ง local dependencies หรือ .env เข้า compiler
- **โฟลเดอร์ `docs/`**
  - `build-local.md` — เพิ่ม: คำสั่ง BAT, prerequisites, output, BackendUrl และลำดับ Git/server packaging
  - `deploy-line-server.md` — แก้: ใช้ชื่อ build-backend.bat และเชื่อมคู่มือ frontend/local
- **โฟลเดอร์ `dist/line-linux/`**
  - `backend`, `backend.sha256`, `release.tag`, `release.json` — อัปเดต: build จริงบน local เป็น rel-20261024115310-65923ac32c6f; binary SHA256 65923ac32c6f1c8fab3ff2925cb2bfadc4b616a26f10ed9bdac9974028db275f
- **ผลตรวจสอบ:** build-backend.bat ผ่านจริง; cargo fmt --check, cargo clippy -- -D warnings และ cargo test ผ่าน (5 passed, 2 Mongo integration tests ignored); runall.bat -Check ผ่าน; ทดสอบ frontend ด้วย Next.js fixture ใน data/ ที่ถูก ignore ผ่านทั้ง lint/build, runtime Docker, HTTP หน้าเว็บ/public/static, checksum, การกัน .env, Git inclusion และการแทน release พร้อม backup; full-stack runtime image build/tag/label และ stack config ผ่าน โดยจำลองขั้น deploy จึงไม่เปลี่ยน Swarm จริง; PowerShell parse, BAT ASCII/CRLF, Linux files LF, deployment regression, HTML desktop/mobile/filter/search/print และ git diff --check ผ่าน
- **ข้อจำกัด:** repository ยังไม่มี frontend/package.json จึงยังไม่ได้ build frontend จริงของโปรเจกต์; BAT จะแจ้ง source ที่ขาดและไม่แก้ release เดิม สคริปต์พร้อมใช้งานเมื่อเพิ่ม frontend source แล้ว ไม่มีการ commit/push หรือแก้บริการบน Ubuntu ในงานนี้

## 2026-10-24 18:34 — แยก Database unavailable ออกจากผล rollback ของ backend
- **คำอธิบาย:** ตรวจประวัติ task ที่ผู้ใช้ส่ง พบ image release ที่ถูกต้องเคยรันและล้มด้วย Database unavailable ก่อนกลับไปเป็น image/constraint เก่า line.sh ซึ่งสอดคล้องกับ failure_action: rollback; MongoDB ปัจจุบันรันอยู่แต่ยังยืนยัน authentication/เส้นทาง Swarm จากสิทธิ์ SSH ของ agent ไม่ได้ จึงไม่สรุปว่าเป็น startup race หรือรหัสผ่านผิดโดยไม่มีผลตรวจ
- **โฟลเดอร์ `./` (root)**
  - `diagnose-line.sh` — เพิ่ม: โหลด config บน server และเรียก diagnostic แบบไม่แก้ services เก็บเฉพาะผลที่ตัดข้อมูลลับแล้วใน data/line-diagnostics-latest.txt พร้อมแสดงบน terminal
  - `.gitignore` — แก้: กัน Python __pycache__/bytecode ของ diagnostic ออกจาก Git
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: เพิ่มการแยกปัญหาฐานข้อมูลกับ rollback และย้ำว่าแก้ YAML อย่างเดียวยังไม่เปลี่ยน service จริง
- **โฟลเดอร์ `scripts/`**
  - `diagnose-line.py` — เพิ่ม: ตรวจ image/update state/constraint/network, เปรียบเทียบ URI/credentials กับ .env แบบ boolean และทดสอบ Mongo authentication ผ่าน localhost/service DNS/tasks DNS ภายใน Mongo container; ไม่พิมพ์ env, password/token, URI เต็ม, raw logs หรือข้อความ driver
  - `build-line.ps1` — แก้: รวมไฟล์ diagnostic ใน archive สำรองของ local build
- **โฟลเดอร์ `docs/`**
  - `deploy-line-server.md` — แก้: อธิบายผลตรวจ authentication/Swarm DNS/rollback และข้อจำกัดของ Mongo replicas 1/1 กับ startup ping timeout 5 วินาที
- **ผลตรวจสอบ:** ทดสอบ Mongo probe จริงใน Docker local ที่มี authentication ทั้ง localhost/service/tasks aliases ผ่าน; ทดสอบ redaction ด้วย mock service/rollback/config แล้วไม่มี password/token/URI หลุด; ทดสอบ Linux release เดิมกับ authenticated MongoDB ผ่านทั้ง health, signed webhook, user/chat, dedup, auth และ restart; Chrome desktop/mobile และ git diff --check ผ่าน ไม่มีการแก้หรือ compile Rust และไม่มีการเปลี่ยน services/volume บน Ubuntu
- **สถานะ server:** ติดตั้ง diagnose-line.sh และ scripts/diagnose-line.py ใน /home/ubuntu/alex-connectingpoint แล้ว ยืนยัน .env/compiled binary ไม่เปลี่ยน; ต้องให้ผู้ใช้รัน sudo bash ./diagnose-line.sh เพราะ SSH ของ agent ไม่มีสิทธิ์ Docker/sudo แบบไม่ใส่รหัส ณ เวลาบันทึกยังไม่มีรายงานจาก server จึงยังไม่ยืนยันสาเหตุ Database unavailable หรือว่า backend กลับมาทำงานแล้ว
- **ไฟล์ช่วยตรวจใน `data/` (ignored):** test-line-diagnostics.py, upload-line-diagnostics.py, verify-authenticated-runtime.py และ update-diagnostic-guide.py ใช้ทดสอบ/เตรียมงานนี้ด้วยค่าจำลองและไม่ส่ง secret ขึ้น log

## 2026-10-24 18:24 — ให้ server pull compiled object จาก Git แล้วสร้าง Docker
- **คำอธิบาย:** ปรับตามลำดับที่ผู้ใช้กำหนด: local compile และ push Linux release พร้อมโค้ด → Ubuntu pull จาก Git → สร้าง Docker image จาก compiled object → deploy image ที่สร้างแล้ว โดยไม่มีการ compile โปรแกรมบน Ubuntu
- **โฟลเดอร์ `./` (root)**
  - `.gitignore` — แก้: อนุญาตเฉพาะ backend, backend.sha256, release.tag และ release.json ใน dist/line-linux เข้า Git เพื่อให้ pull แล้ว build image ได้ทันที; Windows output/archives/.env/billing_wth_text ยังถูก ignore
  - `.gitattributes` — แก้: ระบุ backend เป็น binary ไม่แปลง line endings และบังคับ metadata Linux release เป็น LF
  - `docker-build.sh` — แก้: ค่าเริ่มต้นสร้างเฉพาะ LINE backend จาก compiled object โดยไม่ต้องมี frontend; ใส่ SHA256 label ให้ image และมี --full-stack สำหรับชุดที่มี frontend
  - `docker-deploy.sh` — แก้: ค่าเริ่มต้น deploy ชุด LINE ผ่าน --skip-build ไม่สร้าง image ซ้ำ; รองรับ --init-swarm และคงชุดเต็มผ่าน --full-stack
  - `deploy-line.sh` — แก้: เรียก docker-build.sh สำหรับ build image และเพิ่ม --skip-build ตรวจว่า image ที่มีอยู่ตรง SHA256 ของ release ก่อนใช้ deploy
  - `loadgit.sh` — แก้: เก็บ tracked server edits ใน stash ก่อน pull โดยไม่ทับไฟล์จาก Git ด้วย stash เก่า; ใช้ identity เฉพาะคำสั่งสำรอง ไม่เปลี่ยน global config; แสดงลำดับ build/deploy หลัง pull
  - `AGENTS.md`, `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: ระบุ Linux objects ที่เข้า Git และแยกหน้าที่ local build/push กับ server pull/build image/deploy ให้ตรงกัน
- **โฟลเดอร์ `scripts/`**
  - `build-line.ps1` — แก้: แสดงคำสั่ง commit/push แล้ว pull/build/deploy บน server; รวม docker-build.sh/docker-deploy.sh/loadgit.sh ใน archive สำรองด้วย
- **โฟลเดอร์ `docs/`**
  - `deploy-line-server.md` — แก้: คู่มือหลักเป็น Git workflow พร้อม config, first-time Swarm และการตรวจรับ; archive เป็นผลลัพธ์เสริมของ local build
- **โฟลเดอร์ `dist/line-linux/`**
  - `backend`, `backend.sha256`, `release.tag`, `release.json` — เตรียม: Linux/amd64 release รุ่น rel-20261024112304-65923ac32c6f จาก local build; binary SHA256 65923ac32c6f1c8fab3ff2925cb2bfadc4b616a26f10ed9bdac9974028db275f พร้อมให้ผู้ใช้ commit/push โดย rungit.bat ยังไม่มีการ stage/commit/push อัตโนมัติ
- **ผลตรวจสอบ:** real local build-line.bat และ docker-build.sh ผ่าน ตรวจ image SHA256 label ตรง binary; real Git clone/fetch/pull ในโฟลเดอร์ทดสอบบน Ubuntu ผ่าน พร้อมเก็บ server edits ใน stash และรักษา .env/ไฟล์ ignored; mock deploy ผ่านทั้งแยก build/deploy, init Swarm, skip-build และปฏิเสธ image hash ผิด; ยืนยันมีเพียง 4 release files ที่เข้า Git และ Git filters ไม่เปลี่ยน ELF; Bash syntax/LF, git diff --check และ Chrome desktop/mobile filters/search/deep links/print ผ่าน ไม่มีการแก้ Rust หรือ frontend source
- **สถานะ server:** เตรียมเฉพาะ loadgit.sh ที่รองรับ stash ใน /home/ubuntu/alex-connectingpoint พร้อม backup เพื่อให้ดึงรุ่นใหม่ได้หลังผู้ใช้ push; .env และ binary เดิมไม่เปลี่ยน ยังไม่ได้ pull/build Docker/deploy production ในงานนี้ เนื่องจาก Git รุ่นใหม่ยังรอผู้ใช้ push และผู้ใช้เลือกจะรัน sudo เอง
- **ไฟล์ช่วยตรวจใน `data/` (ignored):** verify-release-deploy.py, verify-git-docker-workflow.py, verify-server-git-pull.py, bootstrap-server-pull.py และ update-git-release-guide.py ใช้ตรวจ/เตรียมงานโดยไม่ส่ง secret ขึ้น Git หรือ log

## 2026-10-24 18:00 — แก้ Swarm แปลง node ID และ image tag ผิดจน service Pending
- **คำอธิบาย:** ทำซ้ำด้วย Docker CLI จริงทั้ง local และ Ubuntu พบ `${LINE_NODE_ID:?run deploy-line.sh}` และ `${TAG:?run deploy-line.sh}` ถูก legacy Swarm แปลงเป็น line.sh แม้กำหนดค่าแล้ว ส่งผลให้ทั้งสอง service ไม่มี node ตรง constraint และ backend อ้าง image ผิด
- **โฟลเดอร์ `./` (root)**
  - `docker-stack.line.yml` — แก้: ใช้การแทนตัวแปรโดยตรงสำหรับ node ID, tag และค่าจำเป็น โดย deploy script ตรวจค่าก่อนใช้งาน
  - `deploy-line.sh` — แก้: ตรวจ node ID, สถานะ ready/active และตรวจ config ที่ Docker แปลงแล้วว่า image ตรง release และ constraints ทั้งสองรายการตรง node ก่อนสร้าง runtime image/deploy; ไม่พิมพ์หรือบันทึก config ที่มี credentials
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: ใช้ path /home/ubuntu/alex-connectingpoint ให้ตรงโฟลเดอร์ Git บน server; เพิ่มแนวทางแก้ Pending/no suitable node ใน HTML
- **โฟลเดอร์ `docs/`**
  - `deploy-line-server.md` — แก้: ปรับ path และเพิ่มการตรวจ Pending, node state และกรณี image/constraint เป็น line.sh
- **ผลตรวจสอบ:** actual docker stack config บน local และ Ubuntu ยืนยัน node constraints, release image, LINE/admin และ Mongo credentials ตรงค่าจำลอง; mock deploy ผ่าน รวมกรณี node ID ผิด, drain และ rendered config ผิดต้องหยุดก่อน build/deploy; Bash syntax, git diff --check และ Docker runtime image build บน local ผ่าน; Chrome desktop/mobile role filters/search/deep links/print ผ่าน ไม่มีการแก้หรือ compile Rust
- **การแก้บน server:** สำรองและเปลี่ยน deploy-line.sh/docker-stack.line.yml ใน /home/ubuntu/alex-connectingpoint และโฟลเดอร์ release เดิมแล้ว ตรวจ .env และ compiled binary ไม่เปลี่ยน; SSH ของ agent ไม่มี sudo แบบไม่ถามรหัส จึงขอให้ผู้ใช้รัน sudo bash ./deploy-line.sh ตามที่เลือกไว้ ณ เวลาบันทึก health พอร์ต 18080 ยังไม่ผ่านและยังไม่ได้ยืนยัน deploy สำเร็จ
- **ไฟล์ช่วยตรวจใน `data/` (ignored):** apply-swarm-interpolation-fix.py, verify-release-deploy.py และ update-swarm-fix-guide.py ใช้ตรวจ regression/อัปโหลดเฉพาะไฟล์ที่แก้โดยไม่ส่งหรือพิมพ์ secret

## 2026-10-24 14:29 — เติม Linux release ในโฟลเดอร์ Git บน Ubuntu
- **คำอธิบาย:** แก้ Missing Linux release หลัง clone ไป /home/ubuntu/alex-connectingpoint เพราะ dist/ ไม่ถูกส่งผ่าน Git โดยคัดลอก release ที่ compile บน local และส่งไว้ใน /home/ubuntu/alex-order-connector แล้ว ไปยังโฟลเดอร์ที่ผู้ใช้ deploy จริง
- **โฟลเดอร์ `/home/ubuntu/alex-connectingpoint/dist/line-linux/` บน Ubuntu (ignored)**
  - `backend`, `backend.sha256`, `release.tag`, `release.json` — เพิ่ม: precompiled Linux/amd64 release รุ่น rel-20261023183243-65923ac32c6f โดยเตรียมในโฟลเดอร์ชั่วคราวแล้วเปลี่ยนชื่อเข้าปลายทาง ไม่ทับ release ที่มีอยู่
- **ผลตรวจสอบ:** source Rust/Cargo ของสองโฟลเดอร์ตรงกัน; verify-line-release.sh ผ่านทั้ง checksum/tag/ELF; config ที่จำเป็นครบ .env ไม่เปลี่ยนและสิทธิ์ยัง 600; git check-ignore ยืนยัน release และ .env ถูกกันออก; ไม่มีการ compile โปรแกรมหรือ deploy Docker บน Ubuntu ผู้ใช้ยังรัน Docker/deploy ด้วย sudo เอง

## 2026-10-24 11:29 — ตั้ง Git และ origin ให้ rungit ใช้งานได้
- **คำอธิบาย:** แก้ปัญหา rungit แจ้งว่าโฟลเดอร์ไม่ใช่ Git repository โดยสร้าง Git local สาขา main และตั้ง origin เป็น https://github.com/ceebell/alex-connectingpoint.git ตาม URL ที่ผู้ใช้ให้ ตรวจปลายทางแล้วเป็น repository ว่าง ไม่มีประวัติเดิมต้องรวม และยังไม่ได้ stage/commit/push
- **โฟลเดอร์ `./` (root)**
  - `rungit.bat` — แก้: เพิ่ม --setup สำหรับสร้าง Git/ตั้ง origin และ --check สำหรับตรวจแบบไม่ stage/commit; ตรวจ Git และ origin ก่อน git add/commit พร้อมแจ้งคำสั่งแก้ที่ชัดเจน; ไม่ทับ origin ที่มีอยู่; คง ASCII/CRLF
  - `.gitignore` — แก้: เพิ่ม billing_wth_text/ ตามที่ผู้ใช้สั่ง ไม่รวมโปรแกรมเก่าที่มี Git และ config ของตัวเอง
- **ผลตรวจสอบ:** ทดสอบ setup ด้วย path ที่มีช่องว่าง, --check, การคง remote เดิม และการหยุดก่อน stage/commit เมื่อขาด origin ผ่าน; root --check ผ่าน; git ls-remote เข้าถึง origin ได้และไม่พบ refs; git check-ignore ยืนยัน billing_wth_text/, .env, data/ และ dist/ ถูกกันออก และ index ยังว่าง ไม่มีการ compile/build หรือ deploy ในงานนี้
- **ไฟล์ช่วยตรวจใน `data/` (ignored):** fix-rungit.py และ repository ชั่วคราวใช้ตรวจพฤติกรรม batch โดยไม่มี commit หรือ push

## 2026-10-24 01:38 — Build Linux release บน local และสรุปคู่มือตามหน้าที่
- **คำอธิบาย:** แก้ตามคำสั่งผู้ใช้ให้ compile/build โปรแกรมบนเครื่อง local เท่านั้น Ubuntu รับ Linux release ที่เสร็จแล้วเพื่อนำเข้า Docker runtime image และ deploy; ยกเลิกแนวทาง compile บน server ของงานก่อนหน้า พร้อมย่อ HTML ให้เห็นงานผู้พัฒนา/ผู้ดูแลระบบและ config ที่ต้องลงมือจัดการ
- **โฟลเดอร์ `./` (root)**
  - `build-line.bat` — เพิ่ม: เรียก local Linux release builder ด้วย PowerShell; ASCII/CRLF
  - `build.bat` — แก้: ล้างเฉพาะ Windows output ของ backend/frontend เพื่อเก็บ Linux release และ archive ไว้
  - `deploy-line.sh` — แก้: ตรวจ release ก่อนเรียก Docker/เปลี่ยน Swarm ใช้ tag จาก local และสร้างเฉพาะ runtime image จาก binary ไม่มีการ compile หรือ fallback ไป build source
  - `docker-build.sh`, `docker-deploy.sh` — แก้: ใช้ release tag และ precompiled Linux objects; ชุดเต็มต้องมี frontend standalone ที่ build จาก local ไม่มี latest หรือคำสั่ง compile บน server
  - `.dockerignore` — แก้: อนุญาตเฉพาะ output Linux release ที่ต้อง COPY เข้า image และยังกัน secret/dependencies/cache
  - `.env.example`, `AGENTS.md` — แก้: บันทึกนโยบาย build local / runtime Docker บน Ubuntu และแยกตัวเลือก local build ออกจาก config server
  - `alex-line-hermes-mcp-workflow.html` — แก้: สรุป 6 หัวข้อ เลือกดูตามหน้าที่ มีตาราง config/แหล่งที่มาและขั้นตอน local build → ส่ง release → deploy; พับรายละเอียดเดิม 24 หัวข้อไว้ คงภาพทั้งสองภาพและขั้นตอน LINE/legacy API; รองรับค้นหา ลิงก์ตรง มือถือ และพิมพ์เฉพาะสรุป
  - `alex-line-hermes-mcp-workflow.md` — แก้: v1.4 และหัวข้อ 4.2 ระบุหน้าที่ผู้พัฒนา build บน local และผู้ดูแลระบบ deploy precompiled release บน Ubuntu
- **โฟลเดอร์ `scripts/`**
  - `build-line.ps1` — เพิ่ม: ตรวจ Docker Desktop named pipe ของ local และ builder เฉพาะเครื่อง สร้าง Linux/amd64 ELF พร้อม SHA256/tag/manifest แล้วรวม archive แบบ allowlist ไม่มี source/compiler Dockerfile/.env จริง; ทดสอบทั้ง build ครั้งแรกและใช้ cache ครั้งถัดไป
  - `verify-line-release.sh` — เพิ่ม: ตรวจไฟล์ release, checksum, tag และ ELF x86_64; ปฏิเสธไฟล์ Windows/รุ่นผิด/ข้อมูลเสียก่อนแตะ Swarm
- **โฟลเดอร์ `docker/`**
  - `backend.build.Dockerfile` — เพิ่ม: compiler สำหรับ Docker Desktop บน local เท่านั้น ใช้ locked Cargo dependencies และ export binary
  - `backend.Dockerfile` — แก้: runtime Debian Bookworm COPY binary ที่ compile แล้ว ตั้งสิทธิ์ executable และรันด้วย user 10001 ไม่มี Rust/Cargo
  - `frontend.Dockerfile` — แก้: runtime COPY prebuilt Linux standalone ไม่มี npm install/build; ชุด Step 1 ไม่ใช้ frontend
- **โฟลเดอร์ `docs/`**
  - `deploy-line-server.md` — แก้: ลำดับส่งมอบ release ระหว่างผู้พัฒนากับผู้ดูแลระบบ คำสั่ง local build/upload/server config/deploy/Caddy/ตรวจรับ
  - `step-1-line-inquiry.md` — แก้: ระบุว่า compile Linux บน local ก่อนส่งไปใช้งานจริงบน Ubuntu
- **ผลลัพธ์ build และ server:** สร้าง binary Linux/amd64 บน local สำเร็จ SHA256 `65923ac32c6f1c8fab3ff2925cb2bfadc4b616a26f10ed9bdac9974028db275f`; ส่ง archive `line-rel-20261023183243-65923ac32c6f.tar.gz` ที่มีเฉพาะ release/runtime/เอกสาร 15 ไฟล์ขึ้น server และแตกลง /home/ubuntu/alex-order-connector แล้ว ตรวจ checksum/ELF ผ่านและยืนยัน .env เดิมไม่เปลี่ยน สิทธิ์ยัง 600; ไม่มีการ compile โปรแกรมหรือ deploy Docker บน Ubuntu ผู้ใช้เลือกจะรัน sudo เอง
- **ผลตรวจสอบ:** cargo fmt --check, clippy -D warnings และ cargo test ผ่าน (5 passed, 2 Mongo tests ignored); สร้าง runtime Docker image และทดสอบ HTTP กับ MongoDB ใน Docker บน local ผ่านสำหรับ health, signed webhook, user/chat, dedup, admin auth และ restart/persistence; runtime ไม่มี compiler และไม่รัน root; Bash syntax และ mock deploy ผ่านทั้ง success/init และกรณี release/config/checksum/platform/tag/package/health ล้มเหลว; Chrome desktop/mobile ผ่าน role filter/search/deep links/ภาพ/print และไม่มีหน้าเลื่อนล้น; runall.bat -Check ผ่าน ไม่มี frontend/package.json จึงไม่ได้รัน npm lint/build หรือ build frontend image
- **ไฟล์ช่วยตรวจใน `data/` (ignored):** draft สรุป/CSS/JavaScript, สคริปต์ปรับเอกสาร/เตรียม release, mock deploy, HTTP runtime, browser checks, ภาพ preview และ uploader แบบ allowlist ใช้ตรวจงานนี้เท่านั้น ไม่รวมใน release หรือส่ง secret ขึ้น log

## 2026-10-23 23:44 — เตรียม deploy LINE backend บน Ubuntu ด้วย Caddy
- **คำอธิบาย:** เปลี่ยนแนวทางใช้งานจริงเป็น backend และ MongoDB บน server เดียวกับ Caddy โดยเตรียมชุด deploy ที่ไม่ต้องมี frontend ตามที่ผู้ใช้เลือกให้รัน Docker ด้วย sudo เอง
- **โฟลเดอร์ `./` (root)**
  - `deploy-line.sh` — เพิ่ม: ตรวจ config/Buildx/Swarm, build backend ด้วย tag จาก git SHA หรือ hash ของ source, deploy เฉพาะ backend และ MongoDB, ตรึง node ของ Caddy และรอ health; ไม่แสดง config ที่มี credentials หรือ Swarm join token
  - `docker-stack.line.yml` — เพิ่ม: Swarm services backend/mongo พร้อม volume ถาวร, host port 18080, memory/log limits และ update แบบ stop-first
  - `.env.example` — แก้: เพิ่มตัวเลือก LINE_HTTP_PORT, SWARM_ADVERTISE_ADDR และ BUILD_JOBS สำหรับ server
  - `.dockerignore` — แก้: กัน billing_wth_text ออกจาก build context
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: ปรับเป็น v1.3 เพิ่มหัวข้อ 4.2 และสารบัญสำหรับ deploy บน Ubuntu จริง โดยเก็บทุก section ภาพ CSS และ JavaScript เดิมไว้
- **โฟลเดอร์ `docker/`**
  - `backend.Dockerfile` — แก้: เพิ่ม CARGO_BUILD_JOBS เริ่มต้น 1 เพื่อลดการใช้ทรัพยากรระหว่าง compile บน server
  - `Caddyfile.line.example` — เพิ่ม: webhook ไป backend พอร์ต 18080 และเส้นทางอื่นไป Hermes พอร์ต 8646
- **โฟลเดอร์ `docs/`**
  - `deploy-line-server.md` — เพิ่ม: ขั้นตอน Docker/Buildx, config, deploy, Caddy, health และทดสอบ LINE บน hermes.salepie.online
  - `step-1-line-inquiry.md` — แก้: เชื่อมคู่มือ server แยกจาก runall สำหรับ Windows
- **การเตรียม server:** อัปโหลด source/config ที่จำเป็น 30 ไฟล์ไป /home/ubuntu/alex-order-connector โดยไม่ส่งโปรแกรมเก่า ข้อมูล MongoDB ในคอม หรือ build cache; เตรียม .env สิทธิ์ 600 ส่งเฉพาะค่า LINE/admin ผ่าน SSH แบบไม่แสดงค่า และสร้าง Mongo password สุ่มบน server; ยืนยัน Caddy โหลด route webhook แล้ว
- **ผลตรวจสอบ:** Docker CLI ตรวจ schema ของ stack ผ่านด้วยค่าจำลอง; bash -n บน Ubuntu ผ่าน; mock tests ผ่านสำหรับ deploy สำเร็จ, init Swarm, missing config, ห้าม latest, build ล้มเหลว, health ล้มเหลว และ source เปลี่ยนแล้ว image tag เปลี่ยน; Caddy adapt ผ่าน; ตรวจ encoding/line endings และ HTML anchors/fingerprints เดิมครบ
- **สถานะ:** ยังไม่ได้ build image หรือ deploy service จริง เพราะบัญชี ubuntu ไม่มีสิทธิ์ Docker socket และ sudo ต้องถามรหัส ผู้ใช้เลือกจะรัน sudo เอง; คำสั่งถัดไปคือ sudo apt-get install -y docker-buildx แล้ว sudo bash ./deploy-line.sh --init-swarm บน server ไม่มีการเปลี่ยน Caddy หรือ Hermes ในงานนี้

## 2026-10-23 14:31 — เติม admin token เพื่อให้ runall ผ่าน config
- **คำอธิบาย:** ผู้ใช้เปิด runall ไม่ได้เพราะ INQUIRY_ADMIN_TOKEN ยังว่าง จึงสร้าง token สุ่ม 256 บิตสำหรับ API แอดมินลง config ในเครื่อง โดยไม่แสดงหรือบันทึกค่า secret
- **โฟลเดอร์ `./` (root)**
  - `.env` (ไฟล์ local ที่อยู่ใน .gitignore) — แก้: เติมเฉพาะ INQUIRY_ADMIN_TOKEN ที่ว่าง หลังตรวจว่ามีบรรทัดเดียว; คงค่าอื่นและ encoding/line endings เดิมไว้
  - `revising-log.md` — แก้: บันทึกการตั้งค่าโดยไม่มีค่า token
- **ผลตรวจสอบ:** ตรวจ token เป็น hexadecimal 64 ตัวและข้อมูลส่วนอื่นไม่เปลี่ยน; cmd.exe /c runall.bat -Check ผ่าน โดยไม่เปิดบริการหรือเรียก LINE/Alex API
- **ขอบเขต:** ไม่แก้โค้ด backend/frontend หรือสคริปต์ และไม่เปลี่ยน config บน Ubuntu; ผู้ใช้เปิด runall.bat อีกครั้งเพื่อเริ่มบริการได้

## 2026-10-23 13:38 — เก็บ LINE user ID และประวัติแชทรายคนใน MongoDB
- **คำอธิบาย:** เพิ่มทะเบียนผู้ใช้จาก webhook ที่ตรวจลายเซ็นแล้ว และ API เรียกดูข้อความของผู้ใช้แต่ละคน ตามที่ผู้ใช้ยืนยันให้เก็บใน MongoDB ของระบบเรา โดยใช้ line_events เดิมเป็นประวัติข้อความ
- **โฟลเดอร์ `backend/src/db/`**
  - `users.rs` — เพิ่ม: upsert line_users ตาม shop/channel/user ID พร้อม first/last seen แบบ min/max รองรับ duplicate และเหตุการณ์มาถึงสลับลำดับ; query รายชื่อและประวัติแยกผู้ใช้พร้อม cursor
  - `events.rs` — แก้: บันทึกทะเบียนผู้ใช้ก่อน ACK และเติมทะเบียนที่เขียนค้างจาก redelivery; รวมการซ่อน payload/media ของข้อความที่ unsend สำหรับ API เหตุการณ์และประวัติ
  - `indexes.rs`, `mod.rs` — แก้: เพิ่ม unique index ทะเบียนผู้ใช้และ index สำหรับ pagination/ประวัติ พร้อมลงทะเบียนโมดูล
- **โฟลเดอร์ `backend/src/routes/`**
  - `users.rs` — เพิ่ม: GET /api/line/users และ GET /api/line/users/{user_id}/messages พร้อม next cursor สูงสุด 100 ผู้ใช้/200 ข้อความต่อหน้า
  - `mod.rs`, `admin.rs` — แก้: เพิ่มเส้นทางภายใต้ admin bearer middleware และใช้การซ่อน unsend ร่วมกัน; คง Cache-Control: no-store
- **โฟลเดอร์ `backend/src/`**
  - `tests.rs` — แก้: ตรวจสิทธิ์ API ใหม่ และเพิ่ม MongoDB integration test สำหรับ follow/ข้อความ, การเขียนสำเร็จบางส่วนก่อน ACK, redelivery, first/last seen, ผู้ส่งในกลุ่ม, shop/channel isolation, pagination และ unsend ก่อน/หลัง purge
- **โฟลเดอร์ `./` (root)**
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: ปรับเป็น v1.2 เพิ่มหัวข้อ 4.1 พร้อมโครงสร้าง MongoDB, API และตัวอย่าง PowerShell รับ token แบบซ่อน; เพิ่มสารบัญ HTML โดยเก็บภาพ/CSS/JS/ขั้นตอนตั้งค่า LINE และคู่มือ legacy API เดิมไว้
- **โฟลเดอร์ `docs/`**
  - `step-1-line-inquiry.md` — แก้: เพิ่มทะเบียนผู้ใช้ วิธีอ่านประวัติ ขอบเขตข้อมูลเก่า/ข้อความจาก OA Manager และรายละเอียดการตรวจสอบ
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo build และ cargo test -- --include-ignored ผ่านครบ 7 tests กับ MongoDB ทดสอบจริง; HTTP จริงตรวจ startup/indexes, บันทึก user/chat, restart, dedup, 401 และ no-store ผ่าน; HTML ตรวจ desktop/mobile, search/สารบัญ/table wrappers/copy button และ fingerprint ภาพ/CSS/JS/คู่มือเดิมไม่เปลี่ยน; ตัวอย่าง PowerShell ผ่าน parser และ mock; runall preflight ผ่านเมื่อใช้ config จำลองครบ และปิด process ทดสอบทั้งหมดแล้ว
- **ขอบเขต:** ไม่เรียก LINE/Alex API จริง ไม่แก้ .env ของผู้ใช้ และยังไม่มี inbox UI; ทะเบียนเริ่มจาก webhook ใหม่ ส่วนข้อความเก่าที่มี source.userId ยังเรียกดูได้; ตรวจ config จริงของ runall พบว่ายังขาด INQUIRY_ADMIN_TOKEN ซึ่งต้องตั้งก่อนเปิดบริการ

## 2026-10-23 10:08 — ปรับ guideline ตามวิธีเรียก API ใน billing_wth_text
- **คำอธิบาย:** ตรวจ client เดิมพบ POST login, GetProductItems, checkq และ AddRentalOrder จึงเพิ่ม request mapping ที่ยืนยันจากโค้ด แยกจาก data contract ของ Connector พร้อมระบุสิ่งที่ยังต้องยืนยันกับ server
- **โฟลเดอร์ `./` (root)**
  - `alex-line-hermes-mcp-workflow.md` — แก้: ปรับเอกสารเป็น v1.1 เพิ่มหัวข้อ 10.5 อธิบาย bearer authentication, productItemId, body เช็คคิวแบบ array, orderDetail/orderItems สำหรับ submit, mapping วันที่/จำนวน/สาขา/ส่วนลด, หลักฐานไฟล์และบรรทัดในโค้ด, ข้อจำกัด response/Order ID/atomicity/idempotency และลำดับทดสอบ; ปรับ Step 3–5, capability matrix, pseudocode, แผนพัฒนาและคำถามถึง Alex ให้สอดคล้องกับหลักฐานที่พบ
  - `alex-line-hermes-mcp-workflow.html` — แก้: อัปเดตเนื้อหาที่เกี่ยวข้องให้ตรง Markdown เพิ่มหัวข้อและสารบัญ 10.5 ตัวอย่าง JSON/HTTP ที่คัดลอกได้ และ metadata v1.1 โดยเก็บภาพ คู่มือ LINE, CSS และ JavaScript เดิมไว้
- **ผลตรวจสอบ:** เทียบ endpoint กับ AST ของ config/client; ทดสอบ product search และ checkq ด้วย mock HTTP; เทียบชื่อ field/ชนิดข้อมูล/วันที่/ส่วนลดของตัวอย่างกับ create_order_json ที่รันแยกจากแอป; ทดสอบ JavaScript addOrder ด้วย mock fetch ยืนยัน POST headers/body และพบว่า success เดิมไม่อ่าน response JSON; ตรวจตัวอย่าง JSON ทั้ง 5 ชุดของ API ใหม่ตรงกันใน Markdown/HTML, links/IDs, credentials ไม่หลุด, ไฟล์โปรแกรมเก่าไม่เปลี่ยน และภาพ/CSS/JS/คู่มือ LINE เดิมไม่เปลี่ยน; เปิด Chrome ตรวจ desktop/mobile, search, สารบัญ, copy buttons และ table wrappers ผ่าน
- **ขอบเขต:** อ่านโค้ดใน billing_wth_text โดยไม่แก้โปรแกรมเดิม ไม่เพิ่ม Alex adapter/MCP ใน backend และไม่เรียก API production หรือ submit ออเดอร์จริง; response ของ Order ID, atomic booking และ idempotency ยังต้องยืนยันกับ Alex

## 2026-10-22 15:50 — เพิ่มภาพหน้าจอ LINE OA ใน guideline
- **คำอธิบาย:** ใส่ภาพที่ผู้ใช้วงชื่อ OA, Settings และ Account settings ไว้ เพื่อช่วยทำตามขั้นตอนตั้งค่า Messaging API
- **โฟลเดอร์ `./` (root)**
  - `alex-line-hermes-mcp-workflow.html` — แก้: ฝังภาพต้นฉบับ 1886×876 เป็น data URI ในขั้นที่ 2 ของคู่มือตั้งค่า Step 1 ทำให้เปิด HTML ไฟล์เดียวแล้วภาพไม่หาย พร้อม alt text และคำบรรยายว่าขั้นถัดไปให้คลิก Messaging API ทางซ้ายใต้ Response settings; ปรับภาพย่อขยายตามความกว้างหน้าจอโดยคงสัดส่วนเดิม
- **ผลตรวจสอบ:** ตรวจ bytes ของภาพที่ฝังตรงกับไฟล์แนบต้นฉบับ; ตรวจ caption/anchor IDs และยืนยันภาพเดิม CSS และ JavaScript ไม่เปลี่ยน; เปิดตรวจด้วย Chrome ทั้ง desktop/mobile ภาพโหลดครบและไม่ล้นกรอบ

## 2026-10-22 14:09 — เพิ่มคู่มือหาค่า LINE และตั้ง .env ใน HTML
- **คำอธิบาย:** เพิ่มขั้นตอนแก้ Missing configuration สำหรับผู้เริ่มใช้งาน runall.bat โดยระบุแหล่งที่มาของค่าทั้ง 6 ตัวและขั้นตอนตั้งค่าจนเชื่อม webhook ได้
- **โฟลเดอร์ `./` (root)**
  - `alex-line-hermes-mcp-workflow.html` — แก้: เพิ่มหัวข้อเริ่มใช้งาน Step 1 พร้อมลิงก์จากบทนำและสารบัญ ตารางที่มาของ LINE_SHOP_ID, LINE_CHANNEL_ID, LINE_CHANNEL_SECRET, LINE_CHANNEL_ACCESS_TOKEN, LINE_DESTINATION_ID และ INQUIRY_ADMIN_TOKEN; เพิ่มขั้นตอนเปิด Messaging API, หา bot user ID ผ่าน bot-info API, สร้าง admin token ลง .env โดยไม่แสดงค่า, ตรวจ config, เปิดระบบ, ตั้ง HTTPS webhook และแก้ปัญหาที่พบบ่อย พร้อมลิงก์เอกสาร LINE ทางการ
- **ผลตรวจสอบ:** ตรวจ HTML IDs/anchor links ครบและไม่ซ้ำ; ตรวจ fingerprint ยืนยันภาพ CSS และ JavaScript เดิมไม่เปลี่ยน; ตรวจ PowerShell examples ผ่าน parser และทดสอบ bot-info ด้วย mock; ทดสอบสร้าง admin token ในไฟล์จำลองได้ 64 hex characters โดยไม่แสดงค่า ไม่เปลี่ยนค่าอื่น และปฏิเสธการเขียนทับ/บรรทัดซ้ำ; เปิดตรวจจริงด้วย Chrome ทั้ง desktop/mobile และตรวจปุ่มคัดลอกกับ table wrappers ผ่าน
- **ขอบเขต:** เพิ่มเฉพาะคู่มือ HTML ไม่แก้ backend/launcher ไม่สร้างหรืออ่าน credentials จริงของผู้ใช้ และไม่ได้เรียก LINE API จริงระหว่างทดสอบตัวอย่าง

## 2026-10-21 23:51 — ปรับ runall ให้รัน Step 1 ได้โดยยังไม่มี frontend
- **คำอธิบาย:** แก้ dev launcher ที่เดิมบังคับให้มี frontend ให้เริ่ม MongoDB/backend ตามโครงสร้างปัจจุบัน ตรวจ config ก่อนเปิดบริการ และรองรับ frontend เมื่อเพิ่มภายหลัง
- **โฟลเดอร์ `./` (root)**
  - `runall.bat` — แก้: เรียก PowerShell launcher ด้วย process-scoped ExecutionPolicy Bypass ส่งต่อ arguments/exit code และรักษา ASCII/CRLF
- **โฟลเดอร์ `scripts/`**
  - `runall.ps1` — เพิ่ม: ตรวจค่าจำเป็นจาก environment/.env โดยไม่แสดง secret, ข้าม frontend และไม่บังคับ Node/npm เมื่อยังไม่มี package.json, รองรับ -Check, อ่านพอร์ตจาก config, รอ MongoDB พร้อม, ไม่เปิด MongoDB ใหม่สำหรับ external/authenticated URI, ตรวจ backend health ก่อนใช้ process เดิม, ใช้ npm.cmd และส่ง BACKEND_URL แบบ IPv4 ให้ frontend
- **โฟลเดอร์ `docs/`**
  - `step-1-line-inquiry.md` — แก้: เปลี่ยนวิธีเริ่ม dev เป็น runall.bat อธิบาย preflight/optional frontend และใช้ 127.0.0.1 สำหรับ API ตัวอย่าง
- **ผลตรวจสอบ:** PowerShell syntax และ ASCII/CRLF ผ่าน; ทดสอบ batch จากอีก working directory ที่มีช่องว่าง, config ที่ขาด/ว่าง/เป็น comment, environment precedence, URI/port ผิด, พอร์ต backend ถูกใช้งาน, ไม่แสดง secret และ npm install ล้มเหลวก่อนเปิดบริการ; ทดสอบ launcher เปิด MongoDB/backend จริงพร้อม HTTP health และ frontend fixture ผ่าน npm install/dev, proxy URL ตามพอร์ตจริง และรันซ้ำโดยไม่สร้าง process เพิ่ม; ปิด process ทดสอบทั้งหมดแล้ว
- **การใช้งาน:** กรอกค่าที่จำเป็นใน .env หรือ environment แล้วรัน runall.bat; ตรวจความพร้อมอย่างเดียวด้วย runall.bat -Check ปัจจุบัน repo ยังไม่มี .env จึงตรวจแล้วแจ้งชื่อค่าที่ขาดและ exit 1 ก่อนเปิดบริการ

## 2026-10-21 23:01 — สร้าง Step 1 รับ inquiry จาก LINE OA
- **คำอธิบาย:** เพิ่ม backend สำหรับรับ LINE webhook แบบ durable ตรวจลายเซ็นจาก raw body กันเหตุการณ์ซ้ำ ผูกร้าน/ลูกค้า/บทสนทนา/inquiry เก็บรูปแบบ private และเตรียมคิวให้ Step 2 โดยยังไม่เรียก Hermes หรือสร้างออเดอร์
- **โฟลเดอร์ `backend/`**
  - `Cargo.toml`, `Cargo.lock` — เพิ่ม: Rust package ชื่อ backend พร้อม dependencies สำหรับ axum, MongoDB, HMAC และดาวน์โหลด media
- **โฟลเดอร์ `backend/src/`**
  - `main.rs` — เพิ่ม: startup, structured logging และ graceful shutdown บน Windows/Linux
  - `config.rs`, `state.rs` — เพิ่ม: env config, trusted shop/channel mapping, MongoDB majority+journal และ HTTP client ที่ไม่ตาม redirect
  - `error.rs` — เพิ่ม: AppError และ JSON error response รูปแบบเดียวกันโดยไม่ส่งรายละเอียด credentials
  - `worker.rs` — เพิ่ม: durable queue worker, atomic lease/heartbeat, retry, inquiry linking, media processing และ cleanup
  - `tests.rs` — เพิ่ม: unit/HTTP tests และ integration test กับ MongoDB จริง ครอบคลุม durability, concurrent duplicate/worker, ลำดับข้อความ, burst, แยกร้าน, admin assign, postback, GridFS, unsend และ retry
- **โฟลเดอร์ `backend/src/models/`**
  - `mod.rs`, `line.rs` — เพิ่ม: webhook/event/source models, validation และ conversation identity ที่แยกผู้ส่งใน group/room
- **โฟลเดอร์ `backend/src/db/`**
  - `mod.rs`, `indexes.rs` — เพิ่ม: database modules และ indexes ที่สร้างซ้ำได้ตอน startup
  - `events.rs` — เพิ่ม: durable event intake, deduplication, claim, lease renewal, completion และ bounded retry
  - `inquiries.rs` — เพิ่ม: conversation/inquiry mapping, analysis job handoff และการจัดกลุ่มข้อความตาม event time
  - `media.rs` — เพิ่ม: ดาวน์โหลดรูปจาก LINE Content API, private GridFS, size/retention policy, unsend tombstone และ cleanup
- **โฟลเดอร์ `backend/src/routes/`**
  - `mod.rs` — เพิ่ม: API router, health endpoint, CORS, request size limit และ JSON fallback
  - `webhook.rs` — เพิ่ม: POST /api/line/webhook ตรวจ HMAC-SHA256 และ destination ก่อนบันทึก ตอบรับหลังเขียนฐานข้อมูลสำเร็จ
  - `admin.rs` — เพิ่ม: API ภายในที่ใช้ bearer token สำหรับ inquiry/context, event monitoring, เลือก inquiry, retry และอ่าน media ตามสิทธิ์
- **โฟลเดอร์ `./` (root)**
  - `.env.example` — แก้: เพิ่มตัวแปร LINE, admin API token, burst window และ media policy โดยไม่ใส่ credentials จริง
  - `docker-stack.yml` — แก้: ส่ง env ของ LINE inquiry intake ให้ backend service และรักษา line ending แบบ LF
- **โฟลเดอร์ `docs/`**
  - `step-1-line-inquiry.md` — เพิ่ม: คู่มือตั้งค่า LINE OA/HTTPS, รัน backend, API, data collections, retention, การทดสอบ และขอบเขตส่งต่อ Step 2
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy -- -D warnings, cargo clippy --all-targets -- -D warnings, cargo build และ tests ทั้ง 6 รายการผ่าน (รวม MongoDB integration); HTTP smoke ผ่านทั้ง health, signed webhook, duplicate, background worker, context และ restart; docker stack config ผ่านด้วย env จำลอง
- **ข้อจำกัด:** ยังไม่ได้เชื่อม LINE OA จริงเนื่องจากต้องตั้ง credentials/HTTPS; docker build ตรวจไม่ได้เพราะ Docker Engine ไม่ได้รัน; งานนี้ไม่มี frontend จึงยังใช้ runall.bat ครบสามส่วนไม่ได้ ให้รัน cargo run ใน backend ตามคู่มือ

## 2026-10-21 20:57 — สร้างชุดไฟล์เริ่มต้นของโปรเจกต์
- **คำอธิบาย:** วางโครงเอกสาร, สคริปต์ dev/build/git, และ Docker Swarm config สำหรับ Rust axum + Next.js + MongoDB (ยังไม่มีโค้ดแอป — ใช้ `prompts/00-bootstrap.md` สร้าง skeleton)
- **โฟลเดอร์ `./` (root)**
  - `AGENTS.md` — สร้างใหม่: กฎโปรเจกต์ stack, design system, คำสั่ง, กฎ agent, รูปแบบ log
  - `CLAUDE.md` — สร้างใหม่: import `AGENTS.md` + ข้อกำหนดเฉพาะ Claude Code
  - `revising-log.md` — สร้างใหม่ (ไฟล์นี้)
  - `runall.bat` — สร้างใหม่: เปิด MongoDB + backend + frontend ด้วยคำสั่งเดียว
  - `build.bat` — สร้างใหม่: build Rust release + Next.js standalone แล้วเก็บเฉพาะผลลัพธ์ใน `dist/`
  - `rungit.bat` — สร้างใหม่: add + commit + push
  - `loadgit.sh` — สร้างใหม่: pull โค้ดลง Ubuntu server
  - `docker-build.sh` — สร้างใหม่: build image backend/frontend (tag = git SHA)
  - `docker-deploy.sh` — สร้างใหม่: deploy stack เข้า Swarm
  - `docker-stack.yml` — สร้างใหม่: services mongo, backend, frontend
  - `.env.example`, `.gitignore`, `.gitattributes`, `.dockerignore` — สร้างใหม่
- **โฟลเดอร์ `docker/`**
  - `backend.Dockerfile` — สร้างใหม่: multi-stage Rust → debian-slim
  - `frontend.Dockerfile` — สร้างใหม่: multi-stage Next.js standalone → node alpine
- **โฟลเดอร์ `prompts/`**
  - `00-bootstrap.md`, `01-backend-feature.md`, `02-frontend-page.md`, `03-deploy-check.md` — สร้างใหม่: prompt สำเร็จรูป

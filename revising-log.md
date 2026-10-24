# revising-log

บันทึกการแก้ไขโค้ดทุกครั้ง — entry ใหม่อยู่บนสุด · รูปแบบและกฎดู `AGENTS.md` ข้อ 10

<!-- entries -->

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

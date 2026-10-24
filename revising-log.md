# revising-log

บันทึกการแก้ไขโค้ดทุกครั้ง — entry ใหม่อยู่บนสุด · รูปแบบและกฎดู `AGENTS.md` ข้อ 10

<!-- entries -->

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

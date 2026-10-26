# Step 2 — หน้าหลังบ้านและร่างออเดอร์

ทำแล้วทั้ง Next.js frontend และ Rust backend: แอดมินเข้าสู่ระบบ → เลือก inquiry → อ่านแชท/รูป → บันทึกสรุป → กด **เตรียมออเดอร์ด้วย AI** → ตรวจ/แก้ร่าง → บันทึกเป็นเวอร์ชันใหม่ การเตรียมร่างไม่ส่งข้อความ เช็คคิว หรือสร้างออเดอร์กับ Alex ปุ่มตอบลูกค้าและอนุมัติออเดอร์ในฐานข้อมูลเป็นการทำงานแยกที่แอดมินต้องกดเอง

การใช้งานปัจจุบันให้แอดมินตอบลูกค้าจาก Alex ข้อความขาออกจะถูกรวมในบริบทสกัดร่าง ตัวเชื่อม OA Manager เป็นทางเลือกและปิดเป็นค่าเริ่มต้น (`OA_CONNECTOR_ENABLED=false`) ดู [คู่มือตัวเชื่อม OA](line-oa-auto-sync.md) หากต้องการเปิดเพิ่ม

## ใครต้องทำอะไร

| ผู้รับผิดชอบ | ต้องลงมือทำ |
| --- | --- |
| ผู้พัฒนา | npm install ครั้งแรก; build-backend.bat และ build-frontend.bat บน local; ใช้ rungit.bat เพื่อส่ง source + compiled releases เมื่อพร้อม |
| ผู้ดูแลระบบ | ตั้งบัญชีแอดมิน, Hermes API และ HTTPS; pull Git; สร้าง runtime images และ deploy --full-stack บน Ubuntu |
| แอดมินร้าน | เปิดหน้าหลังบ้าน ล็อกอิน เลือก inquiry ใส่สรุป/เลือกรูป ตรวจร่างและยืนยันข้อมูลที่กรอก |

## ทดสอบบน Windows local

เพิ่มค่าต่อไปนี้ใน `.env` เดิมที่ root โดยใช้รหัสจริงของคุณและไม่ commit:

```dotenv
ADMIN_USERNAME=admin
ADMIN_PASSWORD=REPLACE_WITH_RANDOM_PASSWORD_AT_LEAST_16_CHARACTERS
ADMIN_COOKIE_SECURE=false
HERMES_API_URL=http://127.0.0.1:8642/v1/chat/completions
HERMES_API_KEY=REPLACE_WITH_HERMES_PROFILE_API_KEY
HERMES_MODEL=hermes-agent
HERMES_TIMEOUT_SECONDS=120
```

รัน `runall.bat` แล้วเปิด `http://localhost:3000` ใช้ ADMIN_USERNAME/ADMIN_PASSWORD ล็อกอิน หากเพิ่งเปลี่ยน config ให้หยุดและเริ่ม backend ใหม่ สคริปต์จะใช้ backend ที่ health ผ่านอยู่แล้ว จึงต้อง restart process เดิมเพื่อโหลดค่าที่แก้

หากยังไม่ตั้ง Hermes ปุ่ม AI จะปิด แต่ยังอ่านแชทและกรอกร่างด้วยมือได้ บัญชีแอดมินต้องมี password อย่างน้อย 16 ตัวอักษร ไม่ใช้ INQUIRY_ADMIN_TOKEN เป็นรหัสล็อกอิน

## Frontend บน Windows เชื่อม backend บน Ubuntu

ใช้ `run-frontend.bat` หรือกำหนด `DEV_BACKEND_MODE=remote` ใน root `.env` แล้วรัน `runall.bat` โหมดนี้เปิดเฉพาะ Next.js และ SSH tunnel ไม่เริ่ม MongoDB หรือ Rust backend ในเครื่อง ข้อมูลและการบันทึกทั้งหมดเป็นข้อมูลจริงบน Ubuntu

```dotenv
DEV_BACKEND_MODE=remote
DEV_SSH_HOST=Hermes
DEV_TUNNEL_PORT=28080
DEV_REMOTE_PORT=18080
DEV_FRONTEND_PORT=3000
```

เครื่อง Windows ต้องใช้ `ssh Hermes` เข้า server ได้ด้วย key ก่อน โดย alias อยู่ใน `%USERPROFILE%\.ssh\config` ตัว launcher ส่ง `/api/...` จาก Next.js ไป `127.0.0.1:28080` แล้วลอด SSH ไป `127.0.0.1:18080` บน Ubuntu จึงไม่ต้องเปิดพอร์ต backend เพิ่มหรือเปลี่ยน Caddy/LINE webhook

```bat
run-frontend.bat -Check
run-frontend.bat -ProbeOnly
run-frontend.bat
```

`-Check` ตรวจโปรแกรม/config โดยไม่เปิดบริการ; `-ProbeOnly` ตรวจ health/admin API โดยใช้ tunnel เดิมที่ปลายทางตรงกัน หรือเปิดชั่วคราวแล้วปิดเฉพาะตัวที่สร้างเอง; คำสั่งปกติเปิด `http://localhost:3000` ใช้บัญชีแอดมินจาก `.env` บน Ubuntu

รัน `runall.bat` ซ้ำได้ หาก frontend เป็นของโปรเจกต์นี้และชี้ไป backend ที่ตั้งไว้ สคริปต์จะใช้ตัวเดิม หาก SSH tunnel หลุดจะเปิดกลับมา ให้คงหน้าต่างที่แจ้ง `SSH connection restored` ไว้แล้วรีเฟรช browser เมื่อทั้ง frontend และ tunnel ทำงานอยู่แล้ว คำสั่งจะตรวจการเชื่อมต่อและจบโดยไม่เปิดซ้ำ หากพอร์ตเป็นของโปรแกรมอื่นหรือ frontend ชี้ไป backend คนละตัว จะบอกให้ปิดหน้าต่างต้นทางหรือเปลี่ยนพอร์ต โดยไม่ปิด process ให้อัตโนมัติ กด Ctrl+C ในหน้าต่างที่เปิดบริการเพื่อหยุดเฉพาะบริการที่หน้าต่างนั้นสร้าง

Backend บน Ubuntu ต้องเป็นรุ่น Step 2 ที่มี `/api/admin/session` แล้ว หากได้ 404 ให้ pull Git แล้วรัน `sudo bash ./docker-build.sh` และ `sudo bash ./docker-deploy.sh` บน Ubuntu โดยใช้ compiled release ที่ build ไว้บน local ไม่ต้อง deploy frontend ขึ้น server สำหรับโหมดนี้

Frontend ยังเรียก API แบบ relative และไม่มี token/password ของ server ใน browser bundle; เก็บ `ADMIN_COOKIE_SECURE=true` บน Ubuntu และเปิดผ่านชื่อ `localhost` ตาม launcher สำหรับทดสอบ cookie ใน Chrome หากรัน `npm run dev` เอง ให้ตั้ง `BACKEND_URL=http://127.0.0.1:28080` ใน `frontend/.env.development.local` และเปิด SSH tunnel ค้างไว้ด้วย

กลับมาทดสอบทุกส่วนในเครื่องได้ด้วย `runall.bat -LocalBackend` หรือเปลี่ยน `DEV_BACKEND_MODE=local` การเปลี่ยนปลายทางต้องหยุด frontend แล้วเริ่มใหม่ และล็อกอินใหม่

## ผู้ดูแลระบบ: ตั้ง Hermes สำหรับสกัดร่าง

พอร์ต dashboard 8646 ไม่ใช่ API Server ต้องเปิด Hermes API Server และตั้ง provider/model ที่อ่านภาพได้ก่อน การส่งรูปใช้ image_url แบบ data URL จาก GridFS โดยไม่เปิดรูปเป็น public

ใช้ Hermes profile แยกสำหรับงานสกัดร่าง ปิด tools ทุกตัวใน profile ผ่าน `hermes tools` และปิด memory/recall ของ profile ตามรุ่นที่ติดตั้ง งานนี้ต้องอ่านข้อมูลและคืน JSON เท่านั้น โค้ดตรวจ `GET /v1/toolsets` ก่อนส่งบริบท หากยังมี tools เปิดอยู่ หรือ endpoint นี้ไม่รองรับ จะหยุดด้วย HERMES_TOOLS_ENABLED ไม่ส่งข้อมูลแชทให้โมเดล การใส่ tools=[] หรือคำสั่งใน prompt เพียงอย่างเดียวไม่ได้ยืนยันว่า built-in tools ถูกปิด

ใน environment ของ Hermes profile:

```dotenv
API_SERVER_ENABLED=true
API_SERVER_KEY=REPLACE_WITH_A_SEPARATE_RANDOM_KEY
API_SERVER_PORT=8642
# Bind only on the server's private Docker bridge address, not the public interface.
API_SERVER_HOST=172.18.0.1
```

ตรวจ IP จริงก่อนด้วย `ip -4 addr show docker_gwbridge` ค่า 172.18.0.1 เป็นค่าที่พบใน server นี้ หากเครื่องอื่นใช้ subnet ต่างกันต้องเปลี่ยนให้ตรงกัน แล้ว restart gateway ของ profile ตาม service ที่ติดตั้งอยู่ `hermes -p <profile-name> gateway` เป็นคำสั่งเปิดแบบ interactive สำหรับทดสอบ ไม่ใช่การ compile

ตั้งใน `/home/ubuntu/alex-connectingpoint/.env`:

```dotenv
ADMIN_USERNAME=admin
ADMIN_PASSWORD=REPLACE_WITH_RANDOM_PASSWORD_AT_LEAST_16_CHARACTERS
ADMIN_COOKIE_SECURE=true
FRONTEND_PORT=18081
HERMES_API_URL=http://172.18.0.1:8642/v1/chat/completions
HERMES_API_KEY=REPLACE_WITH_THE_PROFILE_API_SERVER_KEY
HERMES_MODEL=hermes-agent
HERMES_TIMEOUT_SECONDS=120
```

HERMES_MODEL ใช้ alias ที่ profile ประกาศจริง; หากเปิด multi-profile routing ให้ใช้ URL ของ profile เช่น `/p/<profile-name>/v1/chat/completions` และ key ของ profile นั้น Backend ใน Docker ใช้ 127.0.0.1 เพื่อหา Hermes บน host ไม่ได้ ต้องใช้ private address ที่เข้าถึงได้จริง และทดสอบจากเครือข่าย Docker ด้วย

อ้างอิง [Hermes API Server](https://hermes-agent.nousresearch.com/docs/user-guide/features/api-server) และ [การเปิด/ปิด tools](https://hermes-agent.nousresearch.com/docs/user-guide/features/tools)

## Build และ deploy

บน local Windows เปิด Docker Desktop Linux containers:

```bat
build-backend.bat
build-frontend.bat
```

ทั้งคู่ compile บน local เท่านั้น ได้ `dist/line-linux` และ `dist/frontend-linux` พร้อม checksum/tag จากนั้นผู้พัฒนาเรียก `rungit.bat "Add admin workspace and order drafts"` เมื่อพร้อมส่งขึ้น Git

บน Ubuntu:

```sh
cd ~/alex-connectingpoint
bash ./loadgit.sh main
sudo bash ./docker-build.sh --full-stack
sudo bash ./docker-deploy.sh --full-stack
```

Full-stack รุ่นนี้ใช้ docker-stack.line.yml + docker-stack.admin.yml ร่วมกัน ไม่ย้ายไปฐานข้อมูล/volume/network ชุดใหม่ และยังคง LINE webhook ที่พอร์ต 18080 Frontend พอร์ต 18081 ใช้ Next.js proxy ไป backend:8080 ภายใน Docker ไม่มีการ compile Rust หรือ npm build บน Ubuntu

หน้า Alex ใช้ `https://hermes.salepie.online` ร่วมกับ Hermes บน server: Caddy ส่งเฉพาะ path ของ Alex ไป admin frontend (18081) ส่ง LINE webhook ไป backend (18080) ตามเดิม และ path อื่นทั้งหมดไป Hermes (`localhost:8646`) ตามเดิม ตรวจแล้วว่า Hermes ตอบ 404 ทุก path ในรายการนี้จึงไม่ชนกัน

แก้ `/etc/caddy/Caddyfile` (สำรองไฟล์เดิมก่อน):

```caddyfile
hermes.salepie.online {
    handle /api/line/webhook {
        reverse_proxy 127.0.0.1:18080
    }

    # Alex admin workspace: Next.js frontend, which proxies /api to the backend.
    @alex {
        path / /favicon.ico /_next/* /api/health /api/admin/* /api/inquiries /api/inquiries/* /api/conversations/* /api/line/* /api/draft-requests/* /api/media/*
        not path /api/line/webhook
    }
    handle @alex {
        reverse_proxy 127.0.0.1:18081
    }

    handle {
        reverse_proxy localhost:8646
    }
}
```

```sh
sudo cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak.$(date +%Y%m%d-%H%M%S)
sudo nano /etc/caddy/Caddyfile
sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
sudo systemctl reload caddy
```

เปิด `https://hermes.salepie.online` เพื่อล็อกอิน ADMIN_COOKIE_SECURE ต้องเป็น true (ค่าเริ่มต้นของ stack) หาก Hermes เพิ่ม path ใหม่ที่ซ้ำรายการ `@alex` ต้องแก้ matcher ให้ชัดก่อน พอร์ต 18081 แบบ host mode เปิดบนทุก interface เหมือน 18080 ควรให้เข้าผ่าน Caddy/HTTPS เท่านั้น เช่นปิดพอร์ตนี้จากภายนอกด้วย firewall ของ server

## ขั้นตอนทดสอบของแอดมิน

1. ส่งข้อความใหม่เข้า LINE OA หลังเปิด Use webhook; กดโหลดข้อมูลล่าสุดแล้วเลือก inquiry
2. อ่านแชทและรูป เลือกรูปที่จะส่งให้ AI หรือแนบรูปในสรุปโดยตรงได้รวมไม่เกิน 5 รูป/รวม 8 MB
3. วางสรุป เช่น `คุณเอ เช่าชุด DR-012 สีแดง ไซซ์ M จำนวน 1 ชุด รับ 15 พ.ย. 2569 คืน 18 พ.ย. 2569 ค่าเช่า 1,500 บาท มัดจำ 2,000 บาท รับหน้าร้านสาขา A`
4. กด **บันทึกสรุป** เพื่อเก็บข้อความและรูปที่เลือกไว้ก่อน แม้ยังไม่ตั้ง Hermes ก็ใช้ได้ เมื่อกด **เตรียมออเดอร์ด้วย AI** ระบบจะบันทึกสรุปที่แก้ให้อัตโนมัติก่อนเข้าคิว `draft_requests` ปิดหน้าเว็บแล้วกลับมาเปิด inquiry ต่อได้ รวมถึงเมื่อ AI ล้มเหลว
5. ตรวจวัน/จำนวน/เงิน ข้อมูลที่ขาด ข้อขัดแย้ง คำถามที่แนะนำ และหลักฐานอ้างอิง แก้ไขแล้วบันทึกเวอร์ชันใหม่ได้
6. เลือกยืนยันข้อมูลที่กรอกครบเพื่อเป็นข้อมูลอ้างอิงรอบถัดไป การยืนยันนี้ยังไม่ใช่ approval สำหรับ submit order
7. เมื่อมีข้อความใหม่หรือสรุปเปลี่ยนระหว่างวิเคราะห์ งานเดิมจะไม่ทับร่างล่าสุด ให้โหลดข้อมูลและเตรียมใหม่ ร่างที่อ้างสรุปรุ่นเก่าจะแสดงว่าบริบทเปลี่ยนและอนุมัติไม่ได้ หาก source ถูก unsend ทั้งสรุปที่บันทึกไว้และประวัติร่างที่อ้างอิงข้อความนั้นจะถูกซ่อนจาก API/UI
8. หากสองหน้าต่างแก้สรุปเดียวกัน ระบบยอมรับผู้ที่บันทึกก่อน อีกหน้าต่างจะได้รับ `SUMMARY_CONFLICT` และยังเก็บข้อความที่พิมพ์ไว้ให้คัดลอกก่อนกด **โหลดสรุปล่าสุด** การโหลดข้อมูลทั่วไปหรือ polling ไม่เขียนทับสรุปที่กำลังแก้

ข้อมูลเก็บใน MongoDB `admin_summaries`, `draft_requests`, `order_drafts` และ pointer/version ใน `inquiries` สรุปที่บันทึกก่อนใช้ AI อยู่ใน `inquiries.working_summary` มี revision, ผู้แก้ไข, เวลา, รูปที่เลือก และ hash บริบท ส่วน `admin_summaries` เก็บสำเนาสรุปของงานสกัดแต่ละครั้ง ทุกรายการจำกัด shop/channel จาก config ไม่รับร้านจาก browser ประวัติแสดงล่าสุด 50 ร่าง บริบทเตรียมร่างรองรับ 500 events ต่อ inquiry; เกินจะหยุดพร้อมข้อผิดพลาดโดยไม่ตัดข้อมูลเงียบ ๆ

API บันทึกสรุปคือ `POST /api/inquiries/{id}/summary` รับ `expected_revision` (เริ่มที่ 0), `text`, `image_event_ids` และ `context_hash` จาก `GET /api/inquiries/{id}/drafts` ซึ่งคืน `summary` ด้วย ใช้ HttpOnly session + `X-Admin-Request: 1` เหมือน mutation อื่น ผลสำเร็จคืน revision ใหม่; เวอร์ชันชนตอบ 409 `SUMMARY_CONFLICT` และบริบทแชทเปลี่ยนตอบ 409 `CONTEXT_CHANGED` endpoint นี้ไม่เรียก Hermes หรือสร้างร่าง

## เข้าสู่ระบบด้วย LINE Login

หน้า Alex ใช้ LINE Login (OAuth 2.1 / OpenID Connect ของ LINE) และรับเฉพาะ LINE user ID ใน `ADMIN_LINE_USER_IDS` รหัสผ่าน `ADMIN_PASSWORD` เป็นทางสำรอง ปล่อยว่างเพื่อปิดได้ LINE Login ยืนยันตัวตนอย่างเดียว ไม่ได้ให้สิทธิ์อ่านแชทใน OA Manager

1. LINE Developers Console → provider เดียวกับ Messaging API ของ OA → Create a new channel → **LINE Login** → App type: Web app
2. แท็บ LINE Login → Callback URL: `https://hermes.salepie.online/api/admin/line/callback` (ต้องตรงกับ `LINE_LOGIN_REDIRECT_URI` ทุกตัวอักษร) frontend บนเครื่องผ่าน `runall.bat` ใช้ได้เฉพาะรหัสผ่านสำรอง เพราะ LINE redirect กลับไปที่ domain นี้
3. ช่องทางที่ยังเป็น Developing ให้เฉพาะ Admin/Tester ของ channel ล็อกอินได้: เพิ่มบัญชีแอดมินในแท็บ Roles หรือ Publish channel
4. ใส่ใน `.env` บน server เท่านั้น แล้ว `sudo bash ./docker-deploy.sh`:

```dotenv
LINE_LOGIN_CHANNEL_ID=<Channel ID ตัวเลข>
LINE_LOGIN_CHANNEL_SECRET=<Channel secret>
LINE_LOGIN_REDIRECT_URI=https://hermes.salepie.online/api/admin/line/callback
ADMIN_LINE_USER_IDS=
```

5. ให้แอดมินกด **เข้าสู่ระบบด้วย LINE** ครั้งแรก หน้าเว็บจะแจ้งว่ายังไม่มีสิทธิ์พร้อม LINE user ID ของบัญชีนั้น นำ ID มาใส่ `ADMIN_LINE_USER_IDS` (คั่นด้วย comma) แล้ว deploy อีกครั้ง
6. ถอดสิทธิ์: ลบ ID ออกแล้ว deploy session เดิมของคนนั้นจะใช้ไม่ได้ทันที

Session เป็น HttpOnly cookie อายุ 8 ชั่วโมง ผูกกับ channel และรายชื่อที่อนุญาต; state/nonce ป้องกันการปลอม callback และ channel secret อยู่บน server เท่านั้น

## ตอบลูกค้าจาก Alex

ช่องพิมพ์ใต้แชทส่งข้อความถึงลูกค้าจริงด้วย Messaging API Push (`LINE_CHANNEL_ACCESS_TOKEN` เดิม) แล้วเก็บใน `outbound_messages` แสดงด้านขวาพร้อมชื่อแอดมินและป้าย "ส่งจาก Alex"

- ส่งได้เฉพาะแชท 1:1 ที่ลูกค้าทักเข้ามา ไม่ส่งเข้ากลุ่ม/ห้อง
- ใช้ request ID เป็น `X-Line-Retry-Key`: เมื่อ LINE ไม่ตอบหรือเน็ตหลุด ปุ่มเปลี่ยนเป็น **ส่งอีกครั้ง** และส่งซ้ำได้โดยลูกค้าไม่ได้รับสองครั้ง ถ้าแก้ข้อความก่อนส่งใหม่จะนับเป็นข้อความใหม่
- Push นับโควตาข้อความรายเดือนของแพ็กเกจ OA เมื่อเต็ม LINE ตอบ 429 และหน้าเว็บแจ้งว่าโควตาหมด
- ข้อความที่ส่งเป็นบริบทของ AI (`admin_message`) และทำให้ร่างเดิมถูกตรวจว่าบริบทเปลี่ยน: ก่อนอนุมัติออเดอร์ให้บันทึกร่างเป็นเวอร์ชันใหม่อีกครั้ง
- ข้อความที่พิมพ์ใน OA Manager หรือแอป LINE OA ไม่เข้า Alex ทีมควรตอบลูกค้าจาก Alex เท่านั้น ตัวเชื่อม OA Manager ([line-oa-auto-sync.md](line-oa-auto-sync.md)) ปิดไว้เป็นค่าเริ่มต้น (`OA_CONNECTOR_ENABLED=false`): หน้า Alex ไม่แสดงสถานะตัวเชื่อม และ backend ไม่รับข้อมูลจากตัวเชื่อม

## ส่งสติกเกอร์และรูปจาก Alex

- ปุ่ม 😊 ใต้แชทเปิดแผงสติกเกอร์ กดแล้วส่งทันที มีเฉพาะ 15 ชุด (456 ตัว) ที่ Messaging API อนุญาตให้ส่ง ตาม [รายการของ LINE](https://developers.line.biz/en/docs/messaging-api/sticker-list/) สติกเกอร์อื่นที่ OA Manager มีส่งผ่าน API ไม่ได้ backend ตรวจ package/sticker ID ก่อนส่งทุกครั้ง
- ปุ่ม 🖼️ แนบรูป: browser ย่อรูปเป็น JPEG ด้านยาวไม่เกิน 2048px (ไม่เกิน 10 MB) และ preview 240px (ไม่เกิน 1 MB) ตามข้อกำหนด LINE → อัปโหลดเก็บใน GridFS (`outbound_media`) → แสดงตัวอย่างให้กด **ส่งรูป** อีกครั้งก่อนถึงลูกค้า
- LINE ดึงรูปจาก `https://<PUBLIC_BASE_URL>/api/line/outbound-media/<token>/original|preview` โดยไม่ต้องล็อกอิน token สุ่มยาว 64 ตัวอักษร เดาไม่ได้ เก็บไว้ตาม `LINE_MEDIA_RETENTION_DAYS` (ค่าเริ่มต้น 30 วัน) ถ้าไม่ตั้ง `PUBLIC_BASE_URL` ระบบใช้ origin ของ `LINE_LOGIN_REDIRECT_URI` (https://hermes.salepie.online) path นี้อยู่ใน `/api/line/*` ที่ Caddy ส่งไป Alex อยู่แล้ว
- ส่งไม่สำเร็จจะมีปุ่ม "ส่งสติกเกอร์/รูปอีกครั้ง" ซึ่งใช้ request ID เดิม ลูกค้าไม่ได้รับซ้ำ ทุกข้อความนับโควตา Push ของ OA

## ชื่อ รูปโปรไฟล์ และสติกเกอร์ของลูกค้า

- worker ดึงชื่อที่แสดงและรูปโปรไฟล์ด้วย Messaging API **Get profile** (`/v2/bot/profile/{userId}`, token เดิม) เมื่อประมวลผลข้อความ เก็บใน `line_users` (`display_name`, `picture_url`, `profile_status`) และดึงซ้ำไม่เกินวันละครั้งต่อคน ลูกค้าเก่าที่ยังไม่มีข้อมูลจะถูกดึงเบื้องหลังเมื่อเปิดรายการ inquiry (ครั้งละไม่เกิน 20 คน)
- ลูกค้าที่ไม่ได้เป็นเพื่อนหรือบล็อก OA ได้ `profile_status=UNAVAILABLE` และแสดงเป็นรหัส LINE เหมือนเดิม ดึงไม่สำเร็จไม่ทำให้การรับข้อความล้ม
- ชื่อใน LINE ไม่ใช่ชื่อจริง จึงใช้แสดงผลและเป็น placeholder ในแท็บลูกค้าเท่านั้น ไม่ถูกใส่ในร่างออเดอร์อัตโนมัติ
- สติกเกอร์: webhook ส่งเฉพาะ `stickerId`/`keywords` หน้าเว็บโหลดรูปจาก CDN ของ LINE (`stickershop.line-scdn.net`) ซึ่งไม่ใช่ API ที่ LINE รับรอง หากโหลดไม่ได้จะแสดงเป็นข้อความ "สติกเกอร์: keywords"; LINE emoji ในข้อความยังแสดงเป็นข้อความตามเดิม

## ข้อมูลลูกค้าและอนุมัติออเดอร์

- **แท็บ "ลูกค้า":** บันทึกชื่อ เบอร์โทร ที่อยู่ และโน้ตภายในต่อ LINE user ID ลง `customer_profiles` ใช้ร่วมกันทุก inquiry ของลูกค้าคนเดียวกัน ลูกค้าไม่เห็นข้อมูลนี้ บันทึกได้เฉพาะผู้ใช้ที่เคยส่งข้อความผ่าน webhook และใช้ `version` กันการบันทึกทับจากหลายหน้าต่าง หากชนกันระบบจะโหลดข้อมูลล่าสุดมาให้ตรวจก่อนบันทึกใหม่
- **ปุ่ม "อนุมัติเป็นออเดอร์":** แสดงในแท็บร่างล่าสุดเมื่อร่างล่าสุดเป็น READY_FOR_REVIEW (ข้อมูลครบและไม่มีข้อขัดแย้ง) แชทไม่มีข้อความใหม่หลังทำร่าง และแอดมินติ๊กยืนยันแล้ว ระบบ snapshot ฟิลด์ของร่างลง `orders` สถานะ CONFIRMED พร้อมผู้อนุมัติและเวลา และเก็บ `order_id` ใน inquiry ได้ 1 ออเดอร์ต่อ inquiry การกดซ้ำด้วยร่างเดิมคืนออเดอร์เดิม แก้ร่างภายหลังได้แต่ออเดอร์ที่อนุมัติแล้วไม่เปลี่ยนตาม
- ออเดอร์ยังมี `pricing_verified`, `product_verified`, `availability_verified` เป็น false: ยังไม่ได้ตรวจสต็อก คิว หรือราคากับระบบ Alex และยังไม่ส่งข้อความใดถึงลูกค้า

หากบทสนทนามีหลาย inquiry และข้อความยังเป็น NEEDS_ADMIN ต้องใช้ API assign ของ Step 1 เลือก inquiry ให้ชัดก่อน งานเตรียมร่างจะรอและไม่เดาว่าข้อความนั้นเป็นของ inquiry ใด หน้าจอรุ่นนี้ยังไม่มีปุ่ม assign สำหรับกรณีนี้

## API สำหรับผู้พัฒนา

| Method / path | การใช้งาน |
| --- | --- |
| POST /api/admin/login | username/password → HttpOnly SameSite=Strict cookie อายุ 8 ชั่วโมง |
| GET /api/admin/session | ผู้ใช้และสถานะว่าตั้งค่า Hermes หรือยัง |
| DELETE /api/admin/session | ล้าง cookie ของ browser |
| GET /api/inquiries/{id}/drafts | version, context_hash, drafts, latest_request |
| POST /api/inquiries/{id}/prepare | summary, request_id, expected_version, image_event_ids → 202 และ job ID |
| POST /api/inquiries/{id}/draft | fields, summary, request_id, expected_version, context_hash, confirmed_fields → บันทึก manual revision ผ่านคิว |
| GET /api/draft-requests/{id} | สถานะงานและ error_code ที่ไม่มี secret/ข้อความ driver |
| GET /api/admin/login-options | วิธีล็อกอินที่เปิด `{line, password}` |
| GET /api/admin/line/start → /api/admin/line/callback | LINE Login: redirect ไป LINE และรับ code กลับ ตั้ง session cookie หรือ redirect `/?login_error=...` |
| POST /api/inquiries/{id}/media | original, preview (base64 JPEG/PNG ≤10 MB/≤1 MB) → 201 `{media:{_id, preview_url, original_url}}`; 503 IMAGE_UNAVAILABLE เมื่อไม่มี HTTPS base |
| GET /api/line/outbound-media/{token}/{original\|preview} | สาธารณะสำหรับ LINE ดึงรูปที่ส่ง |
| POST /api/inquiries/{id}/messages | request_id (UUID) + อย่างใดอย่างหนึ่ง: text, sticker {package_id, sticker_id}, image_id → 201 ส่งแล้ว; 409 REPLY_UNAVAILABLE / RECORD_CONFLICT; 502 LINE_UNAVAILABLE / LINE_RATE_LIMITED / LINE_SEND_FAILED |
| GET /api/line/users/{user_id}/profile | ข้อมูลลูกค้า (หรือ null) |
| POST /api/line/users/{user_id}/profile | expected_version (0 = สร้างใหม่), display_name, phone, address, notes → 409 RECORD_CONFLICT เมื่อ version ไม่ตรง |
| GET /api/inquiries/{id}/order | ออเดอร์ที่อนุมัติแล้ว (หรือ null) |
| POST /api/inquiries/{id}/order | draft_id, draft_version, context_hash → 201; 409 ORDER_NOT_READY / CONTEXT_CHANGED / ORDER_EXISTS |

API ภายในเดิมยังรับ Authorization Bearer INQUIRY_ADMIN_TOKEN สำหรับเครื่องมือฝั่ง server; browser ใช้ cookie เท่านั้น Mutating cookie requests และ login ต้องมี X-Admin-Request: 1 ป้องกันการส่ง form ข้ามเว็บไซต์ ไม่เก็บ token/password ใน localStorage หรือ frontend bundle

request_id ซ้ำพร้อม payload เดิมคืน job เดิม; payload ต่างกันคืน conflict expected_version ป้องกันการเขียนทับร่างจากผู้ดูแลอีกคน มี lease/recovery และ atomic pointer update โดยไม่ต้องใช้ MongoDB replica-set transactions Job failure ไม่สร้างร่างที่ active และไม่ลบร่างก่อนหน้า หาก AI timeout/failure ให้ตรวจสถานะแล้วกดเตรียมใหม่ด้วย request ใหม่

Validator ตรวจ schema แบบไม่รับ field เกิน, ISO Gregorian date ที่เป็นวันจริง, วันคืนไม่ก่อนรับ, quantity 1–999, เงินเป็นจำนวนเต็มสตางค์, source references อยู่ในบริบท และ quote ตรงต้นฉบับ ทุกค่าที่ AI เติมต้องมีหลักฐาน การตรวจนี้ไม่ได้ยืนยันว่ารหัสสินค้ามีจริง ราคาถูกต้อง หรือคิวว่าง ซึ่งยังต้องทำ Step 3–4


ดู [ขั้นตอนสรุปจากรูปและสร้างบิลใน Alex](image-rental-billing.md) สำหรับวันเช่าแยกรายการ เติมข้อมูลลูกค้าจากแชท เช็คคิว และการส่งออเดอร์จริง

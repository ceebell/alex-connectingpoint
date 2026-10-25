# Step 2 — หน้าหลังบ้านและร่างออเดอร์

ทำแล้วทั้ง Next.js frontend และ Rust backend: แอดมินเข้าสู่ระบบ → เลือก inquiry → อ่านแชท/รูป → ใส่สรุป → กด **เตรียมออเดอร์ด้วย AI** → ตรวจ/แก้ร่าง → บันทึกเป็นเวอร์ชันใหม่ ไม่มีการส่งข้อความถึงลูกค้า เช็คคิว หรือสร้างออเดอร์ในขั้นนี้

เพิ่มตัวเชื่อม OA Manager บน Ubuntu แล้ว ไม่มีช่องคัดลอกข้อความมาบันทึกเอง ต้อง deploy ตัวเชื่อม ล็อกอิน LINE ครั้งแรก และทดสอบ schema ของ OA จริงก่อนถือว่าเปิดซิงก์สำเร็จ ดูคำสั่งและหน้าที่แต่ละฝ่ายที่ [รับข้อความแอดมินอัตโนมัติ](line-oa-auto-sync.md)

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

เพิ่ม DNS `admin.salepie.online` ให้ชี้ server และเพิ่ม site block นี้ใน `/etc/caddy/Caddyfile` โดยเก็บ Hermes/webhook site เดิมไว้:

```caddyfile
admin.salepie.online {
    reverse_proxy 127.0.0.1:18081
}
```

```sh
sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
sudo systemctl reload caddy
```

เปิด `https://admin.salepie.online` เพื่อล็อกอิน hostname นี้เป็นตัวอย่างที่ต้องตั้ง DNS ก่อนจริง การเปิดผ่าน HTTP บน IP โดย ADMIN_COOKIE_SECURE=true จะล็อกอินไม่ได้เพราะ cookie ใช้กับ HTTPS

## ขั้นตอนทดสอบของแอดมิน

1. ส่งข้อความใหม่เข้า LINE OA หลังเปิด Use webhook; กดโหลดข้อมูลล่าสุดแล้วเลือก inquiry
2. อ่านแชทและรูป เลือกรูปที่จะส่งให้ AI ได้ไม่เกิน 4 รูป/รวม 8 MB
3. วางสรุป เช่น `คุณเอ เช่าชุด DR-012 สีแดง ไซซ์ M จำนวน 1 ชุด รับ 15 พ.ย. 2569 คืน 18 พ.ย. 2569 ค่าเช่า 1,500 บาท มัดจำ 2,000 บาท รับหน้าร้านสาขา A`
4. กดเตรียมออเดอร์ด้วย AI งานจะถูกเก็บใน draft_requests แม้ปิดหน้าเว็บก็กลับมาเปิด inquiry เพื่อตรวจผลได้
5. ตรวจวัน/จำนวน/เงิน ข้อมูลที่ขาด ข้อขัดแย้ง คำถามที่แนะนำ และหลักฐานอ้างอิง แก้ไขแล้วบันทึกเวอร์ชันใหม่ได้
6. เลือกยืนยันข้อมูลที่กรอกครบเพื่อเป็นข้อมูลอ้างอิงรอบถัดไป การยืนยันนี้ยังไม่ใช่ approval สำหรับ submit order
7. เมื่อมีข้อความใหม่ระหว่างวิเคราะห์ งานเดิมเป็น STALE ไม่ทับร่างล่าสุด ให้โหลดข้อมูลและเตรียมใหม่ หาก source ถูก unsend ประวัติร่างที่อ้างอิงข้อความนั้นจะถูกซ่อนจาก API/UI

ข้อมูลเก็บใน MongoDB `admin_summaries`, `draft_requests`, `order_drafts` และ pointer/version ใน `inquiries` ทุกรายการจำกัด shop/channel จาก config ไม่รับร้านจาก browser ประวัติแสดงล่าสุด 50 ร่าง บริบทเตรียมร่างรองรับ 500 events ต่อ inquiry; เกินจะหยุดพร้อมข้อผิดพลาดโดยไม่ตัดข้อมูลเงียบ ๆ

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
| GET /api/line/users/{user_id}/profile | ข้อมูลลูกค้า (หรือ null) |
| POST /api/line/users/{user_id}/profile | expected_version (0 = สร้างใหม่), display_name, phone, address, notes → 409 RECORD_CONFLICT เมื่อ version ไม่ตรง |
| GET /api/inquiries/{id}/order | ออเดอร์ที่อนุมัติแล้ว (หรือ null) |
| POST /api/inquiries/{id}/order | draft_id, draft_version, context_hash → 201; 409 ORDER_NOT_READY / CONTEXT_CHANGED / ORDER_EXISTS |

API ภายในเดิมยังรับ Authorization Bearer INQUIRY_ADMIN_TOKEN สำหรับเครื่องมือฝั่ง server; browser ใช้ cookie เท่านั้น Mutating cookie requests และ login ต้องมี X-Admin-Request: 1 ป้องกันการส่ง form ข้ามเว็บไซต์ ไม่เก็บ token/password ใน localStorage หรือ frontend bundle

request_id ซ้ำพร้อม payload เดิมคืน job เดิม; payload ต่างกันคืน conflict expected_version ป้องกันการเขียนทับร่างจากผู้ดูแลอีกคน มี lease/recovery และ atomic pointer update โดยไม่ต้องใช้ MongoDB replica-set transactions Job failure ไม่สร้างร่างที่ active และไม่ลบร่างก่อนหน้า หาก AI timeout/failure ให้ตรวจสถานะแล้วกดเตรียมใหม่ด้วย request ใหม่

Validator ตรวจ schema แบบไม่รับ field เกิน, ISO Gregorian date ที่เป็นวันจริง, วันคืนไม่ก่อนรับ, quantity 1–999, เงินเป็นจำนวนเต็มสตางค์, source references อยู่ในบริบท และ quote ตรงต้นฉบับ ทุกค่าที่ AI เติมต้องมีหลักฐาน การตรวจนี้ไม่ได้ยืนยันว่ารหัสสินค้ามีจริง ราคาถูกต้อง หรือคิวว่าง ซึ่งยังต้องทำ Step 3–4

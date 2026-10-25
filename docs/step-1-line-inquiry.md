# Step 1 — รับ inquiry จาก LINE OA

Backend นี้ทำข้อ 4 ของ `alex-line-hermes-mcp-workflow.md`: รับ webhook → ตรวจลายเซ็น → เก็บเหตุการณ์ถาวร → worker ผูกบทสนทนา/inquiry และเก็บรูป → เตรียมงานวิเคราะห์สำหรับ Step 2

## เริ่มใช้งานบน Windows

หากใช้งานบน Ubuntu server จริง ให้ใช้ [คู่มือ deploy backend และ MongoDB ด้วย Caddy](deploy-line-server.md) แทนขั้นตอน Windows ด้านล่าง ผู้พัฒนาใช้ build-line.bat compile Linux release บน local ก่อนส่งไป server; Ubuntu ใส่ binary ใน Docker และ deploy เท่านั้น หลัง deploy ไม่ต้องเปิด runall.bat หรือ SSH tunnel บนคอมค้างไว้

1. ติดตั้ง Rust และ MongoDB โดยให้ `cargo` และ `mongod` อยู่ใน PATH
2. คัดลอก `.env.example` เป็น `.env` ที่ root แล้วกรอก `LINE_SHOP_ID`, `LINE_CHANNEL_ID`, `LINE_DESTINATION_ID`, `LINE_CHANNEL_SECRET`, `LINE_CHANNEL_ACCESS_TOKEN`, `INQUIRY_ADMIN_TOKEN` และค่าฐานข้อมูล
3. จาก root รัน:

```powershell
.\runall.bat
```

ตรวจ `GET http://127.0.0.1:8080/api/health` ได้ `{"status":"ok"}` ไฟล์ `.env` ถูกโหลดจาก root ผ่าน dotenvy ไม่ต้องส่ง credentials ในคำสั่ง

`LINE_DESTINATION_ID` คือ bot user ID ที่อยู่ใน webhook `destination` ไม่ใช่ channel ID ชื่อร้าน/ช่องทางใช้ค่าที่ server ตั้งไว้เท่านั้น หนึ่ง deployment รองรับหนึ่งร้านและหนึ่ง LINE channel หากต้องแยกหลายร้านให้ใช้ deployment และ credentials แยกกัน

`runall.bat` ตรวจ config ก่อนเปิดบริการ เริ่ม MongoDB ในเครื่องตามพอร์ตของ `MONGODB_URI` และเปิด backend โดยใช้ `PORT` จาก `.env` หากยังไม่มี `frontend/package.json` จะข้าม frontend โดยไม่ต้องติดตั้ง Node.js/npm เมื่อเพิ่ม frontend แล้วจะเริ่มให้พร้อม proxy ไปยังพอร์ต backend ที่ตั้งไว้

ตรวจความพร้อมโดยไม่เปิดบริการด้วย `.\runall.bat -Check` สคริปต์ไม่แสดงค่า secret และไม่เขียนทับ `.env` หากใช้ MongoDB ภายนอกหรือ URI ที่มี credentials สคริปต์จะใช้การเชื่อมต่อที่ตั้งไว้โดยไม่เปิด MongoDB ใหม่ หาก MongoDB หรือ backend ที่ health ผ่านรันอยู่แล้วจะใช้ process เดิม กรณี backend ยังไม่รัน การ compile/startup จะแสดงในหน้าต่าง backend

ถ้าต้องการเปิด backend เอง ให้ใช้ `cargo run --manifest-path backend/Cargo.toml` จาก root เช่นเดิม

## ตั้งค่า LINE OA

1. เปิด Messaging API ให้ LINE OA และนำ channel secret/access token มากรอกใน `.env` โดยไม่ commit ไฟล์จริง
2. ตั้ง HTTPS reverse proxy/tunnel มาที่ backend แล้วกำหนด Webhook URL เป็น `https://<your-domain>/api/line/webhook`
3. กด Verify ใน LINE Developers: payload `events: []` ที่ลงลายเซ็นถูกต้องต้องได้ HTTP 200
4. เปิด Use webhook และเปิด Webhook redelivery ตามการใช้งาน
5. ส่งข้อความ รูป และปุ่ม postback จากบัญชีทดสอบ ตรวจเหตุการณ์และ inquiry ผ่าน API ภายในด้านล่าง

การตรวจ raw body และประมวลผลแบบ asynchronous อ้างอิง [LINE webhook](https://developers.line.biz/en/docs/messaging-api/receiving-messages/) และ [การตรวจลายเซ็น](https://developers.line.biz/en/docs/messaging-api/verify-webhook-signature/) ดาวน์โหลดรูปจาก message ID ผ่าน [Content API](https://developers.line.biz/en/reference/messaging-api/#get-content) โดยไม่เรียก URL ที่ลูกค้าส่งมา

ยังต้องทดสอบกับ LINE OA จริงหลังกรอก credentials และมี HTTPS URL การทดสอบอัตโนมัติใช้ payload จำลองและ MongoDB จริง ไม่ส่งข้อความถึงลูกค้า

## สิ่งที่บันทึก

- `line_users`: ทะเบียนผู้ใช้จาก `source.userId` หนึ่งรายการต่อ shop/channel/user ID พร้อม `first_seen_at`, `last_seen_at`, `created_at`, `updated_at`; รับทั้ง follow และเหตุการณ์อื่นที่มี source แบบ user/group/room พร้อม user ID โดยไม่เรียก profile API
- `line_events`: เหตุการณ์และข้อความต้นฉบับ, shop/channel, message ID, `event_at`, `received_at`, queue status, retry/lease, conversation/inquiry และ media reference
- `conversations`: แยกตาม shop/channel, ประเภทแชท, chat ID และ user ID เพื่อไม่ปนผู้ส่งในกลุ่มเดียวกัน
- `inquiries`: default inquiry ต่อบทสนทนา สถานะเริ่ม `RECEIVED` โหมด `ADMIN_ASSIST`
- `analysis_jobs`: งาน `PENDING` หนึ่งรายการต่อ inquiry พร้อม `event_ids` ที่ไม่ซ้ำและ `ready_at` สำหรับ Step 2
- `withdrawn_messages`: tombstone สำหรับ unsend รวมกรณี unsend มาถึงก่อนข้อความต้นฉบับ
- `fs.files` / `fs.chunks`: รูปใน MongoDB GridFS แชร์ได้ระหว่าง backend replicas และไม่เปิดเป็น public

Collection ของแอปมี `_id` แบบ ObjectId, `created_at`, `updated_at` แบบ BSON DateTime ส่วน GridFS ใช้ schema มาตรฐานของ MongoDB โดยเก็บเวลาเพิ่มเติมใน metadata

ตอบ HTTP 200 หลังทุกเหตุการณ์ใน request ได้รับการยืนยันการเขียนจาก MongoDB ด้วย majority + journal เท่านั้น หากเขียนบางรายการสำเร็จแล้วรายการถัดไปล้มเหลวจะตอบ 503 การส่งซ้ำไม่สร้างรายการเดิมเพิ่มด้วย unique index `(shop_id, channel_id, event_id)` ไม่พึ่งคิวในหน่วยความจำ

worker claim ด้วย atomic update และ lease 120 วินาที เมื่อ process หยุดก่อนจบ replica อื่น/การเริ่มใหม่รับงานต่อหลัง lease หมด งาน retry สูงสุด 8 ครั้งด้วย backoff แล้วเปลี่ยนเป็น `FAILED` มี structured log ที่ไม่ใส่ข้อความลูกค้า/token และมี API ตรวจสถานะ ต้องจัดระบบ monitor log/จำนวน `FAILED` ใน deployment จริง

## API ภายใน

ทุก API ในตารางต้องส่ง `Authorization: Bearer <INQUIRY_ADMIN_TOKEN>` และได้ `Cache-Control: no-store` ควรเรียกผ่าน server/admin client ที่เก็บ token ได้ ห้ามฝัง token ลง frontend หรือใช้ query string ทุก query จำกัดร้านและช่องทางจาก config

| Method / path | การใช้งาน |
| --- | --- |
| `GET /api/line/users` | ทะเบียนผู้ใช้ล่าสุด 100 รายการ ตอบ `{users, next}`; อ่านต่อด้วย `?before=<next>` |
| `GET /api/line/users/{line_user_id}/messages` | ประวัติข้อความและ postback ของผู้ใช้ สูงสุด 200 รายการ ตอบ `{line_user_id, messages, next}` เรียง `event_at`, `event_id`; อ่านต่อด้วย `?after=<next>` |
| `GET /api/inquiries` | รายการ inquiry ล่าสุด 100 รายการ; filter `?status=RECEIVED`, cursor `?before=<ObjectId>` |
| `GET /api/inquiries/{id}/context` | ข้อความเรียงตาม `event_at`, `event_id` และแบ่ง burst ตาม `LINE_BURST_WINDOW_MS` สูงสุด 200 ข้อความต่อหน้า; ใช้ `?after=<next>` อ่านต่อ |
| `POST /api/conversations/{id}/inquiries` | เปิด inquiry ใหม่ของบทสนทนาเดิม |
| `GET /api/line/events` | รายการเหตุการณ์ล่าสุด 100 รายการ; filter `?status=NEEDS_ADMIN` หรือ `FAILED`; cursor `?before=<ObjectId>` |
| `POST /api/line/events/{id}/assign` | เลือก inquiry ให้เหตุการณ์ `NEEDS_ADMIN` ด้วย JSON `{"inquiry_id":"<ObjectId>"}` |
| `POST /api/line/events/{id}/retry` | นำเหตุการณ์ `FAILED` กลับเข้าคิวหลังแก้สาเหตุแล้ว |
| `GET /api/line/status` | จำนวนเหตุการณ์แยกตามสถานะ สำหรับ monitor |
| `GET /api/media/{id}` | อ่านรูปตามสิทธิ์; รูปหมดอายุหรือถูก unsend จะได้ 404 |

ID และวันเวลาจาก BSON ใน JSON ใช้ Extended JSON เช่น `{"$oid":"..."}` และ `{"$date":"..."}`

ตัวอย่างอ่าน inquiry โดยรับ token จาก environment ของเครื่องมือแอดมิน:

```powershell
$headers = @{ Authorization = "Bearer $env:INQUIRY_ADMIN_TOKEN" }
Invoke-RestMethod http://127.0.0.1:8080/api/inquiries -Headers $headers
Invoke-RestMethod http://127.0.0.1:8080/api/line/status -Headers $headers
```

`dotenvy` โหลด `.env` ให้ process backend เท่านั้น ไม่ได้ตั้ง environment ของ PowerShell ให้โดยอัตโนมัติ

เมื่อมี inquiry เปิดอยู่รายการเดียวจะผูกข้อความให้อัตโนมัติ หากมีหลายรายการจะเก็บเหตุการณ์เป็น `NEEDS_ADMIN` โดยไม่เดาว่าเป็นงานใด แอดมินใช้ `/assign` เลือก inquiry ที่ยังเปิดและอยู่ในบทสนทนาเดียวกัน ข้อความที่ไม่มี user ID จะคง `NEEDS_ADMIN` เพื่อให้ตรวจสอบต้นทาง ไม่ผูกกับลูกค้าคนอื่น

## ทะเบียนผู้ใช้และประวัติรายคน

ระบบบันทึก `line_users` ก่อน ACK webhook ควบคู่กับ `line_events` หากทะเบียนเขียนไม่สำเร็จจะตอบ 503 และเติมข้อมูลจาก redelivery ได้โดยไม่สร้างข้อความซ้ำ unique index จำกัด `(shop_id, channel_id, line_user_id)` ส่วน first/last seen ใช้ min/max ของเวลาเหตุการณ์เพื่อรองรับการมาถึงสลับลำดับ ไม่มีการนับ duplicate เป็นผู้ใช้ใหม่

ผู้ใช้เพิ่มเพื่อนอย่างเดียวมีทะเบียนโดยยังไม่เปิด inquiry ผู้ส่งข้อความครั้งแรกมีทะเบียนได้แม้ไม่มี follow event ก่อนหน้า เหตุการณ์ที่ไม่มี user ID ยังถูกเก็บ แต่ไม่สร้างผู้ใช้สมมติ และไม่ใช้ group ID/room ID มาแทน user ID ตาม [คู่มือรับ user ID ของ LINE](https://developers.line.biz/en/docs/messaging-api/getting-user-ids/)

ประวัติอ่านจาก `line_events.payload.source.userId` จึงรวมข้อความเดิมที่เคยเก็บอยู่แล้ว และอ่านข้อความใหม่ได้ทันทีขณะสถานะ `PENDING`, `PROCESSING`, `FAILED` หรือ `NEEDS_ADMIN` โดยไม่รอ inquiry ประมวลผลสำเร็จ ประวัติรวมเฉพาะข้อความและ postback ของผู้ส่งคนนั้นในแชทส่วนตัวและกลุ่ม/ห้องของเขา โดยยังมี `payload.source` แยกแหล่งแชท ไม่รวมข้อความของสมาชิกคนอื่น

ทะเบียนผู้ใช้เก่าจะเพิ่มเมื่อมี webhook ใหม่ ไม่มีการสแกนย้อนหลังตอน startup หรือดึงแชทเก่าจาก LINE และยังไม่บันทึกคำตอบจาก OA Manager อัตโนมัติ เมื่อ unsend จะซ่อน payload/media reference จาก API ประวัติทันทีและเก็บ metadata ที่ `withdrawn=true` ไว้ให้ cursor ยังอ่านต่อได้ เมื่อไม่มีข้อมูล API จะตอบรายการว่าง; cursor ผิดรูปแบบตอบ 400 และ cursor ที่เป็นของร้าน/ช่องทาง/ผู้ใช้อื่นตอบ 404

ตัวอย่างรับ token แบบซ่อนและเรียกดูประวัติอยู่ใน [คู่มือหลัก หัวข้อ 4.1](../alex-line-hermes-mcp-workflow.md#step1-user-history) และ [HTML](../alex-line-hermes-mcp-workflow.html#s04-users) ส่วนหน้าจอ inbox ยังไม่ได้พัฒนาในงานนี้

## รูปและข้อความที่ยกเลิก

รูปดาวน์โหลดเฉพาะ `image/jpeg`, `image/png`, `image/gif`, `image/webp` ขนาดไม่เกิน `LINE_MEDIA_MAX_BYTES` (default 20 MiB) ไม่ตาม HTTP redirect รูปจาก content provider ภายนอกเก็บเป็น `media_status=NEEDS_ADMIN` และไม่ดาวน์โหลด URL ภายนอก

เก็บรูป default 30 วัน ปรับด้วย `LINE_MEDIA_RETENTION_DAYS` worker ลบรูปหมดอายุทุกประมาณ 60 วินาที API ปฏิเสธรูปหมดอายุทันที ไม่ใช้ TTL ลบแค่ `fs.files` เพราะจะทิ้ง chunks ค้าง ข้อความและ event metadata ยังเก็บต่อ ไม่มีการลบตามอายุอัตโนมัติใน Step 1

unsend สร้าง tombstone และลบเนื้อหาข้อความ/รูปออกจาก event และ GridFS โดย API ซ่อนเนื้อหาตั้งแต่พบ tombstone แม้ worker ยังไม่ลบเสร็จ งานวิเคราะห์ถูกทำเครื่องหมายใหม่เพื่ออ่านบริบทล่าสุดจากฐานข้อมูล ไม่คัดลอกข้อความไว้ในคิว

## ขอบเขตส่งต่อ Step 2

Step 1 ไม่เรียก Hermes โดยตรง ปัจจุบันมี [หน้าหลังบ้าน Step 2](step-2-admin.md) ให้แอดมินส่งสรุปและสั่งเตรียมร่างผ่าน draft_requests แล้ว โดยใช้ analysis_jobs เป็นตัวตรวจการรวมข้อความและบริบทที่เปลี่ยนไป ยังไม่มีการเรียก Alex API/MCP หรือส่ง reply ให้ลูกค้า

consumer ต้องรอ `ready_at`, อ่าน event IDs/บริบทล่าสุด เรียงตาม event time ตรวจ tombstone และป้องกันการปิดงานทับข้อความใหม่ด้วย compare-and-set ของ snapshot `updated_at`/event IDs ต้องเช็กว่าข้อความในบทสนทนายังมี worker ประมวลผลค้างหรือไม่ก่อนสรุป burst และรองรับข้อความย้อนหลังที่มาถึงหลังการวิเคราะห์ครั้งก่อน ห้ามใช้ pagination cursor เป็นตัวบ่งชี้ว่าไม่มีเหตุการณ์ย้อนหลังเพิ่ม

`/context` แบ่งหน้าตาม event time จึงอาจแบ่ง burst ตรงขอบหน้า consumer ต้องรวม burst ข้ามหน้าตามเวลาอีกครั้ง การตีความวันที่เช่า การสรุปร่าง และการยืนยันสินค้าจะทำในขั้นถัดไป

## ตรวจสอบ

```powershell
cd backend
cargo fmt --check
cargo clippy -- -D warnings
cargo test
# Use a disposable MongoDB instance; the test creates and removes only its own step1_test_* database.
$env:TEST_MONGODB_URI = "mongodb://127.0.0.1:27018"
cargo test -- --ignored
```

integration tests ตรวจ durable intake, duplicate, expired lease/reconnect, ลำดับข้อความ/burst, แยกร้านและช่องทาง, inquiry กำกวมและ admin assign, GridFS, unsend, ทะเบียนผู้ใช้จาก follow/ข้อความ, การเติมทะเบียนหลังเขียนไม่ครบ, ประวัติแยกผู้ส่งในกลุ่ม และ pagination 100 ผู้ใช้/200 ข้อความ โดยไม่ต้องใช้ LINE credentials จริง

Docker Swarm ส่งตัวแปร LINE และ admin token จาก `.env` บน server ผ่าน `docker-stack.yml` แล้ว ต้องมี HTTPS reverse proxy ที่ route `/api/line/webhook` ไป backend โดยตรง หรือผ่าน frontend proxy เมื่อสร้าง frontend ในงานถัดไป

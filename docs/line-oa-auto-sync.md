# รับข้อความแอดมินจาก LINE OA อัตโนมัติ

> **ไม่จำเป็นเมื่อตอบลูกค้าจาก Alex:** ตั้งแต่ 2026-10-25 แอดมินเข้า Alex ด้วย LINE Login และตอบลูกค้าผ่าน Messaging API ได้ ([step-2-admin.md](step-2-admin.md)) ข้อความจึงเข้า MongoDB โดยตรงด้วย API ทางการ ไม่ต้องรัน `login-oa.bat` ใช้ตัวเชื่อมนี้เฉพาะเมื่อยังต้องเก็บข้อความที่พิมพ์ใน OA Manager; ปิดได้ด้วย `sudo docker stop <ชื่อ container>` โดย profile volume ยังอยู่

เพิ่มตัวเชื่อม OA Manager บน Ubuntu แล้ว: browser ทำงานเบื้องหลัง → อ่านข้อความแอดมิน → backend จับคู่จาก message ID ของลูกค้าที่ได้รับทาง webhook → เก็บ `oa_messages` ใน MongoDB → frontend อัปเดตแชทและแสดงแอดมินด้านขวาพื้นฟ้าอ่อน ไม่มีช่องคัดลอกข้อความเอง และตัวเชื่อมบล็อก API ส่ง/แก้ไขข้อความของ OA

**สถานะ:** source, build scripts และ Linux releases พร้อมแล้ว ทดสอบเส้นทางครบด้วย browser/Docker/MongoDB และข้อมูลจำลองผ่าน ยังไม่ได้ล็อกอินหรือยืนยัน schema ของบัญชี OA จริง จึงยังไม่ถือว่า production sync สำเร็จ รุ่นแรกเก็บข้อความตัวอักษรขาออก; รูปจากลูกค้ายังเข้าระบบทาง webhook เดิม

## ใครต้องทำอะไรเพื่อเปิดใช้จริง

| ผู้รับผิดชอบ | งาน |
| --- | --- |
| ผู้พัฒนา Windows | Build backend/frontend/connector บน local และส่ง compiled releases เข้า Git |
| ผู้ดูแล Ubuntu | Pull Git, package/deploy backend แล้ว deploy companion container ของ OA; ไม่ compile Rust หรือ npm บน server |
| ผู้ดูแลบัญชี OA | รัน `login-oa.bat`, เข้าหน้าควบคุมด้วยบัญชีหลังบ้าน Alex แล้วล็อกอิน LINE ใน browser บน Ubuntu ครั้งแรก |
| แอดมินร้าน | ตอบใน OA Manager ตามปกติ ไม่ต้องเปิดหน้าควบคุมค้าง ไม่ต้องคัดลอกข้อความ; ตรวจสถานะการเชื่อมต่อบน Alex |

## 1. Local Windows: build และส่ง release

งานนี้สร้าง release บน local แล้ว หากแก้ source เพิ่มให้ build ใหม่:

```bat
build-backend.bat
build-frontend.bat
build-oa-connector.bat
```

ส่ง source และ compiled releases ด้วยคำสั่งของผู้ใช้:

```bat
rungit.bat "Add automatic OA Manager connector"
```

Connector ใช้ Playwright 1.64.1 และ browser image รุ่นเดียวกันตาม [Playwright Docker](https://playwright.dev/docs/docker) Runtime Dockerfile บน Ubuntu มีเพียงการเตรียม profile directory และ COPY compiled JavaScript/runtime dependencies ไม่มี npm install/build หรือ compiler

## 2. Ubuntu: เปิดบริการ

ใช้ `.env` เดิมที่ `~/alex-connectingpoint/.env` ต้องมี LINE_DESTINATION_ID, INQUIRY_ADMIN_TOKEN, ADMIN_USERNAME และ ADMIN_PASSWORD ของระบบหลังบ้านแล้ว ไม่ต้องเพิ่มรหัส LINE ลง `.env`

```sh
cd ~/alex-connectingpoint
bash ./loadgit.sh main
sudo bash ./docker-build.sh
sudo bash ./docker-deploy.sh
sudo bash ./deploy-oa-connector.sh
```

Backend อยู่ใน Swarm เดิม ส่วน browser เป็น companion Docker container บน host เดียวกัน ใช้ `--restart unless-stopped` และ volume profile แยก เพื่อให้หน้าควบคุม bind ได้เฉพาะ `127.0.0.1:18082` ไม่แก้ LINE webhook, Caddy, DNS หรือ MongoDB volume เดิม ข้อมูล session อยู่ใน volume ของ browser ไม่เข้า Git และไม่อยู่ใน compiled release

หาก frontend อยู่บน local ให้รัน `runall.bat` แบบ remote ตามเดิม ไม่จำเป็นต้อง deploy frontend บน Ubuntu ถ้าใช้งาน frontend บน server อยู่แล้ว ให้ใช้ `docker-build.sh --full-stack` และ `docker-deploy.sh --full-stack` แทนสองคำสั่ง backend-only

ค่าต่อไปนี้ใช้ default ได้ เปลี่ยนเฉพาะเมื่อจำเป็น:

```dotenv
# server .env only
OA_BACKEND_URL=http://host.docker.internal:18080
OA_CONTROL_HOST_PORT=18082
OA_POLL_SECONDS=30
OA_MEMORY=1g
```

## 3. ล็อกอิน LINE ให้ browser บน Ubuntu

บน Windows:

```bat
login-oa.bat
```

สคริปต์เปิด SSH tunnel และหน้า `http://localhost:18082` เท่านั้น:

1. เข้าหน้าควบคุมด้วย ADMIN_USERNAME/ADMIN_PASSWORD ชุดเดียวกับหลังบ้าน Alex บน Ubuntu
2. กด **เปิด LINE OA** แล้วล็อกอิน LINE ในภาพ browser บน Ubuntu หากมี QR ให้สแกนด้วย LINE ตามขั้นตอนของ LINE
3. คลิกช่องในภาพก่อนพิมพ์ผ่านช่องล็อกอินด้านบน ใช้ปุ่ม Tab/Enter หรือเลื่อนหน้าจอได้ ช่องนี้สำหรับการล็อกอิน ไม่ใช่ช่องบันทึกแชท
4. รอให้สถานะเปลี่ยนจาก NEEDS_LOGIN แล้วทดสอบข้อความจริงตามหัวข้อถัดไป
5. เมื่อเชื่อมแล้วปิดหน้าควบคุมและกด Ctrl+C ปิด login tunnel ได้ ตัวเชื่อมบน Ubuntu ยังทำงานต่อ ไม่ต้องเปิด Chrome ของเครื่องแอดมินค้างไว้

เมื่อ LINE ให้ล็อกอินใหม่ หน้า Alex จะแสดงว่าตัวเชื่อมรอล็อกอิน ให้เปิด `login-oa.bat` อีกครั้ง ไม่เก็บรหัส LINE ใน source/config ไม่ส่งข้อความไปลูกค้าซ้ำ

## 4. ทดสอบกับ OA จริงก่อนถือว่าเปิดใช้งานสำเร็จ

1. ใช้ LINE ลูกค้าส่งข้อความใหม่เข้า OA ให้ข้อความนั้นปรากฏใน inquiry ของ Alex ก่อน
2. ให้แอดมินตอบข้อความตัวอักษรจาก OA Manager ตามปกติ
3. รอรอบ poll ค่าเริ่มต้น 30 วินาที และ frontend อัปเดตภายในรอบ 10 วินาที ข้อความควรปรากฏด้านขวาพื้นฟ้าอ่อนพร้อมป้าย **จาก OA Manager**
4. ตรวจ MongoDB collection `oa_messages` ว่ามี external_message_id และ sender_role=admin โดยข้อความเดียวไม่เพิ่มซ้ำเมื่อ poll รอบถัดไป
5. ปิดหน้า login control แล้วลองตอบจาก OA อีกครั้ง เพื่อยืนยันว่าการเก็บทำงานบน Ubuntu ต่อเนื่อง

| สถานะ | สิ่งที่ต้องทำ |
| --- | --- |
| NEEDS_LOGIN | ล็อกอิน LINE ใน browser ของตัวเชื่อม |
| READY | อ่าน OA ได้ แต่ยังไม่มีข้อความแอดมินที่จับคู่และบันทึกได้ |
| SYNCING | รอบอ่าน OA ผ่าน และมีข้อความที่ backend รับแล้ว; ยังต้องตรวจข้อความจริงตามขั้นตอนทดสอบ |
| UNLINKED | ไม่พบ message ID ขาเข้าที่จับคู่ inquiry ได้; ส่งข้อความลูกค้าใหม่และตรวจ webhook ก่อน หากยังคงอยู่ให้ผู้พัฒนาตรวจ ID mapping |
| UNSUPPORTED_SCHEMA | รูปแบบ API ภายใน OA ต่างจาก adapter; เปิดข้อมูลโครงสร้างสำหรับผู้พัฒนาในหน้าควบคุม ซึ่งแสดงเฉพาะชื่อฟิลด์/ชนิดข้อมูล แล้วให้ผู้พัฒนาปรับ adapter และ build บน local ใหม่ |
| BACKEND_UNAVAILABLE | ตรวจ backend health, release และ token ฝั่ง server; backend รุ่นเก่าจะยังไม่มี connector API |
| OFFLINE | backend ไม่ได้รับ heartbeat เกิน 90 วินาที ตรวจ companion container |

ดู container ด้วย `sudo docker ps --filter name=oa-connector` และ logs ของชื่อ container ที่พบ ตัวสคริปต์ไม่ log ข้อความลูกค้า/cookie/password การ restart ใช้ `sudo docker restart <ชื่อ-container>` โดย profile volume เดิมยังอยู่

## API และข้อจำกัดของรุ่นแรก

- `POST /api/connectors/line-oa/messages` รับเฉพาะ Bearer token ของ connector ไม่รับ session cookie ของ browser; ไม่ใช่ LINE webhook
- `POST /api/connectors/line-oa/status` เก็บ heartbeat; `GET /api/line/oa-sync/status` ให้แอดมินอ่านสถานะผ่าน session ปกติ
- ใช้ incoming message ID ที่มีใน webhook และตรงร้าน/แชนแนลเป็นหลักฐานจับคู่เฉพาะแชท 1:1 ไม่เดาจากชื่อ และไม่รวมหลาย inquiry ที่กำกวม
- ข้อความแอดมินก่อน anchor ที่ยืนยันได้จะไม่ถูกโยงเข้า inquiry อัตโนมัติ ประวัติก่อนเปิด webhook อาจจับคู่ไม่ได้
- ตรวจ pagination และรูปแบบข้อมูลแบบหยุดเมื่อไม่แน่ใจ หากข้อมูลมีมากเกินหน้าที่รองรับหรือ schema เปลี่ยน จะไม่อ้างว่าเก็บครบทั้งบัญชี
- AI แยกหลักฐาน `oa_message` ออกจาก `line_event` ข้อความร้านไม่ใช่การยืนยันจากลูกค้า และข้อความใหม่ทำให้ร่างเก่าถูกตรวจว่าบริบทเปลี่ยน

## สิ่งที่ LINE เปิดให้ใช้

เอกสาร [Receive messages](https://developers.line.biz/en/docs/messaging-api/receiving-messages/) ระบุ message event เป็นข้อความจากผู้ใช้ ตาราง webhook ปกติไม่มี event สำหรับข้อความที่แอดมินส่งจาก OA Manager จึงไม่พบวิธีเปิด config ใน Messaging API เดิมให้ได้ข้อความขาออกเหล่านี้

[Module สำหรับองค์กร](https://developers.line.biz/en/docs/partner-docs/module/) ต้องยื่นขอสิทธิ์และมีเงื่อนไข LINE Marketplace เอกสารที่ตรวจไม่ยืนยันว่าให้ mirror ข้อความแอดมินจาก OA Manager จึงยังสรุปไม่ได้ว่าใช้วิธีนี้เพื่อรับข้อความแอดมินอัตโนมัติได้

พบ [ตัวอย่าง API ภายในของ OA Manager ในโค้ดสาธารณะ](https://github.com/herywinarto/LINEOFFICIAL-WEBAPIBOT/blob/main/function.py) ที่อ่านประวัติผ่าน `GET /api/v1/bots/{botId}/messages/{chatId}` บน `chat.line.biz` โดยใช้ session ที่ล็อกอินอยู่ นี่เป็นหลักฐานว่ามีแนวทางผ่านเว็บ ไม่ใช่เอกสาร API ที่ LINE รับรอง โค้ดตัวอย่างเป็นรุ่นเก่า ต้องตรวจ endpoint, response และสิทธิ์จาก OA รุ่นปัจจุบันก่อนทำตัวเชื่อมจริง

## ตัวเชื่อมอยู่ที่ไหน ต่างกันอย่างไร

| ประเด็น | Chrome บนเครื่องแอดมิน | Browser service บน Ubuntu |
| --- | --- | --- |
| ที่มาข้อมูล | หน้า OA Manager/session ที่เปิดบนเครื่องแอดมิน | หน้า OA Manager/session ของ browser บน server |
| ต้องเปิดเครื่องแอดมินไหม | ต้องเปิด Chrome และ session OA ที่ตัวเชื่อมใช้อยู่ | ไม่ต้อง เครื่องแอดมินปิดได้ |
| ต้องเปิดหน้าจอค้างให้มองเห็นไหม | ไม่จำเป็นต้องอยู่แท็บหน้าสุด แต่ browser ต้องทำงาน | ใช้ browser ที่ทำงานเบื้องหลังได้ หากตรวจแล้วรองรับ |
| การล็อกอิน | ใช้บัญชี OA ที่ล็อกอินใน browser ของแอดมิน | ต้องล็อกอิน OA ให้ตัวเชื่อมบน server แยกต่างหาก |
| เงื่อนไขที่ต้องจัดการ | ปิด browser, sleep, logout หรือเปลี่ยนโครงสร้างเว็บทำให้หยุดอ่าน | session หมดอายุ, การยืนยันตัวตนใหม่ และการเปลี่ยนเว็บของ LINE |
| สถานะในโปรเจกต์ | ไม่ได้สร้าง Chrome extension | สร้าง source/release และทดสอบข้อมูลจำลองแล้ว รอ deploy และล็อกอินทดสอบ OA จริง |

เหตุผลที่ต้องมี browser/session คือ Messaging API ไม่ได้ส่งข้อความแอดมินมาให้ ตัวเชื่อมจึงต้องมีแหล่งอ่านข้อมูลเพิ่มเติม จะซ่อน browser ไว้เบื้องหลังก็ยังต้องรักษาการล็อกอิน ไม่ใช่การเปิด webhook เพิ่มแล้วได้ข้อมูลครบ

## แนวทางให้ใช้งานต่อเนื่องโดยไม่คัดลอก

เป้าหมายคือแอดมินตอบจาก OA Manager ตามปกติ ตัวเชื่อมอ่านข้อความขาออก → backend บน Ubuntu → MongoDB → หน้า Alex แสดงด้านขวาพื้นฟ้าอ่อน โดยไม่มีการส่งข้อความถึงลูกค้าซ้ำ

ก่อนพัฒนาตัวเชื่อมต้องทดสอบกับบัญชีที่ได้รับอนุญาตจริงว่าอ่าน message ID, bot ID, คู่สนทนา, ข้อความ, ผู้ส่ง และเวลาส่งได้ครบ ห้ามระบุว่า sync สำเร็จจากการทดสอบข้อมูลสมมติเท่านั้น

ข้อกำหนดตัวเชื่อม:

- ใช้ session ที่เจ้าของบัญชีล็อกอินให้ ไม่ส่ง cookie หรือรหัส LINE เข้า frontend bundle หรือเก็บใน Git
- อ่านข้อมูลเท่านั้น ไม่กดส่งข้อความหรือเรียก API ส่งข้อความของ LINE
- จับคู่บทสนทนาด้วยรหัสที่ยืนยันได้ เช่น message ID ขาเข้าที่ตรงกับ webhook ห้ามเดาจากชื่อแสดงผล
- เก็บแหล่งที่มาและแยกบทบาทลูกค้า/แอดมิน โดยไม่ปลอมเป็น webhook ที่มีลายเซ็นจาก LINE
- ป้องกันข้อความซ้ำด้วย bot ID และ message ID; รองรับการ retry และแสดงสถานะการเชื่อมต่อจริง
- ต้องตรวจว่าครอบคลุมเฉพาะแชทที่ browser โหลด หรืออ่านแชทที่เปลี่ยนทุกห้องได้ก่อนอ้างว่าเก็บครบทั้งบัญชี
- เมื่อ session หมดอายุ ให้หยุดและแจ้งว่าต้องล็อกอินใหม่ ไม่แสดงว่าเชื่อมต่ออยู่ทั้งที่ไม่มีข้อมูลเข้า

[Chrome content scripts](https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts) อ่านรายละเอียดหน้าเว็บได้ ส่วนการส่งข้อมูลไป backend ของ Alex จาก extension ต้องแยกผ่าน service worker และขอ host permission ตาม [Chrome network requests](https://developer.chrome.com/docs/extensions/develop/concepts/network-requests) ความสามารถเหล่านี้ยืนยันกลไก extension แต่ยังไม่ยืนยันว่าอ่าน schema ของ OA รุ่นปัจจุบันได้แล้ว

หากต้องใช้เฉพาะ API ทางการโดยไม่มี browser/session เพิ่ม แนวทางที่ระบบควบคุมข้อมูลขาออกได้คือให้แอดมินตอบผ่าน Alex แล้วส่งด้วย Messaging API วิธีนี้เปลี่ยนจุดที่แอดมินพิมพ์ตอบ จึงไม่ใช่ข้อสรุปว่าอ่านข้อความที่พิมพ์ใน OA Manager ได้อัตโนมัติ

# Step 3 — ระบุ Product ID และชุดที่จะเช่า

แท็บ **ร่างล่าสุด** ค้นสินค้าจริงจาก Alex แล้วให้แอดมินตรวจรูป รหัส สี ไซซ์ และสาขาก่อนกดเพิ่มลงตะกร้า เก็บ `productItemId` ที่ API ส่งกลับพร้อมผู้เพิ่มใน MongoDB เพื่อเช็คคิวและออกบิล

เมื่อเตรียมร่างด้วย AI ระบบใช้เฉพาะรูปที่เลือก/สรุปแอดมิน ไม่ส่งแชทหรือร่างเก่าให้โมเดลค้นออเดอร์/สินค้า จากนั้น backend ใช้ขั้น login และ `GetProductItems` เดียวกับ search product อัตโนมัติ เก็บผลค้นหาไว้ใน `product_searches` และสถานะใน `catalog_lookup` ของร่าง ถ้าตรงรายการเดียวและตัวเลือกตรงจะเติมข้อมูลสินค้าจริงพร้อม evidence `alex_catalog` ราคาที่สรุปไว้แล้วไม่ถูกแทนที่ด้วยราคาแค็ตตาล็อก ข้อมูลลูกค้าจากแชทใช้ปุ่มค้นข้อมูลลูกค้าแยกต่างหาก

ค้นอัตโนมัติไม่เกิน 5 หน้าต่อชุดตัวกรองและรวมไม่เกิน 60 วินาที หากยังมีหน้าถัดไปหรือมีหลายตัวเลือกจะให้แอดมินเลือก ไม่ถือว่าตรงรายการเดียวจากผลที่ยังอ่านไม่ครบ เมื่อสี/ไซซ์ไม่พบจะค้นชื่อเดิมโดยไม่ใส่ตัวกรองเพื่อเสนอค่าจริงให้เลือก ผลค้นหาแสดงในส่วนค้นหาชุดของแท็บร่างล่าสุดโดยไม่ต้องเริ่มค้นใหม่ และต้องเพิ่มลงตะกร้าเองก่อนเช็คคิว/สร้างบิล ถ้าล็อกอินหรือค้นไม่สำเร็จยังเก็บร่างที่สกัดได้ พร้อมสถานะให้ลองค้นซ้ำ

## การตั้งค่าบน server

เพิ่มใน `.env` บน Ubuntu โดยใช้บัญชีที่ผู้ใช้อนุญาต ไม่ส่งไฟล์นี้เข้า Git:

```dotenv
ALEX_API_URL=https://shop.alexrental.app
ALEX_USERNAME=<account configured on the server>
ALEX_PASSWORD=<password configured on the server>
ALEX_SHOP_INFO_ID=<shopInfoId returned by login>
ALEX_BRANCH_IDS=<allowed branchId values, comma-separated>
ALEX_PRODUCT_URL_PREFIX=
ALEX_IMAGE_ORIGINS=
ALEX_MCP_TOKEN=<separate random secret of at least 32 characters>
```

ตรวจบัญชีที่ผู้ใช้ให้ผ่าน login จริงแล้ว ได้ร้าน `bceba5fa-7263-426c-ad6e-c304787e3536` และสาขาหลัก `0a0d8ad0-ccf4-4613-b015-8f4710dd17fb` ซึ่งต่างจาก `branchId` ในตัวอย่าง request เดิม ให้ใช้ ID ตามสิทธิ์ login ของบัญชีจริง ระบบตรวจ `shopInfoId` ให้ตรง config และจำกัดสาขาด้วย `userBranch` ร่วมกับ `ALEX_BRANCH_IDS` หากไม่กำหนดรายการสาขา จะใช้เฉพาะสาขาที่ login คืนให้

Credentials และ token อยู่ฝั่ง backend การค้นหาแต่ละครั้ง login ใหม่ ไม่เก็บ token ใน MongoDB และไม่ส่งเข้า frontend หรือ prompt ข้อผิดพลาดไม่ log response ที่อาจมี credentials

ทุก env ถูกส่งเข้า backend ใน `docker-stack.line.yml` แล้ว หลังแก้ `.env` อย่างเดียวใช้ `sudo bash ./docker-deploy.sh --full-stack`

## วิธีใช้

1. เปิดแท็บ **ร่างล่าสุด** ส่วน **ค้นหาชุดจาก Alex** (ไม่ต้องบันทึกร่างก่อนค้น) เลือกสาขาเมื่อบัญชีมีหลายสาขา
2. เลือกวิธีค้นหา:
   - **ชื่อหรือคำบรรยาย:** ใช้ keyword และตัวกรอง; เว้นคำค้นว่างเพื่อดูแค็ตตาล็อกในสาขา
   - **รหัสสินค้าแบบตรงตัว:** กรองทุก candidate ในหน้าที่ค้นด้วย code ที่ตรงกัน
   - **ลิงก์สินค้า:** ต้องตั้ง `ALEX_PRODUCT_URL_PREFIX` ให้ตรงรูปแบบลิงก์ของร้าน เช่น `https://catalog.example/products/` รองรับ code ต่อท้าย prefix หรือ query `code`/`productCode` เพียงค่าเดียว ระบบแปลงเป็นรหัสแล้วค้น Alex โดยไม่ fetch URL ที่ผู้ใช้ส่งมา
   - **รูปจากแชท/รูปแนบ:** เลือกรูปที่อยู่ใน inquiry ใช้ Hermes อ่านรหัสที่เห็นในรูป หรือเปรียบเทียบกับรูปแค็ตตาล็อกในหน้าที่กรอง
3. ระบบส่งตัวกรอง `filter`, `showNotAvailable=false`, `branchId`, `keyword`, `size`, `color`, `category`, `texture`, `simpleSearch=false`, `pageNumber`, `pageSize=15` ตาม API ที่ตรวจจริง สี/ไซซ์หลายค่าแยกด้วย `|` ได้ กด **โหลดหน้าถัดไป** เพื่ออ่านเพิ่ม ระบบใช้ `itemNumber` ประกอบ pagination
4. กด **เพิ่มลงตะกร้า** backend ค้น code อีกครั้ง ตรวจว่า Product ID ชื่อ สี ไซซ์ และราคายังตรงกับผลที่แสดง แล้วเก็บใน `cart_items` พร้อมผู้เพิ่มและ search reference เพิ่มสินค้าเดิมซ้ำจะบวกจำนวน
5. ตะกร้าผูกกับ inquiry ไม่ผูกกับเวอร์ชันร่าง จึงไม่หายเมื่อบันทึกร่างใหม่ ปรับจำนวน/เอาออกใช้ revision กันการเขียนทับกัน ผลค้นหามีอายุ 30 นาที ต้องค้นใหม่เมื่อหมดอายุ การเช็คคิวและบิลเป็นขั้นถัดไป

## รูปและราคา

การค้นจากรูปใช้ `HERMES_API_URL` ของ extraction profile ที่รองรับภาพและปิด tools เช่นเดียวกับ Step 2 ส่งรูปที่เลือกและรูปแค็ตตาล็อกเป็น data URL รวมไม่เกิน 8 MB พร้อม ID ที่มีอยู่จริง ตรวจผลโมเดลให้คืนเฉพาะ ID ของรูปที่ส่งให้เปรียบเทียบ คะแนน 0–1 ใช้เรียงตัวเลือก แอดมินเป็นผู้ยืนยันเสมอ

หากอ่านรหัสจากรูปได้ จะค้น code นั้นกับ Alex อีกครั้ง หากอ่านรหัสไม่ได้และไม่มีรูปแค็ตตาล็อกที่ดาวน์โหลดได้ จะตอบ `CATALOG_IMAGES_UNAVAILABLE` ให้ค้นด้วยข้อความหรือเพิ่มรูปสินค้าใน Alex การเปรียบเทียบครอบคลุมรายการในหน้าที่ค้น (15 รายการต่อหน้า) ใช้ตัวกรองและโหลดหน้าถัดไปเพื่อขยายการค้นหา

รูปแค็ตตาล็อกใช้ลิงก์สาธารณะของ Alex แบบเดียวกับหน้าเว็บ Alex: `mainImage` ที่เป็นชื่อไฟล์จะกลายเป็น `{ALEX_API_URL}/images/{ALEX_SHOP_INFO_ID}/item/{mainImage}` ส่วนค่าที่เป็น path หรือ URL เต็มใช้ตามนั้น backend ตรวจว่าเป็น origin ของ Alex API หรือ HTTPS origins ใน `ALEX_IMAGE_ORIGINS` และกันการออกนอกโฟลเดอร์ item ของร้าน แล้วส่งลิงก์นี้ให้ browser แสดงตรง (ไม่มีวันหมดอายุตาม search) การเทียบรูปฝั่ง server ยังดาวน์โหลดผ่าน backend ไม่ตาม redirect และส่ง Alex token เฉพาะ origin ของ Alex

ราคา `rentalPrice`/`bail` แปลงเป็นจำนวนเต็มสตางค์โดยไม่คำนวณด้วย float ตัวเลขที่มีทศนิยมเกินสองหลักหรือ schema ที่ไม่รู้จักถูกปฏิเสธ ค่า `pricing_verified` และ `availability_verified` ยังเป็น false จนกว่าจะทำขั้นถัดไป

## API และข้อมูล

| Endpoint | หน้าที่ |
| --- | --- |
| `GET /api/admin/alex` | สถานะ config และสาขาที่ login ได้ โดยไม่คืน credentials |
| `GET /api/admin/alex/categories` | ประเภทสินค้าที่ใช้งานอยู่จาก Alex `GET /api/category` เป็น `{code, name}` ให้ช่อง **ประเภท** ในส่วนค้นหาชุด |
| `GET /api/inquiries/{id}/products` | ผลจับคู่เฉพาะร่างล่าสุดและสถานะยืนยันครบ |
| `POST /api/inquiries/{id}/products/search` | ค้นด้วย mode `code`, `text`, `product_url`, `image` |
| `GET/POST /api/inquiries/{id}/cart` | อ่านตะกร้า / เพิ่ม candidate จาก search (`search_id`, `product_item_id`, `quantity`) |
| `POST /api/inquiries/{id}/cart/item` | ตั้งจำนวน (`quantity` 1–999) หรือเอาออก (`quantity: null`) พร้อม `item_id` และ `expected_revision` |
| `POST /api/inquiries/{id}/cart/availability` | เช็คคิวตะกร้าด้วย `pickup_date`/`return_date` บนหน้าจอ อ่านอย่างเดียว |
| `POST /api/admin/alex/mcp` | MCP สำหรับอ่านแค็ตตาล็อก |

Browser ใช้ HttpOnly session และ `X-Admin-Request` สำหรับ mutation ตามเดิม Search รับ `branch_id`, `mode`, `query`, ตัวกรอง, `page` และ `image_event_id` เมื่อค้นจากรูป ไม่ต้องมีร่าง (`draft_version`/`context_hash`/`item_index` เป็น optional สำหรับค้นอัตโนมัติจาก AI) `GET /products` คืนเฉพาะผลค้นอัตโนมัติ (`suggestions`) ของร่างล่าสุด

`product_searches` เก็บ candidates ที่อ่านจาก API, input, draft/context/summary revision และวันหมดอายุ `cart_items` มี unique index ตาม shop/channel/inquiry/product_item_id และใช้ revision ป้องกันผู้ดูแลเขียนทับกัน (`product_matches` เดิมเลิกใช้แล้ว) ข้อมูลทุก query จำกัดร้านและ channel จาก config

## ต่อ Hermes ผ่าน MCP

Endpoint รองรับ JSON-RPC Streamable HTTP แบบ JSON response ตาม protocol `2025-03-26`, `2025-06-18`, `2025-11-25` มี initialize, ping, tools/list, tools/call และรับ notifications ด้วย 202 ไม่มี SSE stream จึงตอบ GET เป็น 405 ตรวจ Bearer token แยกและ Origin ทุก request

มีเพียง `search_products` และ `find_product_candidates_by_image` ทั้งสองเป็น read-only และไม่มีเครื่องมือยืนยันสินค้า เช็คคิว หรือสร้างออเดอร์ การยืนยันทำผ่านหน้าจอแอดมิน

เพิ่มใน config ของ **Hermes profile สำหรับประสานงาน** บน server ส่วน extraction profile ที่ `HERMES_API_URL` ชี้อยู่ยังต้องปิด tools เพื่อใช้สกัดข้อมูลและเปรียบเทียบรูป:

```yaml
mcp_servers:
  alex_catalog:
    url: "http://127.0.0.1:18080/api/admin/alex/mcp"
    headers:
      Authorization: "Bearer REPLACE_WITH_ALEX_MCP_TOKEN"
    protocol: legacy
    skip_preflight: true
    timeout: 240
    tools:
      include: [search_products, find_product_candidates_by_image]
      resources: false
      prompts: false
```

ค่า token ต้องตรงกับ backend และอยู่ใน config ฝั่ง server ระบบกลางต้องส่ง `inquiry_id`, `draft_version`, `context_hash`, `item_index` และ `branch_id` ที่ตรวจสิทธิ์แล้วเป็นบริบทให้ Hermes เครื่องมือไม่รับ shop ID จากโมเดล

อ้างอิง [Hermes MCP config](https://hermes-agent.nousresearch.com/docs/reference/mcp-config-reference), [Hermes MCP](https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp) และ [MCP transport 2025-11-25](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2025-11-25/basic/transports.mdx)

## Build และ deploy

Windows local:

```bat
build-backend.bat
build-frontend.bat
rungit.bat "Add Step 3 Alex product matching and read-only MCP"
```

Ubuntu หลัง `ssh Hermes`:

```sh
cd ~/alex-connectingpoint
bash ./loadgit.sh main
sudo bash ./docker-build.sh --full-stack
sudo bash ./docker-deploy.sh --full-stack
```

ใช้ Linux releases ใน `dist/line-linux` และ `dist/frontend-linux` ที่ build บน local เท่านั้น ไม่มีการ compile บน server


ขั้นตอนต่อจากการยืนยัน Product ID ใช้งานได้ในแท็บ **คิว/บิล** ดู [คู่มือสรุปจากรูปและสร้างบิลใน Alex](image-rental-billing.md) รองรับรูปแนบภายในสูงสุด 5 รูปและวันที่แยกแต่ละรายการ

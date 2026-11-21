# revising-log

บันทึกการแก้ไขโค้ดทุกครั้ง — entry ใหม่อยู่บนสุด · รูปแบบและกฎดู `AGENTS.md` ข้อ 10

<!-- entries -->
## 2026-11-22 01:54 — แยกสินค้า ส่ง : flash ออกจากฐานส่วนลดและคิดค่าส่งครั้งเดียว
- **คำอธิบาย:** พบว่าการระบุสินค้าค่าส่งยังไม่รองรับชื่อ `ส่ง : flash` จึงอาจรวม 40 บาทในฐานค่าเช่า แก้ให้ตรวจชื่อที่มีช่องว่างรอบ `:` ได้ และให้ข้อมูลตะกร้าที่เคยมีป้าย false ใช้ชนิดสินค้าจากรหัส/ชื่อที่ตรวจพบ ป้องกันการเอาสินค้าค่าส่งกลับไปเป็นฐานส่วนลด ค่าเช่า Alex 600 บาท เทียบสรุป 550 บาทจึงได้ส่วนลด 50 บาท ค่าประกัน 300 บาทไม่ลด และเมื่อรวมค่าส่ง 40 บาทยอดสุทธิเป็น 890 บาท ค่าส่งอยู่ใน orderItems ครั้งเดียว โดยคงส่วนลดที่แอดมินแก้เองเป็น final
- **โฟลเดอร์ `backend/src/models/`**
  - `shop_settings.rs` — แก้: ตรวจรูปแบบชื่อสินค้าค่าส่ง `ส่ง : flash` / `ส่ง:Flash` โดยไม่นับราคาหรือชื่อเสื้อผ้าที่มีคำทั่วไปว่า Flash/ส่ง เป็นค่าส่ง
- **โฟลเดอร์ `backend/src/db/`**
  - `cart.rs` — แก้: ประเภทสินค้าค่าส่งที่ตรวจได้จาก catalog มีผลเหนือตะกร้า/snapshot เก่าที่มี is_shipping=false ส่ง shipping_detected ให้หน้าเว็บ และเก็บป้าย is_shipping ที่ถูกต้องใน snapshot
- **โฟลเดอร์ `backend/src/`**
  - `tests_shop_settings.rs` — แก้: เพิ่ม regression รูปแบบชื่อ/กรณีไม่ใช่ค่าส่ง และทดสอบตั้งแต่ตะกร้าเก่า ส่วนลดอัตโนมัติ หน้าสรุป การแก้ส่วนลด 0/25 บาท จนถึง submit ผ่าน Alex จำลอง ตรวจ rentalDiscount=50.00, bailDiscount=0.00 ค่าส่งไม่ซ้ำ และ Order ID/Order Number ที่ตอบกลับ
- **โฟลเดอร์ `frontend/src/`**
  - `lib/api.ts` — แก้: รองรับ shipping_detected ในข้อมูลตะกร้า
  - `components/order-cart.tsx` — แก้: ล็อกป้ายค่าส่งที่ระบบระบุจากรหัส/ชื่อ เพื่อไม่ให้สินค้าค่าส่งกลายเป็นฐานส่วนลด พร้อมคำอธิบาย ส่วนสินค้าอื่นยังเลือก/ยกเลิกป้ายเองได้
- **โฟลเดอร์ `docs/` และ `prompts/`**
  - `docs/image-rental-billing.md` — แก้: อธิบายการตรวจชื่อ ส่ง : flash ตะกร้าเก่า ป้ายค่าส่ง และตัวอย่างค่าเช่า 600 − 550 = ส่วนลด 50 รวมค่าส่ง/ประกันเป็น 890 บาท
  - `prompts/requirements-v0.2.md` — แก้: เติมตัวอย่างล่าสุดในรายการเดิมและใส่ Done หลังทดสอบ โดยคงการแก้รายการอื่นที่ผู้ใช้มีอยู่ก่อน
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — แก้: Linux backend จาก local `rel-20261121185118-46901227b146`
  - `frontend-linux/` — แก้: Linux frontend standalone จาก local `front-20261121185300-9a84d36d12c9`
- **ตรวจสอบ:** ยืนยัน regression ชื่อ ส่ง : flash ล้มก่อนแก้และผ่านหลังแก้; cargo fmt --check, cargo clippy -- -D warnings, cargo test -- --include-ignored ผ่าน 94 ข้อ; frontend lint/tests 69 ข้อและ build-all.bat ผ่าน; Chrome/WebKit กว้าง 1440/375/320 รวม 6 กรณี ตรวจค่าเช่าฐาน 600 ส่วนลด 50 ยอดรวม 890 ป้ายค่าส่งที่ล็อก สินค้าสองรายการ และเปลี่ยนส่วนลด 25/0/50 เป็นยอดรวม 915/940/890 โดยไม่บวกส่วนลดอัตโนมัติทับ ไม่มี overflow/page error; checksum frontend 2,101 ไฟล์และ backend ตรง manifest ลบ MongoDB container ที่สร้างเฉพาะทดสอบแล้ว ไม่เรียก Alex/LINE ของลูกค้าจริง ไม่ commit/push/deploy

## 2026-11-21 18:36 — ค้นชื่อและเบอร์โทรจากแชทก่อน Note โดยไม่ใช้ Display name
- **คำอธิบาย:** เปลี่ยนลำดับค้นข้อมูลติดต่อทั้งปุ่มค้นข้อมูลลูกค้าและขั้นตอนเตรียมร่าง AI เป็นแชท → Note แยกตามช่องชื่อและเบอร์โทร ใช้ Note เติมเฉพาะช่องที่แชทไม่มี ตัดชื่อ LINE/OA และชื่อที่แอดมินตั้งเองออกจากข้อมูลที่ส่งให้ AI และหลักฐานใหม่ หากแชทมีข้อมูลขัดแย้งให้เว้นช่องนั้นเพื่อตรวจ ไม่ดึง Note มาทับ
- **โฟลเดอร์ `backend/src/`**
  - `customer_contacts.rs` — แก้: ยกเลิกการอ่าน profile/alias และแหล่ง customer_display จากการค้นข้อมูลติดต่อ ลด schema เหลือ chat/notes บังคับลำดับบน server นำข้อมูลแชทที่ตรวจแล้วมาใช้ในร่างด้วย และตรวจความสดจาก Note โดยไม่พึ่งชื่อ Display
  - `customer_extract.rs` — แก้: ใช้ตัวอ่านแชทเดิมเมื่อไม่มี Note และใช้ตัวรวมแชท/Note เมื่อต้องเติมจาก Note
  - `draft_worker.rs` — แก้: ใช้บริบท Note เฉพาะที่มีข้อมูลและไม่เพิ่ม Display เป็นแหล่งหลักฐานในร่าง
  - `order_draft_prompt.txt` — แก้: กำหนดลำดับแชทก่อน Note และห้ามใช้ชื่อจาก profile เป็นข้อมูลติดต่อ
  - `tests_customer_contacts.rs` — แก้: ทดสอบแชทมีข้อมูลครบ ขาดชื่อ/เบอร์แยกกัน ขาดทั้งคู่ ข้อมูลขัดแย้ง ไม่มีข้อมูลแม้ Display มีชื่อ/เบอร์ การปฏิเสธหลักฐาน Display/แหล่งปลอม การลบ Note การแยกร้าน/ลูกค้า การเปลี่ยน Note ระหว่างอ่าน และการเติมร่าง AI ด้วยชื่อจากแชทพร้อมเบอร์จาก Note
- **โฟลเดอร์ `frontend/src/components/`**
  - `customer-extract-button.tsx` — แก้: อธิบายลำดับแชท → Note ปรับข้อความกรณีไม่พบข้อมูล และใช้ป้ายแหล่งอ้างอิงที่ไม่ชี้ไป Display
- **โฟลเดอร์ `docs/` และ `prompts/`**
  - `docs/customer-notes.md` — แก้: คู่มือลำดับใหม่ การไม่อ่าน Display และการใช้แหล่งข้อมูลต่อช่อง
  - `docs/step-2-admin.md` — แก้: คำอธิบายลิงก์คู่มือข้อมูลติดต่อ
  - `prompts/requirements-v0.2.md` — แก้: ปรับรายการเดิมตามคำสั่งล่าสุดและใส่ Done พร้อมวันเวลาหลังทดสอบ ไม่เพิ่มข้อกำหนดซ้ำ
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — แก้: Linux backend release จาก local `rel-20261121113522-a3f6a77e6899`
  - `frontend-linux/` — แก้: Linux frontend standalone จาก local `front-20261121113351-a601ea9db007`
- **ตรวจสอบ:** cargo fmt --check, cargo clippy -- -D warnings, ชุด backend รวม MongoDB 93 ข้อ และ regression ข้อมูลติดต่อรอบสุดท้าย 3 ข้อผ่าน; frontend lint, tests 69 ข้อ และ build ผ่าน; ตรวจ Chrome/WebKit กว้าง 1440/375/320 รวม 6 กรณี ครอบคลุม hint ใหม่ การเติมชื่อ/เบอร์จากคนละแหล่ง ลิงก์หลักฐานแชท ป้าย Note กรณีไม่พบข้อมูลและ API ผิดพลาดโดยคงค่าฟอร์มเดิม ไม่มี overflow/page error; build-all.bat และ build-backend.bat ล่าสุดผ่านบน local พร้อมตรวจ checksum frontend 2,101 ไฟล์และ backend ตรง manifest; ลบ MongoDB container ที่สร้างเฉพาะทดสอบแล้ว ไม่เรียกข้อมูลลูกค้าจริง ไม่ commit/push/deploy

## 2026-11-19 17:55 — ลบ Note และค้นข้อมูลลูกค้าจาก Note ก่อนชื่อที่แสดงและแชท
- **คำอธิบาย:** เพิ่มปุ่มลบ Note พร้อมยืนยันและตรวจเวอร์ชัน บันทึกผู้ลบ/วันเวลาแบบ soft delete ซ่อนโน้ตและรูปที่ลบจาก API และไม่ใช้จับคู่พัสดุหรือค้นข้อมูลติดต่อ ปรับเนื้อหาและช่องเขียน Note ให้เท่าข้อความแชท ใช้ชื่อและเบอร์โทรตามลำดับ Note → ชื่อที่แสดง → แชท ทั้งปุ่มค้นข้อมูลลูกค้าและ AI เตรียมออเดอร์ โดยตรวจหลักฐานจริงและปฏิเสธผลเมื่อแหล่งข้อมูลเปลี่ยนระหว่างประมวลผล
- **โฟลเดอร์ `backend/src/`**
  - `customer_contacts.rs` — เพิ่ม: โหลดเฉพาะ Note ที่ยังไม่ลบและชื่อที่แสดงของลูกค้าในร้าน/ช่อง LINE เดียวกัน แยกหลักฐานแต่ละแหล่งและบังคับลำดับต่อช่องบน server ที่อยู่ยังอ้างอิงแชท และปิดเครื่องมือในการอ่านข้อมูลติดต่อ
  - `customer_extract.rs` — แก้: อ่านแหล่งข้อมูลตามลำดับ ตรวจหลักฐานชื่อ/เบอร์โทร รวมเบอร์ที่มีขีดหรือเว้นวรรค และตรวจการเปลี่ยน Note/ชื่อที่แสดงก่อนคืนผล
  - `draft_worker.rs` — แก้: ใช้ข้อมูลติดต่อที่ตรวจแล้วในร่าง AI พร้อมเก็บแหล่งอ้างอิง และตรวจ source อีกครั้งก่อนบันทึก ไม่เปลี่ยนเส้นทางกรอกร่างด้วยมือ
  - `order_draft_prompt.txt` — แก้: ระบุช่องชื่อ/เบอร์โทรที่ server จัดหาให้ เพื่อไม่ให้ AI หลักสร้างข้อมูลหรือหลักฐานทับ
  - `main.rs` — แก้: ลงทะเบียนโมดูลข้อมูลติดต่อ
  - `tests.rs` — แก้: ลงทะเบียน regression tests ใหม่
  - `tests_customer_contacts.rs` — เพิ่ม: ทดสอบลบ Note ตาม scope/สิทธิ์/เวอร์ชัน การส่งซ้ำ รูปที่ลบ การไม่คืนโน้ตจากคำขอสร้างเก่า ลำดับชื่อและเบอร์ หลักฐานปลอม การเปลี่ยนแหล่งข้อมูลระหว่างอ่าน และการเติมร่าง AI
- **โฟลเดอร์ `backend/src/db/`**
  - `customer_notes.rs` — แก้: ลบแบบ atomic พร้อม audit ตรวจเวอร์ชัน รองรับ retry ไม่แสดงรายการ/รายละเอียด/รูปที่ลบ และใช้ cursor เดิมหลังโน้ตถูกลบได้
  - `shipping.rs` — แก้: ตัด Note ที่ลบออกจากแหล่งค้นชื่อผู้รับพัสดุ
- **โฟลเดอร์ `backend/src/models/` และ `backend/src/routes/`**
  - `models/customer_notes.rs` — เพิ่ม: payload ลบที่รับเฉพาะ expected_version
  - `routes/customer_notes.rs` — เพิ่ม: handler DELETE ที่ใช้ actor จาก session
  - `routes/mod.rs` — แก้: เปิด DELETE บนเส้นทางโน้ตเดิมภายใต้ระบบตรวจสิทธิ์และ CSRF
- **โฟลเดอร์ `frontend/src/components/`**
  - `customer-notes.tsx` — แก้: ปุ่มลบ ยืนยัน แจ้งข้อผิดพลาด โหลดเวอร์ชันใหม่เมื่อชนกัน เก็บข้อความที่ยังไม่บันทึก และกันผลรายการเก่าคืนโน้ตที่เพิ่งลบ
  - `customer-notes.css` — แก้: ขนาดเนื้อหา/ช่องเขียน/ประวัติให้ใช้ตัวแปรเดียวกับแชท และจัดปุ่มแก้ไข/ลบให้กดได้บนมือถือ
  - `customer-extract-button.tsx` — แก้: ชื่อปุ่มค้นข้อมูลลูกค้า อธิบายลำดับแหล่งข้อมูล และแสดงหลักฐาน Note/ชื่อที่แสดงแยกจากลิงก์ข้อความแชท
  - `admin-workspace.tsx` — แก้: ป้ายแหล่งข้อมูล Note และชื่อที่แสดงในหลักฐานร่าง
- **โฟลเดอร์ `frontend/src/app/`**
  - `workspace-layout.css` — แก้: ใช้ตัวแปรขนาดข้อความร่วมของ Note และแชทบน desktop
  - `mobile.css` — แก้: คงขนาดแชทบนมือถือและให้ Note ใช้ขนาดเดียวกัน
- **โฟลเดอร์ `docs/` และ `prompts/`**
  - `docs/customer-notes.md` — เพิ่ม: คู่มือการลบ audit ลำดับแหล่งข้อมูล การตรวจหลักฐานและข้อจำกัดการอ่าน
  - `docs/step-2-admin.md` — แก้: ลิงก์คู่มือ Note
  - `prompts/requirements-v0.2.md` — แก้: ใส่ Done พร้อมวันเวลาให้ข้อกำหนดสองรายการเดิมหลังพัฒนาและทดสอบ ไม่เพิ่มรายการซ้ำ
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — แก้: Linux backend release จาก local `rel-20261119104453-fc529405d8ad`
  - `frontend-linux/` — แก้: Linux frontend standalone จาก local `front-20261119104602-31855e6a35f4` พร้อม source ปัจจุบัน โดยคงงาน emoji ที่มีอยู่ก่อน
- **ตรวจสอบ:** cargo fmt --check, cargo clippy -- -D warnings, cargo test -- --include-ignored ผ่าน 93 ข้อ; frontend lint, tests 69 ข้อ และ build ผ่าน; ตรวจ Chrome/WebKit ที่กว้าง 1440/375/320 รวม 6 กรณี ครอบคลุมยกเลิกการลบ ลบผิดพลาด retry เวอร์ชันชนกัน รายการเก่าไม่คืนโน้ต ข้อความใหม่ยังอยู่ หลักฐานและการเติมฟอร์ม ขนาด Note เท่ากับแชท desktop 14px/mobile 18px และไม่มี overflow; checksum frontend 2,101 ไฟล์และ backend ตรง manifest; ลบ MongoDB container ที่สร้างเฉพาะทดสอบแล้ว ไม่ส่งข้อมูลลูกค้าจริง ไม่ commit/push/deploy

## 2026-11-19 14:02 — แผงเลือกอีโมจิครบชุดในช่องตอบแชทบน Laptop และมือถือ
- **คำอธิบาย:** เพิ่มปุ่มหน้ายิ้มข้างช่องตอบลูกค้าตามภาพ S__2113554-1.jpg เปิดแผงเลือกอีโมจิที่ค้นภาษาไทย แบ่งหมวด เลือกสีผิว และจำรายการล่าสุดได้ มี 1,906 รายการหลัก รวมรูปแบบ/สีผิวเป็น 3,931 รายการ ใช้ภาพ Twemoji ชุดเดียวกันบน Laptop/มือถือ เลือกเพื่อเติมตรงเคอร์เซอร์หรือแทนข้อความที่เลือกโดยยังไม่ส่ง ลูกค้าจะได้รับข้อความ Unicode ผ่านช่องทางส่งเดิม
- **โฟลเดอร์ `frontend/`**
  - `package.json`, `package-lock.json` — แก้: เพิ่ม emoji-picker-react แบบระบุเวอร์ชัน 5.1.4 พร้อม dependency shipstyles 1.0.0 ใช้ npm ตามโปรเจกต์ ไม่อัปเกรด dependency อื่น
- **โฟลเดอร์ `frontend/src/components/`**
  - `chat-emoji-picker.tsx` — เพิ่ม: ชุดข้อมูลและข้อความ UI ภาษาไทย รูป Twemoji 17.0.3 ผ่าน CDN แบบระบุเวอร์ชัน หมวด สีผิว และรายการล่าสุด โหลดข้อมูลจาก bundle ของแอป ระบุเครดิตภาพใน DOM ไม่โฟกัสช่องค้นหาอัตโนมัติบนจอสัมผัส
  - `reply-composer.tsx` — แก้: ปุ่มเปิดแผงแบบโหลดเมื่อใช้ผ่าน next/dynamic โดยไม่ SSR แทรกตรงเคอร์เซอร์/แทนข้อความที่เลือก เลือกหลายตัวติดกันได้ ไม่ส่งอัตโนมัติ ล้าง request ID เมื่อแก้ข้อความ ปิดด้วยปุ่ม/คลิกนอกแผง/Escape โดยคงข้อความไว้และคืน focus ให้ปุ่ม เปิดใช้ Escape ก่อน handler ภายในของ picker เพื่อให้ปิดได้ครั้งเดียว ปิดแผงเมื่อเปลี่ยนแชท/เปิดเครื่องมืออื่น คงการส่ง/retry/ข้อความด่วนเดิม
  - `reply-composer.css` — แก้: ปุ่มเป้ากด 44px แผงลอยเหนือ composer กว้างตามพื้นที่ รองรับจอ 320px/พื้นที่คีย์บอร์ดจาก visualViewport จำกัดความสูงช่องพิมพ์ขณะเปิดแผง ปรับขนาดตัวเลือกสีผิวและรักษาฟอนต์ไทยเดิม
- **โฟลเดอร์ `frontend/src/lib/`, `frontend/tests/`**
  - `emojis.ts` — เพิ่ม: แปลงรหัสอีโมจิเป็น URL ภาพที่รองรับ leading zeros, variation selectors, keycaps และ ZWJ รวมการแทรกข้อความโดยไม่เกิน maxLength 5,000 หน่วย UTF-16 ของ textarea
  - `emojis.test.mjs` — เพิ่ม: 3 tests ของเคอร์เซอร์/ช่วงข้อความที่เลือก ลำดับอีโมจิหลาย code point ขีดจำกัดข้อความ และ URL ภาพกรณีพิเศษ
- **โฟลเดอร์ `frontend/public/`**
  - `emoji-notices.txt` — เพิ่ม: เครดิตภาพ Twemoji/CC-BY-4.0 และข้อความ license รวม MIT ของ emoji-picker-react ให้บริการที่ /emoji-notices.txt
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `chat-emoji-picker.md` — เพิ่ม: วิธีใช้ ขอบเขต Unicode/Genmoji หน้าตาตามอุปกรณ์ผู้รับ การโหลดข้อมูล/ภาพ เครดิต ชุดข้อมูล และผลทดสอบ
  - `requirements-v0.3.md` — แก้: ใส่ Done พร้อมเวลาจริงใน item อีโมจิเดิมและรายละเอียดทดสอบ คง requirement อื่นที่ผู้ใช้เพิ่มไว้
- **โฟลเดอร์ `dist/frontend-linux/`**
  - `.next/`, `public/emoji-notices.txt`, `frontend-release.json`, `frontend.sha256`, `frontend.tag` — แก้: Linux frontend release สร้างด้วย build-frontend.bat บน local tag `front-20261119065638-db1ca6ed5787` รวม chunks ที่โหลดเมื่อเปิดแผงและชุดข้อมูลภาษาไทย
- **การตรวจสอบ:** frontend lint/TypeScript/tests 69/69 และ Linux production build ผ่าน; audit ฟังก์ชัน URL กับ manifest ไฟล์ Twemoji tag 17.0.3 ครบ 3,931/3,931 รายการ รวมธง keycaps/ZWJ/สีผิว; Chrome/WebKit 8 กรณีของแผงเลือกที่ 320/375/1024/1440px ผ่าน ค้นภาษาไทย/ผลว่าง หมวด ธงไทย สีผิว แทรกกลางข้อความ แทนช่วงที่เลือก เลือกหลายตัว Escape/ปุ่มปิด/คลิกนอกแผงไม่ทำข้อความหาย ไม่ส่งจากช่องค้นหา ส่งเป็น Unicode ที่ถูกต้อง ข้อความด่วนและเปลี่ยนลูกค้าไม่ปะปน ตรวจการโหลดเมื่อเปิดและจำลอง visualViewport 360px ผ่าน; ชุด composer เดิมอีก 8 กรณีผ่าน รวมขยายแปดบรรทัด พื้นที่คีย์บอร์ด ข้อความด่วน ส่ง/retry ด้วย request ID เดิม รวม 16 browser cases API จำลอง ไม่ส่ง LINE จริง ตรวจภาพ Laptop/มือถือแล้ว หลักฐาน local อยู่ใน data/emoji-picker-check/ และ data/composer-height-check/; diff check ของ source/docs ผ่าน
- **งานต่อของ Developer:** Windows ใช้ rungit.bat "Add full Thai-searchable emoji picker to chat replies"; Ubuntu ใช้ cd ~/alex-connectingpoint แล้ว bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy

## 2026-11-19 10:20 — ส่ง Tracking ให้แชทได้โดยไม่รอ Alex Order ID และย่อข้อมูลพัสดุ
- **คำอธิบาย:** ปุ่มส่งเดิมถูกล็อกเมื่อผลค้นมีเพียงแชทที่ตรงกับชื่อ แต่ไม่มีออเดอร์ CONFIRMED พร้อม Alex Order ID แก้ให้แอดมินกดเลือกลูกค้าจากชื่อ/รูปแชทแล้วบันทึกและส่ง Tracking ได้ โดยไม่สร้างหรือยืนยันบิล Alex ส่วนโหมดเลือกออเดอร์ยังตรวจบิล/สาขาตามเดิม ย่อหน้าพัสดุตามภาพโดยย้าย Tracking/รหัสไปรษณีย์เข้าโซนเลือกลูกค้า แสดง EMS ข้างชื่อ และเอาช่องค่าขนส่ง/บริการกับข้อความรอ Alex Order ID ที่ขีดฆ่าออก
- **โฟลเดอร์ `backend/src/models/`**
  - `shipping.rs` — แก้: SaveInput รับ submission_id หรือ inquiry_id เพียงหนึ่งชนิด ตรวจ ObjectId/version/Tracking/ข้อความและข้อมูลพัสดุตามเดิม เพิ่ม unit test ของปลายทางที่ว่าง ซ้ำ ไม่ถูกต้อง และข้อความไม่ตรงเลขพัสดุ
- **โฟลเดอร์ `backend/src/db/`**
  - `shipping.rs` — แก้: เพิ่มโหมดบันทึกแชทจาก inquiry ภายใต้ร้าน/ช่อง LINE ดึง LINE user จาก backend ตรวจบทสนทนา 1:1 ก่อนบันทึก เก็บ metadata ออเดอร์เป็น null เมื่อไม่ได้เลือกออเดอร์ ใช้การตรวจผู้รับร่วมกันตอนอ่าน ส่ง และลบ batch; แถวที่ผูกออเดอร์ยังตรวจ CONFIRMED/สาขา/บิลกับ inquiry เดิม คง version lock, audit, retry key, tombstone และการกันส่งซ้ำ
- **โฟลเดอร์ `backend/src/`**
  - `tests_shipping.rs` — แก้: เพิ่ม MongoDB integration test ของบิล SUBMIT_UNKNOWN ที่ไม่มี Alex Order ID การส่งเข้าแชทที่เลือก การปฏิเสธข้ามร้าน/inquiry ไม่มีจริง/ผู้รับที่ถูกเปลี่ยน การล็อกหลังเริ่มส่ง retry key เดิมและไม่ส่งเพิ่มเมื่อสำเร็จ รวมลบ batch แบบแชทโดยคงประวัติ โดย mock LINE ไม่ส่งจริง
- **โฟลเดอร์ `frontend/src/components/`**
  - `shipping-workspace.tsx` — แก้: เพิ่มปุ่มเลือกลูกค้าคนนี้จากผลค้นบิล/โปรไฟล์/โน้ต ส่ง inquiry_id เมื่อเลือกแชทหรือ submission_id เมื่อเลือกออเดอร์ ล้างการเลือกเมื่อเปลี่ยนคำค้น แสดงสถานะเลือกให้ชัด ปุ่มส่งไม่รอ Alex Order ID; ย้าย Tracking/รหัสไปรษณีย์เข้าโซนเลือกลูกค้า แสดง EMS ข้างชื่อ เอาฟอร์มชื่อผู้รับ/ค่าขนส่ง/บริการซ้ำและคำเตือนที่ขีดฆ่าออก เก็บค่าจากใบเสร็จใน payload เดิม ชื่อไม่มีจาก OCR ใช้ชื่อผู้รับที่เลือกได้ เปลี่ยนปุ่มบันทึกเป็นบันทึก Tracking ไว้ก่อน ลิงก์แท็บใหม่และรูปในกรอบผู้รับตรงกับผู้รับปัจจุบันแม้เปลี่ยนคนจากรายการที่บันทึกแล้ว
  - `shipping-workspace.css` — แก้: ป้าย EMS กรอบลูกค้าที่เลือกและปุ่มเป้ากด 44px จัด Tracking คู่รหัสไปรษณีย์ในโซนเดียวกัน จอไม่เกิน 360px เรียงลงให้เห็นเลข Tracking ครบ
- **โฟลเดอร์ `frontend/src/lib/`**
  - `shipping.ts` — แก้: Tracking รองรับ submission_id/alex_order_id เป็น null สำหรับการเลือกแชทโดยไม่ผูกบิล
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shipping-receipts.md` — แก้: วิธีเลือกลูกค้า/ส่งตรง API บันทึกแบบ inquiry_id หรือ submission_id ขอบเขตสิทธิ์และการส่งซ้ำ แก้คำอธิบายเดิมที่บังคับรอออเดอร์ทุกกรณี
  - `requirements-v0.3.md` — แก้: เพิ่มสอง item เรื่องปุ่มส่งและหน้าจอตามภาพพร้อม Done เวลาจริง ผลทดสอบและ release คง requirement เดิมของผู้ใช้
- **โฟลเดอร์ `dist/line-linux/`**
  - `backend`, `backend.sha256`, `release.json`, `release.tag` — แก้: Linux backend build บน local tag `rel-20261118184728-5b86c66d9921`
- **โฟลเดอร์ `dist/frontend-linux/`**
  - `.next/`, `frontend-release.json`, `frontend.sha256`, `frontend.tag` — แก้: Linux frontend build บน local tag `front-20261118185114-83888a79fc2d`
- **การตรวจสอบ:** backend cargo fmt --check, clippy -D warnings และ tests ผ่าน 90/90 (56 unit + 34 integration ที่เปิดใช้กับ MongoDB สำหรับทดสอบแยกบน 127.0.0.1:27018 และปิด process หลังทดสอบแล้ว); frontend lint/TypeScript/tests 66/66 และ Next.js production build ผ่าน; Chrome/WebKit รวม 40 กรณีจาก production assets ที่ 320/375/1440px ครอบคลุมบิลยังไม่ยืนยัน โปรไฟล์ โน้ต ชื่อซ้ำ การเลือกใหม่หลังเปลี่ยนคำค้น เปลี่ยนผู้รับจากรายการที่บันทึกแล้ว บันทึกล้มเหลว retry ข้อความเดิม โหมดออเดอร์เดิม และพัสดุคนที่สอง ตรวจรูปหน้าจอแล้ว ไม่มีหน้าจอล้น ไม่ส่ง LINE จริง หลักฐาน local อยู่ใน data/shipping-direct-chat-check/ และ data/shipping-message-check/; diff check ของ source/docs/requirements ผ่าน
- **งานต่อของ Developer:** local build ทั้งสองส่วนพร้อมแล้ว Windows ใช้ rungit.bat "Allow tracking messages to selected chats and simplify parcel details"; Ubuntu ใช้ cd ~/alex-connectingpoint แล้ว bash ./deploy-all.sh main ให้ deploy ทั้ง backend/frontend ตามมาตรฐาน full-stack; ยังไม่ได้ commit/push/deploy

## 2026-11-19 00:51 — เปิดแชทจากหน้าพัสดุในแท็บใหม่
- **คำอธิบาย:** ให้แอดมินเปิดแชทเพื่อตรวจลูกค้าได้โดยคงหน้าพัสดุและข้อมูลที่กรอกค้างไว้ เปลี่ยนลิงก์เปิดแชทครบทั้ง 3 จุดเป็นลิงก์แท็บใหม่ของ browser
- **โฟลเดอร์ `frontend/src/components/`**
  - `shipping-workspace.tsx` — แก้: เพิ่ม target="_blank", rel="noopener noreferrer" และ title ภาษาไทยให้ลิงก์แชทแนะนำ ผลค้นหาลูกค้า/โปรไฟล์/โน้ต และออเดอร์ที่เลือกหรือบันทึกแล้ว ใช้ inquiry_id เดิมของลูกค้ารายนั้น
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shipping-receipts.md` — แก้: อธิบายการเปิดแชทในแท็บใหม่และการคงข้อมูลบนหน้าพัสดุ
  - `requirements-v0.3.md` — แก้: ใส่ Done พร้อมเวลาจริงในข้อเปิดแชทแท็บใหม่ ระบุผลทดสอบและ release คงรายการอื่นของผู้ใช้ไว้
- **โฟลเดอร์ `dist/frontend-linux/`**
  - `.next/`, `frontend-release.json`, `frontend.sha256`, `frontend.tag` — แก้: Linux frontend release สร้างบน local ด้วย build-frontend.bat tag `front-20261118175023-246b6342376a`
- **การตรวจสอบ:** frontend lint และ Next.js production build รวม TypeScript ผ่านบน local Docker Desktop; Chrome/WebKit รวม 8 กรณีด้วย release assets และ API จำลอง ครอบคลุมแชทแนะนำ ผลค้นหาโน้ต ออเดอร์ที่เลือกใหม่ และออเดอร์ที่บันทึกแล้ว ทั้ง desktop/mobile เปิดแท็บใหม่ไป #chat/inquiry_id ถูกคน window.opener เป็น null หน้าพัสดุ URL เดิมและค่าฟอร์มที่ยังไม่บันทึกอยู่ครบ ไม่มี page error/คำขอหลุด fixture ไม่ส่ง LINE จริง หลักฐาน local อยู่ใน data/shipping-new-tab-check/; diff check ของ component/docs ผ่าน
- **งานต่อของ Developer:** Windows ใช้ rungit.bat "Open shipping chat links in new browser tabs"; Ubuntu ใช้ cd ~/alex-connectingpoint แล้ว bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy

## 2026-11-19 00:48 — ฟอนต์ไทยมีหัวและเพิ่มตัวอักษรทุกจออีก 1px
- **คำอธิบาย:** เปลี่ยน Mitr เป็น Noto Sans Thai Looped ให้รูปทรงตัวไทยมีหัวใกล้ภาพ LINE OA โดยยังไม่ยืนยันว่าเป็นฟอนต์จริงของ LINE ตั้งน้ำหนักเนื้อหาหลักเป็น regular 400 และเพิ่มค่า font-size เดิม 327 จุดอีก 1px คงส่วนเพิ่มสำหรับมือถือเดิมไว้ จึงเพิ่มขึ้นอีก 1px ทั้งมือถือและเดสก์ท็อป โหลดฟอนต์ผ่าน next/font และให้บริการจากแอป
- **โฟลเดอร์ `frontend/src/app/`**
  - `layout.tsx` — แก้: โหลด Noto Sans Thai Looped subsets thai/latin น้ำหนัก 300–700 และผูก CSS variable ใหม่
  - `globals.css` — แก้: เปลี่ยนฟอนต์หลัก/Tailwind sans เนื้อหาเป็น 17px น้ำหนัก 400 พร้อมเพิ่มขนาดข้อความส่วนกลาง
  - `mobile.css` — แก้: เพิ่มฐานขนาดตัวอักษรของ responsive อีก 1px คง mobile-text-increment เดิม
  - `typography.css` — แก้: เพิ่มข้อความรอง หัวข้อ label และ placeholder อีก 1px
  - `workspace-layout.css` — แก้: เพิ่มข้อความส่วนจัดวาง workspace อีก 1px
- **โฟลเดอร์ `frontend/src/components/`**
  - `brand-combobox.css` — แก้: เพิ่มขนาดตัวอักษรในตัวเลือกแบรนด์อีก 1px
  - `chat-linked-content.css` — แก้: เพิ่มข้อความเนื้อหาแนบในแชทอีก 1px
  - `customer-notes.css` — แก้: เพิ่มข้อความโน้ตลูกค้าอีก 1px
  - `date-input.css` — แก้: เพิ่มข้อความปฏิทินอีก 1px
  - `inquiry-list.css` — แก้: เพิ่มข้อความรายการแชทอีก 1px
  - `invoice.css` — แก้: เพิ่มข้อความใบสรุป/บิลอีก 1px
  - `order-cart.css` — แก้: เพิ่มข้อความค้นหาสินค้าและตะกร้าอีก 1px
  - `quick-replies.css` — แก้: เพิ่มข้อความตอบกลับด่วนอีก 1px
  - `rental-summary.css` — แก้: เพิ่มข้อความสรุปค่าเช่าอีก 1px
  - `reply-composer.css` — แก้: เพิ่มข้อความช่องตอบแชทอีก 1px
  - `shipping-workspace.css` — แก้: เพิ่มข้อความพัสดุ/Tracking ทั้งจอใหญ่และมือถืออีก 1px
  - `shop-dashboard.css` — แก้: เพิ่มข้อความหน้าหลักอีก 1px
  - `shop-settings.css` — แก้: เพิ่มข้อความตั้งค่าร้านอีก 1px
  - `team.css` — แก้: เพิ่มข้อความจัดการทีมอีก 1px
  - `workspace-sidebar.css` — แก้: เพิ่มข้อความเมนูด้านข้างอีก 1px
- **โฟลเดอร์ราก**
  - `AGENTS.md` — แก้: ปรับกฎฟอนต์และขนาดตัวอักษรให้ตรงคำสั่งผู้ใช้ล่าสุด
- **โฟลเดอร์ `prompts/`**
  - `requirements-v0.3.md` — แก้: ใส่ Done พร้อมเวลาจริงให้สองข้อเรื่องฟอนต์และขนาด ระบุผลพัฒนาและทดสอบ คงรายการอื่นของผู้ใช้ไว้
- **โฟลเดอร์ `dist/frontend-linux/`**
  - `.next/`, `frontend-release.json`, `frontend.sha256`, `frontend.tag` — แก้: Linux production release สร้างด้วย build-frontend.bat บน local tag `front-20261118174607-27c34d295ce8`
- **การตรวจสอบ:** frontend lint ผ่าน, tests 66/66 ผ่าน, Next.js Linux production build รวม TypeScript ผ่าน; audit CSS 327 จุดเพิ่มตรง 1px และไม่เปลี่ยนค่า inherit/ส่วนเพิ่มมือถือ; browser Chrome/WebKit รวม 42 กรณีในหน้าหลัก รายการแชท แชท ออเดอร์ พัสดุ ตั้งค่าร้าน และทีม ที่ 320/375/1440px เทียบ computed font-size ก่อน–หลัง 1,098 จุดเพิ่มตรง 1px ฟอนต์โหลดสำเร็จและไม่ล้นแนวนอน ตรวจภาพหน้าจอจริงด้วย assets จาก release และ API จำลอง; หน้าพัสดุชุดเดิมอีก 20 กรณีผ่าน ไม่ส่งข้อความจริง บันทึกหลักฐาน local ใน data/typography-check/ และ data/shipping-message-check/
- **งานต่อของ Developer:** Windows ใช้ rungit.bat เพื่อ commit/push; Ubuntu ใช้ cd ~/alex-connectingpoint แล้ว bash ./deploy-all.sh main; งานนี้ยังไม่ได้ commit/push/deploy

## 2026-11-19 00:36 — ข้อความ EMS ตามแบบและปุ่มส่งใต้ข้อความรายลูกค้า
- **คำอธิบาย:** เปลี่ยนข้อความเตรียมส่งเป็นแบบที่ผู้ใช้ให้ พร้อมแทนเลข Tracking ของพัสดุแต่ละรายการ ย้ายปุ่มส่งไว้ใต้ช่องข้อความ ให้เลือกออเดอร์แล้วบันทึก/ส่งในคลิกเดียว หรือบันทึกไว้ก่อนโดยยังไม่ส่ง ข้อความที่แอดมินแก้/เคยบันทึกไม่ถูกแทนที่เอง มีปุ่มใช้ข้อความมาตรฐานเพื่อเปลี่ยนกลับ
- **โฟลเดอร์ `frontend/src/lib/`, `frontend/src/components/`, `frontend/tests/`**
  - `shipping.ts` — แก้: trackingMessage ใช้ข้อความ "ส่งของเรียบร้อยค่ะ ✨ (ems)" บรรทัดเลขที่ Tracking บรรทัดว่าง และลิงก์ https://track.thailandpost.co.th ตามแบบ ไม่เติมเลขออเดอร์หรือข้อความอื่น
  - `shipping-workspace.tsx` — แก้: ปุ่มส่งข้อความให้ลูกค้าคนนี้อยู่ถัดจากช่องข้อความ เตรียม/บันทึกข้อมูลปัจจุบันก่อนส่งเฉพาะเมื่อจำเป็น ใช้ id/version จากผลบันทึก ถ้าบันทึกไม่ผ่านจะไม่เรียกส่ง คง retry ของข้อความที่เริ่มส่งแล้ว เพิ่มปุ่มใช้ข้อความมาตรฐานและขยายช่องเป็น 6 บรรทัด ยังคงการตรวจผู้รับ/ออเดอร์/เลข Tracking และป้องกันคลิกซ้ำขณะดำเนินการ
  - `shipping-workspace.css` — แก้: เป้ากดปุ่มใต้ข้อความอย่างน้อย 44px รองรับมือถือ
  - `shipping.test.mjs` — แก้: ตรวจข้อความตรงรูปแบบที่ขอและ Tracking ที่ normalize แล้ว
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shipping-receipts.md` — แก้: รูปแบบข้อความ วิธีส่งตรง/บันทึกไว้ก่อน/คืนข้อความมาตรฐาน และผลทดสอบ
  - `requirements-v0.3.md` — แก้: ใส่ Done และรายละเอียดใน item ข้อความ EMS เดิม คงข้อความตัวอย่างและ requirement อื่นที่ผู้ใช้กำลังแก้ ไม่เปลี่ยน Reviewed
- **โฟลเดอร์ `dist/frontend-linux/`**
  - `.next/`, `frontend.sha256`, `frontend.tag`, `frontend-release.json` — แก้: Linux frontend standalone release `front-20261118173325-2464b0b36118` build บน local Docker Desktop พร้อม runtime/assets และ checksum
- **การตรวจสอบ:** frontend lint/TypeScript/tests ผ่าน 66/66 และ npm production build ผ่านบน local Docker Browser fixture `data/shipping-message-check/` ผ่าน Chrome/WebKit 20 กรณีที่ 1440/375/320px รวมส่งตรง บันทึกไว้ก่อนแล้วส่ง ข้อความที่เคยแก้/บันทึก ปุ่มคืนข้อความมาตรฐาน ข้อความแก้ใหม่พร้อมค่าขนส่ง บันทึกล้มเหลว/ข้อความไม่มีเลขแล้วไม่ส่ง retry ไม่บันทึกทับ ผลตอบกลับขาดช่วง และพัสดุคนที่สองไม่ปะปนลูกค้าคนแรก ตรวจภาพ desktop/มือถือแล้ว ไม่มี page error/คำขอหลุด fixture ไม่ส่ง LINE จริง Backend ไม่เปลี่ยนในรอบนี้จึงไม่ build/test ซ้ำ คง release จากงานก่อน ไม่ commit/push/deploy

## 2026-11-19 00:21 — จับคู่ชื่อผู้รับจากโปรไฟล์และโน้ตเมื่อไม่พบชื่อในบิล
- **คำอธิบาย:** เพิ่มหลักฐานสำรองจากข้อความโน้ตที่แอดมินบันทึกในแชทและโน้ตโปรไฟล์ จัดลำดับต่อลูกค้าเป็นชื่อในบิล → โปรไฟล์/ชื่อแชท → โน้ต แสดงชื่อ/รูปแชทเดิมและข้อความอ้างอิงให้ตรวจ ไม่แก้ชื่อ/บิล/โน้ตหรือเลือกออเดอร์จากโน้ตให้อัตโนมัติ
- **โฟลเดอร์ `backend/src/`**
  - `db/shipping.rs` — แก้: ค้น customer_notes.text และ customer_profiles.notes ภายในร้าน/ช่อง/LINE user เดิม เพิ่มลำดับแหล่งชื่อทั้งผลออเดอร์และผลแชท เลือกบิลก่อนโปรไฟล์ก่อนโน้ตเมื่อแหล่งนั้นตรงกับชื่อที่ค้น ผลจากโน้ตใช้คะแนน 80 เพื่อให้ตรวจเอง ไม่ค้นข้อความใน history หรือชื่อไฟล์แนบ และคงสิทธิ์บิลตามสาขา
  - `models/shipping.rs` — แก้: note_excerpt จับชื่อตรงที่ปรากฏจริงในข้อความ อย่างน้อย 3 ตัวอักษรหลัง normalize คืนข้อความรอบชื่อสูงสุด 242 ตัวอักษรรวมเครื่องหมายตัดข้อความ รองรับไทย/อังกฤษ/ช่องว่างข้ามบรรทัด ไม่ fuzzy หรือเรียก AI
  - `tests_shipping.rs` — แก้: ทดสอบลำดับบิลก่อนโปรไฟล์แม้โปรไฟล์สะกดตรงกว่า โน้ตเป็นทางเลือกสุดท้าย การแสดงชื่อ/รูปแชทเดิม โน้ตปัจจุบันเท่านั้น ชื่อสะกดคล้าย/history/ชื่อไฟล์ไม่ถูกใช้ แยกร้าน/ช่อง/สิทธิ์สาขา และค้นแล้วไม่มี mutation
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`, `frontend/tests/`**
  - `shipping-workspace.tsx` — แก้: ช่องค้นและป้ายแหล่งข้อความรองรับโน้ต แสดงกรอบข้อความโน้ตให้ตรวจแม้มีออเดอร์พร้อมแล้ว พร้อมลิงก์เปิดแชทและคำแนะนำให้เลือกออเดอร์เอง
  - `shipping.ts`, `shipping.test.mjs` — แก้: เพิ่มแหล่ง customer_note/profile_note และทดสอบว่าออเดอร์ที่พบจากโน้ตไม่กลายเป็นคำแนะนำอัตโนมัติ
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shipping-receipts.md` — แก้: แหล่งโน้ต ลำดับ fallback ขีดจำกัดข้อความ/ความยาวชื่อ การตรวจหลักฐาน และผลทดสอบ
  - `requirements-v0.3.md` — แก้: ใส่ Done และรายละเอียดเฉพาะ item เพิ่มเติมเรื่องโปรไฟล์/โน้ตตามคำขอนี้ คงข้อความและงานอื่นที่ผู้ใช้กำลังแก้ในไฟล์ไว้ ไม่เพิ่ม requirement ซ้ำหรือเปลี่ยน Reviewed
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - `backend`, `backend.sha256`, `release.tag`, `release.json` — แก้: Linux backend release `rel-20261118171950-fba8ea39331e` build บน local Docker Desktop ตรวจ checksum ตรง
  - `.next/`, `frontend.sha256`, `frontend.tag`, `frontend-release.json` — แก้: Linux frontend standalone release `front-20261118171741-1bb5d22a04ad` พร้อม runtime/assets และ checksum
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 88/88; frontend lint/TypeScript/tests ผ่าน 66/66 และ Linux production build ผ่าน Browser fixture `data/shipping-notes-check/` ผ่าน Chrome/WebKit 8 กรณี และชุดบิล/โปรไฟล์เดิม `data/shipping-profile-check/` ผ่าน 12 กรณี รวม 20 กรณีที่ 1440/375/320px ครอบคลุมโน้ตแชท/โปรไฟล์ ไม่มีออเดอร์ ออเดอร์พร้อมแต่ให้ตรวจข้อความ/เลือกเอง ชื่อซ้ำและการคงข้อมูลหลังรีเฟรช ตรวจภาพ desktop/มือถือแล้ว ไม่มี page error/คำขอหลุด fixture หยุดและลบ MongoDB disposable แล้ว ไม่ส่งข้อความหรือแก้ production ไม่ commit/push/deploy

## 2026-11-18 23:49 — จับคู่ชื่อจากบิลและโปรไฟล์กับแชทแม้บิลยังไม่ยืนยัน
- **คำอธิบาย:** ผู้ใช้ให้ค้นชื่อผู้รับจาก alex_submissions.customer.name หรือชื่อโปรไฟล์ที่แอดมินแก้ แล้วแสดงชื่อ/รูปเดียวกับแชท ตรวจกรณีที่ระบุผ่าน GET จริงพบร่าง SENT มีชื่อลูกค้า แต่บิลยัง SUBMIT_UNKNOWN และไม่มี Alex Order ID/Number จึงถูกตัวกรองออเดอร์ CONFIRMED เดิมตัดออก แยกผลจับคู่แชทออกจากออเดอร์ที่พร้อมผูก Tracking โดยไม่เปลี่ยนสถานะบิลหรือชื่อแชทจริง
- **โฟลเดอร์ `backend/src/`**
  - `db/shipping.rs` — แก้: ค้น customer_profiles.display_name, ชื่อ LINE/OA/alias และชื่อจากประวัติบิลทุกสถานะ รองรับชื่อใน review_input/input/payload ของบิลเก่า เพิ่ม customer_matches/customers_more พร้อมแหล่งชื่อ/สถานะและชื่อรูปแชทจาก LINE user เดิม ตรวจร้าน/ช่อง สาขาบิล และ user/inquiry/บทสนทนา 1:1 ให้ตรง จำกัดผลและแจ้งเมื่อถึงเพดาน ส่วน orders/save/send ยังคงรับเฉพาะออเดอร์ที่ยืนยันพร้อม ID
  - `tests_shipping.rs` — แก้: เพิ่มโปรไฟล์ที่แอดมินแก้และชื่อ OA ในกรณีออเดอร์เก่านอก 1,000 รายการ เพิ่มกรณีบิลไม่ยืนยัน/ไม่มีเลข ชื่อบิล legacy โปรไฟล์ที่ยังไม่มีออเดอร์ แสดงชื่อ/รูปแชทปัจจุบัน ข้อมูลเชื่อมคนละ user/inquiry แยกช่อง/สาขา และไม่อนุญาตให้บันทึกหรือส่ง Tracking จากผลจับคู่แชทอย่างเดียว
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`, `frontend/tests/`**
  - `shipping-workspace.tsx`, `shipping-workspace.css` — แก้: ช่องค้นรวมชื่อโปรไฟล์ แสดงการ์ดแชทพร้อมรูป ชื่อที่ใช้จับคู่/แหล่งข้อมูล และปุ่มเปิดแชทเมื่อยังไม่มีออเดอร์ที่พร้อม แสดงคำอธิบายเมื่อบิล SUBMIT_UNKNOWN ไม่แสดงเป็นตัวเลือกผูกบิล และคงสถานะปุ่มบันทึกตามออเดอร์จริง รองรับมือถือ
  - `shipping.ts`, `shipping.test.mjs` — แก้: type ผลจับคู่แชท และไม่แนะนำออเดอร์รายเดียวเมื่อมีอีกลูกค้าชื่อตรงพอ ๆ กันแต่บิลยังไม่ยืนยัน คงการอ่านผลจาก backend รุ่นก่อนที่ไม่มี customer_matches ได้
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shipping-receipts.md` — แก้: แหล่งชื่อ ฟิลด์ API ความต่างระหว่างหาแชทกับผูกออเดอร์ สถานะ SENT/SUBMIT_UNKNOWN ขอบเขตสิทธิ์ ขีดจำกัด และผลทดสอบ
  - `requirements-v0.3.md` — แก้: ขยาย item จับคู่ชื่อเดิมด้วยแหล่งโปรไฟล์/ประวัติบิลและกรณีไม่มี Order ID อัปเดต Done และผลทดสอบ ไม่เพิ่ม requirement ซ้ำหรือเปลี่ยน Reviewed
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - `backend`, `backend.sha256`, `release.tag`, `release.json` — แก้: Linux backend release `rel-20261118164659-1a2ffab00a30` build บน Docker Desktop local และตรวจ checksum ตรง
  - `.next/`, `frontend.sha256`, `frontend.tag`, `frontend-release.json` — แก้: Linux frontend standalone release `front-20261118164420-1c209ae394b9` พร้อม runtime/assets และ checksum
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 86/86; frontend lint/TypeScript/tests ผ่าน 65/65 และ Linux production build ผ่าน Browser fixture `data/shipping-profile-check/` ผ่าน Chrome/WebKit 12 กรณีที่ 1440/375/320px รวมบิลยังไม่ยืนยัน ชื่อโปรไฟล์ที่ต่างจากแชท/บิล ชื่อซ้ำระหว่างลูกค้าที่ออเดอร์พร้อมกับยังไม่พร้อม ผลค้นไม่ครบ การคงข้อมูลหลังรีเฟรช และผลค้นเก่ากลับมาช้า ชุดลบ batch เดิม `data/shipping-batch-check/` ผ่านซ้ำ 16 กรณี รวม 28 กรณี ตรวจภาพ desktop/มือถือแล้ว ไม่มี page error/คำขอหลุด fixture ตรวจ production ผ่าน GET เท่านั้น ไม่แก้บิล/สถานะหรือส่งข้อความจริง หยุดและลบ MongoDB disposable แล้ว ไม่ commit/push/deploy

## 2026-11-18 23:11 — ลบ Tracking batch และจับคู่ชื่อจากแชทลูกค้า
- **คำอธิบาย:** เพิ่มลบใบเสร็จที่อัปโหลดพร้อมพัสดุทั้ง batch ออกจากการใช้งาน มีการยืนยัน เก็บผู้ลบ/เวลาและประวัติภายในเพื่อคงการป้องกันส่งซ้ำ ไม่ลบออเดอร์ Alex หรือข้อความที่ส่งแล้ว พร้อมค้น/จัดอันดับจากชื่อ LINE และชื่อที่แอดมินตั้งในแชทเพิ่มเติมจากชื่อในบิล แสดงชื่อ/รูปแชทที่แนะนำเฉพาะเมื่อลูกค้าตรงชัดเจนคนเดียว แยกชื่อบนใบเสร็จและยังให้แอดมินเลือกออเดอร์/กดส่งเอง
- **โฟลเดอร์ `backend/src/`**
  - `db/shipping.rs` — แก้: สถานะ DELETED ปิดรายการ/รายละเอียด/รูป/บันทึก/ส่ง/worker ของ batch ที่ลบ ตรวจร้าน/ช่องและทุกสาขาที่ผูก ใช้ lease ใน MongoDB ร่วมกันระหว่างลบ/บันทึก/ส่ง ส่งคำขอลบซ้ำได้ ไม่ทำให้ประวัติหรือ retry key หาย และค้นโปรไฟล์/alias ก่อนจำกัดออเดอร์เพื่อหาออเดอร์เก่าจากชื่อแชท
  - `routes/shipping.rs`, `routes/mod.rs` — แก้: เพิ่ม POST `/api/admin/shipping/receipts/{receipt}/delete` ภายใต้การยืนยันตัวตน/CSRF เดิม
  - `tests_shipping.rs` — แก้: เพิ่มทดสอบชื่อ LINE/alias ต่างจากชื่อในบิล ออเดอร์เก่าเกิน 1,000 รายการ ชื่อซ้ำ แยกร้าน/ช่อง/สาขา ลบทุกสถานะ ลบซ้ำ lease กั้น mutation URL เก่าใช้ไม่ได้ และเก็บประวัติข้อความ/ออเดอร์
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`, `frontend/tests/`**
  - `shipping-workspace.tsx`, `shipping-workspace.css` — แก้: ปุ่มลบทั้ง batch พร้อมยืนยันและข้อความสำเร็จ/ผิดพลาด กันผลโหลดเก่าคืนรายการที่ลบ เปิด batch ถัดไปหรือหน้าอัปโหลดเมื่อหมด แสดงชื่อ/รูปแชทที่แนะนำและชื่อแชทล่าสุดของรายการบันทึกโดยคงชื่อใบเสร็จแยก รองรับมือถือ
  - `shipping.ts`, `shipping.test.mjs` — แก้: แนะนำลูกค้าชัดเจนคนเดียวโดยแยกคนออกจากจำนวนออเดอร์ ไม่แนะนำเมื่อชื่อซ้ำ ผลยังมีต่อ หรือคะแนนไม่ชัดเจน
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shipping-receipts.md` — แก้: วิธีลบ batch ขอบเขตการเก็บประวัติ/สงวน Tracking และ hash ภาพที่เคยอัปโหลด การค้นชื่อแชทและขีดจำกัด พร้อมผลทดสอบ
  - `requirements-v0.3.md` — แก้: ใส่ Done ให้รายการลบ batch และย้าย/ขยาย item จับคู่ชื่อแชทเดิมไว้ในหัวข้อพัสดุ ไม่สร้าง requirement ซ้ำหรือเปลี่ยน Reviewed
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - `backend`, `backend.sha256`, `release.tag`, `release.json` — แก้: Linux backend release `rel-20261118160833-61ae17dbce7e` build บน Docker Desktop local ตรวจ checksum ตรง
  - `.next/`, `frontend.sha256`, `frontend.tag`, `frontend-release.json` — แก้: Linux frontend standalone release `front-20261118160522-d8d0ebaf309f` พร้อม runtime/assets และ checksum สำหรับ Ubuntu
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 85/85; frontend lint/TypeScript/tests ผ่าน 64/64 และ npm production build ผ่านบน local Docker Browser fixture `data/shipping-batch-check/` ผ่าน Chrome/WebKit 16 กรณีที่ 1440/375/320px รวมคงชื่อแชทหลังบันทึก/รีเฟรช ชื่อซ้ำ/หลายออเดอร์ ไม่มีผลตรง ยกเลิกลบ ลบล้มเหลวแล้วลองใหม่ batch ระหว่างอ่าน/ล้มเหลว ผล poll ค้างหลังลบ และ batch สุดท้าย ตรวจภาพ desktop/มือถือแล้ว ไม่มี page error/คำขอหลุด fixture หยุดและลบ MongoDB disposable แล้ว ไม่ส่ง LINE/สร้างออเดอร์จริง ไม่ commit/push/deploy

## 2026-11-18 17:52 — ตรวจปุ่มดาวบน release ที่ใช้งานจริงและการ์ดสินค้ามีรูป
- **คำอธิบาย:** ผู้ใช้แจ้งว่าการ์ดสินค้าตามภาพไม่มีดาว ตรวจ source/local/origin/Ubuntu ตรงกับ commit `186f521` และ frontend release `front-20261117022902-a5f67247c39d` ตรวจ HTTP จากโดเมนจริงพบ page JS และ CSS มีปุ่มดาวและ checksum ตรงกับ local release จึงไม่พบปัญหา server ส่ง release เก่า ณ เวลาตรวจ Backend จริงอ่านรายการโปรดได้ตามปกติ ภาพยังไม่ยืนยันว่าแท็บผู้ใช้ถือ assets รุ่นใด จึงแนะนำบันทึกงานก่อน reload โดยไม่สรุปว่าเป็น browser cache แน่นอน
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `product-favorites.md` — แก้: ระบุตำแหน่งดาวมุมขวาบนรูป ผลตรวจ live assets/backend วิธีโหลดหน้าใหม่โดยคงงานก่อนรีเฟรช และขอบเขตที่ยังต้องตรวจจาก browser ผู้ใช้
  - `requirements-v0.3.md` — แก้: เพิ่มผลตรวจซ้ำใน item Favorite เดิม คงสถานะพัฒนาและทดสอบ Done ไม่เพิ่ม requirement ซ้ำหรือเปลี่ยน Reviewed ของผู้ใช้
- **การตรวจสอบ:** `data/favorite-visible-check/` โหลด HTML/JS/CSS จากโดเมนจริง API/รูปสินค้าเป็น fixture เพื่อไม่แก้ข้อมูล production ทดสอบการ์ดมีรูป รหัส 0053MNANANA และมีอยู่ในตะกร้าแล้ว ผ่าน Chrome/WebKit 4 กรณี (desktop 1440px และมือถือ 375px) ตรวจ star ขนาด 44px/กดได้/elementFromPoint ไม่ถูกรูปบัง/aria-pressed หลังบันทึกและรายการโปรดปรากฏ ไม่มี page error หรือคำขอผิด fixture ตรวจภาพปุ่มแล้ว GET API รายการโปรดจริงผ่าน ไม่มี production mutation source โปรแกรมไม่เปลี่ยนจึงไม่ build release หรือรัน unit tests ซ้ำ ไม่ commit/push/deploy

## 2026-11-17 09:36 — ตรวจข้อความล่าสุดไม่ครบและแก้ประวัติแชทเกิน 200 ข้อความ
- **คำอธิบาย:** ผู้ใช้ระบุว่าข้อความล่าสุดบางแชทไม่เข้า ตรวจ server แบบอ่านอย่างเดียวพบ health ปกติ LINE webhook active และ URL ถูกต้อง ไม่มี PENDING/PROCESSING/FAILED/NEEDS_ADMIN ณ เวลาตรวจ แต่ตัวเชื่อม OA ยัง NEEDS_LOGIN ต้องเข้าใหม่ด้วยบัญชีที่มีสิทธิ์ร้าน อีกจุดที่ทำซ้ำจากโค้ดได้คือหน้าเว็บอ่าน 200 ข้อความเก่าสุดและ poll ตามจำนวนหน้าที่เปิด ทำให้ข้อความใหม่อีกหน้าไม่แสดง แก้ให้เริ่มที่ล่าสุดและ refresh ครอบคลุมช่วงประวัติที่แอดมินเปิดอยู่ โดยยังไม่สรุปว่าสาเหตุนี้ตรงกับข้อความที่หายจริงเฉพาะรายจนกว่าจะมีชื่อ/เวลาตัวอย่าง
- **โฟลเดอร์ `backend/src/`**
  - `db/oa_sync.rs` — แก้: recent_messages สำหรับอ่านย้อนจากล่าสุด รวมสามแหล่งข้อความ และใช้ event_at/event_id/_id เป็นลำดับ cursor เดียวกัน กันข้อความเวลาและ event key ตรงกันถูกข้าม ยังคง forward API เดิม
  - `routes/admin.rs` — แก้: context รองรับ latest=true/before พร้อม previous cursor ตรวจคู่ query และคืน BadRequest รูปแบบเดิม อ่านต่อจาก cursor ของข้อความที่ถูกยกเลิกได้โดยไม่แสดงข้อความนั้น
  - `db/indexes.rs` — แก้: เพิ่ม scoped indexes ตาม inquiry/time/event ID/document ID สำหรับการแบ่งหน้าทั้งสาม collection
  - `tests_chat_history.rs`, `tests.rs` — เพิ่ม/แก้: 650 ข้อความข้ามสามแหล่ง ทดสอบหน้าล่าสุด/เก่าครบไม่ซ้ำ timestamp/key ตรงกัน ข้อความนำเข้าช้า cursor ถูกยกเลิก query ผิดและการแยกร้าน/แชท
- **โฟลเดอร์ `frontend/src/`**
  - `lib/chat-history.ts`, `components/admin-workspace.tsx` — เพิ่ม/แก้: โหลดล่าสุดก่อน อ่านย้อนจนถึงขอบเขตที่เคยเปิดแทนการจำกัดจำนวนหน้า ตัดประวัติเก่าที่ผู้ใช้ยังไม่เปิด คงแถวจากการส่งพร้อม poll และลบแถวที่ถูกยกเลิก ป้องกัน poll/โหลดประวัติชนกันและผลตอบกลับจากแชทเก่า เลื่อนเริ่มที่ล่าสุดและรักษาตำแหน่งเมื่อเพิ่มประวัติด้านบน คงข้อความเดิมพร้อมแจ้งเมื่อโหลดล้มเหลว จำกัดเวลารอ context/OA และให้ OA ขัดข้องไม่หยุดอ่านแชทลูกค้า แจ้ง NEEDS_LOGIN แบบข้อความสั้นในแชทโดยไม่เพิ่มแบนเนอร์สีส้ม
- **โฟลเดอร์ `frontend/tests/`, `docs/`, `prompts/`**
  - `chat-history.test.mjs` — เพิ่ม: 7 กรณีสำหรับ pagination ข้อความใหม่หลายหน้า การไม่เปิดประวัติเก่าเพิ่มเอง cursor ถูกยกเลิก การส่งพร้อม poll และ cursor ไม่เดินหน้า
  - `chat-history-sync.md`, `requirements-v0.3.md` — เพิ่ม/แก้: ผลวิเคราะห์/การใช้งาน/API/วิธี deploy และ login OA พร้อม Done เฉพาะส่วนพัฒนาและทดสอบ โดยระบุขั้นตอนบน production ที่ยังรอผู้ใช้
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — build local สำเร็จ `rel-20261117022748-8da78b08eb3c` ตรวจ checksum ผ่าน
  - `frontend-linux/` — build local สำเร็จ `front-20261117022902-a5f67247c39d` ตรวจ checksum 2,111 ไฟล์ผ่าน
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 83/83; frontend tests 63/63, lint/TypeScript/Linux production build ผ่าน Browser fixture `data/chat-sync-check/` ผ่าน Chrome/WebKit 6 กรณีที่ 1440/375/320px รวมเปิด 200 ล่าสุด โหลดเก่าโดยรักษาตำแหน่ง ข้อความใหม่เพิ่ม 700 รายการ ข้อความ OA นำเข้าช้า ถอนข้อความ โหลดล้มเหลวยังมีข้อความเดิม OA ล้มเหลวแต่ข้อความลูกค้าเข้า เปลี่ยนลูกค้าขณะคำขอเก่าค้าง และข้อความที่ยังไม่ส่ง ไม่มี page error/คำขอหลุด fixture ตรวจภาพ desktop/มือถือแล้ว หยุดและลบ MongoDB disposable แล้ว ไม่ส่งข้อความหรือสร้างบิลจริง ไม่แก้ config server ไม่ commit/push/deploy

## 2026-11-17 02:39 — ดาวรายการโปรดสินค้าและเพิ่มลงตะกร้าจากรายการโปรด
- **คำอธิบาย:** เพิ่มดาวบนการ์ดผลค้นหาและสินค้าแนะนำจาก Alex พร้อมรายการโปรดใกล้ช่องค้นหา มีรูป ชื่อ รหัส สี ไซซ์ ราคา และตัวกรอง เก็บร่วมกันในร้านแยกช่อง LINE/สาขา คงอยู่หลังรีเฟรช/เปลี่ยนลูกค้า กดบวกเพิ่มครั้งละ 1 ชิ้นลงตะกร้าลูกค้าที่เลือก โดยตรวจสินค้าตรง ID/รหัส/สาขาและราคาปัจจุบันจาก Alex ไม่ใช้ราคาเก่าหรือ search ID ที่หมดอายุ การเอาดาวออกไม่ลบสินค้าในตะกร้า
- **โฟลเดอร์ `backend/src/`**
  - `models/product_favorites.rs`, `db/product_favorites.rs`, `routes/product_favorites.rs` — เพิ่ม: อ่าน/บันทึก/นำออก/เพิ่มตะกร้า ตรวจผลค้นหาจากฝั่งเซิร์ฟเวอร์ แยกร้าน/ช่อง/สาขา ใช้ session/CSRF/สิทธิ์ทีมเดิม รองรับ ALEX_BRANCH_IDS แบบไม่ระบุค่า และแสดงข้อผิดพลาดเมื่อสินค้าใช้ไม่ได้โดยคงรายการโปรดไว้
  - `models/mod.rs`, `db/mod.rs`, `routes/mod.rs`, `db/indexes.rs` — แก้: ลงทะเบียนโมดูล/route และ unique index `product_favorites` กันสินค้าเดียวกันในสาขาซ้ำ
  - `tests_product_favorites.rs`, `tests.rs` — เพิ่ม/แก้: ทดสอบบันทึกซ้ำ/พร้อมกัน ไม่รับราคาที่ปลอมจาก client ไม่ใช้ผลค้นหาข้ามลูกค้า การคงรายการข้ามลูกค้า แยกร้าน/ช่อง/สิทธิ์สาขา ราคาเปลี่ยน ผลค้นหาเดิมถูกลบ สินค้าหาย Alex ขัดข้อง และออเดอร์ปิด โดยไม่เพิ่มสินค้าหรือปริมาณเมื่อทำรายการไม่สำเร็จ
- **โฟลเดอร์ `frontend/src/components/`**
  - `product-favorites.tsx` — เพิ่ม: ปุ่มดาว SVG พร้อม aria-pressed/ชื่อที่อ่านออก เป้าสัมผัส 44px รายการโปรดพร้อมรูป/ตัวกรอง/บวกเพิ่ม/สถานะบันทึกและลองโหลดใหม่ เก็บข้อมูลแยกตาม key ลูกค้าและสาขา ป้องกันผลตอบกลับเก่าทับรายการใหม่
  - `order-cart.tsx`, `order-cart.css` — แก้: ผูกดาวกับการ์ดผลค้นหาและสินค้าแนะนำ เพิ่มรายการโปรดในหน้าค้นหา ใช้ตะกร้าเดิมและแจ้งผลเพิ่มสินค้า อ่านตะกร้าใหม่เมื่อคำขอล้มเหลวโดยไม่ส่งซ้ำอัตโนมัติ ล้างผลค้นหาเมื่อเปลี่ยนสาขา และจัดรายการ/ปุ่มสำหรับมือถือ
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `product-favorites.md` — เพิ่ม: วิธีใช้ ขอบเขตข้อมูล API และไฟล์ที่เกี่ยวข้อง
  - `requirements-v0.3.md` — แก้: ใส่ Done พร้อมวันเวลาและผลตรวจให้ item Favorite เดิม ไม่เพิ่มข้อซ้ำ
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — build บน local สำเร็จ `rel-20261116193633-cafdac4013a6` ตรวจ backend checksum ผ่าน
  - `frontend-linux/` — build บน local สำเร็จ `front-20261116193350-1fe16acfa50c` ตรวจ checksum ทั้ง 2,111 ไฟล์ผ่าน
- **การตรวจสอบ:** cargo fmt --check และ cargo clippy --all-targets -- -D warnings ผ่าน; cargo test -- --include-ignored ผ่าน 82/82 กับ MongoDB disposable และรัน integration ของ Favorite ซ้ำหลังเพิ่มการรองรับ branch allowlist ว่าง/ทดสอบแยกร้านผ่าน; frontend lint, tests 56/56 และ Linux production build พร้อม TypeScript ผ่าน; browser fixture `data/favorites-check/` ผ่าน Chrome/WebKit 6 กรณีที่ 1440/375/320px ครอบคลุมดาวด้วยคลิก/คีย์บอร์ด คงรายการหลัง reload แยกไซซ์/สาขา เปลี่ยนลูกค้า ตัวกรอง ราคาใหม่ และข้อผิดพลาดบันทึก/เพิ่ม/นำออก ไม่มี page error หรือคำขอหลุด fixture ตรวจภาพมือถือและ desktop แล้ว ทดสอบกับ Alex จำลอง ไม่สร้างบิล/ส่งข้อความจริง หยุดและลบ MongoDB ทดสอบแล้ว ไม่ commit/push/deploy

## 2026-11-17 02:19 — เติมสถานะ requirement รายข้อให้ตรงกับผลพัฒนาและทดสอบ
- **คำอธิบาย:** ผู้ใช้พบว่างานที่เสร็จแล้วยังไม่มีสถานะ ตรวจเทียบโค้ดกับผลทดสอบที่บันทึกไว้แล้วเติม Done พร้อมวันเวลาทีละข้อ คง Reviewed ของผู้ใช้ และแสดงรายการที่ยังไม่ทำหรือยังไม่ครบพร้อมข้อจำกัด ไม่ถือว่าการทดสอบกับ Alex จำลองแก้ปัญหาผลตอบกลับจริงที่ยังไม่ได้ตรวจแล้ว
- **โฟลเดอร์ `prompts/`**
  - `ai-briefting-requirements.md` — แก้: เติมสถานะของงานสกัดภาพ/ลูกค้า/สินค้า ฟอร์ม ตะกร้า เวอร์ชันร่าง ลิงก์ และมือถือ รวมรายการลำดับเลข คง rich text เป็นงานค้าง แยก Emoji Unicode ที่รองรับแล้วจากเครื่องมือ rich text อธิบายส่วนแถบ iOS ที่เว็บซ่อนไม่ได้ และระบุข้อเก่าที่เปลี่ยนเป็นค่าส่งแบบสินค้า/เมนูบวก/ช่องพิมพ์สูงสุด 8 บรรทัด
  - `requirements-v0.2.md` — แก้: เติม Done ให้เมนูใหม่ Enter/Shift+Enter สินค้าค่าส่งที่ไม่รวมฐานส่วนลด การใช้ราคา catalog และสรุปบิลล่าสุดที่คงข้อมูลแอดมิน แก้ข้อความซ่อนช่องค่าเช่าให้ตรงคำชี้แจงล่าสุดว่าซ่อนช่องค่าส่งแยก และให้ส่วนลดแอดมินเป็น final
  - `requirements-v0.3.md` — แก้: คืนสถานะ Done สองข้อ Tracking ที่พัฒนา/ทดสอบแล้ว คง Done กล่องพิมพ์ 8 บรรทัด เพิ่ม Not started ให้ reply/copy จากเมนูกดค้าง การจอง Google Calendar Favorite และการวิเคราะห์ผลตอบกลับ Alex จริง พร้อมบันทึกข้อกำหนดให้ปรับสถานะหลังจบทุกงาน
- **การตรวจสอบ:** ตรวจ requirement ทุกข้อภายนอก code fence ใน 3 ไฟล์ รวมรายการลำดับเลขแล้ว ไม่มีข้อขาดสถานะ (Done 89, Reviewed 1, Not started 7); ตรวจรูปแบบวันเวลาและ diff/whitespace ผ่าน ใช้ผลทดสอบ local เดิมประกอบการยืนยันสถานะ แก้เฉพาะเอกสารจึงไม่รัน test หรือ build release ซ้ำ ไม่ commit/push/deploy

## 2026-11-17 01:57 — ขยายช่องพิมพ์แชทสูงสุด 8 บรรทัดและยุบหลังแตะแชทหรือส่ง
- **คำอธิบาย:** ช่องตอบลูกค้าขยายตามข้อความที่พิมพ์ วาง หรือเลือกจากข้อความด่วน ทั้งบรรทัดใหม่และการตัดบรรทัด สูงสุด 8 บรรทัดแล้วเลื่อนในช่องได้ แตะบทสนทนา/เปิดเครื่องมือ/กด Escape/กดส่งยุบเหลือ 1 บรรทัดทันที ข้อความที่ยังไม่ส่งหรือส่งล้มเหลวยังคงอยู่ และแตะกลับเพื่อขยายแก้ไขได้
- **โฟลเดอร์ `frontend/src/components/`**
  - `reply-composer.tsx` — แก้: วัด scrollHeight หลังค่า/โฟกัสเปลี่ยน ปรับเมื่อฟอนต์โหลดหรือความกว้างเปลี่ยนผ่าน ResizeObserver และ requestAnimationFrame พร้อม cleanup เพื่อกัน WebKit observer loop ยุบก่อนส่งโดยไม่เปลี่ยนเนื้อหาหรือ retry key เดิม
  - `reply-composer.css` — แก้: อนุญาต 8 บรรทัด ปุ่มบวก/ส่งชิดล่าง เลื่อนเมื่อข้อความเกินเพดาน และลดเพดานเฉพาะมือถือที่พื้นที่ viewport เหลือน้อยเพื่อให้ปุ่มส่งอยู่ในจอ คงขนาดฟอนต์ป้องกัน iPhone focus zoom
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `workspace-navigation.md`, `requirements-v0.3.md` — แก้: อธิบายความสูง/การยุบ/การคงข้อความและแป้น Enter พร้อม Done เฉพาะ requirement กล่องข้อความ โดยคงรูปและข้ออื่นของผู้ใช้
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux frontend release — build บน local สำเร็จ `front-20261116185530-63e4bca9c91c` ตรวจ checksum 2,111 ไฟล์ผ่าน ไม่มีการแก้หรือ build backend เพิ่ม
- **การตรวจสอบ:** frontend tests ผ่าน 56 ข้อ, lint/TypeScript/production build ผ่าน; Chromium/WebKit ผ่าน 8 กรณีที่ 320/375/430/1440px ครอบคลุม 1/2/8/20 บรรทัด ข้อความยาว/Emoji/ข้อความด่วน วาง/ลบข้อความ แตะแชท/แนบ/ESC ส่งระหว่างรอ/ส่งล้มเหลวและ retry ด้วยรหัสเดิม Enter/Shift+Enter/IME เปลี่ยนความกว้างและลด viewport เหมือนเปิดคีย์บอร์ด ไม่มีข้อมูลสูญหาย overflow หรือ page error ใช้ API/LINE จำลอง ไม่ส่งข้อความจริง ไม่ commit/push/deploy

## 2026-11-17 01:41 — อัปโหลดใบเสร็จไปรษณีย์ จับคู่ออเดอร์ และตรวจส่ง Tracking
- **คำอธิบาย:** เพิ่มเมนูพัสดุและ Tracking ให้อัปโหลดภาพใบเสร็จไปรษณีย์ไทย อ่านชื่อผู้รับ เลขพัสดุ บริการ รหัสไปรษณีย์ และค่าขนส่งรายแถวด้วย Hermes เก็บรอส่ง จัดอันดับออเดอร์ที่ยืนยันแล้ว และให้แอดมินเลือกออเดอร์/ตรวจชื่อและรูป LINE ก่อนบันทึกและกดส่ง ไม่มีการจับคู่หรือส่งข้อความอัตโนมัติ และไม่เปลี่ยนยอดบิล Alex
- **โฟลเดอร์ `backend/src/models/`**
  - `shipping.rs` — เพิ่ม: schema ภาพ/ผลอ่าน/รายการพัสดุ ตรวจขนาด ตัวเลขสตางค์ ชื่อภาษาไทย และรูปแบบ Tracking โดยไม่เดา O เป็น 0; การจับคู่ชื่อเป็นข้อเสนอให้แอดมินตรวจเท่านั้น
  - `draft.rs`, `mod.rs` — แก้: ลงทะเบียน shipping และรองรับ EMS/ไปรษณีย์ไทยใน receiving-method mapping ให้ตรงกับเคส EMS ที่มีอยู่ใน workspace โดยคงค่าเริ่มต้น Flash
- **โฟลเดอร์ `backend/src/db/`, `backend/src/routes/`**
  - `shipping.rs` — เพิ่ม: คิวอ่านภาพที่อยู่รอดหลังรีเฟรช/รีสตาร์ต เก็บภาพ GridFS แบบมีสิทธิ์เข้าถึง จับคู่ออเดอร์แยกร้าน/ช่อง LINE/สาขา แสดงชื่อและรูป LINE ปัจจุบันแม้เปิดบันทึกเดิม เก็บ submission/inquiry/LINE user/draft/version/Order ID/Order Number และ snapshot ลูกค้า พร้อมค่าขนส่งและข้อความ ตรวจ version และเก็บผู้แก้/เวลา
  - `db/indexes.rs`, `db/mod.rs`, `routes/mod.rs` — แก้: ลงทะเบียนโมดูล/route และ index กันรูป/คำขออัปโหลด/แถว/เลข Tracking ซ้ำ โดยใช้ auth และ CSRF เดิม
  - `db/outbound.rs` — แก้: เปิด helper ตรวจผู้รับแชทส่วนตัวให้ shipping เรียกใช้ ส่งผ่าน outbound เดิมและ LINE retry key เดิมทุกครั้ง พร้อมติดตามผลจากประวัติ ไม่ส่งซ้ำเมื่อพบผลสำเร็จ; ล็อกแก้ผู้รับ/ข้อความหลังเริ่มส่ง และห้าม retry ผลไม่ชัดเจนเมื่อเกิน 23 ชั่วโมง
- **โฟลเดอร์ `backend/src/`**
  - `main.rs` — แก้: เริ่ม/หยุด worker อ่านใบเสร็จ และกู้สถานะที่หยุดระหว่างอ่านให้ลองใหม่ได้
  - `tests_shipping.rs`, `tests.rs` — เพิ่ม/แก้: OCR และ LINE จำลอง ตรวจ auth/ภาพส่วนตัว/แยกร้านสาขา/ชื่อซ้ำ/ไม่มีการส่งขณะอ่านหรือบันทึก/Tracking ซ้ำ/CAS/ผูกเลขออเดอร์/ผลตอบกลับหาย/ใช้ retry key เดิม/ช่วงเวลาส่งซ้ำ/รูปโปรไฟล์หลัง reload/ปฏิเสธ Hermes profile ที่เปิด tools
- **โฟลเดอร์ `frontend/src/lib/`, `frontend/src/components/`**
  - `shipping.ts`, `shipping-workspace.tsx`, `shipping-workspace.css` — เพิ่ม: เตรียมรูปใบเสร็จแนวยาว อัปโหลดแบบติดตามคำขอเดิม คิว/ประวัติใบเสร็จ การ์ดแต่ละพัสดุพร้อมชื่อและรูป LINE ค้น/เลือกออเดอร์ ดูภาพต้นฉบับ แก้ข้อมูลและข้อความก่อนส่ง บันทึกค่าขนส่งว่างตามที่แอดมินกรอก แสดงสถานะส่ง/ยังไม่ยืนยัน ติดตามคำขอเดิมและอ่านผลหลัง reload รองรับมือถือโดยไม่ zoom ช่องพิมพ์
  - `workspace-navigation.ts`, `api.ts`, `workspace-navigation.tsx`, `workspace-sidebar.tsx`, `admin-portal.tsx`, `shop-dashboard.tsx` — แก้: เส้นทาง #shipping ปุ่มพัสดุในหน้าหลัก/แถบซ้าย/บาร์มือถือ และ EMS mapping
- **โฟลเดอร์ `frontend/src/app/`, `frontend/tests/`**
  - `layout.tsx`, `mobile.css` — แก้: โหลด styles ใหม่และปรับบาร์มือถือเป็น 4 เมนู
  - `shipping.test.mjs`, `workspace-navigation.test.mjs` — เพิ่ม/แก้: การคงตัวเลข Tracking ค่าส่งเป็นสตางค์ ข้อความพร้อมส่ง และเส้นทางเมนู
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shipping-receipts.md`, `requirements-v0.3.md` — เพิ่ม/แก้: คู่มือ workflow/ข้อมูล/ข้อจำกัด/ทดสอบ/deploy และ Done เฉพาะ 2 ข้อ Tracking โดยคงข้อใหม่อื่นและภาพของผู้ใช้ไว้
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux releases — build-all บน local สำเร็จ: backend `rel-20261116183551-7bea5ebec313`, frontend `front-20261116183720-c49c2a10ac10`; ตรวจ checksum backend และ frontend 2,111 ไฟล์ผ่าน
- **การตรวจสอบ:** cargo fmt --check, clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 81 ข้อด้วย MongoDB disposable; frontend test ผ่าน 56 ข้อและ lint/TypeScript/production build ผ่าน; Chrome/WebKit ผ่าน 18 กรณี Tracking ที่ 320/375/1440px รวมอัปโหลดขาดช่วง/ไม่มีชื่อที่ตรง/ชื่อซ้ำ/ค่าขนส่งว่าง/แก้ข้อความ/ส่งล้มเหลว/ผลตอบกลับหาย/reload และไม่มี overflow หรือส่งอัตโนมัติ; regression ผลส่งบิลเดิมผ่าน 24 กรณี ไม่ส่งข้อความจริงหรือวัด OCR ด้วย Hermes ของร้านจริง ไม่ commit/push/deploy และหยุด MongoDB ทดสอบแล้ว
- **งานค้างที่แยกไว้:** งาน Rich Text/Flex ที่เริ่มก่อนหน้านี้ยังไม่เสร็จ เก็บ patch และไฟล์เดิมไว้ที่ `data/wip-rich-text-20261117/` ก่อนเริ่มงานนี้โดยไม่ลบงานผู้ใช้หรือรวม dependency/โค้ดที่ยังไม่พร้อมเข้า release Tracking

## 2026-11-17 00:33 — ตรวจยืนยันสองข้อการส่ง Alex และปรับสถานะ requirement
- **คำอธิบาย:** ตรวจว่ากลไกแจ้งขั้นตอน/ผลส่งและการคืน/เก็บ Order ID/Order Number ผูกลูกค้าและร่าง มีอยู่แล้วใน commit `6b73386` ซึ่งตรงกับ Git remote `main` จึงไม่สร้าง implementation ซ้ำ ปรับสถานะเอกสารในหัวข้อ Data transmission to Alex ที่ยังไม่ได้ระบุ Done
- **โฟลเดอร์ `prompts/`**
  - `requirements-v0.2.md` — แก้: เพิ่ม Done ให้สองข้อใน Data transmission to Alex โดยคงข้ออ้างอิงการผูกเลขออเดอร์เดิมและข้อความของผู้ใช้
- **การตรวจสอบ:** frontend tests ผ่าน 53 ข้อ; ทดสอบ Linux frontend release เดิม `front-20261115202138-297178d4ea01` ด้วย Chrome/WebKit ผ่าน 24 กรณี ครอบคลุมเลขครบ/เลขขาด/ส่งล้มเหลว/timeout/ขาดการเชื่อมต่อ/reload/replay/manual check โดย Alex จำลอง ไม่ได้ส่งบิลจริง; backend release ที่เตรียมไว้คือ `rel-20261115202033-80ea7674980a` รอบนี้แก้เฉพาะเอกสาร ไม่ build/commit/push/deploy ใหม่
- **ข้อจำกัดการตรวจ server:** SSH เข้าได้ แต่บัญชีไม่มีสิทธิ์ Docker socket และ `sudo -n docker service ls` ต้องใช้รหัสผ่าน จึงยังยืนยัน image ที่กำลังรันบน Ubuntu ไม่ได้ ไม่เปลี่ยน service หรือ config ใดบน server

## 2026-11-16 03:23 — คืนเลขออเดอร์ Alex และคงการผูกกับลูกค้าและร่างที่ส่ง
- **คำอธิบาย:** ยืนยันว่าผล submit คืน Order ID/Order Number จากข้อมูลที่บันทึกจริง และใช้คำขอเดิมอ่านเลขเดิม เพิ่มฟิลด์เลขอ้างอิงที่มีค่า null ชัดเจนเมื่อยังไม่ได้รับเลข รวมถึงประวัติรุ่นเก่า และแจ้งกรณี Alex สร้างออเดอร์พร้อม Order ID แต่ยังไม่คืน Order Number โดยไม่ส่งซ้ำหรือเดาเลข
- **โฟลเดอร์ `backend/src/`**
  - `db/checkout.rs` — แก้: เก็บฟิลด์เลขทั้งสองตั้งแต่สร้างบันทึกการส่ง และคืนฟิลด์ทั้งสองเสมอเมื่ออ่านผล พร้อมทดสอบ response รุ่นเก่า/เลขที่ขาด
  - `tests_rental.rs` — แก้: ตรวจทั้งสองเลขในผลส่งและฐานข้อมูล พร้อม inquiry/draft/version/LINE user/conversation/customer; ตรวจ replay ได้ record เดิม และห้ามใช้ review ID กับลูกค้าคนอื่นโดยไม่สร้างออเดอร์เพิ่ม
- **โฟลเดอร์ `frontend/src/lib/`, `frontend/tests/`**
  - `bill-feedback.ts` — แก้: แจ้งว่าบันทึกเลขครบและผูกกับลูกค้า/ร่างแล้ว หรือแจ้งว่ายังไม่ได้รับ Order Number เมื่อมี Order ID ยืนยันการสร้าง ไม่แนะนำให้ส่งซ้ำ
  - `bill-feedback.test.mjs` — แก้: ทดสอบส่งต่อเลขทั้งสองและรหัสติดตาม รวมกรณี Order Number เป็น null
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `image-rental-billing.md`, `requirements-v0.2.md` — แก้: อธิบายฟิลด์ผลส่ง การผูกลูกค้า/ร่าง การติดตามคำขอเดิมและเลขที่ขาด อัปเดต requirement เป็น Done และอ้างถึงข้อเดียวกันแทนข้อความซ้ำ
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux releases — build ด้วย `build-all.bat` บน local: backend `rel-20261115202033-80ea7674980a`, frontend `front-20261115202138-297178d4ea01`; ตรวจ checksum backend และ frontend 2,111 ไฟล์ผ่าน
- **การตรวจสอบ:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test -- --include-ignored` ผ่าน 79 ข้อด้วย MongoDB disposable; frontend test ผ่าน 53 ข้อ, lint/TypeScript/production build ผ่าน; Chrome/WebKit ผ่าน 24 กรณีรวมเลขครบ/เลขขาด/timeout/reload/replay/ตรวจผลเอง ไม่มี duplicate order หรือ API mutation นอก fixture ทดสอบกับ Alex จำลองเท่านั้น ไม่ได้ส่งบิลจริง ไม่ commit/push/deploy และหยุด MongoDB ทดสอบแล้ว

## 2026-11-16 03:09 — แจ้งขั้นตอนและผลส่งออเดอร์ พร้อมติดตามคำขอเดิมเมื่อขาดการเชื่อมต่อ
- **คำอธิบาย:** แก้การส่งแล้วเงียบ/modal ปิดก่อนรู้ผล/ข้อผิดพลาดอยู่นอกจอ แสดงความคืบหน้าและแจ้งสำเร็จ ไม่สำเร็จ หรือยังยืนยันไม่ได้ใกล้ปุ่มส่ง คงฟอร์มเมื่อส่งล้มเหลวหรือต้องตรวจผล รับว่าสำเร็จเฉพาะ CONFIRMED ที่มี Order ID
- **โฟลเดอร์ `frontend/src/lib/`, `frontend/tests/`**
  - `bill-feedback.ts`, `bill-feedback.test.mjs` — เพิ่ม: แยกผลตามสถานะและเลขออเดอร์ เก็บเฉพาะรหัสคำขอ/inquiry/เวอร์ชัน/การยอมรับคิวใน sessionStorage เพื่อกลับมาติดตามหลังรีเฟรช ไม่เก็บข้อมูลลูกค้าหรือ token และลบเมื่อได้ผลยืนยัน
- **โฟลเดอร์ `frontend/src/components/`**
  - `rental-checkout.tsx` — แก้: แสดงบันทึก 1/3 ตรวจราคา/คิว 2/3 และส่ง 3/3; จำกัดเวลารอ write pipeline 90 วินาที/ตรวจผล 15 วินาที; ตรวจประวัติ review ID เดิมเมื่อ POST ขาดช่วง ไม่สร้างรหัสใหม่หรือส่งซ้ำอัตโนมัติ; รักษาค่าในฟอร์มและล็อกแก้ระหว่างยังไม่รู้ผล; แยกการลองใหม่หลัง NOT_CREATED ออกจากการติดตามคำขอเดิม; คง modal เมื่อผลไม่ชัดเจน และลบรหัสติดตามหลังบันทึกผลตรวจด้วยตนเอง
  - `bill-feedback.tsx` — เพิ่ม: แผงแจ้งผลที่มี live region/alert แสดงเหตุผล รหัสติดตาม และเลขออเดอร์ พร้อม spinner และข้อความเมื่อรอนานกว่า 20 วินาที
  - `draft-editor.tsx`, `admin-workspace.tsx` — แก้: ส่งผลยืนยันจาก modal ไปหน้า workspace เปิดสรุปบิลล่าสุดและแสดง toast สำเร็จ 10 วินาที ข้อความสำเร็จยังอยู่ในบิลหลัง toast ปิด ไม่แสดง toast สถานะคลุมเครือค้างหลังตรวจผลด้วยตนเอง
  - `latest-bill-view.tsx` — แก้: แสดงสถานะถาวรด้วยกลไกเดียวกัน รวมคำขอกำลังส่ง/ไม่สำเร็จ/ยังไม่ยืนยัน ไม่ขึ้นข้อความสำเร็จเพียงเพราะ HTTP ตอบกลับ และไม่ซ้ำกล่องเลขออเดอร์สีเขียวที่อาจขัดกับสถานะไม่ยืนยัน
  - `invoice.css` — แก้: แผงผลส่งใกล้ปุ่ม สี/ปุ่ม/ข้อความอ้างอิงยาว รองรับ reduced motion และ toast; กำหนดความสูง modal แน่นอนเพื่อไม่ยุบและทำให้แถบหัวบังปุ่มลองใหม่บน WebKit
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `image-rental-billing.md`, `requirements-v0.2.md` — แก้: อธิบายสถานะ การติดตามและ retry แบบรหัสเดิม การรอผล/timeout และเพิ่ม requirement เป็น Done รอผู้ใช้ตรวจ
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone — แก้: build-frontend.bat บน local ผ่าน release `front-20261115200307-491fb414731a`; ตรวจ SHA256 ครบ 2,111 ไฟล์ผ่าน
- **ตรวจสอบ:** npm test ผ่าน 52/52; lint, TypeScript และ production build ผ่าน; browser กับ Linux release ผ่าน 22 กรณีบน Chrome/WebKit ที่ 320/375/1440px ครอบคลุมสำเร็จ, บันทึก/ตรวจบิลล้มเหลว, HTTP422 ที่ยังยืนยันไม่ได้, NOT_CREATED แล้วใช้ review ใหม่, คำตอบหลุดหลังรับคำขอ, ไม่รับคำขอแล้วรีเฟรช/ติดตามด้วย ID เดิม, SUBMITTING, timeout 90 วินาที, review หมดอายุ และ manual resolve ไม่ค้างข้อความผิดสถานะ; ยืนยันจำนวนการสร้างไม่ซ้ำและค่าที่กรอกยังอยู่ ไม่มี page error/unknown mutation/horizontal overflow ตรวจภาพ WebKit แผงผิดพลาดและยังไม่ยืนยันแล้ว ใช้ Alex จำลอง ไม่มีการสร้างออเดอร์จริง ยังไม่ได้ commit/push/deploy

## 2026-11-16 01:56 — คงส่วนลดและ remark ของแอดมินเป็น final พร้อมอ่านสลิปโอนเงิน
- **คำอธิบาย:** แก้กฎยอดจากสรุปบิลที่ล้างส่วนลดที่บันทึกในร่างจน 100 กลับเป็น 140 ให้ส่วนลดที่ระบุไว้รวม 0 มีลำดับสูงกว่ายอดสรุปเดิม และ modal แก้ต่อได้ ไม่บวกโปรโมชั่นซ้ำ คงหมายเหตุที่แอดมินบันทึกจากร่างไป modal/ข้อมูลส่ง เพิ่มการอ่านสลิปจากรูปที่แอดมินเลือกและใช้เกณฑ์ผลต่างยอดสุทธิไม่เกิน 100 บาทตามคำชี้แจงผู้ใช้
- **โฟลเดอร์ `backend/src/`**
  - `db/checkout.rs` — แก้: ใช้ส่วนลดที่ระบุในร่างก่อนส่วนต่างราคา/กฎร้าน ใส่ discounts_finalized ใน group/review/ผลส่ง ปิดโปรโมชั่นส่วนลดและเปลี่ยน pricing policy hash เพื่อไม่ส่ง review เก่า; prefill เฉพาะหมายเหตุที่มีหลักฐาน admin_edit ของร่างปัจจุบัน โดยค่าที่แก้หรือล้างใน modal ยังชนะ
  - `models/payment_slip.rs` — เพิ่ม: หลักฐานสลิป ธนาคาร สกุลเงิน ยอดจำนวนเต็มสตางค์ เลขอ้างอิง โอนสำเร็จ และ quote; ตรวจผูกภาพที่เลือกและจับคู่ THB กับค่าเช่าสุทธิรวมประกันสุทธิที่คลาดไม่เกิน 10,000 สตางค์ ไม่รวมสลิปหลายใบ
  - `models/draft.rs`, `models/checkout.rs`, `models/mod.rs` — แก้: เพิ่ม bank_transfer_slips, การเลือกวิธีชำระเอง และ source_ref ที่ใช้จับคู่ พร้อม serde defaults สำหรับข้อมูลเก่า
  - `draft_worker.rs`, `order_draft_prompt.txt` — แก้: อ่านเฉพาะภาพที่เลือก ไม่ใช้ยอดคงเหลือ/QR ขอจ่าย/ข้อความอ้างว่าโอน ไม่อนุญาตข้อมูลผู้โอนหรือผู้รับจากสลิปเป็นชื่อลูกค้า; คงหลักฐานขณะปรับร่างเดิม ล้างเมื่อ clone
  - `db/checkout.rs` — เพิ่ม: ตรวจสลิปกับยอดและวิธีชำระอีกครั้งก่อนบันทึก/เตรียม payload เก็บหลักฐานลงฟอร์ม/review/submission/สรุปบิล และกันนำสลิปที่เคยใช้เติมการชำระในบิลเวอร์ชันก่อนมาใช้ซ้ำอัตโนมัติ
  - `tests_rental.rs`, `tests_shop_settings.rs`, `tests_billing_guidance.rs` — แก้: ทดสอบ OCR จำลอง ขอบเขตหลักฐาน ผลต่างราคา ส่วนลด final 100/0 ภายใต้กฎสรุปบิลจนถึง payload, ปิดโปรโมชั่นซ้ำ, วิธีชำระทั้งสองช่องและ snapshot สลิป และ prefill/ล้าง remark ของแอดมิน
- **โฟลเดอร์ `frontend/src/`, `frontend/tests/`**
  - `components/draft-editor.tsx` — แก้: ติดตามส่วนลดที่ระบุไว้หรือที่แอดมินพิมพ์โดยตรง ไม่เดาจากการเท่ากับค่าอัตโนมัติเดิม ไม่เปลี่ยนเมื่ออัปเดตตะกร้า และบังคับบันทึกเจตนาการกรอก 0 แม้เท่าค่าเริ่มต้นเดิม
  - `lib/payment-slip.ts`, `tests/payment-slip.test.mjs` — เพิ่ม: จับคู่สลิปกับยอดสุดท้าย เลือกใบที่ใกล้สุดโดยไม่รวมหลายใบ ทดสอบขอบเขต ±100 บาท/นอกขอบ/ผิดสกุล/ไม่สำเร็จ/ไม่อ่านเลขอ้างอิง/แก้ส่วนลด
  - `components/rental-checkout.tsx` — แก้: เติมโอนเงินทั้งค่าเช่าและประกันจากสลิป คำนวณใหม่ตามส่วนลด หยุดการเติมอัตโนมัติเมื่อไม่เข้าเกณฑ์ เคารพค่าที่แอดมินเลือกเองและมีปุ่มกลับไปใช้สลิป คงหลักฐานที่โหลดไว้เมื่อแชทเปลี่ยน และล็อกโปรโมชั่นส่วนลดเมื่อร่างมีส่วนลด final
  - `components/payment-slip-notice.tsx`, `components/latest-bill-view.tsx`, `components/summary-images.tsx`, `components/invoice.css` — เพิ่ม/แก้: แสดงยอดสลิป/ยอดสุทธิ/ผลต่าง/ธนาคาร/อ้างอิงในใบสรุปและบิลจริง อธิบายว่าอ่านจากภาพยังไม่ตรวจยืนยันกับธนาคาร รองรับอ้างอิงยาวบนจอเล็ก และอธิบายการแนบสลิปให้ AI
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `image-rental-billing.md`, `shop-billing-instructions.md`, `requirements-v0.2.md` — แก้: ลำดับความสำคัญของส่วนลด 640−100=540, remark จากแอดมิน, ขั้นตอนสลิป และสถานะ Done ของสลิป/บั๊กส่วนลด/remark รอผู้ใช้ตรวจ คงรูปและ requirement อื่นของผู้ใช้
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux release — แก้: build บน local ผ่าน backend `rel-20261115185132-c03001328a4a`, frontend `front-20261115184631-c775efac5be4`; ตรวจ SHA256 backend และ frontend 2,111 ไฟล์ผ่าน
- **ตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 78/78 (Windows linker ล็อก exe ชั่วคราว รันซ้ำผ่าน); npm test 49/49, lint และ production build ผ่าน; browser กับ Linux release ผ่าน 14 ชุดบน Chrome/WebKit ที่ 1440/375/320px: 8 ชุดส่วนลด 100/0, การเพิ่มลดจำนวนในตะกร้า, กรอก 0 เท่าค่าอัตโนมัติแล้วต้องบันทึก, modal/ข้อมูลส่ง/สรุปบิลและ remark; 6 ชุดสลิป ±100 เทียบ 100.01, วิธีชำระที่เลือกเอง, restore, save/send failure/retry review ID เดิม, แชท stale ระหว่างเปิด modal และสรุปบิลหลังส่ง; ไม่มี page error/unknown mutation/horizontal overflow ตรวจภาพ WebKit 640−100=540 +300=840 แล้ว ใช้ Hermes/Alex จำลองและ MongoDB disposable ที่หยุด/ลบแล้ว ไม่ส่งออเดอร์จริง ยังไม่ได้ commit/push/deploy

## 2026-11-16 01:05 — เก็บข้อมูลที่แก้ใน modal และแสดงสรุปบิลล่าสุดก่อนและหลังส่ง
- **คำอธิบาย:** เปลี่ยนแท็บคิว/บิลเป็นสรุปบิลล่าสุด แสดงค่าที่แอดมินแก้จาก modal แทนฟอร์มที่เริ่มต้นใหม่ เก็บวิธีชำระเงิน ลูกค้า หมายเหตุ โปรโมชั่น และส่วนลดตามร้าน/ลูกค้า/ร่าง รอการบันทึกก่อนปิดหรือส่ง หลังส่งเปิดสรุปจาก snapshot ที่ส่งจริงพร้อม Order ID/Order Number แม้ต้นฉบับแชทเปลี่ยนภายหลัง
- **โฟลเดอร์ `backend/src/`**
  - `models/checkout.rs`, `routes/checkout.rs`, `routes/mod.rs` — เพิ่ม: API บันทึกฟอร์มสรุปพร้อม expected revision ผ่าน session/CSRF และ scope เดิม
  - `db/checkout.rs` — เพิ่ม: checkout_forms พร้อม CAS, ตรวจต้นฉบับ/สาขา/สิทธิ์/ส่วนลด เก็บผู้แก้และเวลา อ่าน saved_form/latest_bill แยกจากข้อมูลต้นฉบับ และคืน bill_summary ของการส่งจริงโดยไม่เปิดเผย payload ดิบ
  - `db/indexes.rs` — เพิ่ม: unique index ของฟอร์มแต่ละร้าน/ลูกค้า/ร่าง/กลุ่มบิล และ index อ่านรายการล่าสุด
  - `tests_rental.rs` — แก้: ทดสอบบันทึก/โหลดฟอร์ม วิธีชำระและส่วนลด 0, CAS/retry, auth/context/closed guards, snapshot ที่ส่งจริงและอ้างอิงบิลหลังแชทเปลี่ยน
- **โฟลเดอร์ `frontend/src/components/`**
  - `rental-checkout.tsx` — แก้: autosave และ flush ก่อนปิด/ส่ง คืนฟอร์มจากข้อมูลที่บันทึก คงค่าขณะส่งล้มเหลว ล็อกบริบทต้นฉบับและคงข้อมูลผลส่งก่อน refresh ใช้ review ID เดิมเมื่อ retry
  - `latest-bill-view.tsx` — เพิ่ม: สรุปบิลล่าสุดแบบอ่านอย่างเดียว รวมลูกค้า รายการสินค้า วันรับ–คืน วิธีชำระ หมายเหตุ ส่วนลด ยอดรวม เลขออเดอร์ และผู้บันทึก/เวลา
  - `draft-editor.tsx`, `admin-workspace.tsx` — แก้: ส่งจาก modal ในร่างล่าสุด รอการบันทึกก่อนปิดด้วยปุ่มหรือ Escape และเปิดแท็บสรุปบิลล่าสุดหลังได้ผลส่ง
  - `invoice.css` — แก้: รูปแบบข้อมูลบิลอ่านอย่างเดียว รองรับข้อความยาวและจอเล็ก
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `image-rental-billing.md`, `requirements-v0.2.md` — แก้: ขั้นตอน modal/autosave/สรุปบิลล่าสุด และอัปเดต 4 requirements เป็น Done รอผู้ใช้ตรวจ โดยคงรูปและเนื้อหาอื่นของผู้ใช้
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux release — แก้: build-all.bat บน local ผ่าน backend `rel-20261115175851-ea435a157c2b`, frontend `front-20261115175954-fcc0731cee70`; ตรวจ SHA256 backend และ frontend 2,111 ไฟล์ผ่าน
- **ตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 75/75; npm test ผ่าน 45/45, lint/production build ผ่าน; browser กับ Linux release ผ่าน Chrome/WebKit 6 ชุดที่ 1440/375/320px ครอบคลุมแก้-ปิด-เปิด modal, refresh, บันทึกล้มเหลวแล้วคงข้อมูล, ส่งล้มเหลวแล้ว retry review ID เดิม, สรุปยอด/วิธีชำระ/หมายเหตุหลังส่งพร้อม Order ID/Number และแชท stale ไม่มี horizontal overflow หรือ page error ตรวจภาพ WebKit แล้ว หยุดและลบ MongoDB disposable แล้ว ใช้ Alex จำลอง ไม่สร้างบิลจริง ยังไม่ได้ commit/push/deploy

## 2026-11-15 21:50 — ใช้ส่วนลดที่แอดมินแก้เป็น final และเอาช่องค่าส่งแยกออก
- **คำอธิบาย:** ผู้ใช้ชี้แจงว่าต้องซ่อน field ค่าส่ง ไม่ใช่ค่าเช่า และส่วนลดค่าเช่า/ค่าประกันที่แก้ใน confirm modal ต้องเป็นยอดสุดท้าย ใช้แทนค่าคำนวณเดิม รวมถึง 0 โดยไม่บวกส่วนลดเก่าหรือโปรโมชั่นส่วนลดอัตโนมัติเพิ่ม ค่าส่งคิดจากสินค้าที่เลือกในออเดอร์เดียวกันเท่านั้น คงการเก็บ Order ID/Number ผูกลูกค้าและร่างที่ส่งเดิม
- **โฟลเดอร์ `backend/src/`**
  - `db/checkout.rs`, `models/checkout.rs` — แก้: checkout groups ใช้ quoted shipping=0 และไม่รับยอดจาก shipping_fee_satang รุ่นเก่ามาบวก/เขียนหมายเหตุแยก เมื่อมี explicit rental/deposit discount ถือว่า final บังคับ useDiscountPromotion=false เก็บ discounts_finalized กับ review/ผลส่ง และเพิ่ม pricing policy ใน source hash เพื่อให้ review เก่าต้องตรวจใหม่ ส่วนลดส่งตามค่าที่แก้โดยตรง ไม่ใช้ส่วนต่างราคาหรือส่วนลดเดิมบวกซ้อน
  - `tests_rental.rs` — แก้: ทดสอบการเปลี่ยนส่วนลดที่เดิมคำนวณไว้เป็น 0 และจำนวนใหม่ ตรวจยอดสุทธิ/flag final ใน review และประวัติ รวมถึง payload ที่ส่งเข้า Alex ต้องปิดโปรโมชั่นส่วนลดแม้ client ส่ง true และไม่เพิ่มค่าค้างจากช่องค่าส่งเดิม คง tests การจับคู่ Order ID/Number กับลูกค้า/ร่างและป้องกันส่งซ้ำ
- **โฟลเดอร์ `frontend/src/components/`**
  - `draft-editor.tsx` — แก้: เอาช่องค่าจัดส่งแยกออก ปรับหัวข้อเป็นส่วนลดของออเดอร์
  - `rental-checkout.tsx` — แก้: คืนราคาเช่ารายสินค้าให้ตรวจได้ ซ่อนยอด/คำอธิบายค่าส่งแยกนอกตะกร้า ส่ง legacy shipping=0 ใช้ส่วนลดที่แก้เป็น final พร้อมข้อความอธิบาย ปิด/ล้าง checkbox โปรโมชั่นส่วนลดเมื่อแก้ตัวเลข และส่ง promo=false ไม่ให้ส่วนลดอัตโนมัติซ้อนกับยอดที่แก้
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shop-billing-instructions.md`, `requirements-v0.2.md` — แก้: อธิบาย final discount/ช่องค่าส่งที่เลิกใช้และการคิดผ่านสินค้า ปรับ requirement ตามคำชี้แจงล่าสุดเป็น Done รอผู้ใช้ตรวจ
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux backend/frontend release — แก้: build-all.bat บน local ผ่าน backend `rel-20261115144557-2f6b7638fed3`, frontend `front-20261115144729-01b52d0e801b`; ตรวจ SHA256 binary และ frontend 2,111 ไฟล์ผ่าน
- **ตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 75/75 (Windows linker ล็อก executable ชั่วคราว รันซ้ำผ่าน); npm test 45/45, lint และ production build ผ่าน; browser ตรวจ Linux release ผ่าน 28 กรณี Chrome/WebKit (6 กรณีค่าส่ง/Order references/การเลือกโปรแล้วแก้ส่วนลดต้องปิดโปรและไม่มีช่องค่าส่งในร่าง/ใบสรุป + 6 กรณีแก้/reset/รูปแบบส่วนลด/retry + one-click เดิม 16 กรณี) หยุด/ลบ MongoDB disposable แล้ว ทดสอบด้วย Alex จำลอง ไม่สร้างบิลจริง ยังไม่ได้ commit/push/deploy

## 2026-11-15 21:34 — ผูกเลขออเดอร์ Alex กับลูกค้า/ร่าง และแยกค่าส่งจากฐานส่วนลด
- **คำอธิบาย:** เก็บ Order ID กับ Order Number แยกกันจาก response ของคำขอที่ส่งจริง ผูกกับร้าน/ลูกค้า/บทสนทนา/ร่างโดย ID โดยตรง ค่าส่งเป็นสินค้าในออเดอร์เดียวกันแต่ไม่เป็นฐานส่วนลด ซ่อนข้อมูลค่าเช่ารายสินค้า/ราคาที่แจ้งในใบสรุป คงช่องส่วนลดและยอดสุทธิ เพิ่มตาม requirement ที่ผู้ใช้เพิ่มในไฟล์ระหว่างงาน: เมื่อไม่ระบุค่าเช่าหรือค่าประกัน ใช้ราคา Alex แล้วให้แอดมินใส่ส่วนลดเอง แม้เปิดกฎยอดจากสรุปบิล
- **โฟลเดอร์ `backend/src/`**
  - `config.rs`, `alex.rs`, `tests_products.rs` — แก้: รองรับ optional ALEX_ORDER_NUMBER_POINTER แยกจาก path ของ Order ID และปรับ fixture config
  - `db/checkout.rs`, `models/checkout.rs` — แก้: แยกอ่าน Order ID/Number จาก object เดียวที่ระบุชัดหรือ custom JSON pointers ตรวจ HTTP/logical failure ไม่เดาจาก customer/product ID เก็บเลขทั้งคู่ พร้อม customer snapshot, line_user_id, conversation_key, inquiry_id, draft_id/version ที่ส่ง เพิ่มช่องเลขทั้งคู่ในการตรวจผลด้วยมือ คง queue/source/role/idempotency เดิม; คิดส่วนลดเฉพาะราคาสินค้าเช่า รวมสินค้าค่าส่งเต็มราคาและไม่บวก quoted shipping ซ้ำ ถ้าราคาที่แจ้งขาดใช้แค็ตตาล็อกแยกแต่ละยอด เคารพส่วนลดที่แอดมินใส่กับค่าเริ่มต้น แต่ราคาที่ระบุในสรุปยังมีลำดับก่อน
  - `models/cart.rs`, `models/shop_settings.rs`, `db/cart.rs`, `db/shop_settings.rs` — แก้: เพิ่ม is_shipping จากรหัสในกฎร้าน/ชื่อสินค้า หรือป้ายที่แอดมินเลือก เก็บใน cart snapshot และบิล การแก้ป้ายใช้ revision/branch/shop guard เดิม คงค่าส่งที่แอดมินเพิ่มเองแม้ policy มีรหัสเดียวกัน ค่าส่งอัตโนมัติของกฎคำนวณจากค่าเช่าสุทธิที่ไม่รวมค่าส่ง ไม่อนุมานจากราคาอย่างเดียว และโหมด manual ไม่เพิ่ม fee เอง
  - `db/drafts.rs`, `db/indexes.rs` — แก้: ส่งเลขทั้งคู่กับ submission ID ในประวัติแต่ละร่าง เพิ่ม scoped indexes สำหรับ Order ID/Number แบบไม่บังคับ unique เพื่อไม่กระทบข้อมูลเก่า
  - `tests_rental.rs`, `tests_shop_settings.rs`, `db/checkout.rs` — แก้/เพิ่ม: ทดสอบการเก็บและผูกเลขสองแบบต่อเวอร์ชัน/ลูกค้า, ปฏิเสธ response ล้มเหลวหรือเลขกำกวม, ไม่จับเลขข้าม object, manual resolution, ค่าส่งไม่เป็นส่วนลด/ไม่คิดซ้ำ, marker ใน snapshot, ค่าเช่า 550 + ค่าส่ง 40 = 590, ลดเฉพาะค่าเช่า 50 เหลือ 540 และ fallback ราคาเมื่อไม่ระบุทั้งสองยอด/ระบุเพียงยอดเดียวพร้อมส่วนลดเอง
- **โฟลเดอร์ `frontend/src/`, `frontend/tests/`**
  - `lib/api.ts`, `lib/cart-amounts.ts`, `components/order-cart.tsx`, `components/draft-editor.tsx` — แก้/เพิ่ม: ป้ายรายการค่าส่งที่เปลี่ยนได้ในตะกร้า คำนวณส่วนลดอัตโนมัติโดยไม่รวมค่าส่ง รักษาป้ายเมื่อคง/โคลนร่าง แสดง Order ID/Number แยกกัน เพิ่ม unit test ฐานส่วนลดรวมจำนวนและสินค้าค่าส่งหลายชิ้น
  - `components/rental-checkout.tsx`, `components/admin-workspace.tsx`, `components/invoice.css` — แก้: แสดงเลขทั้งคู่ในบิลและประวัติร่าง ซ่อนค่าเช่ารายสินค้า/ราคาที่แจ้ง คงยอดสุทธิและส่วนลด แสดงค่าส่งที่รวมในสินค้า ป้องกันเลขกรอกค้างข้ามบิลเมื่อตรวจผลด้วยมือ แก้ state ส่วนลดให้คงค่าล่าสุดของอีกช่อง และ remount ช่องเงินเมื่อ reset เพื่อไม่ให้ WebKit เก็บค่า input เดิมมาทับยอดใหม่ ป้องกัน reference ยาวล้นจอ
- **ไฟล์ root และโฟลเดอร์ `docs/`, `prompts/`**
  - `.env.example`, `docker-stack.line.yml` — แก้: ส่ง ALEX_ORDER_NUMBER_POINTER ให้ backend บน server โดยไม่มี secret ใน frontend
  - `image-rental-billing.md`, `shop-billing-instructions.md`, `requirements-v0.2.md` — แก้: อธิบาย identity mapping, รูปแบบ response ที่รองรับ/custom pointers, marker ค่าส่ง, catalog fallback และการซ่อนรายละเอียดค่าเช่า พร้อมสถานะ Done ของทั้ง 4 ข้อรอผู้ใช้ตรวจ
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux backend/frontend release — แก้: build บน local เท่านั้น backend `rel-20261115143237-e5225ad79b77`, frontend `front-20261115142215-b191d5ef6741`; ตรวจ SHA256 binary และ frontend 2,111 ไฟล์ผ่าน
- **ตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 75/75 (Windows linker ล็อกไฟล์ทดสอบชั่วคราว รันซ้ำผ่าน); frontend unit tests 45/45, lint และ production build ผ่าน; browser กับ Linux frontend release ล่าสุดผ่าน 28 กรณี Chrome/WebKit (6 จอ 1440/375/320px สำหรับ marker/ค่าส่ง/เลขสองแบบ/สลับบิลตรวจด้วยมือ + 6 กรณีแก้ส่วนลด/reset/format/retry + one-click เดิม 16 กรณี) ไม่มี page errors/horizontal overflow; Swarm YAML interpolation และ runtime backend Docker image build ผ่าน หยุด/ลบ test Mongo และ image ทดสอบแล้ว
- **ขอบเขตยืนยัน:** ใช้ Alex จำลอง ยังไม่ได้ยืนยัน schema response ของร้านจริงหรือส่งบิลจริง ถ้าชื่อฟิลด์ต่างจากที่รองรับให้ตั้ง JSON pointers ตาม response จริง คงข้อมูลออเดอร์เก่า ไม่ backfill หรือเดาเลขที่ยังไม่ทราบ ยังไม่ได้ commit/push/deploy

## 2026-11-15 20:26 — ไม่เติมข้อความจากร่าง AI ลง remark อัตโนมัติ
- **คำอธิบาย:** ผู้ใช้ไม่ต้องการข้อความขนส่ง/ข้อสงสัยวันคืน/คำแนะนำให้เลือกสินค้าค่าขนส่ง และเวลา 8:19 เพิ่มใน remark แก้ต้นทางที่ API ใบสรุปไม่ส่ง fields.notes จากร่างมาเป็นค่าเริ่มต้น ช่องหมายเหตุเริ่มว่าง แอดมินยังกรอกเองได้ ร่างเดิมและประวัติออเดอร์ไม่ถูกลบ ไม่ใช้การตัดคำที่อาจลบหมายเหตุจริงของแอดมิน และคงหมายเหตุยอดค่าจัดส่งจริงตามพฤติกรรมเดิม
- **โฟลเดอร์ `backend/src/`**
  - `db/checkout.rs` — แก้: checkout list คืน notes=null ให้ frontend เดิมเริ่มช่อง remark ว่าง เพิ่ม remark_policy ใน source hash ให้ review ที่เตรียมก่อนเปลี่ยนพฤติกรรมต้องตรวจใหม่ ไม่เปลี่ยนกฎราคา/ส่วนลด/ค่าขนส่ง/การส่งบิล
  - `tests_rental.rs` — แก้: ใส่ตัวอย่างข้อความที่ผู้ใช้ไม่ต้องการไว้ในร่าง ทดสอบว่า API ไม่เติมค่าเริ่มต้น และ review/payload remark1 ว่างได้เมื่อไม่กรอกและไม่มีค่าจัดส่ง พร้อมคงทดสอบว่าหมายเหตุอุปกรณ์ที่แอดมินกรอกเองส่งเข้า Alex ได้
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shop-billing-instructions.md`, `requirements-v0.2.md` — แก้: อธิบายหมายเหตุเริ่มว่างและขอบเขตการกรอกเอง บันทึก requirement เป็น Done รอผู้ใช้ตรวจ
- **โฟลเดอร์ `dist/line-linux/`**
  - `backend`, `backend.sha256`, `release.tag`, `release.json` — แก้: build-backend.bat บน local สำเร็จ release `rel-20261115132448-d9899e7496c2` ตรวจ SHA256 ผ่าน ใช้ frontend release เดิม `front-20261115124957-72f0b245c081` ที่รองรับ notes=null อยู่แล้ว
- **ตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 73/73 (ครั้งแรก Windows linker ถูกล็อกชั่วคราว รันซ้ำผ่าน); browser ตรวจ frontend Linux release เดิมกับ API contract ใหม่ผ่าน 4 กรณี Chrome/WebKit desktop 1440px และมือถือ 375px ทั้งไม่กรอก remark และกรอกเอง ไม่มี page errors; หยุด/ลบ MongoDB disposable แล้ว ไม่สร้างออเดอร์จริง ยังไม่ได้ commit/push/deploy

## 2026-11-15 19:51 — แก้ส่วนลดในใบสรุปและส่งยอดที่แก้จริงเข้า Alex
- **คำอธิบาย:** เพิ่มช่องส่วนลดค่าเช่า/ส่วนลดค่าประกันในหน้าตรวจสรุปก่อนส่ง แอดมินแก้ยอดเป็นบาทได้ทันที มีปุ่มกลับไปใช้ส่วนลดจากร่าง ค่าเริ่มต้นตามสรุปบิลของร้านยังคงอยู่ แต่การแก้โดยตรงใช้กับบิลครั้งนี้ ไม่ต้องเตรียม AI ใหม่และยังส่งคลิกเดียว การแก้ในใบสรุปไม่เขียนกลับเป็นร่างใหม่
- **โฟลเดอร์ `backend/src/`**
  - `models/checkout.rs`, `routes/checkout.rs`, `routes/mod.rs` — แก้: รับส่วนลดที่แอดมินแก้แบบ optional เพื่อรองรับ client เดิม เพิ่ม POST checkout/amounts สำหรับตรวจยอดโดยไม่เขียนบิล และรับ amounts_hash ตอนตรวจร่าง
  - `db/checkout.rs` — แก้: ตรวจส่วนลดตั้งแต่ศูนย์ถึงยอดแค็ตตาล็อก คำนวณค่าเช่า/ประกันใหม่ คำนวณเกณฑ์ค่าขนส่งและสร้าง/ถอดสินค้า fee จากยอดสุทธิที่แก้ ไม่ลดหรือเพิ่ม fee ซ้ำ ล้างหมายเหตุ fee เดิมเมื่อไม่เข้าเกณฑ์ ตรวจ hash ให้ตรงยอดที่แสดงก่อนสร้าง review และใช้ payload ส่วนลดใหม่ตอนส่ง เก็บ review_input ในประวัติพร้อมผู้ส่ง/เวลาส่ง แต่ไม่ส่งข้อมูล audit ภายในกลับ API รายการบิล คง shop/branch scope, catalog/queue/source checks และการกันส่งซ้ำเดิม
  - `tests_rental.rs`, `tests_shop_settings.rs` — แก้: ขยาย integration tests ตรวจ preview ไม่สร้างบิล, auth, ส่วนลดติดลบ/เกินราคา, source/hash ไม่ตรง, ทศนิยมหน่วยสตางค์ใน payload ที่ส่งจริง, การกดซ้ำ, overrides แม้เปิดกฎยอดจากสรุปบิล และค่าขนส่งข้ามเกณฑ์ทั้งสองทิศทาง
- **โฟลเดอร์ `frontend/src/components/`**
  - `rental-checkout.tsx`, `invoice.css` — แก้: ช่องเงินมี label และขนาดเหมาะกับมือถือ คำนวณยอดสุทธิและตรวจเงื่อนไขผ่าน server ก่อนเปิดปุ่มส่ง แสดงสถานะ/ข้อผิดพลาด/ปุ่มลองคำนวณใหม่ ไม่ค้างเมื่อเติม .0 ให้เงินค่าเดิม ยกเลิก review เดิมเมื่อแก้ยอดหรือ source เปลี่ยน ไม่รับผล preview เก่าทับยอดใหม่ คงปุ่มส่งครั้งเดียวและการใช้ review ID เดิมเมื่อ retry
- **โฟลเดอร์ `docs/`, `prompts/`**
  - `shop-billing-instructions.md`, `requirements-v0.2.md` — แก้: อธิบายวิธีแก้ส่วนลด ขอบเขตยอดเฉพาะบิล และ API preview/hash พร้อมบันทึก requirement เป็น Done รอผู้ใช้ตรวจ
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux backend/frontend release — แก้: build-all.bat สำเร็จบน Docker Desktop local; backend `rel-20261115124824-a2cae799312a`, frontend `front-20261115124957-72f0b245c081`; ตรวจ SHA256 ของ binary และ frontend ทั้ง 2,111 ไฟล์ผ่าน
- **ตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test -- --include-ignored ผ่าน 73/73 ด้วย MongoDB disposable ซึ่งหยุดและลบทิ้งแล้ว; npm test ผ่าน 44/44, npm run lint และ production build ผ่าน; browser tests กับ Linux release จริงผ่าน 22 กรณี (แก้ส่วนลด 6 กรณี Chrome/WebKit จอ 1440/375/320px พร้อมลองใหม่/กลับค่าเดิม/แก้หลังตรวจคิว และ one-click checkout เดิม 16 กรณีรวม retry/queue race/staff) ไม่มี horizontal overflow ใช้ Alex จำลอง ไม่สร้างออเดอร์จริง ยังไม่ได้ commit/push/deploy

## 2026-11-15 19:25 — ตรวจแจ้งเตือนเดิมและเตรียมแก้เส้นทาง Service Worker ใน Caddy
- **คำอธิบาย:** ผู้ใช้ถามว่าการแจ้งเตือนข้อความใหม่จำเป็นต้องใช้ Firebase หรือไม่ ตรวจพบระบบปัจจุบัน polling จากหน้าเว็บทุก 10 วินาที ใช้ Notification API/showNotification แต่ยังไม่มี Push subscription, push event handler, backend Web Push หรือ PWA manifest จึงต้องเปิดหน้า Home/แชทค้างไว้ เสนอ Web Push มาตรฐานและอธิบายเงื่อนไข Home Screen ของ iPhone โดยยังไม่ได้เพิ่มระบบ Push
- **ผลตรวจโดเมน/server:** HTTPS /api/health ตอบ 200 แต่ /alex-notifications-sw.js ตอบ 404 จาก Hermes/aiohttp ขณะที่ไฟล์เดียวกันบน frontend 127.0.0.1:18081 ตอบ 200 application/javascript; matcher @alex ใน /etc/caddy/Caddyfile ไม่มี path ของ worker จึงส่งไป fallback ผิดบริการ
- **โฟลเดอร์ `docker/`, `docs/`, `prompts/`**
  - `Caddyfile.line.example` — แก้: เพิ่ม /alex-notifications-sw.js ให้ส่งไป frontend โดยคงเส้นทางอื่น
  - `browser-notifications.md` — เพิ่ม: วิธีเปิดแจ้งเตือนเดิม เงื่อนไขที่ไม่แจ้งแชทที่กำลังอ่าน ปัญหา path 404 วิธีตรวจ/แก้ Caddy ความต่างของ polling กับ Web Push และทางเลือก Firebase พร้อมเอกสาร MDN/WebKit/Firebase
  - `requirements-v0.2.md` — เพิ่ม: สถานะ Done ของงานตรวจและเตรียม config ระบุชัดว่ายังไม่ได้เพิ่ม PWA/background Web Push
- **ไฟล์ชั่วคราวบน Ubuntu:** เตรียม /tmp/alex-caddy-notifications-ksepyblk/Caddyfile, Caddyfile.before และ apply.sh ที่สิทธิ์จำกัดเฉพาะ ubuntu/root; helper ตรวจ baseline ป้องกันทับ config ที่เปลี่ยนไป validate ก่อนติดตั้ง สำรองไฟล์เดิม reload และตรวจ worker ผ่าน HTTPS/JavaScript หากล้มเหลวคืนไฟล์เดิม ไม่มีการรัน helper หรือเปลี่ยน active Caddyfile เพราะ sudo -n ยังต้องใช้รหัสผ่าน
- **การตรวจสอบ:** ตรวจ HTTP จริงทั้งภายนอกและ loopback, อ่าน matcher ที่ใช้อยู่, caddy adapt candidate ผ่านและ bash -n helper ผ่าน แก้เฉพาะ config ตัวอย่าง/เอกสาร ไม่ได้ build โปรแกรม commit/push หรือ deploy; ไม่ได้ยืนยันการแจ้งเตือนบนเครื่องผู้ใช้หลังแก้ เพราะยังต้องให้ผู้ดูแลรัน sudo เพื่อติดตั้ง config

## 2026-11-15 17:46 — เงื่อนไขออกบิลภาษาทั่วไปพร้อมตรวจขอบเขตและแนะนำสินค้าให้กดเพิ่มเอง
- **คำอธิบาย:** ผ่อนการรับเงื่อนไขจากไวยากรณ์ตายตัวเป็นภาษาทั่วไปในขอบเขตบิลร้านนี้ รองรับข้อความตามภาพทั้งคำนำหน้า K. ยึดยอดค่าเช่า/ค่าประกันที่แอดมินสรุป และแนะนำสินค้าค่าขนส่งโดยแอดมินกดเพิ่มเอง ข้อความทั่วไปต้องผ่านการตรวจขอบเขตก่อนบันทึก ไม่เพิ่มช่องทางรันโค้ด เขียนฐานข้อมูลโดยตรง เข้าถึงร้านอื่น หรือเพิ่มสิทธิ์จากข้อความ
- **โฟลเดอร์ `backend/src/models/`, `backend/src/db/`**
  - `shop_settings.rs` — แก้: คง typed rules ทางการเงินเดิมและเพิ่ม guidance, catalog_searches, manual_shipping รองรับประโยคสรุปยอดจากภาพ ตรวจอักขระแฝงและรูปแบบโค้ดเป็นชั้นเสริม คำค้นแนะนำไม่เกิน 3 รายการ ต้องอยู่ในข้อความจริงและเป็นชื่อ/รหัสที่อนุญาต ไม่รับ URL/คำสั่งเป็น query เก็บ policy_schema=2 และตรวจข้อมูลที่โหลดกับข้อความต้นฉบับ ไม่เรียก AI ซ้ำทุกครั้งที่อ่านค่า คง shop/channel scope, expected_version และ audit เดิม
  - `db/shop_settings.rs` — แก้: ตรวจคำสั่งทั่วไปด้วย Hermes แบบ tools disabled ก่อน write ค่า ใช้ namespace ใหม่ผูก shop/channel และรับผลเฉพาะ schema ที่กำหนด ถ้าผลไม่ผ่าน/ผิดรูปแบบ/คำค้นแต่งขึ้น/เครื่องมือเปิดอยู่จะไม่บันทึก โหมดค่าขนส่งให้แอดมินเลือกจะไม่เพิ่มสินค้าอัตโนมัติและไม่นำค่าส่งจากร่างไปคิดซ้ำในหมายเหตุ
  - `models/shop_settings.rs` — เพิ่ม: จำกัด evidence จากเงื่อนไขทั่วไปไว้ที่หมายเหตุและข้อเสนอส่วนลด/ค่าขนส่ง ปฏิเสธการใช้เงื่อนไขเป็นหลักฐานแต่งชื่อ เบอร์ ที่อยู่ รหัสสินค้า วันที่ หรือค่าเช่า/ค่าประกันฐาน
- **โฟลเดอร์ `backend/src/`, `backend/src/routes/`**
  - `billing_policy_prompt.txt` — เพิ่ม: prompt ตรวจ semantic scope ของภาษาธุรกิจ ไม่ยึดตัวอย่างประโยคตายตัว แยกคำสั่งรันสคริปต์/SQL การข้ามร้าน เขียน DB เปลี่ยนสิทธิ์ และนอกงานบิล รวมถึงรองรับคำสั่งปฏิเสธเช่นห้ามดูร้านอื่น การตรวจภาษาไม่ใช่ตัวให้สิทธิ์ API และ DB ยังคุมขอบเขตด้วย server เอง
  - `draft_worker.rs`, `order_draft_prompt.txt` — แก้: ส่ง guidance ที่บันทึกและตรวจแล้วให้ AI อ่านก่อนเตรียมร่าง มี source อ้างอิง shop_billing_guidance พร้อมกติกาว่าถ้าทำไม่ได้อย่างปลอดภัย/ข้อมูลขาดต้องถาม ไม่อ้างว่าได้ทำแล้ว ไม่เปลี่ยน schema/tools/สิทธิ์/ร้าน/ขั้นตอนส่งบิล คงการตรวจหลักฐานและ version fence เดิม
  - `routes/products.rs` — แก้: ส่งเฉพาะคำค้นแนะนำและเวอร์ชันนโยบายให้หน้าแค็ตตาล็อก ใช้ร้าน/สาขาจากบัญชี server ไม่รับ shop ID จากข้อความ
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`**
  - `shop-settings.tsx`, `shop-settings.css`, `api.ts` — แก้: ข้อความและตัวอย่างรองรับเงื่อนไขทั่วไป แสดง guidance ที่บันทึกและโหมดแอดมินเลือกเอง แยก error นอกขอบเขต/รูปแบบไม่ถูกต้อง/ตรวจไม่สำเร็จ คงข้อความเมื่อผิดพลาด และแสดงสถานะกำลังตรวจเงื่อนไข
  - `order-cart.tsx` — แก้: ค้นสินค้าแนะนำในร้านและสาขาที่เลือกแบบอ่านข้อมูล แสดงการ์ดราคา/ค่าประกันพร้อมปุ่มเพิ่มลงตะกร้าเดิม ไม่มีการเพิ่มอัตโนมัติ ตัดผลค้นเก่าเมื่อสาขา/นโยบายเปลี่ยน และมีปุ่มค้นใหม่เมื่ออ่านไม่ได้
  - `admin-workspace.tsx` — แก้: ชื่อหลักฐานเงื่อนไขเพิ่มเติมเป็นภาษาไทย
- **โฟลเดอร์ `backend/src/` (tests), `docs/`, `prompts/`**
  - `tests_billing_guidance.rs`, `tests_shop_settings.rs`, `tests.rs` — เพิ่มและแก้: เงื่อนไขตามภาพ ภาษาทั่วไป รูปแบบ script/SQL อักขระแฝง คำค้นแต่งขึ้น ผลตรวจผิด schema/ข้ามร้าน/เปิด tools ไม่บันทึก สิทธิ์/ขอบเขตร้านเดิม การไม่แต่งข้อมูลลูกค้าจาก policy และการนำเงื่อนไขหมายเหตุไปใช้ใน draft จริงผ่าน mock Hermes โดยไม่มีการเขียน orders
  - `shop-billing-instructions.md` — แก้: วิธีใช้เงื่อนไขทั่วไป คำแนะนำสินค้า ขอบเขตหลักฐาน การตรวจภาษา และเงื่อนไขที่ server บังคับซ้ำ
  - `ai-briefting-requirements.md` — แก้: ลงสถานะ Done พร้อมเวลาจริงที่รายการ Textbox/guardrails เดิม ไม่เพิ่ม requirement ซ้ำ และตัด trailing space ของบรรทัดที่แก้
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - release และ manifests/checksums — build-all.bat บน local สำเร็จ backend `rel-20261115104252-f7bf20540a02` และ frontend `front-20261115104400-5c8d9c87d939` ตรวจ backend checksum และ frontend checksum ครบ 2111 ไฟล์
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 73/73 กับ Mongo disposable ที่ลบแล้ว; npm test ผ่าน 44/44 และ lint/build ผ่าน ตรวจ artifact จริง Chrome/WebKit desktop/375px/320px รวม 6 กรณี บันทึกข้อความตามภาพ เก็บค่าหลัง reload ปฏิเสธ script/ข้ามร้าน คงข้อมูลเมื่อผิดพลาด สิทธิ์ staff และไม่เขียนตะกร้าจนกดเพิ่ม ทดสอบ semantic gate/การเตรียมร่างด้วย mock Hermes ไม่ได้วัดความแม่นยำโมเดล production หรือส่งบิลจริง ไม่ได้ commit/push/deploy server

## 2026-11-15 17:03 — ตรวจเหตุ HTTPS ใช้งานไม่ได้จาก Nginx ชนพอร์ต Caddy
- **คำอธิบาย:** ตรวจโดเมนจริงและ SSH Ubuntu พบ DNS ชี้เครื่องที่เข้า SSH ได้ แต่ HTTPS พอร์ต 443 เชื่อมต่อไม่ได้ Caddy failed ตั้งแต่ 05:08:54 UTC ด้วย status `loading new config: http app module: start: listening on :80: listen tcp :80: bind: address already in use` ขณะ Nginx ทำงานบนพอร์ต 80 และทั้งสอง service ถูก enable ให้เริ่มอัตโนมัติ
- **ผลตรวจ server (อ่านอย่างเดียว):** frontend 127.0.0.1:18081 ตอบ HTTP 200, backend 127.0.0.1:18080/api/health ตอบ status ok; Caddyfile มีเส้นทางไป frontend/backend/Hermes และ adapt config ผ่าน Nginx เปิดเฉพาะ default site พร้อม location proxy webhook เดิม พอร์ต 80 ตอบหน้า default ของ Nginx ไม่มี HTTPS listener
- **แนวทางแก้ที่เตรียม:** ให้ผู้ดูแล validate Caddyfile ด้วย sudo จากนั้น disable --now nginx และ restart caddy แล้วตรวจ HTTPS อีกครั้ง การสั่ง sudo -n ผ่าน SSH ถูกปฏิเสธเพราะต้องใช้รหัสผ่าน จึงยังไม่ได้หยุด/ปิด/รีสตาร์ทหรือแก้ config บริการใดบน production
- **โฟลเดอร์ `prompts/`**
  - `requirements-v0.2.md` — เพิ่มและแก้: รายการตรวจสาเหตุพร้อมสถานะ Done และเวลาจริง ระบุผลตรวจและขั้นตอนที่ยังต้องให้ผู้ดูแลรัน ตัด trailing space ของ placeholder เดิม
- **การตรวจสอบ:** curl HTTPS ภายนอกเชื่อมต่อ 443 ไม่ได้ทั้งก่อนและหลังตรวจ; systemctl/ss/process/config และ HTTP probes ภายในยืนยันผลข้างต้น แก้เฉพาะเอกสาร จึงไม่มี build หรือ deploy โปรแกรม

## 2026-11-15 12:38 — คลังข้อความด่วน คำสั่ง slash และการ์ดรูปภาพสำหรับตอบแชท
- **คำอธิบาย:** เพิ่มคลังข้อความตอบกลับส่วนกลางของร้าน ให้พนักงานที่เข้าใช้หลังบ้านเพิ่ม แก้ไข ลบ และเลือกข้อความ/การ์ดรูปภาพได้ ใช้ร่วมกันข้ามอุปกรณ์ พิมพ์ / นำหน้าในช่องแชทเพื่อแนะนำข้อความที่ตรงชื่อหรือเนื้อหาเหนือช่องพิมพ์ เลือกแล้วแก้ไขก่อนส่ง รูปมีตัวอย่างและปุ่มส่งแยก ไม่ส่งข้อความหรือรูปอัตโนมัติเมื่อบันทึก/เลือกรายการ
- **โฟลเดอร์ `backend/src/models/`, `backend/src/db/`, `backend/src/routes/`**
  - `quick_replies.rs` — เพิ่ม: โมเดล validation และ API คลังข้อความ/รูป แยกร้านและ channel, pagination ทีละ 80, UUID กันสร้างซ้ำ, expected_version กันเขียนทับ/ลบผิดเวอร์ชัน ผู้สร้าง/ผู้แก้พร้อมเวลา ตรวจ session/CSRF ผ่าน middleware เดิม ปฏิเสธ actor ปลอมจาก input และชนิดไฟล์ที่ไม่ใช่ JPEG/PNG
  - `db/quick_replies.rs` — เพิ่ม: เก็บภาพต้นฉบับและ thumbnail แบบถาวรใน GridFS ผ่าน URL ที่ต้องเข้าสู่ระบบ แก้รูปแล้วลบไฟล์เก่าหลัง CAS สำเร็จ ลบรายการแบบ soft delete พร้อมลบไฟล์ต้นฉบับ เตรียมรูปโดยคัดลอกเป็น outbound media ของ inquiry นั้นและตรวจเวอร์ชัน รูปที่เตรียมไว้คงอยู่แม้แก้/ลบการ์ดภายหลัง
  - `db/outbound.rs` — แก้: แยก upload_images สำหรับรูปที่ตรวจแล้ว ใช้ pipeline เดิมของ LINE และคงการตรวจ PUBLIC_BASE_URL ก่อน validation ของ upload ปกติ การส่งยังใช้ LINE retry key เดิม ไม่เพิ่มการส่ง LINE ในขั้นเตรียมการ์ด
  - `models/mod.rs`, `db/mod.rs`, `routes/mod.rs`, `db/indexes.rs` — แก้: ลงทะเบียน module เส้นทาง body limit รูป และ index แบบ idempotent โดยไม่เพิ่ม env หรือ dependency
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`, `frontend/src/app/`**
  - `quick-replies-dialog.tsx`, `quick-replies.css` — เพิ่ม: คลัง 2 แท็บ ข้อความด่วน/การ์ดรูปภาพ ค้นหา เพิ่มจากข้อความที่พิมพ์ค้าง แนบรูป แก้ไข ลบพร้อมยืนยัน บอกผู้แก้และเวลา คงข้อมูลฟอร์มเมื่อ save ชนเวอร์ชัน ใช้ native dialog/portal รองรับมือถือ ป้องกัน submit ฟอร์มใน dialog ลามไปส่งแชท
  - `reply-composer.tsx` — แก้: ปุ่มคลังในเมนู +, autocomplete เหนือช่องพิมพ์เมื่อขึ้นต้น / รองรับ ArrowUp/Down Enter และ Escape จำกัดแนะนำ 8 รายการ ไม่แสดงการ์ดใน slash และไม่ส่ง slash ที่กำลังเลือกให้ลูกค้า เลือกข้อความแล้วเติมลงกล่อง เลือกการ์ดแล้วเตรียมรูปพร้อมปุ่มส่ง คงข้อความที่พิมพ์ค้างและ Enter/Shift+Enter เดิมของคอม/มือถือ
  - `quick-replies.ts`, `reply-images.ts`, `layout.tsx` — เพิ่มและแก้: จัดอันดับชื่อ/ข้อความที่ตรงคำค้น รองรับภาษาไทย/ตัวพิมพ์ใหญ่เล็ก โหลดคลังทุกหน้า และแปลงรูปใช้ร่วมกับการส่งรูปเดิมโดยไม่เพิ่ม dependency
- **โฟลเดอร์ `backend/src/`, `frontend/tests/`, `docs/`, `prompts/`**
  - `tests_quick_replies.rs`, `tests.rs` — เพิ่มและแก้: 3 tests ตรวจ validation/auth, CRUD/idempotency, การแก้พร้อมกัน, การแยกร้าน/channel, เปลี่ยน/ลบไฟล์รูป, รูปถาวร, pagination 81 รายการ และ mock LINE ยืนยันว่าการเตรียมรูปไม่ส่งข้อความ ส่วนส่งซ้ำ UUID เดิมเกิด push เพียงครั้งเดียว
  - `quick-replies.test.mjs` — เพิ่ม: 3 tests แยก slash ที่ต้นข้อความ จัดอันดับ/ค้นข้อความไทย และไม่รวมการ์ดรูป/ไม่ประมวลผล HTML ในข้อความ
  - `quick-replies.md` — เพิ่ม: วิธีใช้คลัง การเลือกข้อความและการ์ด รูปแบบไฟล์ การเก็บข้อมูล สิทธิ์ และ API
  - `requirements-v0.2.md` — แก้: บันทึกสถานะ Done พร้อมเวลาจริงให้ add-on ทั้ง 3 ข้อ รอผู้ใช้ตรวจ และจัด newline ท้ายไฟล์ให้ผ่าน diff --check
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - release และ manifests/checksums — build-all.bat บน local สำเร็จ backend `rel-20261115053559-915359dc8e7d` และ frontend `front-20261115053706-2c233abd5163` ตรวจ backend ELF/checksum และ frontend checksum ครบ 2111 ไฟล์
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 70/70 กับ Mongo disposable ที่ลบแล้ว; npm test ผ่าน 44/44, npm run lint และ npm run build ผ่าน ตรวจ release จริงด้วย Chrome/WebKit desktop/375px/320px รวม 6 กรณี ครอบคลุม slash/keyboard การบันทึก-แก้ไข-ลบ ข้อขัดแย้งเวอร์ชัน การโหลดใหม่ การ์ดรูป การส่งรูปปกติ การไม่ส่งทันที ข้อความค้าง และ retry UUID เดิม ไม่ได้ส่ง LINE หรือสร้างบิลจริง ไม่ได้ commit/push/deploy server

## 2026-11-15 12:13 — กฎร้าน: ใช้ค่าเช่า/ค่าประกันจากสรุปบิลเป็นหลัก ไม่คำนวณจากตะกร้า
- **คำอธิบาย:** ผู้ใช้ต้องการให้บิลใช้ค่าเช่าและค่าประกันตามสรุปบิลของแอดมิน และเพิ่มเงื่อนไขนี้ใน guardrail ของเงื่อนไขร้าน จึงเพิ่มเป็นกฎชนิดที่ 3 ในรูปแบบกฎที่อนุญาต พิมพ์ "แอดมินสรุปบิลให้เอาค่าเช่า ค่าประกันตรงหน้าสรุปบิลเป็นหลักไม่คำนวณจากตะกร้าเอง" แล้วบันทึกได้ เมื่อเปิดกฎ ใบสรุปจะใช้ยอดในสรุปบิลเป็นยอดสุทธิ ส่วนลดที่ส่ง Alex = ราคาแค็ตตาล็อก − ยอดในสรุปบิล โดยไม่ใช้ช่องส่วนลดในร่าง ถ้ายอดขาดหรือสูงกว่าราคาใน Alex จะหยุดไม่ให้ส่งแทนการคิดจากตะกร้า ร้านที่ไม่เปิดกฎยังทำงานแบบเดิม
- **โฟลเดอร์ `backend/src/models/`**
  - `shop_settings.rs` — แก้: เพิ่ม `summary_amounts` (serde default สำหรับ snapshot เก่า) และ parser แบบปิดสำหรับประโยคของผู้ใช้และรูปแบบใกล้เคียง (เอา/ใช้, ค่าเช่า และ/,/ ค่าประกัน, ตรง/จาก/ตาม/ใน (หน้า)สรุปบิล, เป็นหลัก, ไม่คำนวณจากตะกร้า(เอง)) ข้อความต่อท้ายอื่นหรือกฎซ้ำถูกปฏิเสธทั้งชุดเหมือนกฎเดิม
  - `checkout.rs` — แก้: เพิ่ม `amounts_from_summary` ใน Group (serde default)
- **โฟลเดอร์ `backend/src/db/`**
  - `checkout.rs` — แก้: เมื่อเปิดกฎ ไม่ใช้ส่วนลดในร่าง คิดส่วนลดจากยอดในสรุปบิล เพิ่ม `summary_issues` แจ้ง summary_rental/summary_deposit เมื่อไม่มียอด และ summary_*_above_catalog เมื่อยอดเกินราคาใน Alex (ตรวจหลังตะกร้าครบ) กฎและผลรวมอยู่ใน source hash เดิม
  - `shop_settings.rs` — แก้: แยก `net_rental` ให้เกณฑ์ค่าขนส่งใช้ค่าเช่าในสรุปบิลเมื่อเปิดกฎ
- **โฟลเดอร์ `backend/src/`**
  - `order_draft_prompt.txt` — แก้: บอก Hermes ว่ารองรับ summary_amounts ให้คัดลอกค่าเช่า/ค่าประกันจากสรุปบิลตรงตัวพร้อมหลักฐาน ห้ามคำนวณจากราคาสินค้า/ตะกร้า ถ้าไม่มีให้ null และถาม
  - `tests_shop_settings.rs` — แก้: ทดสอบประโยคของผู้ใช้และรูปแบบที่รองรับ การปฏิเสธข้อความแทรก/script/URL/กฎซ้ำ การอ่าน rules เก่า `summary_issues` และ `net_rental` และต่อ integration test ให้ตรวจบิลจริง: ไม่มีค่าประกันในสรุป → หยุด, สรุป 500/200 กับตะกร้า 550/300 → ส่วนลด 50/100 แม้ร่างมีส่วนลด 0 พร้อมค่าขนส่ง และยอดสรุปเกิน Alex → หยุด
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`**
  - `shop-settings.tsx` — แก้: ตัวอย่างเงื่อนไข คำอธิบาย และกฎที่ใช้งานอยู่แสดงกฎยอดตามสรุปบิล
  - `rental-checkout.tsx` — แก้: ข้อความรายการที่ต้องแก้ของกฎใหม่ และแจ้งในสรุปยอดว่าใช้ค่าเช่า/ค่าประกันตามสรุปบิล ส่วนลดคิดจากส่วนต่าง พร้อมแยกค่าขนส่งตามเงื่อนไขร้านที่รวมในค่าเช่าสุทธิ
  - `api.ts` — แก้: ข้อความ BILLING_RULES_INVALID ระบุรูปแบบที่ 3 (แก้เฉพาะบรรทัดนี้ ไม่แตะงาน chat link ที่ค้างอยู่ในไฟล์)
- **โฟลเดอร์ `docs/`**
  - `shop-billing-instructions.md` — แก้: รองรับ 3 กฎ ตัวอย่าง หัวข้อค่าเช่า/ค่าประกันตามสรุปบิล และเพิ่มเงื่อนไขนี้ในหัวข้อการจำกัดขอบเขต (guardrails)
  - `image-rental-billing.md` — แก้: ประโยคแนะนำเงื่อนไขร้าน
- **โฟลเดอร์ `dist/line-linux/`**
  - Linux backend release — แก้: build บน local ผ่าน `build-backend.bat` รุ่น `rel-20261115051231-af2c175eb2a9` ตรวจ ELF/checksum แล้ว ไม่มีไฟล์ env ใน release
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 67/67 ด้วย Mongo disposable (ลบ container/volume แล้ว) ถ้ารัน `cargo test` โดยไม่มี Mongo จะล้ม 1 เทสต์ของ line_tests ที่ต้องต่อ DB ไม่เกี่ยวกับงานนี้ frontend lint, TypeScript, npm test 41/41 และ npm run build ผ่าน ยังไม่ได้ build frontend release เพราะใน working tree มีงาน chat link ที่ยังไม่ commit และยังแก้อยู่ ไม่ได้ตรวจบนเบราว์เซอร์ ไม่ได้เรียกสร้างบิลจริงบน production ไม่ได้ commit/push/deploy
## 2026-11-15 12:07 — ลิงก์ภายนอกและตำแหน่งในแชทพร้อมยืนยันก่อนเปิด
- **คำอธิบาย:** ทำให้ URL ในข้อความลูกค้าและแอดมินกดได้ เปิด popup แสดงเว็บไซต์และ URL ปลายทางก่อนยืนยัน กดตกลงแล้วเปิดแท็บใหม่โดยคงแชทและข้อความที่พิมพ์ค้างไว้ รองรับหมุด LINE พิกัดทศนิยม และปุ่มค้นหาที่อยู่ใน Google Maps ไม่โหลด preview หรือแผนที่ก่อนกดตกลง
- **โฟลเดอร์ `frontend/src/components/`**
  - `chat-linked-content.tsx`, `chat-linked-content.css` — เพิ่ม: ลิงก์ขีดเส้นใต้ การ์ดตำแหน่ง ปุ่มค้นหาที่อยู่ และ native dialog ผ่าน portal รองรับยกเลิก Escape คืน focus และจอมือถือ ปลายทางใหม่ใช้ noopener/noreferrer ไม่ส่ง referrer ไม่บังคับเลือกแอป default ของระบบ
  - `chat-message.tsx` — แก้: แสดงข้อความและ location ผ่าน component ใหม่ คงรูป สติกเกอร์ การแนบรูปให้ AI และข้อความที่ยกเลิกไว้ตามเดิม หากข้อความถูกยกเลิกระหว่างเปิด popup ให้ปิด popup ด้วย
- **โฟลเดอร์ `frontend/src/lib/`, `frontend/src/app/`**
  - `chat-links.ts` — เพิ่ม: แยก HTTP(S)/www โดยคงข้อความและเครื่องหมายเดิม ปฏิเสธ scheme อื่น credentials และอักขระควบคุม ตรวจช่วงพิกัดรวมค่า 0 แยกคู่จำนวนเงินทั่วไป สร้าง Maps URL แบบ encoded query จำกัด 2048 ตัวอักษร และค้นหาที่อยู่ไทย/ข้อความระบุสถานที่แบบ heuristic ตัดชื่อ เบอร์โทร และรายละเอียดออเดอร์ออกจาก query ตามรูปแบบที่ตรวจพบ
  - `api.ts`, `layout.tsx` — แก้: เพิ่มชนิดข้อมูล title/address/latitude/longitude ใน payload เดิม และโหลด CSS ของ component โดยไม่เพิ่ม API หรือ dependency
- **โฟลเดอร์ `frontend/tests/`, `docs/`**
  - `chat-links.test.mjs` — เพิ่ม: 8 tests ครอบคลุม URL/ข้อความไทย punctuation scheme/credentials/control chars โดเมน Maps ปลอม พิกัด ที่อยู่ เบอร์โทร และการเข้ารหัส query
  - `chat-links-and-locations.md` — เพิ่ม: วิธีใช้ รูปแบบที่ตรวจพบ ข้อจำกัด heuristic และการเปิดเบราว์เซอร์/แอป Maps พร้อมเอกสารอ้างอิง
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone, static assets, `frontend.tag`, `frontend-release.json`, `frontend.sha256` — build บน Windows ด้วย Docker Desktop สำเร็จ release `front-20261115050648-c52b2e931a6e` ตรวจ checksum ครบ 2110 ไฟล์
- **การตรวจสอบ:** npm test ผ่าน 41/41, npm run lint และ npm run build ผ่าน ตรวจ artifact สุดท้ายด้วย Chrome/WebKit อย่างละ desktop/375px/320px รวม 6 กรณี ยืนยันว่าไม่มี external request ก่อนตกลง ยกเลิก/Escape ไม่เปิดเว็บ แท็บใหม่ไม่มี opener/referrer Maps ใช้ query ถูกต้อง คงข้อความค้าง และไม่ล้นจอ ไม่ได้ส่งข้อมูลจริงออกไปเว็บภายนอกในการทดสอบ ไม่ได้ commit/push หรือ deploy server

## 2026-11-15 03:28 — ตั้งค่าเงื่อนไข AI ออกบิลรายร้านพร้อมกฎที่จำกัดขอบเขต
- **คำอธิบาย:** เพิ่มไอคอนเฟืองด้านขวาการ์ดชื่อร้านใน Home และหน้า Settings ให้เจ้าของร้านกรอกเงื่อนไขก่อน AI เตรียมออเดอร์ รองรับคำนำหน้าชื่อลูกค้าและสินค้าค่าขนส่งตามยอดเช่าสุทธิ เช่น K. และยอดต่ำกว่า 600 บาทเพิ่มสินค้ารหัส 0053MNANANA ราคา 40 บาท ตัวอย่างไม่ถูกเปิดใช้เอง เงื่อนไขอื่นหรือคำสั่งแทรกจะถูกปฏิเสธทั้งชุด
- **โฟลเดอร์ `backend/src/models/`, `backend/src/db/`**
  - `shop_settings.rs` — เพิ่ม: แปลงข้อความแบบ closed billing language เป็นกฎ typed ที่อนุญาต ไม่ใช้ eval/สคริปต์/คำสั่ง shell/URL หรือส่งข้อความดิบเป็น prompt จำกัดความยาว จำนวนกฎ รูปแบบเงิน และรหัสสินค้า ปฏิเสธอักขระควบคุม กฎซ้ำ ข้อความนอกขอบเขต และ field แทรกจาก JSON บันทึกแยก shop/channel พร้อม CAS version ผู้แก้ วันเวลา และประวัติล่าสุด 100 ครั้ง
  - `checkout.rs` — แก้: บังคับคำนำหน้าชื่อซ้ำที่ server และเพิ่มสินค้าค่าส่งเมื่อค่าเช่าหลังส่วนลดต่ำกว่าเกณฑ์ ใช้รหัสตรงในสาขาที่อนุญาต ต้องพบสินค้าเดียว ราคาและค่าประกันตรงเงื่อนไข จำนวน 1 รายการ ไม่เดาราคา/stock ID ไม่คิดซ้ำจากตะกร้าหรือค่าส่งในหมายเหตุ คงการตรวจ catalog/คิว/กันส่งซ้ำเดิม รวมเวอร์ชันกฎและสินค้าที่คำนวณใน source hash และบันทึก snapshot กฎในใบตรวจ/บิล
  - `drafts.rs`, `indexes.rs`, `mod.rs` — แก้: ผูกงานร่างกับเวอร์ชันเงื่อนไข และเพิ่ม collection/index ของ shop_settings แบบ idempotent
- **โฟลเดอร์ `backend/src/`, `backend/src/routes/`**
  - `draft_worker.rs`, `order_draft_prompt.txt` — แก้: ส่งเฉพาะ billing_rules ที่ตรวจแล้วให้ Hermes อ่านก่อนสรุป ยังคงปิด tools และตรวจหลักฐานข้อมูลลูกค้าตามเดิม เติม prefix ด้วย server หลังตรวจหลักฐานพร้อม evidence shop_billing_rule ตรวจเวอร์ชันเงื่อนไขก่อนและหลังประมวลผล ไม่บันทึกผลจากเงื่อนไขที่เปลี่ยนระหว่างงาน
  - `shop_settings.rs`, `admin.rs`, `mod.rs`, `error.rs` — เพิ่มและแก้: GET/POST /api/admin/shop-settings จำกัดเจ้าของร้านทั้ง middleware/DB ใช้ session/CSRF เดิมและ error code เฉพาะ โดยไม่เพิ่ม env หรือ dependency
- **โฟลเดอร์ `backend/src/` (tests)**
  - `tests_shop_settings.rs`, `tests.rs`, `tests_team.rs` — เพิ่มและแก้: ทดสอบรูปแบบกฎ script/URL/role injection/control chars ขอบเขต 599.99/600 การแยกร้าน สิทธิ์ owner/manager/staff ผ่าน API และ DB, CAS/audit, exact SKU/ราคา/ค่าประกัน/สินค้ากำกวม การเพิ่มค่าส่งเพียงครั้งเดียว ชื่อ K. ในร่างและ payload จริงของใบตรวจ และปฏิเสธใบตรวจ/งานเตรียมร่างที่ใช้กฎเก่า
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/app/`, `frontend/src/lib/`, `frontend/tests/`**
  - `shop-settings.tsx`, `shop-settings.css` — เพิ่ม: Textbox เงื่อนไข รูปแบบตัวอย่าง กฎที่ใช้จริง ผู้แก้/เวลา บันทึก/ยกเลิก/ล้างกฎ และคงข้อความเมื่อ save ชนกับอีกหน้าต่าง ไม่แสดงหรือเปิดหน้าให้บัญชีที่ไม่มีสิทธิ์
  - `shop-dashboard.tsx`, `shop-dashboard.css`, `admin-portal.tsx`, `workspace-navigation.tsx`, `workspace-navigation.ts`, `layout.tsx` — แก้: ไอคอนตั้งค่าตรงการ์ดชื่อร้าน เส้นทาง #settings และเมนูร่วมกับหน้าอื่น
  - `team-workspace.tsx` — แก้: ใช้ viewport hook ร่วมกับ Settings และคงโฟกัสจนปุ่มได้รับ click ก่อนหุบคีย์บอร์ด ป้องกันปุ่มบันทึกมือถือขยับจนต้องแตะซ้ำ
  - `admin-workspace.tsx`, `api.ts`, `workspace-navigation.test.mjs` — แก้: ชื่อหลักฐานเงื่อนไขเป็นภาษาไทย ข้อความ error ที่อธิบายวิธีแก้ และทดสอบเส้นทาง Settings
- **โฟลเดอร์ `docs/`**
  - `shop-billing-instructions.md` — เพิ่ม: รูปแบบ 2 ชนิดที่รองรับ วิธีใช้ เงื่อนไขสินค้า ค่าขนส่ง guardrails และการผูกเวอร์ชัน
  - `workspace-navigation.md`, `image-rental-billing.md` — แก้: ทางเข้าตั้งค่าร้านและลิงก์คู่มือ
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux releases — แก้: build ทั้งสองส่วนบน local พร้อมตรวจ ELF/checksum backend และ manifest frontend ครบ 2,110 ไฟล์ ไม่พบ env ใน release รุ่น backend `rel-20261114201901-1f2d029ec187` และ frontend `front-20261114202110-6ec6e08f9bf9`
- **การตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 65/65 ด้วย Mongo disposable ที่ลบ container/volume หลังทดสอบแล้ว frontend lint/TypeScript/build และ npm test ผ่าน 33/33 ตรวจ Settings บน Chrome/WebKit desktop/มือถือ 6 กรณีบน release สุดท้าย รวมบันทึกครั้งเดียว คำสั่งแทรกไม่ทำงาน ข้อความยังอยู่เมื่อผิดพลาด/ชนเวอร์ชัน ล้างกฎ และปฏิเสธ staff พร้อมทดสอบหน้าเดิม/ฟอร์มทีมย้อนหลังอีก 14 กรณี ไม่พบ page error/คำขอที่ fixture ไม่รองรับ ตรวจภาพ Home/Settings แล้ว ไม่ได้ทดสอบ iPhone เครื่องจริง ไม่ได้เปิดใช้ตัวอย่างหรือเรียกสร้างบิลจริงบน production ไม่ได้ commit/push/deploy

## 2026-11-15 02:43 — ส่งออเดอร์จาก modal คลิกเดียวและตัดช่องที่ไม่จำเป็น
- **คำอธิบาย:** ผู้ใช้ต้องการตัดช่องค่าจัดส่งและวันเกิดออกจากใบสรุป และไม่ต้องติ๊ก checkbox หรือยืนยันซ้ำก่อนส่งเข้า Alex กรณีคิวว่าง ปุ่มส่งจะตรวจข้อมูล/ราคา/คิวและส่งต่อในคลิกเดียว โดยยังใช้ API และการตรวจสิทธิ์/ต้นฉบับ/กันส่งซ้ำของ backend เดิม
- **โฟลเดอร์ `frontend/src/components/`**
  - `rental-checkout.tsx` — แก้: ลบช่องกรอกค่าส่งและวันเกิด ใช้ค่าส่งที่มีในร่างล่าสุดหรือ 0 เมื่อยังไม่ทราบ ส่ง birth_date เป็น null ลบส่วนยืนยันซ้ำและ checkbox ยืนยันทั่วไป รวม review/submit เป็นปุ่มส่งเดียว มีสถานะกำลังส่งและ ref กันกดซ้ำ ลองใหม่หลังเครือข่ายขัดข้องด้วย review ID เดิม รวมกรณีส่งทั้งที่ติดคิว ถ้าคิวไม่ว่างหรือตรวจไม่ได้จะแสดงรายการและเปลี่ยนปุ่มให้ระบุชัดว่าเลือกส่งต่อแม้ติดคิว โดยไม่ใช้ checkbox ยังไม่ข้ามการตรวจคิวอัตโนมัติ และคง checkbox โปรโมชั่น/บันทึกผลจาก Alex ตามหน้าที่เดิม
  - `draft-editor.tsx` — แก้: คำอธิบายเปิดใบสรุปให้สอดคล้องกับปุ่มส่งครั้งเดียว
- **โฟลเดอร์ `frontend/src/lib/`**
  - `api.ts` — แก้: ข้อความ QUEUE_BLOCKED ไม่แนะนำให้ติ๊กช่องยืนยันที่ถูกนำออกแล้ว
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: คู่มือใบสรุป ช่องที่ตัดออก ค่าส่งจากร่าง การส่งครั้งเดียว และกรณีเลือกส่งต่อเมื่อติดคิว
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build บน local ผ่าน `build-frontend.bat` รุ่น `front-20261114194119-3274681b277b` ตรวจ checksum ครบ 2,110 ไฟล์และไม่พบไฟล์ env
- **การตรวจสอบ:** lint, TypeScript และ production build ผ่าน ตรวจ browser fixture บน release สุดท้ายผ่าน 16 กรณี Chrome/WebKit บน desktop/มือถือ ครอบคลุมส่งปกติในคลิกเดียว กดซ้ำระหว่างส่ง ช่องค่าส่ง/วันเกิดหายไป ค่าส่งไม่ทราบเป็น 0 และค่าส่งจากร่างยังคงอยู่ เครือข่ายขัดข้องใช้ review ID เดิมทั้งคิวว่าง/ติดคิว หยุดก่อนข้ามคิว การเปลี่ยนคิวระหว่างส่ง validation ช่องจำเป็น และพนักงานไม่มีสิทธิ์ส่ง ไม่พบ page error หรือคำขอที่ fixture ไม่รองรับ ตรวจภาพท้าย modal จาก WebKit แล้ว ไม่ได้ส่งบิลจริงไป Alex ไม่ได้ commit/push หรือ deploy

## 2026-11-15 02:12 — แยกหน้าหลัก รายการแชท และทีม พร้อมย่อเมนูมือถือ
- **คำอธิบาย:** ผู้ใช้ต้องการเมนูหลักใหม่เป็น Home / รายการแชท / ทีม ลดบาร์ด้านล่างราวครึ่งหนึ่ง เพิ่มความชัดของข้อความ และซ่อนบาร์เมื่อเปิดลูกค้า เปลี่ยนเป็นบาร์สูงประมาณ 40px จากเดิมประมาณ 74px จัดไอคอนกับชื่อในแถวเดียวกัน เพิ่ม dashboard ร้าน Arisa Rental และปุ่มด้านบนของบทสนทนาเรียง กลับรายการ / แชทลูกค้า / ออเดอร์ลูกค้า
- **โฟลเดอร์ `frontend/src/components/`**
  - `shop-dashboard.tsx`, `shop-dashboard.css` — เพิ่ม: ภาพรวมร้าน แชทล่าสุด จำนวนแชทที่โหลดแล้ว ยังไม่อ่าน และมีร่างออเดอร์ พร้อมบัญชีที่ใช้งานและการตั้งค่าแจ้งเตือน ระบุชัดว่าสถิติมาจากข้อมูลที่โหลดอยู่ จัดการพื้นที่เลื่อนและ safe area บนมือถือ
  - `workspace-navigation.tsx` — เพิ่ม: เมนูหลักร่วมกันสำหรับหน้า Home รายการแชท และทีม ใช้ไอคอน SVG กับ aria-current เมนูทีมยังเปิดเฉพาะเจ้าของร้านตามสิทธิ์เดิม
  - `admin-portal.tsx`, `admin-workspace.tsx` — แก้: เปิด Home เป็นหน้าแรก แยกเส้นทางรายการ/บทสนทนา/ออเดอร์ ไม่แสดงบาร์ล่างเมื่อเปิดลูกค้าบนมือถือ ย้ายปุ่มแชทและออเดอร์ขึ้นด้านบน เอาปุ่มทีมและกระดิ่งเดิมออกจากหัวรายการ ย้ายการตั้งค่าแจ้งเตือนไป Home และไม่สร้างแถบเตือน LINE OA จากสถานะ/ความล้มเหลวในการซิงก์ โดยยังซิงก์และแสดงข้อผิดพลาดของข้อมูลหรือการทำรายการอื่นตามเดิม คงข้อความตอบและสรุปที่ยังไม่บันทึกเมื่อสลับแชทกับออเดอร์ของลูกค้าคนเดิม ป้องกันการทำเครื่องหมายอ่านจาก Home หรือหน้าออเดอร์บนมือถือ
  - `team-workspace.tsx`, `workspace-sidebar.tsx` — แก้: เพิ่มทางกลับ Home และใช้เมนูหลักชุดเดียวกันบนหน้าทีม ลิงก์แจ้งเตือนจากหน้าทีมเปิดลูกค้าที่ถูกต้อง คงระบบพนักงาน คำเชิญ และการอนุมัติเดิม
- **โฟลเดอร์ `frontend/src/app/`**
  - `mobile.css` — แก้: ย่อบาร์และไอคอน ใช้ชื่อเมนูแนวนอน ลบ badge จำนวนลูกค้าจากเมนูเดิม และจำกัดการซ่อนเมนูจากเครื่องมือแนบข้อความเฉพาะตอนแสดงแชท ป้องกันเมนูหายเมื่อกลับรายการ
  - `typography.css`, `workspace-layout.css` — แก้: ข้อความตัวอย่างในรายชื่อเพิ่มเป็น 13px บนคอมพิวเตอร์ / 14px บนมือถือ สี slate-600 และข้อความในบับเบิลเพิ่ม 1px ใช้สีเข้มขึ้น
  - `layout.tsx` — แก้: ลงทะเบียน stylesheet ของ dashboard และแถบเมนูบทสนทนา
- **โฟลเดอร์ `frontend/src/lib/`, `frontend/tests/`**
  - `workspace-navigation.ts`, `workspace-navigation.test.mjs` — เพิ่ม: เส้นทาง #home / #chats / #team / #chat/<id> / #order/<id> รองรับลิงก์แจ้งเตือน ?inquiry แบบเดิม ตรวจ ID และ canonicalize ตัวพิมพ์ ทดสอบเส้นทางปกติ ลิงก์ผิดรูป และ fragment คำเชิญ
- **โฟลเดอร์ `docs/`**
  - `workspace-navigation.md` — เพิ่ม: คู่มือเมนูใหม่ ขอบเขตสถิติ สิทธิ์ทีม และการเปิดแชท/ออเดอร์ผ่าน URL
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build บน local ผ่าน `build-frontend.bat` รุ่น `front-20261114190843-e9e089b7593a` ตรวจ checksum ครบ 2,110 ไฟล์และไม่พบไฟล์ env
- **การตรวจสอบ:** lint, TypeScript และ production build ผ่าน `npm test` ผ่าน 33/33 ตรวจ browser fixture บน release สุดท้ายผ่าน 14 กรณี Chrome/WebKit ครอบคลุมมือถือ 320/375/430px แนวนอน 844px แท็บเล็ต 768px และ desktop 1440px รวม Home/รายการ/แชท/ออเดอร์/ทีม เปิดฟอร์มพนักงาน ย้อนกลับด้วยเบราว์เซอร์ ลิงก์ตรงและลิงก์แจ้งเตือนเดิม การเก็บข้อความที่ยังไม่ส่ง/สรุปที่กำลังแก้ การไม่ทำเครื่องหมายอ่านบนหน้าออเดอร์มือถือ และไม่เรียก API ทีมจากบัญชีพนักงาน ไม่พบ page error หรือคำขอที่ fixture ไม่รองรับ ตรวจภาพจาก WebKit แล้ว ไม่ได้ทดสอบ iPhone เครื่องจริง ไม่ได้ commit/push หรือ deploy

## 2026-11-15 02:01 — กด Enter ส่งข้อความแชทบนจอใหญ่ ขึ้นบรรทัดใหม่บนมือถือ
- **คำอธิบาย:** ผู้ใช้ต้องการให้ช่องตอบลูกค้าบนคอมพิวเตอร์/โน้ตบุ๊กกด Enter แล้วส่งทันที Shift+Enter ขึ้นบรรทัดใหม่ ส่วนมือถือหรือจอเล็กกด Enter แล้วขึ้นบรรทัดใหม่ (ส่งด้วยปุ่มส่ง) เดิมต้องกด Ctrl/Cmd+Enter จึงจะส่ง
- **โฟลเดอร์ `frontend/src/components/`**
  - `reply-composer.tsx` — แก้: ตรวจขนาดจอตอนกดปุ่มด้วย media query เดียวกับ `mobile.css` (กว้างไม่เกิน 767px หรือจอเตี้ยแนวนอน) รวมอุปกรณ์จอสัมผัส (`pointer: coarse`) ถือเป็นจอเล็ก; จอใหญ่ Enter ส่งข้อความ Shift+Enter ขึ้นบรรทัดใหม่; ไม่ส่งขณะกำลังประกอบคำด้วย IME; Ctrl/Cmd+Enter ยังส่งได้ทุกขนาดจอ
- **การตรวจสอบ:** `npm run lint` และ `npm run build` ผ่าน ยังไม่ได้ build Linux release เพราะ Docker Desktop บนเครื่องไม่ได้เปิด (`build-frontend.bat` ล้มที่ขั้นเชื่อม Docker) ไม่ได้ทดสอบบนเบราว์เซอร์/มือถือจริง ไม่ได้ commit/push หรือ deploy

## 2026-11-13 00:19 — เปลี่ยนเมนูมือถือเป็นบาร์ไอคอนทรงแคปซูล
- **คำอธิบาย:** ปรับเมนูด้านล่างตามภาพอ้างอิง LINE จากปุ่มแยกสามกล่องเป็นบาร์ขาวโค้งมนหนึ่งชิ้น ไอคอนเส้นสีเข้ม ชื่อเมนูอยู่ใต้ไอคอน และพื้นเทาทรงแคปซูลสำหรับเมนูที่เลือก คงเมนูลูกค้า แชท ออเดอร์ และซ่อนเมนูขณะกรอกข้อมูลตามเดิม
- **โฟลเดอร์ `frontend/src/app/`**
  - `mobile.css` — แก้: รูปทรงบาร์ เงาและขอบบาง ไอคอน 28px ปุ่มสูงอย่างน้อย 60px สถานะกด/ปิดใช้งาน/โฟกัส แยกพื้นที่ safe area ด้านล่าง และลด transition เมื่อผู้ใช้ตั้ง reduced motion โดยไม่ทับพื้นที่เนื้อหา
- **โฟลเดอร์ `frontend/src/components/`**
  - `admin-workspace.tsx` — แก้: จัดไอคอนในกรอบขนาดเดียวกัน ปรับแชทเป็นรูปฟองคำพูดสามจุด และย้ายจำนวนลูกค้าเป็น badge บนไอคอน แสดง 99+ เมื่อเกิน 99 โดยชื่อที่โปรแกรมอ่านหน้าจออ่านยังบอกจำนวนจริง
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build บน local ผ่าน `build-frontend.bat` รุ่น `front-20261112171712-0c85d9767ae3` ตรวจ checksum ครบ 2,110 ไฟล์และไม่พบไฟล์ env
- **การตรวจสอบ:** `npm run lint` และ production build พร้อม TypeScript ผ่าน ตรวจเบราว์เซอร์จำลอง Chrome/WebKit รวม 12 กรณีครอบคลุมมือถือ 320/375/430px แนวนอน 844px แท็บเล็ต 768px และ desktop 1440px ตรวจการสลับเมนูด้วยคลิก/Enter ปิดเมนูที่ยังใช้ไม่ได้ บาร์ไม่เลื่อนตามรายชื่อ ไม่ทับเนื้อหา ไม่ล้นแนวนอน และซ่อนระหว่างค้นหาหรือพิมพ์ตอบ ตรวจภาพบาร์จาก WebKit แล้ว ไม่ได้ทดสอบบน iPhone เครื่องจริง ไม่ได้ commit/push หรือ deploy

## 2026-11-12 23:01 — เติมฟอร์มค้นสินค้าครบ ค้นแบรนด์ใกล้เคียง และอ่านไซซ์ที่ตกลงในแชท
- **คำอธิบาย:** ผู้ใช้แจ้งว่ากดรายการที่ AI อ่านได้แล้วแบรนด์ไม่เติมลงฟอร์ม ต้องการ dropdown พิมพ์กรองชื่อใกล้เคียง และใช้ไซซ์ที่สรุปตกลงในแชทหรือสรุปแอดมิน แก้การจับคู่ชื่อเต็มอย่าง Blush Garden - Long Dress 20 ให้พบแบรนด์ Blush Garden โดยไม่เลือกเองเมื่อมีหลายชื่อใกล้เคียงพอกัน และคงข้อมูลค้นหาจากรายการที่เลือกแม้ผลค้นเดิมว่างหรือเคยขยายตัวกรอง
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`, `frontend/src/app/`**
  - `order-cart.tsx` — แก้: เติมโหมด ชื่อ/รหัส แบรนด์ สี และไซซ์พร้อมกันจากรายการที่กด ผลค้นเดิมไม่ล้างค่าที่เพิ่งเติม ใช้รายการแรกจริงแทนการนำผลค้นของรายการอื่นมาแสดง และยังเติมได้เมื่อค้นไม่พบ; ไซซ์ของรายการมีความสำคัญก่อนตัวกรองเดิมที่ถูกขยายค้น
  - `brand-matching.ts` — เพิ่ม: จัดอันดับชื่อ/รหัสใกล้เคียง รองรับชื่อแบรนด์ปนชื่อรุ่น วรรคตอน ตัวพิมพ์ และคำสะกดคลาดเคลื่อนเล็กน้อย เลือกอัตโนมัติเฉพาะคู่ที่ชัดเจน ไม่เดาจาก prefix สั้นหรือชื่อที่กำกวม
  - `brand-combobox.tsx`, `brand-combobox.css`, `layout.tsx` — เพิ่มและลงทะเบียน: ช่องแบรนด์พิมพ์ค้นพร้อมรายการที่กรองแล้ว มีชื่อและรหัส ปุ่มล้าง/กางรายการ Arrow keys/Enter/Escape, ARIA combobox/listbox และ popup แบบ portal ที่ปรับขึ้นบนหรือลงล่างตามพื้นที่บน desktop/มือถือ ปุ่มแตะอย่างน้อย 44px ไม่เพิ่ม dependency
  - `admin-workspace.tsx` — แก้: คงโฟกัสระหว่างใช้งาน combobox โดยตรวจ data-editor-popup จาก composedPath ของ click รองรับกรณีปุ่มล้างถูกลบจาก DOM หลังล้างค่า ป้องกันการพับคีย์บอร์ดจน dropdown ปิดก่อนพิมพ์ค้นต่อบนมือถือ
- **โฟลเดอร์ `frontend/tests/`**
  - `brand-matching.test.mjs` — เพิ่ม: 5 กรณีทดสอบชื่อปนรุ่น/รหัส คำสะกดใกล้เคียง prefix ที่ไม่ควรเดา รายการว่าง/ไม่เกี่ยวข้อง และชื่อกำกวม/ชื่อซ้ำ
- **โฟลเดอร์ `backend/src/`**
  - `alex.rs` — แก้: จับคู่แบรนด์แบบตรงหรือใกล้เคียงที่ชัดเจนเพื่อส่ง category code ของ Alex จริง ปฏิเสธการเดาเมื่อหลายชื่อใกล้เคียงพอกัน พร้อม unit test ชื่อแบรนด์ปนรุ่นและคำพิมพ์คลาดเคลื่อน
  - `customer_extract.rs`, `draft_worker.rs` — แก้: เพิ่ม size_messages จากข้อความขนาด/ไซซ์ใน inquiry ปัจจุบัน รวมการยืนยันของลูกค้าและแอดมินโดยไม่ใช้คนอื่น ข้อความที่ยกเลิก หรือรูปที่ไม่ได้เลือก ตรวจหลักฐานไซซ์ให้ผูกกับรายการ ข้อความและ source ที่ส่งให้ AI จริง ไม่ยอมรับ L จาก token XL หรือใช้ข้อความแชทกับสินค้า/วันที่/ราคา ข้อมูลติดต่อยังใช้เฉพาะข้อความลูกค้าเดิม; ส่งเวลาอ้างอิงสรุปให้ AI และคง snapshot/version/สิทธิ์ก่อน commit
  - `order_draft_prompt.txt` — แก้: เลือกเฉพาะไซซ์ที่สรุปตกลงจากสรุปแอดมินหรือแชท ไม่ใช้คำถามเทียบไซซ์ ตัวเลือกที่ยังไม่ยืนยัน สัดส่วนร่างกาย หรือตัวเลขรุ่น; หลายรายการต้องระบุได้ว่าไซซ์เป็นของรายการใดและเก็บข้อความอ้างอิง กำหนดการแยกชื่อแบรนด์กับรุ่น เช่น Blush Garden / Long Dress 20 และแก้ข้อความเก่าที่ขัดกับการเติมข้อมูลลูกค้าจากแชท
  - `draft_catalog.rs` — แก้: ไม่เติมไซซ์จาก catalog มาทับค่าที่ตกลงหรือเติมแทนช่องที่ยังไม่ทราบ คงหลักฐานการยืนยันเดิมไว้ และถามให้ยืนยันเมื่อยังไม่มีไซซ์
  - `tests_rental.rs`, `tests_draft_catalog.rs` — เพิ่มและแก้: ทดสอบกรองข้อความไซซ์/หลักฐานรายสินค้า/ขอบเขต source และ integration ที่ brand ใกล้เคียงส่งรหัสจริงไปค้น ใช้ไซซ์ที่แอดมินยืนยัน และไม่เดาไซซ์แม้ catalog มีสินค้าเพียงตัวเดียว
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: คู่มือเติมฟอร์ม ค้นแบรนด์ใกล้เคียง การล้างคำค้น/ใช้คีย์บอร์ด และหลักการเลือกไซซ์ที่ตกลงแล้ว ร่างเดิมต้องเตรียม AI ใหม่เพื่ออ่านไซซ์ล่าสุด
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — build Linux/amd64 บน local: `rel-20261112155027-06735492b924` พร้อม binary/checksum/metadata
  - `frontend-linux/` — build Linux/amd64 standalone บน local: `front-20261112155836-e3e83fbd4238` พร้อม static/runtime/public/checksum/metadata
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings ผ่าน; cargo test -- --include-ignored ผ่าน 61/61 ด้วย MongoDB ชั่วคราวและ mock LINE/Alex/Hermes; frontend lint/TypeScript/build ผ่าน และ npm test ผ่าน 29/29; Chrome/WebKit desktop และมือถือ 375x667 รวม 4 กรณีผ่าน ครอบคลุมเติมฟอร์มแม้ค้นไม่พบ/ผลเดิมถูกขยาย กรองชื่อใกล้เคียง เลือกด้วยคีย์บอร์ด/แตะ ส่ง category code และไซซ์ถูกต้อง ล้าง/ไม่พบ/Escape/เปิดปิด dropdown และพื้นที่ popup ไม่ล้นจอ; regression หน้าเตรียม AI และพวงดาวเดิมอีก 4 กรณีผ่าน; ตรวจ checksum frontend ครบ 2,110 ไฟล์และ backend ตรง metadata, platform linux/amd64/build_location local ไม่มี .env ใน release และ git diff --check เฉพาะไฟล์ที่แก้ผ่าน
- **ขอบเขตและขั้นตอนถัดไป:** ไม่เรียก LINE/Alex/Hermes จริง ไม่สร้างบิลหรือเพิ่มสินค้าลงตะกร้าลูกค้า production ไม่ทดสอบ iPhone เครื่องจริง ลบ MongoDB ชั่วคราวแล้ว ตัวตรวจและภาพอยู่ใน data/brand-size-check/ ที่ถูก ignore คงไฟล์ prompts ของผู้ใช้; ยังไม่ commit/push/deploy ผู้ใช้รัน rungit.bat "Improve brand matching and confirmed chat sizes" บน Windows แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main

## 2026-11-12 18:55 — แสดงพวงดาวหมุนระหว่างเตรียมออเดอร์และเติมข้อมูลลูกค้าจากแชทล่าสุด
- **คำอธิบาย:** ผู้ใช้ขอให้ปุ่มเตรียมออเดอร์ด้วย AI แสดงพวงดาวหมุนระหว่างโหลด และเติมข้อมูลลูกค้าจากแชทล่าสุดไปพร้อมกัน ระบบแสดงสถานะตั้งแต่บันทึกสรุปจนงาน AI สิ้นสุด พร้อมนำชื่อผู้รับ เบอร์โทร และที่อยู่จากข้อความลูกค้าใน inquiry ปัจจุบันมาใส่ร่างโดยมีข้อความอ้างอิงให้แอดมินตรวจ
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/app/`**
  - `admin-workspace.tsx` — แก้: เพิ่มพวงดาว SVG 6 ดวง ข้อความกำลังเตรียมออเดอร์และข้อมูลลูกค้า และ aria-busy/live ระหว่างส่งคำขอ/PENDING/PROCESSING ป้องกันกดซ้ำและหยุดแสดงเมื่อสำเร็จหรือล้มเหลว; อัปเดตคำอธิบายให้ตรงกับการอ่านข้อมูลลูกค้าอัตโนมัติ; รักษาโฟกัสช่องกรอกชั่วคราวระหว่าง pointerdown ของปุ่มบนมือถือ แล้วพับคีย์บอร์ดหลัง click เพื่อป้องกันการคืนเมนูจนปุ่มเลื่อนก่อนรับคลิก โดยไม่ blur ช่องใหม่ที่ action เพิ่งเปิด
  - `globals.css` — เพิ่ม: animation หมุนพวงดาวด้วย currentColor ขนาด 24px และหยุด animation ตาม prefers-reduced-motion โดยคงข้อความสถานะไว้ คงปุ่ม LINE สีเขียวจากงานก่อน
- **โฟลเดอร์ `backend/src/`**
  - `customer_extract.rs` — แก้: แยกตัวคัดข้อความลูกค้าจาก snapshot ใช้ร่วมกับปุ่มค้นข้อมูลลูกค้าเดิมและการเตรียมร่าง เลือกเฉพาะข้อความ text ที่ส่งโดย LINE user ของ inquiry ไม่ใช้ข้อความร้าน/แอดมิน/คนอื่น/ข้อความที่ถูก unsend และคงลำดับเวลา; ตรวจ contact evidence ว่าค่าที่เติมอยู่ในข้อความอ้างอิงจริง ปฏิเสธการอ้างข้อความแชทเพื่อเติมข้อมูลสินค้า วันที่ ราคา หรือฟิลด์อื่น แต่ยังรับหลักฐานรูป LINE ที่แอดมินเลือกได้
  - `draft_worker.rs`, `order_draft_prompt.txt` — แก้: ส่ง customer_messages ของ inquiry ปัจจุบันพร้อม event_ref/เวลาในคำขอ AI เดียวกับรูปและสรุป โดยกำหนดให้แชทใช้กับ customer_name/customer_phone/customer_address เท่านั้น เลือกค่าหรือคำแก้ไขล่าสุดของลูกค้า ไม่เดาชื่อจาก LINE display name หรือโปรไฟล์เก่า ไม่ให้คำแก้เบอร์ลบชื่อและที่อยู่เดิม; ตรวจหลักฐานก่อน normalize และเก็บในร่าง ใช้ input_scope admin_inputs_with_customer_chat สำหรับ AI ส่วนร่างกรอกมือยังใช้ admin_inputs_only; คงการตรวจ context/summary/สิทธิ์ก่อน commit ไม่เขียนทับโปรไฟล์ลูกค้าหรือสร้างออเดอร์ภายนอกอัตโนมัติ
  - `tests_rental.rs` — เพิ่มและแก้: ทดสอบคัดข้อความเฉพาะลูกค้าที่เลือกตามลำดับเวลา ปฏิเสธค่าที่ไม่มีใน quote/คนละ source และข้อความที่ใช้กับข้อมูลออเดอร์ผิดขอบเขต; integration เตรียมร่างจากรูปพร้อมชื่อ/ที่อยู่เดิมและเบอร์ที่ลูกค้าแก้ล่าสุด โดยยืนยัน evidence ครบและไม่เขียน customer_profiles
  - `tests_draft_catalog.rs` — แก้: fixtures ตรวจ scope ใหม่ โดยแยกข้อความลูกค้าสำหรับข้อมูลติดต่อออกจาก source ของการเลือกสินค้าและไม่ดึงข้อมูลร่างเก่า
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: คู่มือพวงดาว สถานะโหลด การเติมข้อมูลลูกค้าอัตโนมัติ การใช้ค่าที่แก้ล่าสุด ข้อความอ้างอิง และปุ่มค้นข้อมูลซ้ำ
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — build Linux/amd64 บน local: `rel-20261112114054-b5fb7834e06f` พร้อม binary/checksum/metadata
  - `frontend-linux/` — build Linux/amd64 standalone บน local: `front-20261112115259-fb26d063a7e0` พร้อม static/runtime/public/checksum/metadata
- **ผลตรวจสอบ:** cargo fmt --check และ cargo clippy --all-targets -- -D warnings ผ่าน; cargo test -- --include-ignored ผ่าน 57/57 ด้วย MongoDB ชั่วคราวและ mock LINE/Alex/Hermes; frontend lint/TypeScript/build ผ่าน และ npm test ผ่าน 24/24; Chrome/WebKit desktop และมือถือ 375x667 รวม 4 กรณีผ่าน ครอบคลุมแตะปุ่มขณะช่องสรุปยังโฟกัส พวงดาวระหว่างบันทึก/ส่ง/รอคิว/PROCESSING การหมุนจริง reduced motion ป้องกันส่งซ้ำ ฟิลด์ลูกค้าครบ การหยุดเมื่อ DONE/FAILED/HTTP error และกดใหม่ได้; regression การแจ้งเตือนกับ focus ทุกฟอร์มเดิมอีก 10 กรณีผ่าน; ตรวจ checksum frontend 2,110 ไฟล์และ backend ตรง metadata, build_location local/platform linux/amd64 และไม่มี .env ใน release; git diff --check เฉพาะไฟล์ที่แก้ผ่าน
- **ขอบเขตและขั้นตอนถัดไป:** ไม่ได้เรียก AI/LINE/Alex จริงหรือทดสอบ iPhone เครื่องจริง; ลบ MongoDB ชั่วคราวที่ใช้ทดสอบแล้ว ภาพและตัวตรวจอยู่ใน data/ai-order-check/ ที่ถูก ignore; ไม่แก้ไฟล์ prompts ของผู้ใช้และคงรายการแก้จากงานก่อน; ยังไม่ commit/push/deploy ผู้ใช้รัน rungit.bat "Add AI loading stars and latest chat customer details" บน Windows แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main

## 2026-11-12 17:47 — เปลี่ยนปุ่มเข้าสู่ระบบ LINE เป็นสีเขียวและเอาไอคอนออก
- **คำอธิบาย:** ผู้ใช้ขอให้ปุ่มเข้าสู่ระบบด้วย LINE บนหน้าล็อกอินเป็นสีเขียวและมีเฉพาะข้อความตรงกลาง ปรับเฉพาะปุ่มนี้ตาม design system เดิม
- **โฟลเดอร์ `frontend/src/components/`**
  - `admin-workspace.tsx` — แก้: ใช้ class line-login แทน primary และลบไอคอนหน้าข้อความ คงลิงก์ /api/admin/line/start และข้อความเดิม
- **โฟลเดอร์ `frontend/src/app/`**
  - `globals.css` — เพิ่ม: ปุ่มสีเขียวจาก semantic token success-ink ข้อความสีขาว พร้อม hover สีเข้มขึ้นและ focus ring สำหรับการใช้คีย์บอร์ด
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone/static/runtime/checksum/metadata — build บน local ด้วย build-frontend.bat: `front-20261112104529-6b3ef15815ec`
- **ผลตรวจสอบ:** npm run lint และ TypeScript ผ่าน; local Linux frontend lint/build ผ่าน; ตรวจ Chrome และ WebKit อย่างละ desktop/มือถือรวม 4 กรณีผ่าน สีปกติ/hover ข้อความไม่มีไอคอน การจัดกึ่งกลาง focus ring ขอบหน้าจอและ href ถูกต้อง; ตรวจ checksum ครบ 2,110 ไฟล์และ metadata build_location local/platform linux/amd64 ผ่าน; git diff --check ผ่าน ภาพตรวจอยู่ใน data/login-button-preview/ ที่ถูก ignore ไม่เพิ่ม automated test สำหรับการปรับหน้าตาขนาดเล็กนี้
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat "Make LINE login button green without icon" จากนั้นบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main; ยังไม่ commit/push/deploy และไม่มีการเปลี่ยน backend

## 2026-11-10 22:57 — เพิ่มระบบเชิญพนักงานและจัดการสิทธิ์ร้าน Arisa Rental
- **คำอธิบาย:** ทำระบบตามแบบที่ผู้ใช้อนุมัติ เจ้าของร้านสร้างคำเชิญอายุ 24 ชั่วโมงเป็นลิงก์หรือ QR พนักงานยืนยัน LINE และรับคำเชิญก่อนรอเจ้าของร้านตรวจบัญชีแล้วอนุมัติ กำหนดบทบาท/สาขา แก้ชื่อทำงาน ระงับหรือคืนสิทธิ์ และดูประวัติผู้ดำเนินการพร้อมวันเวลาได้ ผู้ใช้ยืนยันให้ LINE user ID ที่เข้าได้ปัจจุบันเป็นเจ้าของร้าน จึงย้ายรายการ ADMIN_LINE_USER_IDS เดิมเพียงครั้งเดียว ไม่ให้ผู้สมัครใหม่หรือการเพิ่ม env ภายหลังกลายเป็นเจ้าของร้านอัตโนมัติ
- **โฟลเดอร์ `backend/src/models/`, `backend/src/db/`**
  - `team.rs`, `mod.rs`, `indexes.rs` — เพิ่มและลงทะเบียนบัญชี LINE/สมาชิก/คำเชิญ/ประวัติใน MongoDB พร้อม unique indexes, scope ร้าน/ช่องทาง, atomic acceptance และ version conflict; ป้องกันสองบัญชีรับคำเชิญเดียวและบัญชีเดียวรับหลายคำเชิญพร้อมกัน ลิงก์ใช้โทเคน HMAC แยก scope และเก็บเฉพาะ hash ในฐานข้อมูล
  - `cart.rs`, `products.rs`, `checkout.rs`, `drafts.rs`, `events.rs` — ตรวจสาขาของผู้ใช้กับสินค้า ตะกร้า บิลเก่า/ใหม่ การกู้สถานะและการแก้ผลบิลด้วยมือ; เก็บสมาชิกและเวอร์ชันสิทธิ์ในงาน AI ตรวจใหม่ก่อนบันทึก/ส่งออเดอร์; รองรับ duplicate-key ชนิด Command ของ MongoDB สำหรับการรับคำเชิญพร้อมกัน
- **โฟลเดอร์ `backend/src/routes/`, `backend/src/`**
  - `team.rs`, `access.rs`, `admin.rs`, `session.rs`, `mod.rs` — เพิ่ม API จัดการทีมสำหรับเจ้าของร้าน แยก verified LINE identity ออกจากสิทธิ์เข้าร้าน ผู้รออนุมัติ/ถูกระงับเข้าถึงได้เฉพาะสถานะตนเอง; ใช้ session cookie HttpOnly และ CSRF header เดิม ตรวจ role/status จากฐานข้อมูลทุกคำขอ ปิด shared-password fallback เมื่อเปิดระบบทีม และไม่ให้ service Bearer จัดการสมาชิกบุคคล
  - `cart.rs`, `checkout.rs`, `customers.rs`, `customer_notes.rs`, `drafts.rs`, `products.rs` — ส่งสิทธิ์จาก middleware ไปยัง layer ข้อมูลด้วย ScopedState และบันทึก actor เป็นชื่อทำงานคู่กับ LINE identity เดิม
  - `state.rs`, `main.rs`, `error.rs`, `draft_worker.rs` — เพิ่ม bootstrap เจ้าของร้านหลังสร้าง indexes สถานะเปิดระบบทีม ข้อผิดพลาดสิทธิ์และ conflict และการตรวจสิทธิ์งาน AI ที่เข้าคิว
  - `tests_team.rs`, `tests*.rs` — เพิ่ม integration ของ invitation/OAuth/roles/branches/revocation/audit/races และปรับ AppState fixtures เดิมให้รองรับระบบทีม โดยทดสอบผ่าน MongoDB ชั่วคราวและ mock LINE/Alex เท่านั้น
- **โฟลเดอร์ `frontend/src/components/`, `frontend/src/lib/`, `frontend/src/app/`, `frontend/tests/`**
  - `admin-portal.tsx`, `team-welcome.tsx`, `team-workspace.tsx`, `team.css`, `team-api.ts` — เพิ่มหน้ารับคำเชิญ/รอสถานะและหน้าจัดการทีมบน desktop/มือถือ มีฟอร์มสิทธิ์/สาขา ตรวจชื่อและรูป LINE, QR สร้างในเครื่อง, คัดลอกลิงก์, ประวัติและ pagination ที่ไม่ย้อนกลับเมื่อ refresh เบื้องหลัง; ส่ง token ใน URL fragment แล้วย้ายไป HttpOnly invitation cookie ไม่ส่งใน query; ยืนยันรับด้วย invitation ID/version ที่เห็นเพื่อกันสลับคำเชิญข้ามแท็บ
  - `workspace-sidebar.tsx`, `admin-workspace.tsx`, `draft-editor.tsx`, `rental-checkout.tsx`, `page.tsx`, `layout.tsx` — เพิ่มทางเข้าจัดการทีมสำหรับเจ้าของร้านและแสดงปุ่มตามสิทธิ์ พนักงานเตรียมงานได้แต่ส่งออเดอร์หรือแก้ผลบิลไม่ได้ ผู้จัดการส่งออเดอร์ได้ตามสาขา; ไม่ mount/poll ข้อมูลร้านก่อนอนุมัติหรือหลังระงับ ตรวจ session ซ้ำทุก 10 วินาทีและเมื่อกลับเข้าแท็บ
  - `use-chat-notifications.ts`, `team-workspace.tsx` — แจ้งเตือนข้อความต่อเนื่องระหว่างอยู่หน้าทีม คลิกกลับเข้าแชทลูกค้าได้ ยกเลิกงานแจ้งเตือนที่ค้างและปิด toast ของแอปเมื่อออกจาก workspace
  - `team-access.test.mjs` — ทดสอบ capability ตามสถานะ/บทบาท ความเข้ากันได้ระบบเดิม และการตรวจ fragment คำเชิญ
- **โฟลเดอร์ `frontend/`, `docs/` และ root**
  - `package.json`, `package-lock.json` — เพิ่ม qrcode และชนิด TypeScript สำหรับสร้าง QR ภายในแอป ไม่ส่งลิงก์คำเชิญให้บริการ QR ภายนอก
  - `arisa-rental-team-onboarding.md`, `step-2-admin.md`, `.env.example` — อัปเดตคู่มือใช้งานจริง การย้ายสิทธิ์เดิมครั้งเดียว การตั้ง LINE Login ให้ Published เพื่อรับพนักงานทั่วไป ข้อจำกัดคำสั่งภายนอกที่ส่งไปแล้ว และขั้นตอน deploy; ไม่เพิ่ม env หรือ secret ใหม่
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — build Linux/amd64 บน local: `rel-20261110155401-cafd0a5d7d18` พร้อม binary/checksum/metadata
  - `frontend-linux/` — build Linux/amd64 standalone บน local: `front-20261110155457-95de554f0eb2` พร้อม public/static/runtime/checksum/metadata รวมหน้าทีมและงานก่อนหน้าที่ยังไม่ส่ง Git
- **ผลตรวจสอบ:** cargo fmt --check และ cargo clippy --all-targets -- -D warnings ผ่าน; cargo test -- --include-ignored ผ่าน 55/55 รวม MongoDB integration; frontend lint/build/TypeScript ผ่านและ npm test ผ่าน 24/24; Chrome/WebKit ผ่าน 16 กรณีบน desktop, iPhone จำลอง, จอเล็กและแนวนอน ครอบคลุมสร้างคำเชิญ/QR/อนุมัติ/แก้ไข/ระงับ/ประวัติ/สถานะ/ลิงก์หมดอายุ/notification/pagination และไม่มี request ภายนอกหรือ API ที่ไม่คาดคิด; regression แจ้งเตือนและ focus ของทุกฟอร์มเดิมผ่าน 10 กรณี; npm audit --omit=dev ไม่พบช่องโหว่; ตรวจ checksum frontend ครบ 2,110 ไฟล์และ backend ตรง metadata ยืนยัน build_location local/platform linux/amd64, worker ตรง source, ไม่มี .env ใน release และ git diff --check ผ่าน
- **ขอบเขตและขั้นตอนถัดไป:** ยังไม่ทดสอบการล็อกอิน LINE ของพนักงานจริงหรือ iPhone จริง ไม่สร้างบัญชี/คำเชิญบน production ไม่ส่งข้อความ LINE ไม่เปิด dev stack และลบ MongoDB ชั่วคราวที่ใช้ทดสอบแล้ว; ไม่ commit/push/deploy และคงไฟล์ prompts ของผู้ใช้ ผู้ใช้รัน rungit.bat "Add team invitations and role-based access" บน Windows จากนั้นบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main ซึ่ง pull และ deploy Docker runtime จาก release นี้

## 2026-11-10 18:33 — ออกแบบขั้นตอนเพิ่มพนักงานร้าน Arisa Rental
- **คำอธิบาย:** ผู้ใช้ขอออกแบบขั้นตอน add user ของร้าน ร่างนี้ยึดการเพิ่มพนักงานเข้าหลังบ้าน Alex ตามบริบท โดยถามแยกจากการเพิ่มข้อมูลลูกค้าไว้แล้วและระบุสมมติฐานในเอกสาร ไม่แก้ระบบสิทธิ์หรือสร้างผู้ใช้จริง
- **โฟลเดอร์ `docs/`**
  - `arisa-rental-team-onboarding.md` — เพิ่ม: ขั้นตอนเจ้าของร้านเพิ่มคน/บทบาท/สาขา สร้างคำเชิญอายุ 24 ชั่วโมง ส่งลิงก์หรือ QR ให้พนักงานยืนยัน LINE ยอมรับคำเชิญ รอเจ้าของร้านตรวจบัญชีและอนุมัติก่อนเริ่มใช้ พร้อม Mermaid flow, ข้อความ UI, บทบาทเจ้าของร้าน/ผู้จัดการ/พนักงาน, สถานะและกรณีลิงก์หมดอายุ/ซ้ำ/ผิดบัญชี/ระงับสิทธิ์; ระบุการตรวจสิทธิ์ฝั่ง backend, audit, bootstrap เจ้าของร้านเดิม, ทางเข้ารหัสผ่านที่ห้ามข้ามสิทธิ์ และการแยกสิทธิ์แอปออกจาก OA Manager/Alex Rental ภายนอก
- **ผลตรวจสอบ:** อ่าน session.rs, admin whitelist, sidebar และคู่มือ LINE Login ปัจจุบันยืนยันว่ายังไม่มีระบบทีม/บทบาท/คำเชิญในแอป; ใช้แนวทาง UX สำหรับฟอร์มและ feedback จาก ui-ux-pro-max และตรวจเอกสาร LINE Developers เรื่องตัวตน/provider; ตรวจขั้นตอน สถานะ ตารางสิทธิ์ และลิงก์เอกสารเดิมแล้ว git diff --check ผ่าน; เอกสารเท่านั้นจึงไม่ build/deploy และไม่ commit/push คงไฟล์/รหัสและ release ที่แก้ไว้จากงานก่อนหน้า

## 2026-11-10 16:14 — แจ้งเตือนข้อความลูกค้าผ่าน Chrome และซ่อนเมนูตอนกรอกทุกฟอร์ม
- **คำอธิบาย:** ผู้ใช้ต้องการแจ้งเตือนเมื่อมีข้อความลูกค้าใหม่ และให้เมนูล่างของแอปหายขณะกรอกทุก textbox/form บนมือถือ รวมถึงลดการเสนอกรอกข้อมูลอัตโนมัติ โดยไม่อ้างว่าสามารถซ่อนแถบกุญแจ/บัตร/ตำแหน่ง/ปุ่มพับ keyboard ของ iOS ได้ทั้งหมด
- **โฟลเดอร์ `backend/src/db/`**
  - `inbox.rs` — แก้: ส่ง last_incoming_message_at/preview คู่กับ ID ของข้อความลูกค้าล่าสุดที่ยังไม่ถูกยกเลิก แยกจาก preview รวมซึ่งอาจเป็นคำตอบแอดมิน ไม่ส่ง read token ให้ browser
  - `inquiries.rs` — แก้: เพิ่ม query ตาม ID ภายใต้ shop/channel scope เดิม และตัด lease_token
- **โฟลเดอร์ `backend/src/routes/`**
  - `admin.rs` — แก้: รองรับ GET /api/inquiries?id=ObjectId พร้อม decoration ชื่อ/สถานะเดิม เพื่อเปิดแชทจากแจ้งเตือนที่อยู่นอกหน้าแรกของ inbox โดยยังผ่าน admin authorization
- **โฟลเดอร์ `backend/src/`**
  - `tests_line.rs` — แก้: ตรวจ metadata ข้อความลูกค้า, การอ่าน inquiry ตาม ID, auth/invalid ID และการป้องกันอ่านข้ามร้าน/ช่องทาง
- **โฟลเดอร์ `frontend/src/lib/`**
  - `api.ts` — แก้: เพิ่มชนิด incoming notification metadata
  - `chat-notifications.ts` — เพิ่ม: tracker แยกข้อความใหม่จาก initial snapshot/ประวัติเก่า/คำตอบแอดมิน/unsend rewind, ข้อความแจ้งเตือนตามชื่อลูกค้าที่ใช้จริง และ history จำกัดจำนวน
  - `use-chat-notifications.ts` — เพิ่ม: opt-in ผ่านปุ่มกระดิ่งเท่านั้น, เก็บ preference, รองรับ denied/unsupported/storage blocked, poll ขณะสลับแท็บเมื่อเปิดแจ้งเตือน, ยกเว้นแชทที่กำลังอ่าน, ป้องกันแจ้งซ้ำข้ามแท็บด้วย Web Locks และ ID history, เปิดแชทจาก worker/deep link และปิด toast เมื่อ logout/ปิดแจ้งเตือน
- **โฟลเดอร์ `frontend/public/`**
  - `alex-notifications-sw.js` — เพิ่ม: worker สำหรับ notification โดยใช้ scope /alex-notifications/ ไม่ควบคุมหน้าแอป ไม่ cache หน้าและไม่เรียก API; client handshake และ click เปิดเฉพาะแท็บต้นทางหรือ fallback ของ origin เดียวกัน ตรวจ inquiry ID ก่อนเปิด
- **โฟลเดอร์ `frontend/src/components/`**
  - `admin-workspace.tsx` — แก้: ปุ่มกระดิ่งพร้อมสถานะ/คำแนะนำ, เชื่อม inbox กับ notification tracker และ background polling, เปิดแชทที่เลือกจากแจ้งเตือน, แตะนอกช่องกรอกบนมือถือเพื่อ blur หลัง button click จบแล้วจึงคืน navigation
  - `customer-name-dialog.tsx`, `customer-notes.tsx`, `date-input.tsx`, `draft-editor.tsx`, `order-cart.tsx`, `rental-checkout.tsx`, `admin-workspace.tsx` — แก้: autocomplete off สำหรับทุก form/ช่องข้อความ/ตัวเลข/วันที่/select รวม login ตามคำขอ; ไม่ใส่ fake fields หรือเปลี่ยนประเภท password
- **โฟลเดอร์ `frontend/src/app/`**
  - `mobile.css` — แก้: ซ่อน page header/navigation ทันทีเมื่อ focus ช่องกรอกทุกส่วน รวม invoice portal และ dialog โดยไม่ต้องรอ viewport ยุบ; checkbox/radio/file ไม่ทำให้เข้าโหมดพิมพ์
  - `workspace-layout.css` — แก้: รูปแบบปุ่มกระดิ่งและสถานะเปิดใช้งาน
- **โฟลเดอร์ `frontend/tests/`**
  - `chat-notifications.test.mjs`, `notification-worker.test.mjs` — เพิ่ม: ตรวจ initial/repeated snapshots, incoming-only previews, new customers, pagination, stale/withdrawn snapshots, read toggles, bounded storage, worker click เฉพาะแท็บเป้าหมาย, fallback/deep link และปฏิเสธ ID/ข้อความต่าง origin
- **โฟลเดอร์ `dist/line-linux/`, `dist/frontend-linux/`**
  - Linux releases — แก้: build-all.bat ผ่านบน local Docker Desktop และ build frontend รุ่นสุดท้ายหลังเพิ่ม outside-tap behavior; backend `rel-20261110090759-fd12ec1e5eff`, frontend `front-20261110091043-4b3e961c3eed` พร้อม checksum/runtime/metadata
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy -- -D warnings และ cargo test ผ่าน (33 passed, 20 integration tests ignored ตามปกติ); รัน Mongo integration inbox/read/notification metadata เพิ่มผ่าน 1/1 ด้วย MongoDB disposable บน local ซึ่งหยุด/ลบ container แล้ว; frontend lint/build/TypeScript ผ่าน และ npm test 18/18 ผ่าน; ตรวจ frontend checksum 2,108 ไฟล์, backend checksum, Linux/amd64 build_location local, worker ตรง source และไม่มี .env ใน release
- **ผลทดสอบ browser:** Chrome/WebKit บนจอ 430x739, 375x667, แนวนอน 932x430 และ desktop 1440x900 ผ่าน ตรวจ focus/autocomplete/navigation/layout ของค้นหา แชท สรุป วันที่/โทรศัพท์ในร่าง ลูกค้า/ที่อยู่/select ใน invoice portal, Note และเปลี่ยนชื่อ พร้อมแตะนอกสรุป/Note เพื่อคืนเมนู; API fixture ตรวจ opt-in, incoming preview, hidden-tab poll, click-to-open, ยกเว้นแชทที่เปิดอ่าน, dedup หลายแท็บ, refresh, disable, denied/unsupported/storage blocked โดยไม่เรียก LINE จริง; Chrome แบบเต็ม headless และ profile ทดสอบแยกตรวจ service worker activation/client handshake/showNotification/getNotifications/renotify/cleanup ด้วย API จริงผ่าน โดยหน้า root ไม่ถูก worker ควบคุม
- **ข้อจำกัด:** notification รุ่นนี้ต้องเปิดแท็บ Alex ค้างไว้ browser/ระบบอาจชะลอ background timer; ยังไม่มี server Web Push สำหรับปิดทุกแท็บหรือปิด browser; native OS toast ที่ผู้ใช้มองเห็นและคีย์บอร์ด/AutoFill บน iPhone จริงยังต้องลองหลัง deploy เนื่องจาก headless ไม่จำลอง UI ของระบบ iOS; Safari อาจไม่ทำตาม autocomplete off และไม่มี CSS/เว็บ API ให้ซ่อนแถบระบบนี้ทั้งหมด อ้างอิง https://developer.apple.com/documentation/safari-developer-tools/autofill และ https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes/autocomplete
- **ตัวตรวจ:** `data/notification-check/` ถูก ignore เก็บ fixture และผลจำลอง; คงไฟล์ prompts ที่ผู้ใช้แก้ไว้อยู่แล้ว
- **ขั้นตอนถัดไป:** build-all.bat บน local ผ่านแล้ว ผู้ใช้รัน rungit.bat "Add Chrome message notifications and hide mobile navigation on form focus" แล้วบน Ubuntu รัน cd ~/alex-connectingpoint ตามด้วย bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy


## 2026-11-10 14:07 — ย่อ guideline HTML เป็น build-all/rungit และ deploy-all
- **คำอธิบาย:** ผู้ใช้ต้องการ guideline ใน HTML ให้ local ใช้เพียง build-all.bat แล้ว rungit.bat พร้อมข้อความ commit และ Ubuntu ใช้ cd ~/alex-connectingpoint ตามด้วย bash ./deploy-all.sh main ซึ่งทำ git load/pull และ deploy Docker ให้อัตโนมัติ
- **โฟลเดอร์ `/`**
  - `alex-line-hermes-mcp-workflow.html` — แก้: ปรับสรุปงานผู้พัฒนา ส่วน deploy หลัก ข้อ 4.2 และตาราง/ขั้นตอน Step 2 ให้ใช้ workflow เดียวกัน; code block ฝั่ง local มี 2 คำสั่งพร้อมตัวอย่าง rungit.bat "Compact mobile chat composer and attachment menu"; ฝั่ง Ubuntu ใช้ deploy-all.sh main แทนการแยก loadgit/docker-build/docker-deploy; อธิบายการ pull Git, package runtime images และ deploy --full-stack อัตโนมัติ พร้อมเก็บงานติดตั้ง/อัปเดต companion OA ไว้ในรายละเอียดแยก
- **ผลตรวจสอบ:** อ่าน deploy-all.sh ยืนยันลำดับ loadgit → docker-build --full-stack → docker-deploy --full-stack; เปิด HTML ด้วย Chrome headless ตรวจ code blocks ครบทั้ง 3 ส่วนว่าตรงตัวอย่าง local/Ubuntu และไม่มี script error; git diff --check ผ่าน แก้เอกสารเท่านั้นจึงไม่ build โปรแกรม และไม่ได้รัน script deploy/commit/push

## 2026-11-10 13:57 — ย่อแถบพิมพ์มือถือและรวมรูป/สติกเกอร์หลังปุ่มบวก
- **คำอธิบาย:** ผู้ใช้ต้องการแถบพิมพ์แถวเดียวแบบ LINE ซ่อน navigation ตอนพิมพ์ ซ่อนรูป/สติกเกอร์จนกดปุ่มบวก และแตะพื้นที่แชทเพื่อพับ keyboard/แผงเลือกทั้งหมด โดยคงข้อความและรูปที่ยังไม่ส่ง
- **โฟลเดอร์ `frontend/src/components/`**
  - `reply-composer.tsx` — แก้: เพิ่มปุ่มบวก/ปิดสำหรับเปิดตัวเลือกสติกเกอร์และรูป เปลี่ยนปุ่มส่งเป็นไอคอนพร้อม accessible label, แยก state แสดงรูปออกจากไฟล์ที่เตรียมไว้, blur ช่องพิมพ์เมื่อเปิดเครื่องมือและปิดแผงเมื่อแตะภายนอก/Escape; รูปที่กำลังอัปโหลดไม่เปิดกลับเองเมื่อผู้ใช้สั่งซ่อน; คงข้อความและรูปที่ยังไม่ส่งเพื่อเปิดกลับมาใช้ต่อ; ใส่ autocomplete off ทั้ง form/textarea และ inputMode text; กัน focus blur ระหว่าง pointer gesture บนปุ่มส่งไม่ให้แถบ/ปุ่มขยับก่อนปล่อยนิ้วบน WebKit
  - `reply-composer.css` — แก้: แถบพิมพ์หนึ่งแถวสูงประมาณ 56–60px ช่องข้อความสูงคงที่ 44px แบบโค้งพื้นอ่อนและฟอนต์มือถือ 17px, ปุ่มสัมผัส 44px, แผงแนบรูป/สติกเกอร์วางซ้อนเหนือช่องพิมพ์โดยไม่เพิ่มความสูง composer
- **โฟลเดอร์ `frontend/src/app/`**
  - `mobile.css` — แก้: ซ่อนเมนูล่างและ page header ขณะ keyboard เปิดหรือ composer กำลังใช้งาน/เปิดแผงเลือก และคืนเมนูเมื่อพับ composer
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build ใหม่บน local Docker Desktop เป็น `front-20261110065307-093cb09de42e` พร้อม runtime/static/metadata/checksum
- **ผลตรวจสอบ:** frontend lint/build/TypeScript ผ่าน; npm test 8/8 ผ่าน; ตรวจ checksum 2,107 ไฟล์และ metadata Linux/amd64 build_location local ผ่าน ไม่มี .env ใน release; Chrome/WebKit ผ่าน 10 กรณี (iPhone จำลอง 430x739, จอเล็ก 375x667, แนวนอน 932x430, tablet 834x1112 และ desktop 1440x900 ต่อ browser) ตรวจแถบแถวเดียว/ช่อง 44px, ซ่อน navigation, จำลอง keyboard/offset, ปุ่มบวก/สติกเกอร์/รูป, focus/blur/Escape, แตะแชทพับทั้งหมด, spacebar ในข้อความยังพิมพ์ช่องว่างตามปกติ, พับระหว่าง upload แล้วไม่เด้งกลับ, เปิดรูปเดิมกลับมา, ส่งข้อความ/รูป/สติกเกอร์ผ่าน API จำลอง และ Ctrl+Enter บนจอใหญ่; ไม่มี request ถึง LINE หรือ backend จริง
- **ข้อจำกัด:** ทดสอบด้วย browser headless และจำลอง visual viewport ไม่ใช่ iPhone จริง; แถบ Password/บัตร/ปุ่มพับ keyboard ที่ iOS แสดงเป็น UI ของ Safari/ระบบ ไม่สามารถสั่งซ่อนด้วย CSS ของเว็บได้ ตั้ง autocomplete off ให้ช่องแชทแล้วโดยไม่ปิดการ pinch zoom หรือการพิมพ์ปกติ อ้างอิง https://developer.apple.com/documentation/uikit/uiresponder/inputassistantitem และ https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes/autocomplete
- **ภาพและตัวตรวจ:** `data/compact-composer-check/` ถูก ignore; iphone-composer.png, iphone-typing.png และ iphone-tools.png เป็น preview ของ release ใหม่; คงงานชื่อ/Note และไฟล์ prompts ที่แก้ไว้ก่อนหน้า
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy

## 2026-11-10 13:35 — ย่อหัวแถว เพิ่มชื่อลูกค้าใน Alex และ Note พร้อมประวัติ
- **คำอธิบาย:** เอาหัวแถวรายชื่อลูกค้าและแชทที่ขีดฆ่าออก รวม footer ทั้งสองด้าน ย้ายชื่อลูกค้าและป้าย LINE OA ไปหัวด้านขวาแทนหัวข้อร่าง เพิ่มการตั้งชื่อใน Alex/reset กลับชื่อ LINE และแท็บ Note ต่อ LINE user ซึ่งเพิ่มโน้ตพร้อมรูปได้หลายรายการ แก้ไขได้และมีประวัติผู้สร้าง/ผู้แก้พร้อมวันเวลา
- **โฟลเดอร์ `backend/src/models/`**
  - `customer_notes.rs` — เพิ่ม: DTO ชื่อและโน้ต ตรวจ expected_version/request UUID/text/รูป JPEG-PNG สูงสุด 5 รูป รูปละ 2 MB พร้อมปฏิเสธข้อมูลผู้สร้าง/ผู้แก้ที่ client พยายามส่งเอง
  - `mod.rs` — แก้: ลงทะเบียน module
- **โฟลเดอร์ `backend/src/db/`**
  - `customers.rs` — แก้: เก็บ customer_aliases แยกจากชื่อ LINE/OA ใช้ optimistic versions และบันทึก actor/time; reset ล้างเฉพาะ custom_name
  - `customer_notes.rs` — เพิ่ม: customer_notes แยกตามร้าน/ช่องทาง/LINE user, เพิ่มแบบ idempotent, แก้แบบ CAS, เก็บ history ทุกเวอร์ชันพร้อม actor ID/ชื่อ/เวลา/ข้อความ/รูป; เรียงตามวันสร้างล่าสุดและ paginate 50 รายการ; รูปเก็บ GridFS แบบถาวรและเปิดผ่าน authenticated endpoint ที่ตรวจขอบเขตลูกค้า/ร้าน/ช่องทาง
  - `indexes.rs` — แก้: เพิ่ม unique alias และ note request ID รวม index ลำดับโน้ตตามลูกค้า
  - `mod.rs` — แก้: ลงทะเบียน module
- **โฟลเดอร์ `backend/src/routes/`**
  - `customer_notes.rs` — เพิ่ม: GET/POST /api/line/users/{user_id}/alias, GET/POST notes และ notes/{note}, GET note-images/{image}; ใช้ actor จาก session/Bearer ฝั่ง server และ cache รูปแบบ private/no-store
  - `admin.rs` — แก้: ตกแต่งรายชื่อด้วย custom_name/name_version และคง line_display_name เพื่อค้นหา/reset ได้ โดยการเปลี่ยนชื่อ OA ไม่ทับ alias
  - `mod.rs` — แก้: ลงทะเบียน routes ภายใต้ admin authorization/CSRF middleware เดิม และกำหนด body limit สำหรับรูป
- **โฟลเดอร์ `backend/src/`**
  - `tests_customer_notes.rs`, `tests.rs` — เพิ่ม | แก้: ทดสอบ validation/auth, alias/reset และ OA เปลี่ยนชื่อ, optimistic conflicts, note retries, private/persistent photos, ประวัติผู้แก้, isolation ข้ามลูกค้า/ช่องทาง และ pagination กับ MongoDB จริง
- **โฟลเดอร์ `frontend/src/components/`**
  - `admin-workspace.tsx` — แก้: ลบหัวแถว/footers ตามภาพ ย้ายชื่อลูกค้าไปด้านขวา เพิ่มปุ่มดินสอในรายชื่อและแท็บ Note, ค้นหาชื่อ LINE หรือชื่อที่ตั้งเองได้; ป้องกัน poll รุ่นเก่าทับชื่อที่เพิ่งบันทึก
  - `customer-name-dialog.tsx` — เพิ่ม: หน้าตั้งชื่อ/reset มีชื่อ LINE ปัจจุบันแสดงให้ตรวจ และเก็บข้อความที่พิมพ์ไว้เมื่อมีแอดมินแก้พร้อมกัน
  - `customer-notes.tsx`, `customer-notes.css` — เพิ่ม: เพิ่มโน้ตด้วยปุ่มบวก แนบ/ดูรูป แก้ข้อความ/รูป ดูประวัติ และโหลดรายการเก่า; วันที่ dd/mm/yyyy HH:mm เวลาไทย, เก็บข้อความค้างเมื่อสลับแท็บ, retry ไม่สร้างซ้ำ, แจ้ง conflict ใกล้ช่องแก้ และ merge ตามเวอร์ชัน/วันสร้างเพื่อกันผลตอบกลับเก่าทับข้อมูลใหม่
- **โฟลเดอร์ `frontend/src/lib/` และ `frontend/src/app/`**
  - `api.ts` — แก้: เพิ่ม types ของ alias/โน้ต/รูป/ประวัติ
  - `layout.tsx` — แก้: โหลด customer-notes.css
- **โฟลเดอร์ `dist/line-linux/` และ `dist/frontend-linux/`**
  - Linux releases — แก้: build บน local Docker Desktop เป็น backend `rel-20261110062300-16c1c07c8f6c` และ frontend `front-20261110063214-f43cea511f31` พร้อม binary/standalone/runtime/static/metadata/checksums
- **ผลตรวจสอบ:** cargo fmt --check และ cargo clippy -- -D warnings ผ่าน; cargo test ผ่าน 33 tests โดย 20 Mongo integration tests ไม่รันในชุดปกติ; รัน integration ใหม่ mongo_customer_aliases_and_notes ผ่านกับ MongoDB 7 แบบ disposable บน local แล้วหยุดและลบ test container; frontend lint/build/TypeScript และ npm test 8/8 ผ่าน; ตรวจ backend checksum และ frontend 2,107 checksums พร้อม metadata build_location local Linux/amd64 ไม่มี .env ใน frontend release
- **Browser checks:** Chrome/WebKit ผ่าน 6 กรณี (desktop 1920x1080, iPhone จำลอง 430x739 และจอเล็ก 375x667 ต่อ browser) ตรวจหัวแถว/footers, ตั้งชื่อ/reset, refresh และ stale poll ไม่ทับชื่อใหม่, Note พร้อมรูป, retry หลังจำลองเครือข่ายขาดไม่ซ้ำ, เรียงล่าสุด, reload แล้วยังอยู่, ประวัติพร้อมผู้แก้/เวลา, แก้พร้อมกันคงข้อความค้าง, แยกลูกค้า และ mobile state; API browser ใช้ fixture ไม่มีข้อมูลส่งเข้า LINE หรือ production
- **ภาพและตัวตรวจ:** `data/customer-notes-check/` ถูก ignore; customer-notes.png และ phone-notes.png แสดง UI ใหม่; คงไฟล์ prompts ที่ผู้ใช้แก้ไว้เดิม
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main เพื่ออัปเดตทั้ง backend/frontend; ยังไม่ได้ commit/push/deploy

## 2026-11-10 12:13 — แก้จอ iPhone เลื่อนหลุดและย่อช่องตอบแชท
- **คำอธิบาย:** ผู้ใช้พบเมนูล่างเลื่อนขึ้นจนเกิดพื้นที่ว่าง และ iPhone ซูมตอนแตะช่องพิมพ์ ต้องการตัวอักษรมือถือใหญ่ขึ้น 1px ปุ่มแนบอยู่เหนือข้อความและไม่มีกรอบ พร้อมเอาคำอธิบายที่ขีดฆ่าออก
- **โฟลเดอร์ `frontend/src/app/`**
  - `mobile.css` — แก้: ตรึง shell ตามพื้นที่มองเห็น ล็อก document เฉพาะหน้าทำงานให้เลื่อนภายใน panel, กำหนดเมนูล่างชิดขอบจอ และปรับ dialog ให้ตามตำแหน่ง visual viewport; เพิ่ม --mobile-text-increment: 1px เฉพาะ breakpoint มือถือ
  - `globals.css`, `typography.css`, `workspace-layout.css` — แก้: ใช้ font-size ร่วมกับ mobile increment เพื่อเพิ่มข้อความ 1px บนมือถือโดยค่า desktop เดิมไม่เปลี่ยน
  - `layout.tsx` — แก้: โหลด CSS ของ composer ใหม่
- **โฟลเดอร์ `frontend/src/components/`**
  - `admin-workspace.tsx` — แก้: ติดตาม VisualViewport resize/scroll และ offsetTop เพื่อคง layout เมื่อ browser chrome หรือ keyboard เปลี่ยน; ไม่บังคับ layout ระหว่าง pinch zoom; เอาข้อความใต้ชื่อลูกค้าและ notice ลูกค้าซ้าย/แอดมินขวา/ติดต่อผู้ดูแลออก และถอด state ที่ใช้เฉพาะ notice เดิม
  - `reply-composer.tsx` — แก้: ย้ายปุ่มสติกเกอร์และแนบรูปขึ้นเหนือช่องพิมพ์ ใช้ SVG ไอคอนพร้อม accessible labels, ลด textarea เป็นหนึ่งแถว, ย่อ placeholder และเอาคำอธิบายท้ายกล่องออก โดยการส่ง/การส่งซ้ำ/แนบรูปเดิมยังใช้ API เดิม
  - `reply-composer.css` — เพิ่ม: toolbar แนวนอนไม่มีกรอบ และช่องตอบมือถือสูงคงที่ 44px เลื่อนข้อความภายในได้; ใช้ตัวอักษร 17px ที่มี specificity สูงกว่ากฎ desktop เดิม 12px โดยยังอนุญาต pinch zoom
  - `date-input.css`, `inquiry-list.css`, `invoice.css`, `order-cart.css`, `rental-summary.css`, `workspace-sidebar.css` — แก้: font-size ใช้ mobile increment เพื่อเพิ่มทุกส่วนของแอปตาม breakpoint เดียวกัน
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build บน local Docker Desktop เป็น `front-20261110050730-f2f22b0f799d` พร้อม runtime/static/metadata/checksum
- **ผลตรวจสอบ:** local Linux frontend lint/build/TypeScript ผ่าน; npm test ผ่าน 8/8; ตรวจ checksum 2,107 ไฟล์ Linux/amd64 build_location local และไม่มี .env ใน release; Chrome/WebKit ผ่าน 12 กรณี 6 viewport ต่อ browser ครอบคลุม iPhone/จอยาว/จอเล็ก/แนวนอน/tablet/desktop ตรวจ 70 ลูกค้าและ 80 ข้อความ การเลื่อน document, เมนูชิดขอบ, browser chrome resize, จำลอง visual keyboard/offset/zoom, textarea 44px คงเดิมเมื่อพิมพ์หลายบรรทัด, computed mobile fonts +1px, ปุ่มไร้กรอบอยู่เหนือช่องพิมพ์, ข้อความถูกลบ, สลับ panel คงข้อความค้าง, ส่งข้อความและ Ctrl+Enter ผ่าน API จำลอง, ปุ่มสติกเกอร์และตัวเลือกไฟล์; ตรวจเพิ่มใบสรุปบน iPhone จำลองทั้งสอง browser ว่าวันที่ถูกต้อง ฟอนต์ช่องกรอก 17px และ dialog ตาม visual viewport เมื่อ keyboard เปิด
- **ขอบเขตการตรวจ:** ใช้ Chrome/WebKit แบบ headless และจำลอง keyboard/viewport ไม่ใช่การทดสอบบน iPhone จริง; ทุก API ใช้ fixture ไม่มีข้อความหรือสถานะอ่านถูกส่งไป LINE จริง; git diff --check ผ่านเฉพาะไฟล์งาน frontend/dist/revising-log โดยคงไฟล์ prompts ที่ผู้ใช้แก้ไว้เดิม
- **ภาพและตัวตรวจ:** `data/iphone-chat-check/` ถูก ignore; `iphone-chat.png` แสดง layout ใหม่ และ `iphone-keyboard-layout.png` แสดงพื้นที่จำลองเหนือ keyboard
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat เพื่อส่ง source/release แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy ในงานนี้

## 2026-11-10 11:11 — ใช้วันที่ dd/mm/yyyy ทั้งแอป
- **คำอธิบาย:** ผู้ใช้ต้องการวัน/เดือน/ปีแบบตัวเลขทั้งแอป รวมการแสดงผลและช่องกรอก ใช้ปี ค.ศ. เลขอารบิก และเขตเวลา Asia/Bangkok โดยข้อมูลวันที่ที่ส่ง API และเก็บในระบบยังใช้ ISO ตามสัญญาเดิม
- **โฟลเดอร์ `frontend/src/lib/`**
  - `dates.ts` — เพิ่ม: formatter ส่วนกลางสำหรับ dd/mm/yyyy และ dd/mm/yyyy HH:mm พร้อม parser ตรวจวันที่จริง ปีอธิกสุรทิน และช่วงปี 1900–2199 ให้ตรง backend; วันที่ผิดหรือว่างแสดงขีดแทนโดยไม่ทำให้หน้าจอล้ม
- **โฟลเดอร์ `frontend/src/components/`**
  - `date-input.tsx`, `date-input.css` — เพิ่ม: ช่องกรอก dd/mm/yyyy พร้อมปฏิทิน native การแปลงค่าเป็น ISO และข้อความตรวจสอบภาษาไทย; รูปแบบข้อความคงเดิมแม้เบราว์เซอร์ใช้ locale อื่น
  - `admin-workspace.tsx` — แก้: รายการแชท สรุป ประวัติร่าง การอนุมัติ/ส่งบิล/ยกเลิก ใช้ formatter ส่วนกลาง และวันที่รับ–คืนในร่างอ่านอย่างเดียวไม่แสดง ISO ดิบ
  - `chat-message.tsx`, `order-cart.tsx` — แก้: วันเวลาในแชทและรายการเลือกรูปอ้างอิงใช้ dd/mm/yyyy HH:mm
  - `draft-editor.tsx` — แก้: ตัวเลือกวันรับ–คืนและช่องกรอกใช้รูปแบบใหม่; ตรวจช่องวันที่ก่อนบันทึกหรือเปิดใบสรุปเพื่อกันวันที่ผิดไม่ให้กลายเป็นข้อมูลที่ถูกบันทึก
  - `rental-checkout.tsx` — แก้: วันรับ–คืนและประวัติส่งบิลใช้ dd/mm/yyyy, วันเกิดใช้ช่องกรอกใหม่ และเวลาหมดอายุใบตรวจแสดงวันที่พร้อมเวลาไทย
- **โฟลเดอร์ `frontend/src/app/`**
  - `layout.tsx` — แก้: โหลด CSS ของช่องวันที่ใหม่
- **โฟลเดอร์ `frontend/`**
  - `package.json`, `tests/dates.test.mjs` — แก้ | เพิ่ม: npm test และ regression tests 8 กรณี ครอบคลุม Gregorian, Bangkok ข้ามวัน/ข้ามปี, timezone เครื่อง, timestamp, leap year, วันที่ผิด/ไม่ครบ และแปลงเป็น ISO
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build ใหม่บน local Docker Desktop เป็น `front-20261110040936-7e0c9e810c20` พร้อม static/runtime/metadata/checksum
- **ผลตรวจสอบ:** npm run lint, npm test (8/8), local Linux frontend build/TypeScript ผ่าน; checksum 2,107 ไฟล์และ metadata Linux/amd64 ผ่าน ไม่มี .env ใน release; browser checks Chrome/WebKit ผ่าน 4 กรณี desktop en-US/America/Los_Angeles และ phone th-TH/Asia/Bangkok ตรวจวันเวลา แก้วันที่ เลือกปฏิทิน กันวันที่ผิดก่อนบันทึก payload ISO ประวัติ รูปอ้างอิง ใบสรุปและวันเกิด ใช้ API จำลองทั้งหมด
- **ภาพและตัวตรวจ:** `data/date-format-check/` ถูก ignore; `invoice-dates.png` แสดงวันที่ในใบสรุป
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat เพื่อส่ง source/release แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy ในงานนี้

## 2026-11-10 11:01 — ย่อ sidebar เป็นแถบไอคอนพร้อมเมนูกางด้านข้าง
- **คำอธิบาย:** ผู้ใช้ต้องการ sidebar ซ้ายสุดแคบ ใช้ไอคอนแทนชื่อเมนู และแสดงเมนูย่อยเมื่อ hover แบบ LINE OA เปลี่ยนเป็น rail กว้าง 64px พร้อม flyout ซ้อนเหนือพื้นที่ทำงานโดยไม่เปลี่ยนความกว้างคอลัมน์ขณะเปิดเมนู
- **โฟลเดอร์ `frontend/src/components/`**
  - `workspace-sidebar.tsx` — เพิ่ม: sidebar ใช้ SVG ที่มีรูปแบบเดียวกัน พร้อม badge จำนวนเรื่อง เมนูแชทที่ไปยังรายชื่อ/บทสนทนา/ร่างออเดอร์ ข้อมูลพื้นที่ทำงาน และบัญชี/ออกจากระบบ; รองรับ hover, focus, click/tap, Tab/ArrowRight/ArrowDown, Escape คืน focus และคลิกภายนอกเพื่อปิด มี accessible labels และ aria-expanded/controls
  - `workspace-sidebar.css` — เพิ่ม: แถบ 64px ปุ่ม 44px ไอคอน 24px flyout มีพื้นที่เชื่อมระหว่างแถบกับเมนูเพื่อไม่หุบระหว่างเลื่อนเมาส์ เมนูด้านล่างจัดชิดขอบล่างและซ้อนเหนือ workspace โดยไม่ถูก sidebar ตัด
  - `admin-workspace.tsx` — แก้: ใช้ component sidebar ใหม่แทนเมนูข้อความเดิม ส่งข้อมูลบัญชี/จำนวนเรื่องและ callback เดิม เพิ่ม focus target ของแต่ละ panel โดยคงแชท/ร่าง/ข้อความที่พิมพ์ไว้ และถอด Next Link ที่ไม่ใช้แล้ว
- **โฟลเดอร์ `frontend/src/app/`**
  - `layout.tsx` — แก้: โหลด stylesheet sidebar หลัง typography โดยกฎซ่อน sidebar บนมือถือยังทำงานตาม breakpoint เดิม
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build ใหม่บน local Docker Desktop เป็น `front-20261110035826-64182baae8db` พร้อม static/runtime/metadata/checksum
- **ผลตรวจสอบ:** frontend lint/build/TypeScript ผ่าน; ตรวจ checksum 2,107 ไฟล์และ metadata Linux/amd64 ผ่าน ไม่มี .env ใน release; Chrome และ WebKit ผ่านรวม 12 กรณี (6 viewport ต่อ browser) ตรวจ rail 64px, hover และข้ามช่องว่างเข้า flyout, hit test ว่าเมนูอยู่เหนือ workspace, ไม่มี layout shift, ปิดเมื่อออก/คลิกภายนอก, keyboard/focus/Escape/Tab, touch บน tablet, เมนูบัญชีอยู่ในขอบจอ, panel navigation คงข้อความที่ยังไม่บันทึก, logout และ mobile navigation ใช้ API จำลองทั้งหมด ไม่ส่งข้อมูล/สถานะอ่านไป LINE จริง
- **ภาพและตัวตรวจ:** `data/sidebar-rail-check/` ถูก ignore; `hover-menu.png` แสดงเมนูกาง และ `compact-rail.png` แสดงแถบปกติ
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat เพื่อส่ง source/release แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy ในงานนี้

## 2026-11-10 09:40 — แยกหัวข้อและคำอธิบาย พร้อมลดสีข้อความรองหนึ่งเฉด
- **คำอธิบาย:** ผู้ใช้ต้องการคำอธิบาย/placeholder ขนาด 12px สีจางลงหนึ่งเบอร์ และหัวข้อใหญ่กว่าคำอธิบาย จัดหัวข้อส่วนเป็น 16px ป้ายกำกับช่องกรอก 14px และข้อความรอง/placeholder 12px โดยข้อความรองทั่วไปลด slate-500 เป็น slate-400 และ placeholder ลด slate-400 เป็น slate-300
- **โฟลเดอร์ `frontend/src/app/`**
  - `globals.css` — แก้: เพิ่ม semantic color tokens description และ placeholder ตามเฉดใหม่
  - `typography.css` — เพิ่ม: ขนาด/สีข้อความรองใน header, panel, sidebar, empty state, preview, ฟอร์ม, footer, ใบสรุปและหน้าล็อกอิน; กำหนด placeholder 12px สีอ่อนพร้อม opacity ชัดเจน และขนาดหัวข้อ/ป้ายกำกับแยกกัน โดย notice ยังคงสีสื่อความหมาย
  - `layout.tsx` — แก้: โหลด typography หลัง mobile เพื่อใช้ลำดับขนาด/สีเดียวกันทุกขนาดจอ
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build ใหม่บน local Docker Desktop เป็น `front-20261110023554-5d8c3b9438bc` พร้อม static/runtime/metadata/checksum
- **ผลตรวจสอบ:** frontend lint/build/TypeScript ผ่าน; ตรวจ checksum 2,108 ไฟล์และ metadata Linux/amd64 ผ่าน ไม่มี .env ใน release; Chrome จาก compiled release จริงผ่าน 6 viewport ตรวจ computed font/color ของคำอธิบาย 12px/slate-400, placeholder 12px/slate-300, หัวข้อส่วน 16px และ field label 14px รวมหน้าว่าง แบบฟอร์ม ใบสรุปที่ portal และหน้าล็อกอิน ข้อความแชทยังใช้ขนาด 12px ตามงานก่อนหน้าและพิมพ์ค้างได้ ตรวจ horizontal overflow ผ่าน ใช้ API จำลองทั้งหมด ภาพอยู่ใน `data/typography-hierarchy/` ที่ถูก ignore
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat เพื่อส่ง source/release จาก Windows แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy ในงานนี้ และคงการแก้ข้อความ 12px/บันทึกงานก่อนหน้าไว้ครบ

## 2026-11-10 09:28 — ปรับข้อความตัวอย่าง แชท และช่องตอบเป็น 12px ตามภาพ
- **คำอธิบาย:** ผู้ใช้ระบุขนาด 12px สำหรับข้อความตัวอย่างใต้ชื่อลูกค้า เนื้อหาข้อความในแชท และช่องพิมพ์ตอบรวม placeholder จึงปรับเฉพาะบริเวณที่วงไว้ให้ใช้ขนาดนี้ทั้ง desktop/tablet/mobile
- **โฟลเดอร์ `frontend/src/components/`**
  - `inquiry-list.css` — แก้: ขนาดข้อความตัวอย่างของการ์ดจาก 14px เป็น 12px
- **โฟลเดอร์ `frontend/src/app/`**
  - `workspace-layout.css` — แก้: กำหนด message bubble และ textarea ของช่องตอบเป็น 12px ด้วย selector ที่ครอบคลุมกฎจอใหญ่และมือถือ โดย placeholder รับขนาดจาก textarea
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build ใหม่บน local Docker Desktop เป็น `front-20261110022625-c0638d0f4f95` พร้อม static/runtime/metadata/checksum
- **ผลตรวจสอบ:** lint/build/TypeScript ผ่านใน local Linux build; ตรวจ checksum 2,108 ไฟล์และ metadata Linux/amd64 ผ่าน ไม่มี .env ใน release; WebKit จาก compiled release จริงยืนยัน computed font ของ preview ข้อความลูกค้า/แอดมิน textarea และ placeholder เท่ากับ 12px ครบ 6 viewport (1440, 1920, tablet, phone, small phone, landscape) โดย body ยังเป็น 16px พิมพ์ข้อความและ layout ไม่มี horizontal overflow ใช้ API จำลองทั้งหมด ภาพอยู่ใน `data/typography-12px/` ที่ถูก ignore
- **ขั้นตอนถัดไป:** ผู้ใช้ส่ง source/release ด้วย rungit.bat แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main; ยังไม่ได้ commit/push/deploy ในงานนี้ คง entry และงาน config OA เดิมไว้ครบ

## 2026-11-10 08:52 — เติมค่าเปิด OA sync บน Ubuntu และระบุจุดที่ยังติด
- **คำอธิบาย:** ผู้ใช้เห็นแถบยังไม่ได้เชื่อม LINE OA หลัง deploy ตรวจ server จริงพบ release ใหม่ครบและ backend รองรับ unread_source แล้ว แต่ .env ไม่มี OA_CONNECTOR_ENABLED จึงใช้ default false และ runtime ตอบ NOT_CONFIGURED ขณะ companion connector ตอบ BACKEND_UNAVAILABLE และ browser /screen ตอบ 503
- **โฟลเดอร์ `/home/ubuntu/alex-connectingpoint/` บน Ubuntu**
  - `.env` — แก้: เติม OA_CONNECTOR_ENABLED=true โดยตรวจว่าค่าอื่นทั้งหมดคงเดิม เขียนแบบ atomic และ permission 600; ไฟล์ถูก ignore ใน Git
  - `.env.before-oa-enable-20261110-085044` — เพิ่ม: สำรองไฟล์เดิมก่อนแก้ เก็บ permission 600 และตรวจว่าไม่เข้า Git
- **ผลตรวจสอบ:** backend health ปกติ API คืน 54 รายการ มีข้อมูล unread_source ครบแต่ยังไม่มีรายการที่ใช้ snapshot OA; release บน server ตรงกับ backend rel-20261109152758-6d45ca9228d0, frontend front-20261109175604-ea0aceb3acff และ connector oa-20261109152421-01c6d2488d1b ไม่ได้แก้ source หรือ build โปรแกรมใหม่
- **สถานะที่ยังเหลือ:** ค่าในไฟล์เปิดแล้ว แต่บริการต้อง deploy เพื่อโหลด env ใหม่ และ redeploy companion เพื่อเปิด browser จาก release ปัจจุบัน ใช้ sudo bash ./docker-deploy.sh --full-stack ตามด้วย sudo bash ./deploy-oa-connector.sh; หากขึ้น NEEDS_LOGIN ให้ใช้ login-oa.bat แล้วตรวจ schema/name/read จาก OA จริงต่อ
- **ข้อจำกัดการดำเนินการ:** ตรวจ sudo -n true แล้ว server ต้องใช้รหัสผ่าน จึงยัง deploy/restart container แทนผู้ใช้ไม่ได้ ไม่เปิด SSH tunnel หรือหน้าล็อกอินเอง และไม่ได้ส่งข้อความ/สถานะอ่านไปหาลูกค้า ไม่มี secret หรือค่าลูกค้าในผลคำสั่ง/บันทึก

## 2026-11-10 00:58 — ปรับหน้าหลังบ้านเต็มขอบจอและเลื่อนแต่ละคอลัมน์
- **คำอธิบาย:** ผู้ใช้ต้องการ layout เต็มจอถึงขอบล่างเหมือน LINE OA เปลี่ยนพื้นที่ทำงาน desktop ให้ยึด viewport จริง ตัดระยะรอบการ์ดด้านข้าง/ล่างและเพดานความสูงเดิม โดยคอลัมน์รายชื่อ แชท และร่างออเดอร์เลื่อนแยกกัน
- **โฟลเดอร์ `frontend/src/app/`**
  - `workspace-layout.css` — เพิ่ม: desktop app shell แบบ fixed/inset และ 100dvh, main content แบบ flex ใช้พื้นที่ที่เหลือหลัง header/notice, workspace ไม่มีขอบมนหรือ gutter และยืดถึงขอบขวา/ล่าง; กำหนด min-height:0 และ scrolling ภายใน panel รักษา footer/ช่องตอบให้เข้าถึงได้ รวม tablet สองคอลัมน์และหน้าต่างเตี้ย
  - `layout.tsx` — แก้: โหลด stylesheet ของ workspace หลัง globals และก่อน mobile เพื่อใช้พฤติกรรมมือถือ/แนวนอนที่มีอยู่
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build ใหม่บน local Docker Desktop เป็น `front-20261109175604-ea0aceb3acff` พร้อม static/runtime/metadata/checksum สำหรับ deploy Ubuntu
- **ผลตรวจสอบ:** frontend lint/build และ TypeScript ผ่านใน local Linux build; ตรวจ checksum ครบ 2,108 ไฟล์/metadata Linux/amd64 และไม่มี .env ผ่าน; git diff --check ผ่าน
- **ทดสอบหน้าจอ:** WebKit จาก compiled release จริงผ่าน 10 viewport (มือถือแนวตั้ง/แนวนอน แท็บเล็ต จอ 1440/1920 จอสูง 1600 และหน้าต่างเตี้ย) ตรวจขอบขวา/ล่างตรง viewport ไม่มี document overflow, เปลี่ยนขนาดหน้าต่างขณะมี notice หลายบรรทัดและข้อมูลยาว เลื่อนทั้งสามคอลัมน์ได้โดย footer ยังอยู่ในจอ รวมการสลับ panel/แท็บ ฟอร์ม บิล คีย์บอร์ด และข้อความที่พิมพ์ค้างไว้ ผ่านทั้งหมด ใช้ API จำลอง ไม่เปิด dev app/SSH tunnel หรือส่งสถานะอ่านไป LINE จริง
- **ภาพและตัวตรวจ:** `data/layout-fullheight-check/` ถูก ignore; `desktop-workspace.png` แสดงหน้าปกติ และ `desktop-fullheight.png` แสดงกรณีข้อมูลยาว/notice สำหรับตรวจ layout
- **ขั้นตอนถัดไป:** ผู้ใช้รัน rungit.bat เพื่อส่ง source และ release แล้วบน Ubuntu รัน cd ~/alex-connectingpoint และ bash ./deploy-all.sh main (แพ็กเกจ/deploy แบบ full-stack ใช้ precompiled release เท่านั้น) งานนี้ยังไม่ได้ commit/push/deploy และคงการแก้ไขงาน OA ที่มีอยู่ก่อนเริ่มทั้งหมด

## 2026-11-09 22:34 — เพิ่มซิงก์ชื่อและ read/unread จาก OA เมื่อโหลดข้อมูลใหม่
- **คำอธิบาย:** ผู้ใช้เปลี่ยนข้อกำหนดให้ชื่อที่ตั้งเองและสถานะอ่าน/ยังไม่อ่านอิง OA Manager และรับการเปลี่ยนจาก OA กลับมาเมื่อเปิดหน้าหรือกดโหลดข้อมูล เพิ่มทางซิงก์ผ่าน browser connector เนื่องจาก Messaging API ให้โปรไฟล์และส่งอ่านแล้วได้ แต่ไม่มี webhook แจ้งชื่อที่ตั้งใน OA หรือการสลับ read/unread จาก OA Manager
- **โฟลเดอร์ `backend/src/`**
  - `models/oa_inbox.rs`, `routes/oa_inbox.rs`, `db/oa_inbox.rs` — เพิ่ม: รับ snapshot ชื่อและสถานะอ่าน จับคู่บทสนทนา 1:1 ด้วย message ID ที่ตรง webhook แยกร้าน/ช่องทาง ตรวจ bot/input/เวลา ป้องกัน snapshot เก่าทับการอ่านใหม่ และสร้าง/อ่าน/จบงานขอรีเฟรชสำหรับตัวเชื่อม
  - `db/users.rs` — แก้: ให้ชื่อจาก OA มาก่อนชื่อโปรไฟล์แคชเมื่อเปิดตัวเชื่อม โดยไม่เขียนทับชื่อโปรไฟล์ต้นฉบับ
  - `db/inbox.rs` — แก้: เลือก unread จาก OA เมื่อครอบคลุมข้อความนั้น ระบุแหล่งสถานะ ไม่ล้างข้อความใหม่ที่อยู่นอก snapshot และให้ส่งอ่านแล้วซ้ำได้หากข้อความเดิมถูกตั้ง unread ใน OA อีกครั้ง
  - `db/indexes.rs`, `db/mod.rs`, `models/mod.rs`, `routes/mod.rs` — แก้: ลงทะเบียนโมดูล/route และ index งานซิงก์พร้อม TTL หนึ่งวัน; browser ใช้ admin session/X-Admin-Request ส่วน connector รับ Bearer token เท่านั้น
  - `tests_line.rs`, `tests_oa_sync.rs` — แก้: เพิ่ม integration test ชื่อ/read/unread สลับกลับไปมา retry ข้อความเดิม tenant/auth/input/queue และ race ขณะมีข้อความใหม่ รวม millisecond เดียวกัน
- **โฟลเดอร์ `connectors/line-oa/`**
  - `src/adapter.mjs` — แก้: decode ชื่อ/nickname และ unread แบบตรวจชนิดข้อมูล ไม่ตีความค่าที่หายเป็นอ่านแล้ว พร้อม anchor ข้อความรูป/สื่อของลูกค้า
  - `src/index.mjs` — แก้: ส่ง snapshot ชื่อ/read state และตรวจคิวรีเฟรชทุกวินาที รักษาการอ่าน OA แบบ GET/HEAD เท่านั้น เพิ่ม schema diagnostics จาก response ของ OA UI เฉพาะชื่อฟิลด์/ชนิดข้อมูล และไม่กลบสาเหตุ OA ผิดพลาดด้วย backend error
  - `test/adapter.test.mjs` — แก้: ตรวจ nickname เปลี่ยน/ถูกลบ read/unread ทั้งสองทาง สื่อ และ schema ที่ไม่ชัดเจน
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx` — แก้: ขอรอบซิงก์เมื่อเปิดหน้า/โหลดข้อมูลใหม่ แสดงกำลังซิงก์และรอผลก่อนอ่านรายชื่อ โหลดหน้าที่เคยเปิดเพิ่มใหม่ด้วย แจ้งเมื่อใช้ข้อมูลเก่า แยกชื่อสถานะอ่าน OA/Alex และให้กดเปิดแชทเดิมเพื่อส่งอ่านแล้วอีกครั้ง
  - `lib/api.ts` — แก้: เพิ่มชนิดข้อมูลแหล่ง unread
- **โฟลเดอร์ `docs/`**
  - `line-oa-auto-sync.md`, `step-1-line-inquiry.md` — แก้: อธิบายพฤติกรรมใหม่ API ข้อจำกัด schema และขั้นตอนเปิดตัวเชื่อม แม้ตอบลูกค้าจาก Alex เป็นหลัก
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — แก้: local Linux/amd64 release `rel-20261109152758-6d45ca9228d0`
  - `frontend-linux/` — แก้: local standalone release `front-20261109152751-8066fd8114ba`
  - `oa-connector-linux/` — แก้: local compiled connector release `oa-20261109152421-01c6d2488d1b`
- **ผลตรวจสอบ:** cargo fmt/check และ clippy --all-targets -D warnings ผ่าน; cargo test -- --include-ignored ผ่านครบ 50 กรณีด้วย MongoDB แยกสำหรับทดสอบ; frontend lint/build ผ่าน; connector unit/control 6 กรณีผ่าน; ตรวจ backend checksum, frontend 2,108 checksum และ connector 182 checksum/tag ผ่าน ไม่มี .env ใน releases
- **ทดสอบระบบจำลอง:** WebKit ผ่าน 6 ขนาดจอและกรณีกดโหลดข้อมูลเพื่อเปลี่ยนชื่อ/read/unread/เปิดแชทเดิมอีกครั้ง; compiled Linux connector กับ Chromium จริงผ่านคิวรีเฟรช เปลี่ยน/ลบ nickname สลับ read flags และปฏิเสธ schema ไม่รองรับ/การเขียนผ่าน OA UI โดย mock LINE responses ทั้งหมด ไม่ส่งข้อความหรือสถานะอ่านให้ลูกค้าจริง ตัวตรวจอยู่ใน `data/oa-inbox-check/` ที่ถูก ignore และหยุด container/network ทดสอบแล้ว
- **ข้อจำกัดก่อนเปิดใช้จริง:** ตรวจ Ubuntu แบบอ่านอย่างเดียวพบ OA_CONNECTOR_ENABLED ยังไม่ได้ตั้ง (default false), backend ตอบ NOT_CONFIGURED, schema มีเพียง csrf/bots และ browser control /screen ตอบ 503 จึงยังไม่ยืนยัน schema ชื่อ/unread ของบัญชี OA จริง contract ของ adapter ยังต้องตรวจหลัง deploy ห้ามถือว่าทดสอบ production ผ่าน ไม่มีการแก้ .env/server config หรือ commit/push/deploy ในงานนี้
- **ขั้นตอนถัดไป:** ผู้ใช้ส่ง source/release ด้วย rungit.bat; บน Ubuntu คงค่า .env เดิมแล้วตั้ง OA_CONNECTOR_ENABLED=true, รัน deploy-all.sh main และ sudo bash ./deploy-oa-connector.sh; ใช้ login-oa.bat หากต้องล็อกอินใหม่ แล้วตรวจ schema/เปลี่ยนชื่อและ read/unread จริง การ deploy ต้องใช้ sudo password ซึ่ง SSH ของ agent ยังไม่มี

## 2026-11-09 21:24 — เอาสถานะใต้การ์ดแชทออกครบและตรวจ release ก่อน deploy
- **คำอธิบาย:** ผู้ใช้ขอข้อความตัวอย่างแทนรหัสเรื่อง ไม่แสดงสถานะใต้การ์ด และจุดเขียวอ่านแชท โดยยืนยันในงานนี้ให้อิงการอ่านใน Alex เป็นหลักและส่งอ่านแล้วไป LINE พบว่าโค้ดเดิมรองรับ preview/unread/markAsRead แล้ว แต่ยังเหลือบรรทัดร่างและอนุมัติร่างใต้การ์ด จึงนำออกให้ครบตามคำขอ
- **โฟลเดอร์ `frontend/src/components/`**
  - `admin-workspace.tsx` — แก้: ลบบรรทัดร่าง v และป้ายอนุมัติร่างแล้วใต้การ์ดรายชื่อลูกค้า คงข้อความล่าสุดสองบรรทัดและจุดเขียวด้านขวา
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone release — แก้: build บน local Docker Desktop เป็น `front-20261109142241-4f7c4ec354ff` พร้อม runtime/static/checksum/metadata สำหรับ Ubuntu โดยไม่ compile บน server
- **ผลตรวจสอบ:** `cargo fmt --check`, `cargo clippy -- -D warnings` และ `cargo test -- --include-ignored` ผ่านครบ 49 กรณีเมื่อใช้ MongoDB ทดสอบแยกบน local และหยุด container หลังทดสอบ; frontend lint/build ผ่านใน local Linux build; ตรวจ checksum frontend 2,108 ไฟล์และ metadata Linux/amd64 ผ่าน ไม่มีไฟล์ .env ใน release
- **ทดสอบหน้าจอ:** WebKit ผ่าน 6 ขนาดจอด้วย compiled release ใหม่และ API จำลอง ตรวจทั้งรายการมีร่างและมี order ว่าไม่มีบรรทัดสถานะใต้การ์ด รวมถึง preview จุดเขียว การส่งอ่านแล้วเมื่อเปิดแชท และการคงจุดเขียวเมื่อมีข้อความใหม่แต่แชทมือถือยังปิดอยู่ โดยไม่ส่งสถานะอ่านไปหาลูกค้าจริง
- **ตรวจ Ubuntu แบบอ่านอย่างเดียว:** Git บน server อยู่ที่ `095dc586` และ checksum release เดิมตรงกับ local แต่ backend ที่กำลังรันยังไม่คืน preview/unread ทั้ง 50 รายการ จึงต้อง deploy runtime ใหม่; backend release ที่พร้อมใช้คือ `rel-20261109135848-567eed1a6d1d` ไม่มีการแก้ backend ในงานนี้
- **ขั้นตอนถัดไป:** ผู้ใช้รัน `rungit.bat` เพื่อส่ง source/release ใหม่ แล้วบน Ubuntu รัน `cd ~/alex-connectingpoint && bash ./deploy-all.sh main`; agent ยังไม่ได้ commit/push/deploy เนื่องจากไม่ได้รับคำสั่ง Git และ `sudo -n` บน server ต้องใช้รหัสผ่าน

## 2026-11-09 15:44 — เพิ่มข้อความตัวอย่างและจุดเขียวอ่านแชท พร้อมส่งอ่านแล้วไป LINE
- **คำอธิบาย:** ผู้ใช้ขอให้การ์ดลูกค้าแสดงข้อความแทนรหัสเรื่อง ลบป้ายรับเรื่อง และมีจุดเขียวด้านขวา โดยยืนยันให้สถานะอิงการอ่านใน Alex เป็นหลักและส่งอ่านแล้วไป LINE ไม่ต้องซิงก์การเปิดอ่านจาก OA Manager กลับมา ใช้แนวทาง skill `ui-ux-pro-max` ตรวจหน้าจอและข้อความสำหรับ screen reader
- **โฟลเดอร์ `backend/src/`**
  - `db/inbox.rs` — เพิ่ม: ดึงข้อความล่าสุดของแต่ละเรื่องแบบ batch จากข้อความขาเข้า/ขาออก แสดงข้อความจริงหรือประเภทสื่อ จำกัดความยาว ระบุคำตอบแอดมินด้วยคุณ และตัดข้อความที่ unsend ตั้งแต่ tombstone ก่อน worker ลบเนื้อหา; คำนวณ unread เทียบจุดอ่านร่วมของบทสนทนา และส่ง `POST /v2/bot/chat/markAsRead` ด้วย token ที่เก็บฝั่ง server
  - `models/inbox.rs`, `routes/inbox.rs` — เพิ่ม: API `POST /api/inquiries/{id}/read` รับเฉพาะ ID ข้อความที่โหลดเห็นแล้ว ตรวจรูปแบบและสิทธิ์แยกร้าน/ช่องทาง ไม่ข้ามไปอ่านข้อความใหม่ที่เข้าทีหลัง และคืนสถานะ LINE/ข้อความที่ยังไม่อ่าน
  - `db/mod.rs`, `models/mod.rs`, `routes/mod.rs` — แก้: ลงทะเบียนโมดูลและ route ภายใต้ admin session/token และการตรวจ X-Admin-Request เดิม
  - `routes/admin.rs`, `routes/users.rs`, `db/events.rs`, `db/drafts.rs` — แก้: เพิ่ม preview/unread ในรายการและซ่อน read token จาก API/ข้อมูลที่ส่งต่อ AI โดยคง context hash เดิม เพื่อไม่ทำให้ร่างค้างเปลี่ยนสถานะเพียงเพราะซ่อน token
  - `tests_line.rs` — แก้: เพิ่ม LINE mock และ integration test ครอบคลุมสถานะอ่าน ข้อความที่เข้าระหว่างอ่าน ขอบเขต timestamp เดียวกัน การ retry/idempotency ความล้มเหลวของ LINE ข้อความเก่าไม่มี token การซ่อน token และ unsend รวมถึง tenant/auth/input validation; unit test ใน `db/inbox.rs` ตรวจประเภทสื่อ ข้อความขาออก และความยาวภาษาไทย
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx` — แก้: แสดงข้อความล่าสุดสองบรรทัดแทนรหัสเรื่อง เอาป้ายรับเรื่องออก เพิ่มจุดเขียวด้านขวาพร้อมข้อความสำหรับ screen reader ค้นจากข้อความตัวอย่างได้ และส่งอ่านแล้วเฉพาะแชทที่แสดงอยู่ ไม่ทำเครื่องหมายขณะหน้าเว็บซ่อนหรือแถบแชทมือถือปิด
  - `components/inquiry-list.css`, `app/globals.css` — เพิ่ม/แก้: รูปแบบข้อความตัวอย่างตัดบรรทัดและจุดเขียวด้วย semantic success color รองรับจอแคบ
  - `lib/api.ts` — แก้: เพิ่มข้อมูล preview/unread/ข้อความขาเข้าล่าสุด และคำอธิบายกรณีส่งสถานะอ่านไป LINE ไม่สำเร็จ
- **โฟลเดอร์ `docs/`**
  - `step-1-line-inquiry.md` — แก้: อธิบาย API อ่าน เงื่อนไข Chat ของ LINE ความหมายสถานะ กรณีไม่มี token และขอบเขตการอ่านที่อิง Alex ตามที่ผู้ใช้ยืนยัน
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — แก้: Linux/amd64 release `rel-20261109084235-567eed1a6d1d` พร้อม checksum/metadata
  - `frontend-linux/` — แก้: Linux standalone release `front-20261109083832-8838dd8c765c` พร้อม static/runtime/checksum
- **ผลตรวจสอบ:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test -- --include-ignored` ผ่านครบ 49 กรณี; frontend lint ผ่านและ lint/build ใน local Docker ผ่าน; ตรวจ backend ELF/checksum และ frontend 2,108 checksum กับ metadata local Linux/amd64 ผ่าน ไม่มี `.env` ใน release
- **ทดสอบหน้าจอ:** ใช้ไฟล์ Linux release จริงกับ WebKit ผ่าน 6 ขนาดจอ รวม iPhone 15 Pro Max แนวตั้ง/แนวนอน ตรวจ preview และป้ายที่เอาออก จุดเขียวหายเมื่อเปิดแชท ข้อความใหม่ยังมีจุดเขียวขณะอยู่ในแถบออเดอร์ และไม่ล้างสรุป/ข้อความที่พิมพ์ไว้ ตัวตรวจและภาพอยู่ใน `data/mobile-responsive-check/` ที่ถูก ignore ใช้เฉพาะ API จำลอง ไม่เปิด dev app/SSH tunnel หรือส่งสถานะอ่านให้ลูกค้าจริงระหว่างทดสอบ
- **ตรวจ LINE จริงแบบอ่านอย่างเดียว:** OA Arisa เปิด Chat และ markAsReadMode=manual ข้อความล่าสุดที่ตรวจ 91 ข้อความมี read token ครบ โดยไม่แสดงค่า token และไม่แก้ config/OA browser connector ยังต้อง deploy บน Ubuntu เพื่อทดสอบการอ่านกับ LINE จริง

## 2026-11-09 15:25 — แก้รายชื่อลูกค้าไม่อัปเดตอัตโนมัติและตรวจพอร์ต Caddy
- **คำอธิบาย:** ผู้ใช้พบข้อมูลไม่ตรงกับ OA Manager และยืนยันว่าจะตอบจากหน้าหลังบ้านเป็นหลัก พบว่ารายชื่อโหลดเฉพาะตอนเข้าหน้า/กดรีเฟรช เรียงตามเวลาเปิดเรื่อง และการ์ดไม่ได้แสดงเวลาข้อความล่าสุด จึงทำให้แชทที่รับข้อความแล้วดูเก่าหรือไม่ปรากฏจนกว่าจะโหลดหน้าใหม่
- **โฟลเดอร์ `backend/src/`**
  - `db/inquiries.rs` — แก้: เก็บ `last_activity_at` แบบเพิ่มขึ้นเท่านั้นเมื่อรับข้อความ เติมค่ารายการเดิมจากข้อความขาเข้า/ขาออกที่บันทึกสำเร็จ แยกร้าน/ช่องทาง และเรียง/แบ่งหน้าตามเวลาแชทพร้อม cursor snapshot
  - `db/indexes.rs` — แก้: เพิ่ม index รายการตามเวลาแชทและเรียกเติม metadata ตอนเริ่มระบบแบบทำซ้ำได้
  - `db/outbound.rs`, `db/oa_sync.rs` — แก้: อัปเดตเวลาแชทหลังส่งจากหลังบ้านหรือรับข้อความจากตัวเชื่อม โดย retry ที่สำเร็จแล้วไม่ส่งข้อความซ้ำ
  - `routes/admin.rs` — แก้: รับ `before_at` คู่กับ cursor ตรวจรูปแบบผ่าน AppError และตั้งเวลาเริ่มต้นเมื่อเปิดเรื่องเอง
  - `tests.rs`, `tests_line.rs`, `tests_oa_sync.rs` — แก้: ตรวจลำดับข้อความที่มาสลับเวลา การเติม metadata ย้อนหลังจากสามแหล่ง การแบ่งหน้าเมื่อ cursor มีข้อความเพิ่ม การแยกร้าน/ช่องทาง และเวลาแชทหลังแอดมินตอบ
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx` — แก้: ตรวจรายชื่อและแชททุก 10 วินาทีขณะหน้าเว็บแสดงอยู่ โหลดทันทีเมื่อกลับมาที่หน้า เก็บรายการที่โหลดเพิ่มไว้ ไม่ซ้ำ ไม่เปลี่ยนลูกค้าที่เลือก และแสดงเวลา/เรียงจากข้อความล่าสุด
  - `lib/api.ts` — แก้: เพิ่มฟิลด์เวลาแชทใน Inquiry
- **โฟลเดอร์ `docs/`**
  - `step-1-line-inquiry.md` — แก้: อธิบายลำดับ/ตัวแบ่งหน้า การอัปเดตอัตโนมัติ และความต่างของข้อความที่ตอบจากหลังบ้านกับ OA Manager
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — แก้: Linux/amd64 release `rel-20261109082306-3b8534da2f75` พร้อม checksum และ metadata
  - `frontend-linux/` — แก้: Linux standalone release `front-20261109082408-5944cdfde4a0` พร้อม static/runtime และ checksum
- **ตรวจ Ubuntu/Caddy:** อ่านทั้ง Caddyfile และ config ที่ใช้งานจริง พบ webhook ไป `127.0.0.1:18080` และหน้า admin/API ผ่าน frontend ที่ `127.0.0.1:18081` ถูกต้อง Health ทั้งสามเส้นทางผ่าน; API ภายในและ HTTPS เห็น inquiry 34 รายการตรงกัน; LINE webhook เปิดใช้งานและ URL ถูกต้อง; ส่ง webhook ว่างพร้อมลายเซ็นที่ถูกต้องทั้งภายในและ HTTPS ได้ HTTP 200 accepted=0 ไม่สร้างข้อความลูกค้าทดสอบ ไม่แก้ Caddy หรือเปิดตัวเชื่อม OA เพิ่ม รุ่นบน server ขณะตรวจยังไม่มี `last_activity_at` จึงต้อง deploy release ใหม่
- **ผลตรวจสอบ:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test -- --include-ignored` ผ่านครบ 47 กรณี รวม MongoDB ชั่วคราวที่ปิดหลังทดสอบ; frontend lint ผ่าน และ lint/build ผ่านใน `build-all.bat` บน local Docker; ตรวจ ELF backend, metadata Linux/amd64 build_location=local และ checksum frontend 2,108 ไฟล์ผ่าน ไม่มี `.env` ใน release
- **ทดสอบหน้าจอ:** ใช้ไฟล์ Linux release จริงกับ WebKit ผ่าน 6 ขนาดจอ รวม iPhone 15 Pro Max และทดสอบเพิ่มว่าลูกค้าใหม่/ข้อความใหม่เข้าหน้าจอโดยไม่กดรีเฟรช เรียงตามเวลาแชท โหลดเมื่อ focus กลับมา รายการไม่ซ้ำ และไม่ล้างลูกค้าที่เลือก ข้อความรอส่ง หรือสรุปที่ยังไม่บันทึก ใช้ข้อมูลจำลองผ่าน request interception ใน `data/mobile-responsive-check/` ซึ่งถูก ignore ไม่เปิดแอป dev หรือ SSH tunnel และไม่ส่งข้อความหาลูกค้าจริง

## 2026-11-09 11:11 — ปรับหน้าหลังบ้านสำหรับ iPhone 15 Pro Max
- **คำอธิบาย:** หน้าจอมือถือเดิมวางรายชื่อลูกค้า แชท และฟอร์มเรียงยาว จึงปรับเป็นพื้นที่ทำงานทีละส่วน พร้อมปุ่มสลับด้านล่างและรองรับทั้งแนวตั้ง/แนวนอน โดยใช้แนวทางจาก skill `ui-ux-pro-max`
- **โฟลเดอร์ `frontend/src/app/`**
  - `mobile.css` — เพิ่ม: layout มือถือ/โทรศัพท์แนวนอน แถบลูกค้า–แชท–ออเดอร์ รายชื่อลูกค้าแนวตั้ง ฟอร์มหนึ่งคอลัมน์ในจอแคบ ปุ่มอย่างน้อย 44px ช่องกรอก 16px การตัดบรรทัด safe-area รอบจอ พื้นที่เลื่อนแต่ละส่วน และหน้าสรุปบิลเต็มจอ
  - `layout.tsx` — แก้: โหลด CSS มือถือและตั้ง viewport ผ่าน Next.js API โดยรองรับ `viewport-fit=cover` และไม่ปิดการซูม
- **โฟลเดอร์ `frontend/src/components/`**
  - `admin-workspace.tsx` — แก้: เพิ่มปุ่มสลับพื้นที่ทำงานพร้อมสถานะสำหรับ screen reader เลือกลูกค้าแล้วเปิดแชท เก็บ component เดิมไว้เมื่อสลับแถบเพื่อรักษาข้อความ/ฟอร์มที่ยังไม่บันทึก และใช้ VisualViewport ปรับความสูงรวมถึงหน้าสรุปบิลเมื่อคีย์บอร์ดเปิด
  - `draft-editor.tsx` — แก้: ปิดการลากหน้าสรุปบิลบนโทรศัพท์ทั้งแนวตั้งและแนวนอน
- **โฟลเดอร์ `dist/frontend-linux/`**
  - Linux standalone — แก้: release `front-20261109040935-54be574771d2` พร้อม static files, runtime dependencies, checksum และ metadata
- **ผลตรวจสอบ:** `npm run lint` และ `npm run build` ผ่านบน Windows และผ่านอีกครั้งใน `build-frontend.bat` บน local Docker; ตรวจ checksum 2,108 ไฟล์, viewport metadata และไม่มี `.env` ใน release ผ่าน; ตรวจ `git diff --check` ผ่าน ไม่ได้แก้ backend จึงไม่ต้อง build/test Rust
- **ทดสอบหน้าจอ:** ใช้ WebKit กับไฟล์ Linux release จริง ผ่าน 6 ขนาด: 430×739, 430×932, 932×430, 375×667, 834×1112 และ 1440×900 ตรวจหน้า login, รายชื่อลูกค้า, แชท, ฟอร์มและหน้าสรุปบิล ไม่พบ horizontal overflow; ทดสอบสลับแถบ/เลือกลูกค้ารายเดิม/หมุนจอแล้วข้อความที่ยังไม่ส่งและสรุปที่ยังไม่บันทึกอยู่ครบ รวมถึงจำลอง VisualViewport ขณะเปิดคีย์บอร์ดทั้งในแชทและ dialog ผ่าน ไม่มี JavaScript error หรือ API mutation ใช้ข้อมูลจำลองและ request interception โดยไม่เปิด dev server/SSH tunnel หรือเชื่อม backend จริง; ภาพและตัวตรวจอยู่ใน `data/mobile-responsive-check/` ซึ่งไม่เข้า Git ยังไม่ได้ทดสอบบน iPhone จริงหรือ deploy release นี้บน server

## 2026-11-09 10:25 — ใช้ชื่อแบรนด์แทนประเภทสินค้า และเตรียม release สำหรับรันบน server
- **คำอธิบาย:** ผู้ใช้ยืนยันว่า `category` ของ Alex หมายถึงชื่อแบรนด์ จึงปรับข้อความหน้าจอและคำสั่ง AI ให้เข้าใจตรงกัน พร้อมยืนยันแนวทางต่อไปให้ส่งผ่าน Git และรันแอปบน Ubuntu เท่านั้น
- **โฟลเดอร์ root**
  - `AGENTS.md` — แก้: บันทึกว่า Windows ใช้แก้โค้ด ตรวจสอบ และ build Linux release; เปิด dev stack หรือ SSH tunnel เฉพาะเมื่อผู้ใช้ขอโดยตรง
- **โฟลเดอร์ `frontend/src/`**
  - `components/order-cart.tsx` — แก้: ตัวกรอง ตัวเลือกทั้งหมด ป้ายรายการที่ลูกค้าขอ และคำแนะนำค้นหาใช้คำว่าแบรนด์
  - `lib/api.ts` — แก้: ชื่อฟิลด์ `category` บนหน้าจอและคำอธิบายรายการรหัสใช้ความหมายแบรนด์
- **โฟลเดอร์ `backend/src/`**
  - `order_draft_prompt.txt` — แก้: ระบุว่า `category` คือชื่อแบรนด์ เช่น Haatiya, nina, Vappen แยกจากชนิดเสื้อผ้าและชื่อชุด ให้ AI เรียกแบรนด์ในคำถามภาษาไทย และจับคู่กับ category code ของ Alex ตามเดิม
  - `draft_catalog.rs` — แก้: ข้อความแจ้งไม่พบแบรนด์และผลค้นหาทั้งแบรนด์ รวมถึงคำอธิบายการจับคู่
  - `models/draft.rs` — แก้: อธิบายความหมายของฟิลด์แบรนด์
  - `tests_draft_catalog.rs` — แก้: ข้อความที่คาดหวังในการทดสอบค้นหาสินค้าตามแบรนด์
- **โฟลเดอร์ `docs/`**
  - `step-3-products.md` — แก้: อธิบายแบรนด์ ชื่อชุด และการใช้ category code ใน API
- **โฟลเดอร์ `dist/`**
  - `line-linux/` — แก้: Linux backend release `rel-20261109032225-7b53bc402a54` พร้อม checksum และ metadata
  - `frontend-linux/` — แก้: Linux standalone release `front-20261109032435-4296b6fafb6b` พร้อม static files, runtime dependencies, checksum และ metadata
- **ผลตรวจสอบ:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test -- --include-ignored` ผ่าน 46/46 ด้วย MongoDB ชั่วคราวที่ปิดแล้ว; `npm run lint` และ `npm run build` ผ่าน; `build-all.bat` ผ่านบน local Docker หลังเริ่ม engine ใหม่ ตรวจ ELF/sha256/prompt ของ backend และ checksum frontend 2,108 ไฟล์รวมป้ายแบรนด์ผ่าน ไม่พบ `.env` ใน release และไม่พบ credentials ที่ตั้งค่าไว้ใน source/docs ที่แก้; `git diff --check` ผ่าน ไม่มีการเปิด dev frontend/backend หรือ SSH tunnel ยังไม่ได้ทดสอบ AI กับ Hermes จริงหรือ deploy release นี้บน server

## 2026-11-09 09:42 — แก้ rungit รับข้อความ commit ที่พิมพ์พร้อมเครื่องหมายคำพูด
- **คำอธิบาย:** ผู้ใช้รัน `rungit.bat` แล้วพิมพ์ข้อความ commit พร้อมเครื่องหมายคำพูดในช่องถาม ทำให้ `git commit -m` ได้ argument ผิดและล้มด้วย empty pathspec; อีกจุดเตือนว่าไม่มี `revising-log.md` ทั้งที่ไฟล์อยู่ใน staged changes
- **โฟลเดอร์ root**
  - `rungit.bat` — แก้: ย้ายการตัดเครื่องหมายคำพูดและตรวจข้อความว่างไปหลังรับข้อความทั้งจาก argument และช่องถาม; เปลี่ยนการตรวจ log เป็น `git diff --cached --quiet -- revising-log.md` โดยตรง และหยุดเมื่อ Git diff ล้มเหลว คงไฟล์เป็น ASCII/CRLF
- **ผลตรวจสอบ:** จำลองบั๊กจากสคริปต์เดิมได้ แล้วทดสอบสคริปต์ใหม่ใน Git repositories ชั่วคราวด้วย native Git interceptor ซึ่งดัก commit/push ไม่ให้ทำงานจริง ผ่าน 8 กรณี: ช่องถามแบบมี/ไม่มีคำพูด, argument แบบมีคำพูด, ข้อความมี `&`/วงเล็บ, เตือนเมื่อไม่มี log, ปฏิเสธข้อความว่างก่อน stage, ไม่มี changes ให้ push อย่างเดียว และ Git diff ล้มเหลวให้หยุด; ตรวจ ASCII/CRLF และ `git diff --check` ผ่าน คง staged changes เดิมของผู้ใช้ ไม่ได้ commit/push repository จริง ไม่ต้อง build release เพราะแก้เฉพาะสคริปต์
- **LINE Login:** ตรวจ backend และ public API ยังตอบ `line=false` แต่ `.env` และ stack บน Ubuntu มีค่า LINE Login ครบ ผู้ใช้ชี้แจงว่ายังไม่ได้ deploy เพราะติด rungit; การกู้ config บน server ทำโดยตรง จึงสามารถรัน `sudo bash ./docker-deploy.sh --full-stack` เพื่อโหลดค่าได้โดยไม่ต้องรอ Git

## 2026-11-09 09:33 — กู้ค่า LINE Login หลัง config บน server ถูกแทนที่
- **คำอธิบาย:** ผู้ใช้แจ้งว่าปุ่มเข้าสู่ระบบด้วย LINE หาย ตรวจพบว่า `.env` ปัจจุบันไม่มีค่า LINE Login และรายชื่อแอดมิน การกู้ MongoDB รอบก่อนยังไม่ได้คืนส่วนนี้; API ทั้ง backend และ public domain ตอบ `line=false` จึงทำให้ frontend ซ่อนปุ่มตาม config
- **โฟลเดอร์ `/home/ubuntu/alex-connectingpoint/` บน Ubuntu**
  - `.env` — แก้: กู้ `LINE_LOGIN_CHANNEL_ID`, `LINE_LOGIN_CHANNEL_SECRET`, `LINE_LOGIN_REDIRECT_URI`, `ADMIN_LINE_USER_IDS` จาก `.env.recovered-20261109-011843`; ตรวจว่าค่าตรงกับไฟล์สำรองเดิมทุกชุด มีแอดมินที่ได้รับอนุญาต 1 ราย และ callback คือ `https://hermes.salepie.online/api/admin/line/callback` คงค่า LINE OA Arisa, MongoDB และค่าอื่นทั้งหมดไว้ รวมถึง `ADMIN_COOKIE_SECURE=true`
  - `.env.before-line-login-20261109-093157` — เพิ่ม: สำรอง config ก่อนกู้ LINE Login
  - ไฟล์ที่เขียนใช้ permission `600` และตรวจว่าไม่เข้า Git; ไม่มี secret/token หรือ ID แอดมินในผลคำสั่งและบันทึก
- **ผลตรวจสอบ:** ตรวจค่าในไฟล์หลังเขียน, รูปแบบ LINE user ID ของแอดมิน, callback URI และ `bash -n` ผ่าน; runtime ยังตอบ `line=false` จึงยังต้องให้ผู้ใช้รัน `sudo bash ./docker-deploy.sh --full-stack` บน Ubuntu เพื่อโหลดค่าใหม่ เนื่องจาก SSH ของ agent ต้องใช้รหัส sudo และไม่สามารถ deploy แบบ noninteractive ได้ ยังไม่ได้ทดสอบ OAuth callback ด้วยบัญชีจริง ไม่ได้แก้ source จึงไม่ต้อง build release หรือรันชุดทดสอบ backend/frontend

## 2026-11-09 01:19 — กู้ค่า MongoDB และชื่อ deployment เดิมจากไฟล์สำรองบน Ubuntu
- **คำอธิบาย:** ผู้ใช้ยืนยันว่าต้องการกู้ `.env` บน Ubuntu (`ssh Hermes`) เพราะจำรหัส MongoDB ไม่ได้ พบว่าไฟล์ปัจจุบันยังอยู่แต่ถูกแทนด้วย config local ที่ไม่มีค่า MongoDB และพบไฟล์สำรอง 3 ไฟล์ซึ่งมี Mongo username/password ตรงกันทั้งหมด
- **โฟลเดอร์ `/home/ubuntu/alex-connectingpoint/` บน Ubuntu**
  - `.env` — แก้: กู้ `MONGO_USER`, `MONGO_PASSWORD`, `APP_NAME`, `STACK`, `MONGODB_DB`, `LINE_HTTP_PORT` จาก `.env.before-alex-reference-20261026-135154`; คืน stack `alex-line` และ database `alex_line` ให้ตรงระบบเดิม คงค่าปัจจุบันของ LINE Arisa และค่าอื่นทั้งหมด
  - `.env.before-recovery-20261109-011843` — เพิ่ม: สำรอง config ปัจจุบันก่อนแก้
  - `.env.recovered-20261109-011843` — เพิ่ม: เก็บ config เก่าฉบับเต็มที่กู้จากไฟล์สำรอง แยกจากไฟล์ที่ใช้งานปัจจุบัน
  - ไฟล์ที่เขียนทั้งหมดใช้ permission `600` และตรวจว่าไม่เข้า Git; ไม่มีรหัสผ่าน/token ในบันทึกหรือผลคำสั่ง
- **ผลตรวจสอบ:** อ่าน `.env` กลับมาตรวจว่าค่าที่กู้ตรงกับสำรองและค่า LINE Arisa ตรงกับไฟล์ก่อนแก้; ตรวจรูปแบบ Mongo credentials ตาม deploy script ผ่าน; backend เดิมที่ `127.0.0.1:18080/api/health` ตอบ `ok` ไม่ได้ทดสอบ login MongoDB ด้วยรหัสที่กู้ และยังไม่ได้ restart/deploy เพราะ Docker ต้องใช้ sudo password ไม่ได้แก้ source จึงไม่ต้อง build หรือรันชุดทดสอบ backend/frontend

## 2026-11-09 00:44 — ตั้งค่า LINE OA ร้าน Arisa ให้รับแชทจากลูกค้าหลายคน
- **คำอธิบาย:** ใช้ข้อมูลจาก `prompts/arisa.md` ตั้งค่า OA `Arisa.Rental` (`@186pdmsc`) โดยตรวจ token กับ LINE Get bot info จริง ผู้ใช้ยืนยันว่าต้องการรับลูกค้าหลายคนภายในร้านเดียว โค้ดเดิมแยกบทสนทนาตาม `source.userId` อยู่แล้ว ไม่ต้องตั้ง ID ลูกค้าใน env
- **โฟลเดอร์ root**
  - `.env` — แก้: อัปเดต `LINE_CHANNEL_ACCESS_TOKEN` จากไฟล์ต้นทาง และ `LINE_DESTINATION_ID` จาก bot `userId` ที่ LINE ตอบกลับ; ตรวจว่า channel ID/secret ตรงกับต้นทางแล้ว คง shop ID และค่าอื่นเดิม เพิ่มคำอธิบายว่า destination คือ ID บอต ไม่ใช่ ID ลูกค้าหรือผู้พัฒนา; ไฟล์นี้ไม่เข้า Git
  - `.env.example` — แก้: อธิบายการรับลูกค้าหลายคน และวิธีเลือก bot destination ให้ถูกต้อง
  - `.gitignore` — แก้: กัน `prompts/arisa.md` และ `prompts/image-2.png` ที่มี credentials จริงออกจาก Git
- **โฟลเดอร์ `docs/`**
  - `step-1-line-inquiry.md` — แก้: อธิบาย bot destination, การแยกบทสนทนาของลูกค้าหลายคน และแยกความหมายจากสิทธิ์แอดมิน `ADMIN_LINE_USER_IDS`
- **ผลตรวจสอบ:** LINE Get bot info ตอบสำเร็จและยืนยันชื่อ OA; ตรวจ credentials ตรงกับต้นทาง, ไม่มี env key ซ้ำ, ไม่มี credentials Arisa ในไฟล์ config/docs ที่เข้า Git; `runall.bat -Check`, `git diff --check` และการตรวจ ignore ผ่าน ไม่ได้แก้ source จึงไม่ต้อง build release หรือรันชุดทดสอบ backend/frontend และยังไม่ได้ทดสอบ webhook/แชทจริง
- **การนำไปใช้:** local ตั้ง `DEV_BACKEND_MODE=remote` จึงต้องอัปเดต `LINE_CHANNEL_ACCESS_TOKEN` และ `LINE_DESTINATION_ID` ใน `.env` ของ Ubuntu ด้วย แล้วรัน `sudo bash ./docker-deploy.sh --full-stack` จาก `~/alex-connectingpoint`; งานนี้ยังไม่ได้แก้ server หรือ deploy

## 2026-10-31 01:04 — เตรียมร่างจากสรุปบิลที่มีช่องว่างซ้อนไม่ติด VALIDATION_ERROR อีก
- **คำอธิบาย:** สรุปบิลครบ (ชื่อ/ที่อยู่/เบอร์, รายการชุด, ค่าเช่า 600, ค่าประกัน 200, วันรับ–คืน) และมีรูปแนบ แต่ยังได้ "ข้อมูลไม่ผ่านการตรวจสอบ ตรวจวันที่ จำนวน เงิน หรือหลักฐาน แล้วลองใหม่" สาเหตุที่น่าจะเป็นที่สุด: backend ตรวจว่า quote ใน evidence ของ AI ต้องเป็น substring ที่ตรงตัวอักษรกับสรุป แต่บิลมีช่องว่างซ้อน (`ค่าเช่า  600  บาท`, `จัดส่งชุด    :`, `ลูกค้ารับชุด :     16,17`) และ emoji ที่มี variation selector (`⚡️`) พอ AI ยุบเหลือเว้นวรรคเดียว ทั้งร่างจะถูกปฏิเสธ อีกทั้ง worker ไม่ log เหตุผล จึงรู้แค่ code
- **โฟลเดอร์ `backend/src/`**
  - `models/draft.rs` — แก้: `validate_evidence` เทียบ quote กับ source หลังยุบช่องว่างที่ซ้อนกันเหลืออันเดียว และตัด variation selector/zero-width ออก (`quote_text`) ส่วน quote ที่ว่างหลังตัดยังถูกปฏิเสธ; เพิ่ม test `evidence_quotes_ignore_collapsed_spaces_and_emoji_selectors`
  - `draft_worker.rs` — แก้: งานเตรียมร่างที่ FAILED ให้ `tracing::warn!` พร้อม code และเหตุผล (เป็นข้อความคงที่ ไม่มีข้อมูลลูกค้า) เช่น "Evidence must quote a supplied source"
- **ผลตรวจสอบ:**
  - `cargo fmt --check` และ `cargo clippy --all-targets -- -D warnings` ผ่าน
  - `cargo test -- --include-ignored` ผ่าน 46/46 ด้วย MongoDB ชั่วคราวที่ 127.0.0.1:27018
  - ยังไม่ได้ทดสอบกับ Hermes จริงด้วยบิลนี้ ถ้ายังล้ม ให้ดูเหตุผลใน log ของ backend
  - ยังไม่ได้ build release — ใช้ build-backend.bat ก่อน rungit.bat

## 2026-10-27 17:58 — Landing page ของผลิตภัณฑ์ (ไฟล์เดียว)
- **คำอธิบาย:** ผู้ใช้ขอ landing page ของ SaaS เป็นไฟล์เดียว จึงทำเป็น HTML ไฟล์เดียว ไม่พึ่ง build และไม่แตะ frontend/backend เดิม ข้อความเป็นภาษาไทยตามฟีเจอร์ที่มีจริงในระบบ ใช้ฟอนต์ Mitr และโทนสีตาม `AGENTS.md` ข้อ 6
- **โฟลเดอร์ `landing/`**
  - `index.html` — เพิ่ม: header, hero พร้อมภาพจำลองแชท→ร่างออเดอร์, ปัญหา 3 ข้อ, วิธีทำงาน 5 ขั้น, ฟีเจอร์ 6 ข้อ, ความปลอดภัยพร้อมภาพจำลองสถานะเวอร์ชัน, ขั้นเริ่มใช้งาน, FAQ (`<details>`), CTA และ footer; CSS อยู่ในไฟล์ ไม่มี JS; ชื่อ "Order Connector" และลิงก์ CTA (`@your-line-id`, `hello@example.com`) ยังเป็นค่าชั่วคราว
- **ผลตรวจสอบ:** ไม่ต้องรันเช็ก backend/frontend เพราะไม่ได้แก้ส่วนนั้น และยังไม่ได้เปิดดูหน้าในเบราว์เซอร์

## 2026-10-27 10:37 — สถานะเวอร์ชันร่าง (ส่งแล้ว/ยังไม่ส่ง/ยกเลิก), แก้ไข-โคลน-ยกเลิก, ปลดล็อกจากบิลก่อนหน้า, ส่งทั้งที่ติดคิว
- **คำอธิบาย:** ตาม `prompts/ai-briefting-requirements.md` หัวข้อ "สถานะของ order ก่อนส่ง":
  - แต่ละเวอร์ชันมีสถานะ ✅ ส่งเข้า Alex แล้ว (พื้นเขียว) / ยังไม่ส่ง / ยกเลิกแล้ว
  - ปุ่มแก้ไขใช้ได้เฉพาะเวอร์ชันที่ยังไม่ส่ง ส่วนโคลนใช้ได้ทุกสถานะ และทั้งสองแบบคืนตะกร้าของเวอร์ชันนั้น
  - ยกเลิกได้ทุกเวอร์ชันและย้อนกลับไม่ได้
  - บิลของเวอร์ชันอื่นไม่ล็อกการส่งอีกแล้ว ใช้ผลเช็คคิวแทน ส่วนล็อกช่วงที่กำลังส่งยังคงอยู่
  - ถ้าติดคิวหรือตรวจคิวไม่ได้ ติ๊กยืนยันแล้วส่งได้
- **โฟลเดอร์ `backend/src/`**
  - `error.rs` — เพิ่ม: `DRAFT_CLOSED` และ `QUEUE_BLOCKED` (409)
  - `models/checkout.rs` — แก้: `SubmitInput.accept_queue`
  - `models/draft.rs` — เพิ่ม: `CopyInput` และ `CopyMode` (edit/clone)
  - `db/checkout.rs` — แก้:
    - เพิ่มฟังก์ชันจับคู่บิลกับเวอร์ชัน (`version_bills`, `bill_version`, `closed`) บิลเก่าที่ไม่มี `draft_id` นับเป็นของเวอร์ชันล่าสุดที่บันทึกก่อนเวลาส่ง
    - เวอร์ชันที่ปิดแล้วเตรียมหรือส่งบิลไม่ได้ และ list คืนค่า `closed` พร้อม `draft_version` ของแต่ละบิล
    - review ไม่ปฏิเสธเมื่อติดคิวแล้ว แต่คืนผล `queue` แทน
    - submit ตัดเงื่อนไขบิลทับกับบิลเก่า เปลี่ยนเป็นตรวจว่าเวอร์ชันนี้ส่งไปแล้วหรือยังภายใต้ล็อก ติดคิวและไม่ได้ยืนยันจะได้ `QUEUE_BLOCKED` บันทึก `draft_id`/`draft_version`/`queue`/`queue_accepted` และปลดล็อก inquiry ทันทีที่ Alex ตอบ รวมกรณี `SUBMIT_UNKNOWN`
    - ตั้ง `active=false` เมื่อ Alex ตอบแล้ว ทั้งตอนส่งปกติ, resolve และ recover
  - `db/drafts.rs` — เพิ่ม/แก้:
    - เพิ่ม `versions`
    - `safe_drafts` คืน `state`/`bill` และซ่อน `cancelled_by_id`
    - เพิ่ม `cancel` (ทำได้ครั้งเดียว)
    - เพิ่ม `copy` (edit ต้องเป็นเวอร์ชันที่ยังไม่ส่ง) โดยผ่าน `enqueue_job` ที่ใส่ `copied_from` ไว้ในงาน (browser ส่งค่านี้เองไม่ได้)
  - `db/cart.rs` — เพิ่ม: `snapshot`, `from_bill`, `replace` สำหรับเก็บและคืนตะกร้าของเวอร์ชัน
  - `draft_worker.rs` — แก้:
    - ทุกเวอร์ชันเก็บ `cart` ไว้
    - งานโคลนหรือแก้ไขคืนตะกร้า: เวอร์ชันที่ส่งแล้วใช้รายการในบิล, ร่างล่าสุดที่ยังไม่ส่งใช้ตะกร้าปัจจุบัน, เวอร์ชันอื่นใช้ตะกร้าที่บันทึกไว้
    - ค่าที่ต่างจากช่องที่เคยยืนยันไม่ถูกนับเป็นข้อขัดแย้ง
    - บันทึก `copied_from`
  - `routes/drafts.rs`, `routes/mod.rs` — เพิ่ม: `POST /api/inquiries/{id}/drafts/{draft}/copy` และ `/cancel`
  - `db/indexes.rs` — แก้: ตอน startup เคลียร์ `active` ของบิลที่ไม่ใช่ SUBMITTING (idempotent) เพื่อไม่ให้ unique index กันบิลใหม่ที่มีสินค้าและวันที่เดียวกัน
  - `tests_rental.rs` — แก้/เพิ่ม: ส่งเวอร์ชันเดิมซ้ำได้ `DRAFT_CLOSED`; SUBMIT_UNKNOWN ไม่ล็อก; สถานะเวอร์ชัน; แก้ไขเวอร์ชันที่ส่งแล้วถูกปฏิเสธ; โคลนคืนตะกร้าที่ถูกลบ; ติดคิวแล้วไม่ยืนยันได้ `QUEUE_BLOCKED` ยืนยันแล้วส่งได้; ยกเลิกครั้งที่สองถูกปฏิเสธ; recover ปลดล็อก
- **โฟลเดอร์ `frontend/src/`**
  - `lib/api.ts` — แก้: type `VersionState`/`VersionBill`/`Draft.state` ฯลฯ, ข้อความ `DRAFT_CLOSED`/`QUEUE_BLOCKED`/`BILL_PENDING`
  - `components/admin-workspace.tsx` — แก้: แท็บเวอร์ชันก่อนหน้าเป็นการ์ดที่มีป้ายสถานะ ข้อมูลบิล/ผู้ยกเลิก และปุ่ม แก้ไข · โคลน · ยกเลิกเวอร์ชันนี้ (ถามยืนยันก่อน) แก้ไขร่างล่าสุดจะสลับไปแท็บร่างล่าสุดเท่านั้น
  - `components/draft-editor.tsx` — แก้: ร่างล่าสุดที่ส่งแล้วหรือถูกยกเลิกถูกล็อกทั้งฟอร์มด้วย fieldset พร้อมปุ่มโคลนเป็นเวอร์ชันใหม่
  - `components/rental-checkout.tsx` — แก้:
    - ล็อกเฉพาะตอนที่มีบิลกำลังส่ง (SUBMITTING) และเวอร์ชันที่ปิดแล้ว
    - ถ้าติดคิว แสดงรายการที่ติดพร้อม checkbox "ทราบว่าติดคิว ตกลงส่งต่อ" แล้วส่ง `accept_queue`
    - ป้ายคิวบนการ์ดใช้ผลจากใบตรวจ
    - บิลแสดง v ของร่าง และข้อความ SUBMIT_UNKNOWN ไม่บอกว่าล็อกแล้ว
  - `app/globals.css` — เพิ่ม: สไตล์การ์ดเวอร์ชัน ป้ายสถานะ (เขียว/ฟ้า/เทา) และ `.draft-lock`
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: ข้อ 8.1 (ส่งทั้งที่ติดคิว), ข้อ 9 (สถานะเวอร์ชัน แก้ไข/โคลน/ยกเลิก), API ใหม่, พฤติกรรม SUBMIT_UNKNOWN และ `active`
- **ผลตรวจสอบ:**
  - backend: `cargo fmt --check` และ `cargo clippy --all-targets -- -D warnings` ผ่าน; `cargo test -- --include-ignored` ผ่าน 45/45 ด้วย MongoDB ชั่วคราว (`image_upload_requires_a_public_https_base_before_database_access` ต้องมี Mongo ที่ 127.0.0.1:27018)
  - frontend: `npm run lint` และ `npm run build` ผ่าน
  - ยังไม่ได้เปิดหน้าเว็บจริง และยังไม่ได้ทดสอบกับ Alex จริงว่ารับหรือปฏิเสธออเดอร์ที่ติดคิว
  - ยังไม่ได้ build release — ใช้ build-all.bat ก่อน rungit.bat

## 2026-10-27 09:19 — ใบสรุปออเดอร์เปิดกลางจอ ลากย้ายได้ และบอกเหตุผลที่ปุ่มเตรียมบิลกดไม่ได้
- **คำอธิบาย:** modal ใบสรุปก่อนส่งเข้า Alex ไปติดมุมซ้ายบน เพราะ Tailwind preflight ตั้ง `margin:0` ทับ `margin:auto` ของ `<dialog>`; ผู้ใช้ขอให้เปิดกลางจอและลากย้ายได้ และถามว่าทำไมปุ่ม "ตรวจคิวและเตรียมบิล" กดไม่ได้ (คำเตือนอยู่บนสุดของ modal จึงมองไม่เห็นเมื่อเลื่อนลงมา)
- **โฟลเดอร์ `frontend/src/components/`**
  - `invoice.css` — แก้: `.invoice-dialog` ใส่ `margin:auto` ให้กลับมาอยู่กลางจอ และเลื่อนตาม `--drag-x/--drag-y`; แถบบนสุดเป็นที่จับลาก (cursor grab, ข้อความ "ลากแถบนี้เพื่อย้าย"); จอแคบ ≤560px ยังเต็มจอ ไม่ลาก
  - `draft-editor.tsx` — แก้: `InvoiceDialog` ลากแถบบนเพื่อย้ายด้วย pointer capture (กันไม่ให้แถบหลุดจอ), ดับเบิลคลิกแถบเพื่อกลับกลางจอ, กดปุ่มปิดไม่เริ่มลาก; เปิดใหม่ทุกครั้งเริ่มกลางจอ
  - `rental-checkout.tsx` — แก้: ใต้ปุ่ม "ตรวจคิวและเตรียมบิล" แสดงเหตุผลเมื่อถูกล็อก (บิลก่อนหน้ารอตรวจผลใน Alex / ร่างมีข้อมูลขัดกัน / มีงาน AI หรือการบันทึกค้างอยู่); ปุ่มขึ้น "กำลังตรวจ…" ระหว่างเตรียมบิล
- **ผลตรวจสอบ:** npm run lint และ npm run build ผ่าน; ยังไม่ได้ลองลากในเบราว์เซอร์จริง; ยังไม่ได้ build release — ใช้ build-frontend.bat (หรือ build-all.bat ถ้ารวมงานส่วนลดก่อนหน้า) ก่อน rungit.bat

## 2026-10-27 01:23 — ไม่มีส่วนลดให้แสดง 0 ในช่องส่วนลด
- **คำอธิบาย:** ผู้ใช้ขอให้ส่วนลดค่าเช่า/ค่าประกันที่ไม่มี แสดงเป็นเลข 0 แทนช่องว่าง (เช่น Anya Dress ราคาที่แจ้งเท่ากับราคาใน Alex)
- **โฟลเดอร์ `frontend/src/components/`**
  - `draft-editor.tsx` — แก้: `orderLevel` ตั้งส่วนลดที่ว่างเป็น 0; ค่าอัตโนมัติจากตะกร้าเป็น 0 เมื่อไม่มีส่วนต่าง/ไม่มีราคาที่แจ้ง/ตะกร้าว่าง; ตอนตะกร้าโหลดครั้งแรกเติมเฉพาะช่องที่ร่างที่บันทึกไว้ยังว่าง เพื่อไม่ทับ 0 หรือตัวเลขที่แอดมินบันทึกไว้ หลังจากนั้นเปลี่ยนสินค้าแล้วค่าอัตโนมัติตามไปเหมือนเดิม
- **ผลตรวจสอบ:** npm run lint และ npm run build ผ่าน; จำลองตรรกะใน Node: เปิดร่างยังไม่มีตะกร้า → 0/0, ราคาที่แจ้ง 600/300 ตรงกับ Alex → 0/0, เปลี่ยนเป็นชุด 700 → 100/0, 0 ที่บันทึกไว้เปิดใหม่ยังเป็น 0, ตะกร้าว่างแล้วเพิ่มชุด 700 → 100; ยังไม่ได้เปิดในเบราว์เซอร์จริง; ยังไม่ได้ build release — ใช้ build-all.bat ก่อน rungit.bat

## 2026-10-27 01:21 — ใส่ส่วนลดอัตโนมัติลงช่องส่วนลดจริง (ไม่ใช่แค่ placeholder)
- **คำอธิบาย:** ผู้ใช้พบว่าช่องส่วนลดค่าเช่าแสดงแค่ "อัตโนมัติ 100" เป็นข้อความจาง ไม่ได้ใส่ค่าในช่อง จึงให้หน้าร่างเติมตัวเลขลงช่องเมื่อโหลด/เปลี่ยนตะกร้า
- **โฟลเดอร์ `frontend/src/components/`**
  - `draft-editor.tsx` — แก้: `cartChanged` (แทน `onCart={setCart}`) เติมส่วนลดค่าเช่า/ค่าประกัน = ราคา Alex ในตะกร้า − ราคาที่แจ้ง เมื่อช่องว่างหรือยังเป็นค่าอัตโนมัติจากตะกร้าก่อนหน้า (เปลี่ยนสินค้าแล้วตัวเลขตามไป) ค่าที่แอดมินพิมพ์เองรวม 0 ไม่ถูกแทน; ข้อความแจ้งว่าใส่อัตโนมัติแล้ว แก้ได้ และต้องกดบันทึกร่าง
- **ผลตรวจสอบ:** npm run lint และ npm run build ผ่าน; จำลองตรรกะใน Node: โหลดตะกร้า 600/300 กับราคาที่แจ้ง 500/300 → ส่วนลด 100/ว่าง, เปลี่ยนเป็น 700 → 200, ค่าที่พิมพ์ 150 และ 0 คงเดิม, ค่าที่บันทึกไว้ 100 ตามตะกร้าใหม่; ยังไม่ได้เปิดในเบราว์เซอร์จริง; ยังไม่ได้ build release — ใช้ build-all.bat ก่อน rungit.bat

## 2026-10-27 00:56 — อ่านสรุปบิลของแอดมิน: ประเภท/คำค้น/ไซซ์ ส่วนลดจากราคาที่แจ้ง และใช้บิลจากแชท
- **คำอธิบาย:** ผู้ใช้ส่งตัวอย่างสรุปบิล (`prompts/ai-order-summary-ex.md`) ให้ใช้บรรทัด `ชื่อ [category] / คำค้นหา / ไซส์ / จำนวนวัน` หาสินค้าใน Alex (อาจมีหลายรายการ), ค่าเช่า/ค่าประกันที่แจ้งต่ำกว่าระบบแปลว่ามีส่วนลด, วันลูกค้ารับ/ส่งคืนชุดใช้เป็นวันรับ/คืน, วันใช้งานชุดลงหมายเหตุ และสรุปอาจอยู่ในแชทแทนช่องสรุป
- **โฟลเดอร์ `backend/src/`**
  - `order_draft_prompt.txt` — แก้: กติกาสรุปบิล — "ชื่อ X" คือ category ไม่ใช่ชื่อลูกค้า, คำค้นหน้า " - " เป็นชื่อชุดและหลังเป็นสี, "N วัน" ไม่ใช่จำนวน/วันที่, หลายบรรทัดหลายรายการ, ค่าส่ง/`flash 40-.`/ฟรี, ไม่ใช้รวมยอดโอน, ลูกค้ารับชุด/ส่งคืนชุด → วันรับ/คืน, ใช้งานชุด → notes, ปีสองหลักหลังเดือนไทย (69) เป็น พ.ศ. ไม่ใช่ปีกำกวม, ตัด emoji LINE `(blue dot dress)` เงื่อนไขร้านและเลขบัญชีร้าน; ราคาที่แจ้งให้คัดลอกตามจริงและเว้นส่วนลดให้ backend คิด
  - `db/checkout.rs` — แก้: `discount()` ถ้าร่างไม่มีส่วนลดที่เขียนไว้ ส่วนลด = ยอดแค็ตตาล็อกของตะกร้า − ราคาที่แจ้ง (ไม่ต่ำกว่า 0) ทั้งค่าเช่าและค่าประกัน ส่วนลดที่เขียนไว้ (รวม 0) ชนะเสมอ; unit test
  - `draft_catalog.rs` — แก้: ถ้ามีประเภทแต่คำค้นในประเภทนั้นไม่พบ ค้นทั้งประเภท (คงสี/ไซซ์ก่อน) แล้วให้เลือก (NEEDS_SELECTION ไม่เติมให้เอง) พร้อมคำถาม "ไม่พบ … ในประเภท …"
  - `tests_rental.rs`, `tests_draft_catalog.rs` — แก้: บิลที่ไม่กรอกส่วนลดได้ส่วนลดจากราคาที่แจ้ง (ค่าเช่า 950.00 / ค่าประกัน 500.01) ส่วนลดที่เขียนไว้ชนะ; mode 10 ค้นทั้งประเภทเมื่อชื่อไม่พบ (คำค้น Lyn Dress → Lyn Dress → ว่าง)
- **โฟลเดอร์ `frontend/src/components/`**
  - `draft-editor.tsx` — แก้: ช่องส่วนลดที่เว้นว่างแสดง "อัตโนมัติ …" และยอดหลังหักใช้ส่วนลดอัตโนมัติ (กฎเดียวกับ backend) ข้อความอธิบาย/กรอก 0 เพื่อไม่ให้ส่วนลด และคำเตือนเมื่อราคาที่แจ้งสูงกว่า Alex
  - `rental-checkout.tsx` — แก้: ใบสรุปแสดงราคาที่แจ้งลูกค้าและคำเตือนเมื่อสูงกว่าราคาใน Alex
  - `admin-workspace.tsx` — แก้: ปุ่ม **ใช้สรุปบิลล่าสุดจากแชท** คัดลอกข้อความล่าสุดของแอดมินที่มี "ค่าเช่า" และ "ค่าประกัน" ลงช่องสรุป (ยืนยันก่อนแทนข้อความเดิม) AI ยังอ่านเฉพาะสรุป; placeholder แสดงรูปแบบบิล
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: กติกาอ่านสรุปบิล ปุ่มใช้บิลจากแชท ค้นทั้งประเภท และส่วนลดอัตโนมัติ
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 45/45 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint และ npm run build ผ่าน; ยังไม่ได้ทดสอบ prompt ใหม่กับ Hermes จริง ยังไม่ได้ค้นกับ catalog ร้านจริงหรือส่งบิลจริง; ยังไม่ได้ build release — ใช้ build-all.bat ก่อน rungit.bat

## 2026-10-26 23:47 — วิธีชำระไม่บังคับ ค่าส่งลงหมายเหตุ ใบสรุปเลื่อนได้ ลบรูปสรุป และค้นสีด้วย code ของ Alex
- **คำอธิบาย:** ผู้ใช้แจ้ง 5 ข้อ: วิธีชำระค่าเช่า/ค่าประกันไม่บังคับ (ไม่เลือกให้เป็น "ยังไม่ชำระ"), หน้าต่างใบสรุปต้องเลื่อนได้, ค่าส่งถ้ามีให้ลงหมายเหตุ, รูปสรุปสำหรับ AI ลบได้ และค้นสีไม่ได้เพราะ Alex ใช้ code สี (P) แต่คนเขียนชื่อ (ชมพู) — ตรวจ JS ของ Alex พบตัวเลือกชำระเงินจริงและ `GET /api/color` เป็น `{code, description, state}`
- **โฟลเดอร์ `backend/src/`**
  - `models/checkout.rs`, `db/checkout.rs` — แก้: `PAYMENT_METHODS` ตามตัวเลือก Alex (ยังไม่ชำระ, เงินสด, โอนเงิน, Alipay/wechat, บัตรเครดิต, ช่องทางอื่น) ค่าว่างเป็น "ยังไม่ชำระ" ค่าอื่นถูกปฏิเสธ; ลบ `shipping_note_only` ค่าส่งเขียนต่อท้ายหมายเหตุอัตโนมัติ (ไม่มีบรรทัดว่างนำหน้า)
  - `db/attachments.rs`, `routes/drafts.rs`, `routes/mod.rs` — เพิ่ม: `DELETE /api/inquiries/{id}/draft-images/{attachment}` ลบ record และไฟล์ GridFS และดึง id ออกจาก `working_summary.image_event_ids` คืนรายการรูปและ context hash ใหม่; CORS อนุญาต DELETE
  - `alex.rs`, `models/product.rs` — แก้/เพิ่ม: `Category` เปลี่ยนเป็น `CodeName` ใช้ร่วมกับสี, `alex::colors` (เฉพาะที่ใช้งาน) และ `find_color` (ตรงชื่อ/รหัส แล้วคำสีไทย–อังกฤษ เช่น Pink→ชมพู; ตรงหลายสีไม่เดา); unit test
  - `draft_catalog.rs`, `routes/products.rs`, `order_draft_prompt.txt` — แก้/เพิ่ม: AI lookup ค้นด้วย code สีที่แปลงได้ (แปลงไม่ได้ไม่ใส่ตัวกรองสี) เทียบตัวเลือกด้วย code และคงสีที่ลูกค้าเขียนไว้ในร่าง; `GET /api/admin/alex/colors`
  - `tests_rental.rs`, `tests_draft_catalog.rs` — แก้: วิธีชำระผิด 422 / ว่างเป็นยังไม่ชำระ, หมายเหตุค่าส่ง, ลบรูป (record/ไฟล์/สรุป/ลบซ้ำ 404), mock `/api/color` (มีรายการปิดใช้งาน) และค้นด้วย `BR|`/`P|`
- **โฟลเดอร์ `frontend/src/`**
  - `components/rental-checkout.tsx`, `lib/api.ts` — แก้: วิธีชำระเป็น dropdown ไม่บังคับ เริ่มต้นยังไม่ชำระ; ลบช่องติ๊กรับทราบค่าส่ง แสดงข้อความว่าค่าส่งจะลงหมายเหตุ; ใบสรุปแสดงชื่อสี (เช่น ชมพู (P)); helper `CodeName`/`findCode`/`codeLabel`
  - `components/draft-editor.tsx`, `components/invoice.css` — แก้: หน้าต่างใบสรุป render ผ่าน portal ไปที่ body มีหัวคงที่และเนื้อหาเลื่อนในตัว (`overscroll-behavior: contain`) ล็อกการเลื่อนหน้าเบื้องหลังขณะเปิด สูงตามเนื้อหาไม่เกิน 94vh และเต็มจอบนมือถือ
  - `components/summary-images.tsx`, `components/admin-workspace.tsx`, `components/rental-summary.css` — แก้: ปุ่ม 🗑 มุมรูปสรุปแต่ละรูป (ยืนยันก่อนลบ) ลบแล้วเอาออกจากรูปที่เลือก อัปเดต context hash และทำเครื่องหมายร่างเดิมว่าต้องเตรียมใหม่
  - `components/order-cart.tsx` — แก้: ช่อง **สี** เป็น dropdown สีของร้าน (ส่ง code) เลือกให้จากผลค้นอัตโนมัติหรือชื่อที่ตรง การ์ดผลค้นหา/ตะกร้าแสดงชื่อสีพร้อม code
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md`, `step-3-products.md` — แก้: วิธีชำระ ค่าส่ง ลบรูป และ endpoint สี
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 44/44 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint และ npm run build ผ่าน; ทดสอบ dialog ใน headless Edge ด้วย CSS ที่ build จริง: เนื้อหาเลื่อนได้ (scrollTop 700) ขณะ dialog สูง 94vh และใบสั้นสูงตามเนื้อหา; render ปุ่มลบรูป; ยังไม่ได้ทดสอบกับ `/api/color` ของร้านจริงหรือส่งบิลจริง; ยังไม่ได้ build release — ใช้ build-all.bat ก่อน rungit.bat

## 2026-10-26 23:06 — เพิ่ม build-all.bat (backend → frontend)
- **คำอธิบาย:** ผู้ใช้ขอสคริปต์เดียวที่รัน build-backend.bat แล้วต่อด้วย build-frontend.bat
- **โฟลเดอร์ `./`**
  - `build-all.bat` — เพิ่ม: เรียก `build-backend.bat` แล้ว `build-frontend.bat` ตามลำดับ หยุดทันทีเมื่อขั้นไหนล้ม (backend ล้มจะไม่ build frontend) จบแล้วแสดง release tag จาก `dist\line-linux\release.tag` และ `dist\frontend-linux\frontend.tag` พร้อมขั้นตอนถัดไป ไม่ส่ง argument ต่อเพราะสองสคริปต์รับคนละพารามิเตอร์ (`-Jobs` / `-BackendUrl`); ASCII + CRLF
  - `AGENTS.md` — แก้: เพิ่ม `build-all.bat` ในตารางคำสั่งหลักและขั้นตอน deploy มาตรฐาน
- **ผลตรวจสอบ:** ตรวจ CRLF/ASCII และ `.gitattributes` (eol=crlf); ทดสอบลำดับขั้นด้วย stub build-backend/build-frontend ในโฟลเดอร์ชั่วคราว: กรณีผ่านได้ exit 0 และแสดง tag ทั้งสอง กรณี backend ล้มได้ exit 1 และไม่เรียก frontend; ยังไม่ได้รัน build จริงผ่านสคริปต์นี้

## 2026-10-26 22:51 — ใบสรุปออเดอร์แบบ invoice และปุ่มส่งออเดอร์เข้า Alex จากร่างล่าสุด
- **คำอธิบาย:** ผู้ใช้ขอปุ่มสร้าง/ส่งออเดอร์เข้า Alex เพิ่มในหน้าร่างล่าสุด พร้อมหน้าสรุปแบบ invoice การ์ดสินค้าดูง่ายทันสมัย และสรุปยอดเช่า ส่วนลด ยอดรวมค่าประกัน ส่วนลดค่าประกัน ยอดรวมทั้งหมด จึงออกแบบหน้าคิว/บิลใหม่เป็น invoice แล้วใช้ component เดียวกันเปิดเป็นหน้าต่างจากร่างล่าสุด (logic เช็คคิว/เตรียมบิล/ส่ง/บันทึกผลเหมือนเดิม)
- **โฟลเดอร์ `frontend/src/components/`**
  - `rental-checkout.tsx` — แก้: แสดงแบบ invoice หัวใบพร้อมวันรับ/คืนและจำนวนวัน, ลูกค้า (ปุ่มค้นจากแชทอยู่ใต้ฟอร์ม), การ์ดสินค้า (รูป ชื่อ รหัส ชิปสี/ไซซ์/จำนวน ค่าเช่า/ค่าประกันต่อรายการ ป้ายว่าง/ติดคิวหลังเช็คคิว), การรับและชำระเงิน, สรุปยอด (ยอดเช่า ส่วนลดค่าเช่า ค่าเช่าสุทธิ ยอดรวมค่าประกัน ส่วนลดค่าประกัน ค่าประกันสุทธิ ค่าจัดส่ง ยอดรวมทั้งหมด) ใช้ยอดที่ server ตรวจแล้วเมื่อเตรียมบิล; ขั้นยืนยันมีเวลาหมดอายุใบตรวจ ปุ่มแก้ไขข้อมูล และยืนยันส่ง; ลบตัวเลือกกลุ่มบิล (หนึ่ง inquiry หนึ่งออเดอร์)
  - `draft-editor.tsx`, `admin-workspace.tsx` — แก้: ปุ่ม **สรุปออเดอร์และส่งเข้า Alex** ท้ายหน้า เปิดใบสรุปใน `<dialog>` (โฟกัส/Esc จาก browser) ใช้ได้เมื่อบันทึกร่างแล้วและมีสินค้าในตะกร้า ปุ่มบันทึกเป็นสีหลักเมื่อมีข้อมูลค้างบันทึก
  - `invoice.css` — เพิ่ม: สไตล์ใบสรุป การ์ดสินค้า ยอดรวม และหน้าต่าง dialog (import ใน `app/globals.css`)
  - `rental-summary.css` — ลบ: สไตล์ `.bill-lines`/`.bill-review` ที่ไม่มีที่ใช้แล้ว
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: ขั้นตอนใบสรุปออเดอร์
- **ผลตรวจสอบ:** npm run lint (max-warnings 0) และ npm run build ผ่าน; render ใบสรุปด้วย CSS ที่ build จริงใน headless Edge; ไม่ได้แก้ backend; ยังไม่ได้เปิด dialog ในแอปจริงหรือส่งบิลจริง; ยังไม่ได้ build release — ต้องรัน build-backend.bat และ build-frontend.bat ก่อน rungit.bat

## 2026-10-26 22:42 — แยก section หน้าร่างล่าสุดด้วยพื้นสีอ่อน และปุ่มค้นหาสีตามธีม
- **คำอธิบาย:** ผู้ใช้ขอให้ section ลูกค้า / ค้นหาชุดจาก Alex / ตะกร้าสินค้า / วันรับ–คืนชุด / ส่วนลดและค่าจัดส่ง มีพื้นหลังสีอ่อนแยกกัน ย้ายปุ่มค้นข้อมูลลูกค้าจากแชทไปใต้ฟอร์มลูกค้า และให้ปุ่มค้นหาเป็นสีตามธีม
- **โฟลเดอร์ `frontend/src/components/`**
  - `order-cart.css` — แก้: `.order-section` เป็นกล่องมุมโค้งมีขอบ พื้น slate-50 (ลูกค้า, วันรับ–คืน) sky-50 (ค้นหา) blue-50 (ตะกร้า) pink-50 (ส่วนลด); เพิ่มปุ่ม `.button.search` (primary `--color-primary-ink`) และ `.button.info` (`--color-info-ink`) ตัวอักษรขาว ตรวจแล้วว่าตัวแปรธีมอยู่ใน CSS ที่ build
  - `draft-editor.tsx`, `order-cart.tsx` — แก้: ใส่คลาส section แต่ละส่วน ย้ายปุ่มค้นข้อมูลลูกค้าไปใต้ช่องข้อมูลลูกค้า ปุ่มค้นหาสินค้าใน Alex เป็นสี primary ปุ่มเช็คคิวเป็นสี info
  - `customer-extract-button.tsx` — แก้: ปุ่มค้นข้อมูลลูกค้าจากแชทเป็นสี info (ใช้ทั้งร่างล่าสุดและคิว/บิล)
- **ผลตรวจสอบ:** npm run lint และ npm run build ผ่าน; render หน้าตัวอย่างด้วย CSS ที่ build จริงใน headless Edge; ไม่ได้แก้ backend; ยังไม่ได้ build release — ต้องรัน build-backend.bat และ build-frontend.bat ก่อน rungit.bat (backend มีงานรอบก่อนหน้าที่ยังไม่ได้ build)

## 2026-10-26 22:11 — วิธีมารับตามตัวเลือกของ Alex และป้ายติดคิวบนการ์ดตะกร้า
- **คำอธิบาย:** ผู้ใช้ส่งตัวเลือก "วิธีมารับ" จากหน้า Alex (มารับเอง, Line Man, Grab, Kerry, Flash, ขนส่งอื่น) ให้ใช้ชุดเดียวกันโดยเริ่มต้นเป็น Flash ไม่ต้องมีช่องขนส่งและช่องส่งคืนก่อนเวลา เอาเลขรายการบนการ์ดออก และให้แสดง "ติดคิว" สีแดงบนการ์ด
- **โฟลเดอร์ `backend/src/`**
  - `models/draft.rs` — แก้: เพิ่ม `RECEIVING_METHODS`/`STORE_PICKUP` และ `receiving_method` แปลงชื่อขนส่งที่เขียนมา (flash express, เคอรี่, ไลน์แมน ฯลฯ) เป็นตัวเลือกของ Alex; `normalize` ตั้ง delivery+Flash เมื่อว่าง ขนส่งนอกรายการเป็น "ขนส่งอื่น" และเก็บชื่อเดิมในหมายเหตุ (ไม่ซ้ำเมื่อ normalize ซ้ำ) มารับเองล้างขนส่ง; `return_shipping_deadline` เลิกใช้ (รับจากร่างเก่าแต่ไม่เก็บ) ทิ้ง evidence และเลิกตรวจรูปแบบเวลา
  - `db/checkout.rs` — แก้: ใบตรวจบิลรับ `receiving_method` เฉพาะ 6 ตัวเลือกของ Alex; ไม่สร้างหมายเหตุ "ส่งคืนก่อน" จากช่องเดิม
  - `order_draft_prompt.txt` — แก้: เวลาส่งคืนให้อยู่ใน notes, delivery_provider เขียนตามจริงแล้ว backend แปลงเป็นตัวเลือก Alex
  - `tests_rental.rs` — แก้: ทดสอบแปลงขนส่ง/ขนส่งอื่น/มารับเอง, ร่างเก่าที่มีเวลาส่งคืน, บิลปฏิเสธ "รับหน้าร้าน" และรับ "มารับเอง"
- **โฟลเดอร์ `frontend/src/`**
  - `components/draft-editor.tsx`, `lib/api.ts` — แก้: ช่องเดียว **วิธีมารับ** (6 ตัวเลือก เริ่มต้น Flash) แทนวิธีรับ+ขนส่ง ลบช่องส่งคืนก่อนเวลา ร่างเก่ายกเวลาส่งคืน/ชื่อขนส่งนอกรายการลงหมายเหตุ ผลเช็คคิวผูกกับวันที่+ตะกร้าและซ่อนเมื่อเปลี่ยน
  - `components/order-cart.tsx`, `components/order-cart.css` — แก้: ลบเลขรายการบนการ์ด หลังเช็คคิวการ์ดแสดงป้าย **ติดคิว** สีแดง (ขอบการ์ดแดง) / **ว่าง** สีเขียว / **ตรวจคิวไม่ได้** สีส้ม
  - `components/rental-checkout.tsx` — แก้: วิธีมารับในแท็บคิว/บิลเป็น dropdown 6 ตัวเลือก ตั้งค่าจากร่าง (มารับเอง หรือขนส่งในร่าง หรือ Flash)
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: วิธีมารับและป้ายติดคิว
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 43/43 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint และ npm run build ผ่าน; render การ์ดตะกร้าด้วย CSS ที่ build จริงใน headless Edge; ยังไม่ได้ส่งบิลจริงเพื่อยืนยันว่า Alex รับค่า receivingMethod ทั้ง 6 แบบ (ใช้ค่าจาก dropdown ของ Alex) และยังไม่ได้ build release — ต้องรัน build-backend.bat และ build-frontend.bat ก่อน rungit.bat

## 2026-10-26 21:34 — ตัวเลือกวันรับ/คืนจากรูป และวิธีรับเริ่มต้นเป็น Flash
- **คำอธิบาย:** ผู้ใช้พบว่าวันรับ/คืนชุดว่างทั้งที่มีในรูป สาเหตุคือรูปสองชุดมีวันต่างกัน (และ "รับ 5,6" กำกวม) กติกาออเดอร์เดียวจึงให้ AI เว้นวันว่างแล้วถาม ผู้ใช้ขอให้แสดงเป็นตัวเลือก และให้วิธีรับสินค้าเริ่มต้นเป็น Flash
- **โฟลเดอร์ `backend/src/`**
  - `models/draft.rs` — เพิ่ม/แก้: `DateOption {pickup_date, return_date, label}` และ `DraftResult.date_options` (เป็นคำแนะนำเหมือน questions ไม่ต้องมี evidence) ตรวจไม่เกิน 10 ตัว วันที่ถูกต้อง คืนไม่ก่อนรับ label 1–300 ตัวอักษร; `normalize` ตั้งวิธีรับว่างเป็น delivery และขนส่งว่างของ delivery เป็น Flash
  - `draft_worker.rs` — แก้: วันรับ/คืนต่อรายการรูปแบบเก่าที่ AI อาจยังส่งมาถูกยกเป็น date_options แทนการทิ้ง; การบันทึกร่างด้วยมือคง date_options จากร่างก่อนหน้า
  - `order_draft_prompt.txt` — แก้: ให้ส่ง date_options ทุกคู่วันที่เห็นในรูป/สรุป (วันกำกวมให้หนึ่งตัวเลือกต่อความเป็นไปได้) และไม่ถามเรื่องวิธีรับเพราะ backend ตั้ง Flash ให้
  - `tests_rental.rs`, `tests_step2.rs` — แก้: ตรวจ date_options จาก AI และจากวันต่อรายการแบบเก่า, การตรวจตัวเลือกวันที่ผิด, ค่าเริ่มต้น delivery/Flash และ store_pickup ไม่ถูกเติมขนส่ง, ข้อมูลขาดเหลือวันคืน
- **โฟลเดอร์ `frontend/src/`**
  - `components/draft-editor.tsx`, `components/order-cart.css`, `lib/api.ts` — แก้: ปุ่ม **วันที่อ่านได้จากรูป/สรุป** เหนือช่องวันที่ กดแล้วใส่วันรับ/คืน และแสดงปุ่มที่เลือกอยู่; วิธีรับเริ่มต้น "จัดส่ง" + ขนส่ง Flash และเลือกจัดส่งแล้วเติม Flash ถ้าว่าง
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: ตัวเลือกวันที่จากรูปและค่าเริ่มต้น Flash
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 43/43 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint และ npm run build ผ่าน; ร่างที่สร้างก่อนรุ่นนี้ไม่มีตัวเลือกวันที่ ต้องกดเตรียมออเดอร์ด้วย AI ใหม่; ยังไม่ได้ build release — ต้องรัน build-backend.bat และ build-frontend.bat ก่อน rungit.bat

## 2026-10-26 17:23 — ชื่อบนโปสเตอร์เป็นประเภท (category) ของ Alex: หา code ก่อนค้นสินค้า
- **คำอธิบาย:** ผู้ใช้ชี้ว่ารูปแบบโปสเตอร์ร้าน ชื่อตัวใหญ่ (เช่น Haatiya) คือประเภทสินค้า ไม่ใช่ชื่อสินค้า ต้องหา code ของประเภทก่อนแล้วจึงค้น ตรวจ JS สาธารณะของ shop.alexrental.app พบว่าเว็บ Alex อ่านประเภทจาก `GET /api/category` (ต้องใช้ token) เป็น `{code, description, state}` และกรอง `state=true`; `GetProductItems` รับ `category` เป็น code อยู่แล้ว
- **โฟลเดอร์ `backend/src/`**
  - `alex.rs`, `models/product.rs` — เพิ่ม: `Category {code, name}`, `alex::categories` (เฉพาะที่ใช้งาน) และ `find_category` เทียบชื่อแบบไม่สนตัวพิมพ์/ช่องว่างซ้ำ แล้วจึงเทียบ code; unit test
  - `routes/products.rs`, `routes/mod.rs` — เพิ่ม: `GET /api/admin/alex/categories`
  - `models/draft.rs`, `order_draft_prompt.txt` — แก้: รายการสินค้ามี `category`; prompt ให้ชื่อตัวใหญ่บนโปสเตอร์/การ์ดจองเป็น category ไม่ใช่ product_name/product_code
  - `draft_catalog.rs`, `db/products.rs` — แก้: AI lookup หา code ของประเภทก่อน แล้วค้นด้วย `category=<code>` (ไม่ใส่ keyword เมื่อไม่มีชื่อ/รหัส) พร้อมสี/ไซซ์; ไม่พบประเภทหรือประเภทปิดใช้งาน → ค้นด้วยชื่อแทนและถามให้ตรวจ; ผลค้นอัตโนมัติส่ง `category` ให้หน้าเว็บ
  - `tests_draft_catalog.rs` — เพิ่ม: mock `/api/category` และทดสอบ "haatiya" → `HT` ค้นด้วย code แล้ว MATCHED และประเภทที่ปิดใช้งานไม่ถูกใช้ ค้นด้วยชื่อแทนพร้อมคำถาม
- **โฟลเดอร์ `frontend/src/`**
  - `components/order-cart.tsx`, `components/draft-editor.tsx`, `lib/api.ts` — แก้: ช่อง **ประเภท** (dropdown จาก Alex) ในตัวกรองหลักแทนช่องรหัสหมวดหมู่ ชิปสิ่งที่ลูกค้าขอแสดงประเภทและเลือกประเภทให้อัตโนมัติ ร่างเก็บ `category` ของรายการไว้เมื่อบันทึก
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md`, `step-3-products.md` — แก้: ตัวอย่าง Haatiya เป็นประเภท และ endpoint categories
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 43/43 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint และ npm run build ผ่าน; ยังไม่ได้เรียก `/api/category` ของร้านจริง (ใช้รูปแบบจาก JS ของ Alex) และยังไม่ได้ build release — `dist/` ยังเป็นรุ่นก่อนงานหน้าออเดอร์เดียว ต้องรัน build-backend.bat และ build-frontend.bat ก่อน rungit.bat

## 2026-10-26 16:01 — หน้าออเดอร์เดียวในร่างล่าสุด: ค้นชุด + ตะกร้า + เช็คคิว, ข้อมูลระดับออเดอร์
- **คำอธิบาย:** ผู้ใช้ขอ (1) ค้นและเลือกสินค้า Alex พร้อมรูปในแท็บร่างล่าสุดแล้วตัดแท็บเลือกชุด (2) วันรับ/คืน หมายเหตุ ส่วนลดค่าเช่า/ค่าประกันเป็นต่อออเดอร์ไม่ใช่ต่อสินค้า (3) เช็คคิวในหน้าเดียวกัน (4) ตัดแท็บลูกค้า (5) แสดงสินค้าที่ได้แบบตะกร้า (6) ตัดการ์ดสถิติ และยืนยันรูปแบบลิงก์รูป Alex `/images/{shopInfoId}/item/{file}` ซึ่งตรงกับที่ทำไว้ เดิมผลเลือกชุดผูกกับเวอร์ชันร่างและรายการ จึงหายทุกครั้งที่บันทึกร่าง ตอนนี้ย้ายเป็นตะกร้าต่อ inquiry
- **โฟลเดอร์ `backend/src/`**
  - `models/cart.rs`, `db/cart.rs`, `routes/cart.rs` — เพิ่ม: collection `cart_items` (unique shop/channel/inquiry/product_item_id) และ `GET/POST /api/inquiries/{id}/cart` (เพิ่มจาก search โดยค้น code ใน Alex ซ้ำและตรวจชื่อ/สี/ไซซ์/ราคา, เพิ่มซ้ำบวกจำนวน, ไม่เกิน 20 รายการ, สาขาเดียว), `POST .../cart/item` (ตั้งจำนวน 1–999 หรือเอาออก กัน revision), `POST .../cart/availability` (เช็คคิวตะกร้าด้วยวันที่บนหน้าจอ อ่านอย่างเดียว)
  - `db/checkout.rs`, `models/checkout.rs` — แก้: บิลสร้างจากตะกร้า + วันที่/วิธีรับ/หมายเหตุ/ส่วนลดระดับออเดอร์ของร่างล่าสุดเป็นกลุ่มเดียว ส่วนลดอ่านจากร่าง (ลบออกจาก review input) และห้ามเกินยอดแค็ตตาล็อก รายการขาดเป็น `{field, code?}` (`cart`, วันที่, วิธีรับ, `catalog_price`, `mixed_branches`) ลบการตรวจ Product ID ซ้ำข้ามกลุ่ม; Line มี `image_url`
  - `models/draft.rs` — แก้: `rental`/`amounts`/`notes` ต่อรายการเลิกใช้ (รับจากร่างเก่าแต่ไม่เก็บ) และทิ้ง evidence ของช่องเหล่านี้; `amounts` เพิ่ม `rental_discount_satang`/`deposit_discount_satang`; ข้อมูลขาดเหลือชื่อลูกค้า วันรับ วันคืน วิธีรับ
  - `models/product.rs`, `db/products.rs`, `routes/products.rs`, `routes/mod.rs`, `error.rs`, `db/indexes.rs` — แก้/ลบ: search ไม่ต้องผูกร่าง (draft_version/context_hash/item_index เป็น optional) ลบ confirm, `products/match`, `product_matches` และ `PRODUCT_OPTIONS_DIFFER`; `GET /products` คืนเฉพาะผลค้นอัตโนมัติของร่างล่าสุด
  - `draft_catalog.rs`, `order_draft_prompt.txt` — แก้: ไม่คัดลอกราคาแค็ตตาล็อกลงร่าง ข้อความถามชี้ไปส่วนค้นหาชุด/ตะกร้า; prompt ให้สินค้ามีแค่ code/name/color/size/quantity วันที่ ค่าเงิน ส่วนลด และหมายเหตุอยู่ระดับออเดอร์ วันต่างกันให้ถามว่าจะแยกออเดอร์ ส่วนลดเฉพาะที่ระบุชัด
  - `tests_products.rs`, `tests_rental.rs`, `tests_draft_catalog.rs`, `tests_step2.rs` — แก้: ทดสอบ search+ตะกร้า (Product ID ปลอม, ราคาเปลี่ยน, จำนวนเกิน, revision เก่า, เอาออก, ตะกร้าคงอยู่หลังบันทึกร่าง), บิลตะกร้าเป็นออเดอร์เดียวพร้อมเช็คคิวด้วยวันบนจอ ส่วนลดจากร่างเกินยอด 422 ออเดอร์ที่สองด้วยวันใหม่และผลไม่ชัดเจน, ร่างเก่าที่มีข้อมูลต่อรายการ
- **โฟลเดอร์ `frontend/src/`**
  - `components/order-cart.tsx`, `components/order-cart.css` — เพิ่ม: ค้นหาชุด (สาขา วิธีค้น ชื่อ/รหัส สี ไซซ์ ตัวกรองเพิ่มเติม) ผลค้นหาเป็นการ์ดรูป Alex พร้อมปุ่มเพิ่มลงตะกร้า ชิปสิ่งที่ลูกค้าขอจากรูป/สรุปเพื่อเปิดผลค้นอัตโนมัติ ตะกร้าแบบการ์ด Alex พร้อม −/+ และเอาออก ยอดรวม และ `QueueCheck` เช็คคิว (แทน `product-matcher.css`)
  - `components/draft-editor.tsx` — แก้: หน้าออเดอร์เดียว ลูกค้า → ค้นหา/ตะกร้า → วันรับ–คืน/วิธีรับ/ขนส่ง + เช็คคิว → ส่วนลดค่าเช่า/ค่าประกัน/ค่าส่ง (เทียบยอดตะกร้า) + หมายเหตุ → บันทึก; ยกค่าต่อรายการของร่างเก่าขึ้นระดับออเดอร์อัตโนมัติ
  - `components/admin-workspace.tsx`, `app/globals.css` — แก้: ลบการ์ดสถิติ (และ CSS) แท็บเลือกชุด แท็บลูกค้า ขยายความสูงพื้นที่ทำงาน
  - `components/rental-checkout.tsx`, `lib/api.ts` — แก้: ส่วนลดแสดงจากร่างแบบอ่านอย่างเดียว รายการขาดแบบใหม่ ชนิดข้อมูลตะกร้า/ส่วนลด `filledKeys` ข้ามช่องต่อรายการที่เลิกใช้
  - `components/product-matcher.tsx`, `components/product-matcher.css`, `components/draft-item-editor.tsx`, `components/customer-profile.tsx` — ลบ: ไม่มีที่ใช้แล้ว
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md`, `step-3-products.md` — แก้: ขั้นตอนหน้าออเดอร์เดียว ตะกร้า เช็คคิว ส่วนลดระดับออเดอร์ และ API ตะกร้าแทน confirm/match
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 41/41 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint (max-warnings 0) และ npm run build ผ่าน; render หน้าออเดอร์ด้วย CSS ที่ build จริงใน headless Edge ที่ความกว้างแผงร่าง ยังไม่ได้ทดสอบกับ Alex จริงหรือเปิดแอปเต็มในเบราว์เซอร์; ตะกร้าเก่าไม่มีให้ย้าย ชุดที่เคยยืนยันใน `product_matches` ต้องค้นและเพิ่มลงตะกร้าใหม่
- **ยังไม่ได้ build release:** `build-backend.bat`/`build-frontend.bat` ล้มเพราะ Docker Desktop Linux engine ตอบ 500 (ไม่ได้ restart Docker ให้) `dist/` ยังเป็นรุ่นก่อนหน้า `rel-20261026082840-fb326df853bd` / `front-20261026082941-04f04346e469` ซึ่งไม่มีงานนี้ ต้อง build ใหม่ก่อน `rungit.bat`

## 2026-10-26 15:29 — การ์ดชุดแบบหน้าตะกร้า Alex และรูปจากลิงก์ Alex
- **คำอธิบาย:** ผู้ใช้ส่งภาพการ์ดตะกร้าของ Alex ให้แสดงแบบเดียวกันและใช้รูปจากลิงก์ Alex ตรวจ JS bundle สาธารณะของ shop.alexrental.app พบว่าเว็บ Alex ประกอบรูปเป็น `{origin}/images/{shopInfoId}/item/{mainImage}` (shopInfoId มาจาก login) แต่ backend เดิม join `mainImage` เข้ากับ root ของ API จึงได้ URL ผิดเมื่อเป็นชื่อไฟล์ และรูปของชุดที่ยืนยันแล้วผ่าน proxy ที่ผูกกับ search ซึ่งหมดอายุใน 30 นาที ตรวจแล้วว่า `/images/...` ของ Alex เปิดได้โดยไม่ต้อง login
- **โฟลเดอร์ `backend/src/`**
  - `alex.rs` — แก้: `allowed_image` แปลงชื่อไฟล์เป็น `/images/{ALEX_SHOP_INFO_ID}/item/{file}` (encode ชื่อไฟล์, กัน `..` ออกนอกโฟลเดอร์ร้าน) ค่าที่เป็น path/URL เต็มยังตรวจ origin เดิม; เพิ่ม unit test
  - `db/products.rs`, `routes/products.rs`, `routes/mod.rs`, `models/product.rs` — แก้/ลบ: candidate และชุดที่ยืนยันส่งลิงก์รูป Alex ให้ browser ตรง ลบ `GET /api/inquiries/{id}/product-searches/{search}/images/{product}` ที่ไม่มีผู้ใช้แล้ว (การเทียบรูปฝั่ง server ยังดาวน์โหลดผ่าน `alex::image` ตามเดิม)
  - `tests_products.rs`, `tests_draft_catalog.rs` — แก้: ตรวจว่า `image_url` เป็นลิงก์รูปจาก Alex
- **โฟลเดอร์ `frontend/src/components/`**
  - `product-matcher.tsx`, `product-matcher.css` — แก้: การ์ดชุดตามภาพตัวอย่าง (รูป 100×125 และสี/ไซซ์ด้านซ้าย, ชื่อสีน้ำเงินและรหัสในวงเล็บด้านขวา, ราคา/ค่าประกันสีเขียวแบบตัวเลขบาท, ปุ่ม −/จำนวน/+ และปุ่มเอาออกสีเทากว้าง, ป้ายเลขรายการมุมซ้ายบน) แสดงสาขาเฉพาะเมื่อมีหลายสาขา ปุ่มใช้ข้อความไทย "เอาออก" ตาม AGENTS.md
- **โฟลเดอร์ `docs/`**
  - `step-3-products.md` — แก้: วิธีประกอบลิงก์รูป Alex, endpoint `products/match` และ `quantity` ใน confirm
- **โฟลเดอร์ `dist/` (build บน Docker Desktop local)**
  - `line-linux/` — แก้: backend release `rel-20261026082840-fb326df853bd`
  - `frontend-linux/` — แก้: frontend release `front-20261026082941-04f04346e469`
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 41/41 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint และ npm run build ผ่าน; render การ์ดด้วย CSS จริงใน headless Edge แล้วเทียบกับภาพตัวอย่าง ยังไม่ได้ยืนยันค่า `mainImage` จริงจาก API ของร้าน และชุดที่ยืนยันก่อนรุ่นนี้ยังเก็บลิงก์รูปแบบเดิม ต้องค้นและยืนยันใหม่หากรูปไม่ขึ้น

## 2026-10-26 15:04 — ปรับความกว้างแชท/ร่าง, การ์ดชุดพร้อมจำนวน, ส่วนลด และกติกาเบอร์/จำนวน/วันที่
- **คำอธิบาย:** ผู้ใช้ขอให้ UI ปรับความกว้างซ้าย-ขวาได้ บันทึกเบอร์โดยไม่มีช่องว่าง/ขีดกลาง ใช้จำนวน 1 เมื่อไม่ระบุ มีช่องจำนวนในแท็บเลือกชุดและแสดงชุดเป็นการ์ดแบบหน้าร้าน Alex (รูป ชื่อ รหัส สี ไซซ์ ราคา ค่าประกัน −/+ เอาออก) ใช้แค่วันรับชุด/วันคืนชุดตามที่ Alex ใช้ และกรอกส่วนลดค่าเช่า/ส่วนลดค่าประกันเอง (ผู้ใช้เลือก: เพิ่มช่องส่วนลด, จำนวนอยู่แท็บเลือกชุด, ตัดวันใช้งาน/วันส่งชุดออกทั้งหมด)
- **โฟลเดอร์ `backend/src/`**
  - `models/customer.rs` — แก้: เพิ่ม `normalize_phone` ตัด whitespace และ `-` ใช้ตอนบันทึกโปรไฟล์ อนุญาตเฉพาะตัวเลข `+()` หลังตัด และอัปเดตเทสต์
  - `models/draft.rs` — แก้: `Rental` เหลือ pickup/return/วิธีรับ/ขนส่ง/เวลาส่งคืน ส่วน `usage_date`/`dispatch_date` รับจากร่างเก่าได้แต่ไม่เก็บ ไม่ validate และไม่แสดง (`IgnoredAny` + `skip_serializing`); เพิ่ม `DraftResult::drop_retired_evidence` และ `DraftFields::normalize` (เบอร์โทร, quantity ว่าง → 1)
  - `draft_worker.rs` — แก้: ทิ้ง evidence ของวันที่เลิกใช้ก่อนตรวจ และ normalize หลังตรวจ evidence ทั้งร่าง AI และร่างที่แอดมินแก้
  - `order_draft_prompt.txt` — แก้: schema ไม่มีวันใช้งาน/วันส่งชุด, ห้ามคัดลอกสองวันนี้เป็นวันรับ/คืน, ไม่ถามจำนวนเมื่อไม่ระบุ (backend ใช้ 1)
  - `customer_extract.rs` — แก้: เบอร์ที่ค้นจากแชทส่งกลับแบบตัดช่องว่าง/ขีดกลางแล้ว
  - `models/product.rs`, `db/products.rs`, `routes/products.rs`, `routes/mod.rs` — เพิ่ม/แก้: confirm รับ `quantity` (ค่าเริ่มต้นจากร่างหรือ 1) และเก็บใน `product_matches`; เพิ่ม `POST /api/inquiries/{id}/products/match` ตั้งจำนวน 1–999 หรือเอาชุดออก (`quantity:null`) โดยกัน revision ซ้อน
  - `models/checkout.rs`, `db/checkout.rs` — แก้: ใบตรวจบิลรับ `rental_discount_satang`/`deposit_discount_satang` (ค่าเริ่มต้น 0, ห้ามเกินยอดแค็ตตาล็อก) ส่งเป็น `rentalDiscount`/`bailDiscount` แทนการคำนวณจากราคาที่ตกลง; จำนวนในบิลใช้ค่าจากแท็บเลือกชุด → ร่าง → 1 และไม่ขาดข้อมูล "จำนวน" อีก; ไม่ใส่วันส่ง/วันใช้งานในหมายเหตุ Alex; normalize เบอร์ก่อนตรวจและสร้าง payload
  - `tests_rental.rs`, `tests_customers.rs` — แก้: ทดสอบร่างเก่าที่มีวันเลิกใช้, AI ที่ยังอ้าง dispatch_date, quantity default 1, ตั้งจำนวนผ่าน match (revision เก่า 409, เกิน 999 400, บิลได้จำนวน 2), ส่วนลดเกินยอด 422, payload `bailDiscount` และ `customerPhone` ที่ normalize แล้ว
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx`, `app/globals.css` — แก้: เส้นแบ่งแชท/ร่างลากปรับความกว้างได้ (ลูกศรซ้าย/ขวาบนคีย์บอร์ด, Home หรือดับเบิลคลิกคืนค่าเดิม) จำกัดแชท ≥260px ร่าง ≥320px จำค่าใน localStorage แบบ try/catch ซ่อนเมื่อจอ ≤850px
  - `components/product-matcher.tsx`, `components/product-matcher.css` — แก้: ชุดที่ยืนยันแสดงเป็นการ์ด (รูป ชื่อ รหัส สี ไซซ์ ราคา ค่าประกัน ปุ่ม −/จำนวน/+ และ เอาออก) และมีช่องจำนวนก่อนยืนยันชุด
  - `components/rental-checkout.tsx` — แก้: ช่องส่วนลดค่าเช่า/ส่วนลดค่าประกันแทนค่าเช่า/มัดจำที่ตกลง แสดงยอดแค็ตตาล็อก ราคาที่แจ้งในร่าง และยอดหลังหักส่วนลด ใบตรวจแสดงทั้งสองส่วนลด ลบปุ่มใช้ราคาแค็ตตาล็อก
  - `components/draft-editor.tsx`, `components/draft-item-editor.tsx`, `components/customer-profile.tsx`, `lib/api.ts` — แก้: เหลือวันรับชุด/วันคืนชุด, รายการใหม่จำนวน 1, เบอร์ตัดช่องว่าง/ขีดกลางขณะพิมพ์ (`cleanPhone`), `filledKeys` ข้ามคีย์วันที่เลิกใช้เพื่อไม่ให้ยืนยันร่างเก่าล้ม
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md` — แก้: กติกาเบอร์, การ์ด/จำนวนในแท็บเลือกชุด, ช่องส่วนลด และวันที่ที่ Alex ใช้
- **โฟลเดอร์ `dist/` (build บน Docker Desktop local)**
  - `line-linux/` — แก้: backend release `rel-20261026080252-fca3e52c531b`
  - `frontend-linux/` — แก้: frontend release `front-20261026080355-58508bd94ccf`
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 40/40 บน MongoDB disposable พอร์ต 27018 (ปิดแล้ว); npm run lint (max-warnings 0) และ npm run build ผ่าน; git diff --check ผ่าน ไม่เพิ่ม dependency; ยังไม่ได้ลองลากเส้นแบ่งหรือดูการ์ดในเบราว์เซอร์จริง และยังไม่ได้ทดสอบกับ Alex จริง

## 2026-10-26 13:54 — เปลี่ยนบัญชี Alex ให้ตรงกับ username/password ในไฟล์อ้างอิงของผู้ใช้
- **คำอธิบาย:** ผู้ใช้ย้ำให้ใช้บัญชีใน prompts/alex_login.md ตรวจ request ของไฟล์โดยไม่แสดงรหัสผ่านหรือ token พบว่า username/password บน server เดิมไม่ตรงและเป็นคนละ shopInfoId จึงแก้ค่าฝั่ง server ให้ใช้บัญชีตามไฟล์จริง ผลค้น nina ที่เคยเป็น 0 มาจากบัญชีเก่า
- **Server `Hermes:~/alex-connectingpoint/`**
  - `.env` — แก้: ALEX_API_URL, ALEX_USERNAME, ALEX_PASSWORD ตาม request ในไฟล์ และ ALEX_SHOP_INFO_ID/ALEX_BRANCH_IDS ตามผล login ใหม่ที่ตรวจ active และสิทธิ์จริง ไม่ใช้ token ในตัวอย่าง response สำรองเดิมเป็น .env.before-alex-reference-20261026-135154 เขียน atomic แบบ mode 600 เก็บค่าอื่นรวมถึง Hermes ไว้ครบ
- **ผลตรวจสอบ:** POST /api/user/login ด้วย username/password จากไฟล์ได้ HTTP 200, active=true และมี token สำหรับเรียก API; มีสิทธิ์หนึ่งสาขา ค้น GetProductItems ด้วยบัญชีนี้พบ nina 2 รายการชื่อ nina dress - dark brown สี BR ไซซ์ M/XS และ Haatiya 0 รายการ ตรวจ env หลังเขียนตรงกับบัญชีอ้างอิง ไม่มี key ซ้ำ bash -n ของ .env ผ่าน และค่าที่ไม่ใช่ ALEX_* ไม่เปลี่ยน
- **โค้ดและความลับ:** backend/src/alex.rs มีการ POST username/password จาก config แล้วรับ token จากผล login ก่อนค้นสินค้าอยู่แล้ว จึงไม่แก้โค้ดหรือ build release ใหม่ ไม่เขียนบัญชีจริงลง source/ไฟล์ local/log และคง prompts/alex_login.md ที่มีข้อมูลบัญชีไว้ใน .gitignore ไม่ commit/push
- **ยังต้องดำเนินการ:** บัญชี SSH ไม่มีสิทธิ์ reload Docker โดยไม่ใช้รหัสผ่าน (sudo -n แจ้งว่าต้องใส่รหัส) แจ้งผู้ใช้ให้รัน cd ~/alex-connectingpoint && sudo bash ./docker-deploy.sh --full-stack ผ่าน SSH แล้ว ตรวจสุดท้าย /api/admin/alex ยัง HTTP 200 แต่คืนสาขาของบัญชีเก่า จึงยังไม่ถือว่า process ของแอปโหลดบัญชีใหม่จนกว่าจะตรวจหลัง deploy config สำเร็จ ไม่ build/compile บน Ubuntu และไม่ส่งสร้างออเดอร์จริง

## 2026-10-26 13:45 — สกัดสินค้าจากรูปหรือสรุปแอดมินแล้วค้นและเติมข้อมูลจาก Alex อัตโนมัติ
- **คำอธิบาย:** ผู้ใช้กำหนดให้รูป/สรุปที่แอดมินใส่เป็นต้นทางของออเดอร์ ไม่กวาดออเดอร์หรือสินค้าจากแชท และให้นำชื่อ/รหัส สี ไซซ์ที่สกัดได้ไป search product ผ่าน GetProductItems หลัง login เพื่อเติมข้อมูลจริง จึงต่อขั้นค้นและเติมสินค้าเข้ากับการเตรียมร่าง พร้อมคงการยืนยันสินค้าและการตรวจคิว/บิลแยกตามเดิม
- **โฟลเดอร์ `backend/src/`**
  - `draft_catalog.rs` — เพิ่ม: login Alex แล้วค้น POST /api/v2/GetProductItems ด้วยชื่อ/รหัส สี ไซซ์จากผลสกัด ใช้เฉพาะสาขาที่บัญชีได้รับสิทธิ์ เลือกสาขาอัตโนมัติเมื่อมีสิทธิ์สาขาเดียว ค้นไม่เกิน 5 หน้าต่อตัวกรองและเวลารวม 60 วินาที ไม่ถือว่าตรงรายการเดียวถ้ายังมีหน้าถัดไป ตรวจชื่อเป็นคำเพื่อไม่จับ nina เป็น Ninata และถ้าสี/ไซซ์ไม่ตรงเสนอค่าจริงจาก API ให้เลือก
  - `draft_catalog.rs` — เพิ่ม: เมื่อพบตรงกันรายการเดียวเติมรหัส ชื่อ สี ไซซ์ และค่าเช่า/มัดจำที่ยังว่าง โดยไม่แทนราคาที่แจ้งในรูป/สรุปหรือยอดร่วม เก็บหลักฐาน alex_catalog จาก response จริงและ product search reference ถ้า login/search ล้มเหลว ไม่มีชื่อ สาขาไม่ชัด ไม่พบ หรือหลายตัวเลือก ยังคงร่างที่สกัดได้พร้อมสถานะ/คำถาม ไม่สร้าง Product ID สมมติหรือยืนยันสินค้าอัตโนมัติ
  - `draft_worker.rs`, `order_draft_prompt.txt`, `main.rs` — แก้: เตรียมร่างจาก admin_inputs_only ส่งเฉพาะสรุปแอดมินและภาพที่เลือก ไม่ส่ง events/source_texts จากแชทหรือ previous_draft ให้ AI ใช้ประวัติเดิมเฉพาะฝั่ง server เพื่อตรวจการเปลี่ยนข้อมูลที่เคยยืนยัน เรียก catalog enrichment หลังตรวจหลักฐาน AI แล้วตรวจหลักฐานรวมและบริบท/revision/lease ซ้ำก่อน commit
  - `db/products.rs`, `db/drafts.rs` — แก้: คืนผลค้นหาอัตโนมัติใน suggestions ของรายการสินค้า พร้อม query/filter/page ที่ใช้จริงและ image proxy; search record ผูก inquiry/draft/revision เดียวกันและหมดอายุ 30 นาที การยืนยันยัง login และอ่านรายการจริงซ้ำ ซ่อน catalog_lookup เมื่อหลักฐานถูกยกเลิก ตัด payload snapshot ที่ไม่ได้ส่งให้โมเดลแล้วแต่ยังใช้คำนวณ context hash ตามเดิม
  - `tests_draft_catalog.rs`, `tests.rs`, `tests_rental.rs`, `tests_oa_sync.rs` — เพิ่ม/แก้: ตรวจทั้งรูปและสรุปไม่ส่งแชท/ร่างเก่าให้ AI, login ก่อน GetProductItems, keyword/filter/สาขา, เติมข้อมูล/หลักฐานจริงและยอดที่ขาด, คงยอดแอดมิน, หลายตัวเลือก/ไม่พบ/สีไซซ์ต่าง/login ล้มเหลว/pagination, ยืนยัน Product ID จากผลค้นหาเดิม และไม่ commit เมื่อสรุปเปลี่ยนระหว่างค้น ปรับ assertion ของ snapshot ที่ไม่ใช้ payload ให้ AI แล้ว
- **โฟลเดอร์ `frontend/src/`**
  - `components/product-matcher.tsx` — แก้: โหลดผลค้นหาที่เตรียมไว้พร้อมรายการสินค้าทันที แสดงสถานะและเลือก candidate ที่ตรงไว้ให้ตรวจ โหลดคำค้น/สี/ไซซ์/page จากผลค้นหาจริงเพื่อค้นหน้าถัดไปได้ต่อเนื่อง ไม่ค้นซ้ำเพียงเปิดแท็บ และยังต้องติ๊กยืนยันก่อนบันทึก Product ID
  - `components/admin-workspace.tsx`, `lib/api.ts` — แก้: แสดงขั้นอ่านรูป/สรุปแล้วค้น Alex จำนวนรายการที่ตรงและลิงก์ไปตรวจชุด แสดงหลักฐาน Alex API และอธิบายว่าใช้ข้อมูลลูกค้าจากแชทผ่านปุ่มแยก รองรับ catalog_lookup ในร่าง
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md`, `step-3-products.md` — แก้: ต้นทางข้อมูลใหม่ ขั้น login/search/fill อัตโนมัติ การเก็บราคาที่ตกลงไว้ ผลที่ต้องเลือกเพิ่ม ขีดจำกัด pagination/เวลา และการยืนยันสินค้าก่อนใช้ต่อ
- **โฟลเดอร์ `dist/` (build บน Docker Desktop local เท่านั้น)**
  - `line-linux/` — แก้: backend Linux release `rel-20261026064020-f25500732089` พร้อม checksum/tag/metadata
  - `frontend-linux/` — แก้: frontend Linux release `front-20261026064024-1bbf58790a3b` พร้อม standalone/runtime/static/public และ checksum 2,108 ไฟล์
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test -- --include-ignored ผ่าน 40/40 บน MongoDB disposable พอร์ต 27018; npm.cmd run lint/build ผ่านทั้ง Windows และ Linux release; Docker runtime images ทั้งสองส่วน build ผ่าน; backend/frontend checksums และ git diff --check ผ่าน ไม่เพิ่ม dependency
- **ตรวจหน้าเว็บ:** Chrome headless กับ backend จริงและ mock Hermes/Alex ผ่านการแนบรูปตัวอย่าง/รูป fixture รวม 5 รูป สกัดสองรายการที่ไม่มีข้อความสรุป ค้นสินค้าสองชิ้นอัตโนมัติ เติมรหัส/ราคาที่ขาด โหลด candidate โดยไม่ค้นซ้ำ ติ๊กยืนยัน Product ID ทั้งสองชิ้นโดยไม่สร้างบิลหรือส่ง LINE ไม่มี page error/ล้นที่ 390px ภาพอยู่ใน ignored data/catalog-ui-b9fa43e70d/ และ script ใน data/test-draft-catalog-ui.py
- **ตรวจระบบจริงแบบอ่านข้อมูล:** login Alex ด้วยบัญชีที่ตั้งบน server สำเร็จ GetProductItems คำค้น nina และ Haatiya คืน 0 รายการในสาขาที่มีสิทธิ์ จากนั้นใช้ backend local รุ่นใหม่กับ Mongo แยก ส่งภาพ fixture ผ่าน Hermes จริงและค้น Alex จริง ได้ draft request DONE พร้อมสถานะ NOT_FOUND สำหรับ nina วันส่ง 2026-11-04/วันคืน 2026-11-08 และไม่มี Product ID ที่แต่งขึ้น ค่าบัญชีใช้เฉพาะ memory/environment ของ process ทดสอบ ไม่มี .env ใหม่หรือ secret ใน source/script/log; script ที่ไม่มีค่าบัญชีอยู่ใน ignored data/hermes-alex-live-check.py ไม่เปลี่ยน config/runtime ของ server และไม่สร้างบิลจริง
- **สถานะ:** ปิด backend/frontend/Chrome/HTTP fixture/SSH tunnel/Mongo disposable ที่สร้างเองแล้ว ยังไม่ commit/push/deploy ตาม AGENTS.md เก็บการแก้ AGENTS.md, deploy-all.sh, requirements และ entry 13:24 ของงานอื่นไว้ครบ ผู้ใช้ส่ง source และ dist ด้วย rungit.bat แล้วบน Ubuntu ใช้ bash ./loadgit.sh main, sudo bash ./docker-build.sh --full-stack, sudo bash ./docker-deploy.sh --full-stack หรือ deploy-all.sh ตามคู่มือฉบับล่าสุด

## 2026-10-26 13:24 — เพิ่มคำสั่ง deploy-all สำหรับ Ubuntu
- **คำอธิบาย:** ผู้ใช้ขอรวมขั้นตอน deploy บน server (loadgit → docker-build --full-stack → docker-deploy --full-stack) เป็นคำสั่งเดียว ไม่ compile บน server เหมือนเดิม
- **โฟลเดอร์ `./`**
  - `deploy-all.sh` — เพิ่ม: รัน `loadgit.sh [branch]` (ค่าเริ่มต้น main, ส่ง FORCE/REMOTE ต่อได้), `docker-build.sh --full-stack`, `docker-deploy.sh --full-stack` ตามลำดับ หยุดทันทีเมื่อขั้นไหนล้ม ถาม sudo password ครั้งเดียวก่อนเริ่ม ถ้ารันด้วย sudo จะรัน git ในนามเจ้าของ repo เพื่อไม่ให้ `.git` เป็นของ root ห่อทั้งไฟล์ใน `main` เพื่อไม่พังเมื่อ pull ได้ตัวสคริปต์รุ่นใหม่ระหว่างรัน จบแล้วแสดง release tag ของ backend/frontend; ใช้ LF
  - `AGENTS.md` — แก้: เพิ่ม `deploy-all.sh` ในตารางคำสั่งหลัก (ข้อ 4) และในขั้นตอน deploy มาตรฐาน (ข้อ 8)
- **ผลตรวจสอบ:** `bash -n` ผ่าน, ไม่มี CR ในไฟล์; จำลองรันบน Git Bash ด้วย stub ของ 3 สคริปต์และ sudo: ลำดับ/argument ถูก, branch และ FORCE ส่งต่อได้, argument เกินแสดง usage, exit code ของขั้นที่ล้มถูกส่งออกและไม่รันขั้นถัดไป; กรณีรันเป็น root (`sudo -u` เจ้าของ repo) ทดสอบบน Windows ไม่ได้ ไม่ได้รันบน server จริง

## 2026-10-26 13:10 — เชื่อม Hermes สำหรับสรุปรูปและใช้ปีปัจจุบันเมื่อไม่ได้ระบุปี
- **คำอธิบาย:** ตรวจ server จริงพบว่า .env ของแอปยังไม่มี HERMES_API_URL/HERMES_API_KEY และไม่มี API listener พอร์ต 8642 แม้ dashboard 8646 ทำงานอยู่ ข้อความ AI ยังไม่พร้อมจึงมาจาก config ที่ว่าง อีกปัญหาคือ backend เดิมรับ /v1/toolsets เป็น array เท่านั้น แต่ Hermes ที่ติดตั้งคืน list envelope จึงตั้ง extraction profile จริงและแก้ parser พร้อมเปลี่ยนกติกาวันที่ตามที่ผู้ใช้สั่ง
- **โฟลเดอร์ `backend/src/`**
  - `draft_worker.rs` — แก้: ส่ง current_year จากเวลาสร้างคำขอตาม Asia/Bangkok ให้ AI รักษาปีเดิมเมื่อ retry ข้ามปี และรองรับ toolsets ทั้ง envelope ของรุ่นปัจจุบันและ array ของรุ่นเก่าโดยยังปฏิเสธ tools ที่เปิดอยู่หรือ schema ที่ไม่รู้จัก
  - `order_draft_prompt.txt` — แก้: วันส่ง/รับ/ใช้/คืนที่มีวันเดือนชัดเจนแต่ไม่มีปีใช้ปีปัจจุบันของคำขอ รวมวันที่แยกต่อรูป/ต่อชุด ใช้ปีที่ระบุไว้ก่อนและแปลง พ.ศ. เป็น ค.ศ. ไม่เลื่อนวันในอดีตไปปีหน้าเอง คงคำถามสำหรับวันที่กำกวมหรือไม่ถูกต้องและอ้างหลักฐานต้นฉบับ
  - `tests_rental.rs` — เพิ่ม/แก้: ตรวจปีตรงรอยต่อปีใหม่เวลาไทย parser ของ toolsets ทั้งสองรูปแบบ ผลที่ไม่ปลอดภัย และเส้นทาง image-only ผ่าน list envelope พร้อมวันส่ง/คืนแยกรายการ
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx`, `lib/api.ts` — แก้: อธิบายว่าบันทึกสรุปเก็บข้อมูลและต้องกดเตรียมออเดอร์ด้วย AI เพื่ออ่านภาพ แจ้งสาเหตุเมื่อ server ไม่ได้ตั้ง Hermes และแสดงกติกาปีปัจจุบันโดยไม่บังคับกรอกปีในคำบรรยาย
- **โฟลเดอร์ `docs/`**
  - `image-rental-billing.md`, `step-2-admin.md` — แก้: กติกาปีปัจจุบัน ตัวอย่างวันส่ง/รับ/ใช้/คืน และวิธีตรวจ/restart extraction profile กับรูปแบบ toolsets ของ Hermes ที่ติดตั้งจริง
- **Server `Hermes` (แก้ runtime config เท่านั้น ไม่ compile หรือ deploy source บน Ubuntu)**
  - `~/.hermes/profiles/alex-extraction/` — เพิ่ม: profile สำหรับสกัดข้อมูลภาพโดยเฉพาะ ใช้ model/provider เดิมจาก Hermes และ global auth fallback โดยไม่คัดลอก OAuth token ตั้ง api_server/cli toolsets เป็น [] ปิด memory/user profile ไม่ผูก LINE/Telegram เปิด API เฉพาะ 172.18.0.1:8642 พร้อม key สุ่มแยก
  - `~/.config/systemd/user/hermes-alex-extraction.service` — เพิ่ม: user service ที่ enable และ active แล้ว; Linger=yes; ไม่ restart gateway หลัก ตรวจ gateway หลักยัง active และ endpoint ที่ไม่แนบ key ตอบ 401
  - `~/alex-connectingpoint/.env` — แก้: ตั้ง HERMES_API_URL/HERMES_API_KEY/HERMES_MODEL/HERMES_TIMEOUT_SECONDS สำรองเดิมเป็น .env.before-hermes-20261026-130127 และเขียน atomic แบบ mode 600 ตรวจ key ตรงกับ profile โดยไม่แสดงค่าใน output/log
- **โฟลเดอร์ `dist/` (build บน Docker Desktop local เท่านั้น)**
  - `line-linux/` — แก้: backend Linux release `rel-20261026060511-4bd5b97e9ff0` พร้อม checksum/tag/metadata
  - `frontend-linux/` — แก้: frontend Linux release `front-20261026060513-5d6f1e105e1d` พร้อม runtime/static/public และ checksum 2,108 ไฟล์
- **ผลตรวจสอบ:** cargo fmt --check, cargo clippy --all-targets -- -D warnings และ cargo test -- --include-ignored ผ่าน 37/37 บน MongoDB disposable พอร์ต 27018; npm.cmd run lint/build ผ่าน; runtime Docker image ทั้งสองส่วน build ผ่าน; checksum backend/frontend ทุกไฟล์และ git diff --check ผ่าน ไม่เพิ่ม dependency
- **ทดสอบ Hermes จริง:** ส่งภาพ fixture ที่สร้างเองเพื่อทดสอบ OCR ไป API บน server ได้ HTTP 200 และ finish_reason=stop อ่าน nina/dark brown/M ค่าเช่า 550 บาท ค่าส่ง Flash 40 บาท แปลงวันส่ง 4/11 เป็น 2026-11-04 และคงวันรับ 5,6 เป็นคำถาม จากนั้นทดสอบผ่าน backend local รุ่นใหม่กับ Mongo แยกและ Hermes จริงผ่าน SSH tunnel: อัปโหลดภาพ/บันทึกสรุป/prepare/poll ได้ DONE พร้อมหลักฐานครบทั้ง v1 ที่ใช้ปีปัจจุบัน 2026 และ v2 ที่ระบุ พ.ศ. 2570 แล้วได้ปี 2027 ไม่ส่ง LINE หรือสร้างบิลจริง script ทดสอบอยู่ใน ignored data/hermes-live-check.py และไม่มี key ในไฟล์ ปิด backend/tunnel/HTTP fixture/Mongo disposable ที่สร้างเองแล้ว
- **ยังต้องดำเนินการก่อนใช้บนหน้าเว็บจริง:** ยังไม่ commit/push ตาม AGENTS.md และยังไม่ deploy source/release ใหม่ บัญชี SSH ต้องใส่รหัสผ่าน sudo จึงไม่ restart แอปที่รันอยู่ ผลตรวจสุดท้าย /api/admin/session ของ production ยัง hermes_configured=false เพราะ process เดิมยังไม่โหลด .env ต้องใช้ rungit.bat ส่ง source พร้อม dist แล้วบน Ubuntu รัน bash ./loadgit.sh main, sudo bash ./docker-build.sh --full-stack และ sudo bash ./docker-deploy.sh --full-stack จากนั้นรีเฟรชหน้าเว็บและกดเตรียมออเดอร์ด้วย AI; การ reload เฉพาะ config ให้รุ่นเก่ายังติด parser toolsets จึงต้องใช้ release ใหม่ด้วย

## 2026-10-26 12:44 — รีโหลด server และยืนยัน login Alex ผ่านตัวแอปสำเร็จ
- **คำอธิบาย:** ผู้ใช้สั่งทำต่อให้เสร็จหลังตั้ง ALEX_* ใน .env บน server แล้ว บัญชี SSH ต้องใช้ sudo จึงเปิดหน้าต่าง SSH ให้ผู้ใช้กรอกรหัสผ่าน Ubuntu โดยตรง ผู้ใช้ยืนยันว่ากรอกแล้ว จากนั้นดำเนินการ deploy config และตรวจผลผ่านตัวแอปจริง
- **Server `Hermes:~/alex-connectingpoint/`**
  - Runtime — รัน `sudo bash ./docker-deploy.sh --full-stack` สำเร็จ เพื่อให้ backend โหลด .env ใหม่ ใช้ source/release ที่อยู่บน server เดิม ไม่ compile โปรแกรมบน Ubuntu ไม่เปลี่ยน Mongo volume/network ไม่ส่ง source รุ่นใหม่และไม่สร้างบิลจริง
- **ผลตรวจสอบ:** GET /api/health -> status ok; GET /api/admin/alex ผ่าน internal auth -> HTTP 200, configured=true และคืนสาขาหลักที่บัญชีได้รับสิทธิ์หนึ่งสาขา ยืนยันว่า backend ที่รันอยู่ login Alex จริงได้แล้ว; frontend พอร์ต 18081 ตอบ HTTP 200; ปิดหน้าต่าง helper ที่สร้างเองหลังเสร็จ ไม่มีรหัสผ่าน sudo หรือ token ในไฟล์/log; source และ release local ไม่เปลี่ยนจึงไม่ build ซ้ำ ไม่ commit/push
- **สถานะ:** งานเชื่อมบัญชี Alex บนแอป server เสร็จแล้ว ข้อความรอ reload ใน entry 12:39 ได้รับการดำเนินการใน entry นี้; server ใช้ backend rel-20261026035638-9868a8c39c0b และ frontend front-20261026035745-26c4e8e53c6d เดิม การนำ source/release ฟีเจอร์รูป 5 รูปและคิว/บิลรุ่นใหม่ขึ้น server ยังต้องส่งผ่าน Git ตามขั้นตอนของผู้ใช้ แยกจากการ reload บัญชีรอบนี้


## 2026-10-26 12:39 — ตั้งบัญชี Alex บน server และตรวจ login จริง
- **คำอธิบาย:** ผู้ใช้ชี้ให้ตรวจ `prompts/alex_login.md` พบว่าการทดสอบ login API ก่อนหน้ายังไม่ได้ตั้งบัญชีให้แอปที่ใช้งานบน server ตรวจ `.env` ทั้ง local/server แบบปิดบังค่า พบ ALEX_* ไม่ได้ตั้ง และ `/api/admin/alex` ของแอปยังไม่มีสาขา จึงตั้งค่าบัญชีที่ผู้ใช้ให้บน server จริง
- **Server `Hermes:~/alex-connectingpoint/`**
  - `.env` — แก้: เพิ่ม ALEX_API_URL, ALEX_USERNAME, ALEX_PASSWORD, ALEX_SHOP_INFO_ID, ALEX_BRANCH_IDS จากผล login ที่ตรวจร้านและสาขาที่ได้รับสิทธิ์จริง เก็บเฉพาะฝั่ง server สำรองไฟล์เดิมเป็น `.env.before-alex-20261026-123828` แบบ private permissions และเขียน `.env` แบบ atomic โดยไม่แก้ค่าอื่น ไม่มีรหัสผ่านหรือ token ใน Git/log
- **โฟลเดอร์ `./`**
  - `.gitignore` — แก้: ยกเว้น `prompts/alex_login.md` เพราะ reference ที่ผู้ใช้เพิ่มมี live authentication token ป้องกันนำเข้า commit โดยคงไฟล์ต้นฉบับไว้ในเครื่อง
- **ผลตรวจสอบ:** POST login จาก Ubuntu สำเร็จ HTTP 200 บัญชี active อ่าน GetProductItems จริงได้ 15 รายการ ผูกสาขาหลักที่บัญชีได้รับสิทธิ์ แยกจาก branch ตัวอย่างที่อยู่นอกสิทธิ์; ตรวจ config ทั้ง 5 key มีเพียงหนึ่งค่าและไฟล์เป็น mode 600; login/catalog/checkq จาก local ผ่านเช่นกัน; git check-ignore และ git diff --check ผ่าน ไม่แก้ source/binary จึงไม่ build ซ้ำ
- **ยังต้องดำเนินการ:** แอปที่รันอยู่ยังไม่ได้โหลด env ใหม่ บัญชี SSH ไม่มีสิทธิ์ Docker และ `sudo -n` ขอรหัสผ่าน จึงไม่ได้ restart/deploy ให้ ผู้ใช้ต้องรันบน Ubuntu: `cd ~/alex-connectingpoint && sudo bash ./docker-deploy.sh --full-stack` เพื่อโหลดบัญชี Alex เข้า backend; ยังไม่ถือว่าตัวแอปเชื่อมสำเร็จจนตรวจ `/api/admin/alex` หลัง reload ไม่สร้างบิลจริง ไม่ commit/push และไม่ deploy source รุ่นใหม่


## 2026-10-26 12:32 — สรุปจากรูป 5 รูป เติมลูกค้าจากแชท และเช็คคิว/สร้างบิลใน Alex
- **คำอธิบาย:** ผู้ใช้เปลี่ยนสรุปการเช่าชุดเป็นภาพที่มีชื่อชุด สี ไซซ์ และวันที่ ให้แนบได้ 5 รูป เติมชื่อ/ที่อยู่/โทรจากข้อความลูกค้า และยืนยันว่าต้องการบิล/ออเดอร์ใน Alex ใช้ request contract จาก `prompts/producitem_search.json` และ `prompts/orderheader_create.json` โดยไม่เพิ่ม dependency และไม่แก้ไฟล์ตัวอย่างที่ผู้ใช้ส่งมา
- **โฟลเดอร์ `backend/src/`**
  - `models/draft.rs`, `order_draft_prompt.txt` — แก้: รองรับ image-only สูงสุด 5 รูป ชื่อชุดแยกจากรหัส ที่อยู่ลูกค้า และวัน/ราคา/ค่าส่ง/ขนส่ง/อุปกรณ์แยกรายการ ไม่เดาปี วันรับที่กำกวม จำนวนจากวันเช่า หรือราคา tier ในโปสเตอร์ ตรวจวันที่และเวลา HH:MM
  - `db/attachments.rs`, `db/drafts.rs`, `db/oa_sync.rs`, `draft_worker.rs`, `routes/drafts.rs` — เพิ่ม/แก้: อัปโหลดรูปภายในผ่าน GridFS แยกจากการส่ง LINE ตรวจ shop/channel/inquiry และ hash กันซ้ำ รวมรูปแนบใน context/evidence และคืนรูปในรายการร่าง จำกัดข้อมูลภาพรวม 8 MB ต่อ AI request
  - `customer_extract.rs` — เพิ่ม: สกัดข้อมูลติดต่อเฉพาะข้อความลูกค้าใน inquiry พร้อม exact quote และ source ตรวจค่าที่คืนกับหลักฐานและตรวจบริบทใหม่หลัง AI ไม่ใช้ข้อความร้าน/รูปแนบ/โปรไฟล์เก่า ไม่บันทึกทับอัตโนมัติ ช่องที่ไม่พบหรือขัดกันคงค่าเดิมให้แอดมินตรวจ
  - `models/checkout.rs`, `db/checkout.rs`, `routes/checkout.rs` — เพิ่ม: จัดกลุ่มตามสาขา/ช่วงเช่า/วิธีรับ/ขนส่ง เช็คคิว แยกว่าง/ไม่ว่าง/ยังตรวจไม่ได้ อ่านราคาและคิวซ้ำก่อนเตรียมและส่งบิล เก็บใบตรวจ 5 นาที คำนวณส่วนลดด้วย integer satang ตรวจข้อมูลลูกค้า และสร้าง AddRentalOrder หลังยืนยันเท่านั้น
  - `db/checkout.rs` — เพิ่ม: บันทึกก่อน POST และส่งครั้งเดียวต่อ review ID ล็อก inquiry/กันรายการทับกับบิลก่อนหน้า กู้สถานะเมื่อรีสตาร์ต รักษาประวัติเมื่อร่างเก่า ผลไม่ชัดเจนเป็น SUBMIT_UNKNOWN ไม่ส่งซ้ำ ให้บันทึกผลตรวจจริงใน Alex และรออย่างน้อย 2 นาทีหลังส่งก่อนยืนยันว่าไม่ได้สร้าง ป้องกัน sender เก่าทำงานต่อหลัง manual resolution และกรอง token จาก response ที่เก็บ
  - `alex.rs`, `config.rs`, `error.rs`, `main.rs`, `db/indexes.rs`, `db/mod.rs`, `models/mod.rs`, `routes/mod.rs` — เพิ่ม/แก้: checkq และ AddRentalOrder ฝั่ง server, รับวันที่ตอบกลับแบบ T00:00:00 ที่ตรวจจริง, ALEX_ORDER_ID_POINTER แบบ optional, routes/indexes/error และ modules ใหม่ MCP คงเฉพาะการค้นหาแบบอ่านข้อมูล
  - `tests_rental.rs`, `tests.rs`, `tests_products.rs`, `tests_customers.rs` — เพิ่ม/แก้: image-only/ขีดจำกัด 5 รูป การแยกวัน/เงิน/ปีที่หายไป หลักฐานลูกค้าและ context race รูปไม่ถูกส่ง LINE ผลเช็คคิวหลายแบบ การเตรียมไม่สร้างบิล การส่งซ้ำ/ผลไม่ชัดเจน/manual resolution/กู้ orphan lock และประวัติเมื่อร่างเก่า ปรับ fixture สำหรับ field ใหม่
- **โฟลเดอร์ `frontend/src/`**
  - `components/summary-images.tsx`, `customer-extract-button.tsx`, `draft-item-editor.tsx`, `rental-checkout.tsx`, `rental-summary.css` — เพิ่ม: แนบ/ย่อภาพโดยตรงสูงสุด 5 รูป เติมข้อมูลลูกค้าพร้อมหลักฐาน แก้วันและราคาเฉพาะชุด และแท็บคิว/บิลที่ตรวจรายการ/ลูกค้า/ยอดก่อนติ๊กสร้างออเดอร์จริง แสดงประวัติและผลที่ต้องตรวจจาก Alex
  - `components/admin-workspace.tsx`, `customer-profile.tsx`, `draft-editor.tsx`, `product-matcher.tsx`, `chat-message.tsx`, `lib/api.ts`, `app/globals.css` — แก้: เชื่อมรูปแนบ/ข้อมูลใหม่/แท็บคิวบิล ค้นชื่อชุดเมื่อไม่มี SKU ใช้รูปสรุปเป็นภาพอ้างอิงได้ จำกัดรูปรวม 5 รูป ป้องกันบริบทเก่า แยกคำว่าอนุมัติร่างภายในจากการสร้างบิลจริง และคงรูปแบบหน้าเดิม
- **โฟลเดอร์ `./` และ `docs/`**
  - `.env.example`, `docker-stack.line.yml` — แก้: เพิ่ม ALEX_ORDER_ID_POINTER โดยไม่ใส่บัญชี/token จริงใน source หรือ frontend
  - `docs/image-rental-billing.md` — เพิ่ม: วิธีใช้ ข้อมูลที่ต้องถามเพิ่มจากภาพตัวอย่าง ข้อจำกัดค่าส่ง/ผลตอบกลับ การตรวจบิลค้าง API/config และขั้นตอน local build/full-stack deploy
  - `docs/step-2-admin.md`, `docs/step-3-products.md`, `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: จำนวนภาพ 5 รูป ลิงก์คู่มือและสถานะการใช้งานล่าสุด
- **โฟลเดอร์ `dist/` (build บน Docker Desktop local เท่านั้น)**
  - `line-linux/` — แก้: Linux backend `rel-20261026052929-8225ec8d096a` พร้อม checksum/tag/metadata
  - `frontend-linux/` — แก้: Linux standalone `front-20261026052930-c8f4ed0cc016` พร้อม public/static/runtime และ checksum 2,108 ไฟล์
- **ผลตรวจสอบ:** cargo fmt --check, clippy --all-targets -D warnings, cargo test -- --include-ignored ผ่าน 35/35 บน MongoDB disposable พอร์ต 27018; npm lint/build ผ่านทั้ง Windows และ Linux release build; Docker runtime image backend/frontend build ผ่าน; docker stack config สำหรับ line+admin ผ่านพร้อม env ใหม่และ Mongo volume/network เดิม; ELF/backend checksum และ frontend checksum ทั้งหมดผ่าน ไม่มี .env หรือไฟล์ release ถูก ignore; git diff --check ผ่าน
- **ตรวจหน้าเว็บ:** Chrome headless ใช้ backend จริงและ Mongo แยก พร้อม mock Hermes/Alex อัปโหลดภาพตัวอย่างผ่าน browser encoder รวม 5 รูปโดยไม่มีคำบรรยาย สกัดสองรายการ/วันที่ยังไม่ระบุปี เติมและบันทึกลูกค้า ค้นชื่อและยืนยัน Product ID สองชิ้น แยกสองช่วงวัน เช็คไม่ว่าง/ว่าง ตรวจบิลก่อน explicit submit จำลอง HTTP 500 หลังรับคำขอแล้วล็อกกันส่งซ้ำ ไม่ส่งรูปสรุปเข้า LINE ไม่ล้นที่ 390px ไม่มี page error; ภาพตรวจอยู่ใน ignored `data/rental-ui-141225d5c1/`
- **ขอบเขตและข้อจำกัด:** ทดสอบ Alex จริงเฉพาะอ่าน login/catalog/checkq พบ result=1 และวันที่มี T00:00:00; ไม่สร้างบิลจริง ไม่ทดสอบสกัดกับ Hermes profile จริง ไฟล์ผู้ใช้เป็น request ยังไม่มี successful response contract จึงไม่ถือ HTTP 2xx ว่าสำเร็จหากอ่านเลขบิลไม่ได้ ค่า ALEX_ORDER_ID_POINTER เริ่มต้นว่างและให้ตรวจใน Alex ก่อนบันทึกเลขออเดอร์; template ไม่มีช่องค่าส่งแยก จึงต้องรับทราบว่าเก็บค่าส่งในหมายเหตุโดยไม่รวมในค่าเช่า; ยังไม่ commit/push/deploy และไม่แก้ server หยุด backend/frontend/Chrome/Mongo ทดสอบที่สร้างเองแล้ว คง Docker Desktop ไว้


## 2026-10-26 11:10 — ตรวจรวมงานสติกเกอร์/รูป + Step 3 ก่อน deploy
- **คำอธิบาย:** หลังงาน Step 3 (entry 11:00) เสร็จ ตรวจซ้ำว่า release ล่าสุดรวมงานทั้งสองฝั่งและพร้อม deploy — คำเตือน "ห้าม deploy" ใน entry 10:35 ใช้กับ release 10:33 (rel-20261026033312, front-20261026033426) เท่านั้น ซึ่งถูกแทนที่แล้ว
- **ไฟล์:** ไม่ได้แก้ source/config ในรอบนี้ (บันทึกผลตรวจเท่านั้น)
- **ผลตรวจสอบ:** release ปัจจุบัน `line-linux/` rel-20261026035638-9868a8c39c0b (10:56) และ `frontend-linux/` front-20261026035745-26c4e8e53c6d (10:57) ใหม่กว่า source ทุกไฟล์; backend checksum และ frontend checksum 2,108 ไฟล์ผ่าน ไม่มีไฟล์ release ถูก gitignore; cargo fmt --check, clippy --all-targets -D warnings ผ่าน; cargo test -- --include-ignored ผ่าน 30/30 (MongoDB disposable 27018); npm lint/build ผ่าน; git diff --check ผ่าน; Chrome headless บนโค้ดที่รวมแล้ว: ส่งสติกเกอร์ 446/1988, retry key เดิมหลัง LINE 500, รูป 4000×3000 → 2048×1536 + preview 240×180 เปิดได้โดยไม่ล็อกอิน, ไม่ล้น 390px, ไม่มี page error; หยุดบริการทดสอบที่สร้างเองแล้ว ยังไม่ commit/push/deploy

## 2026-10-26 11:00 — Step 3 ค้นสินค้า Alex และยืนยัน Product ID พร้อม MCP
- **คำอธิบาย:** ผู้ใช้ให้ทำข้อ 6 / Step 3 ต่อจากร่าง Step 2 และส่งตัวอย่าง product search กับบัญชี Alex ให้ทดสอบ อ่าน login/catalog จริงได้แล้ว พบสาขาที่บัญชีได้รับสิทธิ์ต่างจาก branchId ในตัวอย่าง จึงผูกสิทธิ์กับ shopInfoId/userBranch จาก login และให้แอดมินเลือก candidate พร้อมยืนยันทุกครั้ง เก็บ credentials เฉพาะในหน่วยความจำระหว่างทดสอบ ไม่เพิ่ม dependency
- **โฟลเดอร์ `backend/src/`**
  - `alex.rs` — เพิ่ม: server-side login, ตรวจร้าน/สาขา, POST GetProductItems ตาม contract ที่ผู้ใช้ให้ (15 รายการต่อหน้า, showNotAvailable=false, category/texture/สี/ไซซ์), กรอง code แบบตรงตัว, pagination จาก itemNumber, แปลงราคาเป็นสตางค์ และ proxy รูปเฉพาะ origin ที่อนุญาตโดยไม่ตาม redirect
  - `models/product.rs`, `models/mod.rs` — เพิ่ม/แก้: DTO ค้นหา/ยืนยัน, Candidate/Branch, ตรวจ input และ decimal parser ที่ไม่ใช้ float คำนวณเงิน
  - `db/products.rs`, `db/mod.rs`, `db/indexes.rs` — เพิ่ม/แก้: product_searches และ product_matches, unique index ต่อ shop/channel/inquiry/draft/item, ยืนยันจาก snapshot ที่มีอายุ 30 นาทีและอ่าน catalog ซ้ำ, CAS กันเขียนทับ, ผู้ยืนยัน/เวลา/วิธีจับคู่/ค่าที่ต่างจากร่าง, ปฏิเสธบริบทเก่าและข้อความยกเลิก ผลจับคู่เดิมอยู่กับร่างเก่าเมื่อเปลี่ยนเวอร์ชัน
  - `product_vision.rs`, `draft_worker.rs` — เพิ่ม/แก้: ใช้ Hermes JSON client ร่วมกับ Step 2 โดยคงการตรวจ extraction profile ปิด tools; อ่านรหัสจากรูปแล้วค้นตรงตัว หรือเปรียบเทียบกับรูปแค็ตตาล็อกในหน้าที่กรอง ตรวจ ID/score ที่โมเดลคืนให้ตรงรูปที่แนบและจำกัดภาพรวม 8 MB แอดมินยังต้องยืนยัน
  - `routes/products.rs`, `routes/mcp.rs`, `routes/mod.rs` — เพิ่ม/แก้: settings/search/confirm/list/image endpoints ภายใต้ admin auth/CSRF เดิม และ MCP JSON-RPC endpoint `/api/admin/alex/mcp` ที่ใช้ token แยก ตรวจ Origin รองรับ initialize/ping/tools/list/tools/call มีเพียง search_products และ find_product_candidates_by_image ไม่มีเครื่องมือยืนยันหรือสร้างออเดอร์
  - `config.rs`, `error.rs`, `main.rs` — แก้: config ALEX_* แบบ server-only, แยก error สำหรับ scope/schema/selection/รูปแค็ตตาล็อก และลงทะเบียน modules
  - `tests_products.rs`, `tests.rs` — เพิ่ม/แก้: ตรวจ Product URL, MCP lifecycle/auth/read-only, cross-shop/branch, ราคาเปลี่ยน, ID ปลอม, options ที่ต่าง, pagination, ร่างเก่า, image ranking/OCR/ID ที่โมเดลแต่ง และ unsend
- **โฟลเดอร์ `frontend/src/`**
  - `components/product-matcher.tsx`, `components/product-matcher.css` — เพิ่ม: เลือกรายการ/สาขา/วิธีค้นและตัวกรอง แสดงหลาย candidate พร้อม Product ID/สี/ไซซ์/ราคา/รูป/score, โหลดหน้าเพิ่ม, ยืนยันแบบ checkbox และยืนยันส่วนที่ต่างจากร่างเพิ่ม แสดงผู้ยืนยันและผลครบทุก item
  - `components/admin-workspace.tsx`, `app/globals.css`, `lib/api.ts` — แก้: แท็บเลือกชุด, ป้องกันทำงานกับสรุปที่ยังไม่บันทึก, tabs เลื่อนแนวนอนได้ และ error ภาษาไทย
- **โฟลเดอร์ `./` และ `docs/`**
  - `.env.example`, `docker-stack.line.yml` — แก้: ALEX_API_URL/USERNAME/PASSWORD/SHOP_INFO_ID/BRANCH_IDS/PRODUCT_URL_PREFIX/IMAGE_ORIGINS/MCP_TOKEN (ไม่มี credentials จริง)
  - `docs/step-3-products.md` — เพิ่ม: ตั้งค่า env และ shop/branch ที่ตรวจจริง, ขั้นตอนใช้งาน, API/data contract, ขอบเขตค้นภาพต่อหน้า, config ต่อ Hermes MCP คนละ profile กับ extraction และขั้นตอน build/deploy
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: สถานะ Step 3, ลิงก์คู่มือ และแก้ข้อความเก่าว่ายังไม่มี Alex adapter
- **โฟลเดอร์ `dist/` (build จริงบน Docker Desktop local)**
  - `line-linux/` — rel-20261026035638-9868a8c39c0b
  - `frontend-linux/` — front-20261026035745-26c4e8e53c6d
- **ผลตรวจสอบ:** cargo fmt --check และ clippy --all-targets -D warnings ผ่าน; cargo test -- --include-ignored ผ่าน 30/30 ด้วย MongoDB disposable 27018; npm lint/build ผ่านทั้ง Windows และ Linux; Chrome headless กับ backend/Mongo จริงและ Alex production แบบอ่านอย่างเดียวผ่าน login/catalog/scoped branch, exact Product ID, checkbox ยืนยัน, reload, pagination 15→30 candidates และ tools/call ผ่าน MCP ไม่พบ credentials ใน DOM ไม่มี page error และไม่ล้นที่ 390px; build runtime Docker images backend/frontend ผ่าน; full-stack Swarm config มี env Alex ครบและคง LINE network/Mongo volume; ตรวจ ELF amd64/backend checksum และ frontend checksum 2,108 ไฟล์ ไม่มี release ถูก ignore หรือมี .env; git diff --check ผ่าน
- **ขอบเขต:** ไม่สร้างออเดอร์/เช็คคิวหรือแก้สินค้าใน Alex จริง การค้นภาพทดสอบด้วย Hermes จำลองและยังต้องตั้ง vision-enabled extraction profile พร้อมรูปแค็ตตาล็อกจึงเปรียบเทียบได้ คงงาน Step 2/ธีม/ส่งสติกเกอร์และรูปที่มีอยู่แล้ว; ไม่ commit/push/deploy หรือแก้ .env บน server; หยุด backend/frontend/Chrome/Mongo ทดสอบที่สร้างเองแล้ว

## 2026-10-26 10:35 — แอดมินส่งสติกเกอร์และแนบรูปจาก Alex
- **คำอธิบาย:** ผู้ใช้ขอให้แอดมินส่งสติกเกอร์แบบ LINE OA และแนบรูปได้ ตรวจรายการสติกเกอร์ที่ Messaging API อนุญาตจากเอกสาร LINE (15 ชุด) และยืนยัน ID ครบทุกตัวจาก productInfo.meta ของแต่ละชุด (456 ตัว ต่อเนื่องทุกชุด); รูปต้องเป็น HTTPS URL สาธารณะ จึงเก็บใน GridFS และเสิร์ฟผ่าน URL token สุ่ม
- **โฟลเดอร์ `backend/src/`**
  - `models/outbound.rs` — แก้: STICKER_PACKAGES + `valid_sticker`, SendInput รับ text | sticker | image_id อย่างใดอย่างหนึ่ง (enum Outgoing), UploadInput ตรวจ base64/magic bytes JPEG/PNG และขนาด ≤10 MB/≤1 MB ตาม LINE พร้อม unit test
  - `db/outbound.rs` — แก้: push ข้อความทุกชนิด, ตรวจแชท 1:1, render ข้อความ LINE + payload ใน chat, idempotency hash ของข้อความจริง, upload เก็บ original/preview ใน GridFS (media::store เดิม) + collection `outbound_media` (token HMAC 64 hex, หมดอายุตาม LINE_MEDIA_RETENTION_DAYS), read สำหรับ URL สาธารณะ, รูปต้องเป็นของ inquiry เดียวกัน
  - `routes/customers.rs`, `routes/mod.rs` — เพิ่ม: POST `/api/inquiries/{id}/media` (admin, body limit 16 MB) และ GET สาธารณะ `/api/line/outbound-media/{token}/{original|preview}` (nosniff)
  - `config.rs`, `error.rs`, `db/indexes.rs` — แก้: `PUBLIC_BASE_URL` (ค่าเริ่มต้น = origin https ของ LINE_LOGIN_REDIRECT_URI), error IMAGE_UNAVAILABLE (503), unique index outbound_media token
  - `tests.rs`, `tests_line.rs` — แก้/เพิ่ม: ไม่มี HTTPS base ปฏิเสธก่อนแตะ DB, ส่งสติกเกอร์/ID ที่ไม่อนุญาต, อัปโหลด GIF ถูกปฏิเสธ, URL สาธารณะส่ง bytes เดิม + nosniff, ส่งรูปด้วย originalContentUrl/previewImageUrl ที่ถูกต้อง, ใช้รูปข้าม inquiry ไม่ได้, ลำดับ text/sticker/image ใน context
- **โฟลเดอร์ `frontend/src/`**
  - `components/reply-composer.tsx` — เพิ่ม: ช่องตอบแยก component (ข้อความ, ปุ่ม 😊 แผงสติกเกอร์ 15 ชุดกดแล้วส่งทันที, ปุ่ม 🖼️ แนบรูป ย่อด้วย canvas เป็น JPEG ≤2048px + preview 240px แล้วแสดงตัวอย่างก่อนกดส่ง, ปุ่มส่งอีกครั้งด้วย request ID เดิม)
  - `lib/stickers.ts` — เพิ่ม: รายการสติกเกอร์ชุดเดียวกับ backend พร้อมชื่อชุดภาษาไทย
  - `components/admin-workspace.tsx` — แก้: ใช้ ReplyComposer (key ตาม inquiry) แทนฟอร์มเดิม ลบ state/ฟังก์ชันตอบเดิม
  - `components/chat-message.tsx`, `lib/api.ts`, `app/globals.css` — แก้: แสดงรูปที่ส่งจาก Alex, error IMAGE_UNAVAILABLE, สไตล์แผงสติกเกอร์/ตัวอย่างรูป
- **โฟลเดอร์ `./` และ `docs/`**
  - `docker-stack.line.yml`, `.env.example` — แก้: `PUBLIC_BASE_URL` (ตรวจด้วย docker stack config)
  - `docs/step-2-admin.md` — แก้: วิธีส่งสติกเกอร์/รูป ข้อจำกัด และ API
- **ผลตรวจสอบ:** cargo clippy -D warnings ผ่าน และ cargo test -- --include-ignored ผ่าน 26/26 (ก่อนพบงานคู่ขนานด้านล่าง); npm lint/build ผ่าน; Chrome headless กับ backend local + mock LINE: แผงสติกเกอร์ 40 ตัว/ชุด ส่ง 446/1988 ถูกต้อง, LINE 500 แล้วกดส่งสติกเกอร์อีกครั้งใช้ retry key เดิม, รูป PNG 4000×3000 ย่อเป็น JPEG 2048×1536 (1.6 MB) และ preview 240×180 (20 KB) เปิด URL ได้โดยไม่ล็อกอิน, ไม่ล้นที่ 390px, ไม่มี page error
- **⚠️ ห้าม deploy release ชุดนี้:** ระหว่าง build พบ agent อีกตัวกำลังเพิ่มการเชื่อม Alex API (`backend/src/alex.rs`, `models/product.rs`, `config.rs` ALEX_*, `main.rs` สร้าง/แก้ 10:31) ยังไม่ผ่าน cargo fmt และยังไม่มี entry ใน log; release ที่ build 10:33 — `line-linux/` rel-20261026033312-e452fcd91f50 และ `frontend-linux/` front-20261026033426-50ec5812b20d — จึงอาจมีโค้ดที่ยังไม่เสร็จของงานนั้น ต้อง build-backend.bat และ build-frontend.bat ใหม่หลังงาน Step 3 เสร็จและผ่าน fmt/clippy/test ก่อน rungit; ไม่ได้แก้หรือ format ไฟล์ของงานนั้น; หยุดบริการทดสอบที่สร้างเองแล้ว

## 2026-10-26 10:20 — เติม Step 2 ให้บันทึกสรุปแอดมินและกู้กลับหลัง AI ล้มเหลวได้
- **คำอธิบาย:** ตรวจพบว่า Step 2 มีระบบสกัดร่าง/หลักฐาน/ประวัติเวอร์ชันแล้ว แต่หน้าเว็บโหลดสรุปจากร่างที่สำเร็จเท่านั้น ทำให้สรุปใหม่หายจากหน้าจอเมื่อ AI ล้มเหลวแล้วเปิดใหม่ จึงเพิ่มการบันทึกสรุปและรูปก่อนเรียก AI พร้อมป้องกันการเขียนทับและการใช้สรุปหรือแชทรุ่นเก่า ตามขอบเขตข้อ 5 ของ workflow โดยไม่เพิ่ม dependency
- **โฟลเดอร์ `backend/src/`**
  - `models/draft.rs` — เพิ่ม/แก้: SummaryInput, ตรวจ revision/hash/รูปซ้ำ, จำกัดสรุป 20,000 ตัวอักษรให้รองรับภาษาไทยตาม UI และป้องกัน version overflow
  - `db/drafts.rs` — เพิ่ม/แก้: บันทึก `inquiries.working_summary` แบบ compare-and-set พร้อม revision/ผู้แก้ไข/เวลา/รูป/บริบท, ซ่อนสรุปที่อ้างข้อความยกเลิก, ผูกงานและร่างกับ revision ของสรุป และทำให้ร่างรุ่นเก่าเป็น stale
  - `routes/drafts.rs`, `routes/mod.rs`, `error.rs` — เพิ่ม/แก้: POST `/api/inquiries/{id}/summary` ภายใต้ admin authorization/CSRF เดิม, ส่ง summary ใน GET drafts และ error `SUMMARY_CONFLICT`
  - `draft_worker.rs`, `db/orders.rs` — แก้: ตรวจ context_hash ของทั้ง AI และ manual, ปฏิเสธงานเมื่อสรุปเปลี่ยนก่อน/ระหว่างประมวลผล, fence revision ตอน commit และไม่อนุมัติร่างที่อ้างสรุปรุ่นเก่า
  - `tests_step2.rs` — เพิ่ม: บันทึกสรุปได้โดยไม่ต้องมี Hermes, เปิดกลับ, concurrent edits, ข้ามร้าน, รูปที่ไม่ได้อยู่ใน inquiry, แชทเก่า, unsend, stale approval และสรุปเปลี่ยนระหว่าง AI ทำงาน
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx` — เพิ่ม/แก้: ปุ่มบันทึกสรุป, แสดงผู้บันทึก/เวลา/ข้อมูลที่ยังไม่บันทึก, คืนรูปที่เลือกเมื่อเปิดใหม่, บันทึกก่อน AI, คงข้อความที่กำลังแก้เมื่อ refresh/polling หรือเกิด conflict และปุ่มโหลดสรุปล่าสุด; คงการแก้ธีมและกด inquiry เดิมซ้ำที่มีอยู่ก่อนเริ่มงาน
  - `lib/api.ts` — เพิ่ม: AdminSummary และข้อความ error ภาษาไทยสำหรับ summary conflict/รอข้อความประมวลผล
- **โฟลเดอร์ `docs/` และ `./`**
  - `docs/step-2-admin.md`, `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — เพิ่ม/แก้: วิธีบันทึกสรุป/รูป, recovery, API/revision และระบุช่องทางปัจจุบันว่าตอบลูกค้าจาก Alex ส่วนตัวเชื่อม OA เป็นทางเลือก
- **โฟลเดอร์ `dist/` (build จริงบน Docker Desktop local)**
  - `line-linux/` — rel-20261026031818-456623f6ba3c
  - `frontend-linux/` — front-20261026031928-856680f91764 (รวมการแก้ธีมจากงานก่อนหน้าที่ค้าง build)
- **ผลตรวจสอบ:** cargo fmt --check/clippy -D warnings ผ่าน; cargo test -- --include-ignored ผ่านครบ 23/23 ด้วย MongoDB disposable พอร์ต 27019; npm lint/build ผ่านทั้ง Windows และ Linux release; Chrome headless กับ backend/Mongo จริง + mock Hermes ผ่านการบันทึกสรุป/รูปแล้ว reload, คงข้อความขณะ refresh, แก้สรุปชนสองหน้าต่าง, จำลอง AI ตอบ 503 แล้ว reload/retry จนได้ร่าง, แก้ร่างด้วยมือ/ประวัติ/ยืนยันข้อมูล/เงินสตางค์ และไม่ล้นที่ 390px; ตรวจ Linux ELF/backend checksum และ frontend checksum ครบ 2,108 ไฟล์ ไม่มี release ถูก ignore และไม่มี .env ใน release; git diff --check ผ่าน
- **ขอบเขต:** ทดสอบ Hermes จำลอง ยังไม่ได้ทดสอบ extraction profile จริงหรือเชื่อม Alex API/MCP; ไม่ commit/push/deploy และไม่แก้ server; หยุด backend/frontend/Chrome/Mongo ทดสอบที่สร้างเองแล้ว คง Docker Desktop ไว้ให้ใช้งาน build ต่อ

## 2026-10-26 10:09 — แก้โหลดแชทค้างเมื่อกดเรื่องเดิมซ้ำ, ขยายตัวอักษร และเปลี่ยนธีมเป็นขาว-ฟ้า-น้ำเงิน
- **คำอธิบาย:** ผู้ใช้พบว่ากด inquiry เดิมครั้งที่ 2 แล้วค้างที่ "กำลังโหลดบทสนทนา" สาเหตุคือ `selectInquiry` ล้างข้อมูลและตั้ง loading=true แต่ effect โหลดข้อมูลรันเฉพาะเมื่อ selected เปลี่ยน; ผู้ใช้ขอให้ตัวอักษรใหญ่ขึ้น และเปลี่ยนธีมหลักเป็นขาว ฟ้า น้ำเงิน โดยมีเขียว ส้ม ชมพูเป็นสีเสริม
- **โฟลเดอร์ `frontend/src/`**
  - `components/admin-workspace.tsx` — แก้: กดเรื่องที่เปิดอยู่ซ้ำไม่รีเซ็ต (คงสรุป/ข้อความที่พิมพ์ค้าง); ไอคอนสถิติแรกใช้ class `blue`
  - `app/globals.css` — แก้: แปลงสีโทนเขียวทั้งหมดด้วยตารางสีเป็นขาว/ฟ้า (sky)/น้ำเงิน (blue-700 สำหรับปุ่มและตัวอักษร) และเทาอมฟ้า (slate-500 ผ่าน AA); `@theme` มี primary/info/success/accent/highlight; คงเขียวเฉพาะสำเร็จ/ออนไลน์/ป้าย LINE/การ์ดอนุมัติ, ส้มสำหรับเตือน, ชมพูสำหรับป้ายเวอร์ชัน; ขยาย font-size ที่ต่ำกว่า 20px (เช่น 8→11, 9→12, 10→13, 11→14, 12→15, body 14→16) และขยายคอลัมน์ (sidebar 228px, workspace 280/300+/370–460px, login card 420px)
- **โฟลเดอร์ `./`**
  - `AGENTS.md` — แก้: ข้อ 6 ตารางสีใหม่ (primary น้ำเงิน, info ฟ้า, option เขียว/ส้ม/ชมพู) และขนาดตัวอักษรขั้นต่ำ
- **ผลตรวจสอบ:** npm lint/build ผ่าน; Chrome headless กับ backend local + mock LINE: กดเรื่องเดิมซ้ำ 3 ครั้งแชทยังแสดง 2 ข้อความไม่ค้าง, สลับเรื่อง A→B→A ถูกต้อง, ข้อความที่พิมพ์ค้างไม่หายเมื่อกดเรื่องเดิม, body/bubble 16px, ปุ่มหลัก/แท็บที่เลือกเป็น rgb(29,78,216), ไม่ล้นที่ 1280/1024/390px, ไม่มี page error
- **ยังไม่เสร็จ:** build frontend Linux release ไม่ได้เพราะ Docker Desktop ไม่ได้เปิด (`dockerDesktopLinuxEngine` pipe ไม่พบ) — `dist/frontend-linux` ยังเป็น front-20261025170420-588f29abdf8d ซึ่งไม่มีการแก้รอบนี้ ต้องเปิด Docker Desktop แล้วรัน build-frontend.bat ก่อน rungit; backend ไม่ได้แก้ ใช้ rel-20261025170323-3ccacb444b06 เดิม; หยุดบริการทดสอบที่สร้างเองแล้ว

## 2026-10-26 09:45 — เพิ่มขั้นตอน deploy มาตรฐานใน guideline
- **คำอธิบาย:** ผู้ใช้ให้นำขั้นตอน deploy (1. Windows build + rungit, 2. Ubuntu loadgit/docker-build/docker-deploy) ไปไว้ใน guideline; ตรวจแล้วว่าเอกสารเดิมยังเขียนแบบไม่มี `--full-stack` ซึ่งไม่ตรงกับปัจจุบันที่หน้า admin รันบน server และ `deploy-line.sh` ไม่ใช้ `--prune` ถ้าไม่ใส่ `--full-stack` frontend จะค้างรุ่นเก่า
- **โฟลเดอร์ `./`**
  - `AGENTS.md` — แก้: ข้อ 4 ตารางคำสั่งใช้ `--full-stack`; ข้อ 8 เปลี่ยนบรรทัด "ลำดับบน server" เป็นบล็อก "ขั้นตอน deploy มาตรฐาน" (Windows: build เฉพาะส่วนที่แก้ + rungit.bat, Ubuntu: 3 คำสั่ง `--full-stack`, กรณีแก้แค่ .env, ห้ามรัน .sh บน Windows); ข้อ 11 ให้แนบขั้นตอน deploy และ release tag ในสรุป
  - `alex-line-hermes-mcp-workflow.md`, `alex-line-hermes-mcp-workflow.html` — แก้: หัวข้อ 4.2 และการ์ดผู้พัฒนา/ผู้ดูแลระบบใน HTML ใช้ขั้นตอนเดียวกัน (`--full-stack`, backend + frontend + MongoDB); ตรวจ HTML แล้ว `<pre>` 34/34, `<code>` 298/298 และรูปฝัง 4 ภาพครบ

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

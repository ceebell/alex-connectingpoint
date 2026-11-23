# Instagram Integration Requirements

รายการงานจากคำขอให้เชื่อมต่อ Instagram และสรุปสิ่งที่ต้องทำ สถานะทั้งหมดเป็น Not started เพราะรอบนี้จัดทำแผน ยังไม่ได้พัฒนาหรือเชื่อมบัญชีจริง

ขอบเขตเสนอสำหรับเฟสแรก: รับ–ตอบ Instagram DM ในหน้าหลังบ้านเดิม เริ่มหนึ่งบัญชีร้าน ส่วนการเชื่อม AI/Alex, คอมเมนต์, Private Reply และการโพสต์เป็นเฟสต่อยอด ต้องยืนยันขอบเขตก่อนพัฒนา

## Account and Meta App Setup

[Not started]
- เตรียม Instagram Professional Account ประเภท Business หรือ Creator, ผู้ดูแลที่มีสิทธิ์, Meta Developer App และ public HTTPS domain สำหรับ OAuth callback และ webhook ใช้ Unified Login ตามแนวทางล่าสุดของ Meta สำหรับระบบใหม่ โดยต้องมี business portfolio และไม่บังคับมี Facebook Page; หากเลือก Instagram Login ให้บันทึกเหตุผลและใช้ permission/token/API host ของเส้นทางนั้นอย่างสอดคล้องกัน

[Not started]
- ตั้งค่า permissions ขั้นต่ำสำหรับ DM: Unified Login ใช้ instagram_basic และ instagram_manage_messages พร้อมสิทธิ์เพิ่มเติมเฉพาะที่ endpoint/การจัดการ token ต้องใช้; Instagram Login ใช้ instagram_business_basic และ instagram_business_manage_messages ทดสอบบัญชีที่ดูแลเองและ app roles ก่อน หากรับบัญชีภายนอกให้เตรียม Business Verification, App Review, Advanced Access และ Live mode พร้อม Privacy Policy, ช่องทางลบข้อมูล, บัญชีทดสอบและวิดีโอสาธิตตามข้อกำหนด Meta

## Backend and Data

[Not started]
- เพิ่มการเชื่อมต่อ/ยกเลิก/เชื่อมต่อใหม่ เก็บ token ฝั่ง server อย่างปลอดภัย ตรวจ OAuth state และสิทธิ์ admin เก็บ Instagram account ID, business ID และ config_id ตาม login ที่เลือก จัดการ token หมดอายุ/ถูกถอนสิทธิ์และแจ้งสถานะบัญชี ไม่ใส่ secret ใน frontend, log หรือ Git

[Not started]
- เพิ่ม webhook ใต้ /api สำหรับ verification และตรวจ signature ก่อนรับข้อมูล subscribe messaging events ที่ต้องใช้ บันทึกแบบทนต่อ retry, duplicate, event มาผิดลำดับ, ข้อความจากแอดมินใน Instagram และการ unsend/delete ป้องกันข้อความสะท้อนกลับซ้ำ จัดการ media URL หมดอายุและชนิดข้อมูลที่ไม่รองรับ

[Not started]
- ขยาย identity และ indexes ของลูกค้า/แชท/ข้อความให้แยกตาม shop, platform, account และ platform user ID โดยรักษาข้อมูล LINE เดิม ตรวจจุดที่ผูกกับ line_user_id และ channel_id ในสิทธิ์ร้าน, Note, cart, draft, checkout และ tracking ไม่จับคู่คนข้าม LINE/Instagram อัตโนมัติจากชื่อเหมือนกัน วางแผน migration แบบย้อนกลับได้

[Not started]
- เพิ่ม Instagram API client รับ–ส่งข้อความและรูปสำหรับเฟสแรก พร้อมสถานะส่ง, retry/backoff, rate limit และการจัดการผลส่งไม่แน่นอน แยกกฎส่งออกจาก LINE ตรวจ messaging window จาก event จริงก่อนส่ง ข้อความทั่วไปส่วนมากอยู่ใน 24 ชั่วโมง; ข้อยกเว้น Click-to-Direct และ Human Agent ต้องใช้ตามสิทธิ์และเงื่อนไข Meta ไม่ใช้ Human Agent สำหรับข้อความอัตโนมัติหรือ AI ส่งเอง

## Admin Inbox

[Not started]
- เพิ่มปุ่มเชื่อมต่อ Instagram, สถานะบัญชี, ตัวกรอง LINE/Instagram, ป้ายช่องทาง, รายการและรายละเอียดแชท, unread ภายในระบบ, การแจ้งเตือน และการตอบข้อความ/รูป ใช้ UI ภาษาไทย แสดงเหตุผลเมื่อส่งไม่ได้และแยกสถานะอ่านในระบบออกจากสถานะอ่านของ Instagram นำ quick replies แบบข้อความ/รูปมาใช้ตามความสามารถ API ไม่รับประกันดึงประวัติย้อนหลังได้ทั้งหมด

## Follow-up Features

[Not started]
- ประเมินและยืนยันขอบเขตเชื่อม Note, ค้นข้อมูลลูกค้า, AI อ่านแชท/รูป/สลิป, ร่างออเดอร์, cart, Alex checkout และข้อความ tracking ให้รองรับ Instagram โดยรักษา evidence, สิทธิ์ร้าน/สาขา และการตรวจต้นทางเดิม ส่งบิล/Tracking ได้เมื่อกฎ messaging window อนุญาตเท่านั้น และกำหนดทางเลือกเมื่อพ้นเวลา

[Not started]
- หากต้องการ ให้แยกคอมเมนต์/Private Reply, ตอบกลับ Story, โพสต์รูป/Reels และ Insights เป็นงานเพิ่มแต่ละฟีเจอร์ ขอ permission และทดสอบเงื่อนไขของ endpoint นั้นก่อนพัฒนา ไม่รวมการ broadcast หาใครก็ได้ในขอบเขต DM

## Verification and Release

[Not started]
- ทดสอบ webhook ปลอม/ซ้ำ/unsend, token ถูกถอน, สิทธิ์ข้ามร้าน, media, rate limit, send retry และ messaging window รวม regression LINE เดิม ผ่าน cargo fmt --check, cargo clippy -- -D warnings, cargo test, npm run lint และ npm run build ตามส่วนที่แก้ ก่อน build Linux release บน Windows ด้วย build-all.bat เพิ่ม env ตัวอย่างและส่ง env ที่จำเป็นผ่าน docker-stack.line.yml ตามโครงสร้างเดิม ให้ Developer ส่ง Git แล้ว deploy บน Ubuntu ด้วย bash ./deploy-all.sh main ใช้ precompiled release เท่านั้น

## Reference Documentation

ตรวจเอกสาร Meta เมื่อ 2026-11-22 ต้องตรวจซ้ำก่อนเริ่มพัฒนาเนื่องจาก API และนโยบายอาจเปลี่ยน

- [Instagram Platform overview](https://developers.facebook.com/documentation/instagram-platform/overview)
- [Instagram API with Unified Login](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-unified-login)
- [Unified Login permissions](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-unified-login/permissions)
- [Unified Login authentication and tokens](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-unified-login/authentication)
- [Instagram Login messaging reference](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-instagram-login/messaging-api)
- [Instagram Messaging overview](https://developers.facebook.com/documentation/business-messaging/instagram-messaging/overview)

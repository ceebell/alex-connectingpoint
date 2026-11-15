# แจ้งเตือนข้อความใหม่ใน Alex

## ระบบที่มีอยู่

หน้า Home มีปุ่ม **เปิดแจ้งเตือนข้อความ** ในส่วนบัญชีที่ใช้งาน ให้เปิดผ่าน HTTPS แล้วอนุญาตทั้งในเบราว์เซอร์และการตั้งค่าแจ้งเตือนของเครื่อง

โค้ดปัจจุบันอ่านรายการแชทประมาณทุก 10 วินาทีและแสดง browser notification ผ่าน Service Worker ต้องเปิดหน้า Home/รายการแชทของ Alex ค้างไว้ เบราว์เซอร์อาจหน่วง timer ในพื้นหลัง ระบบยังไม่มี Push subscription หรือ backend ส่ง Web Push เมื่อปิดหน้าเว็บ

แจ้งเฉพาะข้อความใหม่ที่รับเข้ามา ไม่ย้อนแจ้งข้อความเก่าตอนเปิดหน้า และไม่แจ้งแชทที่กำลังเปิดอ่านอยู่ ทดสอบโดยเปิดอีกแชทหรือพักแท็บ Alex แล้วส่งข้อความใหม่จาก LINE

ไฟล์หลัก: `frontend/src/lib/use-chat-notifications.ts`, `chat-notifications.ts`, `frontend/public/alex-notifications-sw.js`

## เส้นทาง Caddy ที่ต้องเปิด

Service Worker ต้องอ่านได้จาก `https://hermes.salepie.online/alex-notifications-sw.js` และตอบ HTTP 200 พร้อม JavaScript

ผลตรวจ 15/11/2026: เส้นทางบนโดเมนตอบ 404 จาก Hermes แต่ไฟล์เดียวกันที่ `127.0.0.1:18081` ตอบ 200 JavaScript สาเหตุคือ matcher `@alex` ใน `/etc/caddy/Caddyfile` ยังไม่มีชื่อไฟล์นี้ ต้องเพิ่ม `/alex-notifications-sw.js` ในรายการ `path` ให้ส่งไป frontend ตาม `docker/Caddyfile.line.example` การ deploy โปรแกรมอย่างเดียวไม่ได้แก้ Caddyfile ของ host

รันบน Ubuntu หลังสำรองและแก้ matcher:

```bash
sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile &&
sudo systemctl reload caddy
curl -I https://hermes.salepie.online/alex-notifications-sw.js
```

เมื่อเส้นทางตอบ 200 แล้ว โหลดหน้า Alex ใหม่และกดเปิดแจ้งเตือนอีกครั้ง การแก้ route นี้ทำให้ระบบแจ้งเตือนเดิมลงทะเบียน worker ได้ แต่ไม่ได้เพิ่ม background Web Push

## รับตอนที่ไม่ได้เปิดหน้าเว็บ

แนะนำเพิ่ม PWA และ Web Push มาตรฐานบน backend Ubuntu ที่มีอยู่ ไม่จำเป็นต้องสร้าง Firebase project โดยเพิ่ม:

1. เว็บแอปสำหรับเพิ่มไอคอนบน Home Screen พร้อม manifest/icons และให้ผู้ใช้กดอนุญาตแจ้งเตือน
2. การลงทะเบียนเครื่องที่รับแจ้งเตือน (Push subscription) ผูกกับพนักงานและร้าน เก็บใน MongoDB และใช้ VAPID สำหรับส่งจาก server
3. Backend ส่ง Push เมื่อได้รับข้อความใหม่ และ Service Worker รับ push event แล้วแสดงแจ้งเตือนพร้อมลิงก์กลับแชท

Web Push รับข้อความจาก server ได้แม้หน้าเว็บไม่อยู่ด้านหน้าหรือไม่ได้โหลดอยู่ ตาม [MDN Push API](https://developer.mozilla.org/en-US/docs/Web/API/Push_API) การส่งและการแสดงยังขึ้นกับสิทธิ์ เครือข่าย และการตั้งค่าของเครื่อง

สำหรับ iPhone/iPad ตั้งแต่ iOS/iPadOS 16.4 ต้องเพิ่มเว็บเป็นแอปบน Home Screen เปิดจากไอคอนนั้น แล้วกดอนุญาตแจ้งเตือนจากการแตะปุ่มในแอป การเปิด HTTPS ในแท็บ Chrome เพียงอย่างเดียวไม่ครบเงื่อนไข ตาม [WebKit: Web Push บน iOS/iPadOS](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/)

[Firebase Cloud Messaging](https://firebase.google.com/docs/cloud-messaging/web/get-started) เป็นอีกตัวเลือกสำหรับจัดการส่ง Push แต่ยังต้องตั้งค่า SDK, Service Worker, VAPID และสิทธิ์เบราว์เซอร์ ไม่ได้ข้ามเงื่อนไขของ iOS งานตรวจครั้งนี้ยังไม่ได้เพิ่ม PWA/Web Push/FCM หรือสร้างคีย์ใหม่

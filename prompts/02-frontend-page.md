# 02 — เพิ่มหน้าเว็บ / Component ฝั่ง Frontend (Next.js + Tailwind)

> แก้ค่าใน `{{ }}` ก่อนส่ง

---

อ่าน `AGENTS.md` (ข้อ 6) และ entry ล่าสุดใน `revising-log.md` ก่อน แล้วสร้างหน้าต่อไปนี้

**หน้า:** `{{เช่น รายการคำสั่งซื้อ}}`  ·  **Route:** `{{/orders}}`

**เนื้อหาและพฤติกรรม**

- `{{แสดงตารางคำสั่งซื้อจาก GET /api/orders พร้อม badge สถานะ}}`
- `{{ปุ่ม "เพิ่มคำสั่งซื้อ" เปิดฟอร์ม → POST /api/orders}}`
- `{{ลบรายการพร้อมกล่องยืนยัน}}`
- สถานะ: loading (skeleton), error (ข้อความไทยที่เข้าใจง่าย + ปุ่มลองใหม่), empty state (emoji + คำอธิบาย + ปุ่มเริ่มต้น)

**Design system (ตาม `AGENTS.md`)**

- Font Mitr, Tailwind เท่านั้น (ไม่เขียน CSS แยก นอกจาก token ใน `globals.css`)
- Layout แบ่งเป็น section สลับพื้น `bg-white` / `bg-gray-50`
- สี: `primary` เขียว = ปุ่มหลัก/สำเร็จ · `accent` ส้ม = ไฮไลต์/เตือน · `info` ฟ้า = ลิงก์/ข้อมูล
- ตัวอักษรบนพื้นสีใช้เฉด 700 เพื่อให้ contrast ผ่าน
- ใช้ emoji ประกอบตามสมควร (หัวข้อละไม่เกิน 1 ตัว, ของตกแต่งใส่ `aria-hidden="true"`)
- Responsive (มือถือก่อน) · ปุ่มและฟอร์มใช้งานด้วยคีย์บอร์ดได้ มี label ครบ

**ข้อกำหนดทางเทคนิค**

- App Router · ใช้ Server Component เป็นค่าเริ่มต้น ใช้ `"use client"` เฉพาะส่วนที่ต้องโต้ตอบ
- เรียก API ผ่าน `src/lib/api.ts` แบบ relative `/api/...` — ห้าม hardcode URL backend
- แยก component ที่ใช้ซ้ำไว้ `src/components/`
- TypeScript strict · ห้ามใช้ `any`
- ไม่เพิ่ม dependency ใหม่โดยไม่บอกเหตุผล

**ก่อนจบ:** `npm run lint` · `npm run build` — รายงานผลจริง
จากนั้นเพิ่ม entry ใน `revising-log.md` (วัน-เวลาจากเครื่องจริง, โฟลเดอร์, ไฟล์, สิ่งที่แก้) และไม่ commit/push เอง

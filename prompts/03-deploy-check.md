# 03 — ตรวจ / แก้ปัญหา Build และ Deploy

> เลือกใช้ส่วนที่ตรงกับงาน แล้วแก้ค่าใน `{{ }}`

---

## A) ตรวจความพร้อมก่อน deploy (review อย่างเดียว ยังไม่แก้ไฟล์)

อ่าน `AGENTS.md` ข้อ 8 แล้วตรวจชุดไฟล์ deploy ว่าสอดคล้องกัน และรายงานเป็นรายการ "ผ่าน / ไม่ผ่าน / ไม่แน่ใจ" พร้อมเหตุผล:

1. `docker/backend.Dockerfile`: ชื่อ binary ตรงกับ `Cargo.toml` (`backend`), มี `Cargo.lock`, healthcheck ตรงกับ `GET /api/health`
2. `docker/frontend.Dockerfile`: `next.config.ts` มี `output: "standalone"` และ rewrites ใช้ `BACKEND_URL`, มี `frontend/public/`
3. `docker-stack.yml`: ชื่อ service `backend` ตรงกับ `BACKEND_URL` (`http://backend:8080`), env ที่ backend อ่านครบตามที่ `config.rs` ใช้
4. `docker-build.sh` / `docker-deploy.sh`: ใช้ tag เดียวกัน (git SHA) และชื่อ image ตรงกับที่ stack อ้าง
5. `.env.example` ครอบคลุมทุกตัวแปรที่โค้ดและ stack ใช้
6. ไม่มี secret ถูก commit หรือถูก copy เข้า image (`.gitignore`, `.dockerignore`)

ถ้ามี Docker ในเครื่อง ให้รัน `docker build` ของทั้งสอง image ด้วย (รายงานผลจริง)

## B) แก้ปัญหา build / deploy

**อาการ:** `{{วาง error จริงทั้งหมดที่นี่}}`
**เกิดตอน:** `{{runall.bat | build.bat | loadgit.sh | docker-build.sh | docker-deploy.sh | หลัง deploy}}`
**สภาพแวดล้อม:** `{{Windows 11 / Ubuntu 24.04 / Docker version}}`

ขั้นตอน:

1. อ่าน error แล้วระบุสาเหตุที่น่าจะเป็นที่สุด พร้อมหลักฐานจากไฟล์ใน repo
2. เสนอวิธีแก้ที่แก้น้อยที่สุด — ถ้าต้องแก้สคริปต์หรือ Dockerfile ให้บอกว่าแก้ที่ไหนและเพราะอะไร
3. แก้แล้วทดสอบซ้ำตามที่ทำได้ รายงานผลจริง
4. เพิ่ม entry ใน `revising-log.md` และไม่ commit/push เอง

## C) คำสั่งตรวจสถานะบน server (Ubuntu)

```bash
docker stack services {{myapp}}
docker service ps {{myapp}}_backend --no-trunc
docker service logs --tail 100 {{myapp}}_backend
docker service logs --tail 100 {{myapp}}_frontend
docker service rollback {{myapp}}_backend      # ถอยกลับเมื่อ deploy พัง
```

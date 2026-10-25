# Build release บนเครื่อง local ด้วยไฟล์ BAT

รันจาก root ของโปรเจกต์บน Windows และเปิด Docker Desktop แบบ Linux containers เพื่อสร้าง Linux/amd64 release สำหรับ Ubuntu การ compile/build เกิดบนเครื่อง local เท่านั้น

| งาน | คำสั่ง | ผลลัพธ์ |
| --- | --- | --- |
| Compile Rust backend | `build-backend.bat` | `dist/line-linux/backend` พร้อม checksum/tag/manifest |
| Build frontend | `build-frontend.bat` | `dist/frontend-linux/` พร้อม server.js, dependencies, public/static, checksum/tag/manifest |

## Rust backend

```bat
build-backend.bat
```

เลือกจำนวนงาน compile ได้ เช่น `build-backend.bat -Jobs 4` ชื่อเดิม `build-line.bat` ยังใช้งานได้ สคริปต์ตรวจว่า builder เป็น Docker Desktop บนเครื่องนี้และตรวจ output ว่าเป็น ELF Linux x86_64

## Frontend

มี frontend หลังบ้านใน repository แล้ว ใช้ `frontend/package.json`, `frontend/package-lock.json` และ script `build` พร้อมตั้ง `output: "standalone"` ใน next.config.ts ดูการตั้งบัญชีแอดมิน/Hermes ใน [คู่มือ Step 2](step-2-admin.md)

```bat
build-frontend.bat
```

กำหนด backend URL ที่ฝังใน rewrites ตอน build ได้:

```bat
build-frontend.bat -BackendUrl "http://backend:8080"
```

สคริปต์ใช้ npm ci และ npm run build ภายใน Linux container บน local พร้อมรัน lint หากโปรเจกต์มี script lint แพ็ก standalone รวม runtime dependencies, public และ .next/static โดยใช้ Node/Debian รุ่นเดียวกับ runtime Dockerfile ค่า .env ของ local ไม่ถูกส่งเข้า compiler

รูปแบบ standalone และการแนบ public/static อ้างอิง [เอกสาร Next.js](https://nextjs.org/docs/app/api-reference/config/next-config-js/output)

ตรวจ checksum ทั้งชุดก่อนใช้บน server และสร้าง tag frontend แยกจาก backend จึงอัปเดต frontend ได้แม้ backend ไม่เปลี่ยน หาก build ล้มเหลวจะคง release เดิมไว้ เมื่อสำเร็จจะเก็บ release เก่าไว้ใน data/front-backup-* และแทนที่ output ทั้งโฟลเดอร์เพื่อไม่ให้ไฟล์เก่าปน

## ส่งขึ้น Git และใช้งานบน Ubuntu

Git เก็บ Linux release ทั้งสองส่วน รวม dependencies ที่จำเป็นใน frontend standalone ส่วน .env, Windows output, archives และ cache ถูก ignore ใช้ rungit.bat เพื่อ commit/push เมื่อพร้อม:

```bat
rungit.bat "Update compiled releases"
```

บน Ubuntu สำหรับ backend อย่างเดียว:

```sh
bash ./loadgit.sh main
sudo bash ./docker-build.sh
sudo bash ./docker-deploy.sh
```

เมื่อมี frontend release แล้ว ใช้ `docker-build.sh --full-stack` และ `docker-deploy.sh --full-stack` เพื่อสร้าง image/เปิดบริการทั้งสองส่วน Ubuntu ใช้ compiled objects โดยไม่มี cargo build, npm install หรือ npm run build

`build.bat` เดิมเป็นชุดสร้าง Windows release backend + frontend ส่วน BAT สองคำสั่งด้านบนใช้เตรียม release สำหรับ Ubuntu

# ส่ง Linux release จาก local ไป deploy บน Ubuntu

**ผู้พัฒนา compile/build โปรแกรมบนเครื่อง local เท่านั้น** แล้วส่ง Linux release ที่เสร็จแล้วให้ผู้ดูแลระบบ Ubuntu ใช้ Dockerfile ที่ COPY binary เข้า runtime image ไม่มี Rust/Cargo/npm build บน server

```text
ผู้พัฒนา: Windows + Docker Desktop → compile Linux/amd64 → release.tar.gz
ผู้ดูแลระบบ: Ubuntu → รับ release → ตรวจ checksum → ใส่ binary ใน Docker → deploy Swarm
ใช้งานจริง: LINE → Caddy → backend :18080 → MongoDB
```

## 1. ผู้พัฒนา — build บน local

เปิด Docker Desktop บน Windows และเลือก Linux containers แล้วรันที่ root ของโปรเจกต์:

```bat
build-line.bat
```

สคริปต์ใช้ builder ที่ผูกกับ Docker Desktop ผ่าน named pipe ของเครื่องนี้เท่านั้น และ compile ด้วย `docker/backend.build.Dockerfile` บน local ผลลัพธ์เป็น **Linux x86_64 ELF** สำหรับ server นี้ (`x86_64`) ไม่ใช่ Windows `.exe`

- `dist/line-linux/backend`: release binary
- `dist/line-linux/backend.sha256`, `release.tag`, `release.json`: checksum, รุ่น และ platform
- `dist/line-rel-<เวลา UTC>-<hash>.tar.gz`: ชุดส่งมอบพร้อม Dockerfile runtime, stack, deploy script และคู่มือ

Archive ไม่มี source Rust, Cargo, compiler Dockerfile, `.env` จริง หรือข้อมูล MongoDB เปลี่ยนจำนวนงาน compile ได้ด้วย `build-line.bat -Jobs 2` ส่วน `build.bat` เดิมใช้ทำ Windows release และไม่ได้ใช้ deploy ชุดนี้

เมื่อเปลี่ยนโค้ด ผู้พัฒนาต้อง build บน local และส่ง archive รุ่นใหม่ทุกครั้ง

## 2. ผู้พัฒนา/ผู้ดูแลระบบ — ส่ง release ไป server

รันจาก PowerShell บน local โดยแทนชื่อไฟล์ด้วย archive ที่ build สำเร็จ:

```powershell
scp "dist/line-rel-<เวลา UTC>-<hash>.tar.gz" Hermes:/home/ubuntu/
```

`Hermes` คือ SSH alias ของ server นี้ หากใช้เครื่องอื่นให้เปลี่ยนเป็น `ubuntu@<server-ip>` จากนั้นบน Ubuntu:

```sh
mkdir -p /home/ubuntu/alex-order-connector
cd /home/ubuntu/alex-order-connector
tar -xzf /home/ubuntu/line-rel-<เวลา UTC>-<hash>.tar.gz
```

Archive ไม่ทับ `.env` จริงหรือ volume MongoDB สคริปต์ deploy ตรวจ checksum และชนิด binary ก่อนเปลี่ยน Swarm หาก release ขาดหรือผิดจะหยุด ไม่มีการ compile ทดแทนบน server

## 3. ผู้ดูแลระบบ — config และ deploy บน Ubuntu

ติดตั้ง Docker หากยังไม่มี (ใช้ `sudo` ตามที่เลือกไว้):

```sh
sudo apt-get update
sudo apt-get install -y docker.io
sudo systemctl enable --now docker
```

เตรียม `/home/ubuntu/alex-order-connector/.env` และ `chmod 600 .env` ให้มี LINE ทั้ง 5 ค่า, `INQUIRY_ADMIN_TOKEN` และค่าต่อไปนี้:

```dotenv
APP_NAME=alex-line
STACK=alex-line
MONGODB_DB=alex_line
MONGO_USER=appuser
MONGO_PASSWORD=REPLACE_WITH_RANDOM_HEX
LINE_HTTP_PORT=18080
SWARM_ADVERTISE_ADDR=178.128.22.196
```

เติมรหัสผ่านสุ่มจริง ใช้ตัวอักษร/ตัวเลขเพื่อใส่ใน Mongo URI ห้ามเปลี่ยนรหัสผ่านใน `.env` อย่างเดียวหลัง volume มีข้อมูลแล้ว บัญชี MongoDB เดิมต้องถูกปรับให้ตรงด้วย `INQUIRY_ADMIN_TOKEN` คือรหัสสุ่มของระบบเรา ไม่ได้ขอจาก LINE

ครั้งแรก:

```sh
cd /home/ubuntu/alex-order-connector
sudo bash ./deploy-line.sh --init-swarm
```

อัปเดตรุ่นถัดไปหลังรับและแตก archive ใหม่:

```sh
sudo bash ./deploy-line.sh
```

Ubuntu สร้างเฉพาะ Docker runtime image จาก binary ที่ compile มาแล้ว อาจ download base image/แพ็กเกจ runtime ครั้งแรก แต่ไม่มีขั้นตอน build โปรแกรม Image tag มาจาก release ที่สร้างบน local แต่ละรุ่นมี tag ใหม่ จึงอัปเดต Swarm ได้

ชุด Step 1 ใช้ backend + MongoDB ใน `docker-stack.line.yml` ไม่ต้องมี frontend ส่วน `docker-build.sh`/`docker-deploy.sh` สำหรับชุดเต็มต้องมี frontend standalone ที่ build บน local สำหรับ Linux เพิ่มด้วย และไม่มีการ compile บน server เช่นกัน

## 4. ผู้ดูแลระบบ — Caddy และ LINE

ไฟล์ `/etc/caddy/Caddyfile`:

```caddyfile
hermes.salepie.online {
    handle /api/line/webhook {
        reverse_proxy 127.0.0.1:18080
    }

    handle {
        reverse_proxy localhost:8646
    }
}
```

```sh
sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
sudo systemctl reload caddy
curl --noproxy '*' http://127.0.0.1:18080/api/health
sudo docker stack services alex-line
```

ต้องได้ `{"status":"ok"}` และ backend/mongo เป็น `1/1` จากนั้น LINE Developers → Messaging API → Webhook URL: `https://hermes.salepie.online/api/line/webhook` กด Verify ให้ Success และเปิด Use webhook

ส่งข้อความจาก LINE ส่วนตัวเข้า OA แล้วตรวจ user ID/แชทตาม [คู่มือ Step 1](step-1-line-inquiry.md) Verify อย่างเดียวอาจมี events ว่างและไม่สร้าง user

## 5. ผู้ดูแลระบบ — ตรวจงานหลัง deploy

```sh
sudo docker service ps --no-trunc alex-line_backend
sudo docker service logs --tail 50 alex-line_backend
sudo docker service ps --no-trunc alex-line_mongo
```

Backend/MongoDB ถูกตรึงที่ node ของ Caddy และ MongoDB ใช้ volume ถาวร ไม่ publish port ของ MongoDB ส่วน port 18080 ใช้ให้ Caddy เรียก ควรจำกัดการเข้าจากภายนอกด้วย firewall ของ server อัปเดตแบบ stop-first จึงมีช่วงหยุดสั้น

หลัง deploy สำเร็จ ปิดเครื่อง local และ SSH ได้ บริการรันต่อบน server งานสำรอง/ทดสอบกู้คืน MongoDB, credentials, HTTPS และตรวจ log เป็นหน้าที่ผู้ดูแลระบบ

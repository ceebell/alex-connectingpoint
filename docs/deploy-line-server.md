# Ubuntu: pull จาก Git → สร้าง Docker จาก compiled object

โปรแกรม compile บน local เท่านั้น Git เก็บ Linux release ใน `dist/line-linux/` พร้อมโค้ดและสคริปต์ เพื่อให้ Ubuntu pull แล้วนำ binary เข้า Docker ได้ทันที `.env` เป็น config เฉพาะ server และไม่เข้า Git

```text
ผู้พัฒนา: build-backend.bat / build-frontend.bat บน local → rungit.bat เพื่อ commit/push
ผู้ดูแลระบบ: loadgit.sh → docker-build.sh → docker-deploy.sh
```

## ผู้พัฒนา: เตรียม release บน local

เปิด Docker Desktop แบบ Linux containers แล้วรันใน Windows:

```bat
build-backend.bat
rungit.bat "Update Linux release"
```

ต้อง commit/push ทั้ง binary, checksum, tag, manifest และโค้ดรุ่นเดียวกัน Git เก็บเฉพาะ 4 ไฟล์ใน `dist/line-linux/`: `backend`, `backend.sha256`, `release.tag`, `release.json` ส่วน Windows output, archive, cache, `billing_wth_text/` และ `.env` ยังถูก ignore

หน้าหลังบ้าน Step 2 ใช้ `build-frontend.bat` ได้ Linux standalone ใน `dist/frontend-linux/` ที่เข้า Gitพร้อม dependencies และ tag แยก ดู [คู่มือหน้าหลังบ้าน/config/deploy Step 2](step-2-admin.md) และ [คู่มือ build local](build-local.md)

## ผู้ดูแลระบบ: บน Ubuntu

ครั้งแรก clone ด้วยบัญชี ubuntu หากยังไม่มีโฟลเดอร์:

```sh
cd ~
git clone https://github.com/ceebell/alex-connectingpoint.git
```

**1. Pull โค้ด สคริปต์ และ compiled object จาก Git**

```sh
cd ~/alex-connectingpoint
bash ./loadgit.sh main
```

สคริปต์เก็บการแก้ไฟล์ที่ track บน server ไว้ใน Git stash ก่อน pull และคง stash ไว้เป็นสำรอง `.env` ไม่ถูกเปลี่ยน

**2. สร้าง Docker image จาก compiled object**

```sh
sudo bash ./docker-build.sh
```

ตรวจ SHA256 และ ELF Linux/amd64 ก่อน COPY เข้า image สร้างเฉพาะ backend โดยไม่ต้องมี frontend ไม่มี cargo/npm build บน Ubuntu และยังไม่เปลี่ยนบริการที่รันอยู่ หาก release ขาดให้ผู้พัฒนา build/push จาก local แล้ว pull ใหม่

**3. เปิดหรืออัปเดตบริการด้วย image ที่สร้างแล้ว**

```sh
sudo bash ./docker-deploy.sh
```

ครั้งแรกที่ยังไม่ได้เปิด Swarm ใช้ `sudo bash ./docker-deploy.sh --init-swarm` สคริปต์ตรวจว่า image ตรงกับ release แล้ว deploy backend + MongoDB โดยไม่สร้าง image ซ้ำ จากนั้นรอ health

`deploy-line.sh` ยังใช้เป็นทางลัดสร้าง image + deploy ได้ ส่วนลำดับมาตรฐานแยก build และ deploy ตามข้างบน ชุดเต็มที่มี Linux frontend ใช้ `docker-build.sh --full-stack` และ `docker-deploy.sh --full-stack`

## ผู้ดูแลระบบ: .env และ Caddy

เก็บ `.env` ที่ `/home/ubuntu/alex-connectingpoint/.env` สิทธิ์ `600` หากมี config เดิมอยู่แล้ว คัดลอกเฉพาะครั้งแรก:

```sh
cp -n /home/ubuntu/alex-order-connector/.env /home/ubuntu/alex-connectingpoint/.env
chmod 600 /home/ubuntu/alex-connectingpoint/.env
nano /home/ubuntu/alex-connectingpoint/.env
```

กรอก LINE ทั้ง 5 ค่าและ `INQUIRY_ADMIN_TOKEN` พร้อมค่าฝั่ง server:

```dotenv
APP_NAME=alex-line
STACK=alex-line
MONGODB_DB=alex_line
MONGO_USER=appuser
MONGO_PASSWORD=REPLACE_WITH_RANDOM_HEX
LINE_HTTP_PORT=18080
SWARM_ADVERTISE_ADDR=178.128.22.196
```

ใช้รหัสผ่านสุ่มจริงเป็นตัวอักษร/ตัวเลข การเปลี่ยนรหัสผ่านใน `.env` หลัง volume มีข้อมูล ต้องปรับบัญชี MongoDB ให้ตรงกันด้วย `INQUIRY_ADMIN_TOKEN` คือรหัสสุ่มที่ระบบเรากำหนดสำหรับ API ภายใน

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
```

## ตรวจรับบริการ

```sh
curl --noproxy '*' http://127.0.0.1:18080/api/health
sudo docker stack services alex-line
```

ต้องได้ `{"status":"ok"}` และ backend/mongo เป็น `1/1` แล้วตั้ง LINE Webhook URL เป็น `https://hermes.salepie.online/api/line/webhook` กด Verify ให้ Success และเปิด Use webhook ส่งข้อความจริงเข้า OA และตรวจ user ID/แชทตาม [คู่มือ Step 1](step-1-line-inquiry.md)

หากยังไม่พร้อม:

```sh
sudo docker node ls
sudo docker service ps --no-trunc alex-line_backend
sudo docker service logs --tail 40 alex-line_backend
sudo docker service ps --no-trunc alex-line_mongo
```

หาก Pending/no suitable node ให้ตรวจ node เป็น ready/active และใช้ stack ที่แก้ interpolation แล้ว สคริปต์ตรวจ node/image หลังแทนตัวแปรก่อน deploy โดยไม่แสดง credentials กรณีค่าเป็น `line.sh` ให้ใช้ไฟล์แก้ไขและทำขั้นตอน build/deploy ใหม่

หาก backend เคยเริ่มด้วย image ที่ถูกต้องแล้วล้มว่า `Database unavailable` ก่อนกลับไปเป็น `line.sh` ให้แยกข้อผิดพลาดฐานข้อมูลออกจากผล rollback ตาม update policy การแก้ YAML ด้วย vi ยังไม่เปลี่ยน service ที่ deploy ไปแล้ว

ตรวจสาเหตุด้วยคำสั่งแบบไม่เปลี่ยนบริการ:

```sh
sudo bash ./diagnose-line.sh
```

สคริปต์รายงาน image/update state/constraint, ค่าเชื่อมต่อว่าตรง `.env` หรือไม่, MongoDB health และทดสอบ authentication ผ่าน localhost, `mongo` และ `tasks.mongo` โดยไม่แสดง password/token/URI เต็ม

- `auth_local` ไม่ผ่าน: ตรวจบัญชีใน MongoDB และ credentials ของ volume เดิม
- localhost ผ่านแต่ชื่อ service ไม่ผ่าน: ตรวจ DNS/เครือข่าย Swarm และเส้นทาง VIP
- ทุก probe ผ่าน: MongoDB พร้อมในเวลาตรวจแล้ว แต่ยังต้องใช้ image/constraint ที่ถูกต้องแทน spec เก่าที่ rollback กลับมา

Backend รุ่นปัจจุบันลอง ping ฐานข้อมูลโดยมี server selection timeout 5 วินาทีก่อนเปิด HTTP จึงไม่มีพอร์ต 18080 หากขั้นตอนนั้นล้มเหลว ค่า service MongoDB เป็น `1/1` เพียงอย่างเดียวไม่ได้ยืนยันว่า backend authentication หรือเส้นทางเครือข่ายผ่าน

MongoDB ใช้ volume ถาวรและไม่มี published port ส่วนพอร์ต 18080 ให้ Caddy เรียก อัปเดตแบบ stop-first มีช่วงหยุดสั้น หลัง deploy ปิด local/SSH ได้ ผู้ดูแลระบบรับผิดชอบ backup/ทดสอบกู้คืน MongoDB และดูแล HTTPS/config

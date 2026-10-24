# 01 — เพิ่ม API ฝั่ง Backend (Rust axum + MongoDB)

> แก้ค่าใน `{{ }}` ก่อนส่ง

---

อ่าน `AGENTS.md` (ข้อ 5 และ 7) และ entry ล่าสุดใน `revising-log.md` ก่อน แล้วเพิ่มฟีเจอร์ backend ต่อไปนี้

**ฟีเจอร์:** `{{เช่น จัดการคำสั่งซื้อ (orders)}}`

**Collection:** `{{orders}}`

**Fields:**

| Field | Type | Required | หมายเหตุ |
|---|---|---|---|
| `{{customer_name}}` | `String` | ใช่ | `{{ไม่ว่าง ยาวไม่เกิน 100}}` |
| `{{total}}` | `f64` | ใช่ | `{{มากกว่า 0}}` |
| `{{status}}` | `String` | ใช่ | `{{หนึ่งใน pending, paid, cancelled}}` |

**Endpoints ที่ต้องการ:**

- `{{GET /api/orders — list (รองรับ ?limit=&skip=)}}`
- `{{GET /api/orders/{id}}}`
- `{{POST /api/orders}}`
- `{{PATCH /api/orders/{id}}}`
- `{{DELETE /api/orders/{id}}}`

**Index ที่ต้องมี:** `{{status + created_at (desc)}}`

**ข้อกำหนด**

- แยกไฟล์ตาม layer: `routes/{{orders}}.rs` · `models/{{order}}.rs` · `db/` — ลงทะเบียน route ใน `routes/mod.rs`
- validate input และคืน error ผ่าน `AppError` (400 สำหรับ input ผิด, 404 เมื่อไม่พบ, ObjectId ไม่ถูกต้อง = 400)
- ห้าม `unwrap()` / `expect()` ใน handler
- ทุก document มี `created_at`, `updated_at` (BSON DateTime) · `PATCH` ต้องอัปเดต `updated_at`
- เพิ่ม index ใน `db/indexes.rs` แบบ idempotent
- เพิ่ม test ครอบคลุม validation และ happy path อย่างน้อย 1 กรณีต่อ endpoint
- ไม่เพิ่ม dependency ใหม่ ถ้าจำเป็นต้องเพิ่มให้บอกเหตุผลก่อน

**ก่อนจบ:** `cargo fmt --check` · `cargo clippy -- -D warnings` · `cargo test` — รายงานผลจริง
จากนั้นเพิ่ม entry ใน `revising-log.md` (วัน-เวลาจากเครื่องจริง, โฟลเดอร์, ไฟล์, สิ่งที่แก้) และไม่ commit/push เอง

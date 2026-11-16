# Driver Notes API v1 — contract tham chiếu

Đối chiếu với `backend/src/api/` trước khi sử dụng. Tài liệu theo cụm chức năng ở `docs/backend/`, danh sách route ở `backend/src/api/mod.rs`.

## Quy ước chung

- Base path: `/api/v1`.
- JSON dùng `camelCase`; thời gian là Unix milliseconds; tiền là số nguyên VND.
- Route dữ liệu cần bearer token. Token trả về khi đăng ký/đăng nhập và hết hạn sau 30 ngày. Lỗi backend có dạng `{ "error": "thông báo" }`.
- `GET /health` trả 200; Swagger `/swagger-ui/`; OpenAPI `/api-docs/openapi.json`.
- `POST /api/v1/restore` giới hạn body 10 MiB.

## Route

| Phương thức | Đường dẫn | Đầu vào | Kết quả thành công |
| --- | --- | --- | --- |
| POST | `/auth/register` | `{email, password}` | 201 `AuthResponse` |
| POST | `/auth/login` | `{email, password}` | 200 `AuthResponse` |
| POST | `/auth/logout` | Bearer token | 204 |
| GET | `/auth/me` | Bearer token | 200 `Profile` |
| GET | `/shifts` | Bearer token | 200 `Shift[]` |
| POST | `/shifts` | Bearer token, không body | 201 `Shift` |
| POST | `/shifts/{id}/end` | Bearer token | 200 `Shift` |
| PUT | `/shifts/{id}` | `ShiftInput` | 200 `Shift` |
| DELETE | `/shifts/{id}` | Bearer token | 204 |
| GET | `/entries` | Bearer token | 200 `Entry[]` |
| POST | `/entries` | `EntryInput` | 201 `Entry` |
| PUT | `/entries/{id}` | `EntryInput` | 200 `Entry` |
| DELETE | `/entries/{id}` | Bearer token | 204 |
| GET | `/snapshot` | Bearer token | 200 `Snapshot` |
| POST | `/restore` | `Backup` | 200 `RestoreResult` |

## Kiểu dữ liệu

- `AuthResponse`: `{ userId: string, email: string, token: string, expiresAt: number }`. Đăng ký cần mật khẩu 12–128 ký tự; email được trim và chuyển chữ thường. `Profile`: `{ id: string, email: string }`.
- `Shift`: `{ id, startedAt, endedAt, createdAt, updatedAt }`; `endedAt` là `number | null`, các timestamp khác là `number`. Chỉ có một ca đang chạy cho mỗi tài khoản. Bắt đầu/kết thúc ca lấy giờ server. `ShiftInput` cho PUT gồm `{ startedAt: number, endedAt: number }`; chỉ sửa ca đã kết thúc, `endedAt > startedAt`.
- `Entry`: `{ id, shiftId, kind, deliveryCount, expenseCategory, amountVnd, tipVnd, occurredAt, note, createdAt, updatedAt }`. `shiftId`, `deliveryCount`, `expenseCategory` có thể là `null`.
- `EntryInput`: `shiftId?: string | null`, `kind: "delivery" | "bonus" | "expense"`, `deliveryCount?: 1 | 2 | 3 | null`, `expenseCategory?: "fuel" | "food" | "other" | null`, `amountVnd: number`, `tipVnd?: number` (mặc định 0), `occurredAt: number`, `note?: string` (mặc định rỗng). `amountVnd` phải lớn hơn 0 và không quá 1.000.000.000.000; `tipVnd` từ 0 đến giới hạn đó; `note` tối đa 500 ký tự Unicode.
- Với `delivery`, cần `deliveryCount` 1–3 và không có `expenseCategory`. Với `bonus`, không có `deliveryCount` hay `expenseCategory`, `tipVnd = 0`. Với `expense`, cần `expenseCategory`, không có `deliveryCount`, `tipVnd = 0`.
- `Snapshot`: `{ schemaVersion: 1, fetchedAt, shifts: Shift[], entries: Entry[] }`. Chỉ cập nhật cache offline sau GET thành công. `Backup` dùng `{ schemaVersion: 1, shifts, entries }` từ snapshot; `fetchedAt` không thuộc `Backup`. `RestoreResult`: `{ shiftsRestored, entriesRestored }`.

## Nguồn mã theo nhóm

| Nhóm | Nguồn |
| --- | --- |
| Router, CORS, Swagger | `backend/src/api/mod.rs`, `backend/src/api/docs.rs` |
| Tài khoản, phiên | `backend/src/api/auth.rs` |
| Ca làm | `backend/src/api/shifts.rs` |
| Giao dịch | `backend/src/api/entries.rs` |
| Snapshot, restore | `backend/src/api/backup.rs` |
| Lỗi, thời gian, SQLite | `backend/src/error.rs`, `backend/src/time.rs`, `backend/src/db.rs`, `backend/migrations/` |

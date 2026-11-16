# Driver Notes API

Backend Rust cho nhiều tài xế. Mỗi yêu cầu dữ liệu dùng bearer token; mọi truy vấn ca và giao dịch đều giới hạn theo tài khoản. SQLite trên server là nguồn dữ liệu chính. Frontend PWA có thể lưu phản hồi `GET /api/v1/snapshot` để **chỉ xem** khi mất mạng.

## Cấu trúc

```text
src/
  main.rs          Điểm khởi động
  lib.rs           Khởi tạo server và cấu hình
  api/
    mod.rs         Router và CORS
    docs.rs        OpenAPI và Swagger UI
    auth.rs        Tài khoản, phiên đăng nhập
    shifts.rs      Ca làm
    entries.rs     Thu nhập, thưởng, chi phí
    backup.rs      Snapshot và khôi phục JSON
  db.rs            Kết nối và thao tác SQLite
  error.rs         Lỗi HTTP thống nhất
  time.rs          Quy tắc timestamp
  tests.rs         Kiểm tra hành vi API
migrations/
  001_init.sql     Schema và chỉ mục ban đầu
```

## Chạy local

Yêu cầu Rust stable và bộ công cụ C++ trên Windows. Mở **x64 Developer Command Prompt for Visual Studio** để compiler và linker có trong `PATH`.

```powershell
cargo run
```

Mặc định API nghe tại `127.0.0.1:3000` và tạo `data/driver-notes.db`. Cấu hình bằng biến môi trường:

| Biến | Mục đích |
| --- | --- |
| `BIND_ADDR` | Địa chỉ nghe, mặc định `127.0.0.1:3000` |
| `DATABASE_PATH` | Đường dẫn SQLite có ổ đĩa bền vững |
| `FRONTEND_ORIGIN` | Origin frontend duy nhất được phép gọi API qua CORS, ví dụ `https://drivernotes.site` |

Khi triển khai công khai, đặt API sau HTTPS, giới hạn tốc độ đăng nhập/đăng ký ở proxy và sao lưu file SQLite định kỳ. Cloudflare Pages chỉ host frontend; backend Rust này cần nơi chạy process có ổ đĩa bền vững. Không đặt database trên hệ thống file tạm.

## Thử API bằng Swagger

1. Chạy `cargo run`, rồi mở <http://127.0.0.1:3000/swagger-ui/>. File OpenAPI ở <http://127.0.0.1:3000/api-docs/openapi.json>.
2. Gọi `POST /api/v1/auth/register` hoặc `POST /api/v1/auth/login`. Sao chép giá trị `token` trong phản hồi.
3. Bấm **Authorize** ở đầu trang và dán **chỉ giá trị token**; Swagger tự thêm `Bearer` vào header.
4. Chọn endpoint và bấm **Try it out**. `occurredAt`, `startedAt`, `endedAt` đều là Unix milliseconds; tiền dùng số nguyên VND.

`POST /api/v1/restore` thay toàn bộ dữ liệu hiện có của tài khoản bằng JSON snapshot, nên chỉ thử với tài khoản test.

## API v1

JSON dùng tên trường `camelCase`, thời gian là Unix milliseconds, tiền là số nguyên VND. Các đường dẫn ngoài đăng ký và đăng nhập cần `Authorization: Bearer <token>`.

| Phương thức | Đường dẫn | Mục đích |
| --- | --- | --- |
| `POST` | `/api/v1/auth/register` | Tạo tài khoản với `email`, `password` (12–128 ký tự); trả token |
| `POST` | `/api/v1/auth/login` | Đăng nhập; trả token |
| `POST` | `/api/v1/auth/logout` | Thu hồi token hiện tại |
| `GET` | `/api/v1/auth/me` | Tài khoản hiện tại |
| `GET` | `/api/v1/snapshot` | Toàn bộ ca và giao dịch của tài khoản; dùng để xuất JSON hoặc làm bản xem offline |
| `POST` | `/api/v1/restore` | Thay toàn bộ dữ liệu của tài khoản bằng snapshot JSON phiên bản 1, trong một transaction |
| `GET`, `POST` | `/api/v1/shifts` | Danh sách ca; bắt đầu ca tại thời điểm server |
| `POST` | `/api/v1/shifts/{id}/end` | Kết thúc ca tại thời điểm server |
| `PUT`, `DELETE` | `/api/v1/shifts/{id}` | Sửa thời gian ca đã kết thúc hoặc xóa ca |
| `GET`, `POST` | `/api/v1/entries` | Danh sách hoặc tạo giao dịch |
| `PUT`, `DELETE` | `/api/v1/entries/{id}` | Thay toàn bộ hoặc xóa giao dịch |

Đầu vào `POST/PUT /entries` gồm `kind` (`delivery`, `bonus`, `expense`), `amountVnd`, `occurredAt`, tùy chọn `shiftId`, `note`. Với `delivery` cần `deliveryCount` (1, 2, 3) và có thể có `tipVnd`. Với `expense` cần `expenseCategory` (`fuel`, `food`, `other`). `PUT /shifts/{id}` cần `startedAt`, `endedAt`.

Token có hạn 30 ngày; client cần giữ an toàn và xóa khi đăng xuất. Snapshot chỉ được cập nhật sau khi tải thành công từ server. Backend không xếp hàng lệnh ghi khi offline.

Tài liệu theo từng cụm chức năng nằm trong [mục lục docs](../docs/README.md).

# Khởi động server và quy tắc dùng chung

## File nguồn

| File | Trách nhiệm |
| --- | --- |
| [`backend/src/main.rs`](../../../backend/src/main.rs) | Gọi `driver_notes_api::run()` trong Tokio runtime. |
| [`backend/src/lib.rs`](../../../backend/src/lib.rs) | Đọc cấu hình, mở database, áp dụng schema và chạy Axum. |
| [`backend/src/db.rs`](../../../backend/src/db.rs) | Chia sẻ kết nối SQLite qua `AppState`, thực hiện truy vấn trong `spawn_blocking`. |
| [`backend/src/error.rs`](../../../backend/src/error.rs) | Chuyển lỗi ứng dụng thành HTTP status và JSON. |
| [`backend/src/time.rs`](../../../backend/src/time.rs) | Lấy Unix milliseconds và kiểm tra khoảng thời gian hợp lệ. |

## Luồng khởi động

`main` → `run` → tạo thư mục chứa database nếu cần → mở SQLite → bật WAL và `busy_timeout = 5000` → thực thi `migrations/001_init.sql` → bind TCP → gắn router và phục vụ HTTP. Schema được nhúng khi biên dịch bằng `include_str!`.

| Biến môi trường | Mặc định | Cách dùng |
| --- | --- | --- |
| `DATABASE_PATH` | `data/driver-notes.db` | Đường dẫn tương đối tính từ thư mục chạy process, hoặc đường dẫn tuyệt đối. |
| `BIND_ADDR` | `127.0.0.1:3000` | Địa chỉ TCP của API. |
| `FRONTEND_ORIGIN` | Không đặt | Nếu có giá trị, router bật CORS cho đúng origin đó. |

Ứng dụng có một `Connection` được bọc trong `Arc<Mutex<_>>`. `with_db` chuyển truy vấn sang blocking pool để tránh chặn luồng async; mutex vẫn khiến các thao tác DB trong cùng process chạy lần lượt. Khi cần tải nhiều tài xế hơn, đo tải trước khi đổi cách quản lý kết nối.

## Lỗi và thời gian

`ApiError` trả JSON dạng `{"error":"..."}`. Các trạng thái đang dùng là 400 (dữ liệu sai), 401 (thiếu/sai phiên), 404 (không tìm thấy bản ghi), 409 (xung đột) và 500 (lỗi nội bộ). Lỗi nội bộ không trả chi tiết SQLite cho client.

Thời gian API dùng số nguyên Unix milliseconds. `valid_time` chỉ chấp nhận khoảng từ **01/01/2000** đến **01/01/2100** theo timestamp UTC; ngày/tuần/tháng hiển thị chưa được tính trong backend. Xem thêm [quy tắc ca](../shifts/shift-management.md) và [giao dịch](../entries/transactions.md).

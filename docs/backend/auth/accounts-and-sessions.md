# Tài khoản và phiên đăng nhập

**File nguồn:** [`backend/src/api/auth.rs`](../../../backend/src/api/auth.rs)

**Bảng dữ liệu:** `users`, `sessions` trong [SQLite schema](../storage/sqlite.md).

## Endpoint

| Phương thức | Đường dẫn | Kết quả chính |
| --- | --- | --- |
| `POST` | `/api/v1/auth/register` | Tạo tài khoản và trả token; 201. |
| `POST` | `/api/v1/auth/login` | Xác thực mật khẩu và trả token; 200. |
| `POST` | `/api/v1/auth/logout` | Xóa token đang gửi; 204. |
| `GET` | `/api/v1/auth/me` | Trả `id`, `email` của tài khoản; 200. |

Đăng ký/đăng nhập nhận `email`, `password`. Email được `trim` và chuyển sang chữ thường; kiểm tra hiện tại chỉ gồm chiều dài 3–254 ký tự, có `@` và không có khoảng trắng. Đăng ký yêu cầu mật khẩu dài 12–128 ký tự. Email trùng trả 409; đăng nhập sai trả 401.

## Cách tạo và kiểm tra phiên

1. Mật khẩu được băm bằng Argon2 với salt ngẫu nhiên. Việc băm và xác minh chạy trong `spawn_blocking`.
2. API tạo token ngẫu nhiên từ hai UUID v4 dạng hex; chỉ **SHA-256 của token** được lưu trong bảng `sessions`.
3. Response đăng ký/đăng nhập gồm `userId`, `email`, `token`, `expiresAt`. Hạn phiên là 30 ngày tính từ lúc tạo, biểu diễn bằng Unix milliseconds.
4. Handler cần xác thực đọc header `Authorization: Bearer <token>`, băm token, tìm phiên chưa hết hạn và lấy `user_id`.
5. Đăng xuất xóa hash của token hiện tại. Các endpoint dữ liệu sau đó phải dùng phiên hợp lệ khác hoặc đăng nhập lại.

## Quy tắc tách dữ liệu

Mọi truy vấn ca/giao dịch phải lọc theo `user_id` đã xác thực; không nhận `user_id` từ body hoặc path. Giao dịch gắn `shiftId` còn phải kiểm tra ca đó thuộc cùng người dùng. Xem [ca làm](../shifts/shift-management.md) và [giao dịch](../entries/transactions.md).

Hiện chưa có xác minh email, đặt lại mật khẩu hoặc giới hạn tốc độ ở tầng API. Khi đưa ra Internet, README backend yêu cầu giới hạn tốc độ đăng ký/đăng nhập ở proxy và dùng HTTPS.

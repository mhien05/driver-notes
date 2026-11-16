# SQLite và schema

**File nguồn:** [`backend/migrations/001_init.sql`](../../../backend/migrations/001_init.sql), [`backend/src/db.rs`](../../../backend/src/db.rs), [`backend/src/lib.rs`](../../../backend/src/lib.rs).

Backend mở SQLite từ `DATABASE_PATH` (mặc định `data/driver-notes.db` so với thư mục chạy), bật `foreign_keys`, WAL và `busy_timeout = 5000`. Mọi thao tác DB đi qua `with_db`, chạy trong Tokio blocking pool trên một kết nối được bảo vệ bằng mutex.

## Bảng và quan hệ

| Bảng | Cột chính | Ràng buộc |
| --- | --- | --- |
| `users` | `id`, `email`, `password_hash`, `created_at` | `email` duy nhất. |
| `sessions` | `token_hash`, `user_id`, `expires_at` | Thuộc một user; xóa user thì xóa phiên. |
| `shifts` | `id`, `user_id`, `started_at`, `ended_at`, `created_at`, `updated_at` | `ended_at` rỗng hoặc lớn hơn `started_at`. |
| `entries` | `id`, `user_id`, `shift_id`, `kind`, `delivery_count`, `expense_category`, `amount_vnd`, `tip_vnd`, `occurred_at`, `note`, timestamps | Có `CHECK` cho từng loại giao dịch và số tiền không âm theo vai trò. |

`shifts.user_id` và `entries.user_id` tham chiếu `users`. `entries.shift_id` tham chiếu `shifts`; xóa ca sẽ đặt `shift_id = NULL`, không xóa giao dịch. Handler vẫn phải lọc theo `user_id` và kiểm tra ca thuộc tài khoản khi gán `shiftId`; khóa ngoại không tự bảo đảm hai hàng thuộc cùng user.

## Chỉ mục

- `sessions_by_user` hỗ trợ tra cứu phiên theo user.
- `one_active_shift_per_user` là unique index trên `shifts(user_id)` chỉ khi `ended_at IS NULL`.
- `shifts_by_user_start` và `entries_by_user_time` hỗ trợ danh sách theo tài khoản/thời gian.

## Thay đổi schema và vận hành

`001_init.sql` dùng `CREATE TABLE IF NOT EXISTS` và được chạy khi server khởi động. Hiện chưa có bảng phiên bản hay bộ chạy migration cho thay đổi schema sau này. Khi thêm/sửa cột hoặc ràng buộc, cần thiết kế đường nâng cấp database đang có trước khi phát hành.

Database phải nằm trên ổ đĩa bền vững và được sao lưu định kỳ. Các file database local và `target/` nằm trong `.gitignore`. JSON snapshot theo người dùng được mô tả riêng tại [trang backup](../backup/snapshot-and-restore.md).

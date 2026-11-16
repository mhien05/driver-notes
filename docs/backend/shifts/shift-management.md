# Quản lý ca làm

**File nguồn:** [`backend/src/api/shifts.rs`](../../../backend/src/api/shifts.rs)

**Bảng dữ liệu:** `shifts` trong [SQLite schema](../storage/sqlite.md).

## Mô hình

`Shift` trả `id`, `startedAt`, `endedAt`, `createdAt`, `updatedAt`. `endedAt = null` nghĩa là ca đang chạy. Mỗi ca có `user_id`; chỉ chủ sở hữu được xem, sửa hoặc xóa. Timestamp dùng Unix milliseconds.

## Endpoint và quy tắc

| Phương thức | Đường dẫn | Hành vi |
| --- | --- | --- |
| `GET` | `/api/v1/shifts` | Danh sách ca của tài xế, mới bắt đầu trước. |
| `POST` | `/api/v1/shifts` | Bắt đầu ca tại thời điểm server; trả 201. |
| `POST` | `/api/v1/shifts/{id}/end` | Kết thúc ca đang chạy tại thời điểm server. |
| `PUT` | `/api/v1/shifts/{id}` | Thay toàn bộ `startedAt`, `endedAt` của **ca đã kết thúc**. |
| `DELETE` | `/api/v1/shifts/{id}` | Xóa ca của tài xế. |

Chỉ có một ca đang chạy cho mỗi tài xế nhờ unique index có điều kiện `ended_at IS NULL`. Bắt đầu ca thứ hai trả 409. Khi kết thúc, backend bảo đảm `endedAt` ít nhất lớn hơn `startedAt` **1 ms**, kể cả hai thao tác diễn ra trong cùng millisecond.

Sửa ca yêu cầu hai timestamp hợp lệ và `endedAt > startedAt`. Hiện chưa kiểm tra ca đã kết thúc có chồng lấn nhau hay giao dịch gắn ca có nằm trong khoảng thời gian ca. Khi xóa ca, khóa ngoại của giao dịch liên quan được đặt thành `NULL`; giao dịch vẫn tồn tại.

Thời lượng ca có thể tính từ `endedAt - startedAt`; ca đang chạy dùng thời điểm hiện tại khi hiển thị. Backend không cần timer nền để giữ trạng thái ca. Bài test hiện tại kiểm tra một ca đang chạy và quyền sở hữu giữa hai tài xế.

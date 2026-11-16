# Giao dịch thu nhập, thưởng và chi phí

**File nguồn:** [`backend/src/api/entries.rs`](../../../backend/src/api/entries.rs)

**Bảng dữ liệu:** `entries` trong [SQLite schema](../storage/sqlite.md).

## Endpoint

| Phương thức | Đường dẫn | Hành vi |
| --- | --- | --- |
| `GET` | `/api/v1/entries` | Danh sách giao dịch của tài xế, theo `occurredAt` giảm dần. |
| `POST` | `/api/v1/entries` | Tạo giao dịch; trả 201 và bản ghi đã lưu. |
| `PUT` | `/api/v1/entries/{id}` | Thay toàn bộ nội dung bản ghi thuộc tài xế. |
| `DELETE` | `/api/v1/entries/{id}` | Xóa bản ghi thuộc tài xế; trả 204. |

`Entry` trả thêm `id`, `createdAt`, `updatedAt`. Request dùng `camelCase`; thời gian là Unix milliseconds, tiền là **số nguyên VND**.

## Các loại bản ghi

| `kind` | Trường bắt buộc thêm | Quy tắc |
| --- | --- | --- |
| `delivery` | `deliveryCount` = 1, 2 hoặc 3 | Có thể có `tipVnd`; `expenseCategory` phải vắng mặt. |
| `bonus` | Không | `deliveryCount`, `expenseCategory` vắng mặt; `tipVnd = 0`. |
| `expense` | `expenseCategory` = `fuel`, `food` hoặc `other` | `deliveryCount` vắng mặt; `tipVnd = 0`. |

Mỗi bản ghi `delivery` là **một lượt giao**. Tổng số đơn thực tế là tổng `deliveryCount` của các bản ghi đó. Tiền thưởng là bản ghi riêng, không trộn với tiền giao hàng. Tổng thu nhập có thể tính từ tiền giao hàng + tip + thưởng; chi phí lấy từ bản ghi `expense`.

## Kiểm tra đầu vào và quyền sở hữu

- `amountVnd` phải lớn hơn 0 và không quá 1.000.000.000.000; `tipVnd` từ 0 đến cùng giới hạn, mặc định 0.
- `occurredAt` phải nằm trong khoảng thời gian hợp lệ do `time.rs` định nghĩa. `note` tối đa 500 ký tự Unicode, mặc định chuỗi rỗng.
- `shiftId` có thể bỏ trống. Nếu có, ca phải thuộc chính tài xế đang đăng nhập; backend chưa yêu cầu thời điểm giao dịch nằm trong ca.
- `PUT` dùng toàn bộ `EntryInput`, không phải cập nhật từng trường. Mọi thao tác đọc/sửa/xóa giới hạn bằng `user_id` từ phiên.

Dashboard ngày/tuần/tháng chưa có endpoint tổng hợp riêng; dữ liệu nguồn hiện được trả qua danh sách hoặc [snapshot](../backup/snapshot-and-restore.md).

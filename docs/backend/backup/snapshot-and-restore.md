# Snapshot và khôi phục dữ liệu

**File nguồn:** [`backend/src/api/backup.rs`](../../../backend/src/api/backup.rs)

**Dữ liệu liên quan:** [ca làm](../shifts/shift-management.md), [giao dịch](../entries/transactions.md).

## Xuất snapshot

`GET /api/v1/snapshot` yêu cầu bearer token và trả toàn bộ ca/giao dịch của **một tài khoản**:

```json
{
  "schemaVersion": 1,
  "fetchedAt": 1790000000000,
  "shifts": [],
  "entries": []
}
```

`fetchedAt` là thời điểm server tạo response, không phải thời điểm giao dịch mới nhất. Frontend có thể giữ snapshot tải thành công gần nhất để **chỉ xem** khi mất mạng. Các lệnh ghi vẫn phải gửi lên server và được xác nhận; backend không có hàng đợi đồng bộ offline. Endpoint hiện trả toàn bộ dữ liệu, chưa phân trang.

## Nhập và thay dữ liệu

`POST /api/v1/restore` nhận JSON có `schemaVersion`, `shifts`, `entries`; trường `fetchedAt` từ snapshot xuất ra được bỏ qua khi đọc. Request body tối đa **10 MiB**. Backend kiểm tra:

1. `schemaVersion = 1`; mỗi danh sách tối đa 50.000 bản ghi.
2. ID trong mỗi danh sách là UUID hợp lệ và không trùng; thời gian ca hợp lệ, ca kết thúc phải sau lúc bắt đầu; tối đa một ca đang chạy.
3. Mọi giao dịch vượt qua quy tắc của `EntryInput`; `shiftId` nếu có phải trỏ đến ca trong cùng file nhập.

Sau khi hợp lệ, một SQLite transaction xóa giao dịch và ca cũ **chỉ của tài khoản hiện tại**, rồi chèn bản ghi mới. ID ca/giao dịch được tạo lại, các tham chiếu `shiftId` được ánh xạ sang ID mới; `createdAt` và `updatedAt` được đặt theo thời điểm khôi phục. Nếu transaction lỗi, dữ liệu cũ được rollback. Response trả `shiftsRestored` và `entriesRestored`.

Thao tác này **thay toàn bộ dữ liệu** của tài khoản, nên dùng tài khoản test khi thử trên Swagger. Snapshot JSON của người dùng và bản sao lưu file SQLite của server là hai lớp bảo vệ khác nhau; SQLite trên server vẫn cần được sao lưu định kỳ.

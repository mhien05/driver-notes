# Tài liệu Driver Notes

Tài liệu backend được chia theo cụm chức năng. Mỗi trang ghi file nguồn, luồng xử lý, quy tắc dữ liệu và điểm cần lưu ý khi sửa.

```text
docs/
  README.md
  backend/
    runtime/server.md
    api/routes-and-swagger.md
    auth/accounts-and-sessions.md
    shifts/shift-management.md
    entries/transactions.md
    backup/snapshot-and-restore.md
    storage/sqlite.md
    testing/api-tests.md
```

| File nguồn | Cụm tài liệu |
| --- | --- |
| `backend/src/main.rs` | [Runtime](backend/runtime/server.md) |
| `backend/src/lib.rs` | [Runtime](backend/runtime/server.md) |
| `backend/src/db.rs` | [Runtime](backend/runtime/server.md), [SQLite](backend/storage/sqlite.md) |
| `backend/src/error.rs` | [Runtime](backend/runtime/server.md) |
| `backend/src/time.rs` | [Runtime](backend/runtime/server.md) |
| `backend/src/api/mod.rs` | [Router và Swagger](backend/api/routes-and-swagger.md) |
| `backend/src/api/docs.rs` | [Router và Swagger](backend/api/routes-and-swagger.md) |
| `backend/src/api/auth.rs` | [Xác thực và phiên](backend/auth/accounts-and-sessions.md) |
| `backend/src/api/shifts.rs` | [Quản lý ca](backend/shifts/shift-management.md) |
| `backend/src/api/entries.rs` | [Giao dịch](backend/entries/transactions.md) |
| `backend/src/api/backup.rs` | [Snapshot và khôi phục](backend/backup/snapshot-and-restore.md) |
| `backend/src/tests.rs` | [API tests](backend/testing/api-tests.md) |
| `backend/migrations/001_init.sql` | [SQLite](backend/storage/sqlite.md) |

Hướng dẫn chạy dự án ở [README gốc](../README.md); Swagger UI và danh sách endpoint cũng được mô tả trong [README backend](../backend/README.md).

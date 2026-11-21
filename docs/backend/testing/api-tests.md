# Kiểm tra backend

**File nguồn:** [`backend/src/tests.rs`](../../../backend/src/tests.rs).

Chạy từ `backend/`:

```powershell
cargo fmt -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Trên Windows cần Rust và công cụ liên kết C++ tương thích MSVC; nếu terminal chưa nhận linker, dùng x64 Developer Command Prompt của Visual Studio.

## Test hiện có

| Test | Kiểm tra |
| --- | --- |
| `swagger_documents_live_routes_and_authentication` | OpenAPI JSON có các route, schema request giao dịch và bearer security; đăng ký/đăng nhập công khai; Swagger UI trả 200. |
| `deleting_own_shift_keeps_entries_and_rejects_other_users` | Chỉ chủ sở hữu được xóa ca; giao dịch liên quan vẫn còn và `shiftId` trở thành `null`. |
| `users_are_isolated_and_shifts_survive_reloads` | Đăng ký/đăng nhập/đăng xuất, quyền truy cập giữa hai user, một ca đang chạy, ghi giao dịch, snapshot, từ chối backup sai và khôi phục dữ liệu hợp lệ. |

Test tạo SQLite trong bộ nhớ và gọi Axum router trực tiếp bằng HTTP request, nên không cần server đang chạy hoặc database thật. Chúng xác nhận các luồng quan trọng nêu trên; khi thay đổi quy tắc kinh doanh, thêm trường hợp mới tương ứng thay vì chỉ kiểm tra cấu trúc mã.

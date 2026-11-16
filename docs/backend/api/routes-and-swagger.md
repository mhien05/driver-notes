# Router HTTP và Swagger

## File nguồn

- [`backend/src/api/mod.rs`](../../../backend/src/api/mod.rs): đăng ký route, giới hạn body của restore, CORS và Swagger UI.
- [`backend/src/api/docs.rs`](../../../backend/src/api/docs.rs): tập hợp handler/schema thành OpenAPI và khai báo bearer security.
- Annotation `#[utoipa::path]` nằm ngay trên handler trong `auth.rs`, `shifts.rs`, `entries.rs` và `backup.rs`.

## Route hiện có

| Nhóm | Route |
| --- | --- |
| Kiểm tra sống | `GET /health` |
| Tài khoản | `POST /api/v1/auth/register`, `POST /api/v1/auth/login`, `POST /api/v1/auth/logout`, `GET /api/v1/auth/me` |
| Ca làm | `GET/POST /api/v1/shifts`, `PUT/DELETE /api/v1/shifts/{id}`, `POST /api/v1/shifts/{id}/end` |
| Giao dịch | `GET/POST /api/v1/entries`, `PUT/DELETE /api/v1/entries/{id}` |
| Sao lưu | `GET /api/v1/snapshot`, `POST /api/v1/restore` |

`GET /health` trả 200 và không cần token. Đăng ký và đăng nhập cũng là route công khai; các route dữ liệu dùng `Authorization: Bearer <token>`. Giới hạn JSON cho riêng `POST /api/v1/restore` là **10 MiB**.

## OpenAPI và thử API

- Swagger UI: <http://127.0.0.1:3000/swagger-ui/>
- OpenAPI JSON: <http://127.0.0.1:3000/api-docs/openapi.json>

UI được đóng gói cùng binary. `docs.rs` đặt bearer security mặc định, còn đăng ký/đăng nhập khai báo truy cập công khai. Sau khi đăng nhập, dán **giá trị token** vào nút **Authorize**; Swagger sẽ thêm tiền tố `Bearer`.

Khi thêm/sửa route, cập nhật đồng thời router, annotation trên handler, danh sách `paths`/`schemas` trong `docs.rs` và trang tài liệu của cụm chức năng. Bài test trong [`backend/src/tests.rs`](../../../backend/src/tests.rs) kiểm tra đường dẫn OpenAPI, bearer scheme và trang Swagger.

## CORS khi phát triển

Nếu có `FRONTEND_ORIGIN`, backend chỉ trả header CORS cho origin được cấu hình và cho phép `GET`, `POST`, `PUT`, `DELETE` cùng header `Authorization`, `Content-Type`. Khi chạy Vite local, frontend dùng proxy `/api` đến `127.0.0.1:3000`, nên browser gửi yêu cầu cùng origin frontend.

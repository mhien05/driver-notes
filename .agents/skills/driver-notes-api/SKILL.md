---
name: driver-notes-api
description: Hướng dẫn dùng và phát triển Driver Notes API v1 trong repository này. Dùng khi nối frontend Svelte với backend, tạo client/kiểu dữ liệu, sửa route hoặc schema Rust, kiểm tra phân quyền tài xế, Swagger, snapshot và chế độ xem offline.
---

# Driver Notes API

Skill này áp dụng cho API của repository Driver Notes. Đọc `AGENTS.md` ở root trước khi sửa mã. Dùng thêm `ui-ux-pro-max` khi công việc gồm thiết kế hoặc cải thiện giao diện.

## Nguồn sự thật

1. Xác định root dự án chứa `backend/`, `frontend/` và `docs/backend/`.
2. Đọc `docs/README.md` và trang `docs/backend/` của cụm chức năng liên quan.
3. Đối chiếu `backend/src/api/mod.rs`, handler tương ứng và `backend/src/api/docs.rs`. Khi tài liệu và mã khác nhau, lấy mã cùng migration làm căn cứ rồi cập nhật tài liệu.
4. Xem [bản đồ contract](references/contracts.md) để định hướng nhanh; kiểm tra lại source trước khi thêm trường, mã trạng thái hoặc ràng buộc.

## Quy tắc tích hợp

- API v1 dùng JSON `camelCase`, Unix milliseconds cho thời gian và số nguyên VND cho tiền. Không chuyển tiền sang số thực.
- Ngoài đăng ký và đăng nhập, gửi `Authorization: Bearer <token>`. `GET /health` là route công khai.
- Mỗi tài xế có dữ liệu riêng. Backend lấy `user_id` từ token; không nhận một `userId` do client truyền để quyết định phạm vi dữ liệu. Mọi `shiftId` gắn với giao dịch phải thuộc tài khoản hiện tại.
- Server là nguồn dữ liệu chính. Khi online, tải lại dữ liệu từ API; chỉ lưu snapshot thành công để xem khi offline. Khi offline, hiện thời điểm cập nhật và khóa các thao tác ghi; không tự xếp hàng lệnh ghi.
- Frontend hiện là Svelte 5 + TypeScript + Vite, không phải SvelteKit. Khi tham khảo `ui-ux-pro-max`, chỉ áp dụng hướng dẫn khớp stack này.
- Với `PUT`, gửi đầy đủ các trường của input tương ứng. Phân biệt `null`, trường vắng mặt và giá trị mặc định theo struct Rust. Xử lý phản hồi `204` không có JSON và lỗi dạng `{ "error": "..." }`.
- `POST /api/v1/restore` thay toàn bộ ca và giao dịch của tài khoản trong một transaction; chỉ gọi trong luồng khôi phục mà người dùng chủ động chọn. ID được cấp lại sau khi restore, nên tải snapshot mới sau khi hoàn tất.

## Khi thay đổi API

1. Sửa handler, xác thực và truy vấn SQLite trong `backend/src/api/`; giữ truy vấn theo `user_id` và kiểm tra quyền sở hữu ca.
2. Cập nhật route trong `backend/src/api/mod.rs`, annotation `utoipa` trên handler, danh sách `paths`/`schemas` trong `backend/src/api/docs.rs`, và trang liên quan ở `docs/backend/`.
3. Nếu thay đổi schema SQLite, thêm migration cho database đã tồn tại; không chỉ sửa `001_init.sql`.
4. Cập nhật client và kiểu TypeScript ở `frontend/` nếu contract đổi. Giữ đường gọi `/api` qua Vite proxy khi phát triển local.
5. Kiểm tra hành vi liên quan bằng `cargo fmt -- --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings` từ `backend/`; dùng `npm run build` từ `frontend/` khi sửa frontend. Bổ sung test cho thay đổi về auth, phân quyền, tiền, ca làm hoặc restore.

Swagger UI ở `/swagger-ui/`; OpenAPI JSON ở `/api-docs/openapi.json` khi backend chạy. Dùng OpenAPI để thử request, nhưng handler và migration quyết định hành vi thực tế.

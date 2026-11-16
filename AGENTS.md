# Hướng dẫn làm việc với Driver Notes

## Cấu trúc repository

- `backend/`: Rust, Axum, SQLite; API là nguồn dữ liệu chính.
- `frontend/`: Svelte 5, TypeScript, Vite; hiện là nền tảng giao diện.
- `docs/backend/`: tài liệu theo từng cụm chức năng của backend. Bắt đầu từ [`docs/README.md`](docs/README.md).

## Khi sửa backend

1. Đọc tài liệu của cụm chức năng liên quan, rồi kiểm tra mã nguồn tương ứng. Mã nguồn và migration là căn cứ cuối cùng nếu tài liệu chưa kịp cập nhật.
2. Giữ mọi truy vấn dữ liệu tài xế trong phạm vi `user_id` lấy từ phiên đã xác thực. Kiểm tra quyền sở hữu khi bản ghi tham chiếu ca làm.
3. Dùng `AppState` và `with_db` cho truy cập SQLite; giữ thao tác ghi nhiều bảng trong một transaction khi cần tính toàn vẹn.
4. Khi đổi API, cập nhật route trong `backend/src/api/mod.rs`, annotation OpenAPI trên handler, schema trong `backend/src/api/docs.rs` nếu cần, và tài liệu tương ứng trong `docs/backend/`.
5. Khi đổi cấu trúc dữ liệu, bổ sung đường migration cho database hiện có và ghi rõ cách chuyển dữ liệu; chỉ sửa `001_init.sql` sẽ không nâng cấp bảng đã tạo.
6. Bổ sung kiểm tra hành vi cho thay đổi có rủi ro về xác thực, quyền truy cập, tiền, ca làm hoặc khôi phục dữ liệu.

Kiểm tra từ thư mục `backend/`:

```powershell
cargo fmt -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Khi sửa frontend

- Dùng Svelte 5, TypeScript và CSS thuần; giữ giao diện gọn, ưu tiên điện thoại.
- Trong local, Vite chuyển `/api` đến backend tại `127.0.0.1:3000`.
- Theo quyết định sản phẩm hiện tại, server là nguồn dữ liệu chính; bản offline chỉ để xem dữ liệu đã tải gần nhất. Không tự thêm hàng đợi ghi offline.
- Chạy `npm run build` từ `frontend/` để kiểm tra Svelte/TypeScript và bản build Vite.

## Tài liệu và file cục bộ

- Viết tài liệu và trao đổi với người dùng bằng tiếng Việt. Cập nhật đúng trang trong `docs/backend/` cùng thay đổi mã liên quan; giữ mục lục và đường dẫn nguồn chính xác.
- Không đưa `target/`, `node_modules/`, `dist/`, database local, `.env` hoặc thông tin bí mật vào Git.

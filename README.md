# Driver Notes

Repository gồm hai phần độc lập:

| Thư mục | Công nghệ | Vai trò |
| --- | --- | --- |
| [`backend/`](backend/) | Rust, Axum, SQLite | API, xác thực và dữ liệu tài xế |
| [`frontend/`](frontend/) | Svelte 5, TypeScript, Vite | Nền tảng giao diện web |

## Chạy local

Mở hai terminal tại thư mục gốc của repository:

```powershell
cd backend
cargo run
```

```powershell
cd frontend
npm install
npm run dev
```

Frontend chạy trên địa chỉ do Vite cung cấp, thường là <http://localhost:5173>. Backend chạy tại <http://127.0.0.1:3000>; Swagger UI ở <http://127.0.0.1:3000/swagger-ui/>. Vite chuyển các yêu cầu `/api` đến backend khi phát triển local.

Xem [hướng dẫn backend](backend/README.md), [hướng dẫn frontend](frontend/README.md) và [tài liệu theo cụm chức năng](docs/README.md).

# Driver Notes Frontend

Giao diện nền tảng bằng Svelte 5, TypeScript và Vite.

```powershell
cd frontend
npm install
npm run dev
```

Mở địa chỉ do Vite in ra (thường là <http://localhost:5173>). Trong lúc phát triển, yêu cầu tới `/api` được chuyển sang backend tại `http://127.0.0.1:3000`.

Kiểm tra kiểu và tạo bản build:

```powershell
npm run check
npm run build
```

Thư mục `dist/` là bản build tĩnh để triển khai lên Cloudflare Pages. Kết nối API production và phần PWA sẽ được cấu hình khi triển khai các chức năng giao diện.

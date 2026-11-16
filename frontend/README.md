# Driver Notes Frontend

Giao diện Driver Notes bằng Svelte 5, TypeScript và Vite. Có các màn hình đăng nhập/đăng ký, tổng quan trong ngày, giao dịch, ca làm và tài khoản.

```powershell
cd frontend
npm install
npm run dev
```

Chạy backend trước, rồi mở địa chỉ do Vite in ra (thường là <http://localhost:5173>). Trong lúc phát triển, yêu cầu tới `/api` được chuyển sang backend tại `http://127.0.0.1:3000`.

Giao diện lấy dữ liệu ca và giao dịch từ API. Phiên đăng nhập được giữ trong tab trình duyệt. Sau mỗi lần tải thành công, ứng dụng lưu bản dữ liệu gần nhất theo tài khoản để xem khi mất mạng; khi đó các nút ghi dữ liệu bị khóa. Dữ liệu sẽ được tải lại từ server khi kết nối trở lại.

Kiểm tra kiểu và tạo bản build:

```powershell
npm run check
npm run build
```

Thư mục `dist/` là bản build tĩnh để triển khai lên Cloudflare Pages. Khi triển khai, cần chuyển tiếp `/api` đến backend cùng origin. Phần PWA chưa được cấu hình.

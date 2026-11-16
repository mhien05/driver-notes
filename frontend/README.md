# Driver Notes Frontend

Giao diện Driver Notes bằng Svelte 5, TypeScript và Vite. Có các màn hình đăng nhập/đăng ký, tổng quan trong ngày, giao dịch, ca làm và tài khoản.

```powershell
cd frontend
npm install
npm run dev
```

Chạy backend trước, rồi mở địa chỉ do Vite in ra (thường là <http://localhost:5173>). Trong lúc phát triển, yêu cầu tới `/api` được chuyển sang backend tại `http://127.0.0.1:3000`.

## Thử trên iPhone cùng Wi-Fi

1. Trên máy tính, chạy backend (`cargo run` trong `backend/`).
2. Trong `frontend/`, chạy `npm run dev:phone`. Vite sẽ in ra địa chỉ **Network**, ví dụ `http://192.168.1.10:5173/`.
3. Cho iPhone và máy tính vào cùng Wi-Fi, mở đúng địa chỉ Network đó bằng Safari trên iPhone. Nếu Windows hỏi quyền tường lửa, cho phép trên mạng riêng.
4. Trong Safari, dùng **Chia sẻ → Thêm vào Màn hình chính** rồi mở biểu tượng Driver Notes. Web app trên màn hình chính có vùng lưu trữ riêng, nên lần đầu cần đăng nhập lại.

`localhost` trên iPhone trỏ về chính iPhone, không trỏ về máy tính. Chỉ dùng HTTP qua Wi-Fi tin cậy với tài khoản thử; khi sử dụng thật, triển khai frontend và API qua HTTPS.

Giao diện lấy dữ liệu ca và giao dịch từ API. Phiên đăng nhập được giữ trên thiết bị tới khi đăng xuất hoặc token hết hạn (API hiện cấp token 30 ngày). Sau mỗi lần tải thành công, ứng dụng lưu bản dữ liệu gần nhất theo tài khoản để xem khi mất mạng; khi đó các nút ghi dữ liệu bị khóa. Dữ liệu sẽ được tải lại từ server khi kết nối trở lại. Chưa có service worker, nên không bảo đảm mở lại ứng dụng khi đang mất mạng.

Kiểm tra kiểu và tạo bản build:

```powershell
npm run check
npm run build
```

Thư mục `dist/` là bản build tĩnh để triển khai lên Cloudflare Pages. Mặc định frontend gọi `/api` cùng origin: khi triển khai cần chuyển tiếp đường này đến backend. Nếu API ở origin HTTPS khác, đặt biến build `VITE_API_ORIGIN` bằng origin đó (không thêm `/api`) và đặt `FRONTEND_ORIGIN` của backend bằng origin frontend để CORS cho phép. Cấu hình mở từ màn hình chính và biểu tượng đã có; phần lưu giao diện để mở ngoại tuyến chưa được cấu hình.

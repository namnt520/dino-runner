# Dino Runner

Bản làm lại game khủng long chạy của Chrome (trang "không có kết nối mạng") bằng **Rust** và **[macroquad](https://macroquad.rs/)**.
Toàn bộ hình ảnh được vẽ bằng code và toàn bộ âm thanh được tổng hợp bằng code, nên game không cần file hình hay file âm thanh đi kèm.

## Tính năng

- Chạy, nhảy, cúi và rơi nhanh; tốc độ tăng dần theo thời gian.
- Xương rồng nhiều kiểu cụm, mỗi cây có hitbox riêng; chim bay ở ba độ cao (xuất hiện từ 300 điểm).
- Khoảng cách giữa các chướng ngại luôn đủ để nhảy qua ở mọi tốc độ.
- Chu kỳ ngày và đêm theo điểm số, có sao và mặt trăng.
- Thời tiết ngẫu nhiên từ 150 điểm: mưa, gió, bão cát (gió và bão cát làm cú nhảy yếu đi một chút).
- Âm thanh: tiếng nhảy, tiếng chết, tiếng "ting" mỗi 100 điểm, âm nền cho mưa, gió và bão cát.
- Hiệu ứng: bụi khi tiếp đất, rung màn hình khi chết, điểm nhấp nháy mỗi 100 điểm.
- High score được lưu ra file.
- Skin cho khủng long, mở khóa theo high score.

## Điều khiển

| Phím | Tác dụng |
|---|---|
| `Space` hoặc `↑` | Bắt đầu, nhảy, chơi lại |
| `↓` | Cúi khi ở trên đất, rơi nhanh khi đang nhảy |
| `←` `→` | Đổi skin (ở màn hình chờ và màn hình thua) |

## Skin

| Skin | Mở khóa khi high score đạt |
|---|---|
| Classic | 0 |
| Shades | 300 |
| Top Hat | 700 |
| Crown | 1500 |

## Chạy game

Cần cài [Rust](https://www.rust-lang.org/tools/install) (bản stable).

```bash
cargo run --release
```

Trên Linux, âm thanh cần thư viện ALSA, ví dụ trên Ubuntu hoặc Debian:

```bash
sudo apt install libasound2-dev pkg-config libx11-dev libxi-dev libgl1-mesa-dev
```

Nếu máy không có thiết bị âm thanh, game vẫn chạy bình thường, chỉ im lặng.

## Chạy test

```bash
cargo test
```

Test bao phủ vật lý nhảy, khoảng cách an toàn giữa chướng ngại, hitbox, ghi và đọc high score, tổng hợp âm thanh, thời tiết, chu kỳ ngày đêm và skin.

## High score

High score và skin đang chọn được lưu trong file `highscore.txt` ở **thư mục hiện tại khi chạy** (chạy `cargo run` trong thư mục dự án thì file nằm cạnh `Cargo.toml`). File gồm hai dòng: dòng 1 là high score, dòng 2 là chỉ số skin. File chỉ có một dòng (bản cũ) vẫn đọc được. Xóa file để đặt lại.

## Cấu trúc mã nguồn

```
src/
├── main.rs          Cửa sổ, vòng lặp chính, nối âm thanh với game
├── config.rs        Hằng số: kích thước màn hình, vật lý, tốc độ, ngưỡng điểm
├── theme.rs         Bảng màu ngày và đêm, pha trộn giữa hai bảng
├── input.rs         Đọc phím thành một struct, game không gọi trực tiếp macroquad
├── game.rs          Máy trạng thái (chờ, chơi, thua) và điều phối mọi thành phần
├── score.rs         Điểm, mốc 100 điểm, high score và skin được lưu ra file
├── spawner.rs       Sinh chướng ngại, bảo đảm khoảng cách đủ để nhảy qua
├── collision.rs     Kiểm tra va chạm giữa khủng long và chướng ngại
├── effects.rs       Bụi tiếp đất và rung màn hình
├── ui.rs            Điểm, nhãn thời tiết, màn hình chờ và màn hình thua
├── audio.rs         Phát âm thanh theo sự kiện, quản lý âm nền thời tiết
├── synth.rs         Tổng hợp âm thanh thành file WAV trong bộ nhớ
├── entities/
│   ├── dino.rs      Khủng long: vật lý, cúi, hitbox, hình vẽ
│   ├── skin.rs      Skin, điều kiện mở khóa, phụ kiện
│   ├── cactus.rs    Cụm xương rồng, hitbox từng cây
│   ├── bird.rs      Chim bay ở ba độ cao
│   └── obstacle.rs  Enum gộp xương rồng và chim
└── world/
    ├── ground.rs    Mặt đất và sỏi đá
    ├── cloud.rs     Mây nền
    ├── sky.rs       Chu kỳ ngày đêm, sao, mặt trăng
    └── weather.rs   Mưa, gió, bão cát
```

`Game` chỉ phát các sự kiện (nhảy, chết, qua mốc điểm); việc phát âm thanh do `main.rs` và `audio.rs` quyết định, nên logic game không phụ thuộc thiết bị âm thanh và test được mà không cần màn hình hay loa.

## Giấy phép

Phát hành theo giấy phép [MIT](LICENSE).

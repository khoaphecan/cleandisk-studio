# CleanDisk Studio

> A modern Windows desktop utility for organizing downloads, finding large and
> stale files, detecting exact duplicates, cleaning developer/application
> caches, and running selected disk and Windows maintenance operations.

[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows-0078D4?logo=windows)](https://www.microsoft.com/windows)
[![GUI](https://img.shields.io/badge/GUI-egui%20%2F%20eframe-7B61FF)](https://github.com/emilk/egui)

CleanDisk Studio is a native Rust + egui application for Windows. It is
designed to make storage maintenance understandable and reversible wherever
possible. The application scans first, shows the affected paths and sizes, and
asks for confirmation before destructive operations.

## Language / 言語 / 语言 / Язык

- [Tiếng Việt](#tiếng-việt)
- [English](#english)
- [简体中文](#简体中文)
- [日本語](#日本語)
- [Русский](#русский)

---

## Tiếng Việt

### Tổng quan

CleanDisk Studio là ứng dụng desktop Windows viết bằng Rust, cung cấp các
công cụ phân tích dung lượng, sắp xếp thư mục tải xuống, tìm file trùng,
dọn cache phát triển và thực hiện một số thao tác bảo trì Windows.

Ứng dụng hỗ trợ giao diện:

- Tiếng Việt
- English
- 简体中文
- 日本語
- Русский

Font fallback được cấu hình cho chữ Trung, Nhật và Hàn để hạn chế lỗi ô vuông
khi hiển thị trên Windows.

### Tính năng

#### Sắp xếp file tải xuống

- Theo dõi một hoặc nhiều thư mục.
- Phân loại theo tên thư mục, phần mở rộng và từ khóa trong tên file.
- Chọn ưu tiên từ khóa hoặc phần mở rộng.
- Tính dung lượng và số lượng file trong thư mục.
- Chế độ watcher tự động xử lý file mới trong khi ứng dụng đang chạy.
- Xuất, nhập và khôi phục bộ quy tắc phân loại bằng JSON.

#### Phân tích dung lượng

- Quét một thư mục và toàn bộ thư mục con.
- Liệt kê Top 50 hoặc Top 100 file lớn nhất.
- Ngưỡng lọc 500 MB, 1 GB hoặc 5 GB.
- Tìm file cũ theo thời điểm sửa đổi 30, 60 hoặc 90 ngày.
- Nhóm dung lượng theo loại: Games, Video, Images, Audio, Archives,
  Documents, Code & data và Other.
- Kết quả phân tích là chế độ chỉ đọc.

> Windows có thể tắt hoặc giới hạn việc ghi nhận thời điểm truy cập cuối.
> Vì vậy bộ lọc file cũ sử dụng thời điểm sửa đổi, không sử dụng thời điểm
> mở file.

#### Tìm file trùng chính xác

Quy trình nhiều bước để tránh đọc toàn bộ file không cần thiết:

1. Nhóm file theo kích thước.
2. Chỉ băm các nhóm có cùng kích thước.
3. Dùng BLAKE3 để xác nhận nội dung giống nhau 100%.

Có thể chọn giữ bản mới nhất hoặc cũ nhất. File được chọn có thể chuyển vào
`.cleandisk-quarantine` để khôi phục thủ công, hoặc xử lý theo thao tác xóa
được xác nhận trong giao diện.

#### Dọn cache môi trường lập trình

Quét các thư mục cache/build phổ biến:

- Rust: `target/`
- Node.js: `node_modules/`
- Python: `.venv/`, `venv/`, `__pycache__/`
- Visual Studio / .NET: `bin/`, `obj/`, `.vs/`

Cache lồng nhau được tính cho thư mục cache gần nhất để tránh cộng dung lượng
trùng. Có thể chọn cache và chuyển vào khu cách ly.

#### Dọn cache ứng dụng và shader

Ứng dụng nhận diện các đường dẫn cache phổ biến của:

- Chrome, Edge và Brave
- Discord
- Spotify
- DirectX shader cache
- NVIDIA / AMD shader cache khi đường dẫn tồn tại

Cookie và lịch sử trình duyệt không nằm trong các đường dẫn cache được chọn.
Hãy đóng ứng dụng liên quan trước khi dọn cache để giảm khả năng file bị khóa.

#### Công cụ ổ đĩa và Windows

- Xem danh sách ổ đĩa vật lý và dung lượng trống.
- Đọc thông tin S.M.A.R.T. khi Windows cung cấp dữ liệu.
- SSD: yêu cầu Windows ReTrim/TRIM.
- HDD: Analyze và Defragment thông qua lệnh Windows.
- Nhận diện SSD/HDD ưu tiên `Get-Disk.MediaType`, kết hợp BusType, MediaType
  của PhysicalDisk và tốc độ spindle; nếu Windows không xác định được loại ổ,
  các nút Optimize sẽ bị khóa để tránh dùng sai lệnh.
- Benchmark đọc/ghi tuần tự bằng file tạm.
- Dọn thư mục Temp của người dùng.
- Chạy Windows Update cleanup khi được hỗ trợ.
- Flush DNS bằng `ipconfig /flushdns`.
- Chạy SFC/DISM với lời nhắc UAC.
- Lên lịch bảo trì 7 ngày trong lúc ứng dụng còn chạy.

Lịch bảo trì hiện chỉ tồn tại trong phiên chạy hiện tại; nó không tự chạy sau
khi thoát ứng dụng hoặc sau khi khởi động lại Windows.

#### System tray và tooltip

- Đóng cửa sổ sẽ đưa ứng dụng xuống system tray nếu tray khả dụng.
- Menu tray có Open và Exit.
- Nhấp biểu tượng tray để mở lại cửa sổ.
- Có thể bật **Khởi động cùng Windows** trong tab Organize. Ứng dụng sẽ đăng
  ký cho người dùng hiện tại, khởi chạy ẩn sau khi đăng nhập và tự bật Auto
  Watcher. Khi bật, file khớp quy tắc có thể được di chuyển tự động; hãy kiểm
  tra thư mục và quy tắc trước khi xác nhận. Bỏ chọn để xóa đăng ký.
- Exit từ menu tray kết thúc đúng tiến trình hiện tại bằng PID riêng, không
  ảnh hưởng các ứng dụng CleanDisk khác.
- Tooltip giải thích cách hoạt động và cảnh báo theo ngôn ngữ đang chọn.
- Màu biểu tượng tab và tray giúp nhận diện nhanh hơn.

### Yêu cầu hệ thống

- Windows 10 hoặc Windows 11.
- Rust stable và Cargo nếu build từ mã nguồn.
- Một số thao tác bảo trì cần quyền Administrator và sẽ hiển thị UAC.
- Dung lượng trống đủ cho benchmark nếu muốn chạy kiểm tra tốc độ.

### Cài đặt và chạy từ mã nguồn

```powershell
git clone <repository-url>
cd download_cleaner
cargo run --release
```

Kiểm tra mã nguồn:

```powershell
cargo fmt --all
cargo check --offline --all-targets
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
```

Nếu máy chưa có dependency trong Cargo cache, chạy một lần:

```powershell
cargo fetch
```

### An toàn và quyền quản trị

- Luôn kiểm tra thư mục, kích thước và danh sách file trước khi xác nhận.
- Chức năng quét không tự xóa file.
- Khu cách ly được đặt trong `.cleandisk-quarantine` dưới thư mục quét.
- Không cấp quyền Administrator nếu thao tác không cần.
- Không chạy dọn cache khi ứng dụng đang sử dụng cache đó.
- Benchmark ghi dữ liệu thật lên ổ đĩa và cần dung lượng trống.
- Không sử dụng TRIM/Defragment như công cụ kiểm tra sức khỏe ổ đĩa.
- Hãy sao lưu dữ liệu quan trọng trước thao tác xóa vĩnh viễn.

### Cấu trúc dự án

```text
.
├── Cargo.toml       # Metadata và dependency Rust
├── src/
│   ├── main.rs      # GUI, tray, watcher, Windows commands và application state
│   ├── disk_tools.rs# Quét file, duplicate finder, quarantine, benchmark
│   └── i18n.rs      # Bản dịch 5 ngôn ngữ và tooltip
└── README.md
```

### Kiểm thử

Test hiện có bao phủ:

- Mã hóa lệnh PowerShell UTF-16LE Base64.
- Escape chuỗi PowerShell.
- Dọn đúng nội dung thư mục Temp.
- Tìm duplicate bằng kích thước + BLAKE3.
- Quarantine không chạm vào bản được giữ lại.
- Phát hiện và quarantine dev cache.
- Cache lồng nhau được tính theo cache gần nhất.
- Benchmark tạo, đọc và xóa file tạm.
- Font fallback Trung/Nhật/Hàn trên Windows.
- Bản dịch điều hướng và tooltip cơ bản.

### Giới hạn hiện tại

- Đây là ứng dụng Windows; các lệnh bảo trì Windows không có ý nghĩa trên
  Linux hoặc macOS.
- Lịch bảo trì không phải Windows Task Scheduler và không chạy khi ứng dụng
  đã thoát.
- Thời gian quét phụ thuộc số lượng file, quyền truy cập và tốc độ ổ đĩa.
- Hash BLAKE3 toàn bộ file lớn có thể sử dụng nhiều I/O.
- Một số đường dẫn cache chỉ được dọn nếu tồn tại và có thể truy cập.
- Không nên coi benchmark đơn giản này là phép đo chứng nhận hoặc chẩn đoán
  sức khỏe ổ đĩa chuyên nghiệp.

### Đóng góp

1. Tạo fork và branch riêng.
2. Thực hiện thay đổi nhỏ, có kiểm thử.
3. Chạy `cargo fmt`, `cargo check`, `cargo test` và `cargo clippy`.
4. Mô tả rõ hành vi thay đổi và các quyền Windows liên quan trong Pull Request.

### Giấy phép

Repository hiện chưa khai báo license. Hãy bổ sung file `LICENSE` và trường
license trong `Cargo.toml` trước khi phát hành hoặc tái sử dụng ngoài phạm vi
được chủ sở hữu cho phép.

---

## English

### Overview

CleanDisk Studio is a native Windows desktop application written in Rust. It
combines storage analysis, download-folder organization, exact duplicate
detection, developer-cache cleanup, application-cache cleanup, and selected
Windows maintenance actions in one interface.

The interface is available in Vietnamese, English, Simplified Chinese,
Japanese, and Russian. CJK font fallbacks are configured for Chinese, Japanese,
and Korean glyph coverage on Windows.

### Features

- **Download organization:** classify files by folder name, extension, and
  filename keywords; choose keyword-first or extension-first matching; export
  and import JSON rules; optionally watch folders while the app is running.
- **Space analysis:** recursively scan a folder, show the largest 50 or 100
  files, filter at 500 MB / 1 GB / 5 GB, find files not modified for 30 / 60 /
  90 days, and group space by file category.
- **Exact duplicate finder:** group by file size, then verify matching content
  with a full BLAKE3 hash. Keep the newest or oldest copy and quarantine
  selected duplicates.
- **Developer cache cleanup:** detect Rust `target/`, Node `node_modules/`,
  Python virtual-environment and `__pycache__` folders, and Visual Studio
  `bin/`, `obj/`, and `.vs/` directories. Nested caches are accounted for by
  the nearest cache folder.
- **Application and shader caches:** inspect supported Chrome, Edge, Brave,
  Discord, Spotify, DirectX, NVIDIA, and AMD cache locations without targeting
  browser cookies or history.
- **Disk tools:** inspect disks, query available S.M.A.R.T. information, run
  SSD ReTrim, HDD Analyze/Defragment, measure sequential read/write speed with
  a temporary file, clean user Temp, flush DNS, and invoke SFC/DISM with UAC.
- **Tray and localized UX:** minimize to the system tray, reopen from the tray,
  exit from the tray menu, and display localized feature tooltips with
  operation details and cautions.
- **Optional Windows startup:** register the current executable under the
  current user's `HKCU\...\Run` key with `--auto-watch`. Disable the checkbox
  to remove this entry; automatic cleanup can move matching files after login.

### Important behavior

- Scans are read-only.
- Quarantine moves selected items into `.cleandisk-quarantine` while
  preserving relative paths.
- Maintenance operations may require Administrator approval.
- The seven-day maintenance schedule is session-only and does not survive
  application exit.
- The stale-file filter uses modification time, not last-access time.
- The benchmark writes real data and requires sufficient free space.

### Requirements

- Windows 10 or Windows 11.
- Rust stable and Cargo for source builds.
- Administrator approval for selected Windows maintenance operations.

### Build and run

```powershell
git clone <repository-url>
cd download_cleaner
cargo run --release
```

Validation commands:

```powershell
cargo fmt --all
cargo check --offline --all-targets
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
```

Run `cargo fetch` once if the required crates are not already cached.

### Project layout

```text
src/main.rs       GUI, tray, watcher, Windows commands, and application state
src/disk_tools.rs scanning, duplicate detection, quarantine, and benchmarking
src/i18n.rs       five-language translations and tooltip helpers
```

### Safety

Review every path and size before confirming an operation. Close applications
before clearing their caches. Back up important data before permanent deletion.
Do not treat the built-in benchmark as a professional disk-health diagnostic.

### Testing and contribution

The test suite covers PowerShell encoding and escaping, Temp cleanup,
duplicate detection, quarantine behavior, nested developer caches, benchmark
cleanup, font fallback, and localization. Contributions should include focused
tests and pass formatting, checking, testing, and Clippy.

This repository currently does not declare a license. Add a `LICENSE` file and
appropriate Cargo metadata before publishing or redistributing it.

---

## 简体中文

### 项目简介

CleanDisk Studio 是一个使用 Rust 编写的原生 Windows 桌面工具，用于整理
下载目录、分析磁盘空间、查找完全重复的文件、清理开发缓存和应用缓存，
并执行部分 Windows 维护操作。

界面支持越南语、英语、简体中文、日语和俄语。Windows 下配置了中文、
日文和韩文字体回退，以减少字符显示为方框的问题。

### 主要功能

- **下载目录整理：**按照文件夹名称、扩展名和文件名关键词分类；支持
  关键词优先或扩展名优先；可导入、导出 JSON 规则；应用运行时可监视目录。
- **空间分析：**递归扫描目录，显示最大的 50 或 100 个文件，支持
  500 MB、1 GB 和 5 GB 阈值，以及 30、60、90 天未修改文件筛选。
- **精确重复文件查找：**先按文件大小分组，再使用完整 BLAKE3 哈希确认
  内容完全相同；可以保留最新或最旧副本，并将选中的副本移入隔离区。
- **开发缓存清理：**识别 Rust `target/`、Node.js `node_modules/`、
  Python `.venv/`、`venv/`、`__pycache__/` 以及 Visual Studio 的
  `bin/`、`obj/`、`.vs/`。嵌套缓存按最近的缓存目录计算。
- **应用和着色器缓存：**支持检查 Chrome、Edge、Brave、Discord、Spotify、
  DirectX、NVIDIA 和 AMD 的常见缓存目录；不会把 Cookie 和浏览历史作为
  目标。
- **磁盘和 Windows 工具：**查看磁盘、读取可用的 S.M.A.R.T. 信息、对 SSD
  执行 ReTrim、对 HDD 执行 Analyze/Defragment、测试顺序读写速度、清理
  用户 Temp、刷新 DNS，以及在 UAC 确认后运行 SFC/DISM。
- **系统托盘和本地化提示：**关闭窗口时可隐藏到托盘，从托盘重新打开或
  退出，并显示与当前语言同步的功能说明和注意事项。

### 安全说明

扫描操作是只读的。隔离操作会将文件或文件夹移动到扫描目录下的
`.cleandisk-quarantine`，并保留相对路径。部分维护操作需要管理员权限。
七天维护计划只在应用当前运行会话中有效。旧文件筛选使用修改时间，而
不是最后访问时间。基准测试会向磁盘写入真实临时数据。

### 构建和测试

需要 Windows 10/11、Rust stable 和 Cargo：

```powershell
git clone <repository-url>
cd download_cleaner
cargo run --release
cargo fmt --all
cargo check --offline --all-targets
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
```

如果依赖尚未缓存，可先执行 `cargo fetch`。

### 项目结构

- `src/main.rs`：GUI、托盘、目录监视、Windows 命令和应用状态。
- `src/disk_tools.rs`：扫描、重复文件检测、隔离和基准测试。
- `src/i18n.rs`：五种语言的翻译和 tooltip 辅助函数。

请在确认前检查路径、大小和文件列表。清理缓存前建议关闭相关应用，
重要数据请先备份。项目当前尚未声明开源许可证，发布或再分发前请补充
`LICENSE` 文件和 Cargo license 元数据。

---

## 日本語

### 概要

CleanDisk Studio は Rust と egui で作られた Windows 向けデスクトップ
ユーティリティです。ダウンロードフォルダーの整理、容量分析、完全一致
する重複ファイルの検出、開発キャッシュとアプリケーションキャッシュの
整理、Windows の一部メンテナンス操作を提供します。

画面はベトナム語、英語、簡体字中国語、日本語、ロシア語に対応しています。
Windows では中国語・日本語・韓国語のフォントフォールバックを設定しています。

### 主な機能

- **ダウンロード整理：**フォルダー名、拡張子、ファイル名キーワードで分類。
  キーワード優先・拡張子優先を選択でき、JSON ルールの入出力にも対応。
- **容量分析：**フォルダーとサブフォルダーを再帰的に走査し、最大 50/100
  ファイル、500 MB/1 GB/5 GB の閾値、30/60/90 日間変更されていない
  ファイルを表示。
- **重複ファイル検出：**サイズで候補を絞り、BLAKE3 で内容を完全検証。
  新しいコピーまたは古いコピーを残し、選択したファイルを隔離できます。
- **開発キャッシュ：**Rust `target/`、Node.js `node_modules/`、Python
  `.venv/`・`venv/`・`__pycache__/`、Visual Studio の `bin/`・`obj/`・
  `.vs/` を検出。ネストしたキャッシュは最も近いキャッシュに計上します。
- **ディスクと Windows：**ディスク情報、利用可能な S.M.A.R.T. 情報、SSD
  ReTrim、HDD Analyze/Defragment、順次読み書きテスト、Temp 清掃、DNS
  フラッシュ、UAC 承認付き SFC/DISM。
- **トレイと tooltip：**システムトレイへの最小化、再表示、終了、選択中の
  言語に同期した説明と注意事項。

### 注意事項

スキャンは読み取り専用です。隔離ではスキャン対象フォルダー内の
`.cleandisk-quarantine` に相対パスを保ったまま移動します。一部の処理には
管理者承認が必要です。7 日メンテナンスは現在のアプリセッションのみ有効で、
終了後も自動実行される Windows タスクではありません。古いファイルの判定は
最終アクセス時刻ではなく変更時刻を使用します。

### ビルドとテスト

```powershell
git clone <repository-url>
cd download_cleaner
cargo run --release
cargo fmt --all
cargo check --offline --all-targets
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
```

依存クレートが未取得の場合は、先に `cargo fetch` を実行してください。
重要なデータは事前にバックアップし、キャッシュを削除する前に関連アプリを
終了してください。本リポジトリには現在ライセンスが設定されていないため、
公開・再配布前に `LICENSE` と Cargo の license メタデータを追加してください。

---

## Русский

### Обзор

CleanDisk Studio — нативное приложение для Windows, написанное на Rust и egui.
Оно помогает организовать загрузки, проанализировать место на диске, найти
полные дубликаты файлов, очистить кэши разработки и приложений, а также запустить
выбранные операции обслуживания Windows.

Интерфейс поддерживает вьетнамский, английский, упрощённый китайский, японский
и русский языки. Для Windows настроены fallback-шрифты для китайских, японских
и корейских символов.

### Возможности

- **Организация загрузок:** правила по имени папки, расширению и ключевым словам;
  приоритет ключевых слов или расширений; импорт и экспорт JSON; наблюдение за
  папками во время работы приложения.
- **Анализ места:** рекурсивное сканирование, Top 50/100 самых больших файлов,
  пороги 500 MB/1 GB/5 GB и фильтр файлов, не изменявшихся 30/60/90 дней.
- **Поиск точных дубликатов:** сначала сравнение размера, затем полный хэш BLAKE3.
  Можно оставить самую новую или самую старую копию и переместить выбранные файлы
  в карантин.
- **Кэши разработки:** Rust `target/`, Node.js `node_modules/`, Python
  `.venv/`, `venv/`, `__pycache__/`, Visual Studio `.vs/`, `bin/`, `obj/`.
  Вложенные кэши учитываются по ближайшей папке кэша.
- **Кэши приложений и шейдеров:** распространённые пути Chrome, Edge, Brave,
  Discord, Spotify, DirectX, NVIDIA и AMD; Cookie и история браузера не являются
  целями очистки.
- **Диски и Windows:** сведения о дисках, доступные данные S.M.A.R.T., ReTrim
  SSD, Analyze/Defragment HDD, последовательный тест чтения/записи, очистка
  Temp пользователя, сброс DNS и SFC/DISM с подтверждением UAC.
- **Трей и локализация:** скрытие в системном трее, повторное открытие, выход из
  меню трея и подсказки на выбранном языке.

### Безопасность и ограничения

Сканирование не изменяет файлы. При карантине выбранные элементы перемещаются в
`.cleandisk-quarantine` внутри сканируемой папки с сохранением относительных путей.
Некоторые операции требуют прав администратора. Расписание обслуживания раз в семь
дней работает только пока приложение запущено и не является постоянной задачей
Планировщика Windows. Фильтр старых файлов использует время изменения, а не время
последнего доступа. Тест скорости записывает реальные временные данные на диск.

### Сборка и проверка

```powershell
git clone <repository-url>
cd download_cleaner
cargo run --release
cargo fmt --all
cargo check --offline --all-targets
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
```

Если зависимости отсутствуют в локальном кеше Cargo, сначала выполните
`cargo fetch`. Проверяйте путь и список файлов перед подтверждением, закрывайте
связанные приложения перед очисткой кэша и делайте резервные копии важных данных.
Лицензия в репозитории пока не объявлена; перед публикацией или распространением
добавьте `LICENSE` и соответствующее поле license в `Cargo.toml`.

---

## Security notes

Please report suspected security issues privately to the repository owner rather
than publishing exploit details in a public issue. Do not run cleanup commands
against system folders unless you understand the command and have a verified
backup.

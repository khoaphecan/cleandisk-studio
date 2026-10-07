#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    English,
    #[default]
    Vietnamese,
    Chinese,
    Japanese,
    Russian,
}

impl Language {
    pub const ALL: [Self; 5] = [
        Self::English,
        Self::Vietnamese,
        Self::Chinese,
        Self::Japanese,
        Self::Russian,
    ];

    pub const fn native_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Vietnamese => "Tiếng Việt",
            Self::Chinese => "简体中文",
            Self::Japanese => "日本語",
            Self::Russian => "Русский",
        }
    }
}

pub fn text(language: Language, key: &'static str) -> &'static str {
    let translated = match key {
        "Organize" => ["Organize", "Sắp xếp", "整理", "整理", "Сортировка"],
        "System cleanup" => [
            "System cleanup",
            "Dọn dẹp hệ thống",
            "系统清理",
            "システムクリーンアップ",
            "Очистка системы",
        ],
        "Space analysis" => [
            "Space analysis",
            "Phân tích dung lượng",
            "空间分析",
            "容量分析",
            "Анализ места",
        ],
        "Duplicates" => [
            "Duplicates",
            "File trùng",
            "重复文件",
            "重複ファイル",
            "Дубликаты",
        ],
        "Dev caches" => [
            "Dev caches",
            "Cache lập trình",
            "开发缓存",
            "開発キャッシュ",
            "Кэш разработки",
        ],
        "Windows tools" => [
            "Windows tools",
            "Công cụ Windows",
            "Windows 工具",
            "Windows ツール",
            "Инструменты Windows",
        ],
        "Open CleanDisk Studio" => [
            "Open CleanDisk Studio",
            "Mở CleanDisk Studio",
            "打开 CleanDisk Studio",
            "CleanDisk Studio を開く",
            "Открыть CleanDisk Studio",
        ],
        "Close to tray · background watcher stays active" => [
            "Close to tray · background watcher stays active",
            "Đóng xuống khay · theo dõi nền vẫn hoạt động",
            "关闭到托盘 · 后台监控仍在运行",
            "閉じるとトレイに格納 · 監視を継続",
            "Закрыть в трей · фоновый мониторинг активен",
        ],
        "Rescan Disks" => [
            "Rescan Disks",
            "Quét lại ổ đĩa",
            "重新扫描磁盘",
            "ディスクを再スキャン",
            "Пересканировать диски",
        ],
        "PHYSICAL STORAGE DISKS (CLICK CARD FOR S.M.A.R.T. TELEMETRY)" => [
            "PHYSICAL STORAGE DISKS (CLICK CARD FOR S.M.A.R.T. TELEMETRY)",
            "Ổ ĐĨA VẬT LÝ (NHẤN THẺ ĐỂ XEM THÔNG TIN S.M.A.R.T.)",
            "物理磁盘（点击卡片查看 S.M.A.R.T. 信息）",
            "物理ディスク（カードをクリックして S.M.A.R.T. 情報を表示）",
            "ФИЗИЧЕСКИЕ ДИСКИ (НАЖМИТЕ ДЛЯ ПРОСМОТРА S.M.A.R.T.)",
        ],
        "CleanDisk Studio" => [
            "CleanDisk Studio",
            "CleanDisk Studio",
            "CleanDisk Studio",
            "CleanDisk Studio",
            "CleanDisk Studio",
        ],
        "Disk Optimization & Automation Utility" => [
            "Disk Optimization & Automation Utility",
            "Công cụ tối ưu và tự động hóa ổ đĩa",
            "磁盘优化与自动化工具",
            "ディスク最適化・自動化ツール",
            "Утилита оптимизации и автоматизации дисков",
        ],
        "Starting hardware bus telemetry engine..." => [
            "Starting hardware bus telemetry engine...",
            "Đang khởi động đo đạc phần cứng...",
            "正在启动硬件监测...",
            "ハードウェア監視を起動中...",
            "Запуск мониторинга оборудования...",
        ],
        "Loading file organization rules & JSON schemas..." => [
            "Loading file organization rules & JSON schemas...",
            "Đang tải quy tắc sắp xếp và cấu hình...",
            "正在加载整理规则和配置...",
            "整理ルールと設定を読み込み中...",
            "Загрузка правил сортировки и настроек...",
        ],
        "Calibrating system cluster matrix and clean stores..." => [
            "Calibrating system cluster matrix and clean stores...",
            "Đang chuẩn bị công cụ ổ đĩa và dọn dẹp...",
            "正在初始化磁盘与清理工具...",
            "ディスクとクリーンアップ機能を初期化中...",
            "Подготовка инструментов диска и очистки...",
        ],
        "Ready." => ["Ready.", "Sẵn sàng.", "就绪。", "準備完了。", "Готово."],
        "Close to tray" => [
            "Close to tray",
            "Thu nhỏ vào khay",
            "关闭到托盘",
            "トレイに格納",
            "Свернуть в трей",
        ],
        "Ready for next operation." => [
            "Ready for next operation.",
            "Sẵn sàng cho thao tác tiếp theo.",
            "已就绪，可以执行下一项操作。",
            "次の操作を実行できます。",
            "Готово к следующей операции.",
        ],
        "Status: Idle" => [
            "Status: Idle",
            "Trạng thái: Nhàn rỗi",
            "状态：空闲",
            "状態：待機中",
            "Состояние: ожидание",
        ],
        "[OK] COMPLETED" => [
            "[OK] COMPLETED",
            "[OK] HOÀN TẤT",
            "[OK] 已完成",
            "[OK] 完了",
            "[OK] ЗАВЕРШЕНО",
        ],
        "Organize files into folders using custom rules." => [
            "Organize files into folders using custom rules.",
            "Sắp xếp file vào thư mục theo quy tắc tùy chỉnh.",
            "使用自定义规则将文件整理到文件夹。",
            "カスタムルールでファイルをフォルダーに整理します。",
            "Сортировка файлов по папкам с помощью правил.",
        ],
        "Target Clean Folders:" => [
            "Managed folders:",
            "Thư mục đang quản lý:",
            "受管理的文件夹：",
            "管理対象フォルダー：",
            "Управляемые папки:",
        ],
        "+ Add Folder" => [
            "+ Add Folder",
            "+ Thêm thư mục",
            "+ 添加文件夹",
            "+ フォルダーを追加",
            "+ Добавить папку",
        ],
        "Path" => ["Path", "Đường dẫn", "路径", "パス", "Путь"],
        "Calculated Size" => [
            "Calculated Size",
            "Dung lượng",
            "已计算大小",
            "計算済みサイズ",
            "Размер",
        ],
        "Items" => ["Items", "Số mục", "项目", "項目数", "Элементы"],
        "Action" => ["Action", "Thao tác", "操作", "操作", "Действие"],
        "Scanning..." => [
            "Scanning...",
            "Đang quét...",
            "正在扫描...",
            "スキャン中...",
            "Сканирование...",
        ],
        "Check Size" => [
            "Check Size",
            "Tính dung lượng",
            "计算大小",
            "サイズを計算",
            "Рассчитать размер",
        ],
        "Remove" => ["Remove", "Gỡ", "移除", "削除", "Убрать"],
        "Sort All Folders Now" => [
            "Sort All Folders Now",
            "Sắp xếp tất cả ngay",
            "立即整理所有文件夹",
            "すべてのフォルダーを整理",
            "Сортировать все папки",
        ],
        "Stop Watcher" => [
            "Stop Watcher",
            "Dừng theo dõi",
            "停止监控",
            "監視を停止",
            "Остановить мониторинг",
        ],
        "Auto Watcher" => [
            "Auto Watcher",
            "Tự động theo dõi",
            "自动监控",
            "自動監視",
            "Автомониторинг",
        ],
        "Start with Windows" => [
            "Start with Windows",
            "Khởi động cùng Windows",
            "随 Windows 启动",
            "Windows と同時に起動",
            "Запускать вместе с Windows",
        ],
        "Automatic cleanup starts after login and may move matching files." => [
            "Automatic cleanup starts after login and may move matching files.",
            "Dọn dẹp tự động sẽ chạy sau khi đăng nhập và có thể di chuyển file khớp quy tắc.",
            "登录后将自动清理，可能移动符合规则的文件。",
            "ログイン後に自動整理が実行され、ルールに一致するファイルが移動される場合があります。",
            "После входа запустится автоматическая очистка и подходящие файлы могут быть перемещены.",
        ],
        "Rule Matching Priority:" => [
            "Rule Matching Priority:",
            "Ưu tiên khớp quy tắc:",
            "规则匹配优先级：",
            "ルール照合の優先順位：",
            "Приоритет правил:",
        ],
        "Filename Keywords First" => [
            "Filename Keywords First",
            "Ưu tiên từ khóa tên file",
            "优先匹配文件名关键词",
            "ファイル名キーワードを優先",
            "Сначала ключевые слова",
        ],
        "File Extensions First" => [
            "File Extensions First",
            "Ưu tiên phần mở rộng",
            "优先匹配扩展名",
            "拡張子を優先",
            "Сначала расширение",
        ],
        "Configure Categories, Extensions & Filename Keywords" => [
            "Configure Categories, Extensions & Filename Keywords",
            "Cấu hình nhóm, phần mở rộng và từ khóa",
            "配置类别、扩展名和关键词",
            "カテゴリ・拡張子・キーワードを設定",
            "Настройка категорий, расширений и слов",
        ],
        "Export Backup JSON" => [
            "Export Backup JSON",
            "Xuất bản sao JSON",
            "导出 JSON 备份",
            "JSON バックアップを保存",
            "Экспорт резервной копии JSON",
        ],
        "Load Backup JSON" => [
            "Load Backup JSON",
            "Nạp bản sao JSON",
            "导入 JSON 备份",
            "JSON バックアップを読み込み",
            "Загрузить резервную копию JSON",
        ],
        "Restore Factory Defaults" => [
            "Restore Factory Defaults",
            "Khôi phục mặc định",
            "恢复默认设置",
            "初期設定に戻す",
            "Восстановить настройки",
        ],
        "Add Rule" => [
            "Add Rule",
            "Thêm quy tắc",
            "添加规则",
            "ルールを追加",
            "Добавить правило",
        ],
        "Delete" => ["Delete", "Xóa", "删除", "削除", "Удалить"],
        "Disk space analyzer" => [
            "Disk space analyzer",
            "Phân tích dung lượng ổ đĩa",
            "磁盘空间分析",
            "ディスク容量分析",
            "Анализ места на диске",
        ],
        "Largest files, files not modified recently, and space by file type." => [
            "Largest files, files not modified recently, and space by file type.",
            "Tìm file lớn, file lâu chưa sửa đổi và xem dung lượng theo loại file.",
            "查找大文件、长期未修改的文件，并按类型查看空间占用。",
            "大容量・長期間未更新のファイルを検索し、種類別の容量を表示します。",
            "Поиск больших и давно не изменявшихся файлов, анализ места по типам.",
        ],
        "Scan folder:" => [
            "Scan folder:",
            "Thư mục quét:",
            "扫描文件夹：",
            "スキャンするフォルダー：",
            "Папка для сканирования:",
        ],
        "Choose…" => ["Choose…", "Chọn…", "选择…", "選択…", "Выбрать…"],
        "Minimum size:" => [
            "Minimum size:",
            "Dung lượng tối thiểu:",
            "最小大小：",
            "最小サイズ：",
            "Минимальный размер:",
        ],
        "Top results:" => [
            "Top results:",
            "Số kết quả:",
            "结果数量：",
            "表示件数：",
            "Количество результатов:",
        ],
        "Stale after:" => [
            "Stale after:",
            "Chưa sửa đổi sau:",
            "超过此时间未修改：",
            "未更新期間：",
            "Не изменялись:",
        ],
        "Scan selected folder" => [
            "Scan selected folder",
            "Quét thư mục đã chọn",
            "扫描所选文件夹",
            "選択したフォルダーをスキャン",
            "Сканировать папку",
        ],
        "Scanning files…" => [
            "Scanning files…",
            "Đang quét file…",
            "正在扫描文件…",
            "ファイルをスキャン中…",
            "Сканирование файлов…",
        ],
        "Storage by file type" => [
            "Storage by file type",
            "Dung lượng theo loại file",
            "按文件类型统计空间",
            "ファイル種類別の容量",
            "Место по типам файлов",
        ],
        "Largest files" => [
            "Largest files",
            "File lớn nhất",
            "最大文件",
            "大容量ファイル",
            "Самые большие файлы",
        ],
        "Not modified in" => [
            "Not modified in",
            "Chưa sửa đổi trong",
            "未修改时间：",
            "未更新期間：",
            "Не изменялись",
        ],
        "Exact duplicate finder" => [
            "Exact duplicate finder",
            "Tìm file trùng chính xác",
            "精确查找重复文件",
            "完全一致する重複ファイルを検索",
            "Поиск точных дубликатов",
        ],
        "Find exact duplicates" => [
            "Find exact duplicates",
            "Tìm file trùng chính xác",
            "查找完全重复文件",
            "完全一致するファイルを検索",
            "Найти точные дубликаты",
        ],
        "Developer build caches" => [
            "Developer build caches",
            "Cache build lập trình",
            "开发构建缓存",
            "開発ビルドキャッシュ",
            "Кэш сборки разработчика",
        ],
        "Scan project" => [
            "Scan project",
            "Quét dự án",
            "扫描项目",
            "プロジェクトをスキャン",
            "Сканировать проект",
        ],
        "Scheduled maintenance" => [
            "Scheduled maintenance",
            "Bảo trì theo lịch",
            "计划维护",
            "定期メンテナンス",
            "Обслуживание по расписанию",
        ],
        "Enable maintenance every 7 days" => [
            "Enable maintenance every 7 days",
            "Bật bảo trì mỗi 7 ngày",
            "每 7 天执行维护",
            "7 日ごとにメンテナンス",
            "Обслуживание каждые 7 дней",
        ],
        "Application and shader caches" => [
            "Application and shader caches",
            "Cache ứng dụng và shader",
            "应用与着色器缓存",
            "アプリとシェーダーのキャッシュ",
            "Кэш приложений и шейдеров",
        ],
        "Windows repair and maintenance" => [
            "Windows repair and maintenance",
            "Sửa chữa và bảo trì Windows",
            "Windows 修复与维护",
            "Windows の修復と保守",
            "Восстановление и обслуживание Windows",
        ],
        "Package manager caches" => [
            "Package manager caches",
            "Cache trình quản lý gói",
            "包管理器缓存",
            "パッケージマネージャーのキャッシュ",
            "Кэш менеджеров пакетов",
        ],
        "Utilities for storage health, cleanup, and automation." => [
            "Utilities for storage health, cleanup, and automation.",
            "Công cụ kiểm tra ổ đĩa, dọn dẹp và tự động hóa.",
            "存储检测、清理与自动化工具。",
            "ストレージ診断、クリーンアップ、自動化ツール。",
            "Инструменты диагностики диска, очистки и автоматизации.",
        ],
        "Disk optimizer" => [
            "Disk optimizer",
            "Tối ưu ổ đĩa",
            "磁盘优化",
            "ディスク最適化",
            "Оптимизация диска",
        ],
        "Files are grouped by size first, then verified with a full BLAKE3 content hash." => [
            "Files are grouped by size first, then verified with a full BLAKE3 content hash.",
            "File được nhóm theo kích thước rồi xác minh toàn bộ nội dung bằng mã băm BLAKE3.",
            "先按大小分组，再使用完整 BLAKE3 内容哈希验证文件。",
            "まずサイズで分類し、BLAKE3 の完全な内容ハッシュで検証します。",
            "Сначала файлы группируются по размеру, затем проверяются полным хэшем BLAKE3.",
        ],
        "Scanned {} files" => [
            "Scanned {} files",
            "Đã quét {} file",
            "已扫描 {} 个文件",
            "{} 件をスキャンしました",
            "Просканировано файлов: {}",
        ],
        "cache folder(s) found" => [
            "cache folder(s) found",
            "thư mục cache được tìm thấy",
            "个缓存文件夹",
            "個のキャッシュフォルダー",
            "папок кэша найдено",
        ],
        "Flush DNS cache…" => [
            "Flush DNS cache…",
            "Xóa cache DNS…",
            "清除 DNS 缓存…",
            "DNS キャッシュを消去…",
            "Очистить кэш DNS…",
        ],
        "Run SFC /scannow…" => [
            "Run SFC /scannow…",
            "Chạy SFC /scannow…",
            "运行 SFC /scannow…",
            "SFC /scannow を実行…",
            "Запустить SFC /scannow…",
        ],
        "Run DISM repair…" => [
            "Run DISM repair…",
            "Chạy sửa chữa DISM…",
            "运行 DISM 修复…",
            "DISM の修復を実行…",
            "Запустить восстановление DISM…",
        ],
        "Clean Windows Update downloads…" => [
            "Clean Windows Update downloads…",
            "Dọn file tải Windows Update…",
            "清理 Windows 更新下载…",
            "Windows Update のダウンロードを削除…",
            "Очистить загрузки Центра обновления…",
        ],
        "Clear cache…" => [
            "Clear cache…",
            "Dọn cache…",
            "清理缓存…",
            "キャッシュを削除…",
            "Очистить кэш…",
        ],
        "Clean npm cache…" => [
            "Clean npm cache…",
            "Dọn cache npm…",
            "清理 npm 缓存…",
            "npm キャッシュを削除…",
            "Очистить кэш npm…",
        ],
        "Clean Cargo cache…" => [
            "Clean Cargo cache…",
            "Dọn cache Cargo…",
            "清理 Cargo 缓存…",
            "Cargo キャッシュを削除…",
            "Очистить кэш Cargo…",
        ],
        "Clean NuGet cache…" => [
            "Clean NuGet cache…",
            "Dọn cache NuGet…",
            "清理 NuGet 缓存…",
            "NuGet キャッシュを削除…",
            "Очистить кэш NuGet…",
        ],
        "Run benchmark" => [
            "Run benchmark",
            "Chạy đo tốc độ",
            "运行测速",
            "ベンチマークを実行",
            "Запустить тест скорости",
        ],
        "Sequential read / write benchmark" => [
            "Sequential read / write benchmark",
            "Đo tốc độ đọc / ghi tuần tự",
            "顺序读写测速",
            "シーケンシャル読み書きベンチマーク",
            "Тест последовательного чтения и записи",
        ],
        "Choose folder…" => [
            "Choose folder…",
            "Chọn thư mục…",
            "选择文件夹…",
            "フォルダーを選択…",
            "Выбрать папку…",
        ],
        "Test location:" => [
            "Test location:",
            "Vị trí kiểm tra:",
            "测试位置：",
            "テスト場所：",
            "Папка теста:",
        ],
        "Targeted System Sanitization & Junk Purge" => [
            "Targeted System Sanitization & Junk Purge",
            "Dọn dẹp hệ thống có chọn lọc",
            "定向系统清理",
            "対象を指定したシステムクリーンアップ",
            "Выборочная очистка системы",
        ],
        "Find Rust, Node.js, Python and Visual Studio build/cache folders in a project tree." => [
            "Find Rust, Node.js, Python and Visual Studio build/cache folders in a project tree.",
            "Tìm thư mục build/cache Rust, Node.js, Python và Visual Studio trong cây dự án.",
            "在项目目录中查找 Rust、Node.js、Python 和 Visual Studio 构建/缓存文件夹。",
            "プロジェクト内の Rust、Node.js、Python、Visual Studio のビルド/キャッシュフォルダーを検索します。",
            "Поиск папок сборки и кэша Rust, Node.js, Python и Visual Studio в проекте.",
        ],
        "Only named cache folders are cleared. Browser cookies and history are never selected." => [
            "Only named cache folders are cleared. Browser cookies and history are never selected.",
            "Chỉ dọn đúng thư mục cache được liệt kê; không chọn cookie hay lịch sử trình duyệt.",
            "仅清理列出的缓存文件夹；不会选择浏览器 Cookie 或历史记录。",
            "一覧にあるキャッシュのみを削除します。Cookie や閲覧履歴は対象外です。",
            "Очищаются только указанные папки кэша. Cookies и история браузера не затрагиваются.",
        ],
        "DNS flush and Windows repair commands always request administrator approval through UAC." => [
            "DNS flush and Windows repair commands always request administrator approval through UAC.",
            "Xóa DNS và các lệnh sửa Windows luôn yêu cầu chấp thuận quyền quản trị qua UAC.",
            "刷新 DNS 和 Windows 修复命令始终会通过 UAC 请求管理员批准。",
            "DNS の消去と Windows 修復コマンドは、常に UAC で管理者の承認を求めます。",
            "Очистка DNS и команды восстановления Windows всегда запрашивают разрешение администратора через UAC.",
        ],
        "Commands run only after confirmation. If the package manager is unavailable, the error is reported in the activity log." => [
            "Commands run only after confirmation. If the package manager is unavailable, the error is reported in the activity log.",
            "Lệnh chỉ chạy sau khi xác nhận. Nếu thiếu công cụ quản lý gói, lỗi sẽ hiện trong nhật ký.",
            "命令仅在确认后运行。如果未安装包管理器，错误会记录在活动日志中。",
            "確認後にのみ実行します。パッケージマネージャーがない場合はログにエラーを表示します。",
            "Команды выполняются только после подтверждения. Если менеджер пакетов недоступен, ошибка появится в журнале.",
        ],
        "Recognized: target/, node_modules/, .venv/, venv/, __pycache__/, bin/, obj/, .vs/. Review paths before quarantining; builds may need to be regenerated." => [
            "Recognized: target/, node_modules/, .venv/, venv/, __pycache__/, bin/, obj/, .vs/. Review paths before quarantining; builds may need to be regenerated.",
            "Nhận diện: target/, node_modules/, .venv/, venv/, __pycache__/, bin/, obj/, .vs/. Kiểm tra đường dẫn trước khi cách ly; có thể cần build lại.",
            "识别目录：target/、node_modules/、.venv/、venv/、__pycache__/、bin/、obj/、.vs/。隔离前请检查路径；之后可能需要重新构建。",
            "対象: target/、node_modules/、.venv/、venv/、__pycache__/、bin/、obj/、.vs/。隔離前にパスを確認してください。再ビルドが必要になる場合があります。",
            "Распознаются: target/, node_modules/, .venv/, venv/, __pycache__/, bin/, obj/, .vs/. Проверьте пути перед карантином; может потребоваться пересборка.",
        ],
        "No supported browser, application, or shader cache folders were found." => [
            "No supported browser, application, or shader cache folders were found.",
            "Không tìm thấy thư mục cache trình duyệt, ứng dụng hoặc shader được hỗ trợ.",
            "未找到支持的浏览器、应用或着色器缓存文件夹。",
            "対応するブラウザー、アプリ、シェーダーのキャッシュが見つかりません。",
            "Поддерживаемые папки кэша браузеров, приложений или шейдеров не найдены.",
        ],
        "Runs user Temp cleanup and SSD TRIM while CleanDisk is running (including hidden in the tray). This in-app schedule is not active after you exit the app." => [
            "Runs user Temp cleanup and SSD TRIM while CleanDisk is running (including hidden in the tray). This in-app schedule is not active after you exit the app.",
            "Khi CleanDisk đang chạy (kể cả ẩn dưới khay), ứng dụng sẽ dọn Temp người dùng và TRIM SSD. Lịch không hoạt động sau khi thoát ứng dụng.",
            "CleanDisk 运行时（包括最小化到托盘）清理用户临时文件并执行 SSD TRIM。退出应用后计划不会运行。",
            "CleanDisk の実行中（トレイに格納中も含む）にユーザー Temp の削除と SSD TRIM を行います。アプリ終了後は実行されません。",
            "Пока CleanDisk запущен (в том числе в трее), очищает Temp пользователя и выполняет TRIM SSD. После выхода расписание не действует.",
        ],
        "No recognized, non-empty build cache folders found." => [
            "No recognized, non-empty build cache folders found.",
            "Không tìm thấy thư mục cache build phù hợp có dữ liệu.",
            "未找到符合条件且非空的构建缓存文件夹。",
            "対象となる空でないビルドキャッシュが見つかりません。",
            "Подходящие непустые папки кэша сборки не найдены.",
        ],
        "No files match this size threshold." => [
            "No files match this size threshold.",
            "Không có file nào đạt ngưỡng dung lượng này.",
            "没有文件达到此大小阈值。",
            "このサイズ条件に一致するファイルはありません。",
            "Файлы, соответствующие этому порогу, не найдены.",
        ],
        "No stale files found in this folder." => [
            "No stale files found in this folder.",
            "Không tìm thấy file cũ trong thư mục này.",
            "此文件夹中没有找到长期未修改的文件。",
            "このフォルダーに古いファイルはありません。",
            "В этой папке не найдены давно не изменявшиеся файлы.",
        ],
        "No identical files found. Start a scan to check a folder." => [
            "No identical files found. Start a scan to check a folder.",
            "Chưa tìm thấy file trùng. Hãy quét một thư mục để kiểm tra.",
            "未找到重复文件。请扫描文件夹进行检查。",
            "重複ファイルは見つかりません。フォルダーをスキャンしてください。",
            "Дубликаты не найдены. Запустите сканирование папки.",
        ],
        "Top 50" => ["Top 50", "Top 50", "前 50 个", "上位 50 件", "Топ-50"],
        "Top 100" => ["Top 100", "Top 100", "前 100 个", "上位 100 件", "Топ-100"],
        "30 days" => ["30 days", "30 ngày", "30 天", "30 日", "30 дней"],
        "60 days" => ["60 days", "60 ngày", "60 天", "60 日", "60 дней"],
        "90 days" => ["90 days", "90 ngày", "90 天", "90 日", "90 дней"],
        "500 MB" => ["500 MB", "500 MB", "500 MB", "500 MB", "500 МБ"],
        "1 GB" => ["1 GB", "1 GB", "1 GB", "1 GB", "1 ГБ"],
        "5 GB" => ["5 GB", "5 GB", "5 GB", "5 GB", "5 ГБ"],
        "Clean User Temp" => [
            "Clean User Temp",
            "Dọn Temp người dùng",
            "清理用户临时文件",
            "ユーザー一時ファイルを削除",
            "Очистить временные файлы пользователя",
        ],
        "Clean System Temp" => [
            "Clean System Temp",
            "Dọn Temp hệ thống",
            "清理系统临时文件",
            "システム一時ファイルを削除",
            "Очистить системные временные файлы",
        ],
        "Clean Prefetch" => [
            "Clean Prefetch",
            "Dọn Prefetch",
            "清理预读取缓存",
            "プリフェッチを削除",
            "Очистить Prefetch",
        ],
        "Clean Crash Dumps" => [
            "Clean Crash Dumps",
            "Dọn file crash dump",
            "清理崩溃转储",
            "クラッシュダンプを削除",
            "Очистить дампы сбоев",
        ],
        "Empty Recycle Bin" => [
            "Empty Recycle Bin",
            "Làm trống Thùng rác",
            "清空回收站",
            "ごみ箱を空にする",
            "Очистить корзину",
        ],
        "Clean All Junk Stores" => [
            "Clean All Junk Stores",
            "Dọn tất cả vùng rác",
            "清理所有临时目录",
            "すべての一時領域を削除",
            "Очистить все временные папки",
        ],
        "Project root:" => [
            "Project root:",
            "Thư mục dự án:",
            "项目目录：",
            "プロジェクトルート：",
            "Папка проекта:",
        ],
        "Folder:" => ["Folder:", "Thư mục:", "文件夹：", "フォルダー：", "Папка:"],
        "Keep:" => ["Keep:", "Giữ lại:", "保留：", "保持：", "Оставить:"],
        "Newest copy" => [
            "Newest copy",
            "Bản mới nhất",
            "最新副本",
            "最新のコピー",
            "Самую новую копию",
        ],
        "Oldest copy" => [
            "Oldest copy",
            "Bản cũ nhất",
            "最旧副本",
            "最も古いコピー",
            "Самую старую копию",
        ],
        "Not modified in {}+ days ({})" => [
            "Not modified in {}+ days ({})",
            "Chưa sửa đổi trong {}+ ngày ({})",
            "超过 {} 天未修改（{}）",
            "{} 日以上未更新（{}）",
            "Не изменялись {}+ дн. ({})",
        ],
        "Scanned {} files · {} total" => [
            "Scanned {} files · {} total",
            "Đã quét {} file · tổng {}",
            "已扫描 {} 个文件 · 共 {}",
            "{} 件をスキャン · 合計 {}",
            "Просканировано файлов: {} · всего {}",
        ],
        "{} duplicate groups · up to {} reclaimable" => [
            "{} duplicate groups · up to {} reclaimable",
            "{} nhóm trùng · có thể thu hồi {}",
            "{} 组重复文件 · 最多可释放 {}",
            "{} グループの重複 · 最大 {} を解放可能",
            "Групп дубликатов: {} · можно освободить до {}",
        ],
        "LIVE ACTIVITY CONSOLE" => [
            "LIVE ACTIVITY CONSOLE",
            "NHẬT KÝ HOẠT ĐỘNG",
            "实时活动日志",
            "操作ログ",
            "ЖУРНАЛ СОБЫТИЙ",
        ],
        "Interface language" => [
            "Interface language",
            "Ngôn ngữ giao diện",
            "界面语言",
            "表示言語",
            "Язык интерфейса",
        ],
        "How it works" => [
            "How it works",
            "Cách hoạt động",
            "工作原理",
            "動作の仕組み",
            "Как это работает",
        ],
        "Caution" => ["Caution", "Lưu ý", "注意", "注意事項", "Внимание"],
        "Select the language used for the application interface." => [
            "Select the language used for the application interface.",
            "Chọn ngôn ngữ hiển thị trong giao diện ứng dụng.",
            "选择应用界面使用的语言。",
            "アプリの表示に使用する言語を選択します。",
            "Выберите язык интерфейса приложения.",
        ],
        "Confirm action" => [
            "Confirm action",
            "Xác nhận thao tác",
            "确认操作",
            "操作の確認",
            "Подтвердите действие",
        ],
        "Move to quarantine" => [
            "Move to quarantine",
            "Chuyển vào khu cách ly",
            "移至隔离区",
            "隔離領域に移動",
            "Переместить в карантин",
        ],
        "Delete permanently" => [
            "Delete permanently",
            "Xóa vĩnh viễn",
            "永久删除",
            "完全に削除",
            "Удалить безвозвратно",
        ],
        "Run confirmed action" => [
            "Run confirmed action",
            "Chạy thao tác đã xác nhận",
            "执行已确认的操作",
            "確認した操作を実行",
            "Выполнить подтвержденное действие",
        ],
        "Cancel" => ["Cancel", "Hủy", "取消", "キャンセル", "Отмена"],
        "Selected items will be moved to quarantine and can be restored manually. Nothing is permanently deleted." => [
            "Selected items will be moved to quarantine and can be restored manually. Nothing is permanently deleted.",
            "Các mục đã chọn sẽ được chuyển vào khu cách ly để có thể khôi phục thủ công. Không có mục nào bị xóa vĩnh viễn.",
            "所选项目将移至隔离区，可手动恢复。不会永久删除任何项目。",
            "選択した項目は隔離領域に移動され、手動で復元できます。完全に削除される項目はありません。",
            "Выбранные элементы будут перемещены в карантин и могут быть восстановлены вручную. Ничего не удаляется безвозвратно.",
        ],
        "This permanently deletes the listed files and folders; they cannot be recovered from the Recycle Bin." => [
            "This permanently deletes the listed files and folders; they cannot be recovered from the Recycle Bin.",
            "Thao tác sẽ xóa vĩnh viễn file và thư mục được liệt kê; không thể khôi phục từ Thùng rác.",
            "此操作将永久删除列出的文件和文件夹，无法从回收站恢复。",
            "一覧のファイルとフォルダーを完全に削除します。ごみ箱から復元できません。",
            "Указанные файлы и папки будут удалены безвозвратно; восстановить их из корзины нельзя.",
        ],
        "Windows will show a UAC prompt. After approval, an elevated PowerShell window may open while this operation runs." => [
            "Windows will show a UAC prompt. After approval, an elevated PowerShell window may open while this operation runs.",
            "Windows sẽ hiện lời nhắc UAC. Sau khi chấp thuận, một cửa sổ PowerShell có quyền quản trị có thể mở trong lúc thao tác chạy.",
            "Windows 将显示 UAC 提示。批准后，操作运行期间可能会打开一个提升权限的 PowerShell 窗口。",
            "Windows の UAC が表示されます。承認すると、処理中に管理者権限の PowerShell ウィンドウが開く場合があります。",
            "Windows покажет запрос UAC. После подтверждения во время операции может открыться окно PowerShell с правами администратора.",
        ],
        "Organize tab tooltip" => [
            "Organize files into folders using rules and monitor new downloads.",
            "Sắp xếp file theo quy tắc và theo dõi file tải xuống mới.",
            "按规则整理文件并监控新下载内容。",
            "ルールに従ってファイルを整理し、新しいダウンロードを監視します。",
            "Сортировка файлов по правилам и мониторинг новых загрузок.",
        ],
        "Windows startup tooltip" => [
            "Registers this app for the current Windows user. It starts hidden with Auto Watcher after login; verify folders and rules before enabling. Disable this option to remove the registration.",
            "Đăng ký ứng dụng cho người dùng Windows hiện tại. App khởi chạy ẩn cùng Tự động theo dõi sau khi đăng nhập; hãy kiểm tra thư mục và quy tắc trước khi bật. Bỏ chọn để xóa đăng ký.",
            "为当前 Windows 用户注册应用。登录后应用将隐藏启动并启用自动监控；启用前请检查文件夹和规则。取消勾选即可移除注册。",
            "現在の Windows ユーザーに登録します。ログイン後、非表示で起動して自動監視を有効にします。設定前にフォルダーとルールを確認してください。オフにすると登録を解除します。",
            "Регистрирует приложение для текущего пользователя Windows. После входа оно запускается скрытым и включает мониторинг; перед включением проверьте папки и правила. Снимите флажок для удаления регистрации.",
        ],
        "Disk optimizer tab tooltip" => [
            "View disks, run SSD TRIM, analyze or defragment HDDs, and benchmark sequential speed.",
            "Xem ổ đĩa, chạy TRIM cho SSD, phân tích/chống phân mảnh HDD và đo tốc độ tuần tự.",
            "查看磁盘、执行 SSD TRIM、分析或整理 HDD，并测试顺序读写速度。",
            "ディスクの表示、SSD TRIM、HDD の分析・デフラグ、シーケンシャル速度測定を行います。",
            "Просмотр дисков, TRIM SSD, анализ и дефрагментация HDD, тест скорости.",
        ],
        "System cleanup tab tooltip" => [
            "Clean selected temporary folders after confirmation; locked files may be skipped.",
            "Dọn thư mục tạm đã chọn sau khi xác nhận; file đang khóa có thể bị bỏ qua.",
            "确认后清理所选临时文件夹；锁定文件可能会跳过。",
            "確認後に一時フォルダーを削除します。ロック中のファイルはスキップされます。",
            "Очистка выбранных временных папок после подтверждения; заблокированные файлы пропускаются.",
        ],
        "Space analysis tab tooltip" => [
            "Find large and stale files and view storage by type. Scanning never deletes files.",
            "Tìm file lớn/lâu chưa sửa đổi và xem dung lượng theo loại. Quét không xóa file.",
            "查找大文件和长期未修改文件，并按类型查看空间。扫描不会删除文件。",
            "大容量・古いファイルの検索と種類別容量の表示。スキャンでファイルは削除されません。",
            "Поиск больших и старых файлов, анализ места по типам. Сканирование ничего не удаляет.",
        ],
        "Duplicates tab tooltip" => [
            "Compare sizes and BLAKE3 hashes to find duplicates; select a copy to keep and quarantine extras.",
            "So kích thước và BLAKE3 để tìm bản trùng; chọn bản giữ lại rồi cách ly bản dư.",
            "通过大小和 BLAKE3 哈希查找重复文件；选择保留副本并隔离多余文件。",
            "サイズと BLAKE3 で重複を検索し、残すファイルを選択して他を隔離します。",
            "Поиск дубликатов по размеру и BLAKE3; выберите оригинал, остальные перемещаются в карантин.",
        ],
        "Dev caches tab tooltip" => [
            "Find development build caches and move selected folders to a reversible quarantine.",
            "Tìm cache build và chuyển thư mục đã chọn vào vùng cách ly có thể khôi phục.",
            "查找开发构建缓存，并将所选文件夹移至可恢复的隔离区。",
            "開発ビルドキャッシュを検索し、選択したフォルダーを復元可能な隔離領域に移動します。",
            "Поиск кэша сборки и перемещение выбранных папок в восстанавливаемый карантин.",
        ],
        "Windows tools tab tooltip" => [
            "Clean application caches or run Windows repair commands; each action requires confirmation.",
            "Dọn cache ứng dụng hoặc chạy lệnh sửa Windows; mỗi thao tác cần xác nhận.",
            "清理应用缓存或运行 Windows 修复命令；每项操作均需确认。",
            "アプリキャッシュの削除や Windows 修復を実行します。各操作には確認が必要です。",
            "Очистка кэша приложений и команды восстановления Windows; требуется подтверждение.",
        ],
        "Refresh disk information from Windows." => [
            "Refresh disk information from Windows.",
            "Làm mới thông tin ổ đĩa từ Windows.",
            "从 Windows 刷新磁盘信息。",
            "Windows からディスク情報を更新します。",
            "Обновить сведения о дисках из Windows.",
        ],
        "Choose a folder to create a temporary benchmark file. The app checks available space first." => [
            "Choose a folder to create a temporary benchmark file. The app checks available space first.",
            "Chọn thư mục tạo file đo tạm; ứng dụng sẽ kiểm tra dung lượng trống trước.",
            "选择临时测速文件的保存位置；应用会先检查可用空间。",
            "一時ベンチマークファイルを作成するフォルダーを選択します。先に空き容量を確認します。",
            "Выберите папку для временного файла теста. Сначала приложение проверит свободное место.",
        ],
        _ => [key, key, key, key, key],
    };
    let index = match language {
        Language::English => 0,
        Language::Vietnamese => 1,
        Language::Chinese => 2,
        Language::Japanese => 3,
        Language::Russian => 4,
    };
    translated[index]
}

pub fn tooltip_details(
    language: Language,
    feature: &str,
) -> (&'static str, &'static str, &'static str) {
    let kind = if feature.contains("dung lượng") || feature.contains("Phân tích dung lượng")
    {
        0
    } else if feature.contains("TRIM") || feature.contains("HDD") {
        1
    } else if feature.contains("tốc độ") {
        2
    } else if feature.contains("cách ly")
        || feature.contains("bản sao")
        || feature.contains("file cũ")
        || feature.contains("trùng")
    {
        3
    } else if feature.contains("phát triển") {
        4
    } else if feature.contains("cache") {
        5
    } else if feature.contains("DNS")
        || feature.contains("hệ thống")
        || feature.contains("Windows component")
        || feature.contains("Windows Update")
    {
        6
    } else if feature.contains("package") {
        7
    } else if feature.contains("Dọn") || feature.contains("Làm trống") {
        8
    } else {
        9
    };
    let index = language_index(language);

    let titles = [
        [
            "Read-only analysis",
            "Phân tích chỉ đọc",
            "只读分析",
            "読み取り専用分析",
            "Анализ без изменений",
        ],
        [
            "Windows drive maintenance",
            "Bảo trì ổ đĩa Windows",
            "Windows 磁盘维护",
            "Windows ディスク保守",
            "Обслуживание диска Windows",
        ],
        [
            "Sequential disk benchmark",
            "Đo tuần tự ổ đĩa",
            "磁盘顺序测速",
            "シーケンシャル速度測定",
            "Последовательный тест диска",
        ],
        [
            "Reversible quarantine",
            "Khu cách ly có thể phục hồi",
            "可恢复隔离区",
            "復元可能な隔離",
            "Восстанавливаемый карантин",
        ],
        [
            "Development cache scan",
            "Quét cache phát triển",
            "开发缓存扫描",
            "開発キャッシュのスキャン",
            "Поиск кэша разработки",
        ],
        [
            "Application cache cleanup",
            "Dọn cache ứng dụng",
            "清理应用缓存",
            "アプリキャッシュの削除",
            "Очистка кэша приложений",
        ],
        [
            "Windows system operation",
            "Thao tác hệ thống Windows",
            "Windows 系统操作",
            "Windows システム操作",
            "Операция Windows",
        ],
        [
            "Package cache cleanup",
            "Dọn cache gói",
            "清理软件包缓存",
            "パッケージキャッシュの削除",
            "Очистка кэша пакетов",
        ],
        [
            "Temporary-file cleanup",
            "Dọn file tạm",
            "清理临时文件",
            "一時ファイルの削除",
            "Очистка временных файлов",
        ],
        [
            "File organization",
            "Sắp xếp file",
            "文件整理",
            "ファイル整理",
            "Сортировка файлов",
        ],
    ];
    let descriptions = [
        [
            "Scans file metadata in the selected folder to calculate sizes or identify matching files. It does not modify files.",
            "Đọc metadata trong thư mục đã chọn để tính dung lượng hoặc tìm file phù hợp; không sửa file.",
            "读取所选文件夹中的元数据以统计空间或查找匹配文件，不会修改文件。",
            "選択したフォルダーのメタデータを読み取り、容量を計算または対象ファイルを検索します。ファイルは変更しません。",
            "Читает метаданные выбранной папки для подсчета размера или поиска файлов. Файлы не изменяются.",
        ],
        [
            "Runs the selected Windows Analyze, ReTrim, or Defragment operation on the chosen drive.",
            "Chạy thao tác Analyze, ReTrim hoặc Defragment của Windows trên ổ đĩa đã chọn.",
            "在所选磁盘上运行 Windows Analyze、ReTrim 或 Defragment 操作。",
            "選択したドライブで Windows の Analyze、ReTrim、または Defragment を実行します。",
            "Запускает выбранную операцию Windows Analyze, ReTrim или Defragment на указанном диске.",
        ],
        [
            "Writes a temporary file of the chosen size, reads it back, measures throughput, then removes it.",
            "Ghi file tạm theo kích thước đã chọn, đọc lại để đo tốc độ rồi xóa file đó.",
            "写入指定大小的临时文件，再读回测量速度，最后删除临时文件。",
            "指定サイズの一時ファイルを書き込み、読み戻して速度を測定し、最後に削除します。",
            "Создает временный файл заданного размера, считывает его для измерения скорости и затем удаляет.",
        ],
        [
            "Moves only the selected files or folders into the quarantine folder while preserving their relative paths.",
            "Chỉ chuyển file hoặc thư mục đã chọn vào khu cách ly và giữ cấu trúc đường dẫn để phục hồi.",
            "仅将所选文件或文件夹移入隔离区，并保留相对路径以便恢复。",
            "選択したファイルやフォルダーのみを隔離領域へ移動し、復元できるよう相対パスを保持します。",
            "Перемещает только выбранные файлы или папки в карантин, сохраняя относительные пути для восстановления.",
        ],
        [
            "Searches the selected project tree for recognized build-cache folders and totals their contents.",
            "Tìm thư mục cache build đã nhận diện trong cây dự án đã chọn và tính dung lượng.",
            "在所选项目目录中查找已识别的构建缓存文件夹并统计其大小。",
            "選択したプロジェクト内で認識済みのビルドキャッシュを検索し、容量を集計します。",
            "Ищет известные папки кэша сборки в выбранном проекте и подсчитывает их размер.",
        ],
        [
            "Clears only the listed application or shader-cache folder. Browser cookies and history are outside the selected cache paths.",
            "Chỉ làm trống thư mục cache ứng dụng hoặc shader đang liệt kê; không chọn cookie hay lịch sử trình duyệt.",
            "仅清空列出的应用或着色器缓存目录；浏览器 Cookie 和历史记录不在所选路径中。",
            "一覧にあるアプリまたはシェーダーキャッシュのみを削除します。Cookie と閲覧履歴は対象外です。",
            "Очищает только указанный кэш приложения или шейдеров. Cookies и история браузера не затрагиваются.",
        ],
        [
            "Runs the named Windows maintenance command in an elevated process after you approve the UAC prompt.",
            "Chạy lệnh bảo trì Windows tương ứng bằng tiến trình nâng quyền sau khi bạn chấp thuận UAC.",
            "在你通过 UAC 批准后，以提升权限运行相应的 Windows 维护命令。",
            "UAC で承認すると、管理者権限で該当する Windows メンテナンスコマンドを実行します。",
            "После подтверждения UAC запускает соответствующую команду обслуживания Windows с повышенными правами.",
        ],
        [
            "Runs the selected package manager's cache-clean command; packages may need to be downloaded again.",
            "Chạy lệnh dọn cache của trình quản lý gói đã chọn; gói có thể cần tải lại.",
            "运行所选包管理器的缓存清理命令；之后可能需要重新下载软件包。",
            "選択したパッケージマネージャーのキャッシュ削除コマンドを実行します。後で再ダウンロードが必要な場合があります。",
            "Запускает очистку кэша выбранного менеджера пакетов; пакеты может потребоваться загрузить повторно.",
        ],
        [
            "Deletes the contents of the selected temporary folder. Locked or inaccessible entries may remain.",
            "Xóa nội dung trong thư mục tạm đã chọn; mục đang khóa hoặc không truy cập được có thể còn lại.",
            "删除所选临时文件夹中的内容；被锁定或无法访问的项目可能保留。",
            "選択した一時フォルダーの内容を削除します。ロック中またはアクセスできない項目は残る場合があります。",
            "Удаляет содержимое выбранной временной папки. Заблокированные или недоступные элементы могут остаться.",
        ],
        [
            "Applies the configured rules to files in the selected folders; matching files can be moved to category folders.",
            "Áp dụng quy tắc cho file trong thư mục đã chọn; file khớp có thể được chuyển sang thư mục phân loại.",
            "将配置的规则应用于所选文件夹中的文件；匹配的文件可能会移至分类文件夹。",
            "選択したフォルダー内のファイルにルールを適用します。一致したファイルは分類先へ移動されます。",
            "Применяет правила к файлам в выбранных папках; совпавшие файлы могут быть перемещены в папки категорий.",
        ],
    ];
    let cautions = [
        ["Read-only: this action does not delete or move files.", "Chỉ đọc: thao tác này không xóa hoặc di chuyển file.", "只读：此操作不会删除或移动文件。", "読み取り専用です。ファイルの削除や移動は行いません。", "Только чтение: файлы не удаляются и не перемещаются."],
        ["Administrator approval is requested by Windows. Confirm the drive and operation before approving.", "Windows sẽ yêu cầu chấp thuận quyền quản trị. Kiểm tra ổ đĩa và thao tác trước khi đồng ý.", "Windows 将请求管理员批准。批准前请确认磁盘和操作。", "Windows が管理者の承認を求めます。承認前にドライブと操作を確認してください。", "Windows запросит права администратора. Перед подтверждением проверьте диск и операцию."],
        ["This writes real data to the selected drive and requires free space; results may be affected by file caching.", "Thao tác ghi dữ liệu thật lên ổ đĩa và cần dung lượng trống; cache có thể ảnh hưởng kết quả.", "此操作会向磁盘写入实际数据并需要可用空间；文件缓存可能影响结果。", "実際にドライブへ書き込み、空き容量を使用します。ファイルキャッシュが結果に影響する場合があります。", "Тест записывает данные на диск и требует свободного места; кэш файлов может повлиять на результат."],
        ["Review each selected path. Quarantined data remains recoverable; no permanent deletion occurs here.", "Kiểm tra từng đường dẫn đã chọn. Dữ liệu cách ly có thể phục hồi và không bị xóa vĩnh viễn.", "请检查所选路径。隔离的数据可恢复，此操作不会永久删除。", "選択したパスを確認してください。隔離データは復元可能で、完全には削除されません。", "Проверьте выбранные пути. Данные в карантине можно восстановить; безвозвратного удаления нет."],
        ["Scanning is read-only. Review detected paths before choosing any cleanup action.", "Quét chỉ đọc. Kiểm tra đường dẫn tìm được trước khi chọn thao tác dọn.", "扫描为只读。执行清理前请检查找到的路径。", "スキャンは読み取り専用です。削除操作の前に検出されたパスを確認してください。", "Сканирование не изменяет данные. Проверьте найденные пути перед очисткой."],
        ["Close the related app first. The cache will be rebuilt and may load or compile again on next use.", "Đóng ứng dụng liên quan trước. Cache sẽ được tạo lại và có thể tải hoặc biên dịch lại khi dùng.", "请先关闭相关应用。下次使用时缓存可能需要重新下载或生成。", "先に関連アプリを終了してください。次回利用時にキャッシュが再生成される場合があります。", "Сначала закройте приложение. При следующем запуске кэш может быть создан или загружен заново."],
        ["Administrator approval is required. Canceling the UAC prompt cancels this operation.", "Bắt buộc chấp thuận quyền quản trị. Hủy lời nhắc UAC đồng nghĩa hủy thao tác.", "此操作需要管理员批准。取消 UAC 提示将取消操作。", "管理者の承認が必要です。UAC をキャンセルすると操作もキャンセルされます。", "Требуется разрешение администратора. Отмена запроса UAC отменит операцию."],
        ["Review the command and package manager first; cached packages may need to be fetched again.", "Kiểm tra lệnh và trình quản lý gói trước; gói đã cache có thể phải tải lại.", "请先确认命令和包管理器；缓存的软件包之后可能需要重新下载。", "コマンドとパッケージマネージャーを確認してください。キャッシュ済みパッケージは再取得が必要な場合があります。", "Проверьте команду и менеджер пакетов; кэшированные пакеты может потребоваться загрузить снова."],
        ["This permanently deletes the selected contents after confirmation; it does not use the Recycle Bin.", "Sau khi xác nhận, nội dung đã chọn bị xóa vĩnh viễn, không qua Thùng rác.", "确认后将永久删除所选内容，不会移入回收站。", "確認後、選択した内容はごみ箱を経由せず完全に削除されます。", "После подтверждения выбранные данные будут удалены безвозвратно, минуя корзину."],
        ["Files may be moved without an undo operation. Verify rules and destination paths first.", "File có thể bị di chuyển mà không có hoàn tác. Hãy kiểm tra quy tắc và thư mục đích.", "文件可能会被移动且无法自动撤销。请先检查规则和目标路径。", "ファイル移動は自動で元に戻せません。ルールと移動先を確認してください。", "Перемещение файлов нельзя автоматически отменить. Проверьте правила и папки назначения."],
    ];

    (
        titles[kind][index],
        descriptions[kind][index],
        cautions[kind][index],
    )
}

pub fn tooltip_hint(language: Language, source: &'static str) -> &'static str {
    if language == Language::Vietnamese {
        return source;
    }
    let normalized = source.to_lowercase();
    let kind = if normalized.contains("json")
        && (normalized.contains("lưu") || normalized.contains("xuất"))
    {
        0
    } else if normalized.contains("json") || normalized.contains("backup") {
        1
    } else if normalized.contains("mặc định") {
        2
    } else if normalized.contains("quy tắc") && normalized.contains("thêm") {
        3
    } else if normalized.contains("quy tắc") || normalized.contains("thư mục khỏi") {
        4
    } else if normalized.contains("trùng")
        || normalized.contains("bản sao")
        || normalized.contains("cách ly")
    {
        5
    } else if normalized.contains("smart") {
        6
    } else if normalized.contains("ghi và đọc") {
        9
    } else if normalized.contains("thư mục gốc")
        || normalized.contains("thư mục") && normalized.contains("quét")
    {
        10
    } else if normalized.contains("thời gian sửa đổi")
        || normalized.contains("dung lượng từ")
        || normalized.contains("kết quả lớn nhất")
        || normalized.contains("hiển thị tối đa")
        || normalized.contains("file lớn nhất")
    {
        8
    } else {
        7
    };
    let index = language_index(language);
    let hints = [
        [
            "Exports only the current organization rules as JSON; no file list or personal data is included.",
            "Chỉ xuất quy tắc sắp xếp hiện tại thành JSON; không xuất danh sách file hoặc dữ liệu cá nhân.",
            "仅将当前整理规则导出为 JSON；不包含文件列表或个人数据。",
            "現在の整理ルールのみを JSON にエクスポートします。ファイル一覧や個人データは含まれません。",
            "Экспортируются только текущие правила сортировки в JSON; список файлов и личные данные не включаются.",
        ],
        [
            "Replaces the current rules with the selected backup. Choose a trusted JSON file.",
            "Thay quy tắc hiện tại bằng bản sao lưu đã chọn. Chỉ chọn file JSON đáng tin cậy.",
            "用所选备份替换当前规则。请仅选择可信的 JSON 文件。",
            "現在のルールを選択したバックアップに置き換えます。信頼できる JSON ファイルを選択してください。",
            "Текущие правила заменяются выбранной резервной копией. Выбирайте только доверенный JSON-файл.",
        ],
        [
            "Replaces this session's custom rules with defaults. Export a backup first if needed.",
            "Thay quy tắc tùy chỉnh trong phiên bằng mặc định. Hãy xuất bản sao lưu trước nếu cần.",
            "将本次会话的自定义规则恢复为默认值。如有需要，请先导出备份。",
            "このセッションのカスタムルールを既定値に戻します。必要に応じて先にバックアップしてください。",
            "Заменяет пользовательские правила сеанса значениями по умолчанию. При необходимости сначала сохраните резервную копию.",
        ],
        [
            "Adds a folder-name, extension, and keyword rule. Verify it before sorting because files can be moved.",
            "Thêm quy tắc theo tên thư mục, phần mở rộng và từ khóa. Kiểm tra trước khi sắp xếp vì file có thể bị di chuyển.",
            "添加按文件夹名称、扩展名和关键词匹配的规则。整理前请检查，因为文件可能会被移动。",
            "フォルダー名、拡張子、キーワードのルールを追加します。ファイルが移動されるため、整理前に確認してください。",
            "Добавляет правило по имени папки, расширению и ключевым словам. Перед сортировкой проверьте его: файлы могут быть перемещены.",
        ],
        [
            "Removes this rule from the current configuration; files already organized are not changed.",
            "Gỡ quy tắc khỏi cấu hình hiện tại; không thay đổi các file đã sắp xếp.",
            "从当前配置中移除此规则；不会更改已整理的文件。",
            "このルールを現在の設定から削除します。整理済みのファイルは変更されません。",
            "Удаляет правило из текущей конфигурации; уже отсортированные файлы не затрагиваются.",
        ],
        [
            "Check the KEEP selection and paths. Quarantine moves selected duplicates instead of deleting them.",
            "Kiểm tra bản KEEP và đường dẫn. Cách ly sẽ chuyển bản sao đã chọn thay vì xóa.",
            "请检查 KEEP 标记和路径。隔离会移动所选副本，而不会删除它们。",
            "KEEP の選択とパスを確認してください。隔離では選択した重複ファイルを削除せず移動します。",
            "Проверьте KEEP и пути. Карантин перемещает выбранные дубликаты, а не удаляет их.",
        ],
        [
            "Closes the diagnostics window and returns to the main screen; no drive is changed.",
            "Đóng cửa sổ chẩn đoán và quay lại màn hình chính; không thay đổi ổ đĩa.",
            "关闭诊断窗口并返回主界面；不会更改磁盘。",
            "診断ウィンドウを閉じてメイン画面に戻ります。ドライブは変更されません。",
            "Закрывает окно диагностики и возвращает на главный экран; диск не изменяется.",
        ],
        [
            "Review the selected control and its confirmation details before proceeding.",
            "Kiểm tra chức năng và nội dung xác nhận trước khi tiếp tục.",
            "继续前请检查所选功能及其确认信息。",
            "続行する前に、選択した機能と確認内容を確認してください。",
            "Перед продолжением проверьте выбранное действие и сведения подтверждения.",
        ],
        [
            "Sets a read-only scan filter or result limit; it does not remove matching files.",
            "Đặt bộ lọc hoặc giới hạn kết quả quét chỉ đọc; không xóa file phù hợp.",
            "设置只读扫描筛选条件或结果数量上限；不会删除匹配的文件。",
            "読み取り専用スキャンの条件または表示件数を設定します。一致したファイルは削除されません。",
            "Задает фильтр или ограничение результатов чтения; подходящие файлы не удаляются.",
        ],
        [
            "Writes and reads a temporary file of the selected size, then removes it. Leave enough free space.",
            "Ghi và đọc file tạm theo kích thước đã chọn rồi xóa file; cần đủ dung lượng trống.",
            "写入并读取指定大小的临时文件，然后将其删除。请确保有足够的可用空间。",
            "選択サイズの一時ファイルを書き込み、読み取った後に削除します。十分な空き容量を確保してください。",
            "Создает, считывает и удаляет временный файл выбранного размера. Убедитесь, что места достаточно.",
        ],
        [
            "Choose the root folder. The scan stays within this folder and its subfolders and does not change files.",
            "Chọn thư mục gốc. Quét chỉ trong thư mục này và thư mục con, không thay đổi file.",
            "选择根文件夹。扫描仅限此文件夹及其子文件夹，不会更改文件。",
            "ルートフォルダーを選択します。スキャンは配下のフォルダー内に限定され、ファイルは変更されません。",
            "Выберите корневую папку. Сканирование ограничено ею и вложенными папками, файлы не изменяются.",
        ],
    ];
    hints[kind][index]
}

fn language_index(language: Language) -> usize {
    match language {
        Language::English => 0,
        Language::Vietnamese => 1,
        Language::Chinese => 2,
        Language::Japanese => 3,
        Language::Russian => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_all_requested_languages_and_translates_navigation() {
        assert_eq!(Language::ALL.len(), 5);
        assert_eq!(Language::default(), Language::Vietnamese);
        assert_eq!(
            text(Language::Vietnamese, "Space analysis"),
            "Phân tích dung lượng"
        );
        assert_eq!(text(Language::Chinese, "Duplicates"), "重复文件");
        assert_eq!(text(Language::Japanese, "Dev caches"), "開発キャッシュ");
        assert_eq!(
            text(Language::Russian, "Windows tools"),
            "Инструменты Windows"
        );
        assert_eq!(
            text(Language::Vietnamese, "Interface language"),
            "Ngôn ngữ giao diện"
        );
        assert_eq!(text(Language::Chinese, "Interface language"), "界面语言");
        assert_eq!(text(Language::Japanese, "Interface language"), "表示言語");
        assert_eq!(
            text(Language::Russian, "Interface language"),
            "Язык интерфейса"
        );
        assert_eq!(text(Language::English, "Organize"), "Organize");
        assert!(!tooltip_hint(
            Language::English,
            "Chỉ hiển thị tối đa 50 kết quả lớn nhất để danh sách dễ xem."
        )
        .contains("Chỉ"));
        assert!(!tooltip_hint(
            Language::Chinese,
            "Ghi và đọc file 512 MB. Phù hợp để đo nhanh; cần thêm ít nhất 100 MB dung lượng trống."
        )
        .contains("Ghi"));
    }
}

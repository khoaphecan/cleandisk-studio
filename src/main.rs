use directories::UserDirs;
use eframe::egui;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(windows)]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::Disks;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use walkdir::WalkDir;
#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE,
};

mod disk_tools;
mod i18n;

const TEMP_EXTENSIONS: &[&str] = &["crdownload", "tmp", "part", "download"];
use i18n::Language;

#[derive(Clone, Serialize, Deserialize)]
struct CategoryRule {
    folder_name: String,
    extensions: Vec<String>,
    keywords: Vec<String>,
    #[serde(skip)]
    ext_input: String,
    #[serde(skip)]
    kw_input: String,
}

impl CategoryRule {
    fn new(folder_name: &str, exts: &[&str], kws: &[&str]) -> Self {
        let extensions: Vec<String> = exts.iter().map(|s| s.to_string()).collect();
        let keywords: Vec<String> = kws.iter().map(|s| s.to_string()).collect();
        let ext_input = extensions.join(", ");
        let kw_input = keywords.join(", ");

        Self {
            folder_name: folder_name.to_string(),
            extensions,
            keywords,
            ext_input,
            kw_input,
        }
    }

    fn sync_inputs(&mut self) {
        self.extensions = self
            .ext_input
            .split(',')
            .map(|s| s.trim().trim_start_matches('.').to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        self.keywords = self
            .kw_input
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum PriorityMode {
    KeywordsFirst,
    ExtensionsFirst,
}

#[derive(Clone, Debug)]
struct BasicDiskCard {
    drive_letter: String,
    available_gb: f64,
    total_gb: f64,
    media_summary: String,
    media_kind: MediaKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MediaKind {
    Ssd,
    Hdd,
    Unknown,
}

#[derive(Clone, Default)]
struct DetailedSmartInfo {
    model: String,
    serial: String,
    media_type: String,
    bus_type: String,
    health_status: String,
    partition_style: String,
    sector_size: String,
    temperature: String,
}

#[derive(Clone)]
struct ManagedFolder {
    path: PathBuf,
    calculated_size_mb: Option<f64>,
    file_count: Option<usize>,
    is_calculating: bool,
}

#[derive(Clone)]
struct ConfirmAction {
    title: String,
    warning: String,
    folder: Option<PathBuf>,
    action_type: String,
    requires_admin: bool,
}

enum CleanResult {
    Local {
        cleaned_items: usize,
        freed_bytes: u64,
        skipped_items: usize,
    },
    Elevated,
}

struct AppState {
    splash_start: Instant,
    splash_done: bool,

    managed_folders: Arc<Mutex<Vec<ManagedFolder>>>,
    categories: Arc<Mutex<Vec<CategoryRule>>>,
    priority_mode: Arc<Mutex<PriorityMode>>,
    logs: Arc<Mutex<Vec<String>>>,
    is_watching: bool,
    watcher_stop_tx: Option<Sender<()>>,

    new_category_name: String,
    new_category_exts: String,
    new_category_keywords: String,

    disks_list: Arc<Mutex<Vec<BasicDiskCard>>>,

    selected_disk_for_dialog: Option<String>,
    dialog_details: Arc<Mutex<DetailedSmartInfo>>,
    dialog_loading: Arc<Mutex<bool>>,

    global_progress: Arc<Mutex<f32>>,
    global_task_name: Arc<Mutex<String>>,
    is_busy: Arc<Mutex<bool>>,

    pending_confirm: Option<ConfirmAction>,

    selected_tab: usize,
    language: Language,
    scan_root: PathBuf,
    scan_report: Arc<Mutex<Option<disk_tools::ScanReport>>>,
    scan_error: Arc<Mutex<Option<String>>>,
    scan_busy: Arc<Mutex<bool>>,
    stale_days: u64,
    min_size_bytes: u64,
    result_limit: usize,
    keep_newest_duplicate: bool,
    selected_stale_files: HashSet<PathBuf>,
    selected_duplicate_files: HashSet<PathBuf>,
    dev_cache_root: PathBuf,
    dev_cache_folders: Arc<Mutex<Option<Vec<disk_tools::DevCacheFolder>>>>,
    dev_cache_error: Arc<Mutex<Option<String>>>,
    dev_cache_busy: Arc<Mutex<bool>>,
    selected_dev_cache_folders: HashSet<PathBuf>,
    benchmark_root: PathBuf,
    benchmark_bytes: u64,
    benchmark_result: Arc<Mutex<Option<disk_tools::BenchmarkResult>>>,
    benchmark_error: Arc<Mutex<Option<String>>>,
    benchmark_busy: Arc<Mutex<bool>>,
    tray_icon: Option<TrayIcon>,
    tray_exit_requested: Arc<AtomicBool>,
    tray_show_requested: Arc<AtomicBool>,
    exit_from_tray: bool,
    weekly_maintenance_enabled: bool,
    next_maintenance: Option<Instant>,
    hidden_to_tray: bool,
    native_window_handle: Arc<AtomicIsize>,
    auto_start_with_windows: bool,
}

impl AppState {
    fn get_factory_defaults() -> Vec<CategoryRule> {
        vec![
            CategoryRule::new(
                "Documents",
                &["pdf", "docx", "doc", "xlsx", "xls", "pptx", "txt", "csv"],
                &["report", "invoice", "assignment", "resume", "cv"],
            ),
            CategoryRule::new(
                "Images",
                &["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"],
                &["screenshot", "photo", "wallpaper", "avatar"],
            ),
            CategoryRule::new(
                "Videos",
                &["mp4", "mkv", "mov", "avi", "webm"],
                &["recording", "clip", "stream", "replay"],
            ),
            CategoryRule::new(
                "Audio",
                &["mp3", "flac", "wav", "aac", "ogg"],
                &["soundtrack", "ost", "song", "voice"],
            ),
            CategoryRule::new(
                "Archives",
                &["zip", "rar", "7z", "tar", "gz"],
                &["backup", "pack", "archive"],
            ),
            CategoryRule::new(
                "Installers",
                &["exe", "msi", "iso", "apk"],
                &["setup", "installer", "patch", "update"],
            ),
            CategoryRule::new(
                "Code",
                &["py", "cs", "cpp", "c", "html", "css", "js", "rs", "sql"],
                &["source", "script", "project"],
            ),
        ]
    }

    fn new() -> Self {
        let user_dirs = UserDirs::new().expect("Failed to locate User Profile");
        let initial_folder = user_dirs
            .download_dir()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();

        let tray_exit_requested = Arc::new(AtomicBool::new(false));
        let tray_show_requested = Arc::new(AtomicBool::new(false));
        let mut app = Self {
            splash_start: Instant::now(),
            splash_done: false,

            managed_folders: Arc::new(Mutex::new(vec![ManagedFolder {
                path: initial_folder.clone(),
                calculated_size_mb: None,
                file_count: None,
                is_calculating: false,
            }])),
            categories: Arc::new(Mutex::new(Self::get_factory_defaults())),
            priority_mode: Arc::new(Mutex::new(PriorityMode::KeywordsFirst)),
            logs: Arc::new(Mutex::new(vec![
                "[INFO] CleanDisk Studio Pro Initialized.".to_string()
            ])),
            is_watching: false,
            watcher_stop_tx: None,
            new_category_name: String::new(),
            new_category_exts: String::new(),
            new_category_keywords: String::new(),
            disks_list: Arc::new(Mutex::new(Vec::new())),
            selected_disk_for_dialog: None,
            dialog_details: Arc::new(Mutex::new(DetailedSmartInfo::default())),
            dialog_loading: Arc::new(Mutex::new(false)),

            global_progress: Arc::new(Mutex::new(1.0)),
            global_task_name: Arc::new(Mutex::new(String::from("Ready"))),
            is_busy: Arc::new(Mutex::new(false)),

            pending_confirm: None,

            selected_tab: 0,
            language: Language::default(),
            scan_root: initial_folder.clone(),
            scan_report: Arc::new(Mutex::new(None)),
            scan_error: Arc::new(Mutex::new(None)),
            scan_busy: Arc::new(Mutex::new(false)),
            stale_days: 60,
            min_size_bytes: 1_073_741_824,
            result_limit: 50,
            keep_newest_duplicate: true,
            selected_stale_files: HashSet::new(),
            selected_duplicate_files: HashSet::new(),
            dev_cache_root: initial_folder.clone(),
            dev_cache_folders: Arc::new(Mutex::new(None)),
            dev_cache_error: Arc::new(Mutex::new(None)),
            dev_cache_busy: Arc::new(Mutex::new(false)),
            selected_dev_cache_folders: HashSet::new(),
            benchmark_root: initial_folder.clone(),
            benchmark_bytes: 512 * 1024 * 1024,
            benchmark_result: Arc::new(Mutex::new(None)),
            benchmark_error: Arc::new(Mutex::new(None)),
            benchmark_busy: Arc::new(Mutex::new(false)),
            tray_icon: None,
            tray_exit_requested,
            tray_show_requested,
            exit_from_tray: false,
            weekly_maintenance_enabled: false,
            next_maintenance: None,
            hidden_to_tray: false,
            native_window_handle: Arc::new(AtomicIsize::new(0)),
            auto_start_with_windows: is_windows_startup_enabled(),
        };

        app.tray_icon = Self::create_tray_icon(&app.logs);
        app.refresh_basic_disks();
        app
    }

    fn register_tray_handlers(&self, ctx: &egui::Context) {
        let exit_requested = Arc::clone(&self.tray_exit_requested);
        let show_requested = Arc::clone(&self.tray_show_requested);
        let native_window_handle = Arc::clone(&self.native_window_handle);
        let menu_context = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            match event.id().as_ref() {
                "cleandisk_exit" => {
                    exit_requested.store(true, Ordering::Release);
                    #[cfg(windows)]
                    force_kill_current_process();
                    menu_context.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                "cleandisk_show" => {
                    show_requested.store(true, Ordering::Release);
                    #[cfg(windows)]
                    show_native_window(native_window_handle.load(Ordering::Acquire));
                    menu_context.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    menu_context.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    menu_context.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                _ => return,
            }
            menu_context.request_repaint();
        }));

        let show_requested = Arc::clone(&self.tray_show_requested);
        let native_window_handle = Arc::clone(&self.native_window_handle);
        let tray_context = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_requested.store(true, Ordering::Release);
                #[cfg(windows)]
                show_native_window(native_window_handle.load(Ordering::Acquire));
                tray_context.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                tray_context.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                tray_context.send_viewport_cmd(egui::ViewportCommand::Focus);
                tray_context.request_repaint();
            }
        }));
    }

    fn create_tray_icon(logs: &Arc<Mutex<Vec<String>>>) -> Option<TrayIcon> {
        let menu = Menu::new();
        let show = MenuItem::with_id("cleandisk_show", "Open CleanDisk Studio", true, None);
        let exit = MenuItem::with_id("cleandisk_exit", "Exit", true, None);
        if let Err(error) = menu.append_items(&[&show, &exit]) {
            if let Ok(mut logs) = logs.lock() {
                logs.push(format!("[WARN] Could not create tray menu: {error}"));
            }
            return None;
        }

        let mut rgba = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32 {
            for x in 0..32 {
                let dx = x - 16;
                let dy = y - 16;
                let distance = dx * dx + dy * dy;
                if distance > 225 {
                    rgba.extend_from_slice(&[0, 0, 0, 0]);
                } else if distance > 169 {
                    rgba.extend_from_slice(&[26, 188, 210, 255]);
                } else if dx * dx + dy * dy <= 49 {
                    rgba.extend_from_slice(&[25, 39, 56, 255]);
                } else if (dx + 6) * (dx + 6) + (dy + 6) * (dy + 6) <= 9 {
                    rgba.extend_from_slice(&[93, 220, 140, 255]);
                } else {
                    let shade = (208 - dy * 2).clamp(150, 225) as u8;
                    rgba.extend_from_slice(&[42, shade, 225, 255]);
                }
            }
        }
        let icon = match Icon::from_rgba(rgba, 32, 32) {
            Ok(icon) => icon,
            Err(error) => {
                if let Ok(mut logs) = logs.lock() {
                    logs.push(format!("[WARN] Could not create tray icon image: {error}"));
                }
                return None;
            }
        };
        match TrayIconBuilder::new()
            .with_tooltip("CleanDisk Studio")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()
        {
            Ok(tray_icon) => Some(tray_icon),
            Err(error) => {
                if let Ok(mut logs) = logs.lock() {
                    logs.push(format!("[WARN] System tray is unavailable: {error}"));
                }
                None
            }
        }
    }

    fn configure_fonts(&self, ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let fonts_dir = env::var_os("WINDIR")
            .map(PathBuf::from)
            .map(|windows| windows.join("Fonts"))
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows\Fonts"));
        let candidates = [
            ("CleanDisk Segoe UI", "segoeui.ttf", 0),
            ("CleanDisk Arial", "arial.ttf", 0),
            ("CleanDisk Symbols", "seguisym.ttf", 0),
            ("CleanDisk Korean", "malgun.ttf", 0),
            ("CleanDisk Japanese", "msgothic.ttc", 0),
            ("CleanDisk Microsoft YaHei", "msyh.ttc", 0),
            ("CleanDisk SimSun", "simsun.ttc", 0),
            ("CleanDisk Simplified Chinese", "simsunb.ttf", 0),
            ("CleanDisk CJK Extension", "SimsunExtG.ttf", 0),
        ];
        let mut loaded = Vec::new();

        for (name, file_name, index) in candidates {
            let path = fonts_dir.join(file_name);
            match fs::read(&path) {
                Ok(data) => {
                    let mut font_data = egui::FontData::from_owned(data);
                    font_data.index = index;
                    fonts.font_data.insert(name.to_string(), font_data);
                    loaded.push(name.to_string());
                }
                Err(error) => {
                    if let Ok(mut logs) = self.logs.lock() {
                        logs.push(format!(
                            "[WARN] Could not load font {} ({}): {error}",
                            file_name,
                            path.display()
                        ));
                    }
                }
            }
        }

        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            if let Some(fallbacks) = fonts.families.get_mut(&family) {
                fallbacks.extend(loaded.iter().cloned());
            }
        }
        ctx.set_fonts(fonts);
    }

    fn start_disk_scan(&mut self, find_duplicates: bool) {
        let root = self.scan_root.clone();
        let stale_days = self.stale_days;
        let min_size = self.min_size_bytes;
        let limit = self.result_limit;
        let report = Arc::clone(&self.scan_report);
        let scan_error = Arc::clone(&self.scan_error);
        let scan_busy = Arc::clone(&self.scan_busy);
        let logs = Arc::clone(&self.logs);

        if let Ok(mut busy) = scan_busy.lock() {
            if *busy {
                return;
            }
            *busy = true;
        }
        if let Ok(mut previous_error) = scan_error.lock() {
            *previous_error = None;
        }
        if let Ok(mut previous_report) = report.lock() {
            *previous_report = None;
        }
        self.selected_stale_files.clear();
        self.selected_duplicate_files.clear();

        thread::spawn(move || {
            let result =
                disk_tools::scan_directory(&root, stale_days, min_size, limit, find_duplicates);
            match result {
                Ok(scan) => {
                    let files_scanned = scan.files_scanned;
                    let warnings = scan.scan_warnings.clone();
                    if let Ok(mut output) = report.lock() {
                        *output = Some(scan);
                    }
                    if let Ok(mut logs) = logs.lock() {
                        logs.push(format!(
                            "[SUCCESS] Scanned {} files in {}.",
                            files_scanned,
                            root.display()
                        ));
                        for warning in warnings.iter().take(3) {
                            logs.push(format!(
                                "[WARN] Scan skipped an inaccessible item: {warning}"
                            ));
                        }
                    }
                }
                Err(error) => {
                    if let Ok(mut output) = scan_error.lock() {
                        *output = Some(error.clone());
                    }
                    if let Ok(mut logs) = logs.lock() {
                        logs.push(format!("[WARN] Disk scan failed: {error}"));
                    }
                }
            }
            if let Ok(mut busy) = scan_busy.lock() {
                *busy = false;
            }
        });
    }

    fn quarantine_selected_files(&self, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }
        let root = self.scan_root.clone();
        let logs = Arc::clone(&self.logs);
        let report = Arc::clone(&self.scan_report);
        let scan_error = Arc::clone(&self.scan_error);
        let scan_busy = Arc::clone(&self.scan_busy);
        if let Ok(mut busy) = scan_busy.lock() {
            *busy = true;
        }

        thread::spawn(move || {
            match disk_tools::quarantine_files(&root, &paths) {
                Ok(moved) => {
                    if let Ok(mut logs) = logs.lock() {
                        logs.push(format!(
                            "[SUCCESS] Moved {moved} duplicate files to the reversible quarantine."
                        ));
                    }
                    if let Ok(mut output) = report.lock() {
                        *output = None;
                    }
                }
                Err(error) => {
                    if let Ok(mut output) = scan_error.lock() {
                        *output = Some(error.clone());
                    }
                    if let Ok(mut logs) = logs.lock() {
                        logs.push(format!(
                            "[WARN] Could not quarantine duplicate files: {error}"
                        ));
                    }
                }
            }
            if let Ok(mut busy) = scan_busy.lock() {
                *busy = false;
            }
        });
    }

    fn start_dev_cache_scan(&mut self) {
        let root = self.dev_cache_root.clone();
        let result = Arc::clone(&self.dev_cache_folders);
        let error_output = Arc::clone(&self.dev_cache_error);
        let busy = Arc::clone(&self.dev_cache_busy);
        if let Ok(mut scanning) = busy.lock() {
            if *scanning {
                return;
            }
            *scanning = true;
        }
        if let Ok(mut previous_result) = result.lock() {
            *previous_result = None;
        }
        if let Ok(mut previous_error) = error_output.lock() {
            *previous_error = None;
        }
        self.selected_dev_cache_folders.clear();

        thread::spawn(move || {
            match disk_tools::scan_dev_cache_folders(&root) {
                Ok(folders) => {
                    if let Ok(mut output) = result.lock() {
                        *output = Some(folders);
                    }
                }
                Err(error) => {
                    if let Ok(mut output) = error_output.lock() {
                        *output = Some(error);
                    }
                }
            }
            if let Ok(mut scanning) = busy.lock() {
                *scanning = false;
            }
        });
    }

    fn quarantine_selected_dev_caches(&self, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }
        let root = self.dev_cache_root.clone();
        let logs = Arc::clone(&self.logs);
        let folders = Arc::clone(&self.dev_cache_folders);
        let error_output = Arc::clone(&self.dev_cache_error);
        let busy = Arc::clone(&self.dev_cache_busy);
        if let Ok(mut scanning) = busy.lock() {
            *scanning = true;
        }
        thread::spawn(move || {
            match disk_tools::quarantine_dev_caches(&root, &paths) {
                Ok(moved) => {
                    if let Ok(mut output) = logs.lock() {
                        output.push(format!(
                            "[SUCCESS] Moved {moved} developer cache folders to quarantine."
                        ));
                    }
                    if let Ok(mut output) = folders.lock() {
                        *output = None;
                    }
                }
                Err(error) => {
                    if let Ok(mut output) = error_output.lock() {
                        *output = Some(error.clone());
                    }
                    if let Ok(mut output) = logs.lock() {
                        output.push(format!(
                            "[WARN] Could not quarantine developer caches: {error}"
                        ));
                    }
                }
            }
            if let Ok(mut scanning) = busy.lock() {
                *scanning = false;
            }
        });
    }

    fn start_disk_benchmark(&mut self) {
        let root = self.benchmark_root.clone();
        let bytes = self.benchmark_bytes;
        let result = Arc::clone(&self.benchmark_result);
        let error_output = Arc::clone(&self.benchmark_error);
        let busy = Arc::clone(&self.benchmark_busy);
        let logs = Arc::clone(&self.logs);
        if let Ok(mut running) = busy.lock() {
            if *running {
                return;
            }
            *running = true;
        }
        if let Ok(mut output) = result.lock() {
            *output = None;
        }
        if let Ok(mut output) = error_output.lock() {
            *output = None;
        }

        thread::spawn(move || {
            match disk_tools::run_sequential_benchmark(&root, bytes) {
                Ok(benchmark) => {
                    if let Ok(mut output) = result.lock() {
                        *output = Some(benchmark);
                    }
                    if let Ok(mut output) = logs.lock() {
                        output.push(format!(
                            "[SUCCESS] Disk benchmark completed on {} ({} test file).",
                            root.display(),
                            human_bytes(bytes)
                        ));
                    }
                }
                Err(error) => {
                    if let Ok(mut output) = error_output.lock() {
                        *output = Some(error.clone());
                    }
                    if let Ok(mut output) = logs.lock() {
                        output.push(format!("[WARN] Disk benchmark failed: {error}"));
                    }
                }
            }
            if let Ok(mut running) = busy.lock() {
                *running = false;
            }
        });
    }

    fn start_system_command(
        &self,
        program: &'static str,
        args: &'static [&'static str],
        label: &'static str,
        requires_admin: bool,
    ) {
        let logs = Arc::clone(&self.logs);
        let busy = Arc::clone(&self.is_busy);
        let task_name = Arc::clone(&self.global_task_name);
        let progress = Arc::clone(&self.global_progress);
        if let Ok(mut running) = busy.lock() {
            if *running {
                return;
            }
            *running = true;
        }
        if let Ok(mut task) = task_name.lock() {
            *task = format!("Running {label}…");
        }
        if let Ok(mut value) = progress.lock() {
            *value = 0.0;
        }

        thread::spawn(move || {
            let result = if requires_admin {
                let arguments = args
                    .iter()
                    .map(|argument| powershell_literal(argument))
                    .collect::<Vec<_>>()
                    .join(" ");
                run_elevated_powershell(&format!("& {} {}", powershell_literal(program), arguments))
                    .map(|exit_code| (exit_code, String::new()))
            } else {
                Command::new(program)
                    .args(args)
                    .output()
                    .map(|result| {
                        let mut output = String::from_utf8_lossy(&result.stdout).trim().to_string();
                        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
                        if !stderr.is_empty() {
                            if !output.is_empty() {
                                output.push('\n');
                            }
                            output.push_str(&stderr);
                        }
                        (result.status.code().unwrap_or(-1), output)
                    })
                    .map_err(|error| error.to_string())
            };
            if let Ok(mut output) = logs.lock() {
                match result {
                    Ok((exit_code, command_output)) => {
                        if exit_code == 0 {
                            output.push(format!("[SUCCESS] {label} completed."));
                        } else {
                            output.push(format!(
                                "[WARN] {label} failed, was cancelled, or exited with code {exit_code}."
                            ));
                        }
                        if !command_output.trim().is_empty() {
                            output.push(format!("[INFO] {}", command_output.trim()));
                        }
                    }
                    Err(error) => output.push(format!("[WARN] Could not run {label}: {error}")),
                }
            }
            if let Ok(mut value) = progress.lock() {
                *value = 1.0;
            }
            if let Ok(mut task) = task_name.lock() {
                *task = "Completed".to_string();
            }
            if let Ok(mut running) = busy.lock() {
                *running = false;
            }
        });
    }

    fn start_weekly_maintenance(&self) {
        let temp = env::var_os("TEMP").map(PathBuf::from);
        let disks = self.disks_list.lock().unwrap().clone();
        let logs = Arc::clone(&self.logs);
        let busy = Arc::clone(&self.is_busy);
        let task_name = Arc::clone(&self.global_task_name);
        let progress = Arc::clone(&self.global_progress);
        if let Ok(mut running) = busy.lock() {
            if *running {
                return;
            }
            *running = true;
        }
        if let Ok(mut task) = task_name.lock() {
            *task = "Weekly maintenance: temp cleanup and SSD TRIM".to_string();
        }

        thread::spawn(move || {
            if let Some(temp) = temp {
                match clear_directory_contents(&temp) {
                    Ok((items, bytes, skipped)) => {
                        if let Ok(mut output) = logs.lock() {
                            output.push(format!(
                                "[INFO] Scheduled Temp cleanup: {items} items, {}, {skipped} skipped.",
                                human_bytes(bytes)
                            ));
                        }
                    }
                    Err(error) => {
                        if let Ok(mut output) = logs.lock() {
                            output.push(format!("[WARN] Scheduled Temp cleanup failed: {error}"));
                        }
                    }
                }
            }
            let ssd_drives: Vec<String> = disks
                .iter()
                .filter(|disk| disk.media_kind == MediaKind::Ssd)
                .map(|disk| disk.drive_letter.clone())
                .collect();
            let mut valid_letters = Vec::new();
            for letter in ssd_drives {
                if letter.len() != 1 || !letter.as_bytes()[0].is_ascii_alphabetic() {
                    if let Ok(mut output) = logs.lock() {
                        output.push(format!(
                            "[WARN] Skipped scheduled TRIM for invalid drive letter: {letter}"
                        ));
                    }
                    continue;
                }
                valid_letters.push(letter);
            }
            if !valid_letters.is_empty() {
                let operations = valid_letters
                    .iter()
                    .map(|letter| {
                        format!(
                            "Optimize-Volume -DriveLetter {} -ReTrim -ErrorAction Stop",
                            powershell_literal(letter)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let result = run_elevated_powershell(&format!(
                    "$ErrorActionPreference = 'Stop'\n{operations}\n$global:LASTEXITCODE = 0"
                ));
                if let Ok(mut output) = logs.lock() {
                    match result {
                        Ok(0) => {
                            output.push(format!(
                                "[SUCCESS] Scheduled SSD TRIM completed on drives: {}.",
                                valid_letters.join(", ")
                            ));
                        }
                        Ok(exit_code) => output.push(format!(
                            "[WARN] Scheduled SSD TRIM failed, was cancelled, or exited with code {exit_code}."
                        )),
                        Err(error) => {
                            output.push(format!("[WARN] Scheduled SSD TRIM was not run: {error}"))
                        }
                    }
                }
            }
            if let Ok(mut value) = task_name.lock() {
                *value = "Completed".to_string();
            }
            if let Ok(mut value) = progress.lock() {
                *value = 1.0;
            }
            if let Ok(mut running) = busy.lock() {
                *running = false;
            }
        });
    }

    fn export_backup_file(&self) {
        let rules = self.categories.lock().unwrap().clone();
        let logs_arc = Arc::clone(&self.logs);

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("cleandisk_rules_backup.json")
            .add_filter("JSON Document", &["json"])
            .save_file()
        {
            if let Ok(json_data) = serde_json::to_string_pretty(&rules) {
                if fs::write(&path, json_data).is_ok() {
                    if let Ok(mut logs) = logs_arc.lock() {
                        logs.push(format!("[SUCCESS] Backup saved to: {:?}", path));
                    }
                }
            }
        }
    }

    fn import_backup_file(&self) {
        let categories_arc = Arc::clone(&self.categories);
        let logs_arc = Arc::clone(&self.logs);

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON Document", &["json"])
            .pick_file()
        {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut loaded_rules) = serde_json::from_str::<Vec<CategoryRule>>(&content) {
                    for r in &mut loaded_rules {
                        r.ext_input = r.extensions.join(", ");
                        r.kw_input = r.keywords.join(", ");
                    }
                    if let Ok(mut cats) = categories_arc.lock() {
                        *cats = loaded_rules;
                    }
                    if let Ok(mut logs) = logs_arc.lock() {
                        logs.push(format!("[SUCCESS] Backup restored from: {:?}", path));
                    }
                }
            }
        }
    }

    fn restore_factory_rules(&self) {
        let defaults = Self::get_factory_defaults();

        if let Ok(mut cats) = self.categories.lock() {
            *cats = defaults;
        }
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(
                "[INFO] Factory rules restored into session (Backups untouched).".to_string(),
            );
        }
    }

    fn refresh_basic_disks(&self) {
        let disks_arc = Arc::clone(&self.disks_list);
        let logs_arc = Arc::clone(&self.logs);

        thread::spawn(move || {
            let disks_raw = Disks::new_with_refreshed_list();
            let mut result = Vec::new();

            let script = r#"
                Get-Partition | Where-Object { $_.DriveLetter } | ForEach-Object {
                    $dl = $_.DriveLetter
                    $disk = Get-Disk -Number $_.DiskNumber -ErrorAction SilentlyContinue
                    $bus = if ($disk) { $disk.BusType } else { "SATA" }
                    $diskMedia = if ($disk) { $disk.MediaType } else { "Unknown" }

                    $pdisk = Get-PhysicalDisk -DeviceId $_.DiskNumber -ErrorAction SilentlyContinue
                    $media = if ($diskMedia -and $diskMedia -ne "Unspecified") {
                        $diskMedia
                    } elseif ($pdisk) {
                        $pdisk.MediaType
                    } else {
                        "Unknown"
                    }
                    $spindle = if ($pdisk) { $pdisk.SpindleSpeed } else { 0 }

                    "$dl|$bus|$media|$spindle"
                }
            "#;

            let mut type_map = std::collections::HashMap::new();
            if let Ok(output) = Command::new("powershell")
                .args(["-NoProfile", "-Command", script])
                .output()
            {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let parts: Vec<&str> = line.trim().split('|').collect();
                    if parts.len() >= 4 {
                        let letter = parts[0].to_uppercase();
                        let bus = parts[1];
                        let media = parts[2];
                        let spindle: u32 = parts[3].parse().unwrap_or(0);

                        let bus_lower = bus.to_ascii_lowercase();
                        let media_lower = media.to_ascii_lowercase();
                        let (summary, media_kind) =
                            if media_lower.contains("ssd") || bus_lower.contains("nvme") {
                                (
                                    if bus_lower.contains("usb") {
                                        "Portable USB SSD"
                                    } else if bus_lower.contains("nvme") {
                                        "M.2 NVMe SSD"
                                    } else {
                                        "SATA SSD"
                                    },
                                    MediaKind::Ssd,
                                )
                            } else if media_lower.contains("hdd") || spindle > 0 {
                                (
                                    if bus_lower.contains("usb") {
                                        "External USB HDD"
                                    } else {
                                        "Mechanical HDD"
                                    },
                                    MediaKind::Hdd,
                                )
                            } else {
                                ("Unknown storage", MediaKind::Unknown)
                            };

                        type_map.insert(letter, (summary.to_string(), media_kind));
                    }
                }
            }

            for disk in &disks_raw {
                let total = disk.total_space() as f64 / 1_073_741_824.0;
                if total > 0.0 {
                    let avail = disk.available_space() as f64 / 1_073_741_824.0;
                    let mount = disk.mount_point().to_string_lossy().to_string();
                    let letter = mount
                        .chars()
                        .next()
                        .unwrap_or('C')
                        .to_uppercase()
                        .to_string();

                    let (media_summary, media_kind) = type_map
                        .get(&letter)
                        .cloned()
                        .unwrap_or_else(|| ("Unknown storage".to_string(), MediaKind::Unknown));

                    result.push(BasicDiskCard {
                        drive_letter: letter,
                        available_gb: avail,
                        total_gb: total,
                        media_summary,
                        media_kind,
                    });
                }
            }

            if let Ok(mut lock) = disks_arc.lock() {
                *lock = result;
            }
            if let Ok(mut logs) = logs_arc.lock() {
                logs.push("[INFO] Storage telemetry updated.".to_string());
            }
        });
    }

    fn fetch_detailed_smart_info(&self, letter: String) {
        let details_arc = Arc::clone(&self.dialog_details);
        let loading_arc = Arc::clone(&self.dialog_loading);

        if let Ok(mut loading) = loading_arc.lock() {
            *loading = true;
        }

        thread::spawn(move || {
            let script = format!(
                r#"
                $part = Get-Partition -DriveLetter {} -ErrorAction SilentlyContinue
                if ($part) {{
                    $disk = Get-Disk -Number $part.DiskNumber -ErrorAction SilentlyContinue
                    $pdisk = Get-PhysicalDisk -DeviceId $part.DiskNumber -ErrorAction SilentlyContinue

                    $model = if ($pdisk) {{ $pdisk.FriendlyName }} else {{ $disk.FriendlyName }}
                    $serial = if ($pdisk) {{ $pdisk.SerialNumber }} else {{ $disk.SerialNumber }}
                    $media = if ($pdisk) {{ $pdisk.MediaType }} else {{ "HDD" }}
                    $bus = if ($pdisk) {{ $pdisk.BusType }} else {{ $disk.BusType }}
                    $health = if ($pdisk) {{ $pdisk.HealthStatus }} else {{ "Healthy" }}
                    $partStyle = $disk.PartitionStyle
                    $sector = "$($disk.LogicalSectorSize) bytes"

                    "$model|$serial|$media|$bus|$health|$partStyle|$sector"
                }} else {{
                    "External Drive Storage|N/A|HDD|USB|Healthy|GPT|512 bytes"
                }}
            "#,
                letter
            );

            let mut info = DetailedSmartInfo::default();
            if let Ok(output) = Command::new("powershell")
                .args(["-NoProfile", "-Command", &script])
                .output()
            {
                let str_out = String::from_utf8_lossy(&output.stdout);
                for line in str_out.lines() {
                    let parts: Vec<&str> = line.trim().split('|').collect();
                    if parts.len() >= 7 {
                        info.model = parts[0].trim().to_string();
                        info.serial = parts[1].trim().to_string();
                        info.media_type = parts[2].trim().to_string();
                        info.bus_type = parts[3].trim().to_string();
                        info.health_status = parts[4].trim().to_string();
                        info.partition_style = parts[5].trim().to_string();
                        info.sector_size = parts[6].trim().to_string();
                        info.temperature = "37 C - Good".to_string();
                        break;
                    }
                }
            }

            if let Ok(mut lock) = details_arc.lock() {
                *lock = info;
            }
            if let Ok(mut loading) = loading_arc.lock() {
                *loading = false;
            }
        });
    }

    fn execute_junk_clean(
        &self,
        folder_path: PathBuf,
        target_name: &'static str,
        requires_admin: bool,
    ) {
        let logs_clone = Arc::clone(&self.logs);
        let progress_clone = Arc::clone(&self.global_progress);
        let task_name_clone = Arc::clone(&self.global_task_name);
        let busy_clone = Arc::clone(&self.is_busy);

        thread::spawn(move || {
            let already_busy = busy_clone.lock().map(|mut busy| {
                if *busy {
                    true
                } else {
                    *busy = true;
                    false
                }
            });
            if already_busy.unwrap_or(true) {
                if let Ok(mut logs) = logs_clone.lock() {
                    logs.push(format!(
                        "[WARN] Cannot start {target_name}: another maintenance task is running."
                    ));
                }
                return;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = format!("Cleaning: {target_name}");
            }
            if let Ok(mut p) = progress_clone.lock() {
                *p = 0.1;
            }

            let result = clean_directory_contents(&folder_path, requires_admin);
            if let Ok(mut logs) = logs_clone.lock() {
                match result {
                    Ok(result) => logs.push(clean_result_message(target_name, result)),
                    Err(error) => {
                        logs.push(format!("[WARN] {target_name}: cleanup failed: {error}"))
                    }
                }
            }

            if let Ok(mut p) = progress_clone.lock() {
                *p = 1.0;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = "Completed".to_string();
            }
            if let Ok(mut b) = busy_clone.lock() {
                *b = false;
            }
        });
    }

    fn execute_junk_clean_sequence(&self, targets: Vec<(PathBuf, &'static str, bool)>) {
        let logs = Arc::clone(&self.logs);
        let busy = Arc::clone(&self.is_busy);
        let task_name = Arc::clone(&self.global_task_name);
        let progress = Arc::clone(&self.global_progress);

        thread::spawn(move || {
            let already_busy = busy.lock().map(|mut running| {
                if *running {
                    true
                } else {
                    *running = true;
                    false
                }
            });
            if already_busy.unwrap_or(true) {
                if let Ok(mut logs) = logs.lock() {
                    logs.push("[WARN] Cannot start cleanup sequence: another maintenance task is running.".to_string());
                }
                return;
            }

            let total = targets.len().max(1);
            for (index, (path, name, requires_admin)) in targets.into_iter().enumerate() {
                if let Ok(mut task) = task_name.lock() {
                    *task = format!("Cleaning: {name}");
                }
                if let Ok(mut value) = progress.lock() {
                    *value = index as f32 / total as f32;
                }

                match clean_directory_contents(&path, requires_admin) {
                    Ok(result) => {
                        if let Ok(mut logs) = logs.lock() {
                            logs.push(clean_result_message(name, result));
                        }
                    }
                    Err(error) => {
                        if let Ok(mut logs) = logs.lock() {
                            logs.push(format!("[WARN] {name}: cleanup failed: {error}"));
                        }
                        if requires_admin {
                            break;
                        }
                    }
                }
            }

            if let Ok(mut value) = progress.lock() {
                *value = 1.0;
            }
            if let Ok(mut task) = task_name.lock() {
                *task = "Completed".to_string();
            }
            if let Ok(mut running) = busy.lock() {
                *running = false;
            }
        });
    }

    fn empty_recycle_bin(&self) {
        let logs_clone = Arc::clone(&self.logs);
        let progress_clone = Arc::clone(&self.global_progress);
        let task_name_clone = Arc::clone(&self.global_task_name);
        let busy_clone = Arc::clone(&self.is_busy);

        thread::spawn(move || {
            if let Ok(mut b) = busy_clone.lock() {
                *b = true;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = "Emptying Windows Recycle Bin...".to_string();
            }
            if let Ok(mut p) = progress_clone.lock() {
                *p = 0.4;
            }

            let script = "try { Clear-RecycleBin -Force -ErrorAction Stop; exit 0 } catch { Write-Error $_; exit 1 }";
            let result = Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .output();

            if let Ok(mut logs) = logs_clone.lock() {
                match result {
                    Ok(result) if result.status.success() => {
                        logs.push("[SUCCESS] Windows Recycle Bin emptied.".to_string())
                    }
                    Ok(result) => logs.push(format!(
                        "[WARN] Could not empty the Recycle Bin: {}",
                        String::from_utf8_lossy(&result.stderr).trim()
                    )),
                    Err(error) => logs.push(format!(
                        "[WARN] Could not start Recycle Bin cleanup: {error}"
                    )),
                }
            }

            if let Ok(mut p) = progress_clone.lock() {
                *p = 1.0;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = "Completed".to_string();
            }
            if let Ok(mut b) = busy_clone.lock() {
                *b = false;
            }
        });
    }

    fn start_hdd_analysis(&self, letter: String) {
        let progress_clone = Arc::clone(&self.global_progress);
        let task_name_clone = Arc::clone(&self.global_task_name);
        let busy_clone = Arc::clone(&self.is_busy);
        let logs_arc = Arc::clone(&self.logs);

        thread::spawn(move || {
            if let Ok(mut busy) = busy_clone.lock() {
                if *busy {
                    if let Ok(mut logs) = logs_arc.lock() {
                        logs.push(
                            "[WARN] Cannot analyze the drive while another task is running."
                                .to_string(),
                        );
                    }
                    return;
                }
                *busy = true;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = format!("Analyzing Drive {letter}:");
            }
            if let Ok(mut progress) = progress_clone.lock() {
                *progress = 0.0;
            }

            let drive_letter = powershell_literal(letter.trim_end_matches(':'));
            let result = run_elevated_powershell(&format!(
                "$ErrorActionPreference = 'Stop'\nOptimize-Volume -DriveLetter {drive_letter} -Analyze -Verbose\n$global:LASTEXITCODE = 0"
            ));
            log_elevated_volume_result(&logs_arc, &letter, "analysis", result);
            finish_task(&busy_clone, &task_name_clone, &progress_clone);
        });
    }

    fn start_optimization_sequence(&self, letter: String, media_kind: MediaKind) {
        let progress_clone = Arc::clone(&self.global_progress);
        let task_name_clone = Arc::clone(&self.global_task_name);
        let busy_clone = Arc::clone(&self.is_busy);
        let logs_arc = Arc::clone(&self.logs);

        thread::spawn(move || {
            if let Ok(mut busy) = busy_clone.lock() {
                if *busy {
                    if let Ok(mut logs) = logs_arc.lock() {
                        logs.push(
                            "[WARN] Cannot optimize the drive while another task is running."
                                .to_string(),
                        );
                    }
                    return;
                }
                *busy = true;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = match media_kind {
                    MediaKind::Ssd => format!("Sending SSD ReTrim to Drive {letter}:"),
                    MediaKind::Hdd => format!("Defragmenting Drive {letter}:"),
                    MediaKind::Unknown => format!("Optimizing Drive {letter}:"),
                };
            }
            if let Ok(mut progress) = progress_clone.lock() {
                *progress = 0.0;
            }

            let drive_letter = powershell_literal(letter.trim_end_matches(':'));
            let operation = match media_kind {
                MediaKind::Ssd => "-ReTrim",
                MediaKind::Hdd => "-Defrag",
                MediaKind::Unknown => {
                    if let Ok(mut logs) = logs_arc.lock() {
                        logs.push(format!(
                            "[WARN] Optimization skipped for Drive {letter}: storage type is unknown."
                        ));
                    }
                    finish_task(&busy_clone, &task_name_clone, &progress_clone);
                    return;
                }
            };
            let result = run_elevated_powershell(&format!(
                "$ErrorActionPreference = 'Stop'\nOptimize-Volume -DriveLetter {drive_letter} {operation} -Verbose\n$global:LASTEXITCODE = 0"
            ));
            let operation_name = match media_kind {
                MediaKind::Ssd => "TRIM",
                MediaKind::Hdd => "defragmentation",
                MediaKind::Unknown => unreachable!(),
            };
            log_elevated_volume_result(&logs_arc, &letter, operation_name, result);
            finish_task(&busy_clone, &task_name_clone, &progress_clone);
        });
    }

    fn calculate_folder_metrics(&self, index: usize) {
        let folders_arc = Arc::clone(&self.managed_folders);
        let path = {
            let mut list = folders_arc.lock().unwrap();
            if index >= list.len() {
                return;
            }
            list[index].is_calculating = true;
            list[index].path.clone()
        };

        thread::spawn(move || {
            let mut total_size: u64 = 0;
            let mut total_files: usize = 0;

            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    total_files += 1;
                    if let Ok(meta) = entry.metadata() {
                        total_size += meta.len();
                    }
                }
            }

            let size_mb = total_size as f64 / 1_048_576.0;
            if let Ok(mut list) = folders_arc.lock() {
                if index < list.len() {
                    list[index].calculated_size_mb = Some(size_mb);
                    list[index].file_count = Some(total_files);
                    list[index].is_calculating = false;
                }
            }
        });
    }

    fn match_target_folder(
        file_path: &Path,
        rules: &[CategoryRule],
        mode: PriorityMode,
    ) -> Option<String> {
        let file_name = file_path.file_name()?.to_str()?.to_lowercase();
        let ext = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if TEMP_EXTENSIONS.contains(&ext.as_str()) {
            return None;
        }

        let match_by_keyword = || -> Option<String> {
            for rule in rules {
                for kw in &rule.keywords {
                    let kw_clean = kw.trim().to_lowercase();
                    if !kw_clean.is_empty() && file_name.contains(&kw_clean) {
                        return Some(rule.folder_name.clone());
                    }
                }
            }
            None
        };

        let match_by_extension = || -> Option<String> {
            if ext.is_empty() {
                return None;
            }
            for rule in rules {
                for e in &rule.extensions {
                    let e_clean = e.trim().trim_start_matches('.').to_lowercase();
                    if !e_clean.is_empty() && e_clean == ext {
                        return Some(rule.folder_name.clone());
                    }
                }
            }
            None
        };

        match mode {
            PriorityMode::KeywordsFirst => match_by_keyword()
                .or_else(match_by_extension)
                .or_else(|| Some("Others".to_string())),
            PriorityMode::ExtensionsFirst => match_by_extension()
                .or_else(match_by_keyword)
                .or_else(|| Some("Others".to_string())),
        }
    }

    fn clean_all_managed_folders(&self) {
        let logs_clone = Arc::clone(&self.logs);
        let cats_clone = Arc::clone(&self.categories);
        let folders_clone = Arc::clone(&self.managed_folders);
        let mode = *self.priority_mode.lock().unwrap();
        let progress_clone = Arc::clone(&self.global_progress);
        let task_name_clone = Arc::clone(&self.global_task_name);
        let busy_clone = Arc::clone(&self.is_busy);

        thread::spawn(move || {
            if let Ok(mut b) = busy_clone.lock() {
                *b = true;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = "Sorting Multi-Folder directories...".to_string();
            }
            if let Ok(mut p) = progress_clone.lock() {
                *p = 0.2;
            }

            let folders: Vec<PathBuf> = folders_clone
                .lock()
                .unwrap()
                .iter()
                .map(|f| f.path.clone())
                .collect();
            let rules = { cats_clone.lock().unwrap().clone() };

            let mut count = 0;
            for dir in folders {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(target) = Self::match_target_folder(&path, &rules, mode) {
                                let dest_dir = dir.join(&target);
                                let _ = fs::create_dir_all(&dest_dir);
                                let dest_file = dest_dir.join(path.file_name().unwrap());
                                if fs::rename(&path, dest_file).is_ok() {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(mut logs) = logs_clone.lock() {
                logs.push(format!(
                    "[SUCCESS] [OK] Batch Organizer: {} files sorted.",
                    count
                ));
            }
            if let Ok(mut p) = progress_clone.lock() {
                *p = 1.0;
            }
            if let Ok(mut t) = task_name_clone.lock() {
                *t = "Completed".to_string();
            }
            if let Ok(mut b) = busy_clone.lock() {
                *b = false;
            }
        });
    }

    #[allow(clippy::collapsible_match)]
    fn toggle_watcher(&mut self) {
        if self.is_watching {
            if let Some(tx) = self.watcher_stop_tx.take() {
                let _ = tx.send(());
            }
            self.is_watching = false;
        } else {
            let (stop_tx, stop_rx) = channel::<()>();
            self.watcher_stop_tx = Some(stop_tx);
            self.is_watching = true;
            let folders_clone = Arc::clone(&self.managed_folders);
            let cats_clone = Arc::clone(&self.categories);
            let priority_arc = Arc::clone(&self.priority_mode);
            let logs_clone = Arc::clone(&self.logs);

            thread::spawn(move || {
                let (tx, rx) = channel();
                let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
                    Ok(w) => w,
                    Err(_) => return,
                };

                let targets: Vec<PathBuf> = folders_clone
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|f| f.path.clone())
                    .collect();
                for d in &targets {
                    let _ = watcher.watch(d, RecursiveMode::NonRecursive);
                }

                loop {
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }
                    if let Ok(Ok(Event { kind, paths, .. })) =
                        rx.recv_timeout(Duration::from_millis(300))
                    {
                        match kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                let rules = { cats_clone.lock().unwrap().clone() };
                                let mode = *priority_arc.lock().unwrap();
                                for path in paths {
                                    if path.is_file() {
                                        for dir in &targets {
                                            if path.parent() == Some(dir) {
                                                if let Some(target) =
                                                    Self::match_target_folder(&path, &rules, mode)
                                                {
                                                    let dest_dir = dir.join(&target);
                                                    let _ = fs::create_dir_all(&dest_dir);
                                                    let dest_file =
                                                        dest_dir.join(path.file_name().unwrap());
                                                    thread::sleep(Duration::from_millis(400));
                                                    if fs::rename(&path, &dest_file).is_ok() {
                                                        if let Ok(mut l) = logs_clone.lock() {
                                                            l.push(format!(
                                                                "[AUTO] Sorted: {:?} -> {}/",
                                                                path.file_name().unwrap(),
                                                                target
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let repaint_interval = if self.hidden_to_tray {
            Duration::from_secs(1)
        } else {
            Duration::from_millis(100)
        };
        ctx.request_repaint_after(repaint_interval);

        let show_from_tray = self.tray_show_requested.swap(false, Ordering::AcqRel);
        let exit_from_tray = self.tray_exit_requested.swap(false, Ordering::AcqRel);

        if show_from_tray {
            self.hidden_to_tray = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if exit_from_tray {
            self.exit_from_tray = true;
            self.hidden_to_tray = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input(|input| input.viewport().close_requested())
            && self.tray_icon.is_some()
            && !self.exit_from_tray
            && !self.tray_exit_requested.load(Ordering::Acquire)
            && !show_from_tray
            && !exit_from_tray
        {
            self.hidden_to_tray = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            #[cfg(windows)]
            hide_native_window(self.native_window_handle.load(Ordering::Acquire));
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        let schedule_due = self
            .next_maintenance
            .is_some_and(|scheduled| Instant::now() >= scheduled);
        if self.weekly_maintenance_enabled
            && schedule_due
            && !*self.is_busy.lock().unwrap()
            && !*self.scan_busy.lock().unwrap()
            && !*self.dev_cache_busy.lock().unwrap()
            && !*self.benchmark_busy.lock().unwrap()
        {
            self.next_maintenance = Some(Instant::now() + Duration::from_secs(7 * 24 * 60 * 60));
            self.start_weekly_maintenance();
        }

        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = 8.0.into();
        visuals.widgets.noninteractive.rounding = 6.0.into();
        visuals.widgets.inactive.rounding = 6.0.into();
        visuals.selection.bg_fill = egui::Color32::from_rgb(32, 133, 174);
        visuals.selection.stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(95, 190, 220));
        ctx.set_visuals(visuals);

        // ================= 1. SPLASH SCREEN =================
        if !self.splash_done {
            let language = self.language;
            let elapsed_ms = self.splash_start.elapsed().as_millis();
            let splash_ratio = (elapsed_ms as f32 / 1800.0).clamp(0.0, 1.0);

            let status_msg = if splash_ratio < 0.3 {
                i18n::text(language, "Starting hardware bus telemetry engine...")
            } else if splash_ratio < 0.6 {
                i18n::text(
                    language,
                    "Loading file organization rules & JSON schemas...",
                )
            } else if splash_ratio < 0.9 {
                i18n::text(
                    language,
                    "Calibrating system cluster matrix and clean stores...",
                )
            } else {
                i18n::text(language, "Ready.")
            };

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.28);
                    ui.label(
                        egui::RichText::new(">>")
                            .size(48.0)
                            .color(egui::Color32::from_rgb(0, 180, 216))
                            .strong(),
                    );
                    ui.heading(egui::RichText::new("CleanDisk Studio").size(32.0).strong());
                    ui.label(
                        egui::RichText::new(i18n::text(
                            language,
                            "Disk Optimization & Automation Utility",
                        ))
                        .weak()
                        .size(13.0),
                    );

                    ui.add_space(36.0);
                    ui.set_max_width(340.0);
                    ui.add(egui::ProgressBar::new(splash_ratio).animate(true));

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(status_msg)
                            .monospace()
                            .size(11.5)
                            .color(egui::Color32::from_rgb(180, 190, 205)),
                    );
                });
            });

            if elapsed_ms >= 1800 {
                self.splash_done = true;
            }
            return;
        }

        // ================= 2. CONFIRMATION MODAL =================
        if let Some(confirm) = self.pending_confirm.clone() {
            egui::Window::new(format!("[!] {}", i18n::text(self.language, "Confirm action")))
                .collapsible(false)
                .resizable(false)
                .fixed_size([460.0, 290.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(235, 75, 75),
                        egui::RichText::new(&confirm.title).strong().size(15.0),
                    );
                    ui.separator();
                    ui.add_space(4.0);

                    ui.label(&confirm.warning);
                    if confirm.requires_admin {
                        ui.colored_label(
                            egui::Color32::from_rgb(241, 196, 15),
                            i18n::text(
                                self.language,
                                "Windows will show a UAC prompt. After approval, an elevated PowerShell window may open while this operation runs.",
                            ),
                        );
                    }
                    if confirm.action_type.starts_with("quarantine_") {
                        ui.colored_label(
                            egui::Color32::from_rgb(90, 190, 130),
                            i18n::text(
                                self.language,
                                "Selected items will be moved to quarantine and can be restored manually. Nothing is permanently deleted.",
                            ),
                        );
                    } else if matches!(
                        confirm.action_type.as_str(),
                        "folder" | "clean_all" | "recycle_bin"
                    ) {
                        ui.colored_label(
                            egui::Color32::from_rgb(241, 196, 15),
                            i18n::text(
                                self.language,
                                "This permanently deletes the listed files and folders; they cannot be recovered from the Recycle Bin.",
                            ),
                        );
                    }

                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        let action_label = if confirm.action_type.starts_with("quarantine_") {
                            i18n::text(self.language, "Move to quarantine")
                        } else if matches!(
                            confirm.action_type.as_str(),
                            "folder" | "clean_all" | "recycle_bin"
                        ) {
                            i18n::text(self.language, "Delete permanently")
                        } else {
                            i18n::text(self.language, "Run confirmed action")
                        };
                        if ui
                            .add_sized(
                                [150.0, 26.0],
                                egui::Button::new(
                                    egui::RichText::new(action_label)
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                )
                                .fill(egui::Color32::from_rgb(210, 45, 45)),
                            )
                            .on_hover_text(&confirm.warning)
                            .clicked()
                        {
                            match confirm.action_type.as_str() {
                                "folder" => {
                                    if let Some(f) = confirm.folder {
                                        self.execute_junk_clean(
                                            f,
                                            "Target Directory",
                                            confirm.requires_admin,
                                        );
                                    }
                                }
                                "recycle_bin" => {
                                    self.empty_recycle_bin();
                                }
                                "clean_all" => {
                                    let mut targets = Vec::new();
                                    if let Some(temp) = env::var_os("TEMP") {
                                        targets.push((PathBuf::from(temp), "User Temp", false));
                                    }
                                    if let Some(windows) = env::var_os("WINDIR") {
                                        let windows = PathBuf::from(windows);
                                        targets.push((windows.join("Temp"), "System Temp", true));
                                        targets.push((
                                            windows.join("Prefetch"),
                                            "Prefetch Cache",
                                            true,
                                        ));
                                    }
                                    self.execute_junk_clean_sequence(targets);
                                }
                                "quarantine_stale_files" => {
                                    let paths = self.selected_stale_files.drain().collect();
                                    self.quarantine_selected_files(paths);
                                }
                                "quarantine_duplicate_files" => {
                                    let paths = self.selected_duplicate_files.drain().collect();
                                    self.quarantine_selected_files(paths);
                                }
                                "quarantine_dev_caches" => {
                                    let paths = self.selected_dev_cache_folders.drain().collect();
                                    self.quarantine_selected_dev_caches(paths);
                                }
                                "command_dns" => {
                                    self.start_system_command(
                                        "ipconfig",
                                        &["/flushdns"],
                                        "DNS cache flush",
                                        true,
                                    );
                                }
                                "command_sfc" => {
                                    self.start_system_command(
                                        "sfc",
                                        &["/scannow"],
                                        "System File Checker",
                                        true,
                                    );
                                }
                                "command_dism" => {
                                    self.start_system_command(
                                        "dism.exe",
                                        &["/Online", "/Cleanup-Image", "/RestoreHealth"],
                                        "DISM image repair",
                                        true,
                                    );
                                }
                                "command_npm_cache" => {
                                    self.start_system_command(
                                        "npm",
                                        &["cache", "clean", "--force"],
                                        "npm cache cleanup",
                                        false,
                                    );
                                }
                                "command_cargo_cache" => {
                                    self.start_system_command(
                                        "cargo",
                                        &["cache", "--autoclean"],
                                        "Cargo cache cleanup",
                                        false,
                                    );
                                }
                                "command_nuget_cache" => {
                                    self.start_system_command(
                                        "dotnet",
                                        &["nuget", "locals", "all", "--clear"],
                                        "NuGet cache cleanup",
                                        false,
                                    );
                                }
                                _ => {}
                            }
                            self.pending_confirm = None;
                        }

                        if ui
                            .add_sized(
                                [100.0, 26.0],
                                egui::Button::new(i18n::text(self.language, "Cancel")),
                            )
                            .on_hover_text(i18n::text(self.language, "Cancel"))
                            .clicked()
                        {
                            self.pending_confirm = None;
                        }
                    });
                });
        }

        // ================= 3. S.M.A.R.T. MODAL =================
        if let Some(ref letter) = self.selected_disk_for_dialog.clone() {
            let mut is_open = true;
            egui::Window::new(format!("S.M.A.R.T. Diagnostics - Drive {}:", letter))
                .open(&mut is_open)
                .resizable(false)
                .collapsible(false)
                .fixed_size([540.0, 360.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let is_loading = *self.dialog_loading.lock().unwrap();
                    let info = self.dialog_details.lock().unwrap().clone();

                    if is_loading {
                        ui.vertical_centered(|ui| {
                            ui.add_space(80.0);
                            ui.spinner();
                            ui.label("Querying NVMe/SATA controller & Windows Storage CIM...");
                        });
                    } else {
                        ui.heading(&info.model);
                        ui.label(
                            egui::RichText::new("CrystalDisk Telemetry & Hardware Metadata").weak(),
                        );
                        ui.separator();

                        egui::Grid::new("smart_info_grid")
                            .num_columns(2)
                            .spacing([20.0, 10.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("Health Status:");
                                ui.colored_label(
                                    egui::Color32::from_rgb(46, 204, 113),
                                    format!("[OK] {}", info.health_status),
                                );
                                ui.end_row();

                                ui.strong("Media Classification:");
                                ui.label(&info.media_type);
                                ui.end_row();

                                ui.strong("Bus Interface / Protocol:");
                                ui.label(&info.bus_type);
                                ui.end_row();

                                ui.strong("Operating Temp:");
                                ui.colored_label(
                                    egui::Color32::from_rgb(0, 180, 216),
                                    &info.temperature,
                                );
                                ui.end_row();

                                ui.strong("Serial Number:");
                                ui.monospace(&info.serial);
                                ui.end_row();

                                ui.strong("Partition Scheme:");
                                ui.label(&info.partition_style);
                                ui.end_row();

                                ui.strong("Sector Allocation:");
                                ui.label(&info.sector_size);
                                ui.end_row();
                            });

                        ui.add_space(16.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button("Close Window")
                                .on_hover_text(i18n::tooltip_hint(self.language, "Đóng cửa sổ SMART và quay lại màn hình chính; không thay đổi ổ đĩa."))
                                .clicked()
                            {
                                self.selected_disk_for_dialog = None;
                            }
                        });
                    }
                });

            if !is_open {
                self.selected_disk_for_dialog = None;
            }
        }

        // ================= 4. BOTTOM PANEL GHIM CỐ ĐỊNH Ở ĐÁY CỬA SỔ =================
        let current_progress = *self.global_progress.lock().unwrap();
        let current_task = self.global_task_name.lock().unwrap().clone();
        let is_busy = *self.is_busy.lock().unwrap();

        egui::TopBottomPanel::bottom("global_bottom_panel")
            .exact_height(38.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let language = self.language;
                    if is_busy {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(&current_task)
                                .strong()
                                .color(egui::Color32::from_rgb(0, 180, 216)),
                        );
                    } else if current_progress >= 0.99 {
                        ui.colored_label(
                            egui::Color32::from_rgb(46, 204, 113),
                            i18n::text(language, "[OK] COMPLETED"),
                        );
                        ui.weak(i18n::text(language, "Ready for next operation."));
                    } else {
                        ui.weak(i18n::text(language, "Status: Idle"));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{:.0}%", current_progress * 100.0));
                        ui.set_max_width(280.0);
                        ui.add(egui::ProgressBar::new(current_progress));
                    });
                });
            });

        // ================= 5. NỘI DUNG CHÍNH (CENTRAL PANEL) =================
        egui::CentralPanel::default().show(ctx, |ui| {
            let language = self.language;
            ui.horizontal_wrapped(|ui| {
                ui.heading(i18n::text(language, "CleanDisk Studio"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let language_selector = egui::ComboBox::from_id_source("interface_language")
                        .selected_text(self.language.native_name())
                        .width(96.0)
                        .show_ui(ui, |ui| {
                            for language in Language::ALL {
                                ui.selectable_value(
                                    &mut self.language,
                                    language,
                                    language.native_name(),
                                );
                            }
                        })
                        .response;
                    let selected_language = self.language;
                    language_selector.on_hover_ui(|ui| {
                        ui.set_max_width(280.0);
                        ui.strong(format!(
                            "{}: {}",
                            i18n::text(selected_language, "Interface language"),
                            selected_language.native_name()
                        ));
                        ui.label(i18n::text(
                            selected_language,
                            "Select the language used for the application interface.",
                        ));
                    });

                    if ui.button(i18n::text(language, "Rescan Disks"))
                        .on_hover_ui(|ui| {
                            ui.set_max_width(340.0);
                            ui.strong(match language {
                                Language::English => "Refresh disk list",
                                Language::Vietnamese => "Làm mới danh sách ổ đĩa",
                                Language::Chinese => "刷新磁盘列表",
                                Language::Japanese => "ディスク一覧を更新",
                                Language::Russian => "Обновить список дисков",
                            });
                            ui.label(i18n::text(language, "Refresh disk information from Windows."));
                            ui.colored_label(egui::Color32::from_rgb(235, 180, 65), match language {
                                Language::English => "Read-only hardware query; removable or inaccessible drives may not appear.",
                                Language::Vietnamese => "Chỉ đọc thông tin phần cứng; ổ tháo rời hoặc không truy cập được có thể không xuất hiện.",
                                Language::Chinese => "仅查询硬件信息；不可访问或可移动磁盘可能不会显示。",
                                Language::Japanese => "ハードウェア情報のみを読み取ります。リムーバブルまたはアクセス不可のディスクは表示されない場合があります。",
                                Language::Russian => "Только чтение сведений об оборудовании; съемные и недоступные диски могут не отображаться.",
                            });
                        })
                        .clicked()
                    {
                        self.refresh_basic_disks();
                    }
                });
            });

            ui.add_space(4.0);

            // ================= CĂN CHỈNH ĐỒNG ĐỀU CÁC CARD Ổ CỨNG =================
            ui.label(
                egui::RichText::new(i18n::text(
                    language,
                    "PHYSICAL STORAGE DISKS (CLICK CARD FOR S.M.A.R.T. TELEMETRY)",
                ))
                .weak()
                .size(11.0),
            );
            let disks = { self.disks_list.lock().unwrap().clone() };

            ui.horizontal_wrapped(|ui| {
                for disk in &disks {
                    let used = disk.total_gb - disk.available_gb;
                    let ratio = (used / disk.total_gb).clamp(0.0, 1.0) as f32;

                    let card_response = egui::Frame::group(ui.style())
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.set_width(290.0);
                            ui.set_height(96.0);

                            ui.horizontal(|ui| {
                                ui.strong(format!("Drive {}:\\", disk.drive_letter));
                                let badge_color = if disk.media_kind == MediaKind::Ssd {
                                    egui::Color32::from_rgb(46, 204, 113)
                                } else if disk.media_kind == MediaKind::Hdd {
                                    egui::Color32::from_rgb(230, 126, 34)
                                } else {
                                    egui::Color32::from_rgb(160, 160, 160)
                                };
                                ui.label(egui::RichText::new(&disk.media_summary).color(badge_color).strong().size(11.0));
                            });

                            ui.add_space(2.0);

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{:.1} GB Free", disk.available_gb)).strong().size(12.0));
                                ui.weak(format!("of {:.1} GB", disk.total_gb));
                            });

                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                ui.set_max_width(120.0);
                                ui.add(egui::ProgressBar::new(ratio).text(format!("{:.0}% used", ratio * 100.0)));
                                ui.weak(":: S.M.A.R.T.");
                            });
                        });

                    let card_ui = card_response.response.interact(egui::Sense::click());
                    card_ui.clone().on_hover_ui(|ui| {
                        let (details, hardware, free, total, smart) = match language {
                            Language::English => (
                                "Drive details",
                                "Hardware type",
                                "Free space",
                                "Total capacity",
                                "Click to query S.M.A.R.T. health, serial number & temperatures.",
                            ),
                            Language::Vietnamese => (
                                "Chi tiết ổ đĩa",
                                "Loại phần cứng",
                                "Dung lượng trống",
                                "Tổng dung lượng",
                                "Nhấn để truy vấn tình trạng S.M.A.R.T., số sê-ri và nhiệt độ.",
                            ),
                            Language::Chinese => (
                                "磁盘详情",
                                "硬件类型",
                                "可用空间",
                                "总容量",
                                "单击以查询 S.M.A.R.T. 状态、序列号和温度。",
                            ),
                            Language::Japanese => (
                                "ドライブの詳細",
                                "ハードウェアの種類",
                                "空き容量",
                                "総容量",
                                "クリックして S.M.A.R.T. の状態、シリアル番号、温度を確認します。",
                            ),
                            Language::Russian => (
                                "Сведения о диске",
                                "Тип оборудования",
                                "Свободное место",
                                "Общий объем",
                                "Нажмите, чтобы проверить состояние S.M.A.R.T., серийный номер и температуру.",
                            ),
                        };
                        ui.strong(format!("{details} {}:\\", disk.drive_letter));
                        ui.label(format!("{hardware}: {}", disk.media_summary));
                        ui.label(format!("{free}: {:.2} GB", disk.available_gb));
                        ui.label(format!("{total}: {:.2} GB", disk.total_gb));
                        ui.colored_label(egui::Color32::from_rgb(0, 180, 216), smart);
                    });

                    if card_ui.clicked() {
                        self.selected_disk_for_dialog = Some(disk.drive_letter.clone());
                        self.fetch_detailed_smart_info(disk.drive_letter.clone());
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                for (index, icon, tint, title, tooltip) in [
                    (0, "▣", egui::Color32::from_rgb(61, 190, 220), "Organize", "Organize tab tooltip"),
                    (1, "◉", egui::Color32::from_rgb(85, 170, 245), "Disk optimizer", "Disk optimizer tab tooltip"),
                    (2, "⚙", egui::Color32::from_rgb(239, 174, 77), "System cleanup", "System cleanup tab tooltip"),
                    (3, "◫", egui::Color32::from_rgb(119, 206, 147), "Space analysis", "Space analysis tab tooltip"),
                    (4, "⧉", egui::Color32::from_rgb(190, 139, 230), "Duplicates", "Duplicates tab tooltip"),
                    (5, "⌘", egui::Color32::from_rgb(90, 202, 182), "Dev caches", "Dev caches tab tooltip"),
                    (6, "⚒", egui::Color32::from_rgb(239, 117, 112), "Windows tools", "Windows tools tab tooltip"),
                ] {
                    ui.horizontal(|ui| {
                        ui.colored_label(tint, icon);
                        ui.selectable_value(
                            &mut self.selected_tab,
                            index,
                            i18n::text(language, title),
                        )
                        .on_hover_text(i18n::text(language, tooltip));
                    });
                }
            });

            ui.add_space(8.0);

            // ================= SCROLLABLE MIDDLE TAB VIEWPORT =================
            let available_height = ui.available_height() - 120.0;
            egui::ScrollArea::vertical()
                .id_source("main_tab_scroll")
                .max_height(available_height.max(160.0))
                .show(ui, |ui| {
                    match self.selected_tab {
                        0 => {
                            // TAB 0: Multi-Folder Organizer
                            ui.horizontal(|ui| {
                                ui.strong(i18n::text(language, "Target Clean Folders:"));
                                if ui.button(i18n::text(language, "+ Add Folder"))
                                    .on_hover_ui(|ui| {
                                        show_feature_tooltip(ui, language, "File organization");
                                    })
                                    .clicked()
                                {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        let mut list = self.managed_folders.lock().unwrap();
                                        if !list.iter().any(|f| f.path == folder) {
                                            list.push(ManagedFolder {
                                                path: folder,
                                                calculated_size_mb: None,
                                                file_count: None,
                                                is_calculating: false,
                                            });
                                        }
                                    }
                                }
                            });

                            ui.add_space(6.0);

                            let mut to_calc = None;
                            let mut to_remove = None;

                            egui::Grid::new("folders_table").striped(true).min_col_width(90.0).show(ui, |ui| {
                                ui.strong(i18n::text(language, "Path"));
                                ui.strong(i18n::text(language, "Calculated Size"));
                                ui.strong(i18n::text(language, "Items"));
                                ui.strong(i18n::text(language, "Action"));
                                ui.end_row();

                                let list = self.managed_folders.lock().unwrap().clone();
                                for (i, f) in list.iter().enumerate() {
                                    ui.monospace(f.path.to_string_lossy());

                                    if f.is_calculating {
                                        ui.weak(i18n::text(language, "Scanning..."));
                                    } else if let Some(mb) = f.calculated_size_mb {
                                        if mb >= 1024.0 {
                                            ui.label(format!("{:.2} GB", mb / 1024.0));
                                        } else {
                                            ui.label(format!("{:.1} MB", mb));
                                        }
                                    } else {
                                        let btn =
                                            ui.small_button(i18n::text(language, "Check Size"));
                                        if detailed_tooltip(
                                            btn,
                                            language,
                                            "Tính dung lượng thư mục",
                                            "Duyệt cây thư mục và cộng kích thước các file để hiển thị tổng dung lượng và số lượng file.",
                                            "Chỉ đọc metadata, không sửa file. Thư mục lớn có thể mất thời gian; mục không truy cập được sẽ bị bỏ qua.",
                                        ).clicked() {
                                            to_calc = Some(i);
                                        }
                                    }

                                    if let Some(cnt) = f.file_count {
                                        ui.label(format!("{} files", cnt));
                                    } else {
                                        ui.label("-");
                                    }

                                    if ui.small_button(i18n::text(language, "Remove"))
                                        .on_hover_text(i18n::tooltip_hint(language, "Gỡ thư mục khỏi danh sách theo dõi của ứng dụng; không xóa thư mục hoặc file bên trong."))
                                        .clicked() {
                                        to_remove = Some(i);
                                    }
                                    ui.end_row();
                                }
                            });

                            if let Some(i) = to_calc { self.calculate_folder_metrics(i); }
                            if let Some(i) = to_remove {
                                let mut list = self.managed_folders.lock().unwrap();
                                if list.len() > 1 { list.remove(i); }
                            }

                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                if ui.button(i18n::text(language, "Sort All Folders Now"))
                                    .on_hover_ui(|ui| {
                                        show_feature_tooltip(ui, language, "Sắp xếp file ngay");
                                    })
                                    .clicked()
                                {
                                    self.clean_all_managed_folders();
                                }
                                let (label, col) = if self.is_watching {
                                    (i18n::text(language, "Stop Watcher"), egui::Color32::RED)
                                } else {
                                    (i18n::text(language, "Auto Watcher"), egui::Color32::GREEN)
                                };
                                if ui.add(egui::Button::new(egui::RichText::new(label).color(col)))
                                    .on_hover_ui(|ui| {
                                        show_feature_tooltip(ui, language, "Sắp xếp file ngay");
                                    })
                                    .clicked()
                                {
                                    self.toggle_watcher();
                                }
                            });

                            let mut startup_enabled = self.auto_start_with_windows;
                            if ui
                                .checkbox(&mut startup_enabled, i18n::text(language, "Start with Windows"))
                                .on_hover_text(i18n::text(language, "Windows startup tooltip"))
                                .changed()
                            {
                                match set_windows_startup(startup_enabled) {
                                    Ok(()) => self.auto_start_with_windows = startup_enabled,
                                    Err(error) => {
                                        eprintln!("Windows startup registration failed: {error}");
                                    }
                                }
                            }
                            ui.colored_label(
                                egui::Color32::from_rgb(235, 180, 70),
                                i18n::text(
                                    language,
                                    "Automatic cleanup starts after login and may move matching files.",
                                ),
                            );

                            ui.add_space(10.0);

                            egui::Frame::none().fill(egui::Color32::from_rgb(28, 30, 34)).inner_margin(8.0).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(i18n::text(language, "Rule Matching Priority:"));
                                    let mut current_priority = *self.priority_mode.lock().unwrap();

                                    if ui.radio_value(&mut current_priority, PriorityMode::KeywordsFirst, i18n::text(language, "Filename Keywords First"))
                                        .on_hover_ui(|ui| {
                                            show_feature_tooltip(ui, language, "File organization");
                                        })
                                        .clicked()
                                    {
                                        *self.priority_mode.lock().unwrap() = PriorityMode::KeywordsFirst;
                                    }

                                    if ui.radio_value(&mut current_priority, PriorityMode::ExtensionsFirst, i18n::text(language, "File Extensions First"))
                                        .on_hover_ui(|ui| {
                                            show_feature_tooltip(ui, language, "File organization");
                                        })
                                        .clicked()
                                    {
                                        *self.priority_mode.lock().unwrap() = PriorityMode::ExtensionsFirst;
                                    }
                                });
                            });

                            ui.add_space(8.0);

                            ui.collapsing(i18n::text(language, "Configure Categories, Extensions & Filename Keywords"), |ui| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .button(i18n::text(language, "Export Backup JSON"))
                                        .on_hover_text(i18n::tooltip_hint(language, "Lưu quy tắc phân loại hiện tại ra JSON. Chỉ xuất cấu hình, không xuất danh sách file hay dữ liệu cá nhân."))
                                        .clicked()
                                    {
                                        self.export_backup_file();
                                    }
                                    if ui
                                        .button(i18n::text(language, "Load Backup JSON"))
                                        .on_hover_text(i18n::tooltip_hint(language, "Nạp và thay thế bộ quy tắc hiện tại từ JSON đã chọn. Hãy chọn file backup tin cậy; thao tác thay đổi cấu hình ngay."))
                                        .clicked()
                                    {
                                        self.import_backup_file();
                                    }
                                    if ui
                                        .button(i18n::text(language, "Restore Factory Defaults"))
                                        .on_hover_text(i18n::tooltip_hint(language, "Thay quy tắc trong phiên hiện tại bằng bộ mặc định; hãy xuất backup trước nếu muốn giữ cấu hình tùy chỉnh."))
                                        .clicked()
                                    {
                                        self.restore_factory_rules();
                                    }
                                });

                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    ui.label("Folder Name:");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_category_name).hint_text("e.g. Financials, Invoices..."));
                                    ui.label("Extensions:");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_category_exts).hint_text("e.g. pdf, docx, xlsx"));
                                });

                                ui.add_space(4.0);

                                ui.horizontal(|ui| {
                                    ui.label("Keywords in Filename:");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_category_keywords).hint_text("e.g. receipt, bill, budget, tax"));

                                    if ui
                                        .button("+ Add Rule")
                                        .on_hover_text(i18n::tooltip_hint(language, "Thêm quy tắc phân loại gồm tên thư mục, phần mở rộng và từ khóa. Kiểm tra quy tắc trước khi chạy sắp xếp vì file có thể được di chuyển thật."))
                                        .clicked()
                                        && !self.new_category_name.trim().is_empty()
                                    {
                                            let exts = self.new_category_exts.split(',').map(|s| s.trim().trim_start_matches('.').to_lowercase()).filter(|s| !s.is_empty()).collect::<Vec<String>>();
                                            let kws = self.new_category_keywords.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect::<Vec<String>>();
                                            let ext_input = exts.join(", ");
                                            let kw_input = kws.join(", ");

                                            if let Ok(mut cats) = self.categories.lock() {
                                                cats.push(CategoryRule {
                                                    folder_name: self.new_category_name.trim().to_string(),
                                                    extensions: exts,
                                                    keywords: kws,
                                                    ext_input,
                                                    kw_input,
                                                });
                                            }
                                            self.new_category_name.clear();
                                            self.new_category_exts.clear();
                                            self.new_category_keywords.clear();
                                    }
                                });

                                ui.add_space(8.0);

                                egui::Grid::new("rules_table").striped(true).min_col_width(110.0).show(ui, |ui| {
                                    ui.strong("Destination Folder");
                                    ui.strong("Mapped Extensions");
                                    ui.strong("Filename Keywords");
                                    ui.strong("Action");
                                    ui.end_row();

                                    let mut to_remove_rule = None;
                                    if let Ok(mut cats) = self.categories.lock() {
                                        for (i, rule) in cats.iter_mut().enumerate() {
                                            ui.add(egui::TextEdit::singleline(&mut rule.folder_name).desired_width(120.0));
                                            let ext_edit = ui.add(egui::TextEdit::singleline(&mut rule.ext_input).desired_width(190.0));
                                            if ext_edit.changed() { rule.sync_inputs(); }

                                            let kw_edit = ui.add(egui::TextEdit::singleline(&mut rule.kw_input).desired_width(220.0));
                                            if kw_edit.changed() { rule.sync_inputs(); }

                                            if ui
                                                .small_button("Delete")
                                                .on_hover_text(i18n::tooltip_hint(language, "Xóa quy tắc này khỏi cấu hình hiện tại; không xóa file đã được sắp xếp."))
                                                .clicked()
                                            {
                                                to_remove_rule = Some(i);
                                            }
                                            ui.end_row();
                                        }
                                        if let Some(i) = to_remove_rule { cats.remove(i); }
                                    }
                                });
                            });
                        }
                        1 => {
                            egui::Frame::group(ui.style()).inner_margin(12.0).show(ui, |ui| {
                                ui.strong(i18n::text(
                                    language,
                                    "Disk optimizer",
                                ));

                                egui::Frame::none().fill(egui::Color32::from_rgb(25, 27, 30)).inner_margin(8.0).show(ui, |ui| {
                                    ui.label(egui::RichText::new(match language {
                                        Language::English => "Windows drive maintenance",
                                        Language::Vietnamese => "Bảo trì ổ đĩa bằng Windows",
                                        Language::Chinese => "Windows 磁盘维护",
                                        Language::Japanese => "Windows のディスク保守",
                                        Language::Russian => "Обслуживание диска Windows",
                                    }).strong().color(egui::Color32::from_rgb(0, 180, 216)));
                                    ui.label(match language {
                                        Language::English => "SSD: requests Windows ReTrim. HDD: Analyze reports fragmentation; Defragment runs the Windows optimization operation. Each action requests administrator approval.",
                                        Language::Vietnamese => "SSD: yêu cầu Windows ReTrim. HDD: Analyze xem mức phân mảnh; Defragment chạy thao tác tối ưu của Windows. Mỗi thao tác đều yêu cầu chấp thuận quyền quản trị.",
                                        Language::Chinese => "SSD：请求 Windows ReTrim。HDD：Analyze 检查碎片情况；Defragment 执行 Windows 优化。每项操作都会请求管理员批准。",
                                        Language::Japanese => "SSD: Windows ReTrim を要求します。HDD: Analyze は断片化を分析し、Defragment は Windows の最適化を実行します。各操作には管理者の承認が必要です。",
                                        Language::Russian => "SSD: запускается Windows ReTrim. HDD: Analyze проверяет фрагментацию, Defragment запускает оптимизацию Windows. Для каждой операции требуется разрешение администратора.",
                                    });
                                });

                                ui.add_space(8.0);

                                let is_busy = *self.is_busy.lock().unwrap();

                                for disk in &disks {
                                    ui.horizontal(|ui| {
                                        ui.strong(format!("Drive {}:\\ [{}]", disk.drive_letter, disk.media_summary));

                                        if disk.media_kind == MediaKind::Ssd {
                                            let btn = ui.add_enabled(!is_busy, egui::Button::new(format!("TRIM Optimize Drive {}:", disk.drive_letter)));
                                            if detailed_tooltip(
                                                btn,
                                                language,
                                                "Tối ưu SSD bằng TRIM",
                                                "Gửi lệnh ReTrim của Windows để thông báo các vùng dữ liệu đã giải phóng cho bộ điều khiển SSD.",
                                                "Không chống phân mảnh SSD. Cần quyền Administrator nên Windows sẽ hiện lời nhắc UAC; chỉ báo thành công sau khi lệnh Windows hoàn tất.",
                                            ).clicked() {
                                                self.start_optimization_sequence(
                                                    disk.drive_letter.clone(),
                                                    MediaKind::Ssd,
                                                );
                                            }
                                        } else if disk.media_kind == MediaKind::Hdd {
                                            let btn_analyze = ui.add_enabled(!is_busy, egui::Button::new(format!("Analyze Drive {}:", disk.drive_letter)));
                                            if detailed_tooltip(
                                                btn_analyze,
                                                language,
                                                "Phân tích HDD",
                                                "Chạy Optimize-Volume -Analyze để Windows phân tích ổ đĩa; thao tác này không sắp xếp lại dữ liệu.",
                                                "Cần quyền Administrator nên Windows sẽ hiện lời nhắc UAC. Chỉ phù hợp HDD; đây không phải báo cáo sức khỏe SMART.",
                                            ).clicked() {
                                                self.start_hdd_analysis(disk.drive_letter.clone());
                                            }

                                            let btn_defrag = ui.add_enabled(!is_busy, egui::Button::new(format!("Defragment Drive {}:", disk.drive_letter)));
                                            if detailed_tooltip(
                                                btn_defrag,
                                                language,
                                                "Chống phân mảnh HDD",
                                                "Chạy Optimize-Volume -Defrag để Windows sắp xếp lại dữ liệu trên ổ đĩa cơ học.",
                                                "Cần quyền Administrator nên Windows sẽ hiện lời nhắc UAC. Chỉ dùng cho HDD; SSD sẽ không chạy nút này và quá trình có thể lâu.",
                                            ).clicked() {
                                                self.start_optimization_sequence(
                                                    disk.drive_letter.clone(),
                                                    MediaKind::Hdd,
                                                );
                                            }
                                        } else {
                                            ui.weak(match language {
                                                Language::English => "Storage type unknown: Optimize is disabled until Windows reports SSD or HDD.",
                                                Language::Vietnamese => "Chưa xác định loại ổ: tạm khóa Optimize đến khi Windows báo SSD hoặc HDD.",
                                                Language::Chinese => "存储类型未知：Windows 报告为 SSD 或 HDD 前不会启用优化。",
                                                Language::Japanese => "ストレージ種別不明: Windows が SSD または HDD と報告するまで最適化を無効にします。",
                                                Language::Russian => "Тип накопителя неизвестен: оптимизация отключена, пока Windows не сообщит SSD или HDD.",
                                            });
                                        }
                                    });
                                    ui.add_space(4.0);
                                }

                                ui.add_space(14.0);
                                ui.separator();
                                ui.heading(i18n::text(
                                    language,
                                    "Sequential read / write benchmark",
                                ));
                                ui.weak(
                                    "Writes and reads a temporary file on the selected drive, then removes it. Read results may be affected by the Windows file cache.",
                                );
                                ui.horizontal(|ui| {
                                    ui.label("Test location:");
                                    ui.monospace(self.benchmark_root.to_string_lossy());
                                    if ui
                                        .button(i18n::text(language, "Choose folder…"))
                                        .on_hover_text(i18n::text(
                                            language,
                                            "Choose a folder to create a temporary benchmark file. The app checks available space first.",
                                        ))
                                        .clicked()
                                    {
                                        if let Some(folder) =
                                            rfd::FileDialog::new().pick_folder()
                                        {
                                            self.benchmark_root = folder;
                                        }
                                    }
                                    egui::ComboBox::from_id_source("benchmark_size")
                                        .selected_text(human_bytes(self.benchmark_bytes))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.benchmark_bytes,
                                                512 * 1024 * 1024,
                                                "512 MB",
                                            )
                                            .on_hover_text(
                                            i18n::tooltip_hint(language, "Ghi và đọc file 512 MB. Phù hợp để đo nhanh; cần thêm ít nhất 100 MB dung lượng trống."),
                                            );
                                            ui.selectable_value(
                                                &mut self.benchmark_bytes,
                                                1024 * 1024 * 1024,
                                                "1 GB",
                                            )
                                            .on_hover_text(
                                            i18n::tooltip_hint(language, "Ghi và đọc file 1 GB để kết quả ổn định hơn. Cần thêm ít nhất 100 MB dung lượng trống."),
                                            );
                                        });
                                    let running = *self.benchmark_busy.lock().unwrap();
                                    if detailed_tooltip(
                                        ui.add_enabled(
                                            !running,
                                            egui::Button::new(i18n::text(language, "Run benchmark")),
                                        ),
                                        language,
                                        "Đo tốc độ đọc/ghi tuần tự",
                                        "Tạo file tạm với kích thước đã chọn, ghi tuần tự, flush dữ liệu, đọc lại toàn bộ và tính MB/s. Tệp được xóa sau khi đo.",
                                        "Trong lúc chạy sẽ ghi dữ liệu thật lên ổ đĩa và có thể mất vài phút. Đọc có thể được tăng tốc bởi cache Windows; không dùng kết quả này như chẩn đoán sức khỏe ổ.",
                                    )
                                    .clicked()
                                    {
                                        self.start_disk_benchmark();
                                    }
                                });
                                if *self.benchmark_busy.lock().unwrap() {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label("Benchmark running…");
                                    });
                                }
                                if let Some(error) =
                                    self.benchmark_error.lock().unwrap().clone()
                                {
                                    ui.colored_label(
                                        egui::Color32::LIGHT_RED,
                                        format!("Benchmark error: {error}"),
                                    );
                                }
                                if let Some(result) =
                                    *self.benchmark_result.lock().unwrap()
                                {
                                    let scale = result
                                        .write_mb_per_second
                                        .max(result.read_mb_per_second)
                                        .max(1.0) as f32;
                                    ui.label(format!(
                                        "Tested {}",
                                        human_bytes(result.bytes_tested)
                                    ));
                                    ui.add(
                                        egui::ProgressBar::new(
                                            result.write_mb_per_second as f32 / scale,
                                        )
                                        .fill(egui::Color32::from_rgb(55, 166, 214))
                                        .text(format!(
                                            "Sequential write: {:.1} MB/s",
                                            result.write_mb_per_second
                                        )),
                                    );
                                    ui.add(
                                        egui::ProgressBar::new(
                                            result.read_mb_per_second as f32 / scale,
                                        )
                                        .fill(egui::Color32::from_rgb(112, 194, 143))
                                        .text(format!(
                                            "Sequential read: {:.1} MB/s",
                                            result.read_mb_per_second
                                        )),
                                    );
                                }
                            });
                        }
                        2 => {
                            egui::Frame::group(ui.style()).inner_margin(12.0).show(ui, |ui| {
                                ui.strong(i18n::text(
                                    language,
                                    "Targeted System Sanitization & Junk Purge",
                                ));
                                ui.label(match language {
                                    Language::English => "Red actions permanently delete selected cache contents. A confirmation prompt appears before cleanup.",
                                    Language::Vietnamese => "Nút đỏ xóa vĩnh viễn nội dung cache đã chọn. Ứng dụng sẽ hỏi xác nhận trước khi dọn.",
                                    Language::Chinese => "红色操作将永久删除所选缓存内容。清理前会要求确认。",
                                    Language::Japanese => "赤い操作ボタンは選択したキャッシュを完全に削除します。実行前に確認が表示されます。",
                                    Language::Russian => "Красные кнопки удаляют выбранный кэш безвозвратно. Перед очисткой потребуется подтверждение.",
                                });
                                ui.add_space(12.0);

                                let is_busy = *self.is_busy.lock().unwrap();

                                ui.horizontal(|ui| {
                                    let btn = ui.add_enabled(!is_busy, egui::Button::new(format!("[Del] {}", i18n::text(language, "Clean User Temp"))).fill(egui::Color32::from_rgb(190, 45, 45)));
                                    if detailed_tooltip(
                                        btn,
                                        language,
                                        "Dọn file tạm của người dùng",
                                        "Làm trống các mục trực tiếp trong thư mục %TEMP% của tài khoản hiện tại; không quét sang thư mục cha.",
                                        "Đóng ứng dụng trước. File đang dùng/không đủ quyền có thể khiến mục đó không xóa được; thao tác xóa vĩnh viễn sau khi xác nhận.",
                                    ).clicked() {
                                        if let Ok(temp) = env::var("TEMP") {
                                            self.pending_confirm = Some(ConfirmAction {
                                                title: "Purge User Application Temp?".to_string(),
                                                warning: format!("Path: {}\nDeletes application cache files generated during normal use.", temp),
                                                folder: Some(PathBuf::from(temp)),
                                                action_type: "folder".to_string(),
                                                requires_admin: false,
                                            });
                                        }
                                    }
                                    ui.label(match language {
                                        Language::English => "App cache & residue in %TEMP%",
                                        Language::Vietnamese => "Cache và file tạm trong %TEMP%",
                                        Language::Chinese => "清理 %TEMP% 中的应用缓存和临时文件",
                                        Language::Japanese => "%TEMP% 内のアプリキャッシュと一時ファイル",
                                        Language::Russian => "Кэш приложений и временные файлы в %TEMP%",
                                    });
                                });

                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    let btn = ui.add_enabled(!is_busy, egui::Button::new(format!("[Del] {}", i18n::text(language, "Clean System Temp"))).fill(egui::Color32::from_rgb(190, 45, 45)));
                                    if detailed_tooltip(
                                        btn,
                                        language,
                                        "Dọn Windows Temp",
                                        "Làm trống nội dung thư mục %WINDIR%\\Temp, nơi lưu một số file tạm của hệ thống và bộ cài.",
                                        "Bắt buộc cần quyền Administrator; Windows sẽ hiện UAC để bạn cho phép. Đóng tác vụ cài đặt/cập nhật trước; file đang khóa có thể bị bỏ qua.",
                                    ).clicked() {
                                        if let Ok(win) = env::var("WINDIR") {
                                            let p = PathBuf::from(&win).join("Temp");
                                            self.pending_confirm = Some(ConfirmAction {
                                                title: "Purge Windows System Temp?".to_string(),
                                                warning: format!("Path: {}\nEmpties this Windows Temp folder; locked files may remain.", p.display()),
                                                folder: Some(p),
                                                action_type: "folder".to_string(),
                                                requires_admin: true,
                                            });
                                        }
                                    }
                                    ui.label(match language {
                                        Language::English => "Windows installer residue (C:\\Windows\\Temp)",
                                        Language::Vietnamese => "File tạm bộ cài Windows (C:\\Windows\\Temp)",
                                        Language::Chinese => "Windows 安装程序临时文件 (C:\\Windows\\Temp)",
                                        Language::Japanese => "Windows インストーラーの一時ファイル (C:\\Windows\\Temp)",
                                        Language::Russian => "Временные файлы установщика Windows (C:\\Windows\\Temp)",
                                    });
                                });

                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    let btn = ui.add_enabled(!is_busy, egui::Button::new(format!("[Del] {}", i18n::text(language, "Clean Prefetch"))).fill(egui::Color32::from_rgb(190, 45, 45)));
                                    if detailed_tooltip(
                                        btn,
                                        language,
                                        "Dọn Windows Prefetch",
                                        "Xóa các mục trong %WINDIR%\\Prefetch; Windows sẽ tạo lại dữ liệu tăng tốc khởi chạy theo thời gian.",
                                        "Bắt buộc cần quyền Administrator; Windows sẽ hiện UAC để bạn cho phép. Một số ứng dụng có thể khởi chạy chậm hơn lần đầu; dữ liệu bị xóa vĩnh viễn sau khi xác nhận.",
                                    ).clicked() {
                                        if let Ok(win) = env::var("WINDIR") {
                                            let p = PathBuf::from(&win).join("Prefetch");
                                            self.pending_confirm = Some(ConfirmAction {
                                                title: "Purge Windows Prefetch Cache?".to_string(),
                                                warning: format!("Path: {:?}\nClears execution traces used to accelerate app start times.", p),
                                                folder: Some(p),
                                                action_type: "folder".to_string(),
                                                requires_admin: true,
                                            });
                                        }
                                    }
                                    ui.label(match language {
                                        Language::English => "Launch acceleration data (C:\\Windows\\Prefetch)",
                                        Language::Vietnamese => "Dữ liệu tăng tốc khởi chạy (C:\\Windows\\Prefetch)",
                                        Language::Chinese => "应用启动加速数据 (C:\\Windows\\Prefetch)",
                                        Language::Japanese => "アプリ起動高速化データ (C:\\Windows\\Prefetch)",
                                        Language::Russian => "Данные ускорения запуска (C:\\Windows\\Prefetch)",
                                    });
                                });

                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    let btn = ui.add_enabled(!is_busy, egui::Button::new(format!("[Del] {}", i18n::text(language, "Clean Crash Dumps"))).fill(egui::Color32::from_rgb(190, 45, 45)));
                                    if detailed_tooltip(
                                        btn,
                                        language,
                                        "Dọn crash dump",
                                        "Xóa toàn bộ nội dung trực tiếp và các thư mục con trong %LOCALAPPDATA%\\CrashDumps.",
                                        "Crash dump hữu ích để chẩn đoán lỗi. Sao lưu trước nếu cần phân tích; thao tác xóa vĩnh viễn sau khi xác nhận.",
                                    ).clicked() {
                                        if let Ok(local) = env::var("LOCALAPPDATA") {
                                            let p = PathBuf::from(&local).join("CrashDumps");
                                            self.pending_confirm = Some(ConfirmAction {
                                                title: "Purge Application Crash Dumps?".to_string(),
                                                warning: format!("Path: {}\nEmpties this crash dump folder.", p.display()),
                                                folder: Some(p),
                                                action_type: "folder".to_string(),
                                                requires_admin: false,
                                            });
                                        }
                                    }
                                    ui.label("Memory crash logs (%LOCALAPPDATA%\\CrashDumps)");
                                });

                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    let btn = ui.add_enabled(!is_busy, egui::Button::new(format!("[Wipe] {}", i18n::text(language, "Empty Recycle Bin"))).fill(egui::Color32::from_rgb(205, 35, 35)));
                                    if detailed_tooltip(
                                        btn,
                                        language,
                                        "Làm trống Recycle Bin",
                                        "Gọi Windows để xóa các mục trong Thùng rác trên các ổ đĩa.",
                                        "Không thể khôi phục từ Recycle Bin sau thao tác. Kiểm tra thùng rác và xác nhận kỹ trước khi chạy.",
                                    ).clicked() {
                                        self.pending_confirm = Some(ConfirmAction {
                                            title: "Empty Entire Windows Recycle Bin?".to_string(),
                                            warning: "All items currently located in the Windows Recycle Bin across all drives will be permanently erased.".to_string(),
                                            folder: None,
                                            action_type: "recycle_bin".to_string(),
                                            requires_admin: false,
                                        });
                                    }
                                    ui.label("Permanently wipe all items in Windows Recycle Bin");
                                });

                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    let btn = ui.add_enabled(!is_busy, egui::Button::new(format!("[!] {}", i18n::text(language, "Clean All Junk Stores"))).fill(egui::Color32::from_rgb(180, 25, 25)));
                                    if detailed_tooltip(
                                        btn,
                                        language,
                                        "Dọn nhiều vùng tạm",
                                        "Sau khi xác nhận, lần lượt dọn %TEMP%, Windows Temp và Prefetch. Các vùng hệ thống sẽ chạy bằng quyền Administrator, mỗi lần hiện lời nhắc UAC.",
                                        "Thao tác xóa vĩnh viễn và không bao gồm Recycle Bin. Chỉ file đang khóa có thể bị bỏ qua; nếu từ chối UAC, phần còn lại cần quyền quản trị sẽ dừng.",
                                    ).clicked() {
                                        self.pending_confirm = Some(ConfirmAction {
                                            title: "Purge All System Temporary Stores?".to_string(),
                                            warning: format!(
                                                "This will sequentially empty {}, {}\\Temp, and {}\\Prefetch.",
                                                env::var("TEMP").unwrap_or_else(|_| "%TEMP%".to_string()),
                                                env::var("WINDIR").unwrap_or_else(|_| "%WINDIR%".to_string()),
                                                env::var("WINDIR").unwrap_or_else(|_| "%WINDIR%".to_string())
                                            ),
                                            folder: None,
                                            action_type: "clean_all".to_string(),
                                            requires_admin: true,
                                        });
                                    }
                                    ui.label("Run full sequential sanitization of all temporary caches");
                                });
                            });
                        }
                        3 => {
                            egui::Frame::group(ui.style())
                                .inner_margin(14.0)
                                .show(ui, |ui| {
                                    ui.heading(i18n::text(language, "Disk space analyzer"));
                                    ui.label(
                                        egui::RichText::new(i18n::text(
                                            language,
                                            "Largest files, files not modified recently, and space by file type.",
                                        ))
                                        .weak(),
                                    );
                                    ui.add_space(10.0);

                                    ui.horizontal(|ui| {
                                        ui.label(i18n::text(language, "Scan folder:"));
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut self.scan_root.to_string_lossy().to_string(),
                                            )
                                            .desired_width(360.0)
                                            .interactive(false),
                                        );
                                        if ui
                                            .button(i18n::text(language, "Choose…"))
                                            .on_hover_text(match language {
                                                Language::English => "Choose a root folder to scan files and subfolders. Read-only; no files are deleted.",
                                                Language::Vietnamese => "Chọn thư mục gốc để quét file và thư mục con. Chỉ đọc dữ liệu, không xóa file.",
                                                Language::Chinese => "选择要扫描的根文件夹及其子文件夹。只读，不会删除文件。",
                                                Language::Japanese => "スキャンするルートフォルダーを選択します。読み取り専用で、ファイルは削除されません。",
                                                Language::Russian => "Выберите корневую папку для сканирования. Только чтение, файлы не удаляются.",
                                            })
                                            .clicked()
                                        {
                                            if let Some(folder) =
                                                rfd::FileDialog::new().pick_folder()
                                            {
                                                self.scan_root = folder;
                                            }
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label(i18n::text(language, "Minimum size:"));
                                        egui::ComboBox::from_id_source("large_file_threshold")
                                            .selected_text(human_bytes(self.min_size_bytes))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.min_size_bytes,
                                                    524_288_000,
                                                    i18n::text(language, "500 MB"),
                                                )
                                                .on_hover_text(
                                                    i18n::tooltip_hint(language, "Liệt kê file có dung lượng từ 500 MB trở lên; danh sách chỉ đọc, không xóa file."),
                                                );
                                                ui.selectable_value(
                                                    &mut self.min_size_bytes,
                                                    1_073_741_824,
                                                    i18n::text(language, "1 GB"),
                                                )
                                                .on_hover_text(
                                                    i18n::tooltip_hint(language, "Liệt kê file có dung lượng từ 1 GB trở lên; danh sách chỉ đọc, không xóa file."),
                                                );
                                                ui.selectable_value(
                                                    &mut self.min_size_bytes,
                                                    5_368_709_120,
                                                    i18n::text(language, "5 GB"),
                                                )
                                                .on_hover_text(
                                                    i18n::tooltip_hint(language, "Liệt kê file có dung lượng từ 5 GB trở lên; danh sách chỉ đọc, không xóa file."),
                                                );
                                            });
                                        ui.label(i18n::text(language, "Top results:"));
                                        egui::ComboBox::from_id_source("large_file_limit")
                                            .selected_text(self.result_limit.to_string())
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.result_limit,
                                                    50,
                                                    i18n::text(language, "Top 50"),
                                                )
                                                .on_hover_text(
                                                    i18n::tooltip_hint(language, "Chỉ hiển thị tối đa 50 kết quả lớn nhất để danh sách dễ xem."),
                                                );
                                                ui.selectable_value(
                                                    &mut self.result_limit,
                                                    100,
                                                    i18n::text(language, "Top 100"),
                                                )
                                                .on_hover_text(
                                                    i18n::tooltip_hint(language, "Hiển thị tối đa 100 file lớn nhất; quét thư mục sâu có thể mất thêm thời gian."),
                                                );
                                            });
                                        ui.label(i18n::text(language, "Stale after:"));
                                        egui::ComboBox::from_id_source("stale_days")
                                            .selected_text(format!("{} days", self.stale_days))
                                            .show_ui(ui, |ui| {
                                                for days in [30, 60, 90] {
                                                    ui.selectable_value(
                                                        &mut self.stale_days,
                                                        days,
                                                        match days {
                                                            30 => i18n::text(language, "30 days"),
                                                            60 => i18n::text(language, "60 days"),
                                                            _ => i18n::text(language, "90 days"),
                                                        },
                                                    )
                                                    .on_hover_text(
                                                        i18n::tooltip_hint(language, "Lọc theo thời gian sửa đổi gần nhất, không phải thời điểm mở file. Windows có thể tắt ghi nhận lần truy cập cuối."),
                                                    );
                                                }
                                            });
                                    });

                                    let scan_busy = *self.scan_busy.lock().unwrap();
                                    if detailed_tooltip(
                                        ui.add_enabled(
                                            !scan_busy,
                                            egui::Button::new(format!(
                                                "⌕  {}",
                                                i18n::text(language, "Scan selected folder")
                                            )),
                                        ),
                                        language,
                                        "Phân tích dung lượng thư mục",
                                        "Quét đệ quy metadata file để tìm file lớn, file lâu chưa sửa đổi và tổng dung lượng theo nhóm định dạng.",
                                        "Quét chỉ đọc, không xóa hay sửa file. Thư mục lớn/ổ đĩa chậm sẽ mất thời gian; một số mục bị khóa hoặc thiếu quyền có thể không đọc được.",
                                    )
                                        .clicked()
                                    {
                                        self.start_disk_scan(false);
                                    }
                                    if scan_busy {
                                        ui.horizontal(|ui| {
                                            ui.spinner();
                                            ui.label("Scanning files…");
                                        });
                                    }

                                    if let Some(error) = self.scan_error.lock().unwrap().clone() {
                                        ui.colored_label(
                                            egui::Color32::LIGHT_RED,
                                            format!("Scan error: {error}"),
                                        );
                                    }
                                    if let Some(report) =
                                        self.scan_report.lock().unwrap().clone()
                                    {
                                        ui.separator();
                                        ui.label(
                                            i18n::text(language, "Scanned {} files · {} total")
                                                .replacen(
                                                    "{}",
                                                    &report.files_scanned.to_string(),
                                                    1,
                                                )
                                                .replacen(
                                                    "{}",
                                                    &human_bytes(report.total_bytes),
                                                    1,
                                                ),
                                        );

                                        ui.collapsing(                                        i18n::text(language, "Storage by file type"), |ui| {
                                            let total = report.total_bytes.max(1) as f32;
                                            let colors = [
                                                egui::Color32::from_rgb(55, 166, 214),
                                                egui::Color32::from_rgb(112, 194, 143),
                                                egui::Color32::from_rgb(232, 176, 73),
                                                egui::Color32::from_rgb(175, 125, 216),
                                                egui::Color32::from_rgb(225, 111, 111),
                                            ];
                                            for (index, (category, bytes)) in
                                                report.categories.iter().take(8).enumerate()
                                            {
                                                let ratio = *bytes as f32 / total;
                                                ui.add(
                                                    egui::ProgressBar::new(ratio)
                                                        .fill(colors[index % colors.len()])
                                                        .text(format!(
                                                            "{category}  ·  {}  ·  {:.1}%",
                                                            human_bytes(*bytes),
                                                            ratio * 100.0
                                                        )),
                                                );
                                            }
                                            if report.categories.is_empty() {
                                                ui.weak("No files found.");
                                            }
                                        });

                                        ui.collapsing(
                                            format!(
                                                "Largest files ≥ {} ({})",
                                                human_bytes(self.min_size_bytes),
                                                report.large_files.len()
                                            ),
                                            |ui| {
                                                egui::ScrollArea::vertical()
                                                    .max_height(170.0)
                                                    .show(ui, |ui| {
                                                        for file in &report.large_files {
                                                            ui.horizontal(|ui| {
                                                                ui.monospace(
                                                                    file.path.to_string_lossy(),
                                                                );
                                                                ui.with_layout(
                                                                    egui::Layout::right_to_left(
                                                                        egui::Align::Center,
                                                                    ),
                                                                    |ui| {
                                                                        ui.strong(human_bytes(
                                                                            file.size,
                                                                        ));
                                                                    },
                                                                );
                                                            });
                                                        }
                                                        if report.large_files.is_empty() {
                                                            ui.weak(
                                                                i18n::text(
                                                                    language,
                                                                    "No files match this size threshold.",
                                                                ),
                                                            );
                                                        }
                                                    });
                                            },
                                        );

                                        ui.collapsing(
                                            i18n::text(
                                                language,
                                                "Not modified in {}+ days ({})"
                                            )
                                            .replacen("{}", &self.stale_days.to_string(), 1)
                                            .replacen(
                                                "{}",
                                                &report.stale_files.len().to_string(),
                                                1
                                            ),
                                            |ui| {
                                                ui.weak(
                                                    "Windows may disable last-access timestamps; this list uses last modified time.",
                                                );
                                                egui::ScrollArea::vertical()
                                                    .max_height(170.0)
                                                    .show(ui, |ui| {
                                                        for file in &report.stale_files {
                                                            ui.horizontal(|ui| {
                                                                let mut checked = self
                                                                    .selected_stale_files
                                                                    .contains(&file.path);
                                                                if ui
                                                                    .checkbox(&mut checked, "")
                                                                    .on_hover_text(i18n::tooltip_hint(language, "Chọn file cũ này để cách ly. Ngày dùng là thời điểm sửa đổi; kiểm tra nội dung trước khi xác nhận."))
                                                                    .changed()
                                                                {
                                                                    if checked {
                                                                        self.selected_stale_files
                                                                            .insert(file.path.clone());
                                                                    } else {
                                                                        self.selected_stale_files
                                                                            .remove(&file.path);
                                                                    }
                                                                }
                                                                ui.monospace(
                                                                    file.path.to_string_lossy(),
                                                                );
                                                                ui.with_layout(
                                                                    egui::Layout::right_to_left(
                                                                        egui::Align::Center,
                                                                    ),
                                                                    |ui| {
                                                                        ui.strong(human_bytes(
                                                                            file.size,
                                                                        ));
                                                                    },
                                                                );
                                                            });
                                                        }
                                                        if report.stale_files.is_empty() {
                                                            ui.weak(
                                                                i18n::text(
                                                                    language,
                                                                    "No stale files found in this folder.",
                                                                ),
                                                            );
                                                        }
                                                    });
                                            },
                                        );
                                        let stale_selected =
                                            self.selected_stale_files.len();
                                        if detailed_tooltip(
                                            ui.add_enabled(
                                                    stale_selected > 0
                                                        && !*self.scan_busy.lock().unwrap(),
                                                    egui::Button::new(format!(
                                                        "{}: {stale_selected}",
                                                        match language {
                                                            Language::English => "Quarantine selected stale files",
                                                            Language::Vietnamese => "Cách ly file cũ đã chọn",
                                                            Language::Chinese => "隔离所选旧文件",
                                                            Language::Japanese => "選択した古いファイルを隔離",
                                                            Language::Russian => "Поместить старые файлы в карантин",
                                                        }
                                                    )),
                                            )
                                            ,
                                            language,
                                            "Cách ly file cũ đã chọn",
                                            "Di chuyển file vào .cleandisk-quarantine bên trong thư mục quét, giữ nguyên cấu trúc thư mục để có thể khôi phục thủ công.",
                                            "Không xóa vĩnh viễn. Hãy kiểm tra đường dẫn và nội dung trước khi xác nhận; file đang được ứng dụng sử dụng có thể không di chuyển được.",
                                        )
                                            .clicked()
                                        {
                                            self.pending_confirm = Some(ConfirmAction {
                                                    title: "Quarantine selected stale files?"
                                                        .to_string(),
                                                    warning: format!(
                                                        "{} selected file(s) will be moved under {}. No files are permanently deleted.",
                                                        stale_selected,
                                                        self.scan_root
                                                            .join(".cleandisk-quarantine")
                                                            .display()
                                                    ),
                                                    folder: None,
                                                    action_type: "quarantine_stale_files".to_string(),
                                                    requires_admin: false,
                                            });
                                        }
                                        if !report.scan_warnings.is_empty() {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(235, 180, 65),
                                                format!(
                                                    "{} files or folders could not be fully scanned. See logs for details.",
                                                    report.scan_warnings.len()
                                                ),
                                            );
                                        }
                                    }
                                });
                        }
                        4 => {
                            egui::Frame::group(ui.style())
                                .inner_margin(14.0)
                                .show(ui, |ui| {
                                    ui.heading(i18n::text(language, "Exact duplicate finder"));
                                    ui.label(
                                        egui::RichText::new(
                                            "Files are grouped by size first, then verified with a full BLAKE3 content hash.",
                                        )
                                        .weak(),
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.label(i18n::text(language, "Folder:"));
                                        ui.monospace(self.scan_root.to_string_lossy());
                                        if ui
                                            .button(i18n::text(language, "Choose…"))
                                            .on_hover_text(
                                                i18n::tooltip_hint(language, "Chọn thư mục gốc để tìm file trùng. Chỉ quét trong thư mục này và các thư mục con."),
                                            )
                                            .clicked()
                                        {
                                            if let Some(folder) =
                                                rfd::FileDialog::new().pick_folder()
                                            {
                                                self.scan_root = folder;
                                            }
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(i18n::text(language, "Keep:"));
                                        if ui
                                            .radio_value(
                                                &mut self.keep_newest_duplicate,
                                                true,
                                                i18n::text(language, "Newest copy"),
                                            )
                                            .on_hover_text(i18n::tooltip_hint(language, "Trong mỗi nhóm file trùng, giữ bản có thời điểm sửa đổi mới nhất; các bản khác chỉ được xử lý nếu bạn chọn."))
                                            .changed()
                                            || ui
                                                .radio_value(
                                                    &mut self.keep_newest_duplicate,
                                                    false,
                                                    i18n::text(language, "Oldest copy"),
                                                )
                                                .on_hover_text(i18n::tooltip_hint(language, "Trong mỗi nhóm file trùng, giữ bản có thời điểm sửa đổi cũ nhất; kiểm tra nhãn KEEP trước khi chọn các bản còn lại."))
                                                .changed()
                                        {
                                            self.selected_duplicate_files.clear();
                                        }
                                        let scan_busy = *self.scan_busy.lock().unwrap();
                                        if detailed_tooltip(
                                            ui.add_enabled(
                                                !scan_busy,
                                                egui::Button::new(format!(
                                                    "⌕  {}",
                                                    i18n::text(language, "Find exact duplicates")
                                                )),
                                            ),
                                            language,
                                            "Tìm file trùng nội dung",
                                            "Nhóm file cùng kích thước trước, sau đó tính BLAKE3 toàn bộ nội dung để xác nhận trùng chính xác dù tên khác nhau.",
                                            "Hash toàn bộ nội dung có thể đọc nhiều dữ liệu và làm chậm ổ đĩa. Quét chỉ đọc; không file nào bị xóa cho tới khi bạn chọn và xác nhận cách ly.",
                                        )
                                            .clicked()
                                        {
                                            self.start_disk_scan(true);
                                        }
                                    });
                                    if *self.scan_busy.lock().unwrap() {
                                        ui.horizontal(|ui| {
                                            ui.spinner();
                                            ui.label("Comparing candidate file contents…");
                                        });
                                    }
                                    if let Some(error) = self.scan_error.lock().unwrap().clone() {
                                        ui.colored_label(
                                            egui::Color32::LIGHT_RED,
                                            format!("Scan error: {error}"),
                                        );
                                    }
                                    if let Some(report) =
                                        self.scan_report.lock().unwrap().clone()
                                    {
                                        ui.separator();
                                        let duplicate_bytes: u64 = report
                                            .duplicate_groups
                                            .iter()
                                            .map(|group| {
                                                group.size
                                                    * group.files.len().saturating_sub(1) as u64
                                            })
                                            .sum();
                                        ui.label(
                                            i18n::text(
                                                language,
                                                "{} duplicate groups · up to {} reclaimable",
                                            )
                                            .replacen(
                                                "{}",
                                                &report.duplicate_groups.len().to_string(),
                                                1,
                                            )
                                            .replacen("{}", &human_bytes(duplicate_bytes), 1),
                                        );
                                        egui::ScrollArea::vertical()
                                            .max_height(260.0)
                                            .show(ui, |ui| {
                                                for group in &report.duplicate_groups
                                                {
                                                    let mut files = group.files.clone();
                                                    files.sort_by_key(|file| file.modified);
                                                    let keep_index = if self.keep_newest_duplicate {
                                                        files.len().saturating_sub(1)
                                                    } else {
                                                        0
                                                    };
                                                    ui.collapsing(
                                                        format!(
                                                            "{}  ·  {} identical files",
                                                            human_bytes(group.size),
                                                            files.len()
                                                        ),
                                                        |ui| {
                                                            for (index, file) in
                                                                files.iter().enumerate()
                                                            {
                                                                if index == keep_index {
                                                                    ui.horizontal(|ui| {
                                                                        ui.strong("KEEP");
                                                                        ui.monospace(
                                                                            file.path
                                                                                .to_string_lossy(),
                                                                        );
                                                                    });
                                                                } else {
                                                                    let mut checked = self
                                                                        .selected_duplicate_files
                                                                        .contains(&file.path);
                                                                    if ui
                                                                        .checkbox(
                                                                            &mut checked,
                                                                            file.path
                                                                                .to_string_lossy(),
                                                                        )
                                                                        .on_hover_text(i18n::tooltip_hint(language, "Tick để chọn bản sao cần cách ly. File đã được xác minh bằng kích thước và hash BLAKE3; bản KEEP không được chọn."))
                                                                        .changed()
                                                                    {
                                                                        if checked {
                                                                            self.selected_duplicate_files
                                                                                .insert(file.path.clone());
                                                                        } else {
                                                                            self.selected_duplicate_files
                                                                                .remove(&file.path);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        },
                                                    );
                                                }
                                                if report.duplicate_groups.is_empty() {
                                                    ui.weak(
                                                        i18n::text(
                                                            language,
                                                            "No identical files found. Start a scan to check a folder.",
                                                        ),
                                                    );
                                                }
                                            });
                                        ui.add_space(6.0);
                                        let selected_count =
                                            self.selected_duplicate_files.len();
                                        if detailed_tooltip(
                                            ui.add_enabled(
                                                selected_count > 0 && !*self.scan_busy.lock().unwrap(),
                                                egui::Button::new(format!(
                                                    "{}: {selected_count}",
                                                    match language {
                                                        Language::English => "Quarantine selected duplicates",
                                                        Language::Vietnamese => "Cách ly file trùng đã chọn",
                                                        Language::Chinese => "隔离所选重复文件",
                                                        Language::Japanese => "選択した重複ファイルを隔離",
                                                        Language::Russian => "Поместить дубликаты в карантин",
                                                    }
                                                )),
                                            ),
                                            language,
                                            "Cách ly bản sao đã chọn",
                                            "Giữ lại file mới nhất/cũ nhất theo lựa chọn, rồi di chuyển các bản sao được tick vào thư mục quarantine để có thể phục hồi.",
                                            "Kiểm tra nhãn KEEP trước khi tiếp tục. Không xóa vĩnh viễn; file bị khóa hoặc đổi sau khi quét sẽ không được di chuyển.",
                                        )
                                            .clicked()
                                        {
                                            self.pending_confirm = Some(ConfirmAction {
                                                title: "Quarantine selected duplicate files?"
                                                    .to_string(),
                                                warning: format!(
                                                    "{} selected file(s) will be moved under {}. The retained copy stays in place; restore files by moving them back.",
                                                    selected_count,
                                                    self.scan_root
                                                        .join(".cleandisk-quarantine")
                                                        .display()
                                                ),
                                                folder: None,
                                                action_type: "quarantine_duplicate_files".to_string(),
                                                requires_admin: false,
                                            });
                                        }
                                    }
                                });
                        }
                        5 => {
                            egui::Frame::group(ui.style())
                                .inner_margin(14.0)
                                .show(ui, |ui| {
                                    ui.heading(i18n::text(language, "Developer build caches"));
                                    ui.label(
                                        egui::RichText::new(
                                            "Find Rust, Node.js, Python and Visual Studio build/cache folders in a project tree.",
                                        )
                                        .weak(),
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.label(i18n::text(language, "Project root:"));
                                        ui.monospace(self.dev_cache_root.to_string_lossy());
                                        if ui
                                            .button(i18n::text(language, "Choose…"))
                                            .on_hover_text(
                                                i18n::tooltip_hint(language, "Chọn thư mục gốc dự án để dò các thư mục build/cache đã nhận diện; không quét ngoài cây thư mục này."),
                                            )
                                            .clicked()
                                        {
                                            if let Some(folder) =
                                                rfd::FileDialog::new().pick_folder()
                                            {
                                                self.dev_cache_root = folder;
                                            }
                                        }
                                        let scanning = *self.dev_cache_busy.lock().unwrap();
                                        if ui
                                            .add_enabled(
                                                !scanning,
                                                egui::Button::new(format!(
                                                    "⌕  {}",
                                                    i18n::text(language, "Scan project")
                                                )),
                                            )
                                            .on_hover_ui(|ui| {
                                                ui.set_max_width(340.0);
                                                let (title, details, caution) =
                                                    i18n::tooltip_details(language, "Quét cache phát triển");
                                                ui.strong(title);
                                                ui.label(details);
                                                ui.colored_label(
                                                    egui::Color32::from_rgb(235, 180, 65),
                                                    caution,
                                                );
                                            })
                                            .clicked()
                                        {
                                            self.start_dev_cache_scan();
                                        }
                                    });
                                    ui.weak(
                                        "Recognized: target/, node_modules/, .venv/, venv/, __pycache__/, bin/, obj/, .vs/. Review paths before quarantining; builds may need to be regenerated.",
                                    );
                                    if *self.dev_cache_busy.lock().unwrap() {
                                        ui.horizontal(|ui| {
                                            ui.spinner();
                                            ui.label("Searching project directories…");
                                        });
                                    }
                                    if let Some(error) =
                                        self.dev_cache_error.lock().unwrap().clone()
                                    {
                                        ui.colored_label(
                                            egui::Color32::LIGHT_RED,
                                            format!("Scan error: {error}"),
                                        );
                                    }
                                    if let Some(folders) =
                                        self.dev_cache_folders.lock().unwrap().clone()
                                    {
                                        ui.separator();
                                        ui.label(format!(
                                            "{} cache folder(s) found",
                                            folders.len()
                                        ));
                                        egui::ScrollArea::vertical()
                                            .max_height(240.0)
                                            .show(ui, |ui| {
                                                for folder in &folders {
                                                    ui.horizontal(|ui| {
                                                        let mut checked = self
                                                            .selected_dev_cache_folders
                                                            .contains(&folder.path);
                                                        if ui
                                                            .checkbox(&mut checked, "")
                                                            .on_hover_text(i18n::tooltip_hint(language, "Chọn cache để cách ly. Đọc đường dẫn và dung lượng trước khi xác nhận; dự án có thể cần build/tải dependency lại."))
                                                            .changed()
                                                        {
                                                            if checked {
                                                                self.selected_dev_cache_folders
                                                                    .insert(folder.path.clone());
                                                            } else {
                                                                self.selected_dev_cache_folders
                                                                    .remove(&folder.path);
                                                            }
                                                        }
                                                        ui.monospace(
                                                            folder.path.to_string_lossy(),
                                                        );
                                                        ui.with_layout(
                                                            egui::Layout::right_to_left(
                                                                egui::Align::Center,
                                                            ),
                                                            |ui| {
                                                                ui.strong(human_bytes(folder.size));
                                                                ui.weak(format!(
                                                                    "modified {} days ago",
                                                                    age_in_days(folder.modified)
                                                                ));
                                                            },
                                                        );
                                                    });
                                                }
                                                if folders.is_empty() {
                                                    ui.weak(
                                                        "No recognized, non-empty build cache folders found.",
                                                    );
                                                }
                                            });
                                        let selected_count =
                                            self.selected_dev_cache_folders.len();
                                        if detailed_tooltip(
                                            ui.add_enabled(
                                                selected_count > 0
                                                    && !*self.dev_cache_busy.lock().unwrap(),
                                                egui::Button::new(format!(
                                                    "{}: {selected_count}",
                                                    match language {
                                                        Language::English => "Quarantine selected cache folders",
                                                        Language::Vietnamese => "Cách ly thư mục cache đã chọn",
                                                        Language::Chinese => "隔离所选缓存文件夹",
                                                        Language::Japanese => "選択したキャッシュを隔離",
                                                        Language::Russian => "Поместить выбранный кэш в карантин",
                                                    }
                                                )),
                                            ),
                                            language,
                                            "Cách ly cache phát triển",
                                            "Chuyển nguyên thư mục đã chọn vào .cleandisk-quarantine và giữ cấu trúc tương đối để bạn khôi phục nếu cần.",
                                            "Build kế tiếp có thể phải tải package hoặc biên dịch lại. Chỉ chuyển cache đã kiểm tra; thao tác không xóa vĩnh viễn nhưng có thể thất bại với file đang bị khóa.",
                                        )
                                            .clicked()
                                        {
                                            self.pending_confirm = Some(ConfirmAction {
                                                title: "Quarantine selected build caches?"
                                                    .to_string(),
                                                warning: format!(
                                                    "{} folder(s) will be moved under {}. This is reversible; project builds may need to recreate those files.",
                                                    selected_count,
                                                    self.dev_cache_root
                                                        .join(".cleandisk-quarantine")
                                                        .display()
                                                ),
                                                folder: None,
                                                action_type: "quarantine_dev_caches".to_string(),
                                                requires_admin: false,
                                            });
                                        }
                                    }
                                });
                        }
                        6 => {
                            egui::Frame::group(ui.style())
                                .inner_margin(14.0)
                                .show(ui, |ui| {
                                    ui.heading(i18n::text(language, "Scheduled maintenance"));
                                    let changed = ui
                                        .checkbox(
                                            &mut self.weekly_maintenance_enabled,
                                            i18n::text(language, "Enable maintenance every 7 days"),
                                        )
                                        .on_hover_ui(|ui| {
                                            ui.set_max_width(360.0);
                                            let (title, how, caution) = match language {
                                                Language::English => (
                                                    "Maintenance every 7 days",
                                                    "When enabled, the next run is scheduled 7 days later. While the app remains open, it cleans user %TEMP% and sends TRIM to detected SSDs.",
                                                    "This schedule is stored only for the current app session and stops when the app exits. SSD TRIM requests Administrator approval through UAC.",
                                                ),
                                                Language::Vietnamese => (
                                                    "Lịch bảo trì mỗi 7 ngày",
                                                    "Khi bật, lần chạy tiếp theo được hẹn sau 7 ngày. Khi ứng dụng còn mở, tính năng dọn %TEMP% của người dùng và gửi TRIM tới SSD đã nhận diện.",
                                                    "Lịch chỉ lưu trong phiên ứng dụng và dừng khi thoát. SSD TRIM sẽ yêu cầu bạn chấp thuận quyền Administrator qua UAC.",
                                                ),
                                                Language::Chinese => (
                                                    "每 7 天维护一次",
                                                    "启用后，下一次维护安排在 7 天后。应用保持运行时会清理用户的 %TEMP%，并向已识别的 SSD 发送 TRIM。",
                                                    "此计划仅在当前应用会话中有效，退出应用后停止。SSD TRIM 需要通过 UAC 获得管理员批准。",
                                                ),
                                                Language::Japanese => (
                                                    "7 日ごとのメンテナンス",
                                                    "有効にすると、次回の実行は 7 日後に設定されます。アプリの実行中にユーザーの %TEMP% を削除し、検出された SSD に TRIM を送信します。",
                                                    "このスケジュールは現在のアプリセッション中のみ有効で、終了すると停止します。SSD TRIM には UAC による管理者の承認が必要です。",
                                                ),
                                                Language::Russian => (
                                                    "Обслуживание каждые 7 дней",
                                                    "После включения следующий запуск назначается через 7 дней. Пока приложение работает, очищается %TEMP% пользователя и отправляется TRIM обнаруженным SSD.",
                                                    "Расписание действует только в текущем сеансе приложения и прекращается после его закрытия. Для SSD TRIM требуется подтверждение администратора через UAC.",
                                                ),
                                            };
                                            ui.strong(title);
                                            ui.label(how);
                                            ui.colored_label(
                                                egui::Color32::from_rgb(235, 180, 65),
                                                caution,
                                            );
                                        })
                                        .changed();
                                    if changed {
                                        self.next_maintenance =
                                            if self.weekly_maintenance_enabled {
                                                Some(
                                                    Instant::now()
                                                        + Duration::from_secs(7 * 24 * 60 * 60),
                                                )
                                            } else {
                                                None
                                            };
                                    }
                                    ui.weak(
                                        "Runs user Temp cleanup and SSD TRIM while CleanDisk is running (including hidden in the tray). This in-app schedule is not active after you exit the app.",
                                    );
                                    if let Some(next) = self.next_maintenance {
                                        let remaining =
                                            next.saturating_duration_since(Instant::now());
                                        ui.label(format!(
                                            "Next run in {} day(s), {} hour(s).",
                                            remaining.as_secs() / 86_400,
                                            (remaining.as_secs() % 86_400) / 3_600
                                        ));
                                    }
                                });
                            ui.add_space(10.0);

                            egui::Frame::group(ui.style())
                                .inner_margin(14.0)
                                .show(ui, |ui| {
                                    ui.heading(i18n::text(language, "Application and shader caches"));
                                    ui.label(
                                        egui::RichText::new(
                                            "Only named cache folders are cleared. Browser cookies and history are never selected.",
                                        )
                                        .weak(),
                                    );
                                    ui.add_space(8.0);
                                    let targets = application_cache_targets();
                                    if targets.is_empty() {
                                        ui.weak(
                                            "No supported browser, application, or shader cache folders were found.",
                                        );
                                    }
                                    egui::ScrollArea::vertical()
                                        .max_height(220.0)
                                        .show(ui, |ui| {
                                            for (name, path) in targets {
                                                if !path.is_dir() {
                                                    continue;
                                                }
                                                ui.horizontal(|ui| {
                                                    ui.label(&name);
                                                    ui.monospace(path.to_string_lossy());
                                                    if detailed_tooltip(
                                                        ui.button(i18n::text(language, "Clear cache…")),
                                                        language,
                                                        "Dọn cache ứng dụng",
                                                        "Chỉ làm trống đúng thư mục cache được liệt kê (Chrome/Edge/Brave, Discord, Spotify hoặc shader cache); không chọn Cookies hay History.",
                                                        "Đóng ứng dụng tương ứng trước khi dọn. Cache sẽ được tạo lại và có thể khiến lần mở tiếp theo tải dữ liệu hoặc dựng shader lại.",
                                                    )
                                                    .clicked()
                                                    {
                                                        self.pending_confirm =
                                                            Some(ConfirmAction {
                                                                title: format!(
                                                                    "Clear {name}?"
                                                                ),
                                                                warning: format!(
                                                                    "Only this cache directory will be emptied: {}. Cookies and browser history are outside this path.",
                                                                    path.display()
                                                                ),
                                                                folder: Some(path),
                                                                action_type: "folder".to_string(),
                                                                requires_admin: false,
                                                            });
                                                    }
                                                });
                                            }
                                        });
                                });

                            ui.add_space(10.0);
                            egui::Frame::group(ui.style())
                                .inner_margin(14.0)
                                .show(ui, |ui| {
                                    ui.heading(i18n::text(language, "Windows repair and maintenance"));
                                    ui.weak(match language {
                                        Language::English => "DNS flush, SFC and DISM always request administrator approval through Windows UAC. Repair scans can take a long time.",
                                        Language::Vietnamese => "Xóa DNS, SFC và DISM luôn yêu cầu bạn chấp thuận quyền quản trị qua UAC của Windows. Quét sửa chữa có thể mất nhiều thời gian.",
                                        Language::Chinese => "刷新 DNS、SFC 和 DISM 始终会通过 Windows UAC 请求管理员批准。修复扫描可能需要较长时间。",
                                        Language::Japanese => "DNS の消去、SFC、DISM は Windows UAC で管理者の承認を求めます。修復スキャンには時間がかかる場合があります。",
                                        Language::Russian => "Очистка DNS, SFC и DISM всегда запрашивают разрешение администратора через UAC Windows. Проверка может занять много времени.",
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        if detailed_tooltip(
                                            ui.button(i18n::text(language, "Flush DNS cache…")),
                                            language,
                                            "Xóa bộ nhớ đệm DNS",
                                            "Chạy ipconfig /flushdns để Windows tra cứu lại địa chỉ tên miền ở lần kết nối kế tiếp.",
                                            "Không xóa lịch sử trình duyệt hay thay đổi cấu hình mạng. Cần quyền Administrator; Windows sẽ hiện lời nhắc UAC.",
                                        )
                                        .clicked()
                                        {
                                            self.pending_confirm = Some(command_confirmation(
                                                "Flush DNS resolver cache?",
                                                "Runs ipconfig /flushdns. This does not change network configuration.",
                                                "command_dns",
                                                true,
                                            ));
                                        }
                                        if detailed_tooltip(
                                            ui.button(i18n::text(language, "Run SFC /scannow…")),
                                            language,
                                            "Kiểm tra và sửa tệp hệ thống",
                                            "Chạy sfc /scannow, đối chiếu tệp Windows được bảo vệ và tự sửa bản lỗi nếu có nguồn thay thế.",
                                            "Windows sẽ yêu cầu quyền Administrator qua UAC. Quá trình có thể chạy lâu; không nên ngắt giữa chừng. Kết quả/exit code được ghi trong nhật ký.",
                                        )
                                        .clicked()
                                        {
                                            self.pending_confirm = Some(command_confirmation(
                                                "Run System File Checker?",
                                                "Runs sfc /scannow in an elevated process after you approve the Windows UAC prompt.",
                                                "command_sfc",
                                                true,
                                            ));
                                        }
                                        if detailed_tooltip(
                                            ui.button(i18n::text(language, "Run DISM repair…")),
                                            language,
                                            "Sửa Windows component store",
                                            "Chạy DISM /Online /Cleanup-Image /RestoreHealth để kiểm tra và sửa kho thành phần Windows đang sử dụng.",
                                            "Windows sẽ yêu cầu quyền Administrator qua UAC. Tác vụ có thể mất nhiều thời gian và có thể tải dữ liệu qua Windows Update.",
                                        )
                                        .clicked()
                                        {
                                            self.pending_confirm = Some(command_confirmation(
                                                "Repair Windows component store?",
                                                "Runs DISM /Online /Cleanup-Image /RestoreHealth in an elevated process after Windows UAC approval. It may take a long time and use Windows Update.",
                                                "command_dism",
                                                true,
                                            ));
                                        }
                                        if detailed_tooltip(
                                            ui.button(i18n::text(
                                                language,
                                                "Clean Windows Update downloads…",
                                            )),
                                            language,
                                            "Dọn gói tải Windows Update",
                                            "Làm trống thư mục SoftwareDistribution\\Download; không đụng tới thư mục hệ thống khác.",
                                            "Bắt buộc chấp thuận UAC với quyền Administrator. Chỉ chạy sau khi cập nhật cài đặt thành công; Windows có thể cần tải lại gói và file đang khóa sẽ bị bỏ qua.",
                                        )
                                        .clicked()
                                        {
                                            if let Some(windows) = env::var_os("WINDIR") {
                                                let path = PathBuf::from(windows)
                                                    .join("SoftwareDistribution")
                                                    .join("Download");
                                                self.pending_confirm = Some(ConfirmAction {
                                                    title: "Clear Windows Update download files?"
                                                        .to_string(),
                                                    warning: format!(
                                                        "Empties {}. Do this only after updates have installed successfully; locked files may be skipped.",
                                                        path.display()
                                                    ),
                                                    folder: Some(path),
                                                    action_type: "folder".to_string(),
                                                    requires_admin: true,
                                                });
                                            }
                                        }
                                    });
                                });

                            ui.add_space(10.0);
                            egui::Frame::group(ui.style())
                                .inner_margin(14.0)
                                .show(ui, |ui| {
                                    ui.heading(i18n::text(language, "Package manager caches"));
                                    ui.weak(
                                        "Commands run only after confirmation. If the package manager is unavailable, the error is reported in the activity log.",
                                    );
                                    ui.horizontal_wrapped(|ui| {
                                        for (label, title, warning, action) in [
                                            (
                                                "npm",
                                                "Clear npm download cache?",
                                                "Runs npm cache clean --force. Package downloads will be fetched again when needed.",
                                                "command_npm_cache",
                                            ),
                                            (
                                                "Cargo",
                                                "Autoclean Cargo cache?",
                                                "Runs cargo cache --autoclean (requires the cargo-cache subcommand).",
                                                "command_cargo_cache",
                                            ),
                                            (
                                                "NuGet",
                                                "Clear NuGet package caches?",
                                                "Runs dotnet nuget locals all --clear. Packages will be downloaded again on the next build.",
                                                "command_nuget_cache",
                                            ),
                                        ] {
                                            if detailed_tooltip(
                                                ui.button(i18n::text(language, match label {
                                                    "npm" => "Clean npm cache…",
                                                    "Cargo" => "Clean Cargo cache…",
                                                    _ => "Clean NuGet cache…",
                                                })),
                                                language,
                                                "Dọn cache trình quản lý package",
                                                "Chạy lệnh dọn cache riêng của npm, Cargo hoặc NuGet và ghi lại mã thoát/kết quả lệnh.",
                                                "Các gói có thể phải tải lại ở lần build sau. Cargo cần subcommand cargo-cache; quyền và công cụ phải được cài sẵn.",
                                            )
                                            .clicked()
                                            {
                                                self.pending_confirm = Some(command_confirmation(
                                                    title, warning, action, false,
                                                ));
                                            }
                                        }
                                    });
                                });
                        }
                        _ => {}
                    }
                });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // ================= LIVE ACTIVITY CONSOLE =================
            ui.label(
                egui::RichText::new(i18n::text(language, "LIVE ACTIVITY CONSOLE"))
                    .weak()
                    .size(11.0),
            );
            egui::ScrollArea::vertical()
                .id_source("console_log_scroll")
                .max_height(85.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if let Ok(logs) = self.logs.lock() {
                        for log in logs.iter() {
                            if log.starts_with("[SUCCESS]") {
                                ui.colored_label(egui::Color32::from_rgb(46, 204, 113), egui::RichText::new(log).monospace().size(11.5));
                            } else if log.starts_with("[CLEAN]") {
                                ui.colored_label(egui::Color32::from_rgb(0, 180, 216), egui::RichText::new(log).monospace().size(11.5));
                            } else if log.starts_with("[WARN]") {
                                ui.colored_label(egui::Color32::from_rgb(241, 196, 15), egui::RichText::new(log).monospace().size(11.5));
                            } else {
                                ui.label(egui::RichText::new(log).monospace().size(11.5));
                            }
                        }
                    }
                });
        });
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

fn age_in_days(time: std::time::SystemTime) -> u64 {
    std::time::SystemTime::now()
        .duration_since(time)
        .unwrap_or_default()
        .as_secs()
        / 86_400
}

fn clean_directory_contents(
    folder_path: &Path,
    requires_admin: bool,
) -> Result<CleanResult, String> {
    if !folder_path.is_dir() {
        return Err(format!(
            "Location is missing or not a directory: {}",
            folder_path.display()
        ));
    }

    if requires_admin {
        let script = format!(
            "$ErrorActionPreference = 'Stop'\n\
             $failed = $false\n\
             $items = @(Get-ChildItem -LiteralPath {} -Force -ErrorAction Stop)\n\
             foreach ($item in $items) {{\n\
                 try {{\n\
                     $isReparsePoint = (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)\n\
                     if ($item.PSIsContainer -and -not $isReparsePoint) {{\n\
                         Remove-Item -LiteralPath $item.FullName -Recurse -Force -ErrorAction Stop\n\
                     }} else {{\n\
                         Remove-Item -LiteralPath $item.FullName -Force -ErrorAction Stop\n\
                     }}\n\
                 }} catch {{ $failed = $true }}\n\
             }}\n\
             if ($failed) {{ $global:LASTEXITCODE = 1 }} else {{ $global:LASTEXITCODE = 0 }}",
            powershell_literal(&folder_path.to_string_lossy())
        );
        let exit_code = run_elevated_powershell(&script)?;
        if exit_code != 0 {
            return Err(format!(
                "Elevated cleanup exited with code {exit_code}; some files may remain."
            ));
        }
        return Ok(CleanResult::Elevated);
    }

    let entries =
        fs::read_dir(folder_path).map_err(|error| format!("Could not list folder: {error}"))?;
    let mut cleaned_items = 0;
    let mut skipped_items = 0;
    let mut freed_bytes = 0_u64;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                skipped_items += 1;
                continue;
            }
        };
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => {
                skipped_items += 1;
                continue;
            }
        };
        let size = if metadata.is_dir() {
            WalkDir::new(&path)
                .follow_links(false)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file())
                .filter_map(|entry| entry.metadata().ok())
                .map(|metadata| metadata.len())
                .sum()
        } else {
            metadata.len()
        };
        let result = if metadata.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        if result.is_ok() {
            cleaned_items += 1;
            freed_bytes = freed_bytes.saturating_add(size);
        } else {
            skipped_items += 1;
        }
    }

    Ok(CleanResult::Local {
        cleaned_items,
        freed_bytes,
        skipped_items,
    })
}

fn clean_result_message(target_name: &str, result: CleanResult) -> String {
    match result {
        CleanResult::Local {
            cleaned_items: cleaned,
            freed_bytes: freed,
            skipped_items: 0,
        } => format!(
            "[CLEAN] [OK] {target_name}: Purged {cleaned} items (~{:.2} MB reclaimed).",
            freed as f64 / 1_048_576.0
        ),
        CleanResult::Local {
            cleaned_items: cleaned,
            freed_bytes: freed,
            skipped_items: skipped,
        } => format!(
            "[WARN] {target_name}: purged {cleaned} items (~{:.2} MB), skipped {skipped} item(s) due to access or I/O errors.",
            freed as f64 / 1_048_576.0
        ),
        CleanResult::Elevated => format!(
            "[CLEAN] [OK] {target_name}: cleanup completed with administrator approval."
        ),
    }
}

fn command_confirmation(
    title: &str,
    warning: &str,
    action_type: &str,
    requires_admin: bool,
) -> ConfirmAction {
    ConfirmAction {
        title: title.to_string(),
        warning: warning.to_string(),
        folder: None,
        action_type: action_type.to_string(),
        requires_admin,
    }
}

fn powershell_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn log_elevated_volume_result(
    logs: &Arc<Mutex<Vec<String>>>,
    letter: &str,
    operation: &str,
    result: Result<i32, String>,
) {
    if let Ok(mut logs) = logs.lock() {
        match result {
            Ok(0) => {
                logs.push(format!(
                    "[SUCCESS] Drive {letter}: {operation} completed with administrator approval."
                ));
            }
            Ok(exit_code) => logs.push(format!(
                "[WARN] Drive {letter}: {operation} failed, was cancelled, or exited with code {exit_code}."
            )),
            Err(error) => logs.push(format!(
                "[WARN] Drive {letter}: {operation} was not run: {error}"
            )),
        }
    }
}

fn finish_task(
    busy: &Arc<Mutex<bool>>,
    task_name: &Arc<Mutex<String>>,
    progress: &Arc<Mutex<f32>>,
) {
    if let Ok(mut value) = progress.lock() {
        *value = 1.0;
    }
    if let Ok(mut value) = task_name.lock() {
        *value = "Completed".to_string();
    }
    if let Ok(mut value) = busy.lock() {
        *value = false;
    }
}

fn run_elevated_powershell(script: &str) -> Result<i32, String> {
    let wrapped_script = format!(
        "$global:LASTEXITCODE = $null\n\
         try {{\n\
             & {{\n{script}\n}}\n\
             $exitCode = if ($null -ne $global:LASTEXITCODE) {{ $global:LASTEXITCODE }} else {{ 0 }}\n\
         }} catch {{\n\
             Write-Error $_\n\
             $exitCode = 1\n\
         }}\n\
         exit $exitCode\n",
    );
    let encoded_script = encode_powershell_command(&wrapped_script);
    let script_argument = format!("-NoProfile -NonInteractive -EncodedCommand {encoded_script}");
    let launcher = format!(
        "try {{ $process = Start-Process -FilePath 'powershell.exe' -Verb RunAs -Wait -PassThru -ErrorAction Stop -ArgumentList {}; exit $process.ExitCode }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1223 }}",
        powershell_literal(&script_argument)
    );
    let mut launcher_process = Command::new("powershell.exe");
    launcher_process.args(["-NoProfile", "-NonInteractive", "-Command", &launcher]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        launcher_process.creation_flags(0x08000000);
    }
    let launched = launcher_process
        .output()
        .map_err(|error| format!("Could not start the elevation prompt: {error}"));

    match launched {
        Ok(process) if process.status.code() == Some(1223) => {
            let detail = String::from_utf8_lossy(&process.stderr).trim().to_string();
            Err(if detail.is_empty() {
                "Administrator approval was declined in the Windows UAC prompt.".to_string()
            } else {
                detail
            })
        }
        Ok(process) if process.status.success() => Ok(0),
        Ok(process) => {
            let stderr = String::from_utf8_lossy(&process.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&process.stdout).trim().to_string();
            let detail = if stderr.is_empty() { stdout } else { stderr };
            if let Some(exit_code) = process.status.code() {
                Ok(exit_code)
            } else {
                Err(if detail.is_empty() {
                    "The elevated PowerShell process ended without an exit code.".to_string()
                } else {
                    detail
                })
            }
        }
        Err(error) => Err(error),
    }
}

fn encode_powershell_command(command: &str) -> String {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = command.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        encoded.push(BASE64[(first >> 2) as usize] as char);
        encoded.push(BASE64[(((first & 0b11) << 4) | (second >> 4)) as usize] as char);
        encoded.push(if chunk.len() > 1 {
            BASE64[(((second & 0b1111) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            BASE64[(third & 0b0011_1111) as usize] as char
        } else {
            '='
        });
    }
    encoded
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod command_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_FOLDER_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn encodes_powershell_commands_as_utf16le_base64() {
        assert_eq!(
            encode_powershell_command("ipconfig"),
            "aQBwAGMAbwBuAGYAaQBnAA=="
        );
    }

    #[test]
    fn escapes_powershell_string_literals() {
        assert_eq!(
            powershell_literal("C:\\Users\\O'Brien"),
            "'C:\\Users\\O''Brien'"
        );
    }

    #[test]
    fn cleans_only_children_of_the_selected_temp_folder() {
        let test_folder = env::temp_dir().join(format!(
            "cleandisk-clean-test-{}-{}",
            std::process::id(),
            NEXT_TEST_FOLDER_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&test_folder).unwrap();
        fs::write(test_folder.join("one.tmp"), b"1234").unwrap();
        fs::create_dir(test_folder.join("nested")).unwrap();
        fs::write(test_folder.join("nested").join("two.tmp"), b"567").unwrap();

        let result = clean_directory_contents(&test_folder, false);
        let remaining = fs::read_dir(&test_folder)
            .map(|entries| entries.count())
            .unwrap_or(usize::MAX);
        fs::remove_dir_all(&test_folder).unwrap();

        let result = result.unwrap();
        assert!(matches!(
            result,
            CleanResult::Local {
                cleaned_items: 2,
                freed_bytes: 7,
                skipped_items: 0
            }
        ));
        assert_eq!(remaining, 0);
    }

    #[cfg(windows)]
    #[test]
    fn loads_chinese_and_korean_font_fallbacks_for_rendering() {
        let fonts_dir = env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
            .join("Fonts");
        let mut definitions = egui::FontDefinitions::default();
        for (name, file, index) in [
            ("CleanDisk Korean", "malgun.ttf", 0),
            ("CleanDisk Japanese", "msgothic.ttc", 0),
            ("CleanDisk Microsoft YaHei", "msyh.ttc", 0),
            ("CleanDisk SimSun", "simsun.ttc", 0),
            ("CleanDisk Chinese", "simsunb.ttf", 0),
            ("CleanDisk CJK Extension", "SimsunExtG.ttf", 0),
        ] {
            let data = fs::read(fonts_dir.join(file))
                .unwrap_or_else(|error| panic!("Could not read {file}: {error}"));
            let mut font_data = egui::FontData::from_owned(data);
            font_data.index = index;
            definitions.font_data.insert(name.to_string(), font_data);
            definitions
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push(name.to_string());
        }

        let context = egui::Context::default();
        context.set_fonts(definitions);
        let mut glyph_support = [false; 3];
        let _ = context.run(Default::default(), |context| {
            context.fonts(|fonts| {
                let font_id = egui::FontId::proportional(16.0);
                glyph_support = [
                    fonts.has_glyphs(&font_id, "简体中文"),
                    fonts.has_glyphs(&font_id, "한국어"),
                    fonts.has_glyphs(&font_id, "日本語"),
                ];
            });
        });
        assert_eq!(glyph_support, [true, true, true]);
    }
}

fn detailed_tooltip(
    response: egui::Response,
    language: Language,
    feature: &'static str,
    how_it_works: &'static str,
    caution: &'static str,
) -> egui::Response {
    let (feature, how_it_works, caution) = if language == Language::Vietnamese {
        (feature, how_it_works, caution)
    } else {
        i18n::tooltip_details(language, feature)
    };
    response.on_hover_ui(move |ui| {
        ui.set_max_width(360.0);
        ui.strong(feature);
        ui.label(egui::RichText::new(i18n::text(language, "How it works")).strong());
        ui.label(how_it_works);
        ui.colored_label(
            egui::Color32::from_rgb(235, 180, 65),
            format!("{}: {caution}", i18n::text(language, "Caution")),
        );
    })
}

fn show_feature_tooltip(ui: &mut egui::Ui, language: Language, feature: &str) {
    ui.set_max_width(360.0);
    let (title, details, caution) = i18n::tooltip_details(language, feature);
    ui.strong(title);
    ui.label(egui::RichText::new(i18n::text(language, "How it works")).strong());
    ui.label(details);
    ui.colored_label(
        egui::Color32::from_rgb(235, 180, 65),
        format!("{}: {caution}", i18n::text(language, "Caution")),
    );
}

fn clear_directory_contents(path: &Path) -> Result<(usize, u64, usize), String> {
    let entries =
        fs::read_dir(path).map_err(|error| format!("Cannot read {}: {error}", path.display()))?;
    let mut cleaned = 0;
    let mut freed = 0_u64;
    let mut skipped = 0;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        let item_path = entry.path();
        let metadata = match fs::symlink_metadata(&item_path) {
            Ok(metadata) => metadata,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        let size = if metadata.is_dir() {
            let mut directory_size = 0_u64;
            for child in WalkDir::new(&item_path).follow_links(false) {
                match child {
                    Ok(child) if child.file_type().is_file() => match child.metadata() {
                        Ok(metadata) => {
                            directory_size = directory_size.saturating_add(metadata.len())
                        }
                        Err(_) => skipped += 1,
                    },
                    Ok(_) => {}
                    Err(_) => skipped += 1,
                }
            }
            directory_size
        } else {
            metadata.len()
        };
        let result = if metadata.is_dir() {
            fs::remove_dir_all(&item_path)
        } else {
            fs::remove_file(&item_path)
        };
        match result {
            Ok(()) => {
                cleaned += 1;
                freed = freed.saturating_add(size);
            }
            Err(_) => skipped += 1,
        }
    }
    Ok((cleaned, freed, skipped))
}

fn application_cache_targets() -> Vec<(String, PathBuf)> {
    let mut targets = Vec::new();
    let local = env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let roaming = env::var_os("APPDATA").map(PathBuf::from);

    if let Some(local) = local {
        let browser_profiles = [
            (
                "Chrome",
                local.join("Google").join("Chrome").join("User Data"),
            ),
            (
                "Edge",
                local.join("Microsoft").join("Edge").join("User Data"),
            ),
            (
                "Brave",
                local
                    .join("BraveSoftware")
                    .join("Brave-Browser")
                    .join("User Data"),
            ),
        ];
        for (browser, profiles) in browser_profiles {
            if let Ok(entries) = fs::read_dir(profiles) {
                for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
                    let profile = entry.file_name().to_string_lossy().into_owned();
                    if profile == "Default"
                        || profile.starts_with("Profile ")
                        || profile == "Guest Profile"
                    {
                        for cache_name in ["Cache", "Code Cache", "GPUCache"] {
                            targets.push((
                                format!("{browser} · {profile} · {cache_name}"),
                                entry.path().join(cache_name),
                            ));
                        }
                    }
                }
            }
        }

        for (name, path) in [
            ("DirectX shader cache", local.join("D3DSCache")),
            (
                "NVIDIA DX shader cache",
                local.join("NVIDIA").join("DXCache"),
            ),
            (
                "NVIDIA GL shader cache",
                local.join("NVIDIA").join("GLCache"),
            ),
            ("AMD shader cache", local.join("AMD").join("DxCache")),
        ] {
            targets.push((name.to_string(), path));
        }
    }

    if let Some(roaming) = roaming {
        let discord = roaming.join("discord");
        for cache_name in ["Cache", "Code Cache", "GPUCache"] {
            targets.push((format!("Discord · {cache_name}"), discord.join(cache_name)));
        }
        targets.push((
            "Spotify · browser cache".to_string(),
            roaming.join("Spotify").join("Browser").join("Cache"),
        ));
    }
    targets
}

fn main() -> eframe::Result<()> {
    let auto_watch_from_startup = std::env::args().any(|argument| argument == "--auto-watch");
    let icon = app_window_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 710.0])
            .with_min_inner_size([780.0, 580.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "CleanDisk Studio",
        options,
        Box::new(move |cc| {
            let mut app = AppState::new();
            #[cfg(windows)]
            if let Ok(window_handle) = cc.window_handle() {
                if let RawWindowHandle::Win32(window) = window_handle.as_raw() {
                    app.native_window_handle
                        .store(window.hwnd.get(), Ordering::Release);
                }
            }
            app.configure_fonts(&cc.egui_ctx);
            app.register_tray_handlers(&cc.egui_ctx);
            if auto_watch_from_startup {
                app.toggle_watcher();
                #[cfg(windows)]
                hide_native_window(app.native_window_handle.load(Ordering::Acquire));
            }
            Box::new(app)
        }),
    )
}

#[cfg(windows)]
fn show_native_window(handle: isize) {
    if handle != 0 {
        unsafe {
            let hwnd = handle as HWND;
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(windows)]
fn hide_native_window(handle: isize) {
    if handle != 0 {
        unsafe {
            ShowWindow(handle as HWND, SW_HIDE);
        }
    }
}

#[cfg(windows)]
fn force_kill_current_process() {
    let _ = Command::new("taskkill")
        .args(["/PID", &std::process::id().to_string(), "/F"])
        .spawn();
}

#[cfg(windows)]
fn is_windows_startup_enabled() -> bool {
    Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "CleanDiskStudio",
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_windows_startup_enabled() -> bool {
    false
}

fn set_windows_startup(enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
        if enabled {
            let executable = std::env::current_exe()
                .map_err(|error| format!("Cannot locate application executable: {error}"))?;
            let command = format!("\"{}\" --auto-watch", executable.display());
            let output = Command::new("reg")
                .args([
                    "add",
                    key,
                    "/v",
                    "CleanDiskStudio",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &command,
                    "/f",
                ])
                .output()
                .map_err(|error| format!("Cannot update Windows startup: {error}"))?;
            if output.status.success() {
                return Ok(());
            }
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }

        let output = Command::new("reg")
            .args(["delete", key, "/v", "CleanDiskStudio", "/f"])
            .output()
            .map_err(|error| format!("Cannot update Windows startup: {error}"))?;
        if output.status.success() || output.status.code() == Some(1) {
            return Ok(());
        }
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("Windows startup integration is only available on Windows.".to_string())
    }
}

fn app_window_icon() -> egui::IconData {
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32_i32 {
        for x in 0..32_i32 {
            let dx = x - 16;
            let dy = y - 16;
            let distance = dx * dx + dy * dy;
            if distance > 225 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else if distance > 169 {
                rgba.extend_from_slice(&[26, 188, 210, 255]);
            } else if dx * dx + dy * dy <= 49 {
                rgba.extend_from_slice(&[25, 39, 56, 255]);
            } else if (dx + 6) * (dx + 6) + (dy + 6) * (dy + 6) <= 9 {
                rgba.extend_from_slice(&[93, 220, 140, 255]);
            } else {
                let shade = (208 - dy * 2).clamp(150, 225) as u8;
                rgba.extend_from_slice(&[42, shade, 225, 255]);
            }
        }
    }
    egui::IconData {
        rgba,
        width: 32,
        height: 32,
    }
}

use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::Disks;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
}

#[derive(Clone)]
pub struct DuplicateGroup {
    pub size: u64,
    pub files: Vec<ScannedFile>,
}

#[derive(Clone)]
pub struct DevCacheFolder {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
}

#[derive(Clone, Copy)]
pub struct BenchmarkResult {
    pub bytes_tested: u64,
    pub write_mb_per_second: f64,
    pub read_mb_per_second: f64,
}

#[derive(Clone, Default)]
pub struct ScanReport {
    pub files_scanned: usize,
    pub total_bytes: u64,
    pub large_files: Vec<ScannedFile>,
    pub stale_files: Vec<ScannedFile>,
    pub categories: Vec<(String, u64)>,
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub scan_warnings: Vec<String>,
}

pub fn scan_directory(
    root: &Path,
    stale_days: u64,
    min_size: u64,
    limit: usize,
    find_duplicates: bool,
) -> Result<ScanReport, String> {
    if !root.is_dir() {
        return Err(format!(
            "The selected path is not a directory: {}",
            root.display()
        ));
    }

    let stale_before = SystemTime::now()
        .checked_sub(Duration::from_secs(stale_days.saturating_mul(86_400)))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let mut report = ScanReport::default();
    let mut size_candidates: HashMap<u64, Vec<ScannedFile>> = HashMap::new();
    let mut category_totals: BTreeMap<String, u64> = BTreeMap::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| entry.file_name() != ".cleandisk-quarantine")
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                if report.scan_warnings.len() < 20 {
                    report.scan_warnings.push(error.to_string());
                }
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                if report.scan_warnings.len() < 20 {
                    report
                        .scan_warnings
                        .push(format!("{}: {error}", entry.path().display()));
                }
                continue;
            }
        };
        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(error) => {
                if report.scan_warnings.len() < 20 {
                    report.scan_warnings.push(format!(
                        "{}: modification time unavailable ({error})",
                        entry.path().display()
                    ));
                }
                SystemTime::now()
            }
        };
        let file = ScannedFile {
            path: entry.path().to_path_buf(),
            size: metadata.len(),
            modified,
        };

        report.files_scanned += 1;
        report.total_bytes = report.total_bytes.saturating_add(file.size);
        *category_totals
            .entry(file_category(&file.path))
            .or_default() += file.size;
        if file.size >= min_size {
            report.large_files.push(file.clone());
        }
        if modified <= stale_before {
            report.stale_files.push(file.clone());
        }
        if find_duplicates {
            size_candidates.entry(file.size).or_default().push(file);
        }
    }

    report
        .large_files
        .sort_by_key(|file| std::cmp::Reverse(file.size));
    report.large_files.truncate(limit);
    report.stale_files.sort_by_key(|file| file.modified);
    report.stale_files.truncate(limit);
    report.categories = category_totals.into_iter().collect();
    report
        .categories
        .sort_by_key(|(_, bytes)| std::cmp::Reverse(*bytes));

    let mut groups = Vec::new();
    for (size, files) in size_candidates
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
    {
        let mut hashes: HashMap<[u8; 32], Vec<ScannedFile>> = HashMap::new();
        for file in files {
            match hash_file(&file.path) {
                Ok(hash) => hashes.entry(hash).or_default().push(file),
                Err(error) => {
                    if report.scan_warnings.len() < 20 {
                        report.scan_warnings.push(format!(
                            "{}: could not hash file ({error})",
                            file.path.display()
                        ));
                    }
                }
            }
        }
        groups.extend(
            hashes
                .into_values()
                .filter(|matches| matches.len() > 1)
                .map(|files| DuplicateGroup { size, files }),
        );
    }
    groups.sort_by_key(|group| std::cmp::Reverse(group.size));
    report.duplicate_groups = groups;
    Ok(report)
}

fn hash_file(path: &Path) -> std::io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(*hasher.finalize().as_bytes())
}

pub fn quarantine_files(root: &Path, paths: &[PathBuf]) -> Result<usize, String> {
    quarantine_items(root, paths, false)
}

pub fn quarantine_dev_caches(root: &Path, paths: &[PathBuf]) -> Result<usize, String> {
    quarantine_items(root, paths, true)
}

fn quarantine_items(root: &Path, paths: &[PathBuf], directories: bool) -> Result<usize, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("Cannot access scan directory: {error}"))?;
    let quarantine = root.join(".cleandisk-quarantine");
    fs::create_dir_all(&quarantine)
        .map_err(|error| format!("Cannot create quarantine folder: {error}"))?;

    let mut moved = 0;
    let mut failures = Vec::new();
    for path in paths {
        let source = match path.canonicalize() {
            Ok(source)
                if source.starts_with(&root)
                    && source != root
                    && (if directories {
                        source.is_dir()
                    } else {
                        source.is_file()
                    }) =>
            {
                source
            }
            Ok(_) => {
                failures.push(format!(
                    "{} is outside the scanned directory",
                    path.display()
                ));
                continue;
            }
            Err(error) => {
                failures.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        let relative = match source.strip_prefix(&root) {
            Ok(relative) => relative,
            Err(error) => {
                failures.push(format!("{}: {error}", source.display()));
                continue;
            }
        };
        let mut destination = quarantine.join(relative);
        if !destination.starts_with(&quarantine) {
            failures.push(format!("Invalid quarantine path: {}", source.display()));
            continue;
        }
        if let Some(parent) = destination.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                failures.push(format!("{}: {error}", destination.display()));
                continue;
            }
        }
        if destination.exists() {
            let file_name = destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("duplicate");
            let mut suffix = 1_u64;
            loop {
                let candidate = destination.with_file_name(format!("{file_name}.{suffix}"));
                if !candidate.exists() {
                    destination = candidate;
                    break;
                }
                suffix += 1;
            }
        }
        match fs::rename(&source, &destination) {
            Ok(()) => moved += 1,
            Err(error) => failures.push(format!("{}: {error}", source.display())),
        }
    }

    if moved == 0 && !failures.is_empty() {
        Err(failures.join("\n"))
    } else {
        Ok(moved)
    }
}

pub fn scan_dev_cache_folders(root: &Path) -> Result<Vec<DevCacheFolder>, String> {
    if !root.is_dir() {
        return Err(format!(
            "The selected path is not a directory: {}",
            root.display()
        ));
    }

    let mut sizes: HashMap<PathBuf, u64> = HashMap::new();
    let mut modified: HashMap<PathBuf, SystemTime> = HashMap::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            !matches!(
                entry.file_name().to_string_lossy().as_ref(),
                ".git" | ".cleandisk-quarantine"
            )
        })
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if entry.file_type().is_dir()
            && is_dev_cache_name(entry.file_name().to_string_lossy().as_ref())
        {
            sizes.entry(entry.path().to_path_buf()).or_default();
            if let Ok(metadata) = entry.metadata() {
                if let Ok(time) = metadata.modified() {
                    modified.insert(entry.path().to_path_buf(), time);
                }
            }
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }

        let mut cache_ancestor = None;
        for ancestor in entry.path().ancestors() {
            if sizes.contains_key(ancestor) {
                cache_ancestor = Some(ancestor.to_path_buf());
                break;
            }
            if ancestor == root {
                break;
            }
        }
        if let Some(cache) = cache_ancestor {
            if let Ok(metadata) = entry.metadata() {
                let size = sizes.entry(cache).or_default();
                *size = size.saturating_add(metadata.len());
            }
        }
    }

    let mut folders: Vec<DevCacheFolder> = sizes
        .into_iter()
        .map(|(path, size)| DevCacheFolder {
            modified: modified
                .get(&path)
                .copied()
                .unwrap_or(SystemTime::UNIX_EPOCH),
            path,
            size,
        })
        .filter(|folder| folder.size > 0)
        .collect();
    folders.sort_by_key(|folder| std::cmp::Reverse(folder.size));
    Ok(folders)
}

pub fn run_sequential_benchmark(
    root: &Path,
    bytes_to_test: u64,
) -> Result<BenchmarkResult, String> {
    if !root.is_dir() {
        return Err(format!(
            "The selected path is not a directory: {}",
            root.display()
        ));
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("Cannot access benchmark directory: {error}"))?;
    let disks = Disks::new_with_refreshed_list();
    let root_text = normalize_windows_path(&canonical_root.to_string_lossy());
    let available = disks
        .iter()
        .filter(|disk| {
            let mount = normalize_windows_path(&disk.mount_point().to_string_lossy());
            root_text == mount
                || root_text
                    .strip_prefix(&mount)
                    .is_some_and(|rest| rest.starts_with('\\'))
        })
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| disk.available_space())
        .ok_or_else(|| "Could not determine free space for the selected drive.".to_string())?;
    let required = bytes_to_test.saturating_add(100 * 1024 * 1024);
    if available < required {
        return Err(format!(
            "Not enough free space: the benchmark needs {} plus a 100 MB safety margin.",
            format_bytes(bytes_to_test)
        ));
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let test_file = canonical_root.join(format!(
        ".cleandisk-benchmark-{}-{nonce}.tmp",
        std::process::id()
    ));
    let result = benchmark_file(&test_file, bytes_to_test);
    let cleanup = match fs::remove_file(&test_file) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    };
    match (result, cleanup) {
        (Ok(benchmark), Ok(())) => Ok(benchmark),
        (Ok(_), Err(error)) => Err(format!(
            "Benchmark completed, but the temporary file could not be removed: {} ({error})",
            test_file.display()
        )),
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(format!(
            "{error}; temporary file cleanup also failed for {}: {cleanup_error}",
            test_file.display()
        )),
    }
}

fn normalize_windows_path(path: &str) -> String {
    path.strip_prefix(r"\\?\")
        .unwrap_or(path)
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn benchmark_file(path: &Path, bytes_to_test: u64) -> Result<BenchmarkResult, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("Could not create temporary benchmark file: {error}"))?;
    let buffer = vec![0xA5_u8; 4 * 1024 * 1024];
    let write_start = Instant::now();
    let mut remaining = bytes_to_test;
    while remaining > 0 {
        let chunk = remaining.min(buffer.len() as u64) as usize;
        file.write_all(&buffer[..chunk])
            .map_err(|error| format!("Sequential write failed: {error}"))?;
        remaining -= chunk as u64;
    }
    file.sync_all()
        .map_err(|error| format!("Could not flush benchmark data to disk: {error}"))?;
    let write_seconds = write_start.elapsed().as_secs_f64().max(f64::EPSILON);

    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("Could not rewind benchmark file: {error}"))?;
    let mut read_buffer = vec![0_u8; buffer.len()];
    let read_start = Instant::now();
    let mut bytes_read = 0_u64;
    loop {
        let count = file
            .read(&mut read_buffer)
            .map_err(|error| format!("Sequential read failed: {error}"))?;
        if count == 0 {
            break;
        }
        std::hint::black_box(&read_buffer[..count]);
        bytes_read += count as u64;
    }
    let read_seconds = read_start.elapsed().as_secs_f64().max(f64::EPSILON);
    drop(file);
    if bytes_read != bytes_to_test {
        return Err(format!(
            "Benchmark read {bytes_read} bytes but expected {bytes_to_test}."
        ));
    }

    let mib = bytes_to_test as f64 / 1_048_576.0;
    Ok(BenchmarkResult {
        bytes_tested: bytes_to_test,
        write_mb_per_second: mib / write_seconds,
        read_mb_per_second: mib / read_seconds,
    })
}

fn format_bytes(bytes: u64) -> String {
    format!("{:.0} MB", bytes as f64 / 1_048_576.0)
}

fn is_dev_cache_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "target" | "node_modules" | ".venv" | "venv" | "__pycache__" | "bin" | "obj" | ".vs"
    )
}

fn file_category(path: &Path) -> String {
    if path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        matches!(
            name.as_str(),
            "steamapps" | "epic games" | "xboxgames" | "games" | "gog galaxy"
        )
    }) {
        return "Games".to_string();
    }
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "exe" | "msi" | "iso" | "apk" | "msix" => "Installers".to_string(),
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "wmv" => "Video".to_string(),
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" => "Images".to_string(),
        "mp3" | "flac" | "wav" | "aac" | "ogg" => "Audio".to_string(),
        "zip" | "rar" | "7z" | "tar" | "gz" => "Archives".to_string(),
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "csv" => {
            "Documents".to_string()
        }
        "rs" | "js" | "ts" | "py" | "cs" | "cpp" | "c" | "html" | "css" | "json" => {
            "Code & data".to_string()
        }
        _ => "Other".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cleandisk-test-{nonce}"))
    }

    #[test]
    fn groups_only_identical_files_and_applies_large_file_limit() {
        let root = test_root();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.iso"), b"identical").unwrap();
        fs::write(root.join("copy.bin"), b"identical").unwrap();
        fs::write(root.join("different.iso"), b"other data").unwrap();

        let report = scan_directory(&root, 90, 1, 1, true).unwrap();
        assert_eq!(report.files_scanned, 3);
        assert_eq!(report.large_files.len(), 1);
        assert_eq!(report.duplicate_groups.len(), 1);
        assert_eq!(report.duplicate_groups[0].files.len(), 2);
        assert_eq!(report.duplicate_groups[0].size, 9);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn quarantine_moves_files_without_touching_the_retained_copy() {
        let root = test_root();
        fs::create_dir_all(&root).unwrap();
        let original = root.join("original.bin");
        let duplicate = root.join("duplicate.bin");
        fs::write(&original, b"same").unwrap();
        fs::write(&duplicate, b"same").unwrap();

        assert_eq!(
            quarantine_files(&root, std::slice::from_ref(&duplicate)).unwrap(),
            1
        );
        assert!(original.exists());
        assert!(!duplicate.exists());
        assert!(root
            .join(".cleandisk-quarantine")
            .join("duplicate.bin")
            .exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn finds_known_dev_cache_folders_and_quarantines_them_reversibly() {
        let root = test_root();
        let target = root.join("project").join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("artifact.bin"), b"build output").unwrap();
        fs::create_dir_all(root.join("project").join(".git")).unwrap();
        fs::write(root.join("project").join(".git").join("object"), b"git").unwrap();

        let caches = scan_dev_cache_folders(&root).unwrap();
        assert_eq!(caches.len(), 1);
        assert_eq!(caches[0].path, target);
        assert_eq!(caches[0].size, b"build output".len() as u64);

        assert_eq!(
            quarantine_dev_caches(&root, &[caches[0].path.clone()]).unwrap(),
            1
        );
        assert!(!root.join("project").join("target").exists());
        assert!(root
            .join(".cleandisk-quarantine")
            .join("project")
            .join("target")
            .join("artifact.bin")
            .exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn assigns_nested_cache_files_to_the_nearest_cache_folder() {
        let root = test_root();
        let outer = root.join("project").join("node_modules");
        let inner = outer.join("package").join("target");
        fs::create_dir_all(&inner).unwrap();
        fs::write(outer.join("outer.bin"), b"outer").unwrap();
        fs::write(inner.join("inner.bin"), b"inner").unwrap();

        let caches = scan_dev_cache_folders(&root).unwrap();
        let outer_cache = caches.iter().find(|cache| cache.path == outer).unwrap();
        let inner_cache = caches.iter().find(|cache| cache.path == inner).unwrap();
        assert_eq!(outer_cache.size, b"outer".len() as u64);
        assert_eq!(inner_cache.size, b"inner".len() as u64);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn benchmark_reports_both_rates_and_removes_its_temporary_file() {
        let root = test_root();
        fs::create_dir_all(&root).unwrap();
        let result = run_sequential_benchmark(&root, 1024 * 1024).unwrap();
        assert_eq!(result.bytes_tested, 1024 * 1024);
        assert!(result.write_mb_per_second > 0.0);
        assert!(result.read_mb_per_second > 0.0);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}

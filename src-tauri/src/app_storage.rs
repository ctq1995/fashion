use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const CONFIG_FILE_NAME: &str = "storage-preferences.json";
const SNAPSHOT_FILE_NAME: &str = "persistence-state.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoragePreferencesFile {
    data_directory: Option<String>,
    download_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePreferences {
    pub data_directory: Option<String>,
    pub download_directory: Option<String>,
    pub effective_data_directory: String,
    pub effective_download_directory: String,
    pub uses_default_data_directory: bool,
    pub uses_default_download_directory: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistenceSnapshotFile {
    entries: HashMap<String, String>,
    updated_at: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistenceBootstrap {
    pub entries: HashMap<String, String>,
    pub preferences: StoragePreferences,
}

fn storage_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn ensure_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| error.to_string())
}

fn normalize_optional_path(path: Option<String>) -> Option<String> {
    path.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn parse_custom_directory(path: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(path);
    if !candidate.is_absolute() {
        return Err("Directory must be an absolute path.".to_string());
    }
    Ok(candidate)
}

fn config_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|error| error.to_string())
}

fn config_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(config_directory(app)?.join(CONFIG_FILE_NAME))
}

fn default_data_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

fn default_download_directory(app: &AppHandle) -> Result<PathBuf, String> {
    match app.path().download_dir() {
        Ok(path) => Ok(path),
        Err(_) => default_data_directory(app).map(|path| path.join("Downloads")),
    }
}

fn read_preferences_file(app: &AppHandle) -> Result<StoragePreferencesFile, String> {
    let path = config_file_path(app)?;
    if !path.exists() {
        return Ok(StoragePreferencesFile::default());
    }

    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn write_preferences_file(
    app: &AppHandle,
    preferences: &StoragePreferencesFile,
) -> Result<(), String> {
    let dir = config_directory(app)?;
    ensure_directory(&dir)?;

    let raw = serde_json::to_string_pretty(preferences).map_err(|error| error.to_string())?;
    fs::write(dir.join(CONFIG_FILE_NAME), raw).map_err(|error| error.to_string())
}

fn resolve_data_directory(
    app: &AppHandle,
    preferences: &StoragePreferencesFile,
) -> Result<PathBuf, String> {
    match preferences.data_directory.as_deref() {
        Some(path) => parse_custom_directory(path),
        None => default_data_directory(app),
    }
}

fn resolve_download_directory(
    app: &AppHandle,
    preferences: &StoragePreferencesFile,
) -> Result<PathBuf, String> {
    match preferences.download_directory.as_deref() {
        Some(path) => parse_custom_directory(path),
        None => default_download_directory(app),
    }
}

fn snapshot_file_path(
    app: &AppHandle,
    preferences: &StoragePreferencesFile,
) -> Result<PathBuf, String> {
    Ok(resolve_data_directory(app, preferences)?.join(SNAPSHOT_FILE_NAME))
}

fn read_snapshot_file(
    app: &AppHandle,
    preferences: &StoragePreferencesFile,
) -> Result<PersistenceSnapshotFile, String> {
    let path = snapshot_file_path(app, preferences)?;
    if !path.exists() {
        return Ok(PersistenceSnapshotFile::default());
    }

    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn write_snapshot_file(
    app: &AppHandle,
    preferences: &StoragePreferencesFile,
    entries: &HashMap<String, String>,
) -> Result<(), String> {
    let data_directory = resolve_data_directory(app, preferences)?;
    ensure_directory(&data_directory)?;

    let snapshot = PersistenceSnapshotFile {
        entries: entries.clone(),
        updated_at: now_unix_ms(),
    };
    let raw = serde_json::to_string_pretty(&snapshot).map_err(|error| error.to_string())?;
    fs::write(data_directory.join(SNAPSHOT_FILE_NAME), raw).map_err(|error| error.to_string())
}

fn to_storage_preferences(
    app: &AppHandle,
    preferences: StoragePreferencesFile,
) -> Result<StoragePreferences, String> {
    let effective_data_directory = resolve_data_directory(app, &preferences)?;
    let effective_download_directory = resolve_download_directory(app, &preferences)?;
    let uses_default_data_directory = preferences.data_directory.is_none();
    let uses_default_download_directory = preferences.download_directory.is_none();

    Ok(StoragePreferences {
        uses_default_data_directory,
        uses_default_download_directory,
        data_directory: preferences.data_directory,
        download_directory: preferences.download_directory,
        effective_data_directory: effective_data_directory.display().to_string(),
        effective_download_directory: effective_download_directory.display().to_string(),
    })
}

pub fn current_download_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let _guard = storage_lock().lock().map_err(|error| error.to_string())?;
    let preferences = read_preferences_file(app)?;
    resolve_download_directory(app, &preferences)
}

#[tauri::command]
pub fn load_persistence_bootstrap(app: AppHandle) -> Result<PersistenceBootstrap, String> {
    let _guard = storage_lock().lock().map_err(|error| error.to_string())?;
    let preferences = read_preferences_file(&app)?;
    let entries = read_snapshot_file(&app, &preferences)?.entries;

    Ok(PersistenceBootstrap {
        entries,
        preferences: to_storage_preferences(&app, preferences)?,
    })
}

#[tauri::command]
pub fn save_persistence_snapshot(
    app: AppHandle,
    entries: HashMap<String, String>,
) -> Result<StoragePreferences, String> {
    let _guard = storage_lock().lock().map_err(|error| error.to_string())?;
    let preferences = read_preferences_file(&app)?;
    write_snapshot_file(&app, &preferences, &entries)?;
    to_storage_preferences(&app, preferences)
}

#[tauri::command]
pub fn write_persistence_entry(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let _guard = storage_lock().lock().map_err(|error| error.to_string())?;
    let preferences = read_preferences_file(&app)?;
    let mut snapshot = read_snapshot_file(&app, &preferences)?;
    snapshot.entries.insert(key, value);
    write_snapshot_file(&app, &preferences, &snapshot.entries)
}

#[tauri::command]
pub fn set_data_directory(
    app: AppHandle,
    path: Option<String>,
    entries: HashMap<String, String>,
) -> Result<StoragePreferences, String> {
    let _guard = storage_lock().lock().map_err(|error| error.to_string())?;
    let mut preferences = read_preferences_file(&app)?;
    preferences.data_directory = normalize_optional_path(path);

    if let Some(custom_directory) = preferences.data_directory.as_deref() {
        parse_custom_directory(custom_directory)?;
    }

    write_snapshot_file(&app, &preferences, &entries)?;
    write_preferences_file(&app, &preferences)?;
    to_storage_preferences(&app, preferences)
}

#[tauri::command]
pub fn set_download_directory(
    app: AppHandle,
    path: Option<String>,
) -> Result<StoragePreferences, String> {
    let _guard = storage_lock().lock().map_err(|error| error.to_string())?;
    let mut preferences = read_preferences_file(&app)?;
    preferences.download_directory = normalize_optional_path(path);

    if let Some(custom_directory) = preferences.download_directory.as_deref() {
        let directory = parse_custom_directory(custom_directory)?;
        ensure_directory(&directory)?;
    }

    write_preferences_file(&app, &preferences)?;
    to_storage_preferences(&app, preferences)
}

#[tauri::command]
pub fn pick_folder(start_directory: Option<String>) -> Result<Option<String>, String> {
    let mut dialog = FileDialog::new();
    if let Some(start_directory) = normalize_optional_path(start_directory) {
        dialog = dialog.set_directory(PathBuf::from(start_directory));
    }

    Ok(dialog.pick_folder().map(|path| path.display().to_string()))
}

use lofty::{
    config::ParseOptions,
    file::{AudioFile, TaggedFileExt},
    picture::{MimeType, PictureType},
    probe::Probe,
    tag::{Accessor, ItemKey},
};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};
use symphonia::{
    core::{
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
    },
    default::get_probe,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::Emitter;
use tauri::{AppHandle, Manager, Window};
use tokio::time::sleep;

use crate::app_storage;

#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use tauri::{PhysicalPosition, PhysicalSize, Position, Size};

#[cfg(windows)]
const MINI_PLAYER_DOCK_THRESHOLD: i32 = 24;
#[cfg(windows)]
const MINI_PLAYER_PEEK_SIZE: i32 = 16;
#[cfg(windows)]
const DOCK_RECHECK_DELAY_MS: u64 = 120;
#[cfg(windows)]
const MINI_PLAYER_COLLAPSE_DELAY_MS: u64 = 2_000;
#[cfg(windows)]
const MINI_PLAYER_HOVER_POLL_MS: u64 = 80;
#[cfg(windows)]
const MINI_PLAYER_HOVER_TRIGGER_DISTANCE: i32 = 28;
#[cfg(windows)]
const MINI_PLAYER_LEAVE_BUFFER_MS: u64 = 300;

const API_BASE_LO: &str = "https://music-api.gdstudio.xyz/api.php";
const API_BASE_CN: &str = "https://music-api-cn.gdstudio.xyz/api.php";
const API_BASE_HK: &str = "https://music-api-hk.gdstudio.xyz/api.php";
const API_BASE_US: &str = "https://music-api-us.gdstudio.xyz/api.php";
const API_REFERER: &str = "https://music.gdstudio.xyz/";
const API_ORIGIN: &str = "https://music.gdstudio.xyz";
const PROXY_BASE: &str = "https://music-proxy.gdstudio.org";
const GEQUBAO_BASE: &str = "https://www.gequbao.com";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Fashion/0.1";
const AUDIO_CACHE_ROOT_DIR: &str = "cache";
const AUDIO_CACHE_FILES_DIR: &str = "audio";

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockedEdge {
    Left,
    Right,
    Top,
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[cfg(windows)]
impl WindowRect {
    fn right(self) -> i32 {
        self.x + self.width as i32
    }

    fn bottom(self) -> i32 {
        self.y + self.height as i32
    }
}

#[cfg(windows)]
fn clamp_rect_to_work_area(window: WindowRect, work_area: WindowRect) -> WindowRect {
    let max_x = work_area.right() - window.width as i32;
    let max_y = work_area.bottom() - window.height as i32;
    WindowRect {
        x: window.x.clamp(work_area.x, max_x),
        y: window.y.clamp(work_area.y, max_y),
        width: window.width,
        height: window.height,
    }
}

#[cfg(windows)]
fn pick_docked_edge(window: WindowRect, work_area: WindowRect, threshold: i32) -> Option<DockedEdge> {
    let distances = [
        (DockedEdge::Left, (window.x - work_area.x).abs()),
        (DockedEdge::Right, (work_area.right() - window.right()).abs()),
        (DockedEdge::Top, (window.y - work_area.y).abs()),
    ];

    distances
        .into_iter()
        .filter(|(_, distance)| *distance <= threshold)
        .min_by_key(|(edge, distance)| {
            let priority = match edge {
                DockedEdge::Left => 0,
                DockedEdge::Right => 1,
                DockedEdge::Top => 2,
            };
            (*distance, priority)
        })
        .map(|(edge, _)| edge)
}

#[cfg(windows)]
fn compute_docked_rect(window: WindowRect, work_area: WindowRect, edge: DockedEdge) -> WindowRect {
    let clamped = clamp_rect_to_work_area(window, work_area);
    match edge {
        DockedEdge::Left => WindowRect { x: work_area.x, ..clamped },
        DockedEdge::Right => WindowRect {
            x: work_area.right() - clamped.width as i32,
            ..clamped
        },
        DockedEdge::Top => WindowRect { y: work_area.y, ..clamped },
    }
}

#[cfg(windows)]
fn compute_collapsed_rect(expanded: WindowRect, work_area: WindowRect, edge: DockedEdge, peek: i32) -> WindowRect {
    match edge {
        DockedEdge::Left => WindowRect {
            x: work_area.x - expanded.width as i32 + peek,
            ..expanded
        },
        DockedEdge::Right => WindowRect {
            x: work_area.right() - peek,
            ..expanded
        },
        DockedEdge::Top => WindowRect {
            y: work_area.y - expanded.height as i32 + peek,
            ..expanded
        },
    }
}

#[cfg(windows)]
fn compute_collapsed_hover_rect(work_area: WindowRect, edge: DockedEdge, distance: i32) -> WindowRect {
    match edge {
        DockedEdge::Left => WindowRect {
            x: work_area.x,
            y: work_area.y,
            width: distance as u32,
            height: work_area.height,
        },
        DockedEdge::Right => WindowRect {
            x: work_area.right() - distance,
            y: work_area.y,
            width: distance as u32,
            height: work_area.height,
        },
        DockedEdge::Top => WindowRect {
            x: work_area.x,
            y: work_area.y,
            width: work_area.width,
            height: distance as u32,
        },
    }
}

#[cfg(windows)]
fn point_in_rect(x: i32, y: i32, rect: WindowRect) -> bool {
    x >= rect.x && x <= rect.right() && y >= rect.y && y <= rect.bottom()
}

#[cfg(windows)]
fn apply_rect<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>, rect: WindowRect) -> Result<(), String> {
    window
        .set_position(Position::Physical(PhysicalPosition { x: rect.x, y: rect.y }))
        .map_err(|e| e.to_string())?;
    window
        .set_size(Size::Physical(PhysicalSize {
            width: rect.width,
            height: rect.height,
        }))
        .map_err(|e| e.to_string())
}

fn make_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(18))
        .build()
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: Value,
    pub name: String,
    pub artist: Vec<Value>,
    pub album: Value,
    pub pic_id: Value,
    pub url_id: Value,
    pub lyric_id: Value,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicUrl {
    pub url: Option<String>,
    pub local_path: Option<String>,
    pub br: Option<Value>,
    pub size: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalTrackRecord {
    pub id: String,
    pub path: String,
    pub file_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_sec: Option<u64>,
    pub cover_path: Option<String>,
    pub lyric_path: Option<String>,
    pub file_size: u64,
    pub modified_at: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalLibraryScanResult {
    pub scanned_files: u64,
    pub imported_files: u64,
    pub updated_files: u64,
    pub removed_files: u64,
    pub skipped_files: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub path: String,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalLibraryScanResponse {
    pub folders: Vec<String>,
    pub tracks: Vec<LocalTrackRecord>,
    pub last_scan_at: u64,
    pub scan_result: LocalLibraryScanResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PicUrl {
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LyricResult {
    pub lyric: Option<String>,
    pub tlyric: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GequbaoApiResponse<T> {
    code: i64,
    #[serde(default)]
    data: Option<T>,
    #[serde(default)]
    msg: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GequbaoSearchStartData {
    #[serde(default)]
    k: Option<String>,
    #[serde(default)]
    u: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GequbaoPlayUrlData {
    url: String,
}

#[derive(Debug, Deserialize)]
struct GequbaoAppData {
    mp3_id: Value,
    play_id: String,
    #[serde(default)]
    mp3_cover: String,
}

#[derive(Debug)]
struct GequbaoMusicDetail {
    id: String,
    play_id: String,
    cover_url: Option<String>,
    lyric: String,
}

#[derive(Debug, Deserialize)]
struct KuwoConvertUrlData {
    code: i64,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioCacheMetadata {
    source: String,
    id: String,
    bitrate: u32,
    extension: String,
    content_type: Option<String>,
    remote_url: String,
    cached_at: u64,
}

#[derive(Debug)]
struct CachedAudioFile {
    path: PathBuf,
    metadata: AudioCacheMetadata,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheClearResult {
    files_removed: u64,
    bytes_freed: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    os: String,
    is_mobile: bool,
}

#[tauri::command]
pub fn get_runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        os: std::env::consts::OS.to_string(),
        is_mobile: cfg!(any(target_os = "android", target_os = "ios")),
    }
}

fn signature_for_seed(seed: &str) -> String {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn audio_cache_fill_store() -> &'static Mutex<HashSet<String>> {
    static STORE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashSet::new()))
}

fn audio_cache_key(source: &str, id: &str, bitrate: u32) -> String {
    signature_for_seed(&format!("{}:{id}:{bitrate}", strip_source_suffix(source)))
}

fn cache_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_storage::current_data_directory(app)?.join(AUDIO_CACHE_ROOT_DIR))
}

fn audio_cache_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(cache_root(app)?.join(AUDIO_CACHE_FILES_DIR))
}

fn audio_cache_source_dir(app: &AppHandle, source: &str) -> Result<PathBuf, String> {
    Ok(audio_cache_root(app)?.join(strip_source_suffix(source)))
}

fn audio_cache_metadata_path(source_dir: &Path, cache_key: &str) -> PathBuf {
    source_dir.join(format!("{cache_key}.json"))
}

fn audio_cache_file_path(source_dir: &Path, cache_key: &str, extension: &str) -> PathBuf {
    source_dir.join(format!("{cache_key}.{extension}"))
}

fn read_audio_cache(
    app: &AppHandle,
    source: &str,
    id: &str,
    bitrate: u32,
) -> Result<Option<CachedAudioFile>, String> {
    let source_dir = audio_cache_source_dir(app, source)?;
    let cache_key = audio_cache_key(source, id, bitrate);
    let metadata_path = audio_cache_metadata_path(&source_dir, &cache_key);

    if !metadata_path.exists() {
        return Ok(None);
    }

    let raw = match std::fs::read_to_string(&metadata_path) {
        Ok(raw) => raw,
        Err(_) => return Ok(None),
    };
    let metadata = match serde_json::from_str::<AudioCacheMetadata>(&raw) {
        Ok(metadata) => metadata,
        Err(_) => {
            let _ = std::fs::remove_file(&metadata_path);
            return Ok(None);
        }
    };
    let audio_path = audio_cache_file_path(&source_dir, &cache_key, &metadata.extension);

    if !audio_path.exists() {
        let _ = std::fs::remove_file(&metadata_path);
        return Ok(None);
    }

    Ok(Some(CachedAudioFile {
        path: audio_path,
        metadata,
    }))
}

async fn cache_audio_file(
    app: &AppHandle,
    client: &Client,
    source: &str,
    id: &str,
    bitrate: u32,
    remote_url: &str,
) -> Result<CachedAudioFile, String> {
    if let Some(cached) = read_audio_cache(app, source, id, bitrate)? {
        return Ok(cached);
    }

    let source_dir = audio_cache_source_dir(app, source)?;
    tokio::fs::create_dir_all(&source_dir)
        .await
        .map_err(|error| error.to_string())?;

    let response = client
        .get(remote_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("音频缓存失败: {}", response.status()));
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let final_url = response.url().to_string();
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    let extension = infer_extension(&final_url, content_type.as_deref(), bitrate);
    let cache_key = audio_cache_key(source, id, bitrate);
    let output_path = audio_cache_file_path(&source_dir, &cache_key, &extension);
    let temp_path = source_dir.join(format!("{cache_key}.part"));
    let metadata_path = audio_cache_metadata_path(&source_dir, &cache_key);

    tokio::fs::write(&temp_path, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    if output_path.exists() {
        let _ = tokio::fs::remove_file(&output_path).await;
    }
    tokio::fs::rename(&temp_path, &output_path)
        .await
        .map_err(|error| error.to_string())?;

    let metadata = AudioCacheMetadata {
        source: strip_source_suffix(source).to_string(),
        id: id.to_string(),
        bitrate,
        extension,
        content_type,
        remote_url: final_url,
        cached_at: now_unix_ms(),
    };
    let raw = serde_json::to_vec_pretty(&metadata).map_err(|error| error.to_string())?;
    tokio::fs::write(&metadata_path, raw)
        .await
        .map_err(|error| error.to_string())?;

    Ok(CachedAudioFile {
        path: output_path,
        metadata,
    })
}

fn spawn_audio_cache_fill(
    app: AppHandle,
    source: String,
    id: String,
    bitrate: u32,
    remote_url: String,
) {
    if remote_url.trim().is_empty() || ensure_supported_remote_url(&remote_url).is_err() {
        return;
    }

    let cache_key = audio_cache_key(&source, &id, bitrate);
    let Ok(mut store) = audio_cache_fill_store().lock() else {
        return;
    };
    if !store.insert(cache_key.clone()) {
        return;
    }
    drop(store);

    tokio::spawn(async move {
        if let Ok(client) = make_client() {
            let _ = cache_audio_file(&app, &client, &source, &id, bitrate, &remote_url).await;
        }

        if let Ok(mut store) = audio_cache_fill_store().lock() {
            store.remove(&cache_key);
        }
    });
}

fn collect_cache_file_stats(path: &Path, result: &mut CacheClearResult) -> Result<(), String> {
    let entries = std::fs::read_dir(path).map_err(|error| error.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let entry_path = entry.path();
        let metadata = entry.metadata().map_err(|error| error.to_string())?;

        if metadata.is_dir() {
            collect_cache_file_stats(&entry_path, result)?;
            continue;
        }

        if metadata.is_file() {
            result.files_removed += 1;
            result.bytes_freed += metadata.len();
        }
    }

    Ok(())
}

fn normalize_local_text(value: Option<&str>, fallback: &str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn file_stem_or_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| path.file_name().and_then(|value| value.to_str()))
        .unwrap_or("未知歌曲")
        .to_string()
}

fn system_time_to_unix_ms(value: Option<SystemTime>) -> u64 {
    value
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn is_local_audio_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "mp3" | "flac" | "m4a" | "wav" | "ogg")
    )
}

fn sibling_lyric_path(path: &Path) -> Option<String> {
    let lyric = path.with_extension("lrc");
    lyric.exists().then(|| lyric.display().to_string())
}

fn local_cover_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_storage::current_data_directory(app)?.join("cache").join("local-covers");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn picture_extension(mime_type: Option<MimeType>) -> &'static str {
    match mime_type {
        Some(MimeType::Png) => "png",
        Some(MimeType::Jpeg) => "jpg",
        Some(MimeType::Tiff) => "tiff",
        Some(MimeType::Bmp) => "bmp",
        Some(MimeType::Gif) => "gif",
        _ => "img",
    }
}

fn extract_track_duration_sec(path: &Path) -> Option<u64> {
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();
    let file = fs::File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if !extension.is_empty() {
        hint.with_extension(extension);
    }

    let probed = get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .ok()?;
    let format = probed.format;
    let track = format.default_track()?;
    let params = &track.codec_params;
    let frames = params.n_frames?;
    let time_base = params.time_base?;
    let duration = time_base.calc_time(frames);
    Some(duration.seconds)
}

fn save_embedded_cover(app: &AppHandle, track_id: &str, bytes: &[u8], mime_type: Option<MimeType>) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    let extension = picture_extension(mime_type);
    let cache_dir = local_cover_cache_dir(app).ok()?;
    let cover_path = cache_dir.join(format!("{track_id}.{extension}"));

    if fs::write(&cover_path, bytes).is_err() {
        return None;
    }

    Some(cover_path.display().to_string())
}

fn pick_embedded_cover(tagged_file: &lofty::file::TaggedFile) -> Option<&lofty::picture::Picture> {
    tagged_file
        .primary_tag()
        .and_then(|tag| {
            tag.pictures()
                .iter()
                .find(|picture| picture.pic_type() == PictureType::CoverFront)
                .or_else(|| tag.pictures().first())
        })
        .or_else(|| {
            tagged_file.tags().iter().find_map(|tag| {
                tag.pictures()
                    .iter()
                    .find(|picture| picture.pic_type() == PictureType::CoverFront)
                    .or_else(|| tag.pictures().first())
            })
        })
}

fn parse_local_audio_metadata(app: &AppHandle, path: &Path, track_id: &str) -> (String, String, String, Option<u64>, Option<String>) {
    let fallback_title = file_stem_or_name(path);
    let mut title = fallback_title.clone();
    let mut artist = "未知艺术家".to_string();
    let mut album = "未知专辑".to_string();
    let mut duration_sec = extract_track_duration_sec(path);
    let mut cover_path = None;

    if let Ok(tagged_file) = Probe::open(path)
        .and_then(|probe| probe.options(ParseOptions::new()).read()) {
        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            let title_text = tag
                .title()
                .map(|value| value.to_string())
                .or_else(|| tag.get_string(&ItemKey::TrackTitle).map(|value| value.to_string()));
            let artist_text = tag
                .artist()
                .map(|value| value.to_string())
                .or_else(|| tag.get_string(&ItemKey::TrackArtist).map(|value| value.to_string()));
            let album_text = tag
                .album()
                .map(|value| value.to_string())
                .or_else(|| tag.get_string(&ItemKey::AlbumTitle).map(|value| value.to_string()));

            title = normalize_local_text(title_text.as_deref(), &fallback_title);
            artist = normalize_local_text(artist_text.as_deref(), "未知艺术家");
            album = normalize_local_text(album_text.as_deref(), "未知专辑");
        }

        if duration_sec.is_none() {
            duration_sec = Some(tagged_file.properties().duration().as_secs());
        }

        if let Some(picture) = pick_embedded_cover(&tagged_file) {
            cover_path = save_embedded_cover(app, track_id, picture.data(), picture.mime_type().cloned());
        }
    }

    (title, artist, album, duration_sec.filter(|value| *value > 0), cover_path)
}

fn build_local_track_record(app: &AppHandle, path: &Path) -> Result<LocalTrackRecord, String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    let display_path = path.display().to_string();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let track_id = signature_for_seed(&display_path);
    let updated_at = system_time_to_unix_ms(metadata.modified().ok());
    let (title, artist, album, duration_sec, cover_path) = parse_local_audio_metadata(app, path, &track_id);

    Ok(LocalTrackRecord {
        id: format!("local-{track_id}"),
        path: display_path,
        file_name,
        title,
        artist,
        album,
        duration_sec,
        cover_path,
        lyric_path: sibling_lyric_path(path),
        file_size: metadata.len(),
        modified_at: updated_at,
        created_at: system_time_to_unix_ms(metadata.created().ok()),
        updated_at,
    })
}

fn collect_local_tracks(app: &AppHandle, path: &Path, tracks: &mut Vec<LocalTrackRecord>, result: &mut LocalLibraryScanResult) {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => {
            result.skipped_files += 1;
            return;
        }
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                result.skipped_files += 1;
                continue;
            }
        };

        if metadata.is_dir() {
            collect_local_tracks(app, &entry_path, tracks, result);
            continue;
        }

        if !metadata.is_file() || !is_local_audio_extension(&entry_path) {
            continue;
        }

        result.scanned_files += 1;
        match build_local_track_record(app, &entry_path) {
            Ok(record) => tracks.push(record),
            Err(_) => result.skipped_files += 1,
        }
    }
}

#[tauri::command]
pub fn clear_cached_audio_files(app: AppHandle) -> Result<CacheClearResult, String> {
    let cache_dir = cache_root(&app)?;
    let mut result = CacheClearResult::default();

    if !cache_dir.exists() {
        if let Ok(mut store) = audio_cache_fill_store().lock() {
            store.clear();
        }
        return Ok(result);
    }

    collect_cache_file_stats(&cache_dir, &mut result)?;
    std::fs::remove_dir_all(&cache_dir).map_err(|error| error.to_string())?;

    if let Ok(mut store) = audio_cache_fill_store().lock() {
        store.clear();
    }

    Ok(result)
}

#[tauri::command]
pub fn scan_local_library(app: AppHandle, folders: Vec<String>) -> Result<LocalLibraryScanResponse, String> {
    let normalized_folders: Vec<String> = folders
        .into_iter()
        .map(|folder| folder.trim().to_string())
        .filter(|folder| !folder.is_empty())
        .collect();

    let mut tracks = Vec::new();
    let mut scan_result = LocalLibraryScanResult {
        scanned_files: 0,
        imported_files: 0,
        updated_files: 0,
        removed_files: 0,
        skipped_files: 0,
    };

    for folder in &normalized_folders {
        let path = PathBuf::from(folder);
        if !path.exists() || !path.is_dir() {
            scan_result.skipped_files += 1;
            continue;
        }
        collect_local_tracks(&app, &path, &mut tracks, &mut scan_result);
    }

    tracks.sort_by(|left, right| {
        left.title
            .cmp(&right.title)
            .then_with(|| left.path.cmp(&right.path))
    });
    scan_result.imported_files = tracks.len() as u64;

    Ok(LocalLibraryScanResponse {
        folders: normalized_folders,
        tracks,
        last_scan_at: now_unix_ms(),
        scan_result,
    })
}

fn sanitize_filename_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => ' ',
            _ => ch,
        })
        .collect::<String>();

    let sanitized = sanitized.trim().trim_matches('.').to_string();
    if sanitized.is_empty() {
        "untitled".to_string()
    } else {
        sanitized
    }
}

fn infer_extension(url: &str, content_type: Option<&str>, bitrate: u32) -> String {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(last) = parsed.path_segments().and_then(|segments| segments.last()) {
            if let Some(ext) = Path::new(last).extension().and_then(|ext| ext.to_str()) {
                let ext = ext.trim().to_ascii_lowercase();
                if !ext.is_empty() {
                    return ext;
                }
            }
        }
    }

    if let Some(content_type) = content_type {
        let ct = content_type.to_ascii_lowercase();
        if ct.contains("flac") {
            return "flac".to_string();
        }
        if ct.contains("mpeg") || ct.contains("mp3") {
            return "mp3".to_string();
        }
        if ct.contains("mp4") || ct.contains("m4a") {
            return "m4a".to_string();
        }
        if ct.contains("ogg") {
            return "ogg".to_string();
        }
        if ct.contains("aac") {
            return "aac".to_string();
        }
        if ct.contains("wav") {
            return "wav".to_string();
        }
    }

    if bitrate > 320 {
        "flac".to_string()
    } else {
        "mp3".to_string()
    }
}

fn unique_download_path(download_dir: &Path, file_stem: &str, extension: &str) -> PathBuf {
    let mut candidate = download_dir.join(format!("{file_stem}.{extension}"));
    let mut counter = 1;

    while candidate.exists() {
        candidate = download_dir.join(format!("{file_stem} ({counter}).{extension}"));
        counter += 1;
    }

    candidate
}

fn strip_source_suffix(source: &str) -> &str {
    source
        .strip_suffix("_album")
        .or_else(|| source.strip_suffix("_playlist"))
        .unwrap_or(source)
}

fn source_node(source: &str) -> &'static str {
    match strip_source_suffix(source) {
        "kuwo" | "tencent" => "lo",
        "migu" | "kugou" | "ximalaya" => "cn",
        "joox" => "hk",
        "qobuz" | "ytmusic" => "us",
        _ => "lo",
    }
}

fn api_base_for_source(source: &str) -> &'static str {
    match source_node(source) {
        "cn" => API_BASE_CN,
        "hk" => API_BASE_HK,
        "us" => API_BASE_US,
        _ => API_BASE_LO,
    }
}

fn api_root_for_source(source: &str) -> &'static str {
    match source_node(source) {
        "cn" => "https://music-api-cn.gdstudio.xyz/",
        "hk" => "https://music-api-hk.gdstudio.xyz/",
        "us" => "https://music-api-us.gdstudio.xyz/",
        _ => "https://music-api.gdstudio.xyz/",
    }
}

fn is_cache_source(source: &str) -> bool {
    matches!(
        strip_source_suffix(source),
        "ytmusic" | "deezer" | "spotify" | "apple"
    )
}

fn needs_proxy(source: &str, url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    matches!(strip_source_suffix(source), "qobuz" | "ytmusic")
        || lower.contains("googlevideo.com")
        || lower.contains("ytimg.com")
        || lower.contains("qobuz")
}

fn ensure_supported_remote_url(url: &str) -> Result<String, String> {
    let parsed = Url::parse(url).map_err(|e| e.to_string())?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed.to_string()),
        _ => Err("Unsupported remote url scheme".to_string()),
    }
}

fn proxy_audio_url(source: &str, remote_url: &str) -> Result<String, String> {
    let encoded = urlencoding::encode(remote_url);
    let source_name = strip_source_suffix(source);
    Ok(format!(
        "{PROXY_BASE}/audio?url={encoded}&source={source_name}"
    ))
}

fn normalize_music_url(source: &str, raw_url: &str) -> Result<String, String> {
    let mut url = raw_url.trim().to_string();
    if url.is_empty() {
        return Ok(url);
    }

    if is_cache_source(source) && Url::parse(&url).is_err() {
        url = format!(
            "{}{}",
            api_root_for_source(source),
            url.trim_start_matches('/')
        );
    }

    if strip_source_suffix(source) == "kuwo" {
        url = normalize_kuwo_url(&url);
    }

    if needs_proxy(source, &url) && !url.starts_with(PROXY_BASE) {
        url = proxy_audio_url(source, &url)?;
    }

    ensure_supported_remote_url(&url)
}

fn normalize_kuwo_url(raw_url: &str) -> String {
    let mut parsed = match Url::parse(raw_url) {
        Ok(url) => url,
        Err(_) => return raw_url.to_string(),
    };

    if parsed.scheme() == "http" {
        let _ = parsed.set_scheme("https");
    }

    if let Some(host) = parsed.host_str() {
        if host == "kuwo.cn" || host.ends_with(".kuwo.cn") {
            let parts: Vec<&str> = host.split('.').collect();
            if parts.len() > 2 {
                let sub = parts[..parts.len() - 2].join("-");
                let domain = parts[parts.len() - 2..].join(".");
                let _ = parsed.set_host(Some(&format!("{sub}.{domain}")));
            }
        }
    }

    parsed.to_string()
}

fn is_gequbao_source(source: &str) -> bool {
    matches!(strip_source_suffix(source), "gequbao")
}

fn gequbao_absolute_url(path: &str) -> Result<Url, String> {
    Url::parse(GEQUBAO_BASE)
        .map_err(|e| e.to_string())?
        .join(path)
        .map_err(|e| e.to_string())
}

fn normalize_gequbao_cover_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("//") {
        return Some(format!("https:{trimmed}"));
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Some(trimmed.to_string());
    }

    gequbao_absolute_url(trimmed).ok().map(|url| url.to_string())
}

async fn fetch_text(client: &Client, url: Url, referer: Option<&str>) -> Result<String, String> {
    let mut request = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/json");

    if let Some(referer) = referer {
        request = request
            .header("Referer", referer)
            .header("Origin", GEQUBAO_BASE);
    }

    let response = request.send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let snippet = &text[..text.len().min(240)];
        return Err(format!("Request failed ({status}): {snippet}"));
    }

    Ok(text)
}

async fn post_form_json<T: DeserializeOwned>(
    client: &Client,
    url: Url,
    form: &[(&str, String)],
    referer: &str,
) -> Result<T, String> {
    let response = client
        .post(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .header("Referer", referer)
        .header("Origin", GEQUBAO_BASE)
        .header("X-Requested-With", "XMLHttpRequest")
        .form(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let snippet = &text[..text.len().min(240)];
        return Err(format!("Request failed ({status}): {snippet}"));
    }

    serde_json::from_str(&text).map_err(|e| {
        format!(
            "Failed to parse response: {e} | body: {}",
            &text[..text.len().min(300)]
        )
    })
}

fn parse_selector(selector: &str) -> Result<Selector, String> {
    Selector::parse(selector).map_err(|e| e.to_string())
}

fn parse_music_id_from_href(href: &str) -> Option<String> {
    href.trim()
        .strip_prefix("/music/")
        .map(|value| value.trim_matches('/').to_string())
        .filter(|value| !value.is_empty())
}

fn collect_joined_text<'a, I>(segments: I, joiner: &str) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    segments
        .into_iter()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(joiner)
}

fn decode_html_text(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .replace('\u{a0}', " ")
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn normalize_lyric_text(input: &str) -> String {
    input
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn parse_gequbao_search_results(
    html: &str,
    count: u32,
    page: u32,
) -> Result<Vec<SearchResult>, String> {
    let document = Html::parse_document(html);
    let row_selector = parse_selector(
        "div.card.mb-1 div.card-text > div.row.no-gutters.py-2d5.border-top.align-items-center",
    )?;
    let link_selector = parse_selector("a[href^=\"/music/\"]")?;
    let title_selector = parse_selector("span.text-primary")?;
    let artist_selector = parse_selector("small.text-jade")?;

    let mut results = Vec::new();

    for row in document.select(&row_selector) {
        let Some(link) = row.select(&link_selector).next() else {
            continue;
        };
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let Some(id) = parse_music_id_from_href(href) else {
            continue;
        };

        let title = row
            .select(&title_selector)
            .next()
            .map(|element| collect_joined_text(element.text(), ""))
            .filter(|value| !value.is_empty())
            .or_else(|| link.value().attr("title").map(str::to_string))
            .map(|value| decode_html_text(&value))
            .unwrap_or_else(|| id.clone());

        let artist = row
            .select(&artist_selector)
            .next()
            .map(|element| collect_joined_text(element.text(), ""))
            .filter(|value| !value.is_empty())
            .map(|value| decode_html_text(&value))
            .unwrap_or_else(|| "Unknown Artist".to_string());

        results.push(SearchResult {
            id: Value::String(id.clone()),
            name: title,
            artist: vec![Value::String(artist)],
            album: Value::String(String::new()),
            pic_id: Value::String(id.clone()),
            url_id: Value::String(id.clone()),
            lyric_id: Value::String(id.clone()),
            source: "gequbao".to_string(),
        });
    }

    let page = page.max(1);
    let start = ((page - 1) * count) as usize;
    Ok(results
        .into_iter()
        .skip(start)
        .take(count as usize)
        .collect())
}

fn decode_js_string(raw: &str) -> Result<String, String> {
    let escaped = raw.replace("\\'", "'");
    serde_json::from_str::<String>(&format!("\"{escaped}\"")).map_err(|e| e.to_string())
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

fn extract_gequbao_app_data(html: &str) -> Result<GequbaoAppData, String> {
    let marker = "window.appData = JSON.parse('";
    let start = html
        .find(marker)
        .ok_or_else(|| "Missing gequbao appData".to_string())?
        + marker.len();
    let end = html[start..]
        .find("');")
        .map(|offset| start + offset)
        .ok_or_else(|| "Invalid gequbao appData payload".to_string())?;

    let encoded = &html[start..end];
    let decoded = decode_js_string(encoded)?;
    serde_json::from_str(&decoded).map_err(|e| format!("Failed to parse gequbao appData: {e}"))
}

fn parse_gequbao_lyric(html: &str) -> Result<String, String> {
    let document = Html::parse_document(html);
    let lyric_selector = parse_selector("#content-lrc")?;
    let Some(element) = document.select(&lyric_selector).next() else {
        return Ok(String::new());
    };

    let raw = element
        .inner_html()
        .replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n");
    let without_tags = strip_html_tags(&raw);
    Ok(normalize_lyric_text(&decode_html_text(&without_tags)))
}

async fn resolve_gequbao_music_detail(client: &Client, id: &str) -> Result<GequbaoMusicDetail, String> {
    let path = format!("/music/{id}");
    let html = fetch_text(client, gequbao_absolute_url(&path)?, Some(GEQUBAO_BASE)).await?;
    let app_data = extract_gequbao_app_data(&html)?;
    let lyric = parse_gequbao_lyric(&html)?;

    Ok(GequbaoMusicDetail {
        id: value_to_string(&app_data.mp3_id),
        play_id: app_data.play_id,
        cover_url: normalize_gequbao_cover_url(&app_data.mp3_cover),
        lyric,
    })
}

async fn resolve_gequbao_search_url(client: &Client, keyword: &str) -> Result<Url, String> {
    let response: GequbaoApiResponse<GequbaoSearchStartData> = post_form_json(
        client,
        gequbao_absolute_url("/api/s")?,
        &[("keyword", keyword.to_string())],
        GEQUBAO_BASE,
    )
    .await?;

    match response.code {
        1 => {
            let path = response
                .data
                .and_then(|data| data.u)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| format!("/s/{}", urlencoding::encode(keyword)));
            gequbao_absolute_url(&path)
        }
        2 => {
            let wait_keyword = response
                .data
                .and_then(|data| data.k)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| keyword.to_string());

            for _ in 0..8 {
                let poll: GequbaoApiResponse<Value> = post_form_json(
                    client,
                    gequbao_absolute_url("/api/query-map")?,
                    &[("keyword", wait_keyword.clone())],
                    GEQUBAO_BASE,
                )
                .await?;

                if poll.code == 1 {
                    return gequbao_absolute_url(&format!("/s/{}", urlencoding::encode(&wait_keyword)));
                }

                sleep(Duration::from_millis(1500)).await;
            }

            Err("Gequbao search is still queued. Please retry in a moment.".to_string())
        }
        _ => Err(response
            .msg
            .unwrap_or_else(|| "Gequbao search failed".to_string())),
    }
}

async fn search_gequbao_music(
    client: &Client,
    keyword: &str,
    count: u32,
    page: u32,
) -> Result<Vec<SearchResult>, String> {
    let search_url = resolve_gequbao_search_url(client, keyword).await?;
    let html = fetch_text(client, search_url, Some(GEQUBAO_BASE)).await?;
    parse_gequbao_search_results(&html, count, page)
}

async fn convert_kuwo_play_url(client: &Client, raw_url: &str) -> Result<String, String> {
    let url = Url::parse(raw_url).map_err(|e| e.to_string())?;
    if url.host_str() != Some("antiserver.kuwo.cn") {
        return Ok(raw_url.to_string());
    }

    let response_mode = url
        .query_pairs()
        .find_map(|(key, value)| (key == "response").then(|| value.into_owned()))
        .unwrap_or_default();

    if response_mode.eq_ignore_ascii_case("res") {
        return Ok(raw_url.to_string());
    }

    let existing_pairs = url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    let mut converted_url = url.clone();
    converted_url.set_query(None);

    {
        let mut next_pairs = converted_url.query_pairs_mut();
        let mut has_type = false;
        let mut has_response = false;

        for (key, value) in existing_pairs {
            match key.as_str() {
                "type" => {
                    next_pairs.append_pair("type", "convert_url3");
                    has_type = true;
                }
                "response" => {
                    next_pairs.append_pair("response", "url");
                    has_response = true;
                }
                _ => {
                    next_pairs.append_pair(&key, &value);
                }
            }
        }

        if !has_type {
            next_pairs.append_pair("type", "convert_url3");
        }
        if !has_response {
            next_pairs.append_pair("response", "url");
        }
    }

    let body = fetch_text(client, converted_url, None).await?;
    let trimmed = body.trim().trim_matches('"');
    if let Ok(url) = ensure_supported_remote_url(trimmed) {
        return Ok(url);
    }

    let response = serde_json::from_str::<KuwoConvertUrlData>(&body)
        .map_err(|error| format!("Failed to parse Kuwo play url conversion response: {error}"))?;
    if response.code != 200 {
        return Err(response
            .msg
            .unwrap_or_else(|| "Failed to convert Kuwo play url".to_string()));
    }

    response
        .url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Missing converted play url".to_string())
}

async fn get_gequbao_music_url(client: &Client, id: &str) -> Result<MusicUrl, String> {
    let detail = resolve_gequbao_music_detail(client, id).await?;
    let referer = format!("{GEQUBAO_BASE}/music/{}", detail.id);
    let response: GequbaoApiResponse<GequbaoPlayUrlData> = post_form_json(
        client,
        gequbao_absolute_url("/api/play-url")?,
        &[("id", detail.play_id)],
        &referer,
    )
    .await?;

    if response.code != 1 {
        return Err(response
            .msg
            .unwrap_or_else(|| "Failed to fetch gequbao play url".to_string()));
    }

    let raw_url = response
        .data
        .map(|data| data.url)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Missing gequbao play url".to_string())?;
    let url = convert_kuwo_play_url(client, &raw_url).await?;

    Ok(MusicUrl {
        url: Some(url),
        local_path: None,
        br: None,
        size: None,
    })
}

async fn resolve_music_url(
    client: &Client,
    source: &str,
    id: &str,
    br: u32,
) -> Result<MusicUrl, String> {
    if is_gequbao_source(source) {
        return get_gequbao_music_url(client, id).await;
    }

    let url = build_api_url(
        source,
        &[
            ("types", "url".to_string()),
            ("source", source.to_string()),
            ("id", id.to_string()),
            ("br", br.to_string()),
        ],
    )?;

    let mut result = fetch_json::<MusicUrl>(client, url).await?;

    if let Some(url) = result.url.as_deref() {
        result.url = Some(normalize_music_url(source, url)?);
    }
    result.local_path = None;

    Ok(result)
}

async fn fetch_json<T: DeserializeOwned>(client: &Client, url: Url) -> Result<T, String> {
    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Referer", API_REFERER)
        .header("Origin", API_ORIGIN)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    response.json::<T>().await.map_err(|e| e.to_string())
}

fn build_api_url(source: &str, params: &[(impl AsRef<str>, String)]) -> Result<Url, String> {
    let mut url = Url::parse(api_base_for_source(source)).map_err(|e| e.to_string())?;
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in params {
            pairs.append_pair(key.as_ref(), value);
        }
    }
    Ok(url)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub is_fill: bool,
    pub is_lyric_fullscreen: bool,
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
struct WindowBounds {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
struct LyricFullscreenBounds {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
}

#[cfg(windows)]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MiniPlayerDockPayload {
    docked_edge: Option<&'static str>,
    collapsed: bool,
}

#[cfg(windows)]
#[derive(Debug, Default, Clone, Copy)]
struct MiniPlayerDockState {
    docked_edge: Option<DockedEdge>,
    collapsed: bool,
    expanded_rect: Option<WindowRect>,
    collapsed_rect: Option<WindowRect>,
    work_area_rect: Option<WindowRect>,
    pointer_outside_expanded: bool,
    leave_session: u64,
    hover_session: u64,
    collapse_session: u64,
    move_session: u64,
}

#[cfg(windows)]
impl MiniPlayerDockState {
    fn reset(&mut self) {
        self.docked_edge = None;
        self.collapsed = false;
        self.expanded_rect = None;
        self.collapsed_rect = None;
        self.work_area_rect = None;
        self.pointer_outside_expanded = false;
        self.leave_session = self.leave_session.wrapping_add(1);
        self.hover_session = self.hover_session.wrapping_add(1);
        self.collapse_session = self.collapse_session.wrapping_add(1);
        self.move_session = self.move_session.wrapping_add(1);
    }

    fn payload(&self) -> MiniPlayerDockPayload {
        MiniPlayerDockPayload {
            docked_edge: self.docked_edge.map(|edge| match edge {
                DockedEdge::Left => "left",
                DockedEdge::Right => "right",
                DockedEdge::Top => "top",
            }),
            collapsed: self.collapsed,
        }
    }
}

#[cfg(windows)]
fn mini_player_dock_store() -> &'static Mutex<HashMap<String, MiniPlayerDockState>> {
    static STORE: OnceLock<Mutex<HashMap<String, MiniPlayerDockState>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(windows)]
fn maximized_bounds_store() -> &'static Mutex<HashMap<String, WindowBounds>> {
    static STORE: OnceLock<Mutex<HashMap<String, WindowBounds>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(windows)]
fn lyric_fullscreen_store() -> &'static Mutex<HashMap<String, LyricFullscreenBounds>> {
    static STORE: OnceLock<Mutex<HashMap<String, LyricFullscreenBounds>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(windows)]
fn rect_from_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) -> Result<WindowRect, String> {
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    Ok(WindowRect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

#[cfg(windows)]
fn rect_from_work_area<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) -> Result<WindowRect, String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| String::from("monitor not found"))?;
    let work_area = monitor.work_area();
    Ok(WindowRect {
        x: work_area.position.x,
        y: work_area.position.y,
        width: work_area.size.width,
        height: work_area.size.height,
    })
}

#[cfg(windows)]
fn emit_mini_player_dock_state<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>, state: &MiniPlayerDockState) {
    let _ = window.emit("mini-player:dock-state", state.payload());
}

#[cfg(windows)]
fn is_fill_bounds(window: &Window) -> bool {
    let Ok(position) = window.outer_position() else {
        return false;
    };
    let Ok(size) = window.outer_size() else {
        return false;
    };
    let Ok(Some(monitor)) = window.current_monitor() else {
        return false;
    };
    let work_area = monitor.work_area();
    position.x == work_area.position.x
        && position.y == work_area.position.y
        && size.width == work_area.size.width
        && size.height == work_area.size.height
}

fn build_window_state(window: &Window) -> WindowState {
    #[cfg(windows)]
    let is_fill = is_fill_bounds(window);
    #[cfg(not(windows))]
    let is_fill = window.is_maximized().unwrap_or(false);

    #[cfg(windows)]
    let is_lyric_fullscreen = lyric_fullscreen_store()
        .lock()
        .map(|store| store.contains_key(window.label()))
        .unwrap_or(false);
    #[cfg(not(windows))]
    let is_lyric_fullscreen = window.is_fullscreen().unwrap_or(false);

    WindowState {
        is_fill,
        is_lyric_fullscreen,
    }
}

#[cfg(windows)]
fn store_mini_player_dock_state<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    edge: DockedEdge,
    expanded_rect: WindowRect,
    collapsed_rect: WindowRect,
    work_area_rect: WindowRect,
) -> Result<(), String> {
    let mut store = mini_player_dock_store().lock().map_err(|e| e.to_string())?;
    let state = store.entry(window.label().to_string()).or_default();
    state.docked_edge = Some(edge);
    state.collapsed = false;
    state.expanded_rect = Some(expanded_rect);
    state.collapsed_rect = Some(collapsed_rect);
    state.work_area_rect = Some(work_area_rect);
    state.pointer_outside_expanded = false;
    state.leave_session = state.leave_session.wrapping_add(1);
    state.hover_session = state.hover_session.wrapping_add(1);
    state.collapse_session = state.collapse_session.wrapping_add(1);
    emit_mini_player_dock_state(window, state);
    Ok(())
}

#[cfg(windows)]
pub fn reset_mini_player_dock_state_for_label(label: &str) {
    let Ok(mut store) = mini_player_dock_store().lock() else {
        return;
    };
    let state = store.entry(label.to_string()).or_default();
    state.reset();
}

#[cfg(windows)]
pub fn reset_mini_player_dock_state_for_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    reset_mini_player_dock_state_for_label(window.label());
    if let Ok(store) = mini_player_dock_store().lock() {
        if let Some(state) = store.get(window.label()) {
            emit_mini_player_dock_state(window, state);
        }
    }
}

#[cfg(windows)]
fn reset_mini_player_dock_state<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    reset_mini_player_dock_state_for_window(window);
}

#[cfg(windows)]
fn current_cursor_position() -> Result<(i32, i32), String> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT::default();
    unsafe {
        GetCursorPos(&mut point).map_err(|e| e.to_string())?;
    }
    Ok((point.x, point.y))
}

#[cfg(windows)]
fn schedule_mini_player_collapse<R: tauri::Runtime>(window: tauri::WebviewWindow<R>) {
    let label = window.label().to_string();
    let session = {
        let Ok(mut store) = mini_player_dock_store().lock() else {
            return;
        };
        let state = store.entry(label.clone()).or_default();
        state.collapse_session = state.collapse_session.wrapping_add(1);
        state.collapse_session
    };

    tauri::async_runtime::spawn(async move {
        sleep(Duration::from_millis(MINI_PLAYER_COLLAPSE_DELAY_MS)).await;

        let mut store = match mini_player_dock_store().lock() {
            Ok(store) => store,
            Err(_) => return,
        };
        let Some(state) = store.get_mut(&label) else {
            return;
        };
        if state.collapse_session != session || state.collapsed {
            return;
        }
        let Some(collapsed_rect) = state.collapsed_rect else {
            return;
        };

        if apply_rect(&window, collapsed_rect).is_ok() {
            state.collapsed = true;
            emit_mini_player_dock_state(&window, state);
        }
    });
}

#[cfg(windows)]
fn schedule_mini_player_leave_buffer<R: tauri::Runtime>(window: tauri::WebviewWindow<R>) {
    let label = window.label().to_string();
    let session = {
        let Ok(mut store) = mini_player_dock_store().lock() else {
            return;
        };
        let state = store.entry(label.clone()).or_default();
        state.leave_session = state.leave_session.wrapping_add(1);
        state.leave_session
    };

    tauri::async_runtime::spawn(async move {
        sleep(Duration::from_millis(MINI_PLAYER_LEAVE_BUFFER_MS)).await;

        let should_collapse = match mini_player_dock_store().lock() {
            Ok(store) => store
                .get(&label)
                .map(|state| {
                    state.leave_session == session
                        && !state.collapsed
                        && state.pointer_outside_expanded
                        && state.docked_edge.is_some()
                })
                .unwrap_or(false),
            Err(_) => false,
        };
        if !should_collapse {
            return;
        }

        schedule_mini_player_collapse(window);
    });
}

#[cfg(windows)]
fn schedule_mini_player_dock_detection<R: tauri::Runtime>(window: tauri::WebviewWindow<R>) {
    let label = window.label().to_string();
    let session = {
        let Ok(mut store) = mini_player_dock_store().lock() else {
            return;
        };
        let state = store.entry(label.clone()).or_default();
        state.move_session = state.move_session.wrapping_add(1);
        state.move_session
    };

    tauri::async_runtime::spawn(async move {
        sleep(Duration::from_millis(DOCK_RECHECK_DELAY_MS)).await;

        let should_run = match mini_player_dock_store().lock() {
            Ok(store) => store
                .get(&label)
                .map(|state| state.move_session == session)
                .unwrap_or(false),
            Err(_) => false,
        };
        if !should_run {
            return;
        }

        let _ = mini_player_detect_dock_after_drag(window).await;
    });
}

#[cfg(windows)]
fn start_mini_player_hover_tracking<R: tauri::Runtime>(window: tauri::WebviewWindow<R>) {
    let label = window.label().to_string();
    let session = {
        let Ok(mut store) = mini_player_dock_store().lock() else {
            return;
        };
        let state = store.entry(label.clone()).or_default();
        state.hover_session = state.hover_session.wrapping_add(1);
        state.hover_session
    };

    tauri::async_runtime::spawn(async move {
        loop {
            sleep(Duration::from_millis(MINI_PLAYER_HOVER_POLL_MS)).await;

            let (cursor_x, cursor_y) = match current_cursor_position() {
                Ok(position) => position,
                Err(_) => break,
            };

            let mut store = match mini_player_dock_store().lock() {
                Ok(store) => store,
                Err(_) => break,
            };
            let Some(state) = store.get_mut(&label) else {
                break;
            };
            if state.hover_session != session || state.docked_edge.is_none() {
                break;
            }

            if state.collapsed {
                if let (Some(edge), Some(expanded_rect), Some(work_area_rect)) = (
                    state.docked_edge,
                    state.expanded_rect,
                    state.work_area_rect,
                ) {
                    let hover_rect = compute_collapsed_hover_rect(
                        work_area_rect,
                        edge,
                        MINI_PLAYER_HOVER_TRIGGER_DISTANCE,
                    );
                    if point_in_rect(cursor_x, cursor_y, hover_rect) {
                        if apply_rect(&window, expanded_rect).is_ok() {
                            state.collapsed = false;
                            state.pointer_outside_expanded = false;
                            state.leave_session = state.leave_session.wrapping_add(1);
                            state.collapse_session = state.collapse_session.wrapping_add(1);
                            emit_mini_player_dock_state(&window, state);
                        }
                    }
                }
            } else if let Some(expanded_rect) = state.expanded_rect {
                if point_in_rect(cursor_x, cursor_y, expanded_rect) {
                    if state.pointer_outside_expanded {
                        state.pointer_outside_expanded = false;
                        state.leave_session = state.leave_session.wrapping_add(1);
                        state.collapse_session = state.collapse_session.wrapping_add(1);
                    }
                } else if !state.pointer_outside_expanded {
                    state.pointer_outside_expanded = true;
                    drop(store);
                    schedule_mini_player_leave_buffer(window.clone());
                    continue;
                }
            }
        }
    });
}

#[cfg(windows)]
async fn mini_player_detect_dock_after_drag<R: tauri::Runtime>(window: tauri::WebviewWindow<R>) -> Result<(), String> {
    if window.label() != "mini-player" {
        return Ok(());
    }

    let current = rect_from_window(&window)?;
    let work_area = rect_from_work_area(&window)?;
    let Some(edge) = pick_docked_edge(current, work_area, MINI_PLAYER_DOCK_THRESHOLD) else {
        let preserve_existing_dock = mini_player_dock_store()
            .lock()
            .ok()
            .and_then(|store| store.get(window.label()).copied())
            .map(|state| state.collapsed)
            .unwrap_or(false);
        if !preserve_existing_dock {
            reset_mini_player_dock_state(&window);
        }
        return Ok(());
    };

    let expanded = compute_docked_rect(current, work_area, edge);
    let collapsed = compute_collapsed_rect(expanded, work_area, edge, MINI_PLAYER_PEEK_SIZE);

    apply_rect(&window, expanded)?;
    store_mini_player_dock_state(&window, edge, expanded, collapsed, work_area)?;
    start_mini_player_hover_tracking(window);
    Ok(())
}

#[tauri::command]
pub async fn search_music(
    keyword: String,
    source: Option<String>,
    count: Option<u32>,
    pages: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let client = make_client()?;
    let source = source.unwrap_or_else(|| "netease".to_string());
    let count = count.unwrap_or(20);
    let pages = pages.unwrap_or(1);

    if is_gequbao_source(&source) {
        return search_gequbao_music(&client, &keyword, count, pages).await;
    }

    let url = build_api_url(
        &source,
        &[
            ("types", "search".to_string()),
            ("source", source.clone()),
            ("name", keyword),
            ("count", count.to_string()),
            ("pages", pages.to_string()),
        ],
    )?;

    fetch_json::<Vec<SearchResult>>(&client, url).await
}

#[tauri::command]
pub async fn get_music_url(
    app: AppHandle,
    id: String,
    source: Option<String>,
    br: Option<u32>,
) -> Result<MusicUrl, String> {
    let source = source.unwrap_or_else(|| "netease".to_string());
    let br = br.unwrap_or(320);

    if let Some(cached) = read_audio_cache(&app, &source, &id, br)? {
        return Ok(MusicUrl {
            url: None,
            local_path: Some(cached.path.display().to_string()),
            br: None,
            size: None,
        });
    }

    let client = make_client()?;
    let result = resolve_music_url(&client, &source, &id, br).await?;
    if let Some(remote_url) = result.url.clone() {
        spawn_audio_cache_fill(app, source, id, br, remote_url);
    }
    Ok(result)
}

#[tauri::command]
pub async fn download_music(
    app: AppHandle,
    source: String,
    id: String,
    bitrate: Option<u32>,
    title: String,
    artist: String,
) -> Result<DownloadResult, String> {
    let bitrate = bitrate.unwrap_or(320);
    let download_dir = app_storage::current_download_directory(&app)
        .map_err(|e| format!("无法定位下载目录: {e}"))?;
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;

    let file_stem = format!(
        "{} - {}",
        sanitize_filename_part(&title),
        sanitize_filename_part(&artist)
    );
    let cached = if let Some(cached) = read_audio_cache(&app, &source, &id, bitrate)? {
        cached
    } else {
        let music_url = get_music_url(app.clone(), id.clone(), Some(source.clone()), Some(bitrate)).await?;
        let download_url = music_url
            .local_path
            .clone()
            .or(music_url.url.clone())
            .ok_or_else(|| "未获取到下载链接".to_string())?;

        if let Some(local_path) = music_url.local_path {
            let extension = Path::new(&local_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("mp3")
                .to_string();
            CachedAudioFile {
                path: PathBuf::from(local_path),
                metadata: AudioCacheMetadata {
                    source: strip_source_suffix(&source).to_string(),
                    id: id.clone(),
                    bitrate,
                    extension,
                    content_type: None,
                    remote_url: download_url,
                    cached_at: now_unix_ms(),
                },
            }
        } else {
            let client = make_client()?;
            cache_audio_file(&app, &client, &source, &id, bitrate, &download_url).await?
        }
    };
    let extension = cached.metadata.extension.clone();
    let output_path = unique_download_path(&download_dir, &file_stem, &extension);

    tokio::fs::copy(&cached.path, &output_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DownloadResult {
        path: output_path.display().to_string(),
        file_name: output_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "downloaded-audio".to_string()),
    })
}

#[tauri::command]
pub async fn get_pic_url(id: String, source: Option<String>) -> Result<PicUrl, String> {
    let client = make_client()?;
    let source = source.unwrap_or_else(|| "netease".to_string());

    if is_gequbao_source(&source) {
        let detail = resolve_gequbao_music_detail(&client, &id).await?;
        return Ok(PicUrl { url: detail.cover_url });
    }

    let url = build_api_url(
        &source,
        &[
            ("types", "pic".to_string()),
            ("source", source.clone()),
            ("id", id),
        ],
    )?;

    fetch_json::<PicUrl>(&client, url).await
}

#[tauri::command]
pub async fn get_lyric(id: String, source: Option<String>) -> Result<LyricResult, String> {
    let client = make_client()?;
    let source = source.unwrap_or_else(|| "netease".to_string());

    if is_gequbao_source(&source) {
        let detail = resolve_gequbao_music_detail(&client, &id).await?;
        return Ok(LyricResult {
            lyric: Some(detail.lyric),
            tlyric: None,
        });
    }

    let url = build_api_url(
        &source,
        &[
            ("types", "lyric".to_string()),
            ("source", source.clone()),
            ("id", id),
        ],
    )?;

    fetch_json::<LyricResult>(&client, url).await
}

#[tauri::command]
pub async fn get_aux_lyric(id: String, source: Option<String>) -> Result<LyricResult, String> {
    get_lyric(id, source).await
}

#[tauri::command]
pub async fn get_playlist_detail(
    id: String,
    source: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, String> {
    let client = make_client()?;
    let source = source.unwrap_or_else(|| "netease".to_string());
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let url = build_api_url(
        &source,
        &[
            ("types", "playlist".to_string()),
            ("source", source.clone()),
            ("id", id),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ],
    )?;

    fetch_json::<Value>(&client, url).await
}

#[tauri::command]
pub async fn get_user_playlists() -> Result<Vec<Value>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn get_recommend_playlist() -> Result<Vec<Value>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn search_once(keyword: String, source: String) -> Result<Vec<SearchResult>, String> {
    let client = make_client()?;

    if is_gequbao_source(&source) {
        return search_gequbao_music(&client, &keyword, 10, 1).await;
    }

    let url = build_api_url(
        &source,
        &[
            ("types", "search".to_string()),
            ("source", source.clone()),
            ("name", keyword),
            ("count", "10".to_string()),
            ("pages", "1".to_string()),
        ],
    )?;

    fetch_json::<Vec<SearchResult>>(&client, url).await
}

#[tauri::command]
pub async fn window_minimize(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        window.minimize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn window_maximize(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        Ok(())
    }

    #[cfg(all(windows, not(any(target_os = "android", target_os = "ios"))))]
    {
        let label = window.label().to_string();
        let mut store = maximized_bounds_store().lock().map_err(|e| e.to_string())?;

        if is_fill_bounds(&window) {
            if let Some(bounds) = store.remove(&label) {
                if window.is_fullscreen().unwrap_or(false) {
                    let _ = window.set_fullscreen(false);
                }
                window
                    .set_position(Position::Physical(bounds.position))
                    .map_err(|e| e.to_string())?;
                window
                    .set_size(Size::Physical(bounds.size))
                    .map_err(|e| e.to_string())?;
                window.set_focus().map_err(|e| e.to_string())?;
                return Ok(());
            }

            if window.is_fullscreen().unwrap_or(false) {
                return window.set_fullscreen(false).map_err(|e| e.to_string());
            }
            if window.is_maximized().unwrap_or(false) {
                return window.unmaximize().map_err(|e| e.to_string());
            }
            return Ok(());
        }

        let position = window.outer_position().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;
        let monitor = window
            .current_monitor()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No current monitor found".to_string())?;
        let work_area = monitor.work_area();
        store.insert(label, WindowBounds { position, size });

        if window.is_fullscreen().unwrap_or(false) {
            let _ = window.set_fullscreen(false);
        }
        window
            .set_position(Position::Physical(work_area.position))
            .map_err(|e| e.to_string())?;
        window
            .set_size(Size::Physical(work_area.size))
            .map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(all(not(windows), not(any(target_os = "android", target_os = "ios"))))]
    {
        if window.is_maximized().unwrap_or(false) {
            window.unmaximize().map_err(|e| e.to_string())
        } else {
            window.maximize().map_err(|e| e.to_string())
        }
    }
}

#[tauri::command]
pub async fn window_get_state(window: Window) -> Result<WindowState, String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        Ok(WindowState {
            is_fill: false,
            is_lyric_fullscreen: false,
        })
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        Ok(build_window_state(&window))
    }
}

#[tauri::command]
pub async fn window_toggle_lyric_fullscreen(
    window: Window,
    force: Option<bool>,
) -> Result<WindowState, String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        let _ = force;
        Ok(WindowState {
            is_fill: false,
            is_lyric_fullscreen: false,
        })
    }

    #[cfg(all(windows, not(any(target_os = "android", target_os = "ios"))))]
    {
        let label = window.label().to_string();
        let currently = lyric_fullscreen_store()
            .lock()
            .map_err(|e| e.to_string())?
            .contains_key(&label);
        let target = force.unwrap_or(!currently);

        if target == currently {
            return Ok(build_window_state(&window));
        }

        let mut previous_bounds: Option<LyricFullscreenBounds> = None;

        if target {
            let position = window.outer_position().map_err(|e| e.to_string())?;
            let size = window.outer_size().map_err(|e| e.to_string())?;
            lyric_fullscreen_store()
                .lock()
                .map_err(|e| e.to_string())?
                .insert(label.clone(), LyricFullscreenBounds { position, size });
        } else {
            previous_bounds = lyric_fullscreen_store()
                .lock()
                .map_err(|e| e.to_string())?
                .remove(&label);
        }

        if let Err(error) = window.set_fullscreen(target) {
            if target {
                let monitor = window
                    .current_monitor()
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "No current monitor".to_string())?;
                let screen_pos = monitor.position();
                let screen_size = monitor.size();
                window
                    .set_position(Position::Physical(PhysicalPosition {
                        x: screen_pos.x,
                        y: screen_pos.y,
                    }))
                    .map_err(|e| e.to_string())?;
                window
                    .set_size(Size::Physical(PhysicalSize {
                        width: screen_size.width,
                        height: screen_size.height,
                    }))
                    .map_err(|e| e.to_string())?;
            } else if let Some(previous) = previous_bounds {
                window
                    .set_position(Position::Physical(previous.position))
                    .map_err(|e| e.to_string())?;
                window
                    .set_size(Size::Physical(previous.size))
                    .map_err(|e| e.to_string())?;
            } else {
                return Err(error.to_string());
            }
        }

        window.set_focus().map_err(|e| e.to_string())?;

        return Ok(build_window_state(&window));
    }

    #[cfg(all(not(windows), not(any(target_os = "android", target_os = "ios"))))]
    {
        let target = force.unwrap_or(!window.is_fullscreen().unwrap_or(false));
        window.set_fullscreen(target).map_err(|e| e.to_string())?;
        Ok(build_window_state(&window))
    }
}

#[tauri::command]
pub async fn window_close(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        window.close().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn window_hide(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        window.hide().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn window_show(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn window_set_always_on_top(window: Window, always_on_top: bool) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        let _ = always_on_top;
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        window
            .set_always_on_top(always_on_top)
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn emit_app_event(app: AppHandle, event: String, payload: Option<String>) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        let _ = event;
        let _ = payload;
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        match payload {
            Some(value) => app.emit(&event, value).map_err(|e| e.to_string()),
            None => app.emit(&event, ()).map_err(|e| e.to_string()),
        }
    }
}

#[tauri::command]
pub async fn window_start_dragging(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        window.start_dragging().map_err(|e| e.to_string())
    }
}

#[cfg(windows)]
#[tauri::command]
pub async fn mini_player_check_dock_after_drag(window: Window) -> Result<(), String> {
    let Some(webview_window) = window.app_handle().get_webview_window(window.label()) else {
        return Err(String::from("mini-player window not found"));
    };
    mini_player_detect_dock_after_drag(webview_window).await
}

#[cfg(windows)]
#[tauri::command]
pub async fn mini_player_start_dragging(window: Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn mini_player_check_dock_after_drag(_window: Window) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn mini_player_start_dragging(window: Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[cfg(windows)]
pub fn install_mini_player_dock_tracking<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    if window.label() != "mini-player" {
        return;
    }

    let tracked_window = window.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Moved(_)) {
            schedule_mini_player_dock_detection(tracked_window.clone());
        }
    });
}

#[cfg(not(windows))]
pub fn reset_mini_player_dock_state_for_label(_label: &str) {}

#[cfg(not(windows))]
pub fn install_mini_player_dock_tracking<R: tauri::Runtime>(_window: &tauri::WebviewWindow<R>) {}

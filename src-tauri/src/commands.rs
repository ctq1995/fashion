use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::{AppHandle, Window};

use crate::app_storage;

#[cfg(windows)]
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
#[cfg(windows)]
use tauri::{PhysicalPosition, PhysicalSize, Position, Size};

const API_BASE_LO: &str = "https://music-api.gdstudio.xyz/api.php";
const API_BASE_CN: &str = "https://music-api-cn.gdstudio.xyz/api.php";
const API_BASE_HK: &str = "https://music-api-hk.gdstudio.xyz/api.php";
const API_BASE_US: &str = "https://music-api-us.gdstudio.xyz/api.php";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Fashion/0.1";

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
pub struct MusicUrl {
    pub url: Option<String>,
    pub br: Option<Value>,
    pub size: Option<Value>,
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

fn signature_for_seed(seed: &str) -> String {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn build_api_url(source: &str, params: &[(&str, String)]) -> Result<Url, String> {
    let mut url = Url::parse(api_base_for_source(source)).map_err(|e| e.to_string())?;
    let signature_seed = params
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    let signature = signature_for_seed(&signature_seed);

    {
        let mut query = url.query_pairs_mut();
        for (key, value) in params {
            query.append_pair(key, value);
        }
        query.append_pair("s", &signature);
    }

    Ok(url)
}

async fn fetch_json<T: DeserializeOwned>(client: &Client, url: Url) -> Result<T, String> {
    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let snippet = &text[..text.len().min(240)];
        return Err(format!("请求失败({status}): {snippet}"));
    }

    serde_json::from_str(&text)
        .map_err(|e| format!("解析失败: {} | 响应: {}", e, &text[..text.len().min(300)]))
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

#[cfg(windows)]
#[derive(Clone, Copy)]
struct WindowBounds {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
}

#[cfg(windows)]
#[derive(Clone, Copy)]
struct LyricFullscreenBounds {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
}

#[cfg(windows)]
const FILL_BOUNDS_TOLERANCE: i64 = 24;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub is_fill: bool,
    pub is_lyric_fullscreen: bool,
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
fn is_fill_bounds(window: &Window) -> bool {
    if window.is_fullscreen().unwrap_or(false) || window.is_maximized().unwrap_or(false) {
        return true;
    }

    let Ok(Some(monitor)) = window.current_monitor() else {
        return false;
    };
    let Ok(position) = window.outer_position() else {
        return false;
    };
    let Ok(size) = window.outer_size() else {
        return false;
    };

    // 匹配工作区（最大化）或完整屏幕（全屏）
    let work_area = monitor.work_area();
    let screen_pos = monitor.position();
    let screen_size = monitor.size();

    let tolerance = FILL_BOUNDS_TOLERANCE;

    let fits_work_area = (position.x - work_area.position.x).abs() as i64 <= tolerance
        && (position.y - work_area.position.y).abs() as i64 <= tolerance
        && (size.width as i64 - work_area.size.width as i64).abs() <= tolerance
        && (size.height as i64 - work_area.size.height as i64).abs() <= tolerance;

    let fits_screen = (position.x - screen_pos.x).abs() as i64 <= tolerance
        && (position.y - screen_pos.y).abs() as i64 <= tolerance
        && (size.width as i64 - screen_size.width as i64).abs() <= tolerance
        && (size.height as i64 - screen_size.height as i64).abs() <= tolerance;

    let pos_x = position.x as i64;
    let pos_y = position.y as i64;
    let right = pos_x + size.width as i64;
    let bottom = pos_y + size.height as i64;

    let work_left = work_area.position.x as i64;
    let work_top = work_area.position.y as i64;
    let work_right = work_left + work_area.size.width as i64;
    let work_bottom = work_top + work_area.size.height as i64;
    let covers_work_area = pos_x <= work_left + tolerance
        && pos_y <= work_top + tolerance
        && right >= work_right - tolerance
        && bottom >= work_bottom - tolerance;

    let screen_left = screen_pos.x as i64;
    let screen_top = screen_pos.y as i64;
    let screen_right = screen_left + screen_size.width as i64;
    let screen_bottom = screen_top + screen_size.height as i64;
    let covers_screen = pos_x <= screen_left + tolerance
        && pos_y <= screen_top + tolerance
        && right >= screen_right - tolerance
        && bottom >= screen_bottom - tolerance;

    fits_work_area || fits_screen || covers_work_area || covers_screen
}

#[cfg(windows)]
fn is_lyric_fullscreen(window: &Window) -> bool {
    let Ok(store) = lyric_fullscreen_store().lock() else {
        return false;
    };
    store.contains_key(window.label())
}

fn build_window_state(window: &Window) -> WindowState {
    #[cfg(windows)]
    {
        let lyric_fullscreen = is_lyric_fullscreen(window);
        let fill_bounds = is_fill_bounds(window);
        return WindowState {
            is_fill: fill_bounds || lyric_fullscreen,
            is_lyric_fullscreen: lyric_fullscreen,
        };
    }

    #[cfg(not(windows))]
    {
        let is_fullscreen = window.is_fullscreen().unwrap_or(false);
        let is_maximized = window.is_maximized().unwrap_or(false);
        WindowState {
            is_fill: is_fullscreen || is_maximized,
            is_lyric_fullscreen: is_fullscreen,
        }
    }
}

#[tauri::command]
pub async fn search_music(
    source: String,
    keyword: String,
    count: Option<u32>,
    page: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let client = make_client()?;
    let count = count.unwrap_or(30);
    let page = page.unwrap_or(1);

    let url = build_api_url(
        &source,
        &[
            ("types", "search".to_string()),
            ("source", source.clone()),
            ("name", keyword.clone()),
            ("count", count.to_string()),
            ("pages", page.to_string()),
        ],
    )?;

    fetch_json::<Vec<SearchResult>>(&client, url).await
}

#[tauri::command]
pub async fn get_music_url(
    source: String,
    id: String,
    br: Option<u32>,
) -> Result<MusicUrl, String> {
    let client = make_client()?;
    let br = br.unwrap_or(320);

    let url = build_api_url(
        &source,
        &[
            ("types", "url".to_string()),
            ("source", source.clone()),
            ("id", id.clone()),
            ("br", br.to_string()),
        ],
    )?;

    let mut result = fetch_json::<MusicUrl>(&client, url).await?;

    if strip_source_suffix(&source) == "kuwo" {
        if let Some(url) = result.url.as_deref() {
            result.url = Some(normalize_kuwo_url(url));
        }
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
) -> Result<String, String> {
    let bitrate = bitrate.unwrap_or(320);
    let client = make_client()?;
    let music_url = get_music_url(source, id, Some(bitrate)).await?;
    let download_url = music_url
        .url
        .ok_or_else(|| "未获取到下载链接".to_string())?;

    let response = client
        .get(&download_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("下载失败: {}", response.status()));
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let download_dir = app_storage::current_download_directory(&app)
        .map_err(|e| format!("无法定位下载目录: {e}"))?;
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;

    let file_stem = format!(
        "{} - {}",
        sanitize_filename_part(&title),
        sanitize_filename_part(&artist)
    );
    let extension = infer_extension(&download_url, content_type.as_deref(), bitrate);
    let output_path = unique_download_path(&download_dir, &file_stem, &extension);

    tokio::fs::write(&output_path, &bytes)
        .await
        .map_err(|e| e.to_string())?;

    Ok(output_path.display().to_string())
}

#[tauri::command]
pub async fn get_pic_url(source: String, id: String, size: Option<u32>) -> Result<PicUrl, String> {
    let client = make_client()?;
    let size = size.unwrap_or(500);

    let url = build_api_url(
        &source,
        &[
            ("types", "pic".to_string()),
            ("source", source.clone()),
            ("id", id),
            ("size", size.to_string()),
        ],
    )?;

    fetch_json::<PicUrl>(&client, url).await
}

#[tauri::command]
pub async fn get_lyric(source: String, id: String) -> Result<LyricResult, String> {
    let client = make_client()?;

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
pub async fn get_user_playlists(uid: String) -> Result<Value, String> {
    let client = make_client()?;
    let source = "netease".to_string();

    let url = build_api_url(&source, &[("types", "userlist".to_string()), ("uid", uid)])?;

    fetch_json::<Value>(&client, url).await
}

#[tauri::command]
pub async fn get_aux_lyric(
    artist: String,
    title: String,
    album_name: Option<String>,
    duration: Option<u32>,
) -> Result<serde_json::Value, String> {
    let client = make_client()?;
    let mut url = Url::parse("https://lrclib.net/api/get").map_err(|e| e.to_string())?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("artist_name", &artist);
        q.append_pair("track_name", &title);
        if let Some(ref album) = album_name {
            if !album.is_empty() {
                q.append_pair("album_name", album);
            }
        }
        if let Some(dur) = duration {
            q.append_pair("duration", &dur.to_string());
        }
    }

    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().as_u16() == 404 {
        return Ok(serde_json::json!({ "source": "aux" }));
    }

    let text = response.text().await.map_err(|e| e.to_string())?;
    let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let lyric = data
        .get("syncedLyrics")
        .or_else(|| data.get("plainLyrics"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(serde_json::json!({
        "lyric": lyric,
        "tlyric": "",
        "source": "aux"
    }))
}

#[tauri::command]
pub async fn get_recommend_playlist(
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
pub async fn search_once(keyword: String, source: String) -> Result<Vec<SearchResult>, String> {
    let client = make_client()?;

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
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_maximize(window: Window) -> Result<(), String> {
    #[cfg(windows)]
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

    #[cfg(not(windows))]
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
    Ok(build_window_state(&window))
}

#[tauri::command]
pub async fn window_toggle_lyric_fullscreen(
    window: Window,
    force: Option<bool>,
) -> Result<WindowState, String> {
    #[cfg(windows)]
    {
        let label = window.label().to_string();
        let currently = lyric_fullscreen_store()
            .lock()
            .map_err(|e| e.to_string())?
            .contains_key(&label);
        let target = force.unwrap_or(!currently);
        eprintln!("[fullscreen] window_toggle_lyric_fullscreen: force={:?} currently={} target={}", force, currently, target);

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

    #[cfg(not(windows))]
    {
        let target = force.unwrap_or(!window.is_fullscreen().unwrap_or(false));
        window.set_fullscreen(target).map_err(|e| e.to_string())?;
        Ok(build_window_state(&window))
    }
}

#[tauri::command]
pub async fn window_close(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_start_dragging(window: Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

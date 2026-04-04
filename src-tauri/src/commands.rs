use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Window};
use tokio::time::sleep;

use crate::app_storage;

#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use tauri::{PhysicalPosition, PhysicalSize, Position, Size};

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
    matches!(strip_source_suffix(source), "bilibili" | "kuwo")
        || (strip_source_suffix(source) == "ytmusic"
            && url.contains("googlevideo.com/videoplayback"))
}

fn normalize_music_url(source: &str, raw_url: &str) -> String {
    let mut url = raw_url.trim().to_string();
    if url.is_empty() {
        return url;
    }

    if is_cache_source(source) && !url.starts_with("http") {
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
        url = format!("{PROXY_BASE}/{url}");
    }

    url
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

fn is_debug_source(source: &str) -> bool {
    strip_source_suffix(source) == "gequbao"
}

fn debug_source_log(source: &str, stage: &str, detail: impl AsRef<str>) {
    if is_debug_source(source) {
        eprintln!(
            "[gequbao-debug] {stage} | source={} | {}",
            strip_source_suffix(source),
            detail.as_ref()
        );
    }
}

fn shorten_debug_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() <= 36 {
        return trimmed.to_string();
    }

    format!("{}...{}", &trimmed[..18], &trimmed[trimmed.len() - 10..])
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
        debug_source_log(
            source,
            "cache-hit",
            format!("id={id} br={bitrate} path={}", cached.path.display()),
        );
        return Ok(cached);
    }

    debug_source_log(
        source,
        "cache-fetch-start",
        format!(
            "id={id} br={bitrate} remote={}",
            shorten_debug_value(remote_url)
        ),
    );

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

    debug_source_log(
        source,
        "cache-fetch-done",
        format!(
            "id={id} br={bitrate} path={} final={}",
            output_path.display(),
            shorten_debug_value(&metadata.remote_url)
        ),
    );

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
    if remote_url.trim().is_empty() || !remote_url.starts_with("http") {
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
        .header("Referer", API_REFERER)
        .header("Origin", API_ORIGIN)
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

fn is_gequbao_source(source: &str) -> bool {
    strip_source_suffix(source) == "gequbao"
}

fn gequbao_url(path: &str) -> Result<Url, String> {
    Url::parse(GEQUBAO_BASE)
        .and_then(|base| base.join(path))
        .map_err(|e| e.to_string())
}

fn selector(pattern: &str) -> Result<Selector, String> {
    Selector::parse(pattern).map_err(|e| e.to_string())
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

fn parse_gequbao_search_results(
    html: &str,
    count: u32,
    page: u32,
) -> Result<Vec<SearchResult>, String> {
    let document = Html::parse_document(html);
    let row_selector = selector(
        "div.card.mb-1 div.card-text > div.row.no-gutters.py-2d5.border-top.align-items-center",
    )?;
    let link_selector = selector("a[href^=\"/music/\"]")?;
    let title_selector = selector("span.text-primary")?;
    let artist_selector = selector("small.text-jade")?;

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
            .unwrap_or_else(|| id.clone());

        let artist = row
            .select(&artist_selector)
            .next()
            .map(|element| collect_joined_text(element.text(), ""))
            .filter(|value| !value.is_empty())
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

fn normalize_gequbao_cover_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(mut url) = Url::parse(trimmed) {
        if url.scheme() == "http" {
            let _ = url.set_scheme("https");
        }
        return Some(url.to_string());
    }

    Url::parse(GEQUBAO_BASE)
        .ok()
        .and_then(|base| base.join(trimmed).ok())
        .map(|url| url.to_string())
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

fn decode_basic_html_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
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

fn parse_gequbao_lyric(html: &str) -> Result<String, String> {
    let document = Html::parse_document(html);
    let lyric_selector = selector("#content-lrc")?;
    let Some(element) = document.select(&lyric_selector).next() else {
        return Ok(String::new());
    };

    let raw = element
        .inner_html()
        .replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n");
    let without_tags = strip_html_tags(&raw);
    Ok(normalize_lyric_text(&decode_basic_html_entities(
        &without_tags,
    )))
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

fn parse_gequbao_app_data(html: &str) -> Result<GequbaoAppData, String> {
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

async fn fetch_gequbao_music_detail(
    client: &Client,
    id: &str,
) -> Result<GequbaoMusicDetail, String> {
    let path = format!("/music/{id}");
    let html = fetch_text(client, gequbao_url(&path)?, Some(GEQUBAO_BASE)).await?;
    let app_data = parse_gequbao_app_data(&html)?;
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
        gequbao_url("/api/s")?,
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
            gequbao_url(&path)
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
                    gequbao_url("/api/query-map")?,
                    &[("keyword", wait_keyword.clone())],
                    GEQUBAO_BASE,
                )
                .await?;

                if poll.code == 1 {
                    return gequbao_url(&format!("/s/{}", urlencoding::encode(&wait_keyword)));
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

    // gequbao sometimes returns a Kuwo resource endpoint directly. When response=res,
    // the URL itself streams audio bytes and should be used as-is.
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
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(trimmed.to_string());
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
    let detail = fetch_gequbao_music_detail(client, id).await?;
    debug_source_log(
        "gequbao",
        "detail",
        format!(
            "requested_id={id} resolved_id={} play_id={}",
            detail.id,
            shorten_debug_value(&detail.play_id)
        ),
    );
    let referer = format!("{GEQUBAO_BASE}/music/{}", detail.id);
    let response: GequbaoApiResponse<GequbaoPlayUrlData> = post_form_json(
        client,
        gequbao_url("/api/play-url")?,
        &[("id", detail.play_id)],
        &referer,
    )
    .await?;

    if response.code != 1 {
        debug_source_log(
            "gequbao",
            "play-url-failed",
            format!(
                "id={id} code={} msg={}",
                response.code,
                response.msg.as_deref().unwrap_or("")
            ),
        );
        return Err(response
            .msg
            .unwrap_or_else(|| "Failed to fetch gequbao play url".to_string()));
    }

    let raw_url = response
        .data
        .map(|data| data.url)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Missing gequbao play url".to_string())?;
    debug_source_log(
        "gequbao",
        "play-url-raw",
        format!("id={id} raw={}", shorten_debug_value(&raw_url)),
    );
    let url = convert_kuwo_play_url(client, &raw_url).await?;
    debug_source_log(
        "gequbao",
        "play-url-final",
        format!("id={id} final={}", shorten_debug_value(&url)),
    );

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
        result.url = Some(normalize_music_url(source, url));
    }
    result.local_path = None;

    Ok(result)
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

    if is_gequbao_source(&source) {
        return search_gequbao_music(&client, &keyword, count, page).await;
    }

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
    app: AppHandle,
    source: String,
    id: String,
    br: Option<u32>,
) -> Result<MusicUrl, String> {
    let br = br.unwrap_or(320);
    if let Some(cached) = read_audio_cache(&app, &source, &id, br)? {
        debug_source_log(
            &source,
            "get-music-url-cache-return",
            format!("id={id} br={br} local={}", cached.path.display()),
        );
        return Ok(MusicUrl {
            url: None,
            local_path: Some(cached.path.display().to_string()),
            br: None,
            size: None,
        });
    }

    let client = make_client()?;
    let result = resolve_music_url(&client, &source, &id, br).await?;
    debug_source_log(
        &source,
        "get-music-url-remote-return",
        format!(
            "id={id} br={br} remote={} local_path={}",
            result
                .url
                .as_deref()
                .map(shorten_debug_value)
                .unwrap_or_else(|| "<none>".to_string()),
            result.local_path.as_deref().unwrap_or("<none>")
        ),
    );
    if let Some(remote_url) = result.url.clone() {
        debug_source_log(
            &source,
            "get-music-url-cache-fill-queued",
            format!(
                "id={id} br={br} remote={}",
                shorten_debug_value(&remote_url)
            ),
        );
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
) -> Result<String, String> {
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
        let client = make_client()?;
        let music_url = resolve_music_url(&client, &source, &id, bitrate).await?;
        let download_url = music_url
            .url
            .ok_or_else(|| "未获取到下载链接".to_string())?;
        cache_audio_file(&app, &client, &source, &id, bitrate, &download_url).await?
    };
    let extension = cached.metadata.extension.clone();
    let output_path = unique_download_path(&download_dir, &file_stem, &extension);

    tokio::fs::copy(&cached.path, &output_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(output_path.display().to_string())
}

#[tauri::command]
pub async fn get_pic_url(source: String, id: String, size: Option<u32>) -> Result<PicUrl, String> {
    let client = make_client()?;
    let size = size.unwrap_or(500);

    if is_gequbao_source(&source) {
        let detail = fetch_gequbao_music_detail(&client, &id).await?;
        return Ok(PicUrl {
            url: detail.cover_url,
        });
    }

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

    if is_gequbao_source(&source) {
        let detail = fetch_gequbao_music_detail(&client, &id).await?;
        return Ok(LyricResult {
            lyric: Some(detail.lyric),
            tlyric: Some(String::new()),
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
        return Ok(());
    }

    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_maximize(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        return Ok(());
    }

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
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        return Ok(WindowState {
            is_fill: false,
            is_lyric_fullscreen: false,
        });
    }

    Ok(build_window_state(&window))
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
        return Ok(WindowState {
            is_fill: false,
            is_lyric_fullscreen: false,
        });
    }

    #[cfg(windows)]
    {
        let label = window.label().to_string();
        let currently = lyric_fullscreen_store()
            .lock()
            .map_err(|e| e.to_string())?
            .contains_key(&label);
        let target = force.unwrap_or(!currently);
        eprintln!(
            "[fullscreen] window_toggle_lyric_fullscreen: force={:?} currently={} target={}",
            force, currently, target
        );

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
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        return Ok(());
    }

    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_start_dragging(window: Window) -> Result<(), String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = window;
        return Ok(());
    }

    window.start_dragging().map_err(|e| e.to_string())
}

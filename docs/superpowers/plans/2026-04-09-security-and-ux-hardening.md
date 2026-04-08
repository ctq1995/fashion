# Security and UX Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为当前 Tauri 音乐播放器补齐关键安全收口与高收益用户体验改进，降低 WebView 风险、收紧远程媒体边界，并改善搜索、下载与设置页交互反馈。

**Architecture:** 以最小改动优先，不做大规模重构。安全项集中在 Tauri 配置、播放 URL 协议校验、发布版日志收口与命令边界梳理；体验项集中在下载反馈、搜索失败引导、结果行操作密度和设置页层次整理。所有改动遵循先测后改，优先在现有文件中补充能力，仅在必要时增加小型辅助模块。

**Tech Stack:** Tauri 2、Rust、Vue 3、TypeScript、Vitest

---

## File Map

- Modify: `src-tauri/tauri.conf.json`
  - 为应用补最小 CSP
  - 保持现有 `assetProtocol.scope` 不回退
- Modify: `src/stores/player.ts`
  - 为远程播放 URL 增加协议白名单校验
  - 为 gequbao 调试日志增加发布版收口
- Modify: `src/components/DownloadButton.vue`
  - 增加下载成功/失败的明确反馈
  - 展示保存位置和错误原因
- Modify: `src/components/SearchPanel.vue`
  - 为搜索失败态补“切换音源重试”等直接操作
  - 精简结果行操作密度
  - 为 limited 音源增加轻提示
- Modify: `src/components/SettingsPanel.vue`
  - 增加设置分组导航与常用项层次
- Modify: `src/api/music.ts`
  - 如需要，补充供 UI 使用的音源状态提示字段消费方式
- Modify: `src/utils/miniPlayer.test.ts`
  - 如已有源码级字符串测试模式适用，可补最小回归断言
- Create: `src/utils/security.ts`
  - 如收口逻辑不适合留在 `player.ts`，提取最小 URL 协议校验函数
- Create: `src/components/SettingsSectionNav.vue`
  - 如设置页锚点导航需要独立组件，则新增小型组件

---

### Task 1: 为 Tauri WebView 补最小 CSP

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Test: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 先写出目标 CSP 内容，保留现有开发模式与资源加载能力**

将 [tauri.conf.json](src-tauri/tauri.conf.json) 中的：

```json
"security": {
  "csp": null,
  "assetProtocol": {
    "enable": true,
    "scope": [
      "$APPDATA/cache/audio/**/*"
    ]
  }
}
```

改为：

```json
"security": {
  "csp": "default-src 'self' asset: http://asset.localhost; img-src 'self' asset: http://asset.localhost data: https:; media-src 'self' asset: http://asset.localhost https:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' https: http://localhost:5173 ws://localhost:5173; font-src 'self' data:",
  "assetProtocol": {
    "enable": true,
    "scope": [
      "$APPDATA/cache/audio/**/*"
    ]
  }
}
```

- [ ] **Step 2: 运行前端构建，确认 CSP 仅为配置更新不影响产物生成**

Run: `npm run build`
Expected: 构建通过；如仍有项目内既有测试/类型问题，至少不应新增由 CSP 配置导致的构建错误。

- [ ] **Step 3: 运行 Tauri Rust 检查，确认配置文件格式未破坏桌面构建**

Run: `cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`
Expected: 通过；如失败，错误不应指向 `tauri.conf.json` JSON 格式。

- [ ] **Step 4: 提交 CSP 收口**

```bash
git -C "E:/Polaris/music" add src-tauri/tauri.conf.json
git -C "E:/Polaris/music" commit -m "security: add minimal tauri csp"
```

### Task 2: 为远程播放 URL 增加协议白名单校验

**Files:**
- Modify: `src/stores/player.ts`
- Create: `src/utils/security.ts`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 先写最小失败测试，约束只接受 http/https/file-like 可播放地址**

在 `src/utils/miniPlayer.test.ts` 追加源码级回归：

```ts
import securitySource from './security.ts?raw';

it('allows only http and https remote playback urls', () => {
  expect(securitySource).toContain("const ALLOWED_REMOTE_PROTOCOLS = ['http:', 'https:'] as const;");
  expect(securitySource).toContain('export function sanitizeRemoteMediaUrl');
  expect(securitySource).toContain("return ALLOWED_REMOTE_PROTOCOLS.includes(parsed.protocol as (typeof ALLOWED_REMOTE_PROTOCOLS)[number])");
});
```

- [ ] **Step 2: 运行测试确认先失败**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: FAIL，提示 `security.ts?raw` 或目标字符串尚不存在。

- [ ] **Step 3: 新增最小安全工具模块**

创建 `src/utils/security.ts`：

```ts
const ALLOWED_REMOTE_PROTOCOLS = ['http:', 'https:'] as const;

export function sanitizeRemoteMediaUrl(url?: string) {
  if (!url) return '';

  try {
    const parsed = new URL(url);
    return ALLOWED_REMOTE_PROTOCOLS.includes(parsed.protocol as (typeof ALLOWED_REMOTE_PROTOCOLS)[number])
      ? parsed.toString()
      : '';
  } catch {
    return '';
  }
}

export function isProductionLoggingEnabled() {
  return import.meta.env.DEV;
}
```

- [ ] **Step 4: 在播放器里接入 URL 收口**

把 `src/stores/player.ts` 的导入和函数调整为：

```ts
import { sanitizeRemoteMediaUrl, isProductionLoggingEnabled } from '@/utils/security';
```

并把：

```ts
function resolvePlayableAudioUrl(url?: string, localPath?: string) {
  if (localPath) {
    return convertFileSrc(localPath);
  }
  return url ?? '';
}
```

改为：

```ts
function resolvePlayableAudioUrl(url?: string, localPath?: string) {
  if (localPath) {
    return convertFileSrc(localPath);
  }
  return sanitizeRemoteMediaUrl(url);
}
```

- [ ] **Step 5: 收口 gequbao 调试日志到开发环境**

把 `player.ts` 中：

```ts
function logPlaybackDebug(stage: string, payload?: unknown) {
  if (payload === undefined) {
    console.log('[gequbao-debug]', stage);
    return;
  }
  console.log('[gequbao-debug]', stage, payload);
}

function logPlaybackError(stage: string, payload?: unknown) {
  if (payload === undefined) {
    console.error('[gequbao-debug]', stage);
    return;
  }
  console.error('[gequbao-debug]', stage, payload);
}
```

改为：

```ts
function logPlaybackDebug(stage: string, payload?: unknown) {
  if (!isProductionLoggingEnabled()) return;
  if (payload === undefined) {
    console.log('[gequbao-debug]', stage);
    return;
  }
  console.log('[gequbao-debug]', stage, payload);
}

function logPlaybackError(stage: string, payload?: unknown) {
  if (!isProductionLoggingEnabled()) return;
  if (payload === undefined) {
    console.error('[gequbao-debug]', stage);
    return;
  }
  console.error('[gequbao-debug]', stage, payload);
}
```

- [ ] **Step 6: 运行测试与构建验证**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: 新增源码级断言通过；如存在既有 unrelated 失败，需记录但不应由本任务新增。

Run: `npm run build`
Expected: 前端成功构建，`security.ts` 别名解析正常。

- [ ] **Step 7: 提交播放边界收口**

```bash
git -C "E:/Polaris/music" add src/stores/player.ts src/utils/security.ts src/utils/miniPlayer.test.ts
git -C "E:/Polaris/music" commit -m "security: restrict remote playback urls"
```

### Task 3: 增强下载成功与失败反馈

**Files:**
- Modify: `src/components/DownloadButton.vue`
- Modify: `src-tauri/src/commands.rs`
- Test: `src/components/DownloadButton.vue`

- [ ] **Step 1: 先补后端返回结构，让前端拿到保存路径**

将 `download_music` 的返回类型由纯字符串路径整理为：

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub path: String,
    pub file_name: String,
}
```

并把 `download_music` 签名改为：

```rust
pub async fn download_music(
    app: AppHandle,
    source: String,
    id: String,
    bitrate: u32,
    title: String,
    artist: String,
) -> Result<DownloadResult, String>
```

成功返回：

```rust
Ok(DownloadResult {
    path: output_path.to_string_lossy().to_string(),
    file_name: output_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "downloaded-audio".to_string()),
})
```

- [ ] **Step 2: 前端把下载结果改成可展示消息**

将 `DownloadButton.vue` 中：

```ts
await invoke<string>('download_music', {
  source: props.track.source,
  id: props.track.id,
  bitrate,
  title: props.track.name,
  artist: props.track.artist,
});
status.value = 'done';
```

改为：

```ts
interface DownloadResult {
  path: string;
  fileName: string;
}

const feedbackText = ref('');

const buttonTitle = computed(() => {
  if (downloading.value) return '下载中';
  if (status.value === 'done') return feedbackText.value || '已保存到下载目录';
  if (status.value === 'error') return feedbackText.value || '下载失败，请重试';
  return '下载';
});

const result = await invoke<DownloadResult>('download_music', {
  source: props.track.source,
  id: props.track.id,
  bitrate,
  title: props.track.name,
  artist: props.track.artist,
});
feedbackText.value = `已保存：${result.fileName}`;
status.value = 'done';
```

失败路径改为：

```ts
} catch (error) {
  console.error('download_music failed', error);
  feedbackText.value = error instanceof Error ? error.message : '下载失败，请重试';
  status.value = 'error';
}
```

- [ ] **Step 3: 在菜单下方增加简短反馈文本**

在 `DownloadButton.vue` 模板里按钮后追加：

```vue
<span v-if="status !== 'idle'" class="download-feedback">
  {{ feedbackText || buttonTitle }}
</span>
```

并在样式里补：

```css
.download-control {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.download-feedback {
  max-width: 220px;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
```

- [ ] **Step 4: 运行构建验证**

Run: `npm run build`
Expected: 通过，`invoke<DownloadResult>` 类型正确。

Run: `cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`
Expected: 通过，`DownloadResult` 序列化正常。

- [ ] **Step 5: 提交下载反馈优化**

```bash
git -C "E:/Polaris/music" add src/components/DownloadButton.vue src-tauri/src/commands.rs
git -C "E:/Polaris/music" commit -m "feat: improve download feedback"
```

### Task 4: 为搜索失败态增加切源重试与 limited 提示

**Files:**
- Modify: `src/components/SearchPanel.vue`
- Modify: `src/api/music.ts`
- Test: `src/components/SearchPanel.vue`

- [ ] **Step 1: 增加当前音源状态的计算属性**

在 `SearchPanel.vue` 的导入里补：

```ts
import { getSourceMeta } from '@/api/music';
```

并增加：

```ts
const currentSourceMeta = computed(() => getSourceMeta(source.value));
const isLimitedSource = computed(() => currentSourceMeta.value?.state === 'limited');
```

- [ ] **Step 2: 在结果头部增加 limited 音源轻提示**

在 `result-header` 中 `result-sub` 后增加：

```vue
<span v-if="isLimitedSource" class="source-hint">
  当前音源可用性有限，失败时建议切换其他音源。
</span>
```

- [ ] **Step 3: 为错误条增加直接重试动作**

把：

```vue
<div v-if="errMsg" class="err-bar">{{ errMsg }}</div>
```

改为：

```vue
<div v-if="errMsg" class="err-bar err-bar--actionable">
  <span>{{ errMsg }}</span>
  <div class="err-actions">
    <button type="button" class="app-btn app-btn--ghost compact-btn" @click="retrySearch">
      重试
    </button>
    <button type="button" class="app-btn app-btn--ghost compact-btn" @click="switchSourceAndRetry">
      切换音源重试
    </button>
  </div>
</div>
```

- [ ] **Step 4: 增加切源逻辑，只在已启用音源里切换**

在 `SearchPanel.vue` 中补：

```ts
function retrySearch() {
  if (!keyword.value.trim()) return;
  void doSearch(keyword.value, source.value);
}

function switchSourceAndRetry() {
  const enabled = ui.enabledToolbarSources.filter((item) => item !== source.value);
  const next = enabled[0];
  if (!next || !keyword.value.trim()) return;
  source.value = next;
  ui.setToolbarSource(next);
  void doSearch(keyword.value, next);
}
```

- [ ] **Step 5: 为空结果态也补直接操作**

把空结果区域改为：

```vue
<div v-else-if="!filteredResults.length" class="empty-state">
  <p>没有找到相关结果</p>
  <span>换个关键词，或者切换音源再试一次。</span>
  <div class="empty-actions inline-actions">
    <button type="button" class="app-btn app-btn--ghost compact-btn" @click="switchSourceAndRetry">
      切换音源
    </button>
    <button type="button" class="app-btn app-btn--ghost compact-btn" @click="favoriteOnly = false">
      清除筛选
    </button>
  </div>
</div>
```

- [ ] **Step 6: 为新元素补最小样式**

在 `SearchPanel.vue` 样式里补：

```css
.source-hint {
  display: inline-flex;
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-warning);
}

.err-bar--actionable {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.err-actions,
.inline-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
```

- [ ] **Step 7: 运行前端构建验证**

Run: `npm run build`
Expected: 通过，新增方法和样式无报错。

- [ ] **Step 8: 提交搜索失败引导优化**

```bash
git -C "E:/Polaris/music" add src/components/SearchPanel.vue
git -C "E:/Polaris/music" commit -m "feat: improve search failure recovery"
```

### Task 5: 精简搜索结果行的操作密度

**Files:**
- Modify: `src/components/SearchPanel.vue`
- Create: `src/components/SearchRowMoreMenu.vue`
- Test: `src/components/SearchPanel.vue`

- [ ] **Step 1: 新增一个轻量“更多操作”菜单组件**

创建 `src/components/SearchRowMoreMenu.vue`：

```vue
<template>
  <div class="search-row-more">
    <button type="button" class="app-icon-btn" title="更多操作" @click.stop="open = !open">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
        <circle cx="5" cy="12" r="1.8" />
        <circle cx="12" cy="12" r="1.8" />
        <circle cx="19" cy="12" r="1.8" />
      </svg>
    </button>
    <div v-if="open" class="search-row-more__menu" @click.stop>
      <button type="button" class="search-row-more__item" @click="$emit('queue')">加入队列</button>
      <button type="button" class="search-row-more__item" @click="$emit('playlist')">加入歌单</button>
      <button type="button" class="search-row-more__item" @click="$emit('download')">下载</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

defineEmits<{
  queue: [];
  playlist: [];
  download: [];
}>();

const open = ref(false);
</script>

<style scoped>
.search-row-more {
  position: relative;
  display: inline-flex;
}

.search-row-more__menu {
  position: absolute;
  right: 0;
  top: calc(100% + 8px);
  min-width: 120px;
  padding: 6px;
  border-radius: 12px;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  box-shadow: var(--window-shadow);
  z-index: 30;
}

.search-row-more__item {
  width: 100%;
  padding: 8px 10px;
  text-align: left;
  border-radius: 10px;
  color: var(--text-secondary);
}

.search-row-more__item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
```

- [ ] **Step 2: 在搜索结果行只保留高频动作**

将 `SearchPanel.vue` 的操作区：
- 保留：收藏、播放
- 将：加入歌单、加入队列、下载
- 收进 `SearchRowMoreMenu`

替换成：

```vue
<template #actions>
  <div class="action-row">
    <button
      type="button"
      class="app-icon-btn"
      :class="{ active: isFavorite(item) }"
      title="收藏"
      @click.stop="toggleFavorite(item)"
    >
      ...
    </button>
    <SearchRowMoreMenu
      @queue="addQueue(item)"
      @playlist="openPlaylistPicker($event, item)"
      @download="triggerDownload(item)"
    />
    <button type="button" class="app-icon-btn play-btn" title="播放" @click.stop="playNow(item)">
      ...
    </button>
  </div>
</template>
```

- [ ] **Step 3: 为下载按钮改成可编程触发方式**

若 `DownloadButton` 不能直接被菜单触发，则保持组件存在但只在菜单内渲染；更小改法是：
- 保留 `DownloadButton`
- 但默认隐藏文案，仅保留在 `SearchRowMoreMenu` 内部复用

如果不想大改，可在当前任务只收起“加入歌单 / 加入队列”，下载继续保留。这是允许的最小实现。

- [ ] **Step 4: 运行前端构建验证**

Run: `npm run build`
Expected: 通过；若事件透传或参数类型报错，补齐类型后再验证。

- [ ] **Step 5: 提交操作密度优化**

```bash
git -C "E:/Polaris/music" add src/components/SearchPanel.vue src/components/SearchRowMoreMenu.vue
git -C "E:/Polaris/music" commit -m "feat: simplify search row actions"
```

### Task 6: 为设置页增加分组导航与常用项前置

**Files:**
- Modify: `src/components/SettingsPanel.vue`
- Create: `src/components/SettingsSectionNav.vue`
- Test: `src/components/SettingsPanel.vue`

- [ ] **Step 1: 创建设置分组导航组件**

创建 `src/components/SettingsSectionNav.vue`：

```vue
<template>
  <nav class="settings-section-nav">
    <button
      v-for="item in items"
      :key="item.id"
      type="button"
      class="app-chip-btn"
      :class="{ active: activeId === item.id }"
      @click="$emit('jump', item.id)"
    >
      {{ item.label }}
    </button>
  </nav>
</template>

<script setup lang="ts">
defineProps<{
  items: Array<{ id: string; label: string }>;
  activeId: string;
}>();

defineEmits<{
  jump: [id: string];
}>();
</script>

<style scoped>
.settings-section-nav {
  position: sticky;
  top: 0;
  z-index: 5;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-bottom: 12px;
  background: linear-gradient(180deg, var(--bg-panel) 70%, transparent);
}
</style>
```

- [ ] **Step 2: 在设置页顶部接入导航与锚点**

在 `SettingsPanel.vue` 中引入：

```ts
import SettingsSectionNav from '@/components/SettingsSectionNav.vue';
```

并定义：

```ts
const sectionItems = [
  { id: 'sources', label: '音源' },
  { id: 'playback', label: '播放' },
  { id: 'window', label: '桌面与关闭' },
  { id: 'lyrics', label: '歌词' },
] as const;

const activeSectionId = ref('sources');

function jumpToSection(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  activeSectionId.value = id;
}
```

在模板顶部加入：

```vue
<SettingsSectionNav :items="sectionItems" :active-id="activeSectionId" @jump="jumpToSection" />
```

并给主要 `section` 加上：

```vue
<section id="sources" class="settings-card">
<section id="playback" class="settings-card compact-card">
<section id="window" class="settings-card compact-card">
<section id="lyrics" class="settings-card compact-card">
```

- [ ] **Step 3: 将常用项前置到更靠前位置**

在设置布局上优先顺序调整为：
1. 搜索音源
2. 主题 / 音质 / 倍速
3. 桌面与关闭
4. 歌词高级项

只调整 section 顺序，不改原有业务逻辑。

- [ ] **Step 4: 运行前端构建验证**

Run: `npm run build`
Expected: 通过，新增组件导入和 DOM 锚点无错误。

- [ ] **Step 5: 提交设置页层次优化**

```bash
git -C "E:/Polaris/music" add src/components/SettingsPanel.vue src/components/SettingsSectionNav.vue
git -C "E:/Polaris/music" commit -m "feat: improve settings navigation"
```

### Task 7: 为 Rust 命令边界建立审计清单并做最小输入收口

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/commands.rs`

- [ ] **Step 1: 为下载文件名路径生成逻辑补非法字符清理复核**

检查 `commands.rs` 中 `unique_download_path(...)`、文件名拼接和 `title/artist` 组合位置，确保文件名统一经过类似以下逻辑：

```rust
fn sanitize_filename_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string()
}
```

并在拼接下载文件名时统一使用：

```rust
let safe_title = sanitize_filename_segment(&title);
let safe_artist = sanitize_filename_segment(&artist);
let file_stem = format!("{} - {}", safe_title, safe_artist).trim().to_string();
```

- [ ] **Step 2: 为远程 URL 规范化路径统一只允许 http/https**

在 Rust 侧所有远程 URL 返回前，统一复核类似：

```rust
fn ensure_supported_remote_url(url: &str) -> Result<String, String> {
    let parsed = Url::parse(url).map_err(|e| e.to_string())?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed.to_string()),
        _ => Err("Unsupported remote url scheme".to_string()),
    }
}
```

把 `normalize_music_url(...)` 与其他远程 URL 返回路径统一接上该函数。

- [ ] **Step 3: 运行 Rust 检查**

Run: `cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`
Expected: 通过，所有 URL/文件名辅助函数均已被调用。

- [ ] **Step 4: 提交命令边界收口**

```bash
git -C "E:/Polaris/music" add src-tauri/src/commands.rs
git -C "E:/Polaris/music" commit -m "security: harden command input boundaries"
```

### Task 8: 全量验证与回归总结

**Files:**
- Modify: `docs/superpowers/plans/2026-04-09-security-and-ux-hardening.md`

- [ ] **Step 1: 运行前端构建**

Run: `npm run build`
Expected: 通过；若存在既有历史问题，逐条记录，不把未解决错误误报为本次新增。

- [ ] **Step 2: 运行 Rust 检查**

Run: `cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`
Expected: 通过。

- [ ] **Step 3: 运行关键测试**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: 本次新增源码级断言通过；若仍有仓库既有导入问题，单独记录。

- [ ] **Step 4: 手工回归关键交互**

手工检查：
- 搜索失败时能看到“重试 / 切换音源重试”
- limited 音源有轻提示
- 下载完成后能看到文件名反馈
- 下载失败时能看到错误反馈
- 设置页顶部有分组导航，点击能滚动到对应分组
- 非 `http/https` 播放 URL 不会被前端直接喂给播放器
- 发布构建环境不再打印 gequbao 调试日志

- [ ] **Step 5: 更新计划中的验证结果与遗留项**

在本文件末尾追加：

```md
## Verification Notes

- Frontend build:
- Rust check:
- Vitest:
- Manual QA:
- Remaining known issues:
```

- [ ] **Step 6: 提交最终整合结果**

```bash
git -C "E:/Polaris/music" add src-tauri/tauri.conf.json src-tauri/src/commands.rs src/stores/player.ts src/utils/security.ts src/components/DownloadButton.vue src/components/SearchPanel.vue src/components/SearchRowMoreMenu.vue src/components/SettingsPanel.vue src/components/SettingsSectionNav.vue src/utils/miniPlayer.test.ts docs/superpowers/plans/2026-04-09-security-and-ux-hardening.md
git -C "E:/Polaris/music" commit -m "feat: harden security and improve ux"
```

---

## Self-Review

### Spec coverage
- Review 中提出的高优先级安全项已覆盖：CSP、远程 URL 协议校验、调试日志收口、命令边界与文件名/URL 规范化。
- Review 中提出的高收益 UX 项已覆盖：下载反馈、搜索失败恢复、limited 音源提示、结果行操作密度、设置页层次。
- Review 中提出的“commands.rs 过大”仅进行了边界收口，没有在本计划里做大拆分；这是有意控制范围，避免一次性引入大规模重构风险。

### Placeholder scan
- 已移除泛化表述，所有任务都指定了文件、具体代码片段、运行命令和预期输出。
- 唯一允许的范围弹性是 Task 5 Step 3 中对 `DownloadButton` 触发方式给了“最小实现”分支，这是为了避免计划强推大改；但仍给出了可执行的最小落地方式。

### Type consistency
- `DownloadResult` 在 Rust 和前端中使用 camelCase 对齐。
- `sanitizeRemoteMediaUrl` 与 `isProductionLoggingEnabled` 在 `player.ts` 的导入与用法一致。
- 设置导航组件 `SettingsSectionNav` 的 `active-id` 和 `jump` 事件名称已与父组件对齐。

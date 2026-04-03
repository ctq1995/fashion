<template>
  <div class="settings-panel app-scroll">
    <section class="settings-card">
      <div class="section-head">
        <h3>搜索音源</h3>
        <span class="section-side">{{ ui.enabledToolbarSources.length }} / {{ selectableSourceCount }}</span>
      </div>

      <p class="section-tip">已按当前接口状态过滤默认音源。可用源默认开启，失效源保留状态说明但不可启用。</p>

      <div class="source-grid">
        <button
          v-for="source in SOURCES"
          :key="source.value"
          type="button"
          class="source-chip"
          :class="{
            active: ui.isSourceEnabled(source.value),
            current: ui.toolbarSource === source.value,
            limited: source.state === 'limited',
            disabled: !source.selectable,
          }"
          :disabled="!source.selectable || (ui.isSourceEnabled(source.value) && ui.enabledToolbarSources.length === 1)"
          :title="sourceTitle(source)"
          @click="ui.toggleSourceEnabled(source.value)"
        >
          <span class="source-copy">
            <strong class="source-name">{{ source.label }}</strong>
            <small class="source-meta">{{ sourceStatusLabel(source.state) }} · {{ source.note }}</small>
          </span>
          <svg
            v-if="ui.toolbarSource === source.value"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="m5 12 5 5L20 7" />
          </svg>
        </button>
      </div>
    </section>

    <section class="settings-card compact-card">
      <div class="setting-row">
        <span>主题</span>
        <div class="chip-row">
          <button
            v-for="theme in themes"
            :key="theme.value"
            type="button"
            class="app-chip-btn"
            :class="{ active: ui.theme === theme.value }"
            @click="ui.setTheme(theme.value)"
          >
            {{ theme.label }}
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span>音质</span>
        <div class="chip-row">
          <button
            v-for="option in bitrateOptions"
            :key="option.value"
            type="button"
            class="app-chip-btn"
            :class="{ active: option.value === player.preferredBitrate }"
            @click="player.setPreferredBitrate(option.value)"
          >
            {{ option.label }}
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span>倍速</span>
        <div class="chip-row">
          <button
            v-for="rate in player.PLAYBACK_RATES"
            :key="rate"
            type="button"
            class="app-chip-btn"
            :class="{ active: rate === player.playbackRate }"
            @click="player.setPlaybackRate(rate)"
          >
            {{ rate.toFixed(2) }}x
          </button>
        </div>
      </div>
    </section>

    <section class="settings-card compact-card">
      <div class="section-head">
        <h3>歌词常规</h3>
        <span class="section-side">桌面歌词即时生效</span>
      </div>

      <p class="section-tip">
        右下角歌词按钮会弹出桌面歌词。锁定后窗口会忽略鼠标点击，适合悬浮在桌面或其他应用上方。
      </p>

      <div class="setting-row">
        <span>窗口层级</span>
        <div class="chip-row">
          <button
            type="button"
            class="app-chip-btn"
            :class="{ active: ui.lyricSettings.alwaysOnTop }"
            @click="ui.setLyricSettings({ alwaysOnTop: true })"
          >
            始终置顶
          </button>
          <button
            type="button"
            class="app-chip-btn"
            :class="{ active: !ui.lyricSettings.alwaysOnTop }"
            @click="ui.setLyricSettings({ alwaysOnTop: false })"
          >
            普通窗口
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span>交互模式</span>
        <div class="chip-row">
          <button
            type="button"
            class="app-chip-btn"
            :class="{ active: !ui.lyricSettings.locked }"
            @click="ui.setLyricSettings({ locked: false })"
          >
            可拖动
          </button>
          <button
            type="button"
            class="app-chip-btn"
            :class="{ active: ui.lyricSettings.locked }"
            @click="ui.setLyricSettings({ locked: true })"
          >
            锁定穿透
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span>副歌词</span>
        <div class="chip-row">
          <button
            type="button"
            class="app-chip-btn"
            :class="{ active: ui.lyricSettings.showTranslation }"
            @click="ui.setLyricSettings({ showTranslation: true })"
          >
            优先翻译
          </button>
          <button
            type="button"
            class="app-chip-btn"
            :class="{ active: !ui.lyricSettings.showTranslation }"
            @click="ui.setLyricSettings({ showTranslation: false })"
          >
            优先下一句
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span>字号</span>
        <div class="chip-row">
          <button
            v-for="option in lyricFontOptions"
            :key="option.value"
            type="button"
            class="app-chip-btn"
            :class="{ active: option.value === ui.lyricSettings.fontScale }"
            @click="ui.setLyricSettings({ fontScale: option.value })"
          >
            {{ option.label }}
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span>背景浓度</span>
        <div class="chip-row">
          <button
            v-for="option in lyricBackgroundOptions"
            :key="option.value"
            type="button"
            class="app-chip-btn"
            :class="{ active: option.value === ui.lyricSettings.backgroundOpacity }"
            @click="ui.setLyricSettings({ backgroundOpacity: option.value })"
          >
            {{ option.label }}
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span>滚动速度</span>
        <div class="chip-row">
          <button
            v-for="option in lyricScrollOptions"
            :key="option.value"
            type="button"
            class="app-chip-btn"
            :class="{ active: option.value === ui.lyricSettings.scrollSpeed }"
            @click="ui.setLyricSettings({ scrollSpeed: option.value })"
          >
            {{ option.label }}
          </button>
        </div>
      </div>

      <div class="lyric-color-panel">
        <div
          class="lyric-color-preview"
          :style="{
            '--preview-base': ui.lyricSettings.baseColor,
            '--preview-highlight': ui.lyricSettings.highlightColor,
          }"
        >
          <span class="preview-badge">歌词颜色预览</span>
          <div class="preview-main-line">
            <span class="preview-base-line">基础字色</span>
            <span class="preview-highlight-line">高亮渲染色</span>
          </div>
          <div class="preview-sub-line">桌面歌词会按这里的配色实时更新，便于区分主字和渲染进度。</div>
        </div>

        <div class="lyric-color-grid">
          <div class="lyric-color-board">
            <div class="color-board-head">
              <div class="color-board-copy">
                <strong>基础字色</strong>
                <small>主歌词、副歌词和歌曲信息</small>
              </div>

              <button
                type="button"
                class="app-chip-btn"
                @click="resetLyricColor('baseColor')"
              >
                恢复默认
              </button>
            </div>

            <div class="color-board-body">
              <label
                class="color-picker-shell"
                :style="{ '--color-chip-value': ui.lyricSettings.baseColor }"
                title="打开基础字色色板"
              >
                <input
                  type="color"
                  :value="ui.lyricSettings.baseColor"
                  @input="updateLyricColor('baseColor', $event)"
                />
                <span class="color-picker-surface" />
              </label>

              <div class="color-board-fields">
                <label class="color-hex-field">
                  <span>HEX</span>
                  <input
                    type="text"
                    maxlength="7"
                    :value="lyricColorDrafts.baseColor"
                    placeholder="#FFFFFF"
                    @input="updateLyricColorDraft('baseColor', $event)"
                    @blur="commitLyricColorDraft('baseColor')"
                    @keydown.enter="commitLyricColorDraft('baseColor')"
                  />
                </label>

                <div class="color-row">
                  <button
                    v-for="option in lyricBaseColorOptions"
                    :key="option.value"
                    type="button"
                    class="color-chip"
                    :class="{ active: option.value === ui.lyricSettings.baseColor }"
                    :style="{ '--color-chip-value': option.value }"
                    @click="applyLyricColor('baseColor', option.value)"
                  >
                    <span class="color-chip-swatch" />
                    {{ option.label }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="lyric-color-board">
            <div class="color-board-head">
              <div class="color-board-copy">
                <strong>高亮渲染色</strong>
                <small>当前进度和播放态提示色</small>
              </div>

              <button
                type="button"
                class="app-chip-btn"
                @click="resetLyricColor('highlightColor')"
              >
                恢复默认
              </button>
            </div>

            <div class="color-board-body">
              <label
                class="color-picker-shell"
                :style="{ '--color-chip-value': ui.lyricSettings.highlightColor }"
                title="打开高亮渲染色色板"
              >
                <input
                  type="color"
                  :value="ui.lyricSettings.highlightColor"
                  @input="updateLyricColor('highlightColor', $event)"
                />
                <span class="color-picker-surface" />
              </label>

              <div class="color-board-fields">
                <label class="color-hex-field">
                  <span>HEX</span>
                  <input
                    type="text"
                    maxlength="7"
                    :value="lyricColorDrafts.highlightColor"
                    placeholder="#16D6A0"
                    @input="updateLyricColorDraft('highlightColor', $event)"
                    @blur="commitLyricColorDraft('highlightColor')"
                    @keydown.enter="commitLyricColorDraft('highlightColor')"
                  />
                </label>

                <div class="color-row">
                  <button
                    v-for="option in lyricHighlightColorOptions"
                    :key="option.value"
                    type="button"
                    class="color-chip"
                    :class="{ active: option.value === ui.lyricSettings.highlightColor }"
                    :style="{ '--color-chip-value': option.value }"
                    @click="applyLyricColor('highlightColor', option.value)"
                  >
                    <span class="color-chip-swatch" />
                    {{ option.label }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="settings-card">
      <div class="section-head">
        <h3>存储</h3>
        <span class="section-side">实时生效</span>
      </div>

      <p class="section-tip">
        数据目录用于保存主题、历史、收藏、歌单和缓存，下载路径用于保存下载文件。
      </p>

      <p v-if="!storage.supported" class="storage-error">当前运行环境不支持自定义目录。</p>
      <p v-else-if="storage.errorMessage" class="storage-error">{{ storage.errorMessage }}</p>

      <div class="storage-group">
        <div class="storage-row">
          <div class="storage-copy">
            <strong>数据目录</strong>
            <span class="storage-path">{{ storage.preferences.effectiveDataDirectory || 'Not available' }}</span>
            <small>{{ storage.preferences.usesDefaultDataDirectory ? '使用默认路径' : '使用自定义路径' }}</small>
          </div>
          <div class="storage-actions">
            <button
              type="button"
              class="app-chip-btn"
              :disabled="!storage.supported || storage.busyTarget !== null"
              @click="storage.openDataDirectory"
            >
              打开
            </button>
            <button
              type="button"
              class="app-chip-btn active"
              :disabled="!storage.supported || storage.busyTarget !== null"
              @click="storage.chooseDataDirectory"
            >
              选择
            </button>
            <button
              type="button"
              class="app-chip-btn"
              :disabled="!storage.supported || storage.busyTarget !== null || storage.preferences.usesDefaultDataDirectory"
              @click="storage.resetDataDirectory"
            >
              默认
            </button>
          </div>
        </div>

        <div class="storage-row">
          <div class="storage-copy">
            <strong>下载保存路径</strong>
            <span class="storage-path">{{ storage.preferences.effectiveDownloadDirectory || 'Not available' }}</span>
            <small>{{ storage.preferences.usesDefaultDownloadDirectory ? '使用系统下载目录' : '使用自定义路径' }}</small>
          </div>
          <div class="storage-actions">
            <button
              type="button"
              class="app-chip-btn"
              :disabled="!storage.supported || storage.busyTarget !== null"
              @click="storage.openDownloadDirectory"
            >
              打开
            </button>
            <button
              type="button"
              class="app-chip-btn active"
              :disabled="!storage.supported || storage.busyTarget !== null"
              @click="storage.chooseDownloadDirectory"
            >
              选择
            </button>
            <button
              type="button"
              class="app-chip-btn"
              :disabled="!storage.supported || storage.busyTarget !== null || storage.preferences.usesDefaultDownloadDirectory"
              @click="storage.resetDownloadDirectory"
            >
              默认
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="settings-card">
      <div class="section-head">
        <h3>鸣谢</h3>
        <span class="section-side">服务支持</span>
      </div>

      <p class="section-tip">
        鸣谢 GD音乐台（music.gdstudio.xyz）提供音乐接口与相关服务支持。
      </p>

      <div class="credit-card">
        <div class="credit-copy">
          <strong>GD音乐台</strong>
          <span>music.gdstudio.xyz</span>
          <small>本应用的搜索与解析能力基于 GD音乐台 的服务。</small>
        </div>
        <button type="button" class="app-btn app-btn--ghost" @click="openCreditsUrl">
          访问网站
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener';
import { reactive, watch } from 'vue';
import { SOURCES } from '@/api/music';
import { type Bitrate, usePlayerStore } from '@/stores/player';
import { useStorageStore } from '@/stores/storage';
import { useUiStore, type AppTheme } from '@/stores/ui';
import { DEFAULT_DESKTOP_LYRIC_SETTINGS } from '@/utils/desktopLyric';

const CREDITS_URL = 'https://music.gdstudio.xyz';
const HEX_COLOR_PATTERN = /^#(?:[0-9a-f]{3}|[0-9a-f]{6})$/i;

const player = usePlayerStore();
const ui = useUiStore();
const storage = useStorageStore();
const selectableSourceCount = SOURCES.filter((source) => source.selectable).length;

type LyricColorKey = 'baseColor' | 'highlightColor';

const themes: Array<{ value: AppTheme; label: string }> = [
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
];

const bitrateOptions: Array<{ value: Bitrate; label: string }> = [
  { value: 128, label: '128K' },
  { value: 192, label: '192K' },
  { value: 320, label: '320K' },
  { value: 999, label: '无损' },
];

const lyricFontOptions = [
  { value: 0.88, label: '小' },
  { value: 1, label: '标准' },
  { value: 1.16, label: '大' },
  { value: 1.32, label: '超大' },
];

const lyricBackgroundOptions = [
  { value: 0.18, label: '通透' },
  { value: 0.26, label: '柔和' },
  { value: 0.32, label: '标准' },
  { value: 0.44, label: '聚焦' },
];

const lyricBaseColorOptions = [
  { value: '#FFFFFF', label: '纯白' },
  { value: '#FFF3D6', label: '暖白' },
  { value: '#DFF4FF', label: '冰蓝' },
  { value: '#F3E8FF', label: '浅紫' },
];

const lyricHighlightColorOptions = [
  { value: '#16D6A0', label: '薄荷' },
  { value: '#4DA3FF', label: '天蓝' },
  { value: '#FF7A59', label: '珊瑚' },
  { value: '#FFB347', label: '金橙' },
  { value: '#FF5FA2', label: '玫红' },
];

const lyricScrollOptions = [
  { value: 48, label: '慢' },
  { value: 72, label: '标准' },
  { value: 96, label: '快' },
];

const lyricColorDrafts = reactive<Record<LyricColorKey, string>>({
  baseColor: ui.lyricSettings.baseColor,
  highlightColor: ui.lyricSettings.highlightColor,
});

function sourceStatusLabel(state: 'available' | 'limited' | 'disabled') {
  if (state === 'available') return '可用';
  if (state === 'limited') return '受限';
  return '停用';
}

function sourceTitle(source: (typeof SOURCES)[number]) {
  const current = ui.toolbarSource === source.value ? '（当前）' : '';
  return `${source.label}${current} - ${source.note}`;
}

watch(
  () => ui.lyricSettings.baseColor,
  (value) => {
    lyricColorDrafts.baseColor = value;
  },
);

watch(
  () => ui.lyricSettings.highlightColor,
  (value) => {
    lyricColorDrafts.highlightColor = value;
  },
);

async function openCreditsUrl() {
  try {
    await openUrl(CREDITS_URL);
  } catch {
    if (typeof window !== 'undefined') {
      window.open(CREDITS_URL, '_blank', 'noopener,noreferrer');
    }
  }
}

function normalizeLyricColorDraft(value: string) {
  const trimmed = value.trim().toUpperCase();
  if (!trimmed) return '';
  return trimmed.startsWith('#') ? trimmed : `#${trimmed}`;
}

function resolveLyricColor(value: string) {
  const normalized = normalizeLyricColorDraft(value);
  if (!HEX_COLOR_PATTERN.test(normalized)) return null;

  if (normalized.length === 4) {
    return `#${normalized
      .slice(1)
      .split('')
      .map((segment) => `${segment}${segment}`)
      .join('')}`;
  }

  return normalized;
}

function setLyricColor(key: LyricColorKey, value: string) {
  if (key === 'baseColor') {
    ui.setLyricSettings({ baseColor: value });
    return;
  }

  ui.setLyricSettings({ highlightColor: value });
}

function applyLyricColor(key: LyricColorKey, value: string) {
  const resolved = resolveLyricColor(value);
  if (!resolved) return false;

  setLyricColor(key, resolved);
  lyricColorDrafts[key] = resolved;
  return true;
}

function updateLyricColor(key: 'baseColor' | 'highlightColor', event: Event) {
  const target = event.target as HTMLInputElement | null;
  if (!target) return;

  void applyLyricColor(key, target.value);
}

function updateLyricColorDraft(key: LyricColorKey, event: Event) {
  const target = event.target as HTMLInputElement | null;
  if (!target) return;

  const draft = normalizeLyricColorDraft(target.value);
  lyricColorDrafts[key] = draft;

  if (HEX_COLOR_PATTERN.test(draft)) {
    void applyLyricColor(key, draft);
  }
}

function commitLyricColorDraft(key: LyricColorKey) {
  if (applyLyricColor(key, lyricColorDrafts[key])) return;
  lyricColorDrafts[key] = ui.lyricSettings[key];
}

function resetLyricColor(key: LyricColorKey) {
  const value = key === 'baseColor'
    ? DEFAULT_DESKTOP_LYRIC_SETTINGS.baseColor
    : DEFAULT_DESKTOP_LYRIC_SETTINGS.highlightColor;

  setLyricColor(key, value);
  lyricColorDrafts[key] = value;
}
</script>

<style scoped>
.settings-panel {
  height: 100%;
  min-height: 0;
  overflow-y: auto;
  padding: 12px 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings-card {
  border-radius: 18px;
  border: 1px solid var(--border);
  background: linear-gradient(180deg, var(--panel-strong), rgba(255, 255, 255, 0.02));
  padding: 14px;
}

.compact-card {
  padding: 4px 14px;
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.section-head h3 {
  font-size: 15px;
  font-weight: 800;
  color: var(--text-primary);
}

.section-side {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-muted);
}

.section-tip {
  margin-bottom: 10px;
  font-size: 12px;
  color: var(--text-muted);
}

.source-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(112px, 1fr));
  gap: 8px;
}

.source-chip {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  gap: 8px;
  min-height: 64px;
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  border: 1px solid transparent;
  text-align: left;
  position: relative;
  transition: var(--transition);
}

.source-chip:hover:not(:disabled) {
  background: var(--bg-active);
  color: var(--text-primary);
}

.source-chip.active {
  background: var(--accent-dim);
  color: var(--accent);
  border-color: rgba(22, 214, 160, 0.22);
  box-shadow: inset 0 0 0 1px rgba(22, 214, 160, 0.08);
}

.source-chip.limited {
  border-color: rgba(255, 181, 71, 0.24);
}

.source-chip.disabled {
  background: rgba(255, 255, 255, 0.04);
}

.source-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.source-name {
  font-size: 13px;
  font-weight: 700;
  color: inherit;
}

.source-meta {
  font-size: 11px;
  line-height: 1.35;
  color: var(--text-muted);
}

.source-chip.active .source-meta {
  color: color-mix(in srgb, var(--accent) 76%, white 24%);
}

.source-chip.current svg {
  position: absolute;
  top: 10px;
  right: 10px;
}

.source-chip:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.setting-row {
  min-height: 58px;
  padding: 10px 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
}

.setting-row + .setting-row {
  border-top: 1px solid var(--border);
}

.setting-row > span {
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.lyric-color-panel {
  margin-top: 10px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.lyric-color-preview {
  --preview-base: #ffffff;
  --preview-highlight: #16d6a0;

  padding: 16px 18px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--preview-highlight) 18%, var(--border));
  background:
    radial-gradient(circle at top right, color-mix(in srgb, var(--preview-highlight) 24%, transparent), transparent 34%),
    linear-gradient(
      135deg,
      color-mix(in srgb, var(--preview-base) 12%, var(--panel-strong)) 0%,
      color-mix(in srgb, var(--preview-highlight) 14%, var(--panel-strong)) 100%
    );
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.04);
}

.preview-badge {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 0 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  color: color-mix(in srgb, var(--preview-highlight) 72%, white 28%);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.preview-main-line {
  margin-top: 14px;
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 10px 14px;
}

.preview-base-line,
.preview-highlight-line {
  font-size: clamp(21px, 2vw, 30px);
  line-height: 1.08;
  font-weight: 900;
  letter-spacing: 0.02em;
}

.preview-base-line {
  color: var(--preview-base);
}

.preview-highlight-line {
  color: var(--preview-highlight);
}

.preview-sub-line {
  margin-top: 10px;
  font-size: 12px;
  line-height: 1.6;
  color: color-mix(in srgb, var(--preview-base) 72%, var(--text-muted));
}

.lyric-color-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.lyric-color-board {
  padding: 14px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: color-mix(in srgb, var(--panel-strong) 82%, white 2%);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.color-board-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.color-board-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.color-board-copy strong {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-primary);
}

.color-board-copy small {
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-muted);
}

.color-board-body {
  display: grid;
  grid-template-columns: 88px minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.color-picker-shell {
  position: relative;
  width: 88px;
  height: 88px;
  border-radius: 18px;
  overflow: hidden;
  border: 1px solid var(--border);
  background: var(--color-chip-value, #ffffff);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.18),
    0 10px 24px rgba(0, 0, 0, 0.08);
  cursor: pointer;
}

.color-picker-shell input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}

.color-picker-surface {
  position: absolute;
  inset: 0;
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.36), rgba(255, 255, 255, 0)),
    linear-gradient(315deg, rgba(0, 0, 0, 0.18), rgba(0, 0, 0, 0));
}

.color-board-fields {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.color-hex-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.color-hex-field span {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}

.color-hex-field input {
  width: 100%;
  height: 38px;
  padding: 0 12px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.04);
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.color-hex-field input:focus {
  outline: none;
  border-color: color-mix(in srgb, var(--accent) 34%, var(--border));
  box-shadow: 0 0 0 3px rgba(22, 214, 160, 0.08);
}

.color-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-start;
  gap: 8px;
}

.color-chip {
  height: 34px;
  padding: 0 10px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  border: 1px solid transparent;
  transition: var(--transition);
}

.color-chip:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.color-chip.active {
  color: var(--text-primary);
  border-color: var(--border);
  background: color-mix(in srgb, var(--panel-strong) 82%, var(--color-chip-value) 18%);
}

.color-chip-swatch {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  border-radius: 999px;
  background: var(--color-chip-value, #ffffff);
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.14),
    inset 0 0 0 1px rgba(255, 255, 255, 0.3);
}

.storage-group {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.storage-row,
.credit-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.storage-row {
  padding: 2px 0;
}

.storage-copy,
.credit-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.storage-copy strong,
.credit-copy strong {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
}

.storage-path,
.credit-copy span {
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
  word-break: break-all;
  user-select: text;
  -webkit-user-select: text;
}

.storage-copy small,
.credit-copy small {
  font-size: 11px;
  color: var(--text-muted);
}

.storage-actions {
  flex-shrink: 0;
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.storage-error {
  margin-bottom: 10px;
  font-size: 12px;
  color: var(--text-danger);
}

.credit-card {
  padding: 12px 14px;
  border-radius: 16px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
}

@media (max-width: 900px) {
  .setting-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .lyric-color-grid {
    grid-template-columns: 1fr;
  }

  .source-grid {
    grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
  }

  .chip-row,
  .color-row,
  .storage-actions {
    justify-content: flex-start;
  }

  .storage-row,
  .credit-card {
    flex-direction: column;
    align-items: flex-start;
  }
}

@media (max-width: 640px) {
  .color-board-body {
    grid-template-columns: 1fr;
  }

  .color-picker-shell {
    width: 100%;
    height: 76px;
  }

  .preview-main-line {
    gap: 8px 10px;
  }
}
</style>

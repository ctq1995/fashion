<template>
  <div class="settings-panel app-scroll">
    <section class="settings-card">
      <div class="section-head">
        <h3>搜索音源</h3>
        <span class="section-side">{{ ui.enabledToolbarSources.length }} / {{ SOURCES.length }}</span>
      </div>

      <p class="section-tip">点亮表示启用，未点亮表示不使用。</p>

      <div class="source-grid">
        <button
          v-for="source in SOURCES"
          :key="source.value"
          type="button"
          class="source-chip"
          :class="{
            active: ui.isSourceEnabled(source.value),
            current: ui.toolbarSource === source.value,
          }"
          :disabled="ui.isSourceEnabled(source.value) && ui.enabledToolbarSources.length === 1"
          :title="ui.toolbarSource === source.value ? `${source.label}（当前）` : source.label"
          @click="ui.toggleSourceEnabled(source.value)"
        >
          <span>{{ source.label }}</span>
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
import { SOURCES } from '@/api/music';
import { type Bitrate, usePlayerStore } from '@/stores/player';
import { useStorageStore } from '@/stores/storage';
import { useUiStore, type AppTheme } from '@/stores/ui';

const CREDITS_URL = 'https://music.gdstudio.xyz';

const player = usePlayerStore();
const ui = useUiStore();
const storage = useStorageStore();

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

async function openCreditsUrl() {
  try {
    await openUrl(CREDITS_URL);
  } catch {
    if (typeof window !== 'undefined') {
      window.open(CREDITS_URL, '_blank', 'noopener,noreferrer');
    }
  }
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
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 42px;
  padding: 0 12px;
  border-radius: 14px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  border: 1px solid transparent;
  font-size: 13px;
  font-weight: 700;
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

.source-chip.current {
  justify-content: space-between;
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

  .source-grid {
    grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
  }

  .chip-row,
  .storage-actions {
    justify-content: flex-start;
  }

  .storage-row,
  .credit-card {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>

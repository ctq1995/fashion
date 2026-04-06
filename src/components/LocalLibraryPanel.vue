<template>
  <div class="local-library-panel">
    <section class="local-topbar">
      <div class="topbar-copy">
        <div class="topbar-head">
          <h2>本地音乐</h2>
          <span class="count-chip">{{ localLibrary.tracks.length }}</span>
        </div>
        <p class="topbar-note">
          {{ localLibrary.tracks.length ? '已导入本地歌曲，可直接播放、收藏或加入队列。' : '先添加文件夹并扫描，本地歌曲会集中显示在这里。' }}
        </p>
      </div>

      <div class="topbar-actions">
        <button type="button" class="app-btn app-btn--ghost compact-btn" @click="addFolder">
          添加文件夹
        </button>
        <button
          type="button"
          class="app-btn app-btn--primary compact-btn"
          :disabled="!localLibrary.folders.length || localLibrary.scanning"
          @click="scanLibrary"
        >
          {{ localLibrary.scanning ? '扫描中...' : '扫描曲库' }}
        </button>
      </div>
    </section>

    <div v-if="localLibrary.scanError" class="err-bar">{{ localLibrary.scanError }}</div>
    <div v-else-if="scanSummary" class="scan-summary">
      最近扫描：导入 {{ scanSummary.importedFiles }} 首，扫描 {{ scanSummary.scannedFiles }} 个文件
    </div>

    <div v-if="!localLibrary.tracks.length" class="empty-state">
      <p>本地曲库为空</p>
      <span>支持 mp3、flac、m4a、wav、ogg。先添加一个本地音乐文件夹。</span>
    </div>

    <div v-else class="local-list app-scroll">
      <SharedSongRow
        v-for="(item, idx) in localLibrary.tracks"
        :key="item.track.source + '-' + item.track.id"
        class="local-row"
        :title="item.track.name"
        :subtitle="`${item.track.artist} · ${item.track.album}`"
        :duration-text="formatDuration(item.record.durationSec)"
        :playing-label="isCurrentTrack(item.track) ? '播放中' : undefined"
        :active="isCurrentTrack(item.track)"
        @dblclick="player.addToQueue(item.track, true)"
      >
        <template #index>
          <span class="row-index">{{ idx + 1 }}</span>
        </template>

        <template #cover>
          <img v-if="item.track.coverUrl" :src="item.track.coverUrl" :alt="item.track.name" @error="media.markCoverLoadFailed(item.track)" />
          <div v-else class="row-cover-ph">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </template>

        <template #actions>
          <div class="action-row">
            <button type="button" class="app-icon-btn" :class="{ active: library.isFavorite(item.track) }" title="收藏" @click.stop="library.toggleFavorite(item.track)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z" :fill="library.isFavorite(item.track) ? 'currentColor' : 'none'" />
              </svg>
            </button>
            <button type="button" class="app-icon-btn" title="加入队列" @click.stop="player.addToQueue(item.track)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
              </svg>
            </button>
            <button type="button" class="app-icon-btn play-btn" title="播放" @click.stop="player.addToQueue(item.track, true)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5,3 19,12 5,21" />
              </svg>
            </button>
            <button type="button" class="app-icon-btn app-icon-btn--danger" title="从本地曲库移除" @click.stop="localLibrary.removeTrack(item.record.id)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18" />
                <path d="M8 6V4h8v2" />
                <path d="m19 6-1 14H6L5 6" />
                <path d="M10 11v6" />
                <path d="M14 11v6" />
              </svg>
            </button>
          </div>
        </template>
      </SharedSongRow>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SharedSongRow from '@/components/SharedSongRow.vue';
import { useLibraryStore } from '@/stores/library';
import { useLocalLibraryStore } from '@/stores/localLibrary';
import { useMediaStore } from '@/stores/media';
import { usePlayerStore, type Track } from '@/stores/player';
import { formatDuration } from '@/utils/formatters';

const player = usePlayerStore();
const library = useLibraryStore();
const localLibrary = useLocalLibraryStore();
const media = useMediaStore();

const scanSummary = computed(() => localLibrary.lastScanResult);

async function addFolder() {
  await localLibrary.addFolder();
}

async function scanLibrary() {
  await localLibrary.scanLibrary();
}

function isCurrentTrack(track: Track) {
  return player.currentTrack?.id === track.id && player.currentTrack?.source === track.source;
}
</script>

<style scoped>
.local-library-panel {
  height: 100%;
  min-height: 0;
  padding: 10px 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.local-topbar {
  padding: 12px 14px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: linear-gradient(135deg, var(--panel-strong), rgba(255, 255, 255, 0.03));
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.topbar-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.topbar-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.topbar-head h2 {
  font-size: 20px;
  font-weight: 900;
  color: var(--text-primary);
}

.count-chip {
  min-width: 24px;
  height: 24px;
  padding: 0 8px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-hover);
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
}

.topbar-note,
.scan-summary {
  font-size: 12px;
  color: var(--text-muted);
}

.topbar-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.local-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-right: 4px;
}

.local-row {
  padding: 8px 10px;
}

.row-index {
  width: 24px;
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}

.row-cover-ph {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.action-row {
  display: flex;
  gap: 6px;
}

.action-row :deep(.app-icon-btn) {
  width: 42px;
  height: 42px;
  border-radius: 14px;
}

.action-row :deep(svg) {
  width: 18px;
  height: 18px;
}

.action-row .app-icon-btn.active {
  color: var(--text-heart);
}

.play-btn {
  background: var(--accent-dim);
  color: var(--accent);
}

.play-btn:hover {
  background: var(--accent);
  color: var(--text-on-accent);
}

:deep(.local-row .row-cover img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

:deep(.local-row .row-actions) {
  gap: 6px;
}

.empty-state {
  flex: 1;
  min-height: 0;
  border-radius: 20px;
  background: var(--bg-hover);
  border: 1px dashed var(--border);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  text-align: center;
  color: var(--text-muted);
  padding: 24px;
}

.empty-state p {
  color: var(--text-primary);
  font-size: 16px;
  font-weight: 800;
}

@media (max-width: 1100px) {
  .local-topbar {
    align-items: flex-start;
    flex-direction: column;
  }
}

@media (max-width: 760px) {
  .topbar-actions {
    width: 100%;
    flex-wrap: wrap;
  }

  .action-row {
    flex-wrap: wrap;
    justify-content: flex-start;
  }
}
</style>


<template>
  <div class="history-panel">
    <section class="history-topbar">
      <div class="topbar-copy">
        <div class="topbar-head">
          <span class="section-kicker">History</span>
          <h2>最近播放</h2>
          <span class="count-chip">{{ player.history.length }}</span>
        </div>
        <div v-if="heroTrack" class="topbar-track">
          <span class="track-title">{{ heroTrack.name }}</span>
          <span class="track-meta">{{ heroTrack.artist }} · {{ heroTrack.album }}</span>
        </div>
        <div v-else class="topbar-track empty">
          <span class="track-title">暂无播放记录</span>
          <span class="track-meta">播放后会自动记录，重复歌曲只保留最新一条。</span>
        </div>
      </div>

      <div class="topbar-actions">
        <button
          type="button"
          class="app-icon-btn topbar-btn topbar-btn--primary"
          :disabled="!player.history.length"
          title="播放全部"
          @click="playAllHistory"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <polygon points="5,3 19,12 5,21" />
          </svg>
        </button>
        <button
          type="button"
          class="app-icon-btn topbar-btn"
          :disabled="!player.history.length"
          title="清空记录"
          @click="player.clearHistory()"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6" />
            <path d="M8 6V4h8v2" />
            <path d="M19 6l-1 14H6L5 6" />
            <line x1="10" y1="11" x2="10" y2="17" />
            <line x1="14" y1="11" x2="14" y2="17" />
          </svg>
        </button>
      </div>
    </section>

    <div v-if="!player.history.length" class="empty-state">
      <p>暂无播放记录</p>
      <span>播放歌曲后，这里会显示最近的歌曲信息。</span>
    </div>

    <div v-else class="history-board app-scroll">
      <SharedSongRow
        v-for="(item, idx) in player.history"
        :key="item.historyId"
        class="history-row"
        :title="item.name"
        :subtitle="`${item.artist} · ${item.album}`"
        :duration-text="formatDuration(item.durationSnapshot || item.durationSec || null)"
        :active="player.currentTrack?.id === item.id && player.currentTrack?.source === item.source"
        @dblclick="player.playHistory(item)"
      >
        <template #index>
          <span class="row-index">{{ idx + 1 }}</span>
        </template>

        <template #cover>
          <img v-if="item.coverUrl" :src="item.coverUrl" :alt="item.name" @error="media.markCoverLoadFailed(item)" />
          <div v-else class="row-cover-ph">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </template>

        <template #actions>
          <button
            type="button"
            class="app-icon-btn"
            :class="{ active: library.isFavorite(item) }"
            title="收藏"
            @click.stop="library.toggleFavorite(item)"
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path
                d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z"
                :fill="library.isFavorite(item) ? 'currentColor' : 'none'"
              />
            </svg>
          </button>
          <button type="button" class="app-icon-btn" title="播放" @click.stop="player.playHistory(item)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5,3 19,12 5,21" />
            </svg>
          </button>
          <DownloadButton :track="item" />
          <button type="button" class="app-icon-btn app-icon-btn--danger" title="删除记录" @click.stop="player.removeHistory(item.historyId)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </template>
      </SharedSongRow>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import DownloadButton from '@/components/DownloadButton.vue';
import SharedSongRow from '@/components/SharedSongRow.vue';
import { usePlayerStore } from '@/stores/player';
import { useLibraryStore } from '@/stores/library';
import { useMediaStore } from '@/stores/media';
import { formatDuration } from '@/utils/formatters';

const player = usePlayerStore();
const library = useLibraryStore();
const media = useMediaStore();

const heroTrack = computed(() => player.history[0] ?? null);

function playAllHistory() {
  if (!player.history.length) return;
  player.history.forEach((item, index) => {
    player.addToQueue(item, index === 0);
  });
}

</script>

<style scoped>
.history-panel {
  height: 100%;
  min-height: 0;
  padding: 10px 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-topbar {
  padding: 12px 14px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: linear-gradient(135deg, var(--panel-strong), rgba(255, 255, 255, 0.03));
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.topbar-copy { min-width: 0; }
.topbar-head { display: flex; align-items: center; gap: 8px; }
.section-kicker { font-size: 11px; font-weight: 800; letter-spacing: 0.12em; text-transform: uppercase; color: var(--text-muted); }
.topbar-head h2 { font-size: 20px; font-weight: 900; color: var(--text-primary); }
.count-chip { min-width: 24px; height: 24px; padding: 0 8px; border-radius: 999px; display: inline-flex; align-items: center; justify-content: center; background: var(--bg-hover); color: var(--text-muted); font-size: 11px; font-weight: 700; }
.topbar-track { margin-top: 4px; display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.track-title, .track-meta { display: block; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.track-title { font-size: 13px; font-weight: 700; color: var(--text-primary); }
.track-meta { font-size: 12px; color: var(--text-muted); }
.topbar-track.empty .track-title { font-weight: 800; }
.topbar-actions { display: flex; gap: 8px; flex-shrink: 0; }
.topbar-btn { width: 38px; height: 38px; border-radius: 12px; }
.topbar-btn--primary { color: var(--text-on-accent); background: linear-gradient(135deg, var(--accent-light), var(--accent)); box-shadow: 0 10px 20px var(--accent-glow); }
.topbar-btn--primary:hover:not(:disabled) { transform: translateY(-1px); filter: brightness(1.03); }
.history-board { flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain; display: flex; flex-direction: column; gap: 6px; padding-right: 4px; }
.row-index { width: 24px; text-align: center; font-size: 12px; color: var(--text-muted); }
.row-cover-ph { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; color: var(--text-muted); }
:deep(.row-cover img) { width: 100%; height: 100%; object-fit: cover; }
:deep(.row-actions .app-icon-btn) { width: 42px; height: 42px; border-radius: 14px; }
:deep(.row-actions .app-icon-btn.active) { color: var(--text-heart); }
:deep(.row-actions svg), :deep(.row-actions .spinner) { width: 18px; height: 18px; }
.empty-state { flex: 1; min-height: 0; border-radius: 20px; border: 1px dashed var(--border); background: var(--bg-hover); display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; text-align: center; color: var(--text-muted); padding: 24px; }
.empty-state p { font-size: 18px; font-weight: 800; color: var(--text-primary); }
@media (max-width: 980px) { .history-topbar { flex-direction: column; align-items: flex-start; } }
</style>

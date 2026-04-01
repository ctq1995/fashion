<template>
  <div class="history-panel">
    <section class="history-topbar">
      <div class="topbar-copy">
        <div class="topbar-head">
          <span class="section-kicker">History</span>
          <h2>最近播放</h2>
          <span class="count-chip">{{ player.history.length }}</span>
        </div>
        <div class="topbar-track" v-if="heroTrack">
          <span class="track-title">{{ heroTrack.name }}</span>
          <span class="track-meta">{{ heroTrack.artist }} · {{ heroTrack.album }}</span>
        </div>
        <div class="topbar-track empty" v-else>
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
      <div v-for="(item, idx) in player.history" :key="item.historyId" class="history-row" @dblclick="player.playHistory(item)">
        <span class="row-index">{{ idx + 1 }}</span>

        <div class="row-main">
          <div class="row-cover">
            <img v-if="item.coverUrl" :src="item.coverUrl" />
            <div v-else class="row-cover-ph">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <circle cx="12" cy="12" r="10" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </div>
          </div>

          <div class="row-meta">
            <span class="row-title">{{ item.name }}</span>
            <span class="row-sub">{{ item.artist }} · {{ item.album }}</span>
          </div>
        </div>

        <div class="row-side">
          <span class="row-time">{{ fmtPlayedAt(item.playedAt) }}</span>
          <span class="row-progress">{{ item.completed ? '已播完' : `停在 ${fmtTime(item.lastPosition)}` }}</span>
        </div>

        <div class="row-actions">
          <button
            type="button"
            class="app-icon-btn"
            :class="{ active: library.isFavorite(item) }"
            title="收藏"
            @click="library.toggleFavorite(item)"
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path
                d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z"
                :fill="library.isFavorite(item) ? 'currentColor' : 'none'"
              />
            </svg>
          </button>
          <button type="button" class="app-icon-btn" title="播放" @click="player.playHistory(item)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5,3 19,12 5,21" />
            </svg>
          </button>
          <DownloadButton :track="item" />
          <button type="button" class="app-icon-btn app-icon-btn--danger" title="删除记录" @click="player.removeHistory(item.historyId)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import DownloadButton from '@/components/DownloadButton.vue';
import { usePlayerStore } from '@/stores/player';
import { useLibraryStore } from '@/stores/library';

const player = usePlayerStore();
const library = useLibraryStore();

const heroTrack = computed(() => player.history[0] ?? null);

function playAllHistory() {
  if (!player.history.length) return;
  player.history.forEach((item, index) => {
    player.addToQueue(item, index === 0);
  });
}

function fmtTime(value: number): string {
  if (!value || Number.isNaN(value)) return '--:--';
  const minutes = Math.floor(value / 60);
  const seconds = Math.floor(value % 60);
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function fmtPlayedAt(value: number): string {
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(value);
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

.topbar-copy {
  min-width: 0;
}

.topbar-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-kicker {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-muted);
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

.topbar-track {
  margin-top: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.track-title,
.track-meta {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
}

.track-meta {
  font-size: 12px;
  color: var(--text-muted);
}

.topbar-track.empty .track-title {
  font-weight: 800;
}

.topbar-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.topbar-btn {
  width: 38px;
  height: 38px;
  border-radius: 12px;
}

.topbar-btn--primary {
  color: var(--text-on-accent);
  background: linear-gradient(135deg, var(--accent-light), var(--accent));
  box-shadow: 0 10px 20px var(--accent-glow);
}

.topbar-btn--primary:hover:not(:disabled) {
  transform: translateY(-1px);
  filter: brightness(1.03);
}

.history-board {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-right: 4px;
}

.history-row {
  min-height: 58px;
  padding: 8px 10px;
  border-radius: 16px;
  display: grid;
  grid-template-columns: 24px minmax(0, 1.5fr) minmax(98px, 0.55fr) auto;
  gap: 10px;
  align-items: center;
  border: 1px solid transparent;
  transition: var(--transition);
}

.history-row:hover {
  background: var(--bg-hover);
  border-color: var(--border);
}

.row-index {
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}

.row-main {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.row-cover {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  overflow: hidden;
  background: var(--bg-active);
  flex-shrink: 0;
}

.row-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.row-cover-ph {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.row-meta {
  min-width: 0;
}

.row-title,
.row-sub {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}

.row-sub {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-muted);
}

.row-side {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 11px;
  color: var(--text-secondary);
}

.row-progress {
  color: var(--text-muted);
}

.row-actions {
  display: flex;
  gap: 6px;
}

.row-actions :deep(.app-icon-btn) {
  width: 42px;
  height: 42px;
  border-radius: 14px;
}

.row-actions :deep(svg) {
  width: 18px;
  height: 18px;
}

.row-actions :deep(.spinner) {
  width: 18px;
  height: 18px;
}

.row-actions .app-icon-btn.active {
  color: var(--text-heart);
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-muted);
}

.empty-state p {
  font-size: 18px;
  font-weight: 800;
  color: var(--text-primary);
}

@media (max-width: 980px) {
  .history-topbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .history-row {
    grid-template-columns: 24px minmax(0, 1fr);
  }

  .row-side,
  .row-actions {
    grid-column: 2;
  }
}
</style>

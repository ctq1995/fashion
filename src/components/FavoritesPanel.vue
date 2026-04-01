<template>
  <div class="favorites-panel">
    <section class="favorites-topbar">
      <div class="topbar-copy">
        <div class="topbar-head">
          <span class="section-kicker">Library</span>
          <h2>我的喜欢</h2>
          <span class="count-chip">{{ library.favorites.length }}</span>
        </div>
        <div class="topbar-track" v-if="latestFavorite">
          <span class="track-title">{{ latestFavorite.name }}</span>
          <span class="track-meta">{{ latestFavorite.artist }} · {{ latestFavorite.album }}</span>
        </div>
        <div class="topbar-track empty" v-else>
          <span class="track-title">还没有收藏歌曲</span>
          <span class="track-meta">在搜索结果或最近播放里点亮心形按钮即可收藏。</span>
        </div>
      </div>

      <div class="topbar-actions">
        <button
          type="button"
          class="app-icon-btn topbar-btn topbar-btn--primary"
          :disabled="!library.favorites.length"
          title="播放全部"
          @click="playAll"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <polygon points="5,3 19,12 5,21" />
          </svg>
        </button>
      </div>
    </section>

    <div v-if="!library.favorites.length" class="empty-state">
      <p>还没有收藏歌曲</p>
      <span>收藏后的歌曲会保存在本地，并同步显示在这里。</span>
    </div>

    <div v-else class="favorites-list app-scroll">
      <div
        v-for="(track, idx) in library.favorites"
        :key="track.source + '-' + track.id"
        class="favorite-row"
        @dblclick="player.addToQueue(track, true)"
      >
        <span class="row-index">{{ idx + 1 }}</span>

        <div class="row-cover">
          <img v-if="track.coverUrl" :src="track.coverUrl" />
          <div v-else class="row-cover-ph">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </div>

        <div class="row-meta">
          <div class="row-head">
            <span class="row-title">{{ track.name }}</span>
            <span v-if="isCurrentTrack(track)" class="playing-tag">播放中</span>
          </div>
          <span class="row-sub">{{ track.artist }} · {{ track.album }}</span>
        </div>

        <div class="row-actions">
          <button type="button" class="app-icon-btn row-action-btn" title="播放" @click="player.addToQueue(track, true)">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5,3 19,12 5,21" />
            </svg>
          </button>
          <DownloadButton :track="track" />
          <button
            type="button"
            class="app-icon-btn app-icon-btn--danger row-action-btn"
            title="移除收藏"
            @click="library.removeFavorite(track)"
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
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
import { usePlayerStore, type Track } from '@/stores/player';
import { useLibraryStore } from '@/stores/library';

const player = usePlayerStore();
const library = useLibraryStore();

const latestFavorite = computed(() => library.favorites[0] ?? null);

function playAll() {
  library.favorites.forEach((track, index) => player.addToQueue(track, index === 0));
}

function isCurrentTrack(track: Track) {
  return player.currentTrack?.id === track.id && player.currentTrack?.source === track.source;
}
</script>

<style scoped>
.favorites-panel {
  height: 100%;
  min-height: 0;
  padding: 10px 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.favorites-topbar {
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

.favorites-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-right: 4px;
}

.favorite-row {
  min-height: 58px;
  padding: 8px 10px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.02);
  display: grid;
  grid-template-columns: 24px 42px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  transition: var(--transition);
}

.favorite-row:hover {
  background: var(--bg-hover);
  border-color: var(--border);
}

.row-index {
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}

.row-cover {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  overflow: hidden;
  background: var(--bg-active);
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

.row-head {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.row-title {
  min-width: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-sub {
  margin-top: 2px;
  display: block;
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playing-tag {
  padding: 3px 8px;
  border-radius: 999px;
  background: var(--accent-dim);
  color: var(--accent);
  font-size: 10px;
  font-weight: 700;
  white-space: nowrap;
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
  .favorites-topbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .favorite-row {
    grid-template-columns: 24px 42px minmax(0, 1fr);
  }

  .row-actions {
    grid-column: 2 / span 2;
    justify-content: flex-start;
  }
}
</style>

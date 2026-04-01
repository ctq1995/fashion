<template>
  <aside class="playlist-drawer">
    <div class="drawer-head">
      <div>
        <h3>播放列表</h3>
        <p>{{ player.queue.length }} 首歌曲</p>
      </div>
      <div class="drawer-actions">
        <button
          v-if="player.queue.length"
          type="button"
          class="head-btn"
          @click="player.clearQueue()"
        >
          清空
        </button>
        <button type="button" class="close-btn" title="关闭" @click="$emit('close')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>

    <div v-if="!player.queue.length" class="drawer-empty">
      <p>当前队列还是空的</p>
      <span>从推荐页或搜索结果里播放歌曲后，会自动加入这里。</span>
    </div>

    <div v-else class="drawer-list">
      <button
        v-for="(track, idx) in player.queue"
        :key="track.source + '-' + track.id"
        type="button"
        class="drawer-row"
        :class="{ active: idx === player.currentIndex }"
        @dblclick="player.playTrack(idx)"
      >
        <div class="drawer-cover">
          <img v-if="track.coverUrl" :src="track.coverUrl" :alt="track.name" />
          <div v-else class="drawer-cover-ph">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </div>

        <div class="drawer-meta">
          <div class="drawer-title-line">
            <span class="drawer-name">{{ track.name }}</span>
            <span v-if="idx === player.currentIndex" class="playing-badge">正在播放</span>
          </div>
          <span class="drawer-sub">{{ track.artist }}</span>
        </div>

        <div class="drawer-actions-row">
          <button type="button" class="icon-btn play-btn" title="播放" @click.stop="player.playTrack(idx)">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5,3 19,12 5,21" />
            </svg>
          </button>
          <button type="button" class="icon-btn remove-btn" title="移除" @click.stop="player.removeFromQueue(idx)">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { usePlayerStore } from '@/stores/player';

defineEmits<{ close: [] }>();

const player = usePlayerStore();
</script>

<style scoped>
.playlist-drawer {
  width: 408px;
  height: calc(100% - 18px);
  margin: 9px 9px 9px 0;
  border-radius: 26px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 22px 18px 18px;
  background: linear-gradient(180deg, var(--panel-strong), var(--panel-shell));
  border: 1px solid var(--border);
  backdrop-filter: blur(24px);
  box-shadow: var(--window-shadow);
}

.drawer-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.drawer-head h3 {
  font-size: 20px;
  color: var(--text-primary);
  font-weight: 800;
}

.drawer-head p {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-muted);
}

.drawer-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.head-btn,
.close-btn {
  height: 36px;
  min-width: 36px;
  padding: 0 14px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border: 1px solid var(--border);
  transition: var(--transition);
}

.head-btn {
  font-size: 12px;
  font-weight: 700;
}

.head-btn:hover,
.close-btn:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.drawer-empty {
  flex: 1;
  border-radius: 22px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  text-align: center;
  background: var(--bg-hover);
  color: var(--text-muted);
  padding: 24px;
}

.drawer-empty p {
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 700;
}

.drawer-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.drawer-row {
  padding: 12px;
  border-radius: 18px;
  border: 1px solid transparent;
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0);
  transition: var(--transition);
  text-align: left;
}

.drawer-row.active,
.drawer-row:hover {
  background: var(--bg-hover);
  border-color: var(--border);
}

.drawer-row.active {
  box-shadow: inset 0 0 0 1px var(--accent-glow);
}

.drawer-cover {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  overflow: hidden;
  background: var(--bg-hover);
  border: 1px solid var(--border);
}

.drawer-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.drawer-cover-ph {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.drawer-meta {
  min-width: 0;
}

.drawer-title-line {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.drawer-name,
.drawer-sub {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.drawer-name {
  min-width: 0;
  font-size: 14px;
  color: var(--text-primary);
  font-weight: 700;
}

.drawer-sub {
  margin-top: 5px;
  font-size: 12px;
  color: var(--text-muted);
}

.playing-badge {
  flex-shrink: 0;
  padding: 3px 8px;
  border-radius: 999px;
  background: var(--accent-dim);
  color: var(--accent);
  font-size: 10px;
  font-weight: 700;
}

.drawer-actions-row {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.icon-btn {
  width: 36px;
  height: 36px;
  border-radius: 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
  border: 1px solid transparent;
}

.play-btn {
  color: var(--text-on-accent);
  background: linear-gradient(135deg, var(--accent), var(--accent-light));
}

.play-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 22px var(--accent-glow);
}

.remove-btn {
  color: var(--text-secondary);
  background: var(--bg-hover);
  border-color: var(--border);
}

.remove-btn:hover {
  background: var(--danger-soft);
  color: var(--text-danger);
}

@media (max-width: 980px) {
  .playlist-drawer {
    width: min(408px, calc(100vw - 18px));
  }
}

@media (max-width: 640px) {
  .drawer-row {
    grid-template-columns: 44px minmax(0, 1fr);
  }

  .drawer-actions-row {
    grid-column: 1 / -1;
    justify-content: flex-start;
    padding-left: 56px;
  }
}
</style>

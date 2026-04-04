<template>
  <section class="mobile-dock" :class="{ immersive, hidden }">
    <div
      ref="progressBar"
      class="dock-progress-hit"
      :class="{ dragging: isDraggingSeek }"
      @click="onSeekClick"
      @pointerdown.prevent="startSeek"
    >
      <div class="dock-progress-track">
        <div class="dock-progress-fill" :style="{ width: `${progressPct}%` }" />
        <div class="dock-progress-thumb" :style="{ left: `${progressPct}%` }" />
      </div>
    </div>

    <div class="dock-main">
      <button
        type="button"
        class="cover-button"
        :class="{ active: immersive }"
        :disabled="!player.currentTrack"
        title="打开歌词"
        @click="emit('open-lyric')"
      >
        <div class="cover-art">
          <img v-if="player.currentTrack?.coverUrl" :src="player.currentTrack.coverUrl" alt="" />
          <div v-else class="cover-fallback">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </div>
      </button>

      <div class="dock-copy">
        <span class="dock-kicker">{{ statusLabel }}</span>
        <strong>{{ player.currentTrack?.name ?? '还没有开始播放' }}</strong>
        <span>{{ subtitle }}</span>
      </div>

      <button type="button" class="queue-button" title="播放列表" @click="emit('open-playlist')">
        <span v-html="playlistIcon" />
        <span v-if="player.queue.length" class="queue-badge">{{ Math.min(player.queue.length, 99) }}</span>
      </button>
    </div>

    <div class="dock-controls">
      <button
        type="button"
        class="control ghost"
        :class="{ active: currentTrackFavorite }"
        :disabled="!player.currentTrack"
        title="收藏"
        @click="toggleFavorite"
      >
        <span v-html="favoriteIcon" />
      </button>

      <button
        type="button"
        class="control ghost"
        :disabled="!player.queue.length"
        title="上一首"
        @click="player.playPrev()"
      >
        <span v-html="prevIcon" />
      </button>

      <button
        type="button"
        class="control play"
        :disabled="!player.currentTrack"
        title="播放或暂停"
        @click="player.togglePlay()"
      >
        <span v-if="!player.isPlaying" v-html="playIcon" />
        <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="4" width="4" height="16" rx="1" />
          <rect x="14" y="4" width="4" height="16" rx="1" />
        </svg>
      </button>

      <button
        type="button"
        class="control ghost"
        :disabled="!player.queue.length"
        title="下一首"
        @click="player.playNext()"
      >
        <span v-html="nextIcon" />
      </button>

      <button
        type="button"
        class="control ghost"
        title="播放模式"
        @click="player.togglePlayMode()"
      >
        <span v-html="modeIcon" />
      </button>
    </div>

    <div class="dock-foot">
      <span>{{ fmtTime(player.currentTime) }}</span>
      <span>{{ footerLabel }}</span>
      <span>{{ fmtTime(player.duration) }}</span>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import { useLibraryStore } from '@/stores/library';
import { usePlayerStore } from '@/stores/player';
import { fillIcon, strokeIcon } from '@/utils/iconography';

const props = withDefaults(defineProps<{
  immersive?: boolean;
  hidden?: boolean;
}>(), {
  immersive: false,
  hidden: false,
});

const emit = defineEmits<{
  'open-lyric': [];
  'open-playlist': [];
}>();

const player = usePlayerStore();
const library = useLibraryStore();

const progressBar = ref<HTMLElement | null>(null);
const isDraggingSeek = ref(false);
let activeSeekPointerId: number | null = null;

const progressPct = computed(() => {
  if (!player.duration) return 0;
  return Math.min(100, (player.currentTime / player.duration) * 100);
});

const currentTrackFavorite = computed(() =>
  !!player.currentTrack && library.isFavorite({ id: player.currentTrack.id, source: player.currentTrack.source }),
);

const statusLabel = computed(() => {
  if (player.loading) return '缓冲中';
  if (player.currentTrack && player.isPlaying) return '正在播放';
  if (player.currentTrack) return '已暂停';
  return 'Fashion Music';
});

const subtitle = computed(() => {
  if (!player.currentTrack) return '先搜索一首歌，新的移动端播放器会在这里接管控制。';

  const album = player.currentTrack.album?.trim();
  return album
    ? `${player.currentTrack.artist} · ${album}`
    : player.currentTrack.artist;
});

const footerLabel = computed(() => {
  if (player.loading) return '正在加载';

  return ({
    sequence: '顺序播放',
    random: '随机播放',
    single: '单曲循环',
  } as const)[player.playMode];
});

const favoriteIcon = computed(() =>
  strokeIcon('<path d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z"/>', {
    size: 20,
    strokeWidth: 1.9,
    fill: currentTrackFavorite.value ? 'currentColor' : 'none',
  }),
);

const prevIcon = fillIcon('<path d="M6 6h2v12H6zm3.5 6 8.5 6V6z"/>', { size: 20 });
const playIcon = fillIcon('<polygon points="5,3 19,12 5,21"/>', { size: 22 });
const nextIcon = fillIcon('<path d="M6 18l8.5-6L6 6v12zm10-12h2v12h-2z"/>', { size: 20 });
const playlistIcon = strokeIcon('<path d="M3 7h12"/><path d="M3 12h12"/><path d="M3 17h8"/><path d="M19 8v10"/><path d="M14 13h10"/>', { size: 20, strokeWidth: 1.9 });

const modeIcon = computed(() => {
  if (player.playMode === 'random') {
    return strokeIcon('<polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/>', { size: 20, strokeWidth: 1.9 });
  }
  if (player.playMode === 'single') {
    return strokeIcon('<polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/><path d="M12 9v6"/><path d="M10.5 10.5h1.5"/><path d="M10.5 15h3"/>', { size: 20, strokeWidth: 1.9 });
  }
  return strokeIcon('<polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>', { size: 20, strokeWidth: 1.9 });
});

function toggleFavorite() {
  if (!player.currentTrack) return;
  library.toggleFavorite(player.currentTrack);
}

function seekByClientX(clientX: number, bar: HTMLElement) {
  if (!player.duration) return;
  const rect = bar.getBoundingClientRect();
  const pct = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width));
  player.seek(pct * player.duration);
}

function onSeekClick(event: MouseEvent) {
  if (!progressBar.value) return;
  seekByClientX(event.clientX, progressBar.value);
}

function handleSeekMove(event: PointerEvent) {
  if (activeSeekPointerId !== null && event.pointerId !== activeSeekPointerId) return;
  if (!progressBar.value) return;
  seekByClientX(event.clientX, progressBar.value);
}

function stopSeek(event?: PointerEvent) {
  if (event && activeSeekPointerId !== null && event.pointerId !== activeSeekPointerId) return;
  window.removeEventListener('pointermove', handleSeekMove);
  window.removeEventListener('pointerup', stopSeek);
  window.removeEventListener('pointercancel', stopSeek);
  activeSeekPointerId = null;
  isDraggingSeek.value = false;
}

function startSeek(event: PointerEvent) {
  if (typeof event.button === 'number' && event.button > 0) return;
  if (!progressBar.value) return;
  activeSeekPointerId = event.pointerId;
  isDraggingSeek.value = true;
  seekByClientX(event.clientX, progressBar.value);
  window.addEventListener('pointermove', handleSeekMove);
  window.addEventListener('pointerup', stopSeek);
  window.addEventListener('pointercancel', stopSeek);
}

function fmtTime(value: number): string {
  if (!value || Number.isNaN(value)) return '00:00';
  const minutes = Math.floor(value / 60);
  const seconds = Math.floor(value % 60);
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

onBeforeUnmount(() => {
  stopSeek();
});
</script>

<style scoped>
.mobile-dock {
  position: relative;
  overflow: hidden;
  padding: 14px 14px 10px;
  border-radius: 24px;
  border: 1px solid var(--border);
  background: linear-gradient(180deg, var(--panel-strong), rgba(255, 255, 255, 0.02));
  box-shadow: 0 18px 40px rgba(13, 25, 24, 0.14);
  backdrop-filter: blur(20px);
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.mobile-dock::before {
  content: '';
  position: absolute;
  inset: 0 0 auto;
  height: 48px;
  background: linear-gradient(90deg, var(--accent-dim) 0%, rgba(22, 214, 160, 0.18) 54%, rgba(6, 86, 69, 0.2) 100%);
  pointer-events: none;
  opacity: 0.8;
}

.mobile-dock > * {
  position: relative;
  z-index: 1;
}

.mobile-dock.immersive {
  background: linear-gradient(180deg, rgba(16, 28, 26, 0.9), rgba(18, 31, 29, 0.84));
  border-color: color-mix(in srgb, var(--border) 88%, rgba(255, 255, 255, 0.06));
}

.mobile-dock.hidden {
  opacity: 0;
  pointer-events: none;
  transform: translateY(14px);
}

.dock-progress-hit {
  height: 16px;
  display: flex;
  align-items: center;
  cursor: pointer;
  touch-action: none;
}

.dock-progress-track {
  position: relative;
  width: 100%;
  height: 3px;
  border-radius: 999px;
  background: var(--border);
}

.dock-progress-fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #7ef5d2 0%, var(--accent) 54%, #0d7c63 100%);
  box-shadow: 0 0 14px rgba(22, 214, 160, 0.26);
}

.dock-progress-thumb {
  position: absolute;
  top: 50%;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #17dca6;
  transform: translate(-50%, -50%) scale(0.86);
  box-shadow: 0 0 0 5px rgba(22, 214, 160, 0.16), 0 0 14px rgba(22, 214, 160, 0.3);
  transition: transform 0.12s ease;
}

.dock-progress-hit.dragging .dock-progress-thumb,
.dock-progress-hit:hover .dock-progress-thumb {
  transform: translate(-50%, -50%) scale(1);
}

.dock-main {
  margin-top: 8px;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
}

.cover-button {
  border-radius: 18px;
  transition: var(--transition);
}

.cover-button:disabled {
  opacity: 0.5;
}

.cover-button.active .cover-art {
  box-shadow:
    0 0 0 1px color-mix(in srgb, var(--accent) 28%, var(--border)),
    0 12px 24px rgba(13, 25, 24, 0.18);
}

.cover-art {
  width: 58px;
  height: 58px;
  overflow: hidden;
  border-radius: 18px;
  background: var(--panel-shell);
  border: 1px solid var(--border);
}

.cover-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.dock-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dock-kicker {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  width: fit-content;
  padding: 0 9px;
  border-radius: 999px;
  background: rgba(22, 214, 160, 0.12);
  color: var(--accent);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.06em;
}

.dock-copy strong,
.dock-copy span:last-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dock-copy strong {
  font-size: 15px;
  color: var(--text-primary);
}

.dock-copy span:last-child {
  font-size: 12px;
  color: var(--text-muted);
}

.queue-button {
  position: relative;
  width: 42px;
  height: 42px;
  border-radius: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border: 1px solid transparent;
  transition: var(--transition);
}

.queue-button:hover {
  color: var(--text-primary);
  background: var(--bg-active);
  border-color: var(--border);
}

.queue-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-dim);
  color: var(--accent);
  font-size: 10px;
  font-weight: 800;
}

.dock-controls {
  margin-top: 14px;
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 10px;
}

.control {
  min-width: 0;
  height: 46px;
  border-radius: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
}

.control.ghost {
  color: var(--text-secondary);
  background: var(--bg-hover);
  border: 1px solid transparent;
}

.control.ghost:hover:not(:disabled) {
  color: var(--text-primary);
  background: var(--bg-active);
  border-color: var(--border);
}

.control.ghost.active {
  color: var(--text-heart);
  background: rgba(255, 110, 153, 0.1);
  border-color: rgba(255, 110, 153, 0.16);
}

.control.play {
  color: var(--text-on-accent);
  background: linear-gradient(135deg, var(--accent-light), var(--accent));
  box-shadow: 0 14px 28px var(--accent-glow);
}

.control.play:hover:not(:disabled) {
  filter: brightness(1.04);
  transform: translateY(-1px);
}

.control:disabled {
  opacity: 0.4;
}

.dock-foot {
  margin-top: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
}

.dock-foot span:nth-child(2) {
  color: var(--text-secondary);
}
</style>

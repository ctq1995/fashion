<template>
  <div class="player-bar" :class="[scene, { hidden: props.hidden, 'lyric-fullscreen': props.lyricFullscreen }]">
    <div class="bar-progress-surface" :style="{ width: progressPct + '%' }" />

    <div
      ref="progressBar"
      class="bar-progress-hit"
      :class="{ dragging: isDraggingSeek }"
      @click="onSeekClick"
      @pointerdown.prevent="startSeek"
    >
      <div class="bar-progress-track">
        <div class="bar-progress-fill" :style="{ width: progressPct + '%' }" />
        <div class="bar-progress-thumb" :style="{ left: progressPct + '%' }" />
      </div>
    </div>

    <div class="bar-shell">
      <div class="now-playing">
        <button
          class="np-cover-toggle"
          :class="{ active: props.lyricActive }"
          :title="lyricToggleTitle"
          type="button"
          @click="emit('open-lyric')"
        >
          <div class="np-cover">
          <img v-if="player.currentTrack?.coverUrl" :src="player.currentTrack.coverUrl" />
          <div v-else class="np-cover-ph">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
          <div class="np-cover-overlay" :class="{ visible: props.lyricActive }">
            <svg
              v-if="!props.lyricActive"
              width="19"
              height="19"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="15 3 21 3 21 9" />
              <polyline points="9 21 3 21 3 15" />
              <line x1="21" y1="3" x2="14" y2="10" />
              <line x1="3" y1="21" x2="10" y2="14" />
            </svg>
            <svg
              v-else
              width="19"
              height="19"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="9 3 3 3 3 9" />
              <polyline points="15 21 21 21 21 15" />
              <line x1="3" y1="3" x2="10" y2="10" />
              <line x1="21" y1="21" x2="14" y2="14" />
            </svg>
          </div>
          </div>
        </button>

        <div class="np-meta">
          <span class="np-title">{{ player.currentTrack?.name ?? '未在播放' }}</span>
          <span class="np-subtitle">{{ player.currentTrack?.artist ?? '选择歌曲开始播放' }}</span>
        </div>
      </div>

      <div class="bar-center" :class="{ dragging: isDraggingSeek }">
        <div class="time-display">{{ fmtTime(player.currentTime) }} / {{ fmtTime(player.duration) }}</div>
        <div class="transport-row">
        <button
          class="mini-btn"
          :class="{ active: currentTrackFavorite }"
          :disabled="!player.currentTrack"
          @click="toggleCurrentFavorite"
          title="收藏"
        >
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z" :fill="currentTrackFavorite ? 'currentColor' : 'none'" />
          </svg>
        </button>
        <button class="mini-btn" @click="player.playPrev()" title="上一首">
          <span v-html="fillIcon('<path d=&quot;M6 6h2v12H6zm3.5 6 8.5 6V6z&quot;/>', { size: 22 })" />
        </button>
        <button class="play-btn" @click="player.togglePlay()" :disabled="!player.currentTrack" title="播放/暂停">
          <span
            v-if="!player.isPlaying"
            v-html="fillIcon('<polygon points=&quot;5,3 19,12 5,21&quot;/>', { size: 22 })"
          />
          <svg v-else width="26" height="26" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
          </svg>
        </button>
        <button class="mini-btn" @click="player.playNext()" title="下一首">
          <span v-html="fillIcon('<path d=&quot;M6 18l8.5-6L6 6v12zm2-8.14L11.03 12 8 14.14V9.86zM16 6h2v12h-2z&quot;/>', { size: 22 })" />
        </button>
        <button class="mini-btn" @click="player.togglePlayMode()" :title="modeLabel">
          <span v-html="modeIcon" />
          </button>
        </div>
      </div>

      <div class="bar-right">
        <div class="volume-hover" @mouseenter="openVolumePopover" @mouseleave="scheduleVolumePopoverClose">
          <button class="icon-btn volume-anchor" @click="toggleMute" :title="player.volume === 0 ? '取消静音' : '静音'" v-html="volumeIcon" />
          <div class="volume-pop" :class="{ open: volumePopoverOpen }" @mouseenter="openVolumePopover" @mouseleave="scheduleVolumePopoverClose" @click.stop @mousedown.stop>
            <span class="volume-pop-value">{{ Math.round(player.volume * 100) }}</span>
            <div class="volume-pop-track">
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="player.volume"
                class="volume-slider-vertical"
                @input="onVolumeInput"
              />
            </div>
          </div>
        </div>

        <button
          class="icon-btn lyric-btn"
          :class="{ active: props.desktopLyricOpen }"
          :title="desktopLyricTitle"
          @click="emit('toggle-desktop-lyric')"
        >
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9">
            <rect x="4" y="5" width="16" height="12" rx="3" />
            <path d="M8 10h8" />
            <path d="M8 13h5" />
            <path d="M9 19h6" />
          </svg>
        </button>
        <button class="icon-btn" @click="emit('open-playlist')" title="歌单">
          <svg width="23" height="23" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M3 7h12" />
            <path d="M3 12h12" />
            <path d="M3 17h8" />
            <path d="M19 8v10" />
            <path d="M14 13h10" />
          </svg>
        </button>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { usePlayerStore } from '@/stores/player';
import { useLibraryStore } from '@/stores/library';
import { fillIcon, strokeIcon } from '@/utils/iconography';

const props = withDefaults(defineProps<{
  scene: 'light' | 'dark';
  lyricActive?: boolean;
  lyricFullscreen?: boolean;
  desktopLyricOpen?: boolean;
  hidden?: boolean;
}>(), {
  lyricActive: false,
  lyricFullscreen: false,
  desktopLyricOpen: false,
  hidden: false,
});

const emit = defineEmits<{
  'open-lyric': [];
  'open-playlist': [];
  'toggle-lyric-fullscreen': [];
  'toggle-desktop-lyric': [];
}>();

const player = usePlayerStore();
const library = useLibraryStore();

const progressBar = ref<HTMLElement | null>(null);
const lastAudibleVolume = ref(player.volume > 0 ? player.volume : 0.8);
const volumePopoverOpen = ref(false);
const isDraggingSeek = ref(false);
let activeSeekPointerId: number | null = null;
let volumeCloseTimer: number | null = null;

const progressPct = computed(() => {
  if (!player.duration) return 0;
  return Math.min(100, (player.currentTime / player.duration) * 100);
});

const currentTrackFavorite = computed(() =>
  !!player.currentTrack && library.isFavorite({ id: player.currentTrack.id, source: player.currentTrack.source })
);

const lyricToggleTitle = computed(() =>
  props.lyricActive ? '收起歌词界面' : '展开歌词界面'
);

const desktopLyricTitle = computed(() =>
  props.desktopLyricOpen ? '关闭桌面歌词' : '打开桌面歌词'
);

const modeLabel = computed(() => ({
  sequence: '顺序播放',
  random: '随机播放',
  single: '单曲循环',
}[player.playMode]));

const modeIcon = computed(() => {
  if (player.playMode === 'random') {
    return strokeIcon('<polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/>', { size: 22, strokeWidth: 2 });
  }
  if (player.playMode === 'single') {
    return strokeIcon('<polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/><circle cx="12" cy="12" r="2.6"/><path d="M12 10.8v2.4"/><path d="M11.3 11.4H12"/>', { size: 22, strokeWidth: 2 });
  }
  return strokeIcon('<polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>', { size: 22, strokeWidth: 2 });
});

const volumeIcon = computed(() => {
  const v = player.volume;
  if (v === 0) {
    return strokeIcon('<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/>', { size: 22, strokeWidth: 2 });
  }
  if (v < 0.5) {
    return strokeIcon('<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>', { size: 22, strokeWidth: 2 });
  }
  return strokeIcon('<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>', { size: 22, strokeWidth: 2 });
});

function onVolumeInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (value > 0) lastAudibleVolume.value = value;
  player.setVolume(value);
}

function toggleMute() {
  if (player.volume === 0) {
    player.setVolume(lastAudibleVolume.value || 0.8);
    return;
  }
  lastAudibleVolume.value = player.volume;
  player.setVolume(0);
}

function toggleCurrentFavorite() {
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

function clearVolumeCloseTimer() {
  if (volumeCloseTimer !== null) {
    window.clearTimeout(volumeCloseTimer);
    volumeCloseTimer = null;
  }
}

function openVolumePopover() {
  clearVolumeCloseTimer();
  volumePopoverOpen.value = true;
}

function scheduleVolumePopoverClose() {
  clearVolumeCloseTimer();
  volumeCloseTimer = window.setTimeout(() => {
    volumePopoverOpen.value = false;
    volumeCloseTimer = null;
  }, 180);
}

function handleKeydown(event: KeyboardEvent) {
  const target = event.target as HTMLElement | null;
  const tagName = target?.tagName ?? '';
  if (tagName === 'INPUT' || tagName === 'TEXTAREA' || tagName === 'SELECT' || target?.isContentEditable) return;

  if (event.code === 'Space') {
    event.preventDefault();
    void player.togglePlay();
    return;
  }
  if (event.code === 'ArrowRight') {
    event.preventDefault();
    player.seek(Math.min(player.duration, player.currentTime + 5));
    return;
  }
  if (event.code === 'ArrowLeft') {
    event.preventDefault();
    player.seek(Math.max(0, player.currentTime - 5));
    return;
  }
  if (event.code === 'ArrowUp') {
    event.preventDefault();
    player.setVolume(Math.min(1, player.volume + 0.05));
    return;
  }
  if (event.code === 'ArrowDown') {
    event.preventDefault();
    player.setVolume(Math.max(0, player.volume - 0.05));
    return;
  }
  if (event.key.toLowerCase() === 'm') {
    event.preventDefault();
    toggleMute();
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown);
  stopSeek();
  clearVolumeCloseTimer();
});
</script>

<style scoped>
.player-bar {
  position: relative;
  height: var(--player-h);
  padding: 0 12px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  overflow: visible;
  z-index: 30;
  background: var(--panel-shell);
  border-top: 1px solid var(--border);
  backdrop-filter: blur(16px);
  transition: height 0.22s ease, padding 0.22s ease, opacity 0.22s ease, transform 0.22s ease;
}

.player-bar.hidden {
  height: 0;
  padding-top: 0;
  padding-bottom: 0;
  opacity: 0;
  pointer-events: none;
  overflow: hidden;
  transform: translateY(18px);
}

.player-bar.dark {
  background: transparent;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  backdrop-filter: none;
}

.player-bar.dark .mini-btn,
.player-bar.dark .icon-btn {
  color: rgba(255, 255, 255, 0.92);
  background: rgba(9, 18, 17, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.2);
}

.player-bar.dark .mini-btn:hover:not(:disabled),
.player-bar.dark .icon-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.14);
  color: #fff;
  border-color: rgba(255, 255, 255, 0.16);
}

.player-bar.dark .mini-btn.active {
  color: #ff8db0;
  background: rgba(255, 126, 165, 0.14);
  border-color: rgba(255, 126, 165, 0.24);
}

.player-bar.dark .play-btn {
  background: linear-gradient(135deg, #8bf4d3, #16d6a0);
  color: #06261d;
  box-shadow: 0 14px 34px rgba(22, 214, 160, 0.28);
  border: 1px solid transparent;
}

.player-bar.dark .play-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #a6fae1, #2adbb0);
  color: #031b15;
}

.player-bar.dark .np-cover {
  border-color: rgba(255, 255, 255, 0.14);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.16);
}

.player-bar.dark .np-meta {
  padding: 0;
  border-radius: 0;
  background: transparent;
  border: none;
  box-shadow: none;
}

.player-bar.dark .np-title {
  color: rgba(255, 255, 255, 0.98);
  text-shadow: none;
}

.player-bar.dark .np-subtitle {
  color: rgba(255, 255, 255, 0.82);
}

.player-bar.dark .volume-pop {
  background: rgba(14, 23, 22, 0.92);
  border-color: rgba(255, 255, 255, 0.14);
}

.player-bar.dark .bar-progress-track {
  background: rgba(255, 255, 255, 0.16);
}

.bar-progress-surface {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  min-width: 0;
  pointer-events: none;
  z-index: 0;
}

.bar-progress-surface {
  background: linear-gradient(90deg, var(--accent-dim) 0%, rgba(22, 214, 160, 0.18) 54%, rgba(6, 86, 69, 0.28) 100%);
}

.bar-progress-hit {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 14px;
  display: flex;
  align-items: center;
  cursor: pointer;
  touch-action: none;
  z-index: 6;
}

.bar-progress-track {
  position: relative;
  width: 100%;
  height: 2px;
  border-radius: 999px;
  transition: height 0.16s ease;
}

.bar-progress-track {
  background: var(--border);
}

.bar-progress-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #7ef5d2 0%, var(--accent) 48%, #0d7c63 100%);
  box-shadow: 0 0 10px rgba(22, 214, 160, 0.26);
  transition: box-shadow 0.16s ease, filter 0.16s ease;
}

.bar-progress-thumb {
  position: absolute;
  top: 50%;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #17dca6;
  transform: translate(-50%, -50%) scale(0);
  box-shadow: 0 0 0 6px rgba(22, 214, 160, 0.16), 0 0 14px rgba(22, 214, 160, 0.38);
  transition: transform 0.12s ease, box-shadow 0.12s ease, opacity 0.12s ease;
  opacity: 0;
}

.bar-progress-hit:hover .bar-progress-thumb,
.bar-progress-hit.dragging .bar-progress-thumb {
  transform: translate(-50%, -50%) scale(1);
  opacity: 1;
}

.bar-progress-hit:hover .bar-progress-track,
.bar-progress-hit.dragging .bar-progress-track {
  height: 3px;
}

.bar-progress-hit.dragging .bar-progress-fill {
  box-shadow: 0 0 18px rgba(22, 214, 160, 0.42);
  filter: brightness(1.08);
}

.bar-shell {
  width: 100%;
  height: 78px;
  position: relative;
  z-index: 2;
}

.now-playing {
  position: absolute;
  left: 4px;
  top: 50%;
  transform: translateY(-50%);
  width: 286px;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 11px;
  text-align: left;
}

.np-cover-toggle {
  flex-shrink: 0;
  border-radius: 10px;
}

.np-cover {
  position: relative;
  width: 52px;
  height: 52px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
}

.np-cover {
  background: var(--bg-hover);
  border: 1px solid var(--border);
}

.np-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.np-cover-overlay {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(7, 13, 12, 0.42);
  color: #f4fcfa;
  opacity: 0;
  transform: scale(0.9);
  transition: var(--transition);
  pointer-events: none;
}

.np-cover-toggle:hover .np-cover-overlay,
.np-cover-toggle.active .np-cover-overlay,
.np-cover-overlay.visible {
  opacity: 1;
  transform: scale(1);
}

.np-cover-ph {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.np-cover-ph {
  color: var(--text-muted);
}

.np-meta {
  min-width: 0;
}

.np-title,
.np-subtitle {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.np-title {
  font-size: 15px;
  font-weight: 600;
}

.np-title {
  color: var(--text-primary);
}

.np-subtitle {
  margin-top: 2px;
  font-size: 13px;
}

.np-subtitle {
  color: var(--text-secondary);
}

.bar-center {
  position: relative;
  width: 390px;
  height: 56px;
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
}

.time-display {
  position: absolute;
  left: 50%;
  top: 50%;
  text-align: center;
  min-width: 200px;
  font-size: 22px;
  font-weight: 800;
  letter-spacing: 0.04em;
  opacity: 0;
  pointer-events: none;
  transform: translate(-50%, -50%) scale(0.96);
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.time-display {
  color: var(--text-primary);
}

.transport-row {
  position: absolute;
  left: 50%;
  top: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 20px;
  transform: translate(-50%, -50%) scale(1);
  transition: opacity 0.16s ease, transform 0.16s ease;
  min-width: max-content;
}

.bar-center.dragging .time-display {
  opacity: 1;
  transform: translate(-50%, -50%) scale(1);
}

.bar-center.dragging .transport-row {
  opacity: 0;
  pointer-events: none;
  transform: translate(-50%, -50%) scale(0.92);
}

.mini-btn,
.icon-btn {
  width: 42px;
  height: 42px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
}

.mini-btn,
.icon-btn {
  color: var(--text-secondary);
}

.mini-btn :deep(svg),
.icon-btn :deep(svg) {
  width: 20px;
  height: 20px;
}

.play-btn :deep(svg) {
  width: 22px;
  height: 22px;
}

.mini-btn:hover:not(:disabled),
.icon-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.mini-btn:disabled {
  opacity: 0.35;
}

.mini-btn.active {
  color: #ff7ba2;
}

.play-btn {
  width: 72px;
  height: 44px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
}

.play-btn {
  background: var(--bg-active);
  color: var(--text-primary);
}

.play-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}

.play-btn:disabled {
  opacity: 0.35;
}

.bar-right {
  position: absolute;
  right: 6px;
  top: 50%;
  transform: translateY(-50%);
  width: 188px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.volume-hover {
  position: relative;
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 40;
}

.volume-pop {
  position: absolute;
  left: 50%;
  bottom: calc(100% + 10px);
  width: 56px;
  padding: 10px 10px;
  border-radius: 18px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  opacity: 0;
  pointer-events: none;
  transform: translateX(-50%) translateY(8px);
  transition: var(--transition);
  box-shadow: var(--window-shadow);
  z-index: 50;
}

.volume-pop {
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
}

.volume-hover:hover .volume-pop,
.volume-hover:focus-within .volume-pop {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(-50%) translateY(0);
}

.volume-pop.open {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(-50%) translateY(0);
}

.volume-pop-value {
  font-size: 11px;
}

.volume-pop-value {
  color: var(--text-secondary);
}

.volume-pop-track {
  width: 20px;
  height: 108px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.volume-slider-vertical {
  width: 104px;
  -webkit-appearance: none;
  height: 2px;
  border-radius: 999px;
  transform: rotate(-90deg);
}

.volume-slider-vertical {
  background: var(--bg-active);
}

.volume-slider-vertical::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--accent);
}

.lyric-btn.active {
  background: var(--accent-dim);
  color: var(--accent);
  box-shadow: inset 0 0 0 1px rgba(22, 214, 160, 0.14);
}

@media (max-width: 980px) {
  .bar-shell {
    height: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding-top: 8px;
  }

  .bar-right {
    position: static;
    transform: none;
    width: auto;
    justify-content: flex-start;
  }

  .now-playing {
    position: static;
    transform: none;
    width: auto;
  }

  .bar-center {
    position: static;
    transform: none;
    width: auto;
  }
}
</style>

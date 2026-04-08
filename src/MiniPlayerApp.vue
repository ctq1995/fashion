<template>
  <main
    class="mini-player-shell"
    :data-theme="ui.theme"
    :data-docked-edge="dockedEdge ?? ''"
    :data-collapsed="collapsed ? 'true' : 'false'"
    @mousedown="startDragging"
  >
    <div class="mini-player-card" data-tauri-drag-region>
      <div class="mini-grid">
        <button type="button" class="cover-button" data-tauri-drag-region :title="currentTrack?.name ?? '未在播放'">
          <img v-if="currentTrack?.coverUrl" :src="currentTrack.coverUrl" class="cover-image" />
          <div v-else class="cover-fallback">
            <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7">
              <circle cx="12" cy="12" r="9" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </button>

        <section class="mini-body">
          <header class="mini-topbar" data-tauri-drag-region>
            <div class="track-copy" data-tauri-drag-region>
              <strong class="track-title">{{ currentTrack?.name ?? '未在播放' }}</strong>
              <span class="track-subtitle">{{ subtitleText }}</span>
            </div>
            <div class="window-actions" data-no-drag>
              <button type="button" class="window-btn" :title="pinTitle" @click="toggleAlwaysOnTop">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 17v5" />
                  <path d="M8 7V3h8v4" />
                  <path d="M5 7h14l-3 6H8z" />
                </svg>
              </button>
              <button type="button" class="window-btn" title="回到主窗口" @click="showMainWindow">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M15 18l-6-6 6-6" />
                </svg>
              </button>
              <button type="button" class="window-btn close" title="关闭迷你播放器" @click="hideWindow">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="m18 6-12 12" />
                  <path d="m6 6 12 12" />
                </svg>
              </button>
            </div>
          </header>

          <div class="progress-shell" data-no-drag>
            <button
              ref="progressTrackRef"
              type="button"
              class="progress-track"
              data-no-drag
              :disabled="!currentTrack || !totalDuration"
              @click="handleProgressClick"
            >
              <div class="progress-fill" :style="{ width: `${progressPct}%` }" />
            </button>
            <div class="progress-meta">
              <span>{{ formatTime(currentTime) }}</span>
              <span>{{ formatTime(totalDuration) }}</span>
            </div>
          </div>

          <footer class="transport-row" data-no-drag>
            <button type="button" class="transport-btn mode-btn" :disabled="!currentTrack" @click="handleTogglePlayMode" :title="modeLabel">
              <span v-html="modeIcon" />
            </button>
            <button type="button" class="transport-btn" :disabled="!currentTrack" @click="handlePlayPrev" title="上一首">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 6h2v12H6zm3.5 6 8.5 6V6z" />
              </svg>
            </button>
            <button type="button" class="transport-primary" :disabled="!currentTrack" @click="handleTogglePlay" :title="playTitle">
              <svg v-if="!isPlaying" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="6,4 20,12 6,20" />
              </svg>
              <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <rect x="6" y="4" width="4" height="16" rx="1" />
                <rect x="14" y="4" width="4" height="16" rx="1" />
              </svg>
            </button>
            <button type="button" class="transport-btn" :disabled="!currentTrack" @click="handlePlayNext" title="下一首">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 18l8.5-6L6 6v12zm10-12h2v12h-2z" />
              </svg>
            </button>
            <button type="button" class="transport-btn lyric-btn text-lyric-btn" @click="handleToggleDesktopLyric" :title="desktopLyricTitle">
              <span class="lyric-btn-text">词</span>
            </button>
          </footer>
        </section>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import type { MiniPlayerStateSnapshot, Track, PlayMode } from '@/stores/player';
import { useUiStore } from '@/stores/ui';
import {
  MINI_PLAYER_CLOSED_EVENT,
  MINI_PLAYER_DOCK_STATE_EVENT,
  MINI_PLAYER_HIDE_EVENT,
  MINI_PLAYER_PLAY_NEXT_EVENT,
  MINI_PLAYER_PLAY_PREV_EVENT,
  MINI_PLAYER_READY_EVENT,
  MINI_PLAYER_SEEK_EVENT,
  MINI_PLAYER_STATE_EVENT,
  MINI_PLAYER_TOGGLE_DESKTOP_LYRIC_EVENT,
  MINI_PLAYER_TOGGLE_MODE_EVENT,
  MINI_PLAYER_TOGGLE_PLAY_EVENT,
} from '@/utils/miniPlayer';

type MiniPlayerDockStatePayload = {
  dockedEdge: 'left' | 'right' | 'top' | 'bottom' | null;
  collapsed: boolean;
};

const ui = useUiStore();
const appWindow = getCurrentWindow();
const alwaysOnTop = ref(true);
const progressTrackRef = ref<HTMLElement | null>(null);
const currentTrack = ref<Track | null>(null);
const isPlaying = ref(false);
const duration = ref(0);
const currentTime = ref(0);
const playMode = ref<PlayMode>('sequence');
const dockedEdge = ref<MiniPlayerDockStatePayload['dockedEdge']>(null);
const collapsed = ref(false);
let cleanupMiniStateListener: null | (() => void) = null;
let cleanupMiniDockStateListener: null | (() => void) = null;

const subtitleText = computed(() => {
  if (!currentTrack.value) return '仅“回到主窗口”按钮可返回';
  return [currentTrack.value.artist, currentTrack.value.album].filter(Boolean).join(' · ');
});

const totalDuration = computed(() => {
  return duration.value || currentTrack.value?.durationSec || 0;
});

const progressPct = computed(() => {
  if (!totalDuration.value) return 0;
  return Math.max(0, Math.min(100, (currentTime.value / totalDuration.value) * 100));
});

const playTitle = computed(() => isPlaying.value ? '暂停' : '播放');
const pinTitle = computed(() => alwaysOnTop.value ? '取消置顶' : '保持置顶');
const showMainTitle = '回到主窗口';
const desktopLyricTitle = '切换桌面歌词';
const modeLabel = computed(() => ({
  sequence: '顺序播放',
  random: '随机播放',
  single: '单曲循环',
}[playMode.value]));
const modeIcon = computed(() => {
  if (playMode.value === 'random') {
    return '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/></svg>';
  }
  if (playMode.value === 'single') {
    return '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/><circle cx="12" cy="12" r="2.6"/><path d="M12 10.8v2.4"/><path d="M11.3 11.4H12"/></svg>';
  }
  return '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/></svg>';
});

function formatTime(value: number) {
  if (!Number.isFinite(value) || value <= 0) return '00:00';
  const totalSeconds = Math.floor(value);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

function applySnapshot(snapshot: MiniPlayerStateSnapshot) {
  console.log('[mini-sync][mini] apply snapshot', {
    hasTrack: !!snapshot.currentTrack,
    trackName: snapshot.currentTrack?.name ?? null,
    isPlaying: snapshot.isPlaying,
    duration: snapshot.duration,
    currentTime: snapshot.currentTime,
    playMode: snapshot.playMode,
  });
  currentTrack.value = snapshot.currentTrack;
  isPlaying.value = snapshot.isPlaying;
  duration.value = snapshot.duration;
  currentTime.value = snapshot.currentTime;
  playMode.value = snapshot.playMode;
}

async function emitMainEvent(event: string, payload?: unknown) {
  await invoke('emit_app_event', { event, payload: payload ?? null });
}

async function handleProgressClick(event: MouseEvent) {
  if (!currentTrack.value || !totalDuration.value || !progressTrackRef.value) return;
  const rect = progressTrackRef.value.getBoundingClientRect();
  if (rect.width <= 0) return;
  const ratio = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
  await emitMainEvent(MINI_PLAYER_SEEK_EVENT, { time: totalDuration.value * ratio });
}

async function startDragging(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest('[data-no-drag]')) return;
  try {
    await invoke('mini_player_start_dragging');
  } catch (error) {
    console.error('mini player start dragging failed', error);
  }
}

async function showMainWindow() {
  try {
    await emitMainEvent(MINI_PLAYER_CLOSED_EVENT);
    await invoke('window_hide');
  } catch (error) {
    console.error('show main window failed', error);
  }
}

async function hideWindow() {
  try {
    await emitMainEvent(MINI_PLAYER_HIDE_EVENT);
    await invoke('window_hide');
  } catch (error) {
    console.error('hide mini player failed', error);
  }
}

async function handleTogglePlay() {
  try {
    await emitMainEvent(MINI_PLAYER_TOGGLE_PLAY_EVENT);
  } catch (error) {
    console.error('mini player toggle play failed', error);
  }
}

async function handlePlayPrev() {
  try {
    await emitMainEvent(MINI_PLAYER_PLAY_PREV_EVENT);
  } catch (error) {
    console.error('mini player play prev failed', error);
  }
}

async function handlePlayNext() {
  try {
    await emitMainEvent(MINI_PLAYER_PLAY_NEXT_EVENT);
  } catch (error) {
    console.error('mini player play next failed', error);
  }
}

async function handleTogglePlayMode() {
  try {
    await emitMainEvent(MINI_PLAYER_TOGGLE_MODE_EVENT);
  } catch (error) {
    console.error('mini player toggle play mode failed', error);
  }
}

async function handleToggleDesktopLyric() {
  try {
    await emitMainEvent(MINI_PLAYER_TOGGLE_DESKTOP_LYRIC_EVENT);
  } catch (error) {
    console.error('mini player toggle desktop lyric failed', error);
  }
}

async function toggleAlwaysOnTop() {
  const next = !alwaysOnTop.value;
  try {
    await invoke('window_set_always_on_top', { alwaysOnTop: next });
    alwaysOnTop.value = next;
  } catch (error) {
    console.error('toggle mini player always on top failed', error);
  }
}

onMounted(async () => {
  try {
    alwaysOnTop.value = await appWindow.isAlwaysOnTop();
  } catch {
    alwaysOnTop.value = true;
  }

  cleanupMiniStateListener = await listen<string>(MINI_PLAYER_STATE_EVENT, (event) => {
    console.log('[mini-sync][mini] state event received', {
      payloadType: typeof event.payload,
      payloadPreview: typeof event.payload === 'string' ? event.payload.slice(0, 160) : event.payload,
    });
    try {
      const snapshot = JSON.parse(event.payload) as MiniPlayerStateSnapshot;
      applySnapshot(snapshot);
    } catch (error) {
      console.error('apply mini player snapshot failed', error);
    }
  });

  cleanupMiniDockStateListener = await listen<MiniPlayerDockStatePayload>(MINI_PLAYER_DOCK_STATE_EVENT, (event) => {
    dockedEdge.value = event.payload.dockedEdge;
    collapsed.value = event.payload.collapsed;
  });

  try {
    console.log('[mini-sync][mini] emit ready event');
    await emitMainEvent(MINI_PLAYER_READY_EVENT);
  } catch (error) {
    console.error('emit mini player ready failed', error);
  }
});

onBeforeUnmount(() => {
  cleanupMiniStateListener?.();
  cleanupMiniStateListener = null;
  cleanupMiniDockStateListener?.();
  cleanupMiniDockStateListener = null;
});

</script>

<style scoped>
@keyframes miniPlayerReveal {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.mini-player-shell {
  width: 100vw;
  height: 100vh;
  padding: 8px;
  box-sizing: border-box;
  background: transparent;
  color: #f4f7fb;
  user-select: none;
}

.mini-player-card[data-tauri-drag-region] {
  cursor: grab;
}

.mini-player-card[data-tauri-drag-region]:active {
  cursor: grabbing;
}

.mini-player-card {
  animation: miniPlayerReveal 160ms ease-out;
}

.mini-player-shell[data-theme='light'] {
  color: #1f2937;
}

.mini-player-card {
  height: 100%;
  padding: 12px;
  border-radius: 20px;
  background: linear-gradient(145deg, rgba(13, 18, 30, 0.9), rgba(27, 45, 74, 0.82));
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: none;
  backdrop-filter: blur(22px);
}

.mini-player-shell[data-collapsed='true'] .mini-player-card {
  box-shadow: none;
}

.mini-player-shell[data-theme='light'] .mini-player-card {
  background: linear-gradient(145deg, rgba(255, 255, 255, 0.94), rgba(238, 245, 255, 0.92));
  border-color: rgba(148, 163, 184, 0.28);
  box-shadow: none;
}

.mini-grid {
  height: 100%;
  display: grid;
  grid-template-columns: 88px minmax(0, 1fr);
  gap: 12px;
  align-items: stretch;
}

.cover-button {
  align-self: center;
  border: none;
  padding: 0;
  width: 88px;
  height: 88px;
  border-radius: 16px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.1);
  display: grid;
  place-items: center;
  cursor: grab;
}

.cover-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-fallback {
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
  color: currentColor;
  opacity: 0.72;
}

.mini-body {
  min-width: 0;
  display: grid;
  grid-template-rows: auto auto 1fr;
  gap: 10px;
}

.mini-topbar {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  align-items: start;
}

.track-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.track-title,
.track-subtitle {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.track-title {
  font-size: 14px;
  line-height: 1.3;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.track-subtitle {
  font-size: 12px;
  line-height: 1.25;
  opacity: 0.8;
}

.window-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 6px;
}

.window-btn,
.transport-btn,
.transport-primary,
.cover-button,
.progress-track {
  border: none;
  cursor: pointer;
  color: inherit;
  transition: background-color 0.18s ease, border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease, opacity 0.18s ease;
}

.window-btn {
  width: 28px;
  height: 28px;
  border-radius: 9px;
  background: rgba(255, 255, 255, 0.1);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.window-btn.close {
  background: rgba(248, 113, 113, 0.18);
}

.progress-shell {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-track {
  width: 100%;
  height: 8px;
  padding: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.16);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #22c55e, #60a5fa);
}

.progress-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 11px;
  opacity: 0.76;
  font-variant-numeric: tabular-nums;
}

.transport-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-width: 0;
}

.lyric-btn.text-lyric-btn {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.lyric-btn-text {
  line-height: 1;
  transform: translateY(-0.5px);
}

.transport-btn.lyric-btn {
  color: #cfe8ff;
}

.transport-btn.lyric-btn:hover {
  color: #ffffff;
}

.mode-btn {
  width: 36px;
  height: 36px;
}

.transport-btn,
.transport-primary {
  width: 38px;
  height: 38px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.1);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.transport-primary {
  width: 46px;
  height: 46px;
  background: linear-gradient(135deg, #22c55e, #0ea5e9);
  color: white;
}

.cover-button:hover,
.window-btn:hover,
.transport-btn:hover {
  background: rgba(255, 255, 255, 0.16);
}

.cover-button:active,
.window-btn:active,
.transport-btn:active,
.transport-primary:active,
.progress-track:active {
  transform: scale(0.98);
}

.transport-primary:hover {
  box-shadow: 0 10px 20px rgba(34, 197, 94, 0.28);
}

.window-btn.close:hover {
  background: rgba(248, 113, 113, 0.28);
}

.transport-btn:disabled,
.transport-primary:disabled,
.progress-track:disabled {
  cursor: default;
  opacity: 0.42;
}

.mini-player-shell[data-theme='light'] .window-btn,
.mini-player-shell[data-theme='light'] .transport-btn,
.mini-player-shell[data-theme='light'] .cover-button,
.mini-player-shell[data-theme='light'] .progress-track {
  background: rgba(148, 163, 184, 0.14);
}

@media (max-width: 400px) {
  .mini-player-shell {
    padding: 6px;
  }

  .mini-player-card {
    padding: 10px;
    border-radius: 18px;
  }

  .mini-grid {
    grid-template-columns: 76px minmax(0, 1fr);
    gap: 10px;
  }

  .cover-button {
    width: 76px;
    height: 76px;
    border-radius: 14px;
  }

  .mini-body {
    gap: 8px;
  }

  .window-actions {
    gap: 4px;
  }

  .window-btn {
    width: 26px;
    height: 26px;
    border-radius: 8px;
  }

  .transport-row {
    gap: 8px;
  }

  .mode-btn,
  .transport-btn {
    width: 34px;
    height: 34px;
  }

  .transport-primary {
    width: 42px;
    height: 42px;
  }
}
</style>

<template>
  <div
    class="app-shell"
    :data-theme="ui.theme"
    :class="[sceneClass, { 'window-fill': isWindowFill, immersive: isImmersiveScene }]"
  >
    <div class="bg-blur" :style="blurStyle" />
    <div
      class="app-layout"
      :class="{ immersive: isImmersiveScene, 'window-fill': isWindowFill }"
    >
      <div class="workspace" :class="{ immersive: isImmersiveScene }">
        <Sidebar
          v-if="!isImmersiveScene"
          :active="activeNav"
          :scene="isImmersiveScene ? 'dark' : 'light'"
          @update:active="navigateTo"
        />
        <div class="main-stack">
          <TitleBar
            :active-panel="activePanel"
            :can-go-back="canGoBack"
            :can-go-forward="canGoForward"
            :scene="isImmersiveScene ? 'dark' : 'light'"
            :window-fill="isWindowFill"
            :hidden="hideLyricChrome"
            :lyric-fullscreen="isLyricFullscreen"
            @back="goBack"
            @forward="goForward"
            @search="handleToolbarSearch"
            @open-settings="navigateTo('settings')"
            @toggle-lyric-fullscreen="toggleLyricFullscreen()"
          />
          <div class="content-area" :class="{ immersive: isImmersiveScene }">
            <Transition name="fade" mode="out-in">
              <SearchPanel
                v-if="activePanel === 'search'"
                key="search"
                :mode="searchMode"
                @open-library="playlistDrawerOpen = true"
                @open-history="navigateTo('history')"
              />
              <FavoritesPanel v-else-if="activePanel === 'favorites'" key="favorites" />
              <HistoryPanel v-else-if="activePanel === 'history'" key="history" />
              <LyricPanel
                v-else-if="activePanel === 'lyric'"
                key="lyric"
                :fullscreen="isLyricFullscreen"
                @toggle-fullscreen="toggleLyricFullscreen(false)"
              />
              <SettingsPanel v-else-if="activePanel === 'settings'" key="settings" />
            </Transition>
          </div>
        </div>

        <Transition name="drawer-fade">
          <div
            v-if="playlistDrawerOpen"
            class="playlist-overlay"
            @click.self="playlistDrawerOpen = false"
          >
            <PlaylistPanel @close="playlistDrawerOpen = false" />
          </div>
        </Transition>
      </div>
      <PlayerBar
        :scene="isImmersiveScene ? 'dark' : 'light'"
        :lyric-active="isImmersiveScene"
        :lyric-fullscreen="isLyricFullscreen"
        :hidden="hideLyricChrome"
        @open-lyric="toggleLyricPanel"
        @open-playlist="playlistDrawerOpen = !playlistDrawerOpen"
        @toggle-lyric-fullscreen="toggleLyricFullscreen()"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { usePlayerStore } from '@/stores/player';
import { useUiStore } from '@/stores/ui';
import TitleBar from '@/components/TitleBar.vue';
import Sidebar from '@/components/Sidebar.vue';
import SearchPanel from '@/components/SearchPanel.vue';
import FavoritesPanel from '@/components/FavoritesPanel.vue';
import PlaylistPanel from '@/components/PlaylistPanel.vue';
import HistoryPanel from '@/components/HistoryPanel.vue';
import LyricPanel from '@/components/LyricPanel.vue';
import SettingsPanel from '@/components/SettingsPanel.vue';
import PlayerBar from '@/components/PlayerBar.vue';

type Panel = 'search' | 'favorites' | 'history' | 'lyric' | 'settings';
type NavKey = 'recommend' | 'discover' | 'favorites' | 'history' | 'settings';
type WindowState = {
  isFill: boolean;
  isLyricFullscreen: boolean;
};

const player = usePlayerStore();
const ui = useUiStore();

const activePanel = ref<Panel>('search');
const activeNav = ref<NavKey>('recommend');
const searchMode = ref<'recommend' | 'discover'>('recommend');
const panelHistory = ref<Panel[]>(['search']);
const historyIndex = ref(0);
const playlistDrawerOpen = ref(false);
const isWindowFill = ref(false);
const isLyricWindowFullscreen = ref(false);
const showLyricChrome = ref(true);

const isImmersiveScene = computed(() => activePanel.value === 'lyric');
const sceneClass = computed(() => ui.theme === 'dark' ? 'scene-dark' : 'scene-light');
const isLyricFullscreen = computed(() => isImmersiveScene.value && isLyricWindowFullscreen.value);
const hideLyricChrome = computed(() => isLyricFullscreen.value && !showLyricChrome.value);

const blurStyle = computed(() => ({
  backgroundImage: player.currentTrack?.coverUrl
    ? `url(${player.currentTrack.coverUrl})`
    : 'none',
  opacity: player.currentTrack?.coverUrl ? '1' : '0',
}));

const canGoBack = computed(() => historyIndex.value > 0);
const canGoForward = computed(() => historyIndex.value < panelHistory.value.length - 1);

function setPanel(panel: Panel, recordHistory = true) {
  if (activePanel.value === panel && recordHistory) return;

  activePanel.value = panel;

  if (!recordHistory) return;

  panelHistory.value = panelHistory.value.slice(0, historyIndex.value + 1);
  panelHistory.value.push(panel);
  historyIndex.value = panelHistory.value.length - 1;
}

function navigateTo(target: string) {
  if (target === 'recommend' || target === 'discover') {
    activeNav.value = target;
    searchMode.value = target;
    setPanel('search');
    playlistDrawerOpen.value = false;
    return;
  }

  const panel = target as Panel;
  setPanel(panel);
  if (panel === 'favorites' || panel === 'history' || panel === 'settings') {
    activeNav.value = panel;
  }
  playlistDrawerOpen.value = false;
}

function goBack() {
  if (!canGoBack.value) return;
  historyIndex.value -= 1;
  setPanel(panelHistory.value[historyIndex.value], false);
}

function goForward() {
  if (!canGoForward.value) return;
  historyIndex.value += 1;
  setPanel(panelHistory.value[historyIndex.value], false);
}

function handleToolbarSearch(value: string) {
  ui.submitToolbarSearch(value);
  setPanel('search');
}

function toggleLyricPanel() {
  if (activePanel.value === 'lyric') {
    if (canGoBack.value) {
      goBack();
      return;
    }
    navigateTo('recommend');
    return;
  }
  navigateTo('lyric');
}

let cleanupDomResize: null | (() => void) = null;
let lyricChromeTimer: number | null = null;
let syncSeq = 0;
let fullscreenPending = false;
let toggleSeq = 0;

function clearLyricChromeTimer() {
  if (lyricChromeTimer !== null) {
    window.clearTimeout(lyricChromeTimer);
    lyricChromeTimer = null;
  }
}

function scheduleLyricChromeHide() {
  clearLyricChromeTimer();
  lyricChromeTimer = window.setTimeout(() => {
    showLyricChrome.value = false;
    lyricChromeTimer = null;
  }, 1800);
}

function showChrome() {
  showLyricChrome.value = true;
  if (isLyricFullscreen.value) scheduleLyricChromeHide();
}

async function syncWindowFillState() {
  const seq = ++syncSeq;
  try {
    const state = await invoke<WindowState>('window_get_state');
    if (seq !== syncSeq) return;
    isWindowFill.value = state.isFill;
    isLyricWindowFullscreen.value = state.isLyricFullscreen;
  } catch (error) {
    console.error('syncWindowFillState failed', error);
  }
}

async function toggleLyricFullscreen(force?: boolean) {
  const target = force ?? !isLyricWindowFullscreen.value;
  console.log('[fullscreen] toggleLyricFullscreen called, target=', target, 'current=', isLyricWindowFullscreen.value, 'pending=', fullscreenPending);
  // 目标状态与当前一致则忽略（乐观更新后的重复调用）
  if (target === isLyricWindowFullscreen.value && fullscreenPending) {
    console.log('[fullscreen] skipped, same target');
    return;
  }
  fullscreenPending = true;
  const seq = ++toggleSeq;
  // 乐观更新：立即反映目标状态，让 UI 即时响应
  isLyricWindowFullscreen.value = target;
  isWindowFill.value = target;
  showLyricChrome.value = true;
  if (target) scheduleLyricChromeHide();
  else clearLyricChromeTimer();
  try {
    const state = await invoke<WindowState>('window_toggle_lyric_fullscreen', { force: target });
    console.log('[fullscreen] invoke returned:', state, 'seq=', seq, 'current=', toggleSeq);
    if (seq !== toggleSeq) {
      console.log('[fullscreen] stale result, discarded');
      return;
    }
    syncSeq++;
    isWindowFill.value = state.isFill;
    isLyricWindowFullscreen.value = state.isLyricFullscreen;
    showLyricChrome.value = true;
    if (state.isLyricFullscreen) scheduleLyricChromeHide();
    else clearLyricChromeTimer();
    window.setTimeout(() => { void syncWindowFillState(); }, 300);
  } catch (error) {
    console.error('[fullscreen] toggleLyricFullscreen failed', error);
    if (seq === toggleSeq) void syncWindowFillState();
  } finally {
    if (seq === toggleSeq) fullscreenPending = false;
  }
}

onMounted(async () => {
  await syncWindowFillState();

  const onResize = () => { void syncWindowFillState(); };
  const onActivity = () => { if (isLyricFullscreen.value) showChrome(); };

  window.addEventListener('resize', onResize);
  window.addEventListener('pointermove', onActivity);
  window.addEventListener('pointerdown', onActivity);
  window.addEventListener('keydown', onActivity);

  cleanupDomResize = () => {
    window.removeEventListener('resize', onResize);
    window.removeEventListener('pointermove', onActivity);
    window.removeEventListener('pointerdown', onActivity);
    window.removeEventListener('keydown', onActivity);
  };
});

onBeforeUnmount(() => {
  cleanupDomResize?.();
  cleanupDomResize = null;
  clearLyricChromeTimer();
});

// 全屏状态变化时重置 chrome 显示
watch(isLyricFullscreen, (value) => {
  showLyricChrome.value = true;
  if (value) scheduleLyricChromeHide();
  else clearLyricChromeTimer();
});

watch(isImmersiveScene, (value) => {
  console.log('[fullscreen] isImmersiveScene changed:', value, 'isLyricWindowFullscreen=', isLyricWindowFullscreen.value);
  if (value) return;
  clearLyricChromeTimer();
  showLyricChrome.value = true;
  if (isLyricWindowFullscreen.value) {
    void toggleLyricFullscreen(false);
  }
});

function handleLyricFullscreenKeydown(event: KeyboardEvent) {
  if (!isImmersiveScene.value) return;

  if (event.key === 'F11') {
    event.preventDefault();
    void toggleLyricFullscreen();
    return;
  }

  if (event.key === 'Escape' && isLyricWindowFullscreen.value) {
    event.preventDefault();
    void toggleLyricFullscreen(false);
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleLyricFullscreenKeydown, true);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleLyricFullscreenKeydown, true);
});
</script>

<style scoped>
.app-shell {
  width: 100vw;
  height: 100vh;
  position: relative;
  overflow: hidden;
  border-radius: 10px;
  background: var(--bg-primary);
}

.app-shell.window-fill {
  border-radius: 0;
}

.app-shell.immersive {
  border-radius: 0;
}

.app-shell::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  box-shadow:
    inset 0 0 0 1px var(--shell-outline),
    inset 0 0 0 2px var(--shell-outline-strong);
  z-index: 3;
}

.app-shell.window-fill::after {
  box-shadow: none;
}

.app-shell.immersive::after {
  box-shadow: none;
}

.scene-light {
  background:
    radial-gradient(circle at top, rgba(131, 152, 146, 0.08), transparent 35%),
    var(--bg-primary);
}

.scene-light.app-shell::after {
  box-shadow:
    inset 0 0 0 1px rgba(83, 110, 102, 0.3),
    inset 0 0 0 2px rgba(255, 255, 255, 0.7);
}

.scene-dark.app-shell::after {
  box-shadow: none;
}

.scene-dark {
  background:
    radial-gradient(circle at 14% 18%, var(--accent-dim), transparent 34%),
    linear-gradient(180deg, var(--bg-secondary), var(--bg-primary));
}

.bg-blur {
  position: absolute;
  inset: 0;
  background-size: cover;
  background-position: center;
  filter: var(--cover-filter);
  transform: scale(1.15);
  transition: opacity 1s ease, background-image 1s ease;
  z-index: 0;
}

.app-layout {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  border-radius: inherit;
  overflow: hidden;
  box-shadow: none;
}

.app-layout.window-fill {
  border-radius: 0;
  box-shadow: none;
}

.app-layout.immersive {
  border-radius: 0;
}

.scene-light .app-layout {
  background: linear-gradient(180deg, var(--panel-strong), var(--panel-shell));
  box-shadow: inset 0 0 0 1px var(--border), var(--window-shadow);
  backdrop-filter: blur(20px);
}

.scene-light .app-layout.window-fill {
  box-shadow: none;
}

.scene-dark .app-layout {
  background: transparent;
  box-shadow: inset 0 0 0 1px var(--border);
}

.scene-dark .app-layout.window-fill {
  box-shadow: none;
}

/* 歌词沉浸：统一压暗，消除顶部/中部色差 */
.app-layout.immersive {
  background: rgba(8, 14, 13, 0.46);
  box-shadow: none;
}

.workspace {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
  position: relative;
}

.workspace.immersive {
  grid-template-columns: minmax(0, 1fr);
}

.main-stack {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.content-area {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 4px 10px 0 0;
}

.content-area.immersive {
  padding: 0;
}

.playlist-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  justify-content: flex-end;
  background: linear-gradient(90deg, rgba(0, 0, 0, 0), var(--overlay-mask));
  z-index: 20;
}

.drawer-fade-enter-active,
.drawer-fade-leave-active {
  transition: opacity 0.18s ease;
}

.drawer-fade-enter-from,
.drawer-fade-leave-to {
  opacity: 0;
}
</style>

<template>
  <div
    class="app-shell"
    :data-theme="ui.theme"
    :class="[sceneClass, { 'window-fill': isWindowFill, immersive: isImmersiveScene, 'mobile-shell': isMobileLayout }]"
  >
    <div class="bg-blur" :style="blurStyle" />
    <div
      class="app-layout"
      :class="{ immersive: isImmersiveScene, 'window-fill': isWindowFill, mobile: isMobileLayout }"
    >
      <template v-if="isMobileLayout">
        <div class="mobile-layout" :class="{ immersive: isImmersiveScene }">
          <div class="mobile-scroll app-scroll" :class="{ immersive: isImmersiveScene }">
            <MobileHeader
              :active-panel="activePanel"
              :immersive="isImmersiveScene"
              @search="handleMobileSearch"
              @open-settings="navigateTo('settings')"
              @open-playlist="playlistDrawerOpen = true"
              @open-lyric="toggleLyricPanel"
            />
            <div class="mobile-content-shell" :class="{ immersive: isImmersiveScene }">
              <AppPanels
                :active-panel="activePanel"
                :search-mode="searchMode"
                :lyric-fullscreen="isLyricFullscreen"
                @open-library="playlistDrawerOpen = true"
                @open-history="navigateTo('history')"
                @toggle-fullscreen="toggleLyricFullscreen(false)"
              />
            </div>
          </div>

          <Transition name="drawer-fade">
            <div
              v-if="playlistDrawerOpen"
              class="playlist-overlay mobile"
              @click.self="playlistDrawerOpen = false"
            >
              <PlaylistPanel @close="playlistDrawerOpen = false" />
            </div>
          </Transition>

          <div class="mobile-bottom-stack">
            <MobilePlayerDock
              :immersive="isImmersiveScene"
              :hidden="hideLyricChrome"
              @open-lyric="toggleLyricPanel"
              @open-playlist="playlistDrawerOpen = !playlistDrawerOpen"
            />
            <MobileTabBar
              v-if="!hideLyricChrome"
              :active-panel="activePanel"
              @navigate="navigateTo"
            />
          </div>
        </div>
      </template>

      <template v-else>
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
              :show-window-controls="supportsWindowControls"
              :allow-window-dragging="supportsWindowControls"
              :show-lyric-window-toggle="supportsWindowControls"
              :show-mini-player-toggle="supportsWindowControls"
              @back="goBack"
              @forward="goForward"
              @search="handleToolbarSearch"
              @open-settings="navigateTo('settings')"
              @toggle-mini-player="toggleMiniPlayerWindow()"
              @toggle-lyric-fullscreen="toggleLyricFullscreen()"
            />
            <div class="content-area" :class="{ immersive: isImmersiveScene }">
              <AppPanels
                :active-panel="activePanel"
                :search-mode="searchMode"
                :lyric-fullscreen="isLyricFullscreen"
                @open-library="playlistDrawerOpen = true"
                @open-history="navigateTo('history')"
                @toggle-fullscreen="toggleLyricFullscreen(false)"
              />
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
        <Transition name="player-bar-fade" mode="out-in">
          <PlayerBar
            :scene="isImmersiveScene ? 'dark' : 'light'"
            :lyric-active="isImmersiveScene"
            :lyric-fullscreen="isLyricFullscreen"
            :desktop-lyric-open="supportsDesktopLyricWindow && desktopLyricVisible"
            :show-desktop-lyric-button="supportsDesktopLyricWindow"
            :hidden="hideLyricChrome"
            @open-lyric="toggleLyricPanel"
            @toggle-desktop-lyric="toggleDesktopLyricWindow"
            @open-playlist="playlistDrawerOpen = !playlistDrawerOpen"
            @toggle-lyric-fullscreen="toggleLyricFullscreen()"
          />
        </Transition>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { emitTo, listen } from '@tauri-apps/api/event';
import { PhysicalPosition } from '@tauri-apps/api/dpi';
import { availableMonitors, getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { computed, onBeforeUnmount, onMounted, ref, watch, watchEffect } from 'vue';
import type { MiniPlayerStateSnapshot } from '@/stores/player';
import AppPanels from '@/components/AppPanels.vue';
import MobileHeader from '@/components/MobileHeader.vue';
import MobilePlayerDock from '@/components/MobilePlayerDock.vue';
import MobileTabBar from '@/components/MobileTabBar.vue';
import PlayerBar from '@/components/PlayerBar.vue';
import PlaylistPanel from '@/components/PlaylistPanel.vue';
import Sidebar from '@/components/Sidebar.vue';
import TitleBar from '@/components/TitleBar.vue';
import { usePlayerStore } from '@/stores/player';
import { useUiStore } from '@/stores/ui';
import { useRuntimeInfo } from '@/utils/runtime';
import {
  DESKTOP_LYRIC_ACTION_EVENT,
  DESKTOP_LYRIC_CLOSED_EVENT,
  DESKTOP_LYRIC_MOVED_EVENT,
  DESKTOP_LYRIC_READY_EVENT,
  DESKTOP_LYRIC_STATE_EVENT,
  DESKTOP_LYRIC_WINDOW_HEIGHT,
  DESKTOP_LYRIC_WINDOW_LABEL,
  DESKTOP_LYRIC_WINDOW_QUERY,
  DESKTOP_LYRIC_WINDOW_WIDTH,
  sameDesktopLyricSettings,
  type DesktopLyricActionPayload,
  type DesktopLyricSettings,
  type DesktopLyricStatePayload,
  type DesktopLyricWindowPosition,
} from '@/utils/desktopLyric';
import {
  MINI_PLAYER_CLOSED_EVENT,
  MINI_PLAYER_HIDE_EVENT,
  MINI_PLAYER_PLAY_NEXT_EVENT,
  MINI_PLAYER_PLAY_PREV_EVENT,
  MINI_PLAYER_READY_EVENT,
  MINI_PLAYER_SEEK_EVENT,
  MINI_PLAYER_STATE_EVENT,
  MINI_PLAYER_TOGGLE_DESKTOP_LYRIC_EVENT,
  MINI_PLAYER_TOGGLE_MODE_EVENT,
  MINI_PLAYER_TOGGLE_PLAY_EVENT,
  MINI_PLAYER_WINDOW_LABEL,
  MINI_PLAYER_WINDOW_QUERY,
} from '@/utils/miniPlayer';
import {
  TRAY_EXIT_EVENT,
  TRAY_PLAY_NEXT_EVENT,
  TRAY_PLAY_PREV_EVENT,
  TRAY_SHOW_MAIN_EVENT,
  TRAY_TOGGLE_PLAY_EVENT,
} from '@/utils/tray';
import {
  findTranslatedLine,
  resolveLyricLineDuration,
  resolveLyricLineProgress,
} from '@/utils/lyrics';

type Panel = 'search' | 'favorites' | 'local-library' | 'history' | 'lyric' | 'settings';
type NavKey = 'recommend' | 'discover' | 'favorites' | 'local-library' | 'history' | 'settings';
type WindowState = {
  isFill: boolean;
  isLyricFullscreen: boolean;
};

const player = usePlayerStore();
const ui = useUiStore();
const runtime = useRuntimeInfo();

const activePanel = ref<Panel>('search');
const activeNav = ref<NavKey>('recommend');
const searchMode = ref<'recommend' | 'discover'>('recommend');
const panelHistory = ref<Panel[]>(['search']);
const historyIndex = ref(0);
const playlistDrawerOpen = ref(false);
const isWindowFill = ref(false);
const isLyricWindowFullscreen = ref(false);
const showLyricChrome = ref(true);
const desktopLyricVisible = ref(false);
const desktopLyricPending = ref(false);
const miniPlayerVisible = computed(() => player.showMiniPlayer);

const isImmersiveScene = computed(() => activePanel.value === 'lyric');
const isMobileLayout = computed(() => runtime.isMobile);
const sceneClass = computed(() => ui.theme === 'dark' ? 'scene-dark' : 'scene-light');
const supportsWindowControls = computed(() => runtime.supportsWindowControls);
const supportsDesktopLyricWindow = computed(() => runtime.supportsDesktopLyricWindow);
const isLyricFullscreen = computed(() => isImmersiveScene.value && isLyricWindowFullscreen.value);
const hideLyricChrome = computed(() => isLyricFullscreen.value && !showLyricChrome.value);
const miniPlayerTransitioning = ref(false);

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
  if (panel === 'favorites' || panel === 'local-library' || panel === 'history' || panel === 'settings') {
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

function handleMobileSearch() {
  activeNav.value = 'recommend';
  searchMode.value = 'recommend';
  setPanel('search');
  playlistDrawerOpen.value = false;
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

function buildDesktopLyricPayload(): DesktopLyricStatePayload {
  const currentIndex = player.currentLyricIndex;
  const currentLyricLine = currentIndex >= 0 ? player.lyricLines[currentIndex] ?? null : null;
  const currentLineTime = currentLyricLine?.time ?? null;
  const currentLineDuration = currentLyricLine
    ? resolveLyricLineDuration(player.lyricLines, currentIndex)
    : null;
  const currentLine = currentLyricLine?.text ?? '';
  const nextLine = player.lyricLines[currentIndex >= 0 ? currentIndex + 1 : 0]?.text ?? '';
  const translatedLine = currentIndex >= 0
    ? findTranslatedLine(player.lyricLines, player.tlyricLines, currentIndex)
    : '';
  const currentLineProgress = currentLyricLine
    ? resolveLyricLineProgress(player.lyricLines, currentIndex, player.currentTime)
    : null;
  const settings: DesktopLyricSettings = {
    ...ui.lyricSettings,
    windowPosition: ui.lyricSettings.windowPosition
      ? { ...ui.lyricSettings.windowPosition }
      : null,
  };

  return {
    trackTitle: player.currentTrack?.name ?? '',
    trackArtist: player.currentTrack?.artist ?? '',
    coverUrl: player.currentTrack?.coverUrl ?? null,
    playbackTime: player.currentTime,
    playbackUpdatedAt: player.playbackTimeUpdatedAt,
    currentLine,
    currentLineProgress,
    currentLineTime,
    currentLineDuration,
    translatedLine,
    nextLine,
    hasTrack: !!player.currentTrack,
    hasLyric: player.lyricLines.length > 0,
    isPlaying: player.isPlaying,
    settings,
  };
}

function clearDesktopLyricSyncFrame() {
  if (desktopLyricSyncFrame !== null) {
    window.clearTimeout(desktopLyricSyncFrame);
    desktopLyricSyncFrame = null;
  }
  desktopLyricLastSyncAt = 0;
  lastDesktopLyricPayload = null;
}

function shouldSyncDesktopLyricImmediately(payload: DesktopLyricStatePayload) {
  if (!lastDesktopLyricPayload) return true;

  return (
    lastDesktopLyricPayload.trackTitle !== payload.trackTitle ||
    lastDesktopLyricPayload.trackArtist !== payload.trackArtist ||
    lastDesktopLyricPayload.playbackTime > payload.playbackTime ||
    lastDesktopLyricPayload.currentLine !== payload.currentLine ||
    lastDesktopLyricPayload.currentLineTime !== payload.currentLineTime ||
    lastDesktopLyricPayload.currentLineDuration !== payload.currentLineDuration ||
    lastDesktopLyricPayload.translatedLine !== payload.translatedLine ||
    lastDesktopLyricPayload.nextLine !== payload.nextLine ||
    lastDesktopLyricPayload.hasTrack !== payload.hasTrack ||
    lastDesktopLyricPayload.hasLyric !== payload.hasLyric ||
    lastDesktopLyricPayload.isPlaying !== payload.isPlaying ||
    !sameDesktopLyricSettings(lastDesktopLyricPayload.settings, payload.settings)
  );
}

function scheduleDesktopLyricSync(payload: DesktopLyricStatePayload) {
  if (!supportsDesktopLyricWindow.value) return;
  pendingDesktopLyricPayload = payload;

  if (!desktopLyricVisible.value) return;
  const immediate = shouldSyncDesktopLyricImmediately(payload);

  if (desktopLyricSyncFrame !== null) {
    if (!immediate) return;
    window.clearTimeout(desktopLyricSyncFrame);
    desktopLyricSyncFrame = null;
  }

  const delay = immediate
    ? 0
    : Math.max(0, DESKTOP_LYRIC_SYNC_INTERVAL - (performance.now() - desktopLyricLastSyncAt));

  desktopLyricSyncFrame = window.setTimeout(() => {
    desktopLyricSyncFrame = null;
    const nextPayload = pendingDesktopLyricPayload;
    pendingDesktopLyricPayload = null;
    if (!nextPayload || !desktopLyricVisible.value) return;

    desktopLyricLastSyncAt = performance.now();
    lastDesktopLyricPayload = {
      ...nextPayload,
      settings: {
        ...nextPayload.settings,
        windowPosition: nextPayload.settings.windowPosition
          ? { ...nextPayload.settings.windowPosition }
          : null,
      },
    };
    void emitTo(DESKTOP_LYRIC_WINDOW_LABEL, DESKTOP_LYRIC_STATE_EVENT, nextPayload).catch((error) => {
      console.error('desktop lyric sync failed', error);
    });
  }, delay);
}

async function syncDesktopLyricWindowState() {
  if (!supportsDesktopLyricWindow.value) {
    desktopLyricVisible.value = false;
    desktopLyricPending.value = false;
    return;
  }

  try {
    const lyricWindow = await WebviewWindow.getByLabel(DESKTOP_LYRIC_WINDOW_LABEL);
    desktopLyricVisible.value = lyricWindow ? await lyricWindow.isVisible() : false;
  } catch (error) {
    desktopLyricVisible.value = false;
    console.error('syncDesktopLyricWindowState failed', error);
  } finally {
    desktopLyricPending.value = false;
  }
}

async function applyDesktopLyricWindowSettings(
  lyricWindow: WebviewWindow,
  settings: DesktopLyricSettings,
) {
  if (!supportsDesktopLyricWindow.value) return;

  await Promise.allSettled([
    lyricWindow.setAlwaysOnTop(settings.alwaysOnTop),
    lyricWindow.setIgnoreCursorEvents(settings.locked),
    lyricWindow.setFocusable(!settings.locked),
  ]);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

async function restoreDesktopLyricWindowPosition(lyricWindow: WebviewWindow) {
  if (!supportsDesktopLyricWindow.value) return;

  const storedPosition = ui.lyricSettings.windowPosition;
  if (!storedPosition) {
    await lyricWindow.center();
    return;
  }

  const monitors = await availableMonitors();
  if (!monitors.length) {
    await lyricWindow.setPosition(new PhysicalPosition(storedPosition.x, storedPosition.y));
    return;
  }

  const targetMonitor = monitors.find((monitor) => {
    const workArea = monitor.workArea;
    return (
      storedPosition.x >= workArea.position.x &&
      storedPosition.x < workArea.position.x + workArea.size.width &&
      storedPosition.y >= workArea.position.y &&
      storedPosition.y < workArea.position.y + workArea.size.height
    );
  });

  if (!targetMonitor) {
    ui.setDesktopLyricWindowPosition(null);
    await lyricWindow.center();
    return;
  }

  const workArea = targetMonitor.workArea;
  const maxX = workArea.position.x + Math.max(
    0,
    workArea.size.width - Math.round(DESKTOP_LYRIC_WINDOW_WIDTH * targetMonitor.scaleFactor),
  );
  const maxY = workArea.position.y + Math.max(
    0,
    workArea.size.height - Math.round(DESKTOP_LYRIC_WINDOW_HEIGHT * targetMonitor.scaleFactor),
  );
  const nextPosition = {
    x: clamp(storedPosition.x, workArea.position.x, maxX),
    y: clamp(storedPosition.y, workArea.position.y, maxY),
  };

  await lyricWindow.setPosition(new PhysicalPosition(nextPosition.x, nextPosition.y));

  if (
    nextPosition.x !== storedPosition.x
    || nextPosition.y !== storedPosition.y
  ) {
    ui.setDesktopLyricWindowPosition(nextPosition);
  }
}

async function openDesktopLyricWindow() {
  if (!supportsDesktopLyricWindow.value) return;
  if (desktopLyricPending.value) return;

  desktopLyricPending.value = true;
  ui.refreshLyricSettings();

  try {
    const existing = await WebviewWindow.getByLabel(DESKTOP_LYRIC_WINDOW_LABEL);
    if (existing) {
      await applyDesktopLyricWindowSettings(existing, ui.lyricSettings);
      await existing.show();
      desktopLyricVisible.value = true;
      desktopLyricPending.value = false;
      scheduleDesktopLyricSync(buildDesktopLyricPayload());
      return;
    }

    const lyricWindow = new WebviewWindow(DESKTOP_LYRIC_WINDOW_LABEL, {
      url: `/?${DESKTOP_LYRIC_WINDOW_QUERY}=1`,
      title: 'Desktop Lyric',
      width: DESKTOP_LYRIC_WINDOW_WIDTH,
      height: DESKTOP_LYRIC_WINDOW_HEIGHT,
      center: true,
      transparent: true,
      visible: false,
      decorations: false,
      alwaysOnTop: ui.lyricSettings.alwaysOnTop,
      focus: !ui.lyricSettings.locked,
      focusable: !ui.lyricSettings.locked,
      skipTaskbar: true,
      shadow: false,
      resizable: false,
      maximizable: false,
      minimizable: false,
    });

    lyricWindow.once('tauri://created', async () => {
      desktopLyricVisible.value = true;
      desktopLyricPending.value = false;
      await applyDesktopLyricWindowSettings(lyricWindow, ui.lyricSettings);
      await restoreDesktopLyricWindowPosition(lyricWindow);
      await lyricWindow.show();
      scheduleDesktopLyricSync(buildDesktopLyricPayload());
    });

    lyricWindow.once('tauri://error', (error) => {
      desktopLyricVisible.value = false;
      desktopLyricPending.value = false;
      console.error('desktop lyric create failed', error);
    });
  } catch (error) {
    desktopLyricVisible.value = false;
    desktopLyricPending.value = false;
    console.error('openDesktopLyricWindow failed', error);
  }
}

async function closeDesktopLyricWindow() {
  if (!supportsDesktopLyricWindow.value) {
    desktopLyricVisible.value = false;
    desktopLyricPending.value = false;
    return;
  }
  if (desktopLyricPending.value) return;

  desktopLyricPending.value = true;
  const existing = await WebviewWindow.getByLabel(DESKTOP_LYRIC_WINDOW_LABEL);
  clearDesktopLyricSyncFrame();
  pendingDesktopLyricPayload = null;

  if (!existing) {
    desktopLyricVisible.value = false;
    desktopLyricPending.value = false;
    return;
  }

  await existing.destroy().catch((error) => {
    console.error('closeDesktopLyricWindow failed', error);
  });
  desktopLyricVisible.value = false;
  desktopLyricPending.value = false;
}

async function toggleDesktopLyricWindow() {
  if (!supportsDesktopLyricWindow.value) return;
  if (desktopLyricPending.value) return;

  const existing = await WebviewWindow.getByLabel(DESKTOP_LYRIC_WINDOW_LABEL);
  if (existing && await existing.isVisible()) {
    await closeDesktopLyricWindow();
    return;
  }

  await openDesktopLyricWindow();
}

async function pushMiniPlayerState() {
  if (!supportsWindowControls.value) return;
  const snapshot: MiniPlayerStateSnapshot = player.getMiniPlayerStateSnapshot();
  console.log('[mini-sync][main] push state', {
    showMiniPlayer: player.showMiniPlayer,
    hasTrack: !!snapshot.currentTrack,
    trackName: snapshot.currentTrack?.name ?? null,
    isPlaying: snapshot.isPlaying,
    duration: snapshot.duration,
    currentTime: snapshot.currentTime,
    playMode: snapshot.playMode,
  });
  try {
    await invoke('emit_app_event', {
      event: MINI_PLAYER_STATE_EVENT,
      payload: JSON.stringify(snapshot),
    });
  } catch (error) {
    console.error('[mini-sync][main] emit state failed', error);
  }
}

async function openMiniPlayerWindow() {
  if (!supportsWindowControls.value || miniPlayerTransitioning.value || player.showMiniPlayer) return;
  miniPlayerTransitioning.value = true;
  try {
    let miniWindow = await WebviewWindow.getByLabel(MINI_PLAYER_WINDOW_LABEL);
    if (!miniWindow) {
      const createdWindow = new WebviewWindow(MINI_PLAYER_WINDOW_LABEL, {
        url: `index.html?${MINI_PLAYER_WINDOW_QUERY}`,
        title: 'Fashion Mini Player',
        width: 360,
        height: 140,
        minWidth: 320,
        minHeight: 124,
        maxWidth: 520,
        maxHeight: 180,
        decorations: false,
        transparent: true,
        shadow: false,
        resizable: true,
        alwaysOnTop: true,
        skipTaskbar: true,
        visible: false,
      });
      await new Promise<void>((resolve) => {
        createdWindow.once('tauri://created', () => resolve());
        createdWindow.once('tauri://error', () => resolve());
      });
      miniWindow = createdWindow;
    }

    player.setMiniPlayerVisible(true);
    await miniWindow.show();
    await pushMiniPlayerState();
    await miniWindow.setFocus().catch(() => undefined);
    await new Promise((resolve) => window.setTimeout(resolve, 120));
    await getCurrentWindow().hide();
  } catch (error) {
    player.setMiniPlayerVisible(false);
    console.error('open mini player window failed', error);
  } finally {
    miniPlayerTransitioning.value = false;
  }
}

async function closeMiniPlayerWindow({ restoreMainWindow = true }: { restoreMainWindow?: boolean } = {}) {
  if (!supportsWindowControls.value || miniPlayerTransitioning.value) return;
  miniPlayerTransitioning.value = true;
  try {
    const miniWindow = await WebviewWindow.getByLabel(MINI_PLAYER_WINDOW_LABEL);
    if (miniWindow) {
      await miniWindow.hide();
    }
    player.setMiniPlayerVisible(false);
    if (restoreMainWindow) {
      const mainWindow = getCurrentWindow();
      await mainWindow.show();
      await new Promise((resolve) => window.setTimeout(resolve, 120));
      await mainWindow.setFocus().catch(() => undefined);
    }
  } catch (error) {
    console.error('close mini player window failed', error);
  } finally {
    miniPlayerTransitioning.value = false;
  }
}

async function toggleMiniPlayerWindow() {
  if (!supportsWindowControls.value || miniPlayerTransitioning.value) return;
  if (player.showMiniPlayer) {
    await closeMiniPlayerWindow();
    return;
  }
  await openMiniPlayerWindow();
}

let cleanupDomResize: null | (() => void) = null;
let lyricChromeTimer: number | null = null;
let cleanupDesktopLyricEvents: null | (() => void) = null;
let cleanupTrayEvents: null | (() => void) = null;
let cleanupMiniPlayerEvents: null | (() => void) = null;
let desktopLyricSyncFrame: number | null = null;
let pendingDesktopLyricPayload: DesktopLyricStatePayload | null = null;
let desktopLyricLastSyncAt = 0;
let lastDesktopLyricPayload: DesktopLyricStatePayload | null = null;
let syncSeq = 0;
let fullscreenPending = false;
let toggleSeq = 0;
const DESKTOP_LYRIC_SYNC_INTERVAL = 40;

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
  if (!supportsWindowControls.value) {
    isWindowFill.value = false;
    isLyricWindowFullscreen.value = false;
    return;
  }

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
  if (!supportsWindowControls.value) {
    isWindowFill.value = false;
    isLyricWindowFullscreen.value = false;
    return;
  }

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
  await syncDesktopLyricWindowState();

  if (player.showMiniPlayer) {
    await openMiniPlayerWindow();
  }

  const onResize = () => {
    if (!supportsWindowControls.value) return;
    void syncWindowFillState();
  };
  const onActivity = () => {
    if (!supportsWindowControls.value || !isLyricFullscreen.value) return;
    showChrome();
  };

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

  const unlistenTrayShow = await listen(TRAY_SHOW_MAIN_EVENT, async () => {
    await closeMiniPlayerWindow({ restoreMainWindow: true });
    await invoke('window_show');
  });
  const unlistenTrayTogglePlay = await listen(TRAY_TOGGLE_PLAY_EVENT, () => {
    void player.togglePlay();
  });
  const unlistenTrayPrev = await listen(TRAY_PLAY_PREV_EVENT, () => {
    player.playPrev();
  });
  const unlistenTrayNext = await listen(TRAY_PLAY_NEXT_EVENT, () => {
    player.playNext();
  });
  const unlistenTrayExit = await listen(TRAY_EXIT_EVENT, () => {
    player.setMiniPlayerVisible(false);
  });

  cleanupTrayEvents = () => {
    void unlistenTrayShow();
    void unlistenTrayTogglePlay();
    void unlistenTrayPrev();
    void unlistenTrayNext();
    void unlistenTrayExit();
  };

  if (!supportsDesktopLyricWindow.value) {
    cleanupDesktopLyricEvents = null;
  } else {
    const unlistenReady = await listen(DESKTOP_LYRIC_READY_EVENT, () => {
      desktopLyricVisible.value = true;
      desktopLyricPending.value = false;
      scheduleDesktopLyricSync(buildDesktopLyricPayload());
    });
    const unlistenClosed = await listen(DESKTOP_LYRIC_CLOSED_EVENT, () => {
      clearDesktopLyricSyncFrame();
      pendingDesktopLyricPayload = null;
      desktopLyricVisible.value = false;
      desktopLyricPending.value = false;
    });
    const unlistenAction = await listen<DesktopLyricActionPayload>(DESKTOP_LYRIC_ACTION_EVENT, (event) => {
      if (event.payload.type !== 'toggle-always-on-top') return;
      ui.setLyricSettings({ alwaysOnTop: !ui.lyricSettings.alwaysOnTop });
    });
    const unlistenMoved = await listen<DesktopLyricWindowPosition>(DESKTOP_LYRIC_MOVED_EVENT, (event) => {
      ui.setDesktopLyricWindowPosition(event.payload);
    });

    cleanupDesktopLyricEvents = () => {
      void unlistenReady();
      void unlistenClosed();
      void unlistenAction();
      void unlistenMoved();
    };
  }

  const unlistenMiniReady = await listen(MINI_PLAYER_READY_EVENT, async () => {
    console.log('[mini-sync][main] ready event received', {
      showMiniPlayer: player.showMiniPlayer,
      currentTrack: player.currentTrack?.name ?? null,
      isPlaying: player.isPlaying,
    });
    if (!player.showMiniPlayer) return;
    await pushMiniPlayerState();
  });
  const unlistenMiniClosed = await listen(MINI_PLAYER_CLOSED_EVENT, async () => {
    await closeMiniPlayerWindow({ restoreMainWindow: true });
  });
  const unlistenMiniHide = await listen(MINI_PLAYER_HIDE_EVENT, async () => {
    await closeMiniPlayerWindow({ restoreMainWindow: false });
  });
  const unlistenMiniTogglePlay = await listen(MINI_PLAYER_TOGGLE_PLAY_EVENT, () => {
    void player.togglePlay();
  });
  const unlistenMiniPrev = await listen(MINI_PLAYER_PLAY_PREV_EVENT, () => {
    player.playPrev();
  });
  const unlistenMiniNext = await listen(MINI_PLAYER_PLAY_NEXT_EVENT, () => {
    player.playNext();
  });
  const unlistenMiniToggleMode = await listen(MINI_PLAYER_TOGGLE_MODE_EVENT, () => {
    player.togglePlayMode();
  });
  const unlistenMiniToggleDesktopLyric = await listen(MINI_PLAYER_TOGGLE_DESKTOP_LYRIC_EVENT, () => {
    void toggleDesktopLyricWindow();
  });
  const unlistenMiniSeek = await listen<{ time?: number }>(MINI_PLAYER_SEEK_EVENT, (event) => {
    const nextTime = event.payload?.time;
    if (typeof nextTime !== 'number' || Number.isNaN(nextTime)) return;
    player.seek(nextTime);
  });

  cleanupMiniPlayerEvents = () => {
    void unlistenMiniReady();
    void unlistenMiniClosed();
    void unlistenMiniHide();
    void unlistenMiniTogglePlay();
    void unlistenMiniPrev();
    void unlistenMiniNext();
    void unlistenMiniToggleMode();
    void unlistenMiniToggleDesktopLyric();
    void unlistenMiniSeek();
  };
});

onBeforeUnmount(() => {
  cleanupDomResize?.();
  cleanupDomResize = null;
  cleanupDesktopLyricEvents?.();
  cleanupDesktopLyricEvents = null;
  cleanupTrayEvents?.();
  cleanupTrayEvents = null;
  cleanupMiniPlayerEvents?.();
  cleanupMiniPlayerEvents = null;
  clearLyricChromeTimer();
  clearDesktopLyricSyncFrame();
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

watch(
  () => ui.closeBehavior,
  (value) => {
    void invoke('emit_app_event', { event: 'app:close-behavior', payload: value });
  },
  { immediate: true },
);

watch(
  () => ui.lyricSettings,
  (value) => {
    if (!supportsDesktopLyricWindow.value) return;
    if (!desktopLyricVisible.value) return;

    void WebviewWindow.getByLabel(DESKTOP_LYRIC_WINDOW_LABEL)
      .then((lyricWindow) => {
        if (!lyricWindow) return;
        return applyDesktopLyricWindowSettings(lyricWindow, value);
      })
      .catch((error) => {
        console.error('apply desktop lyric settings failed', error);
      });
  },
  { deep: true },
);

watchEffect(() => {
  if (!supportsDesktopLyricWindow.value) return;
  if (!desktopLyricVisible.value) return;
  scheduleDesktopLyricSync(buildDesktopLyricPayload());
});

watch(
  () => [
    player.currentTrack,
    player.isPlaying,
    player.duration,
    player.currentTime,
    player.playMode,
  ],
  () => {
    if (!player.showMiniPlayer) return;
    void pushMiniPlayerState();
  },
  { deep: true },
);


function handleLyricFullscreenKeydown(event: KeyboardEvent) {
  if (!supportsWindowControls.value) return;
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

.app-shell.mobile-shell {
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

.app-shell.mobile-shell::after {
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

.app-layout.mobile {
  background: transparent;
  box-shadow: none;
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
@keyframes playerBarRise {
  from {
    opacity: 0;
    transform: translateY(18px) scale(0.985);
    filter: blur(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
    filter: blur(0);
  }
}

.player-bar-fade-enter-active {
  animation: playerBarRise 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.player-bar-fade-leave-active {
  transition: opacity 160ms ease, transform 160ms ease, filter 160ms ease;
}

.player-bar-fade-leave-to {
  opacity: 0;
  transform: translateY(14px) scale(0.988);
  filter: blur(6px);
}

.mobile-layout {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 12px 12px;
}

.mobile-layout.immersive {
  padding-top: 10px;
}

.mobile-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding-right: 2px;
}

.mobile-scroll.immersive {
  gap: 12px;
}

.mobile-content-shell {
  min-height: 420px;
  border-radius: 30px;
  overflow: hidden;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 90%, white 4%), rgba(255, 255, 255, 0.04));
  border: 1px solid color-mix(in srgb, var(--border) 80%, white 8%);
  box-shadow:
    0 24px 48px rgba(10, 20, 19, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.06);
  backdrop-filter: blur(24px);
}

.mobile-content-shell.immersive {
  min-height: 0;
  flex: 1;
  background: rgba(9, 18, 17, 0.72);
}

.mobile-bottom-stack {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-bottom: max(2px, env(safe-area-inset-bottom, 0px));
}

.playlist-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  justify-content: flex-end;
  background: linear-gradient(90deg, rgba(0, 0, 0, 0), var(--overlay-mask));
  z-index: 20;
}

.playlist-overlay.mobile {
  align-items: flex-end;
  justify-content: stretch;
  padding: 0 8px 8px;
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.02), rgba(7, 13, 12, 0.58));
}

.playlist-overlay.mobile :deep(.playlist-drawer) {
  width: 100%;
  height: min(72vh, 680px);
  margin: 0;
  border-radius: 28px 28px 22px 22px;
}

.mobile-content-shell :deep(.search-panel),
.mobile-content-shell :deep(.favorites-panel),
.mobile-content-shell :deep(.history-panel),
.mobile-content-shell :deep(.settings-panel),
.mobile-content-shell :deep(.lyric-panel) {
  height: 100%;
}

.mobile-content-shell :deep(.search-panel .empty-view) {
  padding: 18px 16px 14px;
  gap: 18px;
}

.mobile-content-shell :deep(.search-panel .empty-hero) {
  padding: 20px;
  box-shadow: none;
}

.mobile-content-shell :deep(.search-panel .empty-copy h2) {
  font-size: 24px;
}

.mobile-content-shell :deep(.search-panel .history-card),
.mobile-content-shell :deep(.favorites-topbar),
.mobile-content-shell :deep(.history-topbar),
.mobile-content-shell :deep(.settings-card) {
  border-radius: 22px;
}

.mobile-content-shell :deep(.result-header) {
  padding: 18px 16px 10px;
}

.mobile-content-shell :deep(.panel-body) {
  padding: 0 8px 14px;
}

.mobile-content-shell :deep(.settings-panel) {
  padding: 14px;
}

.mobile-content-shell :deep(.lyric-panel) {
  padding: 18px 14px 14px;
}

.mobile-content-shell.immersive :deep(.lyric-panel) {
  padding: 10px 10px 8px;
}

.mobile-content-shell :deep(.lyric-stage) {
  max-width: none;
}

.drawer-fade-enter-active,
.drawer-fade-leave-active {
  transition: opacity 0.18s ease;
}

.drawer-fade-enter-from,
.drawer-fade-leave-to {
  opacity: 0;
}

@media (max-width: 640px) {
  .mobile-layout {
    padding-left: 10px;
    padding-right: 10px;
  }

  .mobile-content-shell {
    border-radius: 26px;
  }

  .playlist-overlay.mobile {
    padding-left: 4px;
    padding-right: 4px;
    padding-bottom: 4px;
  }

  .playlist-overlay.mobile :deep(.playlist-drawer) {
    height: min(76vh, 720px);
    border-radius: 26px 26px 18px 18px;
  }
}
</style>

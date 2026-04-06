<template>
  <header
    ref="rootEl"
    class="titlebar"
    :class="[scene, { 'window-fill': windowFill, hidden }]"
    @mousedown.left="startDragging"
    @dblclick="handleTitlebarDoubleClick"
  >
    <template v-if="scene === 'light'">
      <div class="toolbar-left">
        <div class="nav-cluster" data-no-drag @mousedown.stop>
          <button type="button" class="nav-btn" :disabled="!canGoBack" @click="$emit('back')" title="返回">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="m15 18-6-6 6-6" />
            </svg>
          </button>
          <button type="button" class="nav-btn" :disabled="!canGoForward" @click="$emit('forward')" title="前进">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="m9 18 6-6-6-6" />
            </svg>
          </button>
          <button type="button" class="nav-btn" @click="submitSearch" title="刷新当前搜索">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 1 1-3.2-6.9" />
              <path d="M21 3v6h-6" />
            </svg>
          </button>
        </div>

        <div ref="searchWrap" class="search-box" data-no-drag @mousedown.stop>
          <div class="source-picker">
            <button
              type="button"
              class="source-trigger"
              :class="{ open: sourceMenuOpen }"
              @click.stop="toggleSourceMenu"
            >
              <span>{{ selectedSourceLabel }}</span>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="m6 9 6 6 6-6" />
              </svg>
            </button>

            <Transition name="search-pop">
              <div v-if="sourceMenuOpen" class="source-menu">
                <button
                  v-for="item in enabledSources"
                  :key="item.value"
                  type="button"
                  class="source-item"
                  :class="{ active: selectedSource === item.value }"
                  @click.stop="selectSource(item.value)"
                >
                  <span>{{ item.label }}</span>
                  <svg
                    v-if="selectedSource === item.value"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <path d="m5 12 5 5L20 7" />
                  </svg>
                </button>
              </div>
            </Transition>
          </div>

          <input
            v-model="searchText"
            class="search-input"
            placeholder="搜索音乐、歌手或专辑"
            @focus="openSearchMenu"
            @click.stop="openSearchMenu"
            @keyup.enter="submitSearch"
          />

          <button type="button" class="search-submit" title="搜索" @click="submitSearch">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.35-4.35" />
            </svg>
          </button>

          <Transition name="search-pop">
            <div v-if="searchMenuOpen && recentSearches.length" class="search-menu">
              <div class="search-menu-head">最近搜索</div>
              <button
                v-for="item in recentSearches"
                :key="item"
                type="button"
                class="search-menu-item"
                @click.stop="applyRecent(item)"
              >
                <span>{{ item }}</span>
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 8v5l3 2" />
                  <circle cx="12" cy="12" r="9" />
                </svg>
              </button>
            </div>
          </Transition>
        </div>
      </div>

      <div class="titlebar-drag-slot" aria-hidden="true" />

      <div class="toolbar-right" data-no-drag @mousedown.stop>
        <button
          v-if="props.showMiniPlayerToggle"
          type="button"
          class="nav-btn"
          :class="{ active: player.showMiniPlayer }"
          @click="$emit('toggle-mini-player')"
          :title="miniPlayerToggleTitle"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="4" y="5" width="16" height="12" rx="3" />
            <path d="M9 19h6" />
            <path d="M10 9h4" />
            <path d="M12 13h0.01" />
          </svg>
        </button>
        <button type="button" class="nav-btn" @click="$emit('open-settings')" title="设置">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06A1.65 1.65 0 0 0 15 19.4a1.65 1.65 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82L4.21 7.1a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.09A1.65 1.65 0 0 0 15 4.6a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.5 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
        <template v-if="props.showWindowControls">
          <button type="button" class="window-btn" @click="minimize">
            <svg width="10" height="10" viewBox="0 0 11 11">
              <rect x="0.5" y="5" width="10" height="1.5" rx="0.75" fill="currentColor" />
            </svg>
          </button>
          <button type="button" class="window-btn" @click="maximize">
            <svg width="10" height="10" viewBox="0 0 11 11">
              <rect x="1" y="1" width="9" height="9" rx="1.5" stroke="currentColor" stroke-width="1.5" fill="none" />
            </svg>
          </button>
          <button type="button" class="window-btn close" @click="close">
            <svg width="10" height="10" viewBox="0 0 11 11">
              <line x1="1" y1="1" x2="10" y2="10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              <line x1="10" y1="1" x2="1" y2="10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </template>
      </div>
    </template>

    <template v-else>
      <div class="immersive-left" data-no-drag @mousedown.stop>
        <button type="button" class="immersive-back" @click="$emit('back')" :disabled="!canGoBack">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m6 9 6 6 6-6" />
          </svg>
        </button>
      </div>
      <div class="titlebar-drag-slot" aria-hidden="true" />
      <div class="immersive-right" data-no-drag @mousedown.stop>
        <button v-if="props.showLyricWindowToggle" type="button" class="window-btn" @click="$emit('toggle-lyric-fullscreen')">
          <svg
            v-if="!lyricFullscreen"
            width="10"
            height="10"
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
            width="10"
            height="10"
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
        </button>
        <template v-if="props.showWindowControls">
          <button type="button" class="window-btn" @click="minimize">
            <svg width="10" height="10" viewBox="0 0 11 11">
              <rect x="0.5" y="5" width="10" height="1.5" rx="0.75" fill="currentColor" />
            </svg>
          </button>
          <button type="button" class="window-btn" @click="maximize">
            <svg width="10" height="10" viewBox="0 0 11 11">
              <rect x="1" y="1" width="9" height="9" rx="1.5" stroke="currentColor" stroke-width="1.5" fill="none" />
            </svg>
          </button>
          <button type="button" class="window-btn close" @click="close">
            <svg width="10" height="10" viewBox="0 0 11 11">
              <line x1="1" y1="1" x2="10" y2="10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              <line x1="10" y1="1" x2="1" y2="10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </template>
      </div>
    </template>
  </header>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { SOURCES, type MusicSource } from '@/api/music';
import { usePlayerStore } from '@/stores/player';
import { useUiStore } from '@/stores/ui';
import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';

const RECENT_SEARCHES_KEY = 'fashion:recent-searches';
const RECENT_SEARCHES_VERSION = 1;
const MAX_RECENT_SEARCHES = 10;

const props = withDefaults(defineProps<{
  activePanel: string;
  canGoBack: boolean;
  canGoForward: boolean;
  scene: 'light' | 'dark';
  windowFill?: boolean;
  lyricFullscreen?: boolean;
  hidden?: boolean;
  showWindowControls?: boolean;
  allowWindowDragging?: boolean;
  showLyricWindowToggle?: boolean;
  showMiniPlayerToggle?: boolean;
}>(), {
  windowFill: false,
  lyricFullscreen: false,
  hidden: false,
  showWindowControls: true,
  allowWindowDragging: true,
  showLyricWindowToggle: true,
  showMiniPlayerToggle: true,
});

const emit = defineEmits<{
  back: [];
  forward: [];
  search: [value: string];
  'open-settings': [];
  'toggle-lyric-fullscreen': [];
  'toggle-mini-player': [];
}>();

const player = usePlayerStore();
const ui = useUiStore();
const rootEl = ref<HTMLElement | null>(null);
const searchText = ref(ui.toolbarSearch);
const selectedSource = ref<MusicSource>(ui.toolbarSource);
const searchMenuOpen = ref(false);
const sourceMenuOpen = ref(false);
const searchWrap = ref<HTMLElement | null>(null);
const recentSearches = ref<string[]>(loadRecentSearches());
const enabledSources = computed(() =>
  SOURCES.filter((item) => ui.enabledToolbarSources.includes(item.value)),
);

const selectedSourceLabel = computed(() =>
  SOURCES.find((item) => item.value === selectedSource.value)?.label ?? selectedSource.value,
);
const miniPlayerToggleTitle = computed(() => player.showMiniPlayer ? '关闭迷你播放器' : '打开迷你播放器');

function loadRecentSearches() {
  return readVersionedStorage<string[]>(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, {
    fallback: [],
    validate: (value): value is string[] => Array.isArray(value) && value.every((item) => typeof item === 'string'),
  }).slice(0, MAX_RECENT_SEARCHES);
}

function syncRecentSearches() {
  recentSearches.value = loadRecentSearches();
}

function rememberRecentSearch(value: string) {
  const normalized = value.trim();
  if (!normalized) return;

  recentSearches.value = [normalized, ...recentSearches.value.filter((item) => item !== normalized)].slice(0, MAX_RECENT_SEARCHES);
  writeVersionedStorage(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, recentSearches.value);
}

function openSearchMenu() {
  sourceMenuOpen.value = false;
  syncRecentSearches();
  searchMenuOpen.value = recentSearches.value.length > 0;
}

function closeSearchMenu() {
  searchMenuOpen.value = false;
}

function toggleSourceMenu() {
  if (enabledSources.value.length <= 1) return;
  closeSearchMenu();
  sourceMenuOpen.value = !sourceMenuOpen.value;
}

function closeSourceMenu() {
  sourceMenuOpen.value = false;
}

function submitSearch() {
  rememberRecentSearch(searchText.value);
  ui.submitToolbarSearch(searchText.value, selectedSource.value);
  emit('search', searchText.value);
  closeSearchMenu();
  closeSourceMenu();
}

function selectSource(value: MusicSource) {
  selectedSource.value = value;
  ui.setToolbarSource(value);
  closeSourceMenu();

  if (searchText.value.trim()) {
    submitSearch();
  }
}

function applyRecent(value: string) {
  searchText.value = value;
  submitSearch();
}

function handlePointerDown(event: PointerEvent) {
  const target = event.target as Node | null;
  if (searchWrap.value?.contains(target)) return;
  closeSearchMenu();
  closeSourceMenu();
}

async function minimize() {
  if (!props.showWindowControls) return;
  try {
    await invoke('window_minimize');
  } catch (error) {
    console.error('window_minimize failed', error);
  }
}

async function maximize() {
  if (!props.showWindowControls) return;
  try {
    await invoke('window_maximize');
  } catch (error) {
    console.error('window_maximize failed', error);
  }
}

async function close() {
  if (!props.showWindowControls) return;
  try {
    await invoke('window_close');
  } catch (error) {
    console.error('window_close failed', error);
  }
}

async function startDragging(event: MouseEvent) {
  if (!props.allowWindowDragging) return;
  if (event.button !== 0) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest('[data-no-drag]')) return;
  try {
    await invoke('window_start_dragging');
  } catch (error) {
    console.error('window_start_dragging failed', error);
  }
}

async function handleTitlebarDoubleClick(event: MouseEvent) {
  if (!props.allowWindowDragging || !props.showWindowControls) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest('[data-no-drag]')) return;
  await maximize();
}

watch(
  () => ui.toolbarSearch,
  (value) => {
    searchText.value = value;
  },
);

watch(
  () => ui.toolbarSource,
  (value) => {
    selectedSource.value = value;
  },
);

watch(
  () => ui.enabledToolbarSources,
  () => {
    if (!ui.isSourceEnabled(selectedSource.value)) {
      selectedSource.value = ui.toolbarSource;
    }
  },
  { deep: true },
);

onMounted(() => {
  rootEl.value?.querySelectorAll<HTMLElement>('.window-btn[title]').forEach((button) => {
    button.removeAttribute('title');
  });
  window.addEventListener('pointerdown', handlePointerDown);
});

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', handlePointerDown);
});
</script>

<style scoped>
.titlebar {
  height: var(--titlebar-h);
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  transition: height 0.22s ease, padding 0.22s ease, opacity 0.22s ease, transform 0.22s ease;
}

.titlebar.hidden {
  height: 0;
  padding-top: 0;
  padding-bottom: 0;
  opacity: 0;
  pointer-events: none;
  overflow: hidden;
  transform: translateY(-18px);
}

.titlebar-drag-slot {
  flex: 1;
  min-width: 24px;
  align-self: stretch;
  cursor: grab;
}

.titlebar-drag-slot:active {
  cursor: grabbing;
}

.titlebar.light {
  padding: 10px 14px 6px 8px;
}

.toolbar-left,
.toolbar-right,
.nav-cluster {
  display: flex;
  align-items: center;
}

.toolbar-left {
  gap: 12px;
  min-width: 0;
}

.toolbar-right {
  gap: 8px;
  flex-shrink: 0;
}

.nav-cluster {
  gap: 4px;
}

.nav-btn,
.window-btn,
.immersive-back,
.search-submit {
  width: 26px;
  height: 26px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
}

.nav-btn,
.window-btn {
  color: var(--text-secondary);
}

.nav-btn:hover:not(:disabled),
.nav-btn.active,
.window-btn:hover,
.search-submit:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-btn:disabled,
.immersive-back:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.search-box {
  position: relative;
  width: 360px;
  height: 40px;
  padding: 0 8px 0 0;
  border-radius: 999px;
  display: flex;
  align-items: center;
  gap: 0;
  background: var(--panel-strong);
  border: 1px solid var(--border);
  box-shadow: 0 10px 24px rgba(77, 102, 96, 0.08);
}

.source-picker {
  position: relative;
  flex-shrink: 0;
  align-self: stretch;
  display: flex;
}

.source-trigger {
  height: 100%;
  padding: 0 14px 0 16px;
  border-radius: 999px 0 0 999px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: linear-gradient(135deg, rgba(22, 214, 160, 0.12), rgba(255, 255, 255, 0.02));
  border: none;
  border-right: 1px solid var(--border);
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 700;
  transition: var(--transition);
}

.source-trigger.open,
.source-trigger:hover {
  background: linear-gradient(135deg, var(--accent-dim), rgba(255, 255, 255, 0.04));
}

.source-menu {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  width: 154px;
  padding: 8px;
  border-radius: 18px;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  box-shadow: var(--window-shadow);
  z-index: 40;
}

.source-item {
  width: 100%;
  height: 40px;
  padding: 0 12px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--text-secondary);
  text-align: left;
  transition: var(--transition);
}

.source-item:hover,
.source-item.active {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.source-item.active {
  font-weight: 700;
}

.search-input {
  flex: 1;
  min-width: 0;
  padding: 0 12px;
  font-size: 13px;
  color: var(--text-primary);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-submit {
  flex-shrink: 0;
  color: var(--text-muted);
}

.search-menu {
  position: absolute;
  left: 0;
  right: 0;
  top: calc(100% + 8px);
  padding: 8px;
  border-radius: 16px;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  box-shadow: var(--window-shadow);
  z-index: 30;
}

.search-menu-head {
  padding: 0 6px 6px;
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.search-menu-item {
  width: 100%;
  min-height: 32px;
  padding: 0 10px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  color: var(--text-secondary);
  font-size: 12px;
  transition: var(--transition);
}

.search-menu-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.search-pop-enter-active,
.search-pop-leave-active {
  transition: opacity 0.14s ease, transform 0.14s ease;
}

.search-pop-enter-from,
.search-pop-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.window-btn.close:hover {
  background: rgba(248, 113, 113, 0.18);
  color: var(--text-danger);
}

.titlebar.dark {
  height: 44px;
  padding: 8px 14px 0;
  background: transparent;
}

.immersive-left,
.immersive-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.immersive-back,
.dark .window-btn {
  width: 24px;
  height: 24px;
  color: rgba(255, 255, 255, 0.72);
}

.immersive-back:hover:not(:disabled),
.dark .window-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

@media (max-width: 980px) {
  .search-box {
    width: 288px;
  }
}

@media (max-width: 720px) {
  .toolbar-left {
    gap: 8px;
  }

  .search-box {
    width: 232px;
  }

  .source-trigger {
    padding: 0 10px;
  }

  .source-trigger span {
    max-width: 56px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}
</style>

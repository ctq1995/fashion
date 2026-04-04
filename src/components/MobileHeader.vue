<template>
  <header class="mobile-header" :class="{ immersive }">
    <section class="hero-card">
      <div class="hero-top">
        <div class="brand-pill">
          <div class="brand-mark">
            <img :src="brandIcon" alt="Fashion" />
          </div>
          <div class="brand-copy">
            <span class="brand-tag">Fashion</span>
            <strong>Music</strong>
          </div>
        </div>

        <div class="hero-actions">
          <button
            type="button"
            class="hero-action"
            :disabled="!player.currentTrack"
            title="歌词"
            @click="emit('open-lyric')"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <path d="M4 7h16" />
              <path d="M4 12h10" />
              <path d="M4 17h16" />
            </svg>
          </button>

          <button type="button" class="hero-action queue-action" title="播放列表" @click="emit('open-playlist')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <path d="M3 7h12" />
              <path d="M3 12h12" />
              <path d="M3 17h8" />
              <path d="M19 8v10" />
              <path d="M14 13h10" />
            </svg>
            <span v-if="player.queue.length" class="queue-badge">{{ Math.min(player.queue.length, 99) }}</span>
          </button>

          <button type="button" class="hero-action" title="设置" @click="emit('open-settings')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06A1.65 1.65 0 0 0 15 19.4a1.65 1.65 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82L4.21 7.1a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.09A1.65 1.65 0 0 0 15 4.6a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.5 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
          </button>
        </div>
      </div>

      <div class="hero-body">
        <div class="hero-cover">
          <img v-if="player.currentTrack?.coverUrl" :src="player.currentTrack.coverUrl" alt="" />
          <div v-else class="hero-cover-fallback">
            <img :src="brandIcon" alt="Fashion" />
          </div>
        </div>

        <div class="hero-copy-main">
          <span class="hero-kicker">{{ heroKicker }}</span>
          <h1>{{ heroTitle }}</h1>
          <p>{{ heroSubtitle }}</p>
        </div>
      </div>
    </section>

    <section v-if="!immersive" ref="searchWrap" class="search-card">
      <div class="search-field">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <circle cx="11" cy="11" r="7" />
          <path d="m21 21-4.35-4.35" />
        </svg>
        <input
          v-model="searchText"
          type="text"
          class="search-input"
          placeholder="搜索歌曲、歌手、专辑"
          @focus="openSearchMenu"
          @click="openSearchMenu"
          @keyup.enter="submitSearch"
        />
        <button type="button" class="search-submit" @click="submitSearch">搜索</button>
      </div>

      <div class="source-strip app-scroll">
        <button
          v-for="item in enabledSources"
          :key="item.value"
          type="button"
          class="source-pill"
          :class="{ active: selectedSource === item.value }"
          @click="selectSource(item.value)"
        >
          {{ item.label }}
        </button>
      </div>

      <Transition name="mobile-search-pop">
        <div v-if="searchMenuOpen && recentSearches.length" class="recent-sheet">
          <div class="recent-head">
            <span>最近搜索</span>
            <button type="button" class="clear-btn" @click="clearRecentSearches">清空</button>
          </div>
          <div class="recent-list">
            <button
              v-for="item in recentSearches"
              :key="item"
              type="button"
              class="recent-item"
              @click="applyRecent(item)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <circle cx="12" cy="12" r="9" />
                <path d="M12 7v5l3 2" />
              </svg>
              <span>{{ item }}</span>
            </button>
          </div>
        </div>
      </Transition>
    </section>
  </header>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { SOURCES, type MusicSource } from '@/api/music';
import brandIcon from '@/assets/fashion-brand.svg';
import { usePlayerStore } from '@/stores/player';
import { useUiStore } from '@/stores/ui';
import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';

const RECENT_SEARCHES_KEY = 'fashion:recent-searches';
const RECENT_SEARCHES_VERSION = 1;
const MAX_RECENT_SEARCHES = 8;

const props = withDefaults(defineProps<{
  activePanel: string;
  immersive?: boolean;
}>(), {
  immersive: false,
});

const emit = defineEmits<{
  search: [value: string];
  'open-settings': [];
  'open-playlist': [];
  'open-lyric': [];
}>();

const player = usePlayerStore();
const ui = useUiStore();

const searchWrap = ref<HTMLElement | null>(null);
const searchText = ref(ui.toolbarSearch);
const selectedSource = ref<MusicSource>(ui.toolbarSource);
const searchMenuOpen = ref(false);
const recentSearches = ref(loadRecentSearches());

const enabledSources = computed(() =>
  SOURCES.filter((item) => ui.enabledToolbarSources.includes(item.value)),
);

const heroKicker = computed(() => {
  if (player.loading) return '缓冲中';
  if (player.currentTrack && player.isPlaying) return '正在播放';
  if (player.currentTrack) return '准备继续';

  return ({
    search: '发现音乐',
    favorites: '你的收藏',
    history: '最近播放',
    lyric: '歌词视图',
    settings: '应用设置',
  } as Record<string, string>)[props.activePanel] ?? 'Fashion';
});

const heroTitle = computed(() => {
  if (player.currentTrack?.name) return player.currentTrack.name;

  return ({
    search: '把搜索和播放放在更近的位置',
    favorites: '收藏列表重新排版',
    history: '快速回到最近听过的歌',
    lyric: '把歌词作为移动端主视图',
    settings: '音源和显示偏好都集中在这里',
  } as Record<string, string>)[props.activePanel] ?? 'Fashion Music';
});

const heroSubtitle = computed(() => {
  if (player.currentTrack) {
    const album = player.currentTrack.album?.trim();
    return album
      ? `${player.currentTrack.artist} · ${album}`
      : player.currentTrack.artist;
  }

  return ({
    search: '顶部搜索、音源切换和最近搜索放到同一层，减少来回跳转。',
    favorites: '收藏区改成更适合竖屏浏览的单列布局，播放入口靠前。',
    history: '最近播放保留进度和时间，方便继续收听。',
    lyric: '歌词页和播放控制更靠近，封面、进度和切歌不再分散。',
    settings: '保留原有设置项，同时适配手机上的触控尺寸。',
  } as Record<string, string>)[props.activePanel] ?? '移动端布局已重构。';
});

function loadRecentSearches() {
  return readVersionedStorage<string[]>(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, {
    fallback: [],
    validate: (value): value is string[] =>
      Array.isArray(value) && value.every((item) => typeof item === 'string'),
  }).slice(0, MAX_RECENT_SEARCHES);
}

function syncRecentSearches() {
  recentSearches.value = loadRecentSearches();
}

function persistRecentSearches() {
  writeVersionedStorage(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, recentSearches.value);
}

function rememberRecentSearch(value: string) {
  const normalized = value.trim();
  if (!normalized) return;

  recentSearches.value = [
    normalized,
    ...recentSearches.value.filter((item) => item !== normalized),
  ].slice(0, MAX_RECENT_SEARCHES);
  persistRecentSearches();
}

function openSearchMenu() {
  syncRecentSearches();
  searchMenuOpen.value = recentSearches.value.length > 0;
}

function closeSearchMenu() {
  searchMenuOpen.value = false;
}

function clearRecentSearches() {
  recentSearches.value = [];
  persistRecentSearches();
  closeSearchMenu();
}

function submitSearch() {
  const normalized = searchText.value.trim();
  if (normalized) {
    rememberRecentSearch(normalized);
  }
  ui.submitToolbarSearch(normalized, selectedSource.value);
  emit('search', normalized);
  closeSearchMenu();
}

function selectSource(value: MusicSource) {
  selectedSource.value = value;
  ui.setToolbarSource(value);

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
  window.addEventListener('pointerdown', handlePointerDown);
});

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', handlePointerDown);
});
</script>

<style scoped>
.mobile-header {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.mobile-header.immersive {
  gap: 10px;
}

.hero-card,
.search-card {
  position: relative;
  overflow: hidden;
  border-radius: 26px;
  border: 1px solid var(--border);
  background: linear-gradient(180deg, var(--panel-strong), rgba(255, 255, 255, 0.02));
  box-shadow: 0 20px 44px rgba(13, 25, 24, 0.14);
  backdrop-filter: blur(20px);
}

.hero-card {
  padding: 18px;
}

.hero-card::before {
  content: '';
  position: absolute;
  inset: -28% -14% auto auto;
  width: 220px;
  aspect-ratio: 1;
  background: radial-gradient(circle, var(--accent-dim) 0%, transparent 68%);
  pointer-events: none;
}

.hero-top,
.hero-body {
  display: flex;
  align-items: center;
  position: relative;
  z-index: 1;
}

.hero-top {
  justify-content: space-between;
  gap: 12px;
}

.brand-pill {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.brand-mark {
  width: 46px;
  height: 46px;
  border-radius: 14px;
  overflow: hidden;
  background: var(--panel-shell);
  border: 1px solid var(--border);
  box-shadow: 0 10px 22px rgba(22, 214, 160, 0.12);
}

.brand-mark img {
  width: 100%;
  height: 100%;
  display: block;
}

.brand-copy {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.brand-tag {
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.brand-copy strong {
  margin-top: 2px;
  font-size: 18px;
  line-height: 1.1;
  color: var(--text-primary);
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.hero-action {
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border: 1px solid transparent;
  transition: var(--transition);
}

.hero-action:hover:not(:disabled) {
  color: var(--text-primary);
  background: var(--bg-active);
  border-color: var(--border);
}

.hero-action:disabled {
  opacity: 0.36;
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

.hero-body {
  gap: 14px;
  margin-top: 16px;
  align-items: flex-start;
}

.hero-cover {
  width: 72px;
  height: 72px;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: 22px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  box-shadow: 0 12px 24px rgba(13, 25, 24, 0.16);
}

.hero-cover img,
.hero-cover-fallback img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: cover;
}

.hero-cover-fallback {
  width: 100%;
  height: 100%;
  padding: 10px;
  background: linear-gradient(180deg, var(--accent-dim), rgba(255, 255, 255, 0.02));
}

.hero-copy-main {
  min-width: 0;
}

.hero-kicker {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 0 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.08em;
  color: var(--accent);
  background: var(--accent-dim);
}

.hero-copy-main h1 {
  margin-top: 10px;
  font-size: 28px;
  line-height: 1.06;
  color: var(--text-primary);
}

.hero-copy-main p {
  margin-top: 8px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary);
}

.mobile-header.immersive .hero-card {
  padding: 16px;
  border-radius: 24px;
}

.mobile-header.immersive .hero-body {
  margin-top: 14px;
}

.mobile-header.immersive .hero-cover {
  width: 58px;
  height: 58px;
}

.mobile-header.immersive .hero-copy-main h1 {
  font-size: 22px;
}

.search-card {
  padding: 14px;
}

.search-field {
  min-height: 54px;
  padding: 0 8px 0 16px;
  border-radius: 20px;
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--panel-shell);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}

.search-input {
  flex: 1;
  min-width: 0;
  font-size: 15px;
  color: var(--text-primary);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-submit {
  height: 40px;
  padding: 0 14px;
  border-radius: 14px;
  background: linear-gradient(135deg, var(--accent-light), var(--accent));
  color: var(--text-on-accent);
  font-size: 13px;
  font-weight: 800;
  box-shadow: 0 14px 26px rgba(22, 214, 160, 0.16);
}

.source-strip {
  margin-top: 12px;
  padding-bottom: 2px;
  display: flex;
  gap: 8px;
  overflow-x: auto;
}

.source-pill {
  flex-shrink: 0;
  min-width: 0;
  height: 34px;
  padding: 0 14px;
  border-radius: 999px;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border: 1px solid transparent;
  font-size: 12px;
  font-weight: 700;
  transition: var(--transition);
}

.source-pill:hover {
  color: var(--text-primary);
  background: var(--bg-active);
}

.source-pill.active {
  color: var(--accent);
  background: var(--accent-dim);
  border-color: rgba(22, 214, 160, 0.22);
}

.recent-sheet {
  margin-top: 12px;
  padding: 12px;
  border-radius: 18px;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  box-shadow: 0 16px 34px rgba(13, 25, 24, 0.18);
}

.recent-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.recent-head span,
.clear-btn {
  font-size: 12px;
  font-weight: 700;
}

.recent-head span {
  color: var(--text-primary);
}

.clear-btn {
  color: var(--text-muted);
}

.recent-list {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.recent-item {
  min-width: 0;
  min-height: 40px;
  padding: 0 12px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border: 1px solid transparent;
  text-align: left;
  transition: var(--transition);
}

.recent-item span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-item:hover {
  background: var(--bg-active);
  border-color: var(--border);
  color: var(--text-primary);
}

.mobile-search-pop-enter-active,
.mobile-search-pop-leave-active {
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.mobile-search-pop-enter-from,
.mobile-search-pop-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>

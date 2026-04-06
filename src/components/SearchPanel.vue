<template>
  <div class="search-panel">
    <template v-if="!hasQuery">
      <div class="empty-view">
        <div class="empty-hero">
          <div class="empty-icon">
            <svg width="38" height="38" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="11" cy="11" r="7" />
              <path d="m21 21-4.35-4.35" />
            </svg>
          </div>
          <div class="empty-copy">
            <h2>搜索音乐</h2>
            <p>在顶部输入歌曲、歌手或专辑名称，结果会优先读取本地缓存。</p>
          </div>
          <div class="empty-actions">
            <button type="button" class="app-btn app-btn--primary" @click="emit('open-history')">最近播放</button>
            <button type="button" class="app-btn app-btn--ghost" @click="emit('open-library')">打开歌单</button>
          </div>
        </div>

        <section v-if="recentSearches.length" class="history-card">
          <div class="section-head">
            <div>
              <span class="section-kicker">Recent</span>
              <h3>最近搜索</h3>
            </div>
            <button type="button" class="app-btn app-btn--ghost compact-btn" @click="clearAllHistory">清空</button>
          </div>

          <div class="history-list app-scroll">
            <div v-for="term in recentSearches" :key="term" class="history-item">
              <button type="button" class="history-term" @click="applyHistory(term)">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10" />
                  <polyline points="12 6 12 12 16 14" />
                </svg>
                <span>{{ term }}</span>
              </button>
              <button type="button" class="app-icon-btn app-icon-btn--danger" @click="removeHistory(term)" title="删除记录">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          </div>
        </section>
      </div>
    </template>

    <template v-else>
      <div class="result-header">
        <div class="result-meta">
          <span class="section-kicker">Search</span>
          <h2>{{ keyword }}</h2>
          <span class="result-sub">{{ currentSourceLabel }} · {{ searching ? '搜索中' : `${filteredResults.length} 首结果` }}</span>
        </div>

        <div class="filter-row">
          <button
            type="button"
            class="app-chip-btn"
            :class="{ active: favoriteOnly }"
            @click="favoriteOnly = !favoriteOnly"
          >
            只看收藏
          </button>
          <select v-model="sortBy" class="sort-select">
            <option value="default">默认排序</option>
            <option value="name">按歌名</option>
            <option value="artist">按歌手</option>
            <option value="album">按专辑</option>
          </select>
        </div>
      </div>

      <div v-if="errMsg" class="err-bar">{{ errMsg }}</div>

      <div ref="panelBody" class="panel-body app-scroll">
        <div v-if="searching" class="loading-list">
          <div v-for="i in 8" :key="i" class="skeleton-row">
            <div class="sk sk-index" />
            <div class="sk sk-cover" />
            <div class="sk-info">
              <div class="sk sk-title" />
              <div class="sk sk-sub" />
            </div>
            <div class="sk sk-tail" />
          </div>
        </div>

        <div v-else-if="!filteredResults.length" class="empty-state">
          <p>没有找到相关结果</p>
          <span>换个关键词，或者切换音源再试一次。</span>
        </div>

        <div v-else class="result-list">
          <SharedSongRow
            v-for="(item, idx) in filteredResults"
            :key="item.source + '-' + item.id"
            class="result-row search-result-row"
            :data-cover-key="coverKey(item)"
            :title="item.name"
            :subtitle="`${getArtistNames(item.artist)} · ${getAlbumName(item.album)}`"
            :duration-text="formatDuration(item.durationSec ?? null)"
            :playing-label="isCurrentTrack(item) ? '播放中' : undefined"
            :active="isCurrentTrack(item)"
            @dblclick="playNow(item)"
            @mouseenter="loadCover(item)"
          >
            <template #index>
              <span class="row-index">{{ idx + 1 }}</span>
            </template>

            <template #cover>
              <img
                v-if="media.getTrackCoverUrl(item)"
                :src="media.getTrackCoverUrl(item) ?? undefined"
                :alt="item.name"
                @error="media.markCoverLoadFailed(item)"
              />
              <div v-else class="cover-ph">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                  <circle cx="12" cy="12" r="10" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              </div>
            </template>

            <template #extra>
              <span class="source-tag">{{ sourceLabel(item.source) }}</span>
            </template>

            <template #actions>
              <div class="action-row">
                <button
                  type="button"
                  class="app-icon-btn"
                  :class="{ active: isFavorite(item) }"
                  title="收藏"
                  @click.stop="toggleFavorite(item)"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path
                      d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z"
                      :fill="isFavorite(item) ? 'currentColor' : 'none'"
                    />
                  </svg>
                </button>
                <button type="button" class="app-icon-btn" title="加入歌单" @click.stop="openPlaylistPicker($event, item)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M3 7h12" />
                    <path d="M3 12h12" />
                    <path d="M3 17h8" />
                    <path d="M19 8v10" />
                    <path d="M14 13h10" />
                  </svg>
                </button>
                <button type="button" class="app-icon-btn" title="加入队列" @click.stop="addQueue(item)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                </button>
                <DownloadButton :track="toTrack(item)" />
                <button type="button" class="app-icon-btn play-btn" title="播放" @click.stop="playNow(item)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                    <polygon points="5,3 19,12 5,21" />
                  </svg>
                </button>
              </div>
            </template>
          </SharedSongRow>
        </div>
      </div>
    </template>

    <Teleport to="body">
      <Transition name="picker-pop">
        <div v-if="pickerOpen" ref="pickerPanel" class="playlist-picker" :style="pickerStyle" @click.stop>
          <div class="picker-head">加入歌单</div>
          <div v-if="library.playlists.length" class="picker-list">
            <button
              v-for="playlist in library.playlists"
              :key="playlist.id"
              type="button"
              class="picker-item"
              @click="addToPlaylist(playlist.id)"
            >
              <span>{{ playlist.name }}</span>
              <span class="picker-count">{{ playlist.tracks.length }}</span>
            </button>
          </div>
          <div v-else class="picker-empty">还没有创建歌单</div>

          <div class="picker-create">
            <input
              v-model="newPlaylistName"
              class="picker-input"
              placeholder="新歌单名称"
              @keydown.enter="createPlaylistAndAdd"
            />
            <button
              type="button"
              class="app-btn app-btn--primary compact-btn"
              :disabled="!newPlaylistName.trim()"
              @click="createPlaylistAndAdd"
            >
              创建
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import {
  musicApi,
  getArtistNames,
  getAlbumName,
  toStr,
  SOURCES,
  type SearchResult,
  type MusicSource,
} from '@/api/music';
import DownloadButton from '@/components/DownloadButton.vue';
import SharedSongRow from '@/components/SharedSongRow.vue';
import { usePlayerStore, type Track } from '@/stores/player';
import { useLibraryStore } from '@/stores/library';
import { useMediaStore } from '@/stores/media';
import { useUiStore } from '@/stores/ui';
import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';
import { formatDuration } from '@/utils/formatters';

defineProps<{ mode?: string }>();

const emit = defineEmits<{
  'open-library': [];
  'open-history': [];
}>();

const RECENT_SEARCHES_KEY = 'fashion:recent-searches';
const RECENT_SEARCHES_VERSION = 1;
const MAX_RECENT_SEARCHES = 10;

const player = usePlayerStore();
const library = useLibraryStore();
const media = useMediaStore();
const ui = useUiStore();

const keyword = ref('');
const source = ref<MusicSource>(ui.toolbarSource);
const results = ref<SearchResult[]>([]);
const searching = ref(false);
const errMsg = ref('');
const favoriteOnly = ref(false);
const sortBy = ref<'default' | 'name' | 'artist' | 'album'>('default');
const recentSearches = ref(
  readVersionedStorage<string[]>(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, {
    fallback: [],
    validate: (value): value is string[] => Array.isArray(value) && value.every((item) => typeof item === 'string'),
  }).slice(0, MAX_RECENT_SEARCHES),
);
const panelBody = ref<HTMLElement | null>(null);
const pickerOpen = ref(false);
const pickerTrack = ref<Track | null>(null);
const pickerStyle = ref<Record<string, string>>({});
const pickerPanel = ref<HTMLElement | null>(null);
const newPlaylistName = ref('');

const hasQuery = computed(() => keyword.value.trim().length > 0);
const currentSourceLabel = computed(() => sourceLabel(source.value));
const resultLookup = computed(() => new Map(results.value.map((item) => [coverKey(item), item] as const)));

const filteredResults = computed(() => {
  let list = results.value;

  if (favoriteOnly.value) {
    list = list.filter((item) => library.isFavorite({ id: toStr(item.id), source: item.source }));
  }

  if (sortBy.value === 'name') {
    return [...list].sort((a, b) => a.name.localeCompare(b.name));
  }
  if (sortBy.value === 'artist') {
    return [...list].sort((a, b) => getArtistNames(a.artist).localeCompare(getArtistNames(b.artist)));
  }
  if (sortBy.value === 'album') {
    return [...list].sort((a, b) => getAlbumName(a.album).localeCompare(getAlbumName(b.album)));
  }

  return list;
});

let searchToken = 0;
let pickerAnchor: HTMLElement | null = null;
let coverObserver: IntersectionObserver | null = null;

function coverKey(item: SearchResult) {
  return `${item.source}-${toStr(item.id)}`;
}

function sourceLabel(sourceValue: string) {
  return SOURCES.find((item) => item.value === sourceValue)?.label ?? sourceValue;
}

function isFavorite(item: SearchResult) {
  return library.isFavorite({ id: toStr(item.id), source: item.source });
}

function isCurrentTrack(item: SearchResult) {
  return player.currentTrack?.id === toStr(item.id) && player.currentTrack?.source === item.source;
}

function toTrack(item: SearchResult): Track {
  return media.attachTrackCover({
    id: toStr(item.id),
    name: item.name,
    artist: getArtistNames(item.artist),
    album: getAlbumName(item.album),
    pic_id: toStr(item.pic_id),
    lyric_id: toStr(item.lyric_id),
    source: item.source,
  });
}

function playNow(item: SearchResult) {
  player.addToQueue(toTrack(item), true);
}

function addQueue(item: SearchResult) {
  player.addToQueue(toTrack(item), false);
}

function toggleFavorite(item: SearchResult) {
  library.toggleFavorite(toTrack(item));
}

async function loadCover(item: SearchResult) {
  try {
    const track = toTrack(item);
    const coverUrl = await media.ensureTrackCover(track);
    if (coverUrl) {
      player.syncTrackCover(track, coverUrl);
      library.syncTrackCover(track, coverUrl);
    }
  } catch {
    // Ignore cover errors.
  }
}

function disconnectCoverObserver() {
  coverObserver?.disconnect();
  coverObserver = null;
}

function bindCoverObserver() {
  disconnectCoverObserver();

  if (!panelBody.value || !filteredResults.value.length) return;

  filteredResults.value.slice(0, 6).forEach((item) => {
    void loadCover(item);
  });

  if (typeof IntersectionObserver === 'undefined') return;

  coverObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const target = entry.target as HTMLElement;
        const key = target.dataset.coverKey;
        if (!key) continue;

        const item = resultLookup.value.get(key);
        if (item) {
          void loadCover(item);
        }
        coverObserver?.unobserve(target);
      }
    },
    { root: panelBody.value, rootMargin: '140px 0px' },
  );

  panelBody.value
    .querySelectorAll<HTMLElement>('.result-row[data-cover-key]')
    .forEach((row) => coverObserver?.observe(row));
}

async function doSearch() {
  const normalized = keyword.value.trim();
  if (!normalized) {
    results.value = [];
    errMsg.value = '';
    return;
  }

  const token = ++searchToken;
  searching.value = true;
  errMsg.value = '';

  try {
    const data = await musicApi.search(source.value, normalized, 30, 1);
    if (token !== searchToken) return;

    data.forEach((item) => {
      media.primeTrackCover(item);
    });
    results.value = data;
    recentSearches.value = [normalized, ...recentSearches.value.filter((item) => item !== normalized)].slice(0, MAX_RECENT_SEARCHES);
    writeVersionedStorage(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, recentSearches.value);

    await nextTick();
    bindCoverObserver();
  } catch (error: unknown) {
    if (token !== searchToken) return;
    errMsg.value = error instanceof Error ? error.message : '搜索失败，请稍后重试';
    results.value = [];
    disconnectCoverObserver();
  } finally {
    if (token === searchToken) {
      searching.value = false;
    }
  }
}

function syncToolbarSearch() {
  keyword.value = ui.toolbarSearch;
  source.value = ui.toolbarSource;

  if (!keyword.value.trim()) {
    results.value = [];
    errMsg.value = '';
    disconnectCoverObserver();
    return;
  }

  void doSearch();
}

function removeHistory(term: string) {
  recentSearches.value = recentSearches.value.filter((item) => item !== term);
  writeVersionedStorage(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, recentSearches.value);
}

function clearAllHistory() {
  recentSearches.value = [];
  writeVersionedStorage(RECENT_SEARCHES_KEY, RECENT_SEARCHES_VERSION, recentSearches.value);
}

function applyHistory(term: string) {
  ui.submitToolbarSearch(term, source.value);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

async function openPlaylistPicker(event: MouseEvent, item: SearchResult) {
  pickerTrack.value = toTrack(item);
  pickerAnchor = event.currentTarget as HTMLElement;
  pickerOpen.value = true;
  await nextTick();
  updatePickerPosition();
}

function updatePickerPosition() {
  if (!pickerAnchor || !pickerPanel.value) return;

  const rect = pickerAnchor.getBoundingClientRect();
  const width = pickerPanel.value.offsetWidth || 248;
  const height = pickerPanel.value.offsetHeight || 220;

  pickerStyle.value = {
    left: `${clamp(rect.right - width, 10, window.innerWidth - width - 10)}px`,
    top: `${clamp(rect.bottom + 10, 10, window.innerHeight - height - 10)}px`,
  };
}

function closePicker() {
  pickerOpen.value = false;
  pickerTrack.value = null;
  pickerAnchor = null;
  newPlaylistName.value = '';
}

function addToPlaylist(playlistId: string) {
  if (!pickerTrack.value) return;
  library.addTrackToPlaylist(playlistId, pickerTrack.value);
  closePicker();
}

function createPlaylistAndAdd() {
  const playlistId = library.createPlaylist(newPlaylistName.value);
  if (!playlistId || !pickerTrack.value) return;
  library.addTrackToPlaylist(playlistId, pickerTrack.value);
  closePicker();
}

function handlePointerDown(event: PointerEvent) {
  const target = event.target as Node | null;
  const clickedPicker = pickerPanel.value?.contains(target) ?? false;
  const clickedAnchor = pickerAnchor?.contains(target) ?? false;

  if (!clickedPicker && !clickedAnchor) {
    closePicker();
  }
}

function handleResize() {
  if (pickerOpen.value) updatePickerPosition();
}

watch(() => ui.toolbarSearchNonce, syncToolbarSearch);
watch(() => ui.toolbarSource, (value) => {
  source.value = value;
});
watch(filteredResults, async () => {
  await nextTick();
  bindCoverObserver();
});

onMounted(() => {
  window.addEventListener('pointerdown', handlePointerDown);
  window.addEventListener('resize', handleResize);
  syncToolbarSearch();
});

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', handlePointerDown);
  window.removeEventListener('resize', handleResize);
  disconnectCoverObserver();
});
</script>

<style scoped>
.search-panel {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.empty-view {
  flex: 1;
  min-height: 0;
  padding: 28px 22px 18px 18px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.empty-hero,
.history-card {
  border-radius: 24px;
  border: 1px solid var(--border);
  background: linear-gradient(180deg, var(--panel-strong), rgba(255, 255, 255, 0.02));
  box-shadow: 0 16px 42px rgba(0, 0, 0, 0.1);
}

.empty-hero {
  padding: 28px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.empty-icon {
  width: 58px;
  height: 58px;
  border-radius: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  background: var(--accent-dim);
}

.empty-copy h2 {
  font-size: 30px;
  line-height: 1.1;
  font-weight: 900;
  color: var(--text-primary);
}

.empty-copy p {
  margin-top: 10px;
  max-width: 520px;
  font-size: 14px;
  color: var(--text-secondary);
}

.empty-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.history-card {
  min-height: 0;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.section-kicker {
  display: inline-block;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.section-head h3 {
  margin-top: 6px;
  font-size: 20px;
  font-weight: 800;
  color: var(--text-primary);
}

.compact-btn {
  height: 32px;
  padding: 0 12px;
  font-size: 12px;
}

.history-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-height: 248px;
  padding-right: 2px;
  align-content: flex-start;
}

.history-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  max-width: min(220px, calc(50% - 3px));
  padding: 3px;
  border-radius: 999px;
  background: var(--bg-hover);
}

.history-term {
  flex: 0 1 auto;
  min-width: 0;
  max-width: 100%;
  padding: 6px 10px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  gap: 6px;
  text-align: left;
  color: var(--text-secondary);
  font-size: 12px;
  transition: var(--transition);
}

.history-term:hover {
  color: var(--text-primary);
}

.history-term span {
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-item .app-icon-btn {
  width: 26px;
  height: 26px;
  border-radius: 999px;
  flex-shrink: 0;
}

.result-header {
  margin: 10px 14px 0;
  padding: 12px 14px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: linear-gradient(135deg, var(--panel-strong), rgba(255, 255, 255, 0.03));
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-shrink: 0;
}

.result-meta h2 {
  margin-top: 6px;
  font-size: 24px;
  font-weight: 900;
  color: var(--text-primary);
}

.result-sub {
  display: block;
  margin-top: 6px;
  font-size: 13px;
  color: var(--text-muted);
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.sort-select {
  height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  cursor: pointer;
  border: 1px solid transparent;
}

.err-bar {
  margin: 0 18px 10px;
  padding: 12px 14px;
  border-radius: 16px;
  background: var(--danger-soft);
  color: var(--text-danger);
  border: 1px solid rgba(248, 113, 113, 0.18);
}

.panel-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 0 10px 14px;
}

.loading-list,
.result-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skeleton-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 18px;
  background: var(--bg-hover);
}

.sk {
  background: var(--bg-active);
  border-radius: 8px;
  animation: shimmer 1.4s infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.85; }
}

.sk-index { width: 24px; height: 14px; flex-shrink: 0; }
.sk-cover { width: 46px; height: 46px; border-radius: 14px; flex-shrink: 0; }
.sk-info { flex: 1; display: flex; flex-direction: column; gap: 6px; }
.sk-title { width: 52%; height: 14px; }
.sk-sub { width: 36%; height: 12px; }
.sk-tail { width: 116px; height: 14px; flex-shrink: 0; }

.empty-state {
  min-height: 260px;
  margin: 0 14px;
  padding: 24px;
  border-radius: 20px;
  border: 1px dashed var(--border);
  background: var(--bg-hover);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  text-align: center;
  color: var(--text-muted);
}

.empty-state p {
  font-size: 18px;
  font-weight: 800;
  color: var(--text-primary);
}

.result-row {
  padding: 10px 14px;
  border-radius: 18px;
}

.search-result-row:hover {
  background: var(--bg-hover);
}

.row-index {
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}

.search-result-row :deep(.row-cover) {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  background: var(--bg-active);
}

.search-result-row :deep(.row-cover img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-ph {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.source-tag {
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 700;
  white-space: nowrap;
}

.action-row {
  display: flex;
  gap: 6px;
}

.action-row :deep(.app-icon-btn) {
  width: 42px;
  height: 42px;
  border-radius: 14px;
}

.action-row :deep(svg) {
  width: 18px;
  height: 18px;
}

.action-row :deep(.spinner) {
  width: 18px;
  height: 18px;
}

.action-row .app-icon-btn.active {
  color: var(--text-heart);
}

.play-btn {
  background: var(--accent-dim);
  color: var(--accent);
}

.play-btn:hover {
  background: var(--accent);
  color: var(--text-on-accent);
}

.playlist-picker {
  position: fixed;
  width: 248px;
  padding: 12px;
  border-radius: 20px;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  box-shadow: var(--window-shadow);
  z-index: 2200;
}

.picker-head {
  padding: 4px 4px 10px;
  font-size: 12px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.picker-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 10px;
}

.picker-item {
  width: 100%;
  padding: 10px 12px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  color: var(--text-secondary);
  transition: var(--transition);
}

.picker-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.picker-count {
  font-size: 11px;
  color: var(--text-muted);
}

.picker-empty {
  padding: 12px 4px;
  font-size: 13px;
  color: var(--text-muted);
}

.picker-create {
  display: flex;
  gap: 8px;
  border-top: 1px solid var(--border);
  padding-top: 12px;
}

.picker-input {
  flex: 1;
  height: 32px;
  padding: 0 12px;
  border-radius: 999px;
  background: var(--bg-hover);
  color: var(--text-primary);
}

.picker-pop-enter-active,
.picker-pop-leave-active {
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.picker-pop-enter-from,
.picker-pop-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

@media (max-width: 760px) {
  .history-item {
    max-width: 100%;
  }

  .action-row {
    justify-content: flex-start;
  }
}
</style>

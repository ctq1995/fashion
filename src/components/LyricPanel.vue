<template>
  <div class="lyric-panel" :class="{ fullscreen: props.fullscreen }">
    <div class="ambient-layer" />

    <button
      v-if="props.fullscreen"
      type="button"
      class="fullscreen-exit"
      title="退出歌词全屏"
      @click="emit('toggle-fullscreen')"
    >
      <svg
        width="18"
        height="18"
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
      <span>退出全屏</span>
      <small>Esc</small>
    </button>

    <div class="lyric-stage">
      <div class="stage-cover">
        <div class="cover-frame">
          <img v-if="player.currentTrack?.coverUrl" :src="player.currentTrack.coverUrl" :alt="trackTitle" />
          <div v-else class="cover-placeholder">
            <svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </div>
      </div>

      <div class="stage-info">
        <div class="info-head">
          <h1>{{ trackTitle }}</h1>
          <p>{{ trackArtist }}</p>
        </div>

        <div class="info-main">
          <div v-if="player.lyricLines.length" ref="lyricViewport" class="lyric-viewport">
            <div class="lyric-list">
              <div class="lyric-spacer" :style="{ height: `${lyricTopSpacer}px` }" aria-hidden="true" />

              <button
                v-for="(row, index) in player.lyricLines"
                :key="`${row.time}-${index}`"
                type="button"
                class="lyric-row"
                :ref="(el) => setLyricRowRef(index, el)"
                :class="{ active: index === activeLyricIndex }"
                :style="lyricRowStyle(index)"
                @click="player.seek(row.time)"
              >
                <span class="lyric-main">
                  <span class="lyric-main-base">{{ row.text || '' }}</span>
                  <span
                    v-if="index === activeLyricIndex"
                    class="lyric-main-highlight"
                    :style="lyricHighlightStyle(index)"
                  >
                    {{ row.text || '' }}
                  </span>
                </span>
                <span v-if="translatedLine(index)" class="lyric-sub">{{ translatedLine(index) }}</span>
              </button>

              <div class="lyric-spacer" :style="{ height: `${lyricBottomSpacer}px` }" aria-hidden="true" />
            </div>
          </div>

          <template v-else>
            <div class="main-title">{{ mainTitle }}</div>
            <div v-for="credit in credits" :key="credit.label" class="credit-row" :class="credit.level">
              <span class="credit-label">{{ credit.label }}:</span>
              <strong class="credit-value">{{ credit.value }}</strong>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
  type CSSProperties,
  type ComponentPublicInstance,
} from 'vue';
import { usePlayerStore } from '@/stores/player';

type CreditLevel = 'highlight' | 'normal' | 'muted' | 'dimmed';

const MIN_LINE_DURATION = 0.65;
const MAX_LINE_DURATION = 4.2;
const EDGE_SPACER_OFFSET = 52;
const NORMAL_LYRIC_ANCHOR_RATIO = 0.56;
const FULLSCREEN_LYRIC_ANCHOR_RATIO = 0.52;

const props = defineProps<{
  fullscreen?: boolean;
}>();

const emit = defineEmits<{
  'toggle-fullscreen': [];
}>();

const player = usePlayerStore();
const visualTime = ref(0);
const lyricViewport = ref<HTMLElement | null>(null);
const lyricViewportHeight = ref(0);

let visualTimeRaf = 0;
let lastFrameMs = 0;
let lastCenteredIndex = -1;
let viewportResizeObserver: ResizeObserver | null = null;

const lyricRowRefs = new Map<number, HTMLElement>();

const trackTitle = computed(() => player.currentTrack?.name ?? '暂无歌曲');
const trackArtist = computed(() => player.currentTrack?.artist ?? '未知歌手');

const mainTitle = computed(() => {
  if (!player.currentTrack) return '播放一首歌后，这里会显示歌词和歌曲信息';
  return `${trackTitle.value} - ${trackArtist.value}`;
});

const credits = computed<Array<{ label: string; value: string; level: CreditLevel }>>(() => {
  const fallback = trackArtist.value;
  return [
    { label: '词', value: fallback, level: 'highlight' },
    { label: '曲', value: fallback, level: 'normal' },
    { label: '编曲', value: fallback, level: 'muted' },
    { label: '制作', value: fallback, level: 'dimmed' },
  ];
});

const activeLyricIndex = computed(() => {
  if (!player.lyricLines.length) return -1;
  if (player.currentLyricIndex >= 0 && player.currentLyricIndex < player.lyricLines.length) {
    return player.currentLyricIndex;
  }
  return -1;
});

function resolveLyricLineDuration(index: number): number {
  const current = player.lyricLines[index];
  if (!current) return 2.4;

  const prev = player.lyricLines[index - 1];
  const next = player.lyricLines[index + 1];

  let duration = 2.4;
  if (next && next.time > current.time) {
    duration = next.time - current.time;
  } else if (prev && current.time > prev.time) {
    duration = current.time - prev.time;
  }

  return Math.min(MAX_LINE_DURATION, Math.max(MIN_LINE_DURATION, duration));
}

const activeLineProgress = computed(() => {
  const index = activeLyricIndex.value;
  if (index < 0 || index >= player.lyricLines.length) return 0;

  const current = player.lyricLines[index];
  const elapsed = visualTime.value - current.time;
  const duration = resolveLyricLineDuration(index);
  return Math.min(1, Math.max(0, elapsed / duration));
});

function lyricHighlightStyle(index: number): CSSProperties | undefined {
  if (index !== activeLyricIndex.value) return undefined;
  return {
    '--line-progress': `${(activeLineProgress.value * 100).toFixed(2)}%`,
  } as CSSProperties;
}

function lyricRowStyle(index: number): CSSProperties {
  const active = activeLyricIndex.value;
  if (active < 0) {
    return {
      opacity: '0.78',
      transform: 'scale(0.985)',
      zIndex: '1',
    };
  }

  const distance = Math.abs(index - active);
  const opacity = index === active ? 1 : Math.max(0.22, 1 - distance * 0.12);
  const scale = index === active ? 1.02 : Math.max(0.94, 1 - distance * 0.018);
  const zIndex = Math.max(1, 100 - distance);

  return {
    opacity: opacity.toFixed(3),
    transform: `scale(${scale.toFixed(3)})`,
    zIndex: String(zIndex),
  };
}

const translatedLineMap = computed(() => {
  const map = new Map<number, string>();
  if (!player.lyricLines.length || !player.tlyricLines.length) return map;

  for (let i = 0; i < player.lyricLines.length; i += 1) {
    const current = player.lyricLines[i];
    let bestText = '';
    let bestDelta = 0.8;

    for (const line of player.tlyricLines) {
      const delta = Math.abs(line.time - current.time);
      if (!line.text || delta >= bestDelta) continue;
      bestDelta = delta;
      bestText = line.text;
    }

    if (bestText) map.set(i, bestText);
  }

  return map;
});

const lyricAnchorRatio = computed(() => (
  props.fullscreen ? FULLSCREEN_LYRIC_ANCHOR_RATIO : NORMAL_LYRIC_ANCHOR_RATIO
));

const lyricTopSpacer = computed(() => (
  Math.max(0, lyricViewportHeight.value * lyricAnchorRatio.value - EDGE_SPACER_OFFSET)
));

const lyricBottomSpacer = computed(() => (
  Math.max(0, lyricViewportHeight.value * (1 - lyricAnchorRatio.value) - EDGE_SPACER_OFFSET)
));

function translatedLine(index: number): string {
  return translatedLineMap.value.get(index) ?? '';
}

function setLyricRowRef(index: number, element: Element | ComponentPublicInstance | null) {
  if (element instanceof HTMLElement) {
    lyricRowRefs.set(index, element);
    return;
  }

  lyricRowRefs.delete(index);
}

function updateLyricViewportMetrics() {
  lyricViewportHeight.value = lyricViewport.value?.clientHeight ?? 0;
}

async function centerLyricRow(index: number, smooth: boolean) {
  if (index < 0) return;

  await nextTick();

  const container = lyricViewport.value;
  const row = lyricRowRefs.get(index);
  if (!container || !row) return;

  const containerRect = container.getBoundingClientRect();
  const rowRect = row.getBoundingClientRect();
  const rowCenter = rowRect.top - containerRect.top + container.scrollTop + rowRect.height / 2;
  const anchorY = container.clientHeight * lyricAnchorRatio.value;
  const targetTop = rowCenter - anchorY;
  const maxScrollTop = Math.max(0, container.scrollHeight - container.clientHeight);
  const top = Math.max(0, Math.min(targetTop, maxScrollTop));
  const behavior: ScrollBehavior =
    smooth && lastCenteredIndex >= 0 && Math.abs(index - lastCenteredIndex) <= 2 && player.isPlaying
      ? 'smooth'
      : 'auto';

  container.scrollTo({ top, behavior });
  lastCenteredIndex = index;
}

async function resetLyricViewport() {
  await nextTick();
  lyricViewport.value?.scrollTo({ top: 0, behavior: 'auto' });
  lastCenteredIndex = -1;
}

function stopVisualTimeLoop() {
  if (!visualTimeRaf) return;
  window.cancelAnimationFrame(visualTimeRaf);
  visualTimeRaf = 0;
}

function startVisualTimeLoop() {
  if (visualTimeRaf) return;
  visualTime.value = player.currentTime;
  lastFrameMs = performance.now();

  const tick = (now: number) => {
    const dt = Math.min(0.08, Math.max(0, (now - lastFrameMs) / 1000));
    lastFrameMs = now;

    const target = player.currentTime;
    if (player.isPlaying) {
      const predicted = visualTime.value + dt;
      visualTime.value = predicted + (target - predicted) * 0.2;
    } else {
      visualTime.value = visualTime.value + (target - visualTime.value) * 0.35;
      if (Math.abs(target - visualTime.value) < 0.004) {
        visualTime.value = target;
      }
    }

    visualTimeRaf = window.requestAnimationFrame(tick);
  };

  visualTimeRaf = window.requestAnimationFrame(tick);
}

watch(
  () => player.currentTime,
  (value) => {
    if (!player.isPlaying) visualTime.value = value;
  },
);

watch(
  activeLyricIndex,
  (index, previous) => {
    if (index < 0) {
      if (previous >= 0) void resetLyricViewport();
      return;
    }

    void centerLyricRow(index, previous >= 0);
  },
  { flush: 'post' },
);

watch(
  () => player.currentTrack?.id,
  () => {
    lyricRowRefs.clear();
    void resetLyricViewport();
  },
  { flush: 'post' },
);

watch(
  () => player.lyricLines.length,
  () => {
    lyricRowRefs.clear();
    updateLyricViewportMetrics();

    if (activeLyricIndex.value >= 0) {
      void centerLyricRow(activeLyricIndex.value, false);
    } else {
      void resetLyricViewport();
    }
  },
  { flush: 'post' },
);

watch(
  () => props.fullscreen,
  () => {
    updateLyricViewportMetrics();
    if (activeLyricIndex.value >= 0) {
      void centerLyricRow(activeLyricIndex.value, false);
    }
  },
  { flush: 'post' },
);

watch(
  lyricViewport,
  (element, previous) => {
    if (viewportResizeObserver && previous) {
      viewportResizeObserver.unobserve(previous);
    }

    if (!element) {
      lyricViewportHeight.value = 0;
      return;
    }

    updateLyricViewportMetrics();

    if (typeof ResizeObserver === 'undefined') return;
    if (!viewportResizeObserver) {
      viewportResizeObserver = new ResizeObserver(() => {
        updateLyricViewportMetrics();
        if (activeLyricIndex.value >= 0) {
          void centerLyricRow(activeLyricIndex.value, false);
        }
      });
    }

    viewportResizeObserver.observe(element);
  },
  { flush: 'post' },
);

onMounted(() => {
  startVisualTimeLoop();
  updateLyricViewportMetrics();

  if (activeLyricIndex.value >= 0) {
    void centerLyricRow(activeLyricIndex.value, false);
  }
});

onBeforeUnmount(() => {
  stopVisualTimeLoop();
  viewportResizeObserver?.disconnect();
  viewportResizeObserver = null;
});
</script>

<style scoped>
.lyric-panel {
  --lp-text: rgba(255, 255, 255, 0.92);
  --lp-text-sub: rgba(255, 255, 255, 0.62);
  --lp-text-muted: rgba(255, 255, 255, 0.36);
  --lp-accent: #16d6a0;
  --lp-accent-light: #78f0cb;
  --lp-border: rgba(255, 255, 255, 0.08);
  --lp-card: rgba(255, 255, 255, 0.06);

  position: relative;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding: 14px 44px 12px;
  background: transparent;
  color: var(--lp-text);
}

.ambient-layer {
  position: absolute;
  inset: -16% -10%;
  pointer-events: none;
  filter: blur(22px);
  background:
    radial-gradient(circle at 18% 30%, rgba(22, 214, 160, 0.08), transparent 36%),
    radial-gradient(circle at 84% 54%, rgba(93, 127, 121, 0.1), transparent 50%);
}

.fullscreen-exit {
  position: absolute;
  top: 18px;
  right: 20px;
  z-index: 3;
  height: 42px;
  padding: 0 14px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: rgba(255, 255, 255, 0.9);
  background: rgba(7, 13, 12, 0.52);
  border: 1px solid rgba(255, 255, 255, 0.12);
  box-shadow: 0 12px 26px rgba(0, 0, 0, 0.22);
  backdrop-filter: blur(12px);
  transition: var(--transition);
}

.fullscreen-exit:hover {
  background: rgba(255, 255, 255, 0.14);
  color: #fff;
}

.fullscreen-exit span {
  font-size: 12px;
  font-weight: 700;
}

.fullscreen-exit small {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.62);
}

.lyric-stage {
  position: relative;
  z-index: 1;
  height: 100%;
  min-height: 0;
  max-width: 1020px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: minmax(260px, 350px) minmax(300px, 460px);
  justify-content: center;
  align-content: center;
  gap: clamp(34px, 6vw, 80px);
  align-items: center;
}

.lyric-panel.fullscreen {
  padding: 24px 72px 22px;
}

.lyric-panel.fullscreen .lyric-stage {
  max-width: 1360px;
  grid-template-columns: minmax(320px, 420px) minmax(480px, 700px);
  gap: clamp(44px, 7vw, 120px);
}

.stage-cover {
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-frame {
  width: clamp(260px, 34vw, 350px);
  aspect-ratio: 1 / 1;
  border-radius: 16px;
  overflow: hidden;
  background: var(--lp-card);
  border: 1px solid var(--lp-border);
  box-shadow:
    0 24px 70px rgba(6, 13, 13, 0.34),
    0 0 0 1px var(--lp-border);
}

.lyric-panel.fullscreen .cover-frame {
  width: clamp(320px, 28vw, 420px);
  border-radius: 20px;
}

.cover-frame img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--lp-text-muted);
}

.stage-info {
  width: min(100%, 460px);
  height: 350px;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  min-width: 0;
}

.lyric-panel.fullscreen .stage-info {
  width: min(100%, 700px);
  height: 470px;
}

.info-head {
  overflow: visible;
  padding-top: 2px;
}

.info-head h1 {
  font-size: clamp(22px, 2.7vw, 35px);
  line-height: 1.22;
  color: var(--lp-text);
  font-weight: 800;
  letter-spacing: 0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding-bottom: 2px;
}

.info-head p {
  margin-top: 2px;
  font-size: clamp(12px, 1.28vw, 18px);
  line-height: 1.22;
  color: var(--lp-text-sub);
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding-bottom: 2px;
}

.info-main {
  width: 100%;
  height: 272px;
  margin-top: 34px;
  min-height: 0;
  overflow: hidden;
}

.lyric-panel.fullscreen .info-main {
  height: 370px;
  margin-top: 42px;
}

.lyric-viewport {
  width: 100%;
  height: 100%;
  overflow-x: hidden;
  overflow-y: auto;
  padding-right: 6px;
  overscroll-behavior: contain;
  scrollbar-gutter: stable;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.16) transparent;
  mask-image: linear-gradient(
    to bottom,
    transparent 0,
    #000 56px,
    #000 calc(100% - 56px),
    transparent 100%
  );
}

.lyric-viewport::-webkit-scrollbar {
  width: 6px;
}

.lyric-viewport::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.16);
}

.lyric-viewport::-webkit-scrollbar-track {
  background: transparent;
}

.lyric-list {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.lyric-spacer {
  flex: 0 0 auto;
}

.lyric-row {
  position: relative;
  width: 100%;
  padding: 6px 10px;
  border: 0;
  border-radius: 18px;
  text-align: left;
  color: var(--lp-text-muted);
  background: transparent;
  transform-origin: left center;
  will-change: transform, opacity;
  transition: transform 0.22s ease, opacity 0.22s ease, color 0.2s ease, background 0.2s ease;
}

.lyric-row:hover {
  background: rgba(255, 255, 255, 0.05);
}

.lyric-row.active {
  color: var(--lp-text);
  background: rgba(255, 255, 255, 0.06);
}

.lyric-main {
  position: relative;
  display: inline-block;
  max-width: 100%;
}

.lyric-main-base,
.lyric-main-highlight {
  display: inline-block;
  font-size: clamp(19px, 2.35vw, 31px);
  line-height: 1.16;
  font-weight: 600;
  letter-spacing: 0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: clip;
}

.lyric-row.active .lyric-main-base,
.lyric-row.active .lyric-main-highlight {
  font-size: clamp(21px, 2.6vw, 34px);
  font-weight: 800;
}

.lyric-main-highlight {
  position: absolute;
  left: 0;
  top: 0;
  width: var(--line-progress, 0%);
  height: 100%;
  color: var(--lp-accent-light);
  white-space: nowrap;
  overflow: hidden;
  pointer-events: none;
  transition: width 80ms linear;
}

.lyric-sub {
  display: block;
  max-width: 100%;
  margin-top: 4px;
  font-size: 10px;
  color: var(--lp-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: clip;
}

.lyric-row.active .lyric-sub {
  color: var(--lp-text-sub);
}

.main-title {
  font-size: clamp(25px, 2.7vw, 38px);
  line-height: 1.14;
  color: var(--lp-text-muted);
  letter-spacing: 0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.credit-row {
  margin-top: 12px;
  display: flex;
  align-items: baseline;
  gap: 12px;
  white-space: nowrap;
  overflow: hidden;
}

.credit-label {
  font-size: clamp(22px, 2.2vw, 34px);
  font-weight: 700;
  letter-spacing: 0.02em;
}

.credit-value {
  font-size: clamp(28px, 2.8vw, 40px);
  line-height: 1.08;
  font-weight: 800;
  overflow: hidden;
  text-overflow: ellipsis;
}

.credit-row.highlight {
  color: var(--lp-accent);
}

.credit-row.highlight .credit-value {
  color: var(--lp-text);
}

.credit-row.normal {
  color: var(--lp-text-sub);
}

.credit-row.muted {
  color: var(--lp-text-muted);
}

.credit-row.dimmed {
  color: var(--lp-text-muted);
  opacity: 0.5;
}

@media (max-width: 980px) {
  .lyric-panel {
    padding: 18px 18px 10px;
  }

  .lyric-stage {
    grid-template-columns: 1fr;
    gap: 18px;
    align-items: start;
    justify-items: center;
  }

  .cover-frame {
    width: min(66vw, 286px);
  }

  .stage-info {
    width: 100%;
    height: auto;
    display: block;
    text-align: center;
  }

  .info-main {
    width: 100%;
    height: 46vh;
    margin-top: 18px;
  }

  .lyric-row {
    text-align: center;
    transform-origin: center center;
  }

  .credit-row {
    justify-content: center;
    white-space: normal;
  }
}

@media (max-width: 640px) {
  .main-title {
    font-size: 20px;
  }

  .lyric-main-base,
  .lyric-main-highlight {
    font-size: 16px;
  }

  .lyric-row.active .lyric-main-base,
  .lyric-row.active .lyric-main-highlight {
    font-size: 18px;
  }

  .credit-row {
    margin-top: 12px;
    gap: 6px;
  }

  .credit-label {
    font-size: 16px;
  }

  .credit-value {
    font-size: 20px;
  }
}
</style>

<template>
  <div
    ref="shell"
    class="desktop-lyric-shell"
    :class="{ locked: payload.settings.locked }"
    :style="surfaceStyle"
    @pointerenter="handleShellPointerEnter"
    @pointermove="handleShellPointerMove"
    @pointerleave="handleShellPointerLeave"
    @pointerdown="startDrag"
  >
    <div class="desktop-controls" :class="{ visible: controlsVisible && !payload.settings.locked }">
      <div class="control-chip">
        <span class="control-dot" :class="{ playing: payload.isPlaying && payload.hasTrack }" />
        <span class="control-label">{{ controlLabel }}</span>
      </div>

      <div class="control-actions">
        <button
          type="button"
          class="control-btn"
          :class="{ active: payload.settings.alwaysOnTop }"
          :title="payload.settings.alwaysOnTop ? '取消置顶' : '置顶桌面歌词'"
          @pointerdown.stop
          @click="toggleAlwaysOnTop"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M8 4h8" />
            <path d="m10 4 1 7" />
            <path d="m14 4-1 7" />
            <path d="M6 11h12" />
            <path d="M12 11v9" />
          </svg>
        </button>

        <button
          type="button"
          class="control-btn control-btn--danger"
          title="关闭桌面歌词"
          @pointerdown.stop
          @click="closeWindow"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
          </svg>
        </button>
      </div>
    </div>

    <div class="desktop-surface" :class="{ idle: !payload.hasTrack }">
      <div class="surface-body" :class="{ compact: !secondaryLine }">
        <LyricMarqueeText
          class="primary-line"
          :text="primaryLine"
          :active="true"
          :progress="primaryProgress"
          :speed="payload.settings.scrollSpeed"
          scroll-mode="progress"
        />

        <LyricMarqueeText
          v-if="secondaryLine"
          class="secondary-line"
          :text="secondaryLine"
          :active="false"
          scroll-mode="never"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { emitTo } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import LyricMarqueeText from '@/components/LyricMarqueeText.vue';
import { clampLyricProgress } from '@/utils/lyrics';
import {
  DESKTOP_LYRIC_ACTION_EVENT,
  DESKTOP_LYRIC_CLOSED_EVENT,
  DESKTOP_LYRIC_MOVED_EVENT,
  DESKTOP_LYRIC_READY_EVENT,
  DESKTOP_LYRIC_STATE_EVENT,
  DEFAULT_DESKTOP_LYRIC_SETTINGS,
  type DesktopLyricActionPayload,
  type DesktopLyricStatePayload,
  type DesktopLyricWindowPosition,
} from '@/utils/desktopLyric';

const currentWindow = getCurrentWindow();
const HEX_COLOR_PATTERN = /^#(?:[0-9a-f]{3}|[0-9a-f]{6})$/i;

function colorWithAlpha(color: string, alpha: number, fallback: string) {
  const normalized = color.trim();
  if (!HEX_COLOR_PATTERN.test(normalized)) return fallback;

  const hex = normalized.length === 4
    ? normalized
      .slice(1)
      .split('')
      .map((segment) => `${segment}${segment}`)
      .join('')
    : normalized.slice(1);

  const red = Number.parseInt(hex.slice(0, 2), 16);
  const green = Number.parseInt(hex.slice(2, 4), 16);
  const blue = Number.parseInt(hex.slice(4, 6), 16);
  const clampedAlpha = Math.min(1, Math.max(0, alpha));

  return `rgba(${red}, ${green}, ${blue}, ${clampedAlpha.toFixed(3)})`;
}

const shell = ref<HTMLElement | null>(null);
const payload = ref<DesktopLyricStatePayload>({
  trackTitle: '',
  trackArtist: '',
  coverUrl: null,
  playbackTime: 0,
  playbackUpdatedAt: 0,
  currentLine: '',
  currentLineProgress: null,
  currentLineTime: null,
  currentLineDuration: null,
  translatedLine: '',
  nextLine: '',
  hasTrack: false,
  hasLyric: false,
  isPlaying: false,
  settings: DEFAULT_DESKTOP_LYRIC_SETTINGS,
});
const controlsVisible = ref(false);
const visualTime = ref(0);

let unlistenState: null | (() => void) = null;
let unlistenClose: null | (() => void) = null;
let unlistenMoved: null | (() => void) = null;
let moveTimer: number | null = null;
let controlsHideTimer: number | null = null;
let latestPosition: DesktopLyricWindowPosition | null = null;
let closingWindow = false;
let visualTimeRaf = 0;
let lastFrameMs = 0;

const primaryLine = computed(() => {
  if (payload.value.currentLine) return payload.value.currentLine;
  if (payload.value.nextLine) return payload.value.nextLine;
  if (payload.value.hasTrack) {
    return payload.value.hasLyric ? '前奏...' : '当前歌曲暂无同步歌词';
  }
  return '桌面歌词';
});

const secondaryLine = computed(() => {
  if (!payload.value.hasTrack) {
    return '在主窗口点击右下角歌词按钮打开';
  }

  if (!payload.value.currentLine) return '';

  if (payload.value.settings.showTranslation) {
    return payload.value.translatedLine || payload.value.nextLine;
  }

  return payload.value.nextLine || payload.value.translatedLine;
});

const targetVisualTime = computed(() => payload.value.playbackTime);

const primaryProgress = computed(() => {
  if (
    typeof payload.value.currentLineTime === 'number'
    && typeof payload.value.currentLineDuration === 'number'
    && payload.value.currentLineDuration > 0
  ) {
    return Math.min(
      100,
      Math.max(
        0,
        clampLyricProgress(
          visualTime.value,
          payload.value.currentLineTime,
          payload.value.currentLineDuration,
        ) * 100,
      ),
    );
  }

  if (typeof payload.value.currentLineProgress !== 'number') return null;
  return Math.min(100, Math.max(0, payload.value.currentLineProgress * 100));
});

const controlLabel = computed(() => {
  if (!payload.value.hasTrack) return '桌面歌词';
  return payload.value.trackTitle || '当前播放';
});

const surfaceStyle = computed(() => ({
  '--desktop-font-scale': payload.value.settings.fontScale.toFixed(2),
  '--desktop-chip-alpha': Math.max(0.14, payload.value.settings.backgroundOpacity * 0.6).toFixed(2),
  '--desktop-text-color': payload.value.settings.baseColor,
  '--desktop-text-soft': colorWithAlpha(
    payload.value.settings.baseColor,
    0.9,
    'rgba(255, 255, 255, 0.9)',
  ),
  '--desktop-text-muted': colorWithAlpha(
    payload.value.settings.baseColor,
    0.68,
    'rgba(255, 255, 255, 0.68)',
  ),
  '--desktop-text-dim': colorWithAlpha(
    payload.value.settings.baseColor,
    0.26,
    'rgba(255, 255, 255, 0.26)',
  ),
  '--desktop-highlight-color': payload.value.settings.highlightColor,
  '--desktop-highlight-glow': colorWithAlpha(
    payload.value.settings.highlightColor,
    0.12,
    'rgba(120, 240, 203, 0.12)',
  ),
  '--desktop-highlight-border': colorWithAlpha(
    payload.value.settings.highlightColor,
    0.32,
    'rgba(120, 240, 203, 0.32)',
  ),
  '--desktop-highlight-surface': colorWithAlpha(
    payload.value.settings.highlightColor,
    0.18,
    'rgba(21, 61, 50, 0.94)',
  ),
}));

function clearMoveTimer() {
  if (moveTimer !== null) {
    window.clearTimeout(moveTimer);
    moveTimer = null;
  }
}

function clearControlsHideTimer() {
  if (controlsHideTimer !== null) {
    window.clearTimeout(controlsHideTimer);
    controlsHideTimer = null;
  }
}

function flushMovedPosition() {
  clearMoveTimer();
  if (!latestPosition) return;
  void emitTo('main', DESKTOP_LYRIC_MOVED_EVENT, latestPosition);
}

function scheduleMovedPosition(position: DesktopLyricWindowPosition) {
  latestPosition = {
    x: Math.round(position.x),
    y: Math.round(position.y),
  };

  clearMoveTimer();
  moveTimer = window.setTimeout(() => {
    moveTimer = null;
    if (!latestPosition) return;
    void emitTo('main', DESKTOP_LYRIC_MOVED_EVENT, latestPosition);
  }, 160);
}

function showControls(hideDelay: number | null = null) {
  if (payload.value.settings.locked) return;

  clearControlsHideTimer();
  controlsVisible.value = true;

  if (hideDelay === null || hideDelay <= 0) return;

  controlsHideTimer = window.setTimeout(() => {
    controlsVisible.value = false;
    controlsHideTimer = null;
  }, hideDelay);
}

function hideControls(delay = 180) {
  clearControlsHideTimer();

  if (payload.value.settings.locked) {
    controlsVisible.value = false;
    return;
  }

  controlsHideTimer = window.setTimeout(() => {
    controlsVisible.value = false;
    controlsHideTimer = null;
  }, delay);
}

function handleShellPointerEnter() {
  showControls();
}

function handleShellPointerMove() {
  if (payload.value.settings.locked) return;
  showControls();
}

function handleShellPointerLeave() {
  hideControls(120);
}

function stopVisualTimeLoop() {
  if (!visualTimeRaf) return;
  window.cancelAnimationFrame(visualTimeRaf);
  visualTimeRaf = 0;
}

function startVisualTimeLoop() {
  if (visualTimeRaf) return;
  visualTime.value = targetVisualTime.value;
  lastFrameMs = performance.now();

  const tick = (now: number) => {
    const dt = Math.min(0.08, Math.max(0, (now - lastFrameMs) / 1000));
    lastFrameMs = now;

    const target = targetVisualTime.value;
    if (payload.value.isPlaying) {
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

async function startDrag(event: PointerEvent) {
  if (payload.value.settings.locked) return;
  if (event.button !== 0) return;

  const target = event.target as HTMLElement | null;
  if (target?.closest('button')) return;

  await currentWindow.startDragging();
}

async function toggleAlwaysOnTop() {
  const nextPayload: DesktopLyricActionPayload = {
    type: 'toggle-always-on-top',
  };

  showControls();
  await emitTo('main', DESKTOP_LYRIC_ACTION_EVENT, nextPayload).catch((error) => {
    console.error('desktop lyric action emit failed', error);
  });
}

async function closeWindow() {
  if (closingWindow) return;

  closingWindow = true;
  flushMovedPosition();
  clearControlsHideTimer();
  controlsVisible.value = false;
  await currentWindow.destroy().catch((error) => {
    console.error('desktop lyric destroy failed', error);
  });
  await emitTo('main', DESKTOP_LYRIC_CLOSED_EVENT).catch((error) => {
    console.error('desktop lyric close notify failed', error);
  });
  closingWindow = false;
}

watch(
  () => payload.value.settings.locked,
  (locked) => {
    if (locked) {
      clearControlsHideTimer();
      controlsVisible.value = false;
      return;
    }

    showControls(1200);
  },
);

watch(
  targetVisualTime,
  (value) => {
    if (!payload.value.isPlaying) {
      visualTime.value = value;
    }
  },
);

onMounted(async () => {
  startVisualTimeLoop();
  unlistenState = await currentWindow.listen<DesktopLyricStatePayload>(
    DESKTOP_LYRIC_STATE_EVENT,
    (event) => {
      payload.value = event.payload;
    },
  );

  unlistenClose = await currentWindow.onCloseRequested((event) => {
    event.preventDefault();
    void closeWindow();
  });

  unlistenMoved = await currentWindow.onMoved(({ payload: position }) => {
    scheduleMovedPosition(position);
  });

  if (!payload.value.settings.locked) {
    showControls(1200);
  }

  await emitTo('main', DESKTOP_LYRIC_READY_EVENT);
});

onBeforeUnmount(() => {
  stopVisualTimeLoop();
  flushMovedPosition();
  clearControlsHideTimer();
  unlistenState?.();
  unlistenClose?.();
  unlistenMoved?.();
});
</script>

<style scoped>
.desktop-lyric-shell {
  position: relative;
  width: 100%;
  height: 100%;
  padding: 10px 18px 14px;
  overflow: hidden;
  background: transparent;
  isolation: isolate;
}

.desktop-lyric-shell.locked {
  cursor: default;
}

.desktop-controls {
  position: absolute;
  inset: 10px 18px auto;
  z-index: 3;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 10px;
  border-radius: 18px;
  background: rgba(66, 72, 72, 0.78);
  border: 1px solid rgba(255, 255, 255, 0.14);
  box-shadow: 0 10px 22px rgba(0, 0, 0, 0.16);
  opacity: 0;
  transform: translateY(-12px);
  pointer-events: none;
  transition: opacity 0.14s ease, transform 0.14s ease;
}

.desktop-controls.visible {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

.control-chip,
.control-btn {
  background: rgba(255, 255, 255, var(--desktop-chip-alpha, 0.18));
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.control-chip {
  min-width: 0;
  max-width: min(48%, 320px);
  height: 34px;
  padding: 0 12px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: rgba(255, 255, 255, 0.84);
}

.control-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  flex-shrink: 0;
  background: var(--desktop-text-dim, rgba(255, 255, 255, 0.26));
}

.control-dot.playing {
  background: var(--desktop-highlight-color, #78f0cb);
  box-shadow: 0 0 0 4px var(--desktop-highlight-glow, rgba(120, 240, 203, 0.12));
}

.control-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.03em;
}

.control-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.control-btn {
  width: 34px;
  height: 34px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.7);
  transition:
    background 0.18s ease,
    color 0.18s ease,
    border-color 0.18s ease,
    transform 0.18s ease;
}

.control-btn:hover {
  color: #fff;
  border-color: rgba(255, 255, 255, 0.18);
  background: rgba(86, 94, 94, 0.92);
  transform: translateY(-1px);
}

.control-btn.active {
  color: var(--desktop-highlight-color, #7ef3cf);
  border-color: var(--desktop-highlight-border, rgba(120, 240, 203, 0.32));
  background: var(--desktop-highlight-surface, rgba(21, 61, 50, 0.94));
}

.control-btn--danger:hover {
  color: #fff;
  background: rgba(106, 40, 40, 0.94);
  border-color: rgba(248, 113, 113, 0.32);
}

.desktop-surface {
  position: relative;
  width: 100%;
  height: 100%;
  padding: 26px 8px 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--desktop-text-color, #ffffff);
}

.desktop-surface.idle {
  color: var(--desktop-text-soft, rgba(255, 255, 255, 0.96));
}

.surface-body {
  position: relative;
  z-index: 1;
}

.surface-body {
  width: min(100%, 760px);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.surface-body.compact {
  gap: 6px;
}

.primary-line,
.secondary-line {
  width: 100%;
  max-width: 100%;
  min-width: 0;
}

.primary-line {
  --marquee-highlight-color: var(--desktop-highlight-color, #16d6a0);
  font-size: calc(36px * var(--desktop-font-scale));
  line-height: 1.08;
  font-weight: 900;
  letter-spacing: 0.02em;
}

.secondary-line {
  font-size: calc(16px * var(--desktop-font-scale));
  line-height: 1.2;
  font-weight: 600;
  color: var(--desktop-text-soft, rgba(255, 255, 255, 0.9));
}

.primary-line :deep(.lyric-marquee:not(.overflowing) .lyric-marquee-layer),
.secondary-line :deep(.lyric-marquee:not(.overflowing) .lyric-marquee-layer) {
  text-align: center;
}

.primary-line :deep(.lyric-marquee.overflowing .lyric-marquee-layer),
.secondary-line :deep(.lyric-marquee.overflowing .lyric-marquee-layer) {
  text-align: left;
}

.primary-line :deep(.lyric-marquee-layer .lyric-marquee-copy) {
  color: var(--desktop-text-color, rgba(255, 255, 255, 0.98));
  text-shadow: none;
}

.primary-line :deep(.lyric-marquee-layer--highlight .lyric-marquee-copy) {
  color: var(--desktop-highlight-color, #16d6a0);
  text-shadow: none;
}

.secondary-line :deep(.lyric-marquee-copy) {
  color: inherit;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
}

@media (max-width: 760px) {
  .desktop-lyric-shell {
    padding: 10px 12px 12px;
  }

  .desktop-controls {
    inset: 10px 12px auto;
  }

  .control-chip {
    max-width: 56%;
  }

  .primary-line {
    font-size: calc(28px * var(--desktop-font-scale));
  }

  .secondary-line {
    font-size: calc(14px * var(--desktop-font-scale));
  }
}
</style>

<template>
  <span
    ref="viewport"
    class="lyric-marquee"
    :class="rootClass"
    :style="rootStyle"
  >
    <span ref="measure" class="lyric-marquee-measure">{{ safeText }}</span>

    <template v-if="hasHighlight">
      <span class="lyric-marquee-layer">
        <span :key="`base-${trackKey}`" class="lyric-marquee-track" :style="trackStyle">
          <span
            v-for="copyIndex in copyCount"
            :key="`base-${copyIndex}`"
            class="lyric-marquee-copy"
            :aria-hidden="shouldScroll && copyIndex > 1 ? 'true' : undefined"
          >
            {{ safeText }}
          </span>
        </span>
      </span>

      <span class="lyric-marquee-layer lyric-marquee-layer--highlight" aria-hidden="true">
        <span :key="`highlight-${trackKey}`" class="lyric-marquee-track" :style="trackStyle">
          <span
            v-for="copyIndex in copyCount"
            :key="`highlight-${copyIndex}`"
            class="lyric-marquee-copy-mask"
          >
            <span class="lyric-marquee-copy">{{ safeText }}</span>
          </span>
        </span>
      </span>
    </template>

    <template v-else>
      <span class="lyric-marquee-layer">
        <span :key="`plain-${trackKey}`" class="lyric-marquee-track" :style="trackStyle">
          <span
            v-for="copyIndex in copyCount"
            :key="`plain-${copyIndex}`"
            class="lyric-marquee-copy"
            :aria-hidden="shouldScroll && copyIndex > 1 ? 'true' : undefined"
          >
            {{ safeText }}
          </span>
        </span>
      </span>
    </template>
  </span>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';

const props = withDefaults(defineProps<{
  text: string;
  active?: boolean;
  progress?: number | null;
  speed?: number;
  gap?: number;
  scrollMode?: 'loop' | 'once' | 'progress' | 'never';
}>(), {
  active: false,
  progress: null,
  speed: 72,
  gap: 44,
  scrollMode: 'loop',
});

const viewport = ref<HTMLElement | null>(null);
const measure = ref<HTMLElement | null>(null);
const viewportWidth = ref(0);
const textWidth = ref(0);
const animationSeed = ref(0);

let resizeObserver: ResizeObserver | null = null;
let measureFrame = 0;
let lastShouldScroll = false;
const PROGRESS_SCROLL_START = 55;

const safeText = computed(() => props.text || '');
const hasHighlight = computed(() => typeof props.progress === 'number');
const overflowDistance = computed(() => Math.max(0, textWidth.value - viewportWidth.value));
const hasOverflow = computed(() => overflowDistance.value > 8);
const shouldScroll = computed(() => (
  props.scrollMode !== 'never'
  && props.active
  && !!safeText.value
  && hasOverflow.value
));
const isLoopScroll = computed(() => props.scrollMode === 'loop');
const isProgressScroll = computed(() => props.scrollMode === 'progress');
const rootClass = computed(() => ({
  overflowing: hasOverflow.value,
  'scrolling-loop': shouldScroll.value && isLoopScroll.value,
  'scrolling-once': shouldScroll.value && props.scrollMode === 'once',
  'scrolling-progress': shouldScroll.value && isProgressScroll.value,
}));
const copyCount = computed(() => (
  shouldScroll.value && isLoopScroll.value ? 2 : 1
));
const scrollDistance = computed(() => Math.ceil(
  isLoopScroll.value ? textWidth.value + props.gap : overflowDistance.value,
));
const duration = computed(() => (
  isLoopScroll.value
    ? Math.max(4.8, scrollDistance.value / Math.max(1, props.speed) + 1.1)
    : Math.max(2.8, scrollDistance.value / Math.max(1, props.speed) + 1.4)
));
const progressValue = computed(() => {
  if (typeof props.progress !== 'number' || !Number.isFinite(props.progress)) return 0;
  return Math.min(100, Math.max(0, props.progress));
});
const highlightWidth = computed(() => (progressValue.value / 100) * textWidth.value);
const progressScrollRatio = computed(() => {
  if (!shouldScroll.value || !isProgressScroll.value || !hasHighlight.value) return 0;
  return Math.min(
    1,
    Math.max(0, (progressValue.value - PROGRESS_SCROLL_START) / (100 - PROGRESS_SCROLL_START)),
  );
});
const progressScrollOffset = computed(() => {
  if (!shouldScroll.value || !isProgressScroll.value || !hasHighlight.value) return 0;
  return overflowDistance.value * progressScrollRatio.value;
});
const trackKey = computed(() => `${props.scrollMode}-${animationSeed.value}`);

const rootStyle = computed(() => ({
  '--marquee-gap': shouldScroll.value && isLoopScroll.value ? `${props.gap}px` : '0px',
  '--marquee-highlight-width': `${highlightWidth.value.toFixed(2)}px`,
}));

const trackStyle = computed(() => (
  shouldScroll.value
    ? {
        '--marquee-distance': `${scrollDistance.value}px`,
        '--marquee-duration': `${duration.value}s`,
        transform: isProgressScroll.value
          ? `translateX(-${progressScrollOffset.value.toFixed(2)}px)`
          : undefined,
      }
    : undefined
));

function restartOneShotAnimation() {
  if (props.scrollMode !== 'once') return;
  animationSeed.value += 1;
}

function updateMetrics() {
  viewportWidth.value = viewport.value?.clientWidth ?? 0;
  textWidth.value = measure.value?.scrollWidth ?? 0;

  const nextShouldScroll = shouldScroll.value;
  if (props.scrollMode === 'once' && nextShouldScroll && nextShouldScroll !== lastShouldScroll) {
    restartOneShotAnimation();
  }
  lastShouldScroll = nextShouldScroll;
}

function scheduleMeasure() {
  if (measureFrame) {
    window.cancelAnimationFrame(measureFrame);
  }
  measureFrame = window.requestAnimationFrame(() => {
    measureFrame = 0;
    updateMetrics();
  });
}

watch(
  () => [props.text, props.active, props.speed, props.gap, props.scrollMode],
  (current, previous) => {
    scheduleMeasure();

    if (
      props.scrollMode === 'once'
      && previous
      && current.some((value, index) => value !== previous[index])
    ) {
      restartOneShotAnimation();
    }
  },
  { flush: 'post' },
);

onMounted(() => {
  updateMetrics();
  lastShouldScroll = shouldScroll.value;

  if (typeof ResizeObserver === 'undefined') return;

  resizeObserver = new ResizeObserver(() => {
    updateMetrics();
  });

  if (viewport.value) {
    resizeObserver.observe(viewport.value);
  }
  if (measure.value) {
    resizeObserver.observe(measure.value);
  }
});

onBeforeUnmount(() => {
  if (measureFrame) {
    window.cancelAnimationFrame(measureFrame);
  }
  resizeObserver?.disconnect();
  resizeObserver = null;
});
</script>

<style scoped>
.lyric-marquee {
  position: relative;
  display: block;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
}

.lyric-marquee-layer {
  display: block;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  overflow: hidden;
}

.lyric-marquee-layer--highlight {
  position: absolute;
  inset: 0 auto 0 0;
  pointer-events: none;
  color: var(--marquee-highlight-color, currentColor);
}

.lyric-marquee-track {
  display: inline-flex;
  align-items: center;
  column-gap: var(--marquee-gap, 0px);
  min-width: max-content;
  transform: translateX(0);
  will-change: transform;
}

.lyric-marquee.scrolling-loop .lyric-marquee-track {
  animation: lyric-marquee-loop var(--marquee-duration) linear infinite;
}

.lyric-marquee.scrolling-once .lyric-marquee-track {
  animation: lyric-marquee-once var(--marquee-duration) linear 1 forwards;
}

.lyric-marquee.scrolling-progress .lyric-marquee-track {
  transition: transform 0.14s linear;
}

.lyric-marquee-copy {
  display: inline-block;
  flex: 0 0 auto;
  white-space: nowrap;
}

.lyric-marquee-copy-mask {
  display: inline-block;
  width: var(--marquee-highlight-width, 0px);
  flex: 0 0 auto;
  overflow: hidden;
  white-space: nowrap;
}

.lyric-marquee-measure {
  position: absolute;
  inset-inline-start: 0;
  inset-block-start: 0;
  font: inherit;
  letter-spacing: inherit;
  visibility: hidden;
  pointer-events: none;
  white-space: nowrap;
}

@keyframes lyric-marquee-loop {
  0%,
  12% {
    transform: translateX(0);
  }

  100% {
    transform: translateX(calc(-1 * var(--marquee-distance, 0px)));
  }
}

@keyframes lyric-marquee-once {
  0%,
  18% {
    transform: translateX(0);
  }

  100% {
    transform: translateX(calc(-1 * var(--marquee-distance, 0px)));
  }
}
</style>

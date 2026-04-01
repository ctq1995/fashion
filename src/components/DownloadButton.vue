<template>
  <div class="download-control">
    <button
      ref="triggerRef"
      type="button"
      class="app-icon-btn download-btn"
      :class="statusClass"
      :disabled="downloading"
      :title="buttonTitle"
      @click.stop="toggleMenu"
    >
      <svg v-if="status === 'done'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m5 12 5 5L20 7" />
      </svg>
      <svg v-else-if="status === 'error'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 9v4" />
        <path d="M12 17h.01" />
        <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z" />
      </svg>
      <span v-else-if="downloading" class="spinner" />
      <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 3v12" />
        <path d="m7 10 5 5 5-5" />
        <path d="M5 21h14" />
      </svg>
    </button>

    <Teleport to="body">
      <Transition name="download-pop">
        <div v-if="menuOpen" ref="menuRef" class="download-menu" :style="menuStyle" @click.stop>
          <button
            v-for="option in bitrateOptions"
            :key="option.value"
            type="button"
            class="download-option"
            @click="startDownload(option.value)"
          >
            <span>{{ option.label }}</span>
            <small>{{ option.description }}</small>
          </button>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import type { Bitrate } from '@/stores/player';

interface DownloadTrack {
  id: string;
  name: string;
  artist: string;
  source: string;
}

const props = defineProps<{
  track: DownloadTrack;
}>();

const bitrateOptions: Array<{ value: Bitrate; label: string; description: string }> = [
  { value: 128, label: '128K', description: '标准' },
  { value: 192, label: '192K', description: '较高' },
  { value: 320, label: '320K', description: '高品质' },
  { value: 999, label: '无损', description: 'VIP/资源允许时' },
];

const triggerRef = ref<HTMLElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const menuOpen = ref(false);
const downloading = ref(false);
const status = ref<'idle' | 'done' | 'error'>('idle');
const menuStyle = ref<Record<string, string>>({});
let resetTimer: number | null = null;

const buttonTitle = computed(() => {
  if (downloading.value) return '下载中';
  if (status.value === 'done') return '已保存到下载目录';
  if (status.value === 'error') return '下载失败，请重试';
  return '下载';
});

const statusClass = computed(() => ({
  success: status.value === 'done',
  error: status.value === 'error',
}));

function clearResetTimer() {
  if (resetTimer !== null) {
    window.clearTimeout(resetTimer);
    resetTimer = null;
  }
}

function scheduleReset() {
  clearResetTimer();
  resetTimer = window.setTimeout(() => {
    status.value = 'idle';
    resetTimer = null;
  }, 2200);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function updateMenuPosition() {
  if (!triggerRef.value || !menuRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  const width = menuRef.value.offsetWidth || 156;
  const height = menuRef.value.offsetHeight || 196;

  menuStyle.value = {
    left: `${clamp(rect.right - width, 10, window.innerWidth - width - 10)}px`,
    top: `${clamp(rect.bottom + 10, 10, window.innerHeight - height - 10)}px`,
  };
}

async function toggleMenu() {
  if (downloading.value) return;
  status.value = 'idle';
  menuOpen.value = !menuOpen.value;
  if (!menuOpen.value) return;
  await nextTick();
  updateMenuPosition();
}

function closeMenu() {
  menuOpen.value = false;
}

async function startDownload(bitrate: Bitrate) {
  downloading.value = true;
  closeMenu();
  clearResetTimer();

  try {
    await invoke<string>('download_music', {
      source: props.track.source,
      id: props.track.id,
      bitrate,
      title: props.track.name,
      artist: props.track.artist,
    });
    status.value = 'done';
  } catch (error) {
    console.error('download_music failed', error);
    status.value = 'error';
  } finally {
    downloading.value = false;
    scheduleReset();
  }
}

function handlePointerDown(event: PointerEvent) {
  if (!menuOpen.value) return;
  const target = event.target as Node | null;
  const clickedTrigger = triggerRef.value?.contains(target) ?? false;
  const clickedMenu = menuRef.value?.contains(target) ?? false;
  if (!clickedTrigger && !clickedMenu) {
    closeMenu();
  }
}

watch(menuOpen, async (open) => {
  if (!open) return;
  await nextTick();
  updateMenuPosition();
});

onBeforeUnmount(() => {
  clearResetTimer();
  window.removeEventListener('pointerdown', handlePointerDown);
  window.removeEventListener('resize', updateMenuPosition);
});

watch(
  menuOpen,
  (open) => {
    if (open) {
      window.addEventListener('pointerdown', handlePointerDown);
      window.addEventListener('resize', updateMenuPosition);
      return;
    }

    window.removeEventListener('pointerdown', handlePointerDown);
    window.removeEventListener('resize', updateMenuPosition);
  },
  { immediate: true },
);
</script>

<style scoped>
.download-control {
  display: inline-flex;
}

.download-btn.success {
  color: var(--success);
}

.download-btn.error {
  color: var(--text-danger);
}

.spinner {
  width: 18px;
  height: 18px;
  border-radius: 999px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  animation: download-spin 0.7s linear infinite;
}

@keyframes download-spin {
  to {
    transform: rotate(360deg);
  }
}

.download-menu {
  position: fixed;
  width: 164px;
  padding: 8px;
  border-radius: 18px;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  box-shadow: var(--window-shadow);
  z-index: 2300;
}

.download-option {
  width: 100%;
  padding: 10px 12px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  color: var(--text-secondary);
  transition: var(--transition);
}

.download-option:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.download-option small {
  font-size: 11px;
  color: var(--text-muted);
}

.download-pop-enter-active,
.download-pop-leave-active {
  transition: opacity 0.14s ease, transform 0.14s ease;
}

.download-pop-enter-from,
.download-pop-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>

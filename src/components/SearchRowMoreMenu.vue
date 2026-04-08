<template>
  <div class="search-row-more">
    <button ref="triggerRef" type="button" class="app-icon-btn" title="更多操作" @click.stop="open = !open">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
        <circle cx="5" cy="12" r="1.8" />
        <circle cx="12" cy="12" r="1.8" />
        <circle cx="19" cy="12" r="1.8" />
      </svg>
    </button>
    <div v-if="open" class="search-row-more__menu" @click.stop>
      <button type="button" class="search-row-more__item" @click="emitAndClose('queue')">加入队列</button>
      <button type="button" class="search-row-more__item" @click="emitAndClose('playlist')">加入歌单</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits<{
  queue: [];
  playlist: [anchor: HTMLElement | null];
}>();

const open = ref(false);
const triggerRef = ref<HTMLElement | null>(null);

function emitAndClose(type: 'queue' | 'playlist') {
  open.value = false;
  if (type === 'playlist') {
    emit('playlist', triggerRef.value);
    return;
  }
  emit('queue');
}
</script>

<style scoped>
.search-row-more {
  position: relative;
  display: inline-flex;
}

.search-row-more__menu {
  position: absolute;
  right: 0;
  top: calc(100% + 8px);
  min-width: 120px;
  padding: 6px;
  border-radius: 12px;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  box-shadow: var(--window-shadow);
  z-index: 30;
}

.search-row-more__item {
  width: 100%;
  padding: 8px 10px;
  text-align: left;
  border-radius: 10px;
  color: var(--text-secondary);
}

.search-row-more__item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>

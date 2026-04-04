<template>
  <nav class="mobile-tabbar">
    <button
      v-for="item in items"
      :key="item.key"
      type="button"
      class="tab-item"
      :class="{ active: activeKey === item.key }"
      @click="emit('navigate', item.target)"
    >
      <span class="tab-icon" v-html="item.icon" />
      <span class="tab-label">{{ item.label }}</span>
    </button>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { strokeIcon } from '@/utils/iconography';

const props = defineProps<{
  activePanel: string;
}>();

const emit = defineEmits<{
  navigate: [target: string];
}>();

const activeKey = computed(() => {
  if (props.activePanel === 'search') return 'search';
  return props.activePanel;
});

const items = [
  {
    key: 'search',
    target: 'recommend',
    label: '搜索',
    icon: strokeIcon('<circle cx="11" cy="11" r="7"/><path d="m21 21-4.35-4.35"/>', { size: 20, strokeWidth: 1.9 }),
  },
  {
    key: 'favorites',
    target: 'favorites',
    label: '收藏',
    icon: strokeIcon('<path d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z"/>', { size: 20, strokeWidth: 1.9 }),
  },
  {
    key: 'history',
    target: 'history',
    label: '历史',
    icon: strokeIcon('<path d="M3 12a9 9 0 1 0 3-6.7"/><path d="M3 3v6h6"/><path d="M12 7v5l3 3"/>', { size: 20, strokeWidth: 1.9 }),
  },
  {
    key: 'lyric',
    target: 'lyric',
    label: '歌词',
    icon: strokeIcon('<path d="M4 7h16"/><path d="M4 12h10"/><path d="M4 17h16"/>', { size: 20, strokeWidth: 1.9 }),
  },
  {
    key: 'settings',
    target: 'settings',
    label: '设置',
    icon: strokeIcon('<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06A1.65 1.65 0 0 0 15 19.4a1.65 1.65 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82L4.21 7.1a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.09A1.65 1.65 0 0 0 15 4.6a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.5 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>', { size: 20, strokeWidth: 1.8 }),
  },
];
</script>

<style scoped>
.mobile-tabbar {
  padding: 10px 10px calc(12px + env(safe-area-inset-bottom, 0px));
  border-radius: 24px;
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 6px;
  background: linear-gradient(180deg, var(--panel-strong), rgba(255, 255, 255, 0.02));
  border: 1px solid var(--border);
  box-shadow: 0 16px 34px rgba(13, 25, 24, 0.14);
  backdrop-filter: blur(18px);
}

.tab-item {
  min-width: 0;
  min-height: 62px;
  padding: 8px 4px;
  border-radius: 18px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: var(--text-muted);
  border: 1px solid transparent;
  transition: var(--transition);
}

.tab-item:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
  border-color: var(--border);
}

.tab-item.active {
  color: var(--accent);
  background: var(--accent-dim);
  border-color: color-mix(in srgb, var(--accent) 28%, var(--border));
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.tab-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
}

.tab-label {
  min-width: 0;
  font-size: 11px;
  font-weight: 700;
  line-height: 1;
}
</style>

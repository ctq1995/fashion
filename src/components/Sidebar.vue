<template>
  <aside class="sidebar" :class="scene">
    <div class="brand">
      <div class="brand-mark">
        <img :src="brandIcon" alt="Fashion" />
      </div>
      <div class="brand-copy">
        <strong>Fashion</strong>
        <span>Music</span>
      </div>
    </div>

    <div class="nav-group">
      <span class="group-title">发现</span>
      <nav class="nav-list">
        <button
          v-for="item in primaryItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: active === item.key }"
          :title="item.label"
          @click="emit('update:active', item.key)"
        >
          <span class="nav-icon" v-html="item.icon" />
          <span class="nav-label">{{ item.label }}</span>
        </button>
      </nav>
    </div>

    <div class="nav-group music-group">
      <span class="group-title">音乐库</span>
      <nav class="nav-list">
        <button
          v-for="item in libraryItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: active === item.key }"
          :title="item.label"
          @click="emit('update:active', item.key)"
        >
          <span class="nav-icon" v-html="item.icon" />
          <span class="nav-label">{{ item.label }}</span>
          <span v-if="item.key === 'history' && historyCount" class="nav-badge">{{ historyCount }}</span>
        </button>
      </nav>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import brandIcon from '@/assets/fashion-brand.svg';
import { usePlayerStore } from '@/stores/player';
import { strokeIcon } from '@/utils/iconography';

defineProps<{
  active: string;
  scene: 'light' | 'dark';
}>();

const emit = defineEmits<{ 'update:active': [v: string] }>();
const player = usePlayerStore();

const historyCount = computed(() => Math.min(player.history.length, 99));
const navIcon = (paths: string) =>
  strokeIcon(paths, { size: 18, strokeWidth: 1.9, className: 'sidebar-icon-svg' });

const primaryItems = [
  {
    key: 'recommend',
    label: '搜索',
    icon: navIcon('<circle cx="11" cy="11" r="7"/><path d="m21 21-4.35-4.35"/>'),
  },
];

const libraryItems = [
  {
    key: 'favorites',
    label: '我的喜欢',
    icon: navIcon(
      '<path d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z"/>',
    ),
  },
  {
    key: 'local-library',
    label: '本地音乐',
    icon: navIcon('<path d="M12 3v12.55A4 4 0 1 0 14 19V7h5V3Z"/>'),
  },
  {
    key: 'history',
    label: '最近播放',
    icon: navIcon('<path d="M3 12a9 9 0 1 0 3-6.7"/><path d="M3 3v6h6"/><path d="M12 7v5l3 3"/>'),
  },
  {
    key: 'settings',
    label: '设置',
    icon: navIcon(
      '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06A1.65 1.65 0 0 0 15 19.4a1.65 1.65 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82L4.21 7.1a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.09A1.65 1.65 0 0 0 15 4.6a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.5 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
    ),
  },
];
</script>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 14px 12px 12px;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border);
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 8px 14px;
}

.brand-mark {
  width: 34px;
  height: 34px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 10px 22px rgba(22, 214, 160, 0.22);
}

.brand-mark img {
  width: 100%;
  height: 100%;
  display: block;
}

.brand-copy strong {
  display: block;
  font-size: 15px;
  line-height: 1;
  letter-spacing: 0.04em;
  color: var(--text-primary);
}

.brand-copy span {
  display: block;
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-muted);
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.music-group {
  margin-top: 8px;
}

.group-title {
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-muted);
}

.nav-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  height: 36px;
  padding: 0 12px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  transition: var(--transition);
  color: var(--text-secondary);
}

.nav-item:hover,
.nav-item.active {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 18px;
  height: 18px;
}

.nav-label {
  font-size: 13px;
  font-weight: 600;
}

.nav-badge {
  margin-left: auto;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: rgba(18, 216, 160, 0.2);
  color: #16b98d;
  font-size: 10px;
  font-weight: 700;
}
</style>

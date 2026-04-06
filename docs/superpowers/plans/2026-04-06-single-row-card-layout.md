# 共享歌曲行统一布局 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 以“我的喜欢”为母版抽出共享歌曲行组件，把 Favorites、Search、History、Playlist 四个模块统一到同一套歌曲行骨架，并把歌曲总长度固定放在按钮区前的右侧独立位。

**Architecture:** 新增一个只负责结构骨架的共享 Vue 组件，统一输出“序号/封面/标题与副标题/附加信息/总时长/按钮区”这套 DOM。Favorites 先接入作为基准实现，再让 Search、History、Playlist 逐个接入，通过 slot 保留各模块特有信息与按钮，不改本地音乐和业务 store。

**Tech Stack:** Vue 3 SFC、TypeScript、Pinia、Vite、Vitest

---

## 文件结构与职责

- `src/components/SharedSongRow.vue`
  - 新增共享歌曲行骨架组件；负责稳定 DOM 结构、总时长位、按钮区与响应式骨架。
- `src/components/FavoritesPanel.vue`
  - 作为母版接入共享组件，提供“我的喜欢”的标题、副标题、播放中状态与按钮。
- `src/components/SearchPanel.vue`
  - 接入共享组件，保留来源标签和搜索操作按钮。
- `src/components/HistoryPanel.vue`
  - 接入共享组件，保留历史附加信息，但总时长固定到右侧独立位。
- `src/components/PlaylistPanel.vue`
  - 在抽屉场景内接入共享组件，保留队列操作与当前播放状态。
- `src/components/SharedSongRowLayout.test.ts`
  - 新增原始源码结构测试，锁定共享组件与四个面板的接入形态。

## Task 1: 新增共享歌曲行结构测试

**Files:**
- Create: `src/components/SharedSongRowLayout.test.ts`

- [ ] **Step 1: 写共享结构的失败测试**

创建 `src/components/SharedSongRowLayout.test.ts`，先用 `?raw` 锁定以下结构：
- 新组件文件存在 `SharedSongRow`
- 共享组件包含 `row-duration`、`row-actions`、`extra` slot
- Favorites、Search、History、Playlist 都导入了 `SharedSongRow`

测试文件内容：

```ts
import { describe, expect, it } from 'vitest';
import favoritesSource from '@/components/FavoritesPanel.vue?raw';
import historySource from '@/components/HistoryPanel.vue?raw';
import playlistSource from '@/components/PlaylistPanel.vue?raw';
import searchSource from '@/components/SearchPanel.vue?raw';
import sharedSource from '@/components/SharedSongRow.vue?raw';

describe('shared song row layout', () => {
  it('defines a shared song row component with duration and action areas', () => {
    expect(sharedSource).toContain('name="SharedSongRow"');
    expect(sharedSource).toContain('class="row-duration"');
    expect(sharedSource).toContain('class="row-actions"');
    expect(sharedSource).toContain('name="extra"');
  });

  it('uses the shared song row in favorites', () => {
    expect(favoritesSource).toContain("from '@/components/SharedSongRow.vue'");
    expect(favoritesSource).toContain('<SharedSongRow');
  });

  it('uses the shared song row in search', () => {
    expect(searchSource).toContain("from '@/components/SharedSongRow.vue'");
    expect(searchSource).toContain('<SharedSongRow');
  });

  it('uses the shared song row in history', () => {
    expect(historySource).toContain("from '@/components/SharedSongRow.vue'");
    expect(historySource).toContain('<SharedSongRow');
  });

  it('uses the shared song row in playlist', () => {
    expect(playlistSource).toContain("from '@/components/SharedSongRow.vue'");
    expect(playlistSource).toContain('<SharedSongRow');
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: FAIL，因为 `SharedSongRow.vue` 尚不存在，四个面板也尚未接入。

## Task 2: 实现共享歌曲行组件

**Files:**
- Create: `src/components/SharedSongRow.vue`
- Test: `src/components/SharedSongRowLayout.test.ts`

- [ ] **Step 1: 新建共享组件骨架**

创建 `src/components/SharedSongRow.vue`，实现以下最小可复用接口：

```vue
<template>
  <div class="shared-song-row" :class="{ 'is-active': active }">
    <slot name="index" />

    <div class="row-cover">
      <slot name="cover" />
    </div>

    <div class="row-meta">
      <div class="row-head">
        <span class="row-title">{{ title }}</span>
        <span v-if="playingLabel" class="playing-tag">{{ playingLabel }}</span>
      </div>
      <span class="row-sub">{{ subtitle }}</span>
      <div v-if="$slots.extra" class="row-extra">
        <slot name="extra" />
      </div>
    </div>

    <span class="row-duration">{{ durationText }}</span>

    <div class="row-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'SharedSongRow' });

defineProps<{
  title: string;
  subtitle: string;
  durationText: string;
  playingLabel?: string;
  active?: boolean;
}>();
</script>
```

- [ ] **Step 2: 给共享组件补上最小样式**

在同文件 `style scoped` 中先实现稳定骨架：

```css
.shared-song-row {
  min-height: 58px;
  padding: 8px 10px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.02);
  display: grid;
  grid-template-columns: auto 42px minmax(0, 1fr) auto auto;
  gap: 10px;
  align-items: center;
  transition: var(--transition);
}

.row-cover {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  overflow: hidden;
  background: var(--bg-hover);
  border: 1px solid var(--border);
}

.row-meta {
  min-width: 0;
}

.row-head {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.row-title,
.row-sub,
.row-extra {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-duration {
  min-width: 48px;
  text-align: right;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-secondary);
}

.row-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
```

并补一段窄宽度降级样式，让 `row-meta` 优先压缩，`row-duration` 与 `row-actions` 保持稳定可见。

- [ ] **Step 3: 运行结构测试确认共享组件断言通过**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: 仍然 FAIL，但第一条关于 `SharedSongRow.vue` 的断言通过，其他四条面板接入断言继续失败。

## Task 3: 让 Favorites 接入共享组件并补上总时长

**Files:**
- Modify: `src/components/FavoritesPanel.vue`
- Test: `src/components/SharedSongRowLayout.test.ts`

- [ ] **Step 1: 写 Favorites 的时长与接入断言**

在 `src/components/SharedSongRowLayout.test.ts` 中追加 Favorites 细化断言：

```ts
it('renders favorites duration through shared song row props', () => {
  expect(favoritesSource).toContain('durationText="formatDuration(track.durationSec ?? null)"');
  expect(favoritesSource).toContain('playingLabel="播放中"');
});
```

- [ ] **Step 2: 运行测试确认新增断言失败**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: FAIL，因为 Favorites 还没把总时长和 props 接到共享组件上。

- [ ] **Step 3: 在 Favorites 中实现最小接入**

在 `src/components/FavoritesPanel.vue` 中：
- 引入 `SharedSongRow`
- 新增 `formatDuration(value: number | null)`
- 把每一行替换为共享组件
- 保留现有 `playAll`、`isCurrentTrack`、`DownloadButton` 和移除收藏逻辑

目标模板形态：

```vue
<SharedSongRow
  :title="track.name"
  :subtitle="`${track.artist} · ${track.album}`"
  :duration-text="formatDuration(track.durationSec ?? null)"
  :playing-label="isCurrentTrack(track) ? '播放中' : undefined"
>
  <template #index>
    <span class="row-index">{{ idx + 1 }}</span>
  </template>

  <template #cover>
    <img v-if="track.coverUrl" :src="track.coverUrl" />
    <div v-else class="row-cover-ph">...</div>
  </template>

  <template #actions>
    ...existing buttons...
  </template>
</SharedSongRow>
```

目标脚本形态：

```ts
function formatDuration(value: number | null) {
  if (!value || value <= 0) return '--:--';
  const hours = Math.floor(value / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const seconds = Math.floor(value % 60);
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
  }
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}
```

- [ ] **Step 4: 运行测试确认 Favorites 接入通过**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: Favorites 相关断言通过，Search / History / Playlist 相关断言继续失败。

## Task 4: 让 Search 接入共享组件

**Files:**
- Modify: `src/components/SearchPanel.vue`
- Test: `src/components/SharedSongRowLayout.test.ts`

- [ ] **Step 1: 写 Search 的失败断言**

在 `src/components/SharedSongRowLayout.test.ts` 中追加 Search 细化断言：

```ts
it('keeps search source tag in the extra slot and duration before actions', () => {
  expect(searchSource).toContain('<template #extra>');
  expect(searchSource).toContain('class="source-tag"');
  expect(searchSource).toContain('formatDuration(item.durationSec ?? null)');
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: FAIL，因为 Search 尚未使用 extra slot 和统一时长。

- [ ] **Step 3: 在 Search 中接入共享组件**

在 `src/components/SearchPanel.vue` 中：
- 引入 `SharedSongRow`
- 新增本地 `formatDuration`
- 保留 `sourceLabel`、收藏/加歌单/加队列/下载/播放逻辑
- 把结果行改为共享组件
- 让来源标签放入 `#extra`，按钮放入 `#actions`

目标模板形态：

```vue
<SharedSongRow
  :title="item.name"
  :subtitle="`${getArtistNames(item.artist)} · ${getAlbumName(item.album)}`"
  :duration-text="formatDuration(item.durationSec ?? null)"
  :active="isCurrentTrack(item)"
>
  <template #index>
    <span class="row-index">{{ idx + 1 }}</span>
  </template>

  <template #cover>
    <img v-if="media.getTrackCoverUrl(item)" :src="media.getTrackCoverUrl(item) ?? undefined" />
    <div v-else class="cover-ph">...</div>
  </template>

  <template #extra>
    <span class="source-tag">{{ sourceLabel(item.source) }}</span>
  </template>

  <template #actions>
    ...existing search action buttons...
  </template>
</SharedSongRow>
```

- [ ] **Step 4: 运行测试确认 Search 接入通过**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: Favorites 和 Search 相关断言通过，History / Playlist 相关断言继续失败。

## Task 5: 让 History 接入共享组件

**Files:**
- Modify: `src/components/HistoryPanel.vue`
- Test: `src/components/SharedSongRowLayout.test.ts`

- [ ] **Step 1: 写 History 的失败断言**

在 `src/components/SharedSongRowLayout.test.ts` 中追加 History 细化断言：

```ts
it('moves history meta into the extra slot while keeping duration on the right', () => {
  expect(historySource).toContain('<template #extra>');
  expect(historySource).toContain('fmtPlayedAt(item.playedAt)');
  expect(historySource).toContain('formatDuration(item.durationSnapshot || item.durationSec || null)');
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: FAIL，因为 History 还保留旧的 `row-side` 结构。

- [ ] **Step 3: 在 History 中接入共享组件**

在 `src/components/HistoryPanel.vue` 中：
- 引入 `SharedSongRow`
- 保留 `fmtPlayedAt`
- 新增 `formatDuration`
- 用共享组件替换旧的 `row-main + row-side + row-actions`
- 把历史附加信息放进 `#extra`
- 保留收藏 / 播放 / 下载 / 删除记录按钮

目标模板形态：

```vue
<SharedSongRow
  :title="item.name"
  :subtitle="`${item.artist} · ${item.album}`"
  :duration-text="formatDuration(item.durationSnapshot || item.durationSec || null)"
  :playing-label="player.currentTrack?.id === item.id && player.currentTrack?.source === item.source ? '播放中' : undefined"
>
  <template #index>
    <span class="row-index">{{ idx + 1 }}</span>
  </template>

  <template #cover>
    <img v-if="item.coverUrl" :src="item.coverUrl" />
    <div v-else class="row-cover-ph">...</div>
  </template>

  <template #extra>
    <span class="history-extra">{{ fmtPlayedAt(item.playedAt) }} · {{ item.completed ? '已播完' : '未播完' }}</span>
  </template>

  <template #actions>
    ...existing history action buttons...
  </template>
</SharedSongRow>
```

- [ ] **Step 4: 运行测试确认 History 接入通过**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: Favorites / Search / History 相关断言通过，Playlist 相关断言继续失败。

## Task 6: 让 Playlist 在抽屉中接入共享组件

**Files:**
- Modify: `src/components/PlaylistPanel.vue`
- Test: `src/components/SharedSongRowLayout.test.ts`

- [ ] **Step 1: 写 Playlist 的失败断言**

在 `src/components/SharedSongRowLayout.test.ts` 中追加 Playlist 细化断言：

```ts
it('uses shared song row inside the playlist drawer with a queue duration', () => {
  expect(playlistSource).toContain('formatDuration(track.durationSec ?? null)');
  expect(playlistSource).toContain("playingLabel: idx === player.currentIndex ? '正在播放' : undefined");
});
```

如果实现里不是对象字面量，可改为断言：

```ts
expect(playlistSource).toContain(':playing-label="idx === player.currentIndex ? \'正在播放\' : undefined"');
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: FAIL，因为 Playlist 还保留旧的 `drawer-*` 歌曲行骨架。

- [ ] **Step 3: 在 Playlist 中接入共享组件**

在 `src/components/PlaylistPanel.vue` 中：
- 引入 `SharedSongRow`
- 新增 `formatDuration`
- 保留抽屉外层结构 `playlist-drawer`、`drawer-head`、`drawer-list`
- 只替换每个队列项内部歌曲行骨架
- 按钮区保留“播放 / 移除”

目标模板形态：

```vue
<SharedSongRow
  :title="track.name"
  :subtitle="track.artist"
  :duration-text="formatDuration(track.durationSec ?? null)"
  :playing-label="idx === player.currentIndex ? '正在播放' : undefined"
  :active="idx === player.currentIndex"
>
  <template #cover>
    <img v-if="track.coverUrl" :src="track.coverUrl" :alt="track.name" />
    <div v-else class="drawer-cover-ph">...</div>
  </template>

  <template #actions>
    ...existing queue action buttons...
  </template>
</SharedSongRow>
```

并对 `drawer-list` 内部样式做最小调整，确保共享组件在抽屉宽度下不会溢出。

- [ ] **Step 4: 运行测试确认 Playlist 接入通过**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: `src/components/SharedSongRowLayout.test.ts` 全部 PASS。

## Task 7: 做完整验证并修正样式回归

**Files:**
- Modify: `src/components/SharedSongRow.vue`
- Modify: `src/components/FavoritesPanel.vue`
- Modify: `src/components/SearchPanel.vue`
- Modify: `src/components/HistoryPanel.vue`
- Modify: `src/components/PlaylistPanel.vue`
- Test: `src/components/SharedSongRowLayout.test.ts`

- [ ] **Step 1: 运行共享结构测试**

Run: `npm test -- src/components/SharedSongRowLayout.test.ts`

Expected: PASS

- [ ] **Step 2: 运行全量测试**

Run: `npm test`

Expected: PASS

- [ ] **Step 3: 运行构建检查**

Run: `npm run build`

Expected: PASS

- [ ] **Step 4: 如果构建或测试暴露类型/样式问题，做最小修复**

允许的修复范围：
- 调整 `SharedSongRow.vue` props 命名与 slot 结构
- 调整四个面板的导入与模板用法
- 修正抽屉场景和窄宽度下的 grid / flex 压缩问题
- 不把 LocalLibrary 卷入本次改动

可接受的最小样式补丁示例：

```css
.shared-song-row {
  grid-template-columns: auto 42px minmax(0, 1fr) minmax(44px, auto) auto;
}

@media (max-width: 760px) {
  .shared-song-row {
    grid-template-columns: auto 42px minmax(0, 1fr) auto;
  }

  .row-duration {
    order: 4;
  }

  .row-actions {
    order: 5;
  }

  .row-extra {
    white-space: normal;
  }
}
```

- [ ] **Step 5: 再次运行测试与构建确认通过**

Run: `npm test && npm run build`

Expected: 全部 PASS

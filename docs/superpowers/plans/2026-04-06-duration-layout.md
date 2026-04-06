# 歌曲总时长统一显示 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在本地音乐、我的喜欢、搜索结果、播放列表、最近播放五个核心面板中统一补齐歌曲总时长，并把本地音乐的时长位置调整得更紧凑。

**Architecture:** 继续沿用现有各面板独立模板与 scoped CSS 的实现方式，不新增全局组件或 store。每个面板仅在行结构中补一个稳定的右侧时长字段，并在已有右侧信息区中重排时长、状态标签和操作按钮，确保长文本优先压缩左侧区域。

**Tech Stack:** Vue 3 SFC、TypeScript、Pinia、Vite、Vitest

---

## 文件结构与职责

- `src/components/LocalLibraryPanel.vue`
  - 已有总时长展示；本次负责压缩时长与按钮区间距，保持本地音乐布局更紧凑。
- `src/components/FavoritesPanel.vue`
  - 当前无总时长；本次补充 `formatDuration` 并引入右侧 `row-side` 区域。
- `src/components/SearchPanel.vue`
  - 当前右侧只有来源标签和操作按钮；本次补充时长并调整为更稳定的右侧信息区。
- `src/components/PlaylistPanel.vue`
  - 当前队列项无总时长；本次补充 `formatDuration` 并在操作前显示时长。
- `src/components/HistoryPanel.vue`
  - 当前只有播放时间和进度状态；本次补充歌曲总时长，并整理为两行右侧信息。
- `src/components/LocalLibraryPanel.test.ts`
  - 已有原始源码结构回归测试；本次补充对本地音乐时长右侧结构的最小断言。
- `src/components/LocalDurationLayout.test.ts`
  - 新增原始源码测试，覆盖收藏/搜索/播放列表/最近播放的时长结构存在性。

## Task 1: 调整本地音乐时长位置

**Files:**
- Modify: `src/components/LocalLibraryPanel.vue`
- Test: `src/components/LocalLibraryPanel.test.ts`

- [ ] **Step 1: 先补一个会失败的结构断言**

在 `src/components/LocalLibraryPanel.test.ts` 里追加断言，要求本地音乐右侧区域同时存在时长与操作区，并且使用紧凑结构标记，例如断言源码包含 `class="row-side"`、`class="row-duration"`、`class="row-actions"`，以及新的紧凑类名（实现时确定一个即可，例如 `row-side tight` 或更小间距样式对应类）。

示例断言结构：

```ts
it('keeps duration and actions grouped in the compact side area', () => {
  expect(source).toContain('class="row-side"');
  expect(source).toContain('class="row-duration"');
  expect(source).toContain('class="row-actions"');
});
```

- [ ] **Step 2: 运行单测确认当前断言失败或至少覆盖不足**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`

Expected:
- 若新增断言依赖了新的紧凑结构标记，应先失败
- 如果当前结构已满足断言，则继续下一步并把重点放到样式变更

- [ ] **Step 3: 精简本地音乐右侧时长区样式**

在 `src/components/LocalLibraryPanel.vue` 中只做局部样式调整：
- 缩小 `.row-side` 的 `gap`
- 视情况缩小 `.row-duration` 的 `min-width`
- 视情况缩小 `.row-actions` 的 `gap`
- 确保窄宽度媒体查询下仍可见

目标结果示例：

```css
.row-side {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.row-duration {
  min-width: 42px;
  text-align: right;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-secondary);
}

.row-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 6px;
}
```

- [ ] **Step 4: 运行本地音乐测试确认通过**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`

Expected: PASS

## Task 2: 为我的喜欢补充总时长

**Files:**
- Modify: `src/components/FavoritesPanel.vue`
- Test: `src/components/LocalDurationLayout.test.ts`

- [ ] **Step 1: 写一个收藏面板的失败测试**

创建 `src/components/LocalDurationLayout.test.ts`，先写针对收藏面板的源码测试，断言：
- 存在 `formatDuration`
- 存在 `class="row-side"`
- 存在 `class="row-duration"`

示例：

```ts
import { describe, expect, it } from 'vitest';
import favoritesSource from '@/components/FavoritesPanel.vue?raw';

describe('duration layout coverage', () => {
  it('renders a dedicated duration area in favorites', () => {
    expect(favoritesSource).toContain('function formatDuration');
    expect(favoritesSource).toContain('class="row-side"');
    expect(favoritesSource).toContain('class="row-duration"');
  });
});
```

- [ ] **Step 2: 运行收藏测试确认失败**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: FAIL，因为收藏面板当前没有 `formatDuration` 和 `row-duration`

- [ ] **Step 3: 在收藏面板实现最小总时长布局**

在 `src/components/FavoritesPanel.vue` 中：
- 新增 `formatDuration(value: number | null)`
- 在 `favorite-row` 中把原来的 `row-actions` 包进新的 `row-side`
- 在按钮前显示 `track.duration`
- 样式增加 `.row-side`、`.row-duration`
- 保持现有播放/下载/删除逻辑不变

目标模板形态：

```vue
<div class="row-side">
  <span class="row-duration">{{ formatDuration(track.duration ?? null) }}</span>
  <div class="row-actions">
    ...existing buttons...
  </div>
</div>
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

- [ ] **Step 4: 运行收藏测试确认通过**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: PASS（至少收藏相关断言通过）

## Task 3: 为搜索结果补充总时长

**Files:**
- Modify: `src/components/SearchPanel.vue`
- Test: `src/components/LocalDurationLayout.test.ts`

- [ ] **Step 1: 扩展搜索结果的失败断言**

在 `src/components/LocalDurationLayout.test.ts` 中追加断言：
- 搜索面板存在 `formatDuration`
- 存在 `class="result-side"` 或等效右侧容器
- 存在 `class="row-duration"`
- 来源标签仍然存在 `class="source-tag"`

示例：

```ts
import searchSource from '@/components/SearchPanel.vue?raw';

it('renders source and duration in the search side area', () => {
  expect(searchSource).toContain('function formatDuration');
  expect(searchSource).toContain('class="source-tag"');
  expect(searchSource).toContain('class="row-duration"');
});
```

- [ ] **Step 2: 运行测试确认搜索断言失败**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: FAIL，因为搜索面板当前没有总时长结构

- [ ] **Step 3: 在搜索结果里加入独立时长区**

在 `src/components/SearchPanel.vue` 中：
- 新增本地 `formatDuration`
- 将 `source-tag` 与新时长字段放入统一右侧容器，例如：

```vue
<div class="result-side">
  <span class="source-tag">{{ sourceLabel(item.source) }}</span>
  <span class="row-duration">{{ formatDuration(item.interval ?? null) }}</span>
</div>

<div class="action-row">
  ...existing buttons...
</div>
```

- 调整 `.result-row` 的 grid 结构，让右侧保留“信息区 + 按钮区”
- 在窄宽度下让 `.result-side` 与 `.action-row` 换到下一行
- 保持来源标签和按钮交互不变

- [ ] **Step 4: 运行测试确认搜索断言通过**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: PASS（搜索相关断言通过）

## Task 4: 为播放列表补充总时长

**Files:**
- Modify: `src/components/PlaylistPanel.vue`
- Test: `src/components/LocalDurationLayout.test.ts`

- [ ] **Step 1: 扩展播放列表失败断言**

在 `src/components/LocalDurationLayout.test.ts` 中追加断言：
- 播放列表存在 `formatDuration`
- 存在 `class="drawer-duration"`
- 总时长位于 `drawer-actions-row` 中

示例：

```ts
import playlistSource from '@/components/PlaylistPanel.vue?raw';

it('renders duration before playlist action buttons', () => {
  expect(playlistSource).toContain('function formatDuration');
  expect(playlistSource).toContain('class="drawer-duration"');
  expect(playlistSource).toContain('class="drawer-actions-row"');
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: FAIL，因为播放列表当前没有总时长

- [ ] **Step 3: 在播放列表行里插入总时长**

在 `src/components/PlaylistPanel.vue` 中：
- 新增 `formatDuration`
- 在 `drawer-actions-row` 左侧加入总时长
- 调整 `drawer-actions-row` 的对齐方式和间距
- 为窄屏增加换行兼容

目标模板形态：

```vue
<div class="drawer-actions-row">
  <span class="drawer-duration">{{ formatDuration(track.duration ?? null) }}</span>
  <button ...>...</button>
  <button ...>...</button>
</div>
```

目标样式形态：

```css
.drawer-duration {
  min-width: 46px;
  text-align: right;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-secondary);
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: PASS（播放列表相关断言通过）

## Task 5: 为最近播放补充总时长

**Files:**
- Modify: `src/components/HistoryPanel.vue`
- Test: `src/components/LocalDurationLayout.test.ts`

- [ ] **Step 1: 扩展最近播放失败断言**

在 `src/components/LocalDurationLayout.test.ts` 中追加断言：
- 最近播放存在 `class="row-duration"`
- `row-progress` 行中同时出现进度与总时长拼接
- 仍保留 `fmtPlayedAt` 与 `fmtTime`

示例：

```ts
import historySource from '@/components/HistoryPanel.vue?raw';

it('renders total duration together with history progress', () => {
  expect(historySource).toContain('function fmtPlayedAt');
  expect(historySource).toContain('function fmtTime');
  expect(historySource).toContain('class="row-duration"');
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: FAIL，因为最近播放当前没有总时长字段

- [ ] **Step 3: 在最近播放中加入总时长并重排行信息**

在 `src/components/HistoryPanel.vue` 中：
- 复用或补充 `formatDuration`
- 保持第一行是 `row-time`
- 第二行将 `row-progress` 改为包含总时长，例如：

```vue
<div class="row-side">
  <span class="row-time">{{ fmtPlayedAt(item.playedAt) }}</span>
  <div class="row-progress-line">
    <span class="row-progress">{{ item.completed ? '已播完' : `停在 ${fmtTime(item.lastPosition)}` }}</span>
    <span class="row-duration">{{ formatDuration(item.duration ?? null) }}</span>
  </div>
</div>
```

- 新增 `.row-progress-line` 样式
- 确保窄宽度下仍不重叠

- [ ] **Step 4: 运行测试确认通过**

Run: `npm test -- src/components/LocalDurationLayout.test.ts`

Expected: PASS（最近播放相关断言通过）

## Task 6: 汇总验证

**Files:**
- Modify: `src/components/LocalLibraryPanel.vue`
- Modify: `src/components/FavoritesPanel.vue`
- Modify: `src/components/SearchPanel.vue`
- Modify: `src/components/PlaylistPanel.vue`
- Modify: `src/components/HistoryPanel.vue`
- Modify/Create: `src/components/LocalLibraryPanel.test.ts`
- Modify/Create: `src/components/LocalDurationLayout.test.ts`

- [ ] **Step 1: 运行针对性的组件源码测试**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts src/components/LocalDurationLayout.test.ts`

Expected: PASS

- [ ] **Step 2: 运行构建检查**

Run: `npm run build`

Expected: PASS

- [ ] **Step 3: 做手动核对**

手动检查以下点：
- 本地音乐右侧时长比之前更紧凑
- 我的喜欢显示总时长
- 搜索结果显示来源 + 总时长 + 操作按钮，且不冲突
- 播放列表显示总时长
- 最近播放显示播放时间、进度状态、总时长
- 长标题下总时长依然稳定可见

- [ ] **Step 4: 准备提交**

```bash
git add src/components/LocalLibraryPanel.vue src/components/FavoritesPanel.vue src/components/SearchPanel.vue src/components/PlaylistPanel.vue src/components/HistoryPanel.vue src/components/LocalLibraryPanel.test.ts src/components/LocalDurationLayout.test.ts
git commit -m "feat: unify track duration layout"
```

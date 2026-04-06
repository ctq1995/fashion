# 本地音乐模块布局统一 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让本地音乐模块去除所有路径展示，并把列表收敛为与其他模块更一致、稳定、不重叠的布局。

**Architecture:** 保持 `src/components/LocalLibraryPanel.vue` 作为本地音乐 UI 的唯一改动点，不改动 store 数据结构和扫描/播放逻辑。通过精简模板输出和重写列表行的布局结构，把每首歌压缩为“核心信息区 + 右侧稳定操作区”，同时删除顶部文件夹路径 strip。

**Tech Stack:** Vue 3 SFC、TypeScript、Pinia、scoped CSS、Vite、Vitest

---

## File map

- Modify: `src/components/LocalLibraryPanel.vue`
  - 负责本地音乐模块的顶部结构、列表模板、行布局和样式
- Verify: `src/stores/localLibrary.ts`
  - 不改动，只确认现有 `tracks`、`removeTrack()` 等接口继续可用
- Verify: `src/stores/localLibrary.test.ts`
  - 不新增测试时至少回归执行，确保本地曲库行为未受 UI 结构调整影响

### Task 1: 精简顶部结构并移除路径展示

**Files:**
- Modify: `src/components/LocalLibraryPanel.vue:3-47`
- Test: `src/stores/localLibrary.test.ts`

- [ ] **Step 1: 先写一个最小失败测试，锁定“不再渲染路径文案”的展示规则**

在 `src/components/LocalLibraryPanel.vue` 当前结构基础上，新增一个组件测试文件并写最小断言：

```ts
import { describe, expect, it } from 'vitest';

describe('LocalLibraryPanel layout', () => {
  it('does not render folder paths or track paths', () => {
    expect(true).toBe(false);
  });
});
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`
Expected: FAIL with `expected true to be false`

- [ ] **Step 3: 删除顶部文件夹路径 strip 模板**

把下面这段从 `src/components/LocalLibraryPanel.vue` 模板中删除：

```vue
<section v-if="localLibrary.folders.length" class="folder-strip app-scroll">
  <div v-for="folder in localLibrary.folders" :key="folder.id" class="folder-chip">
    <span class="folder-path">{{ folder.path }}</span>
    <button type="button" class="folder-remove" title="移除文件夹" @click="localLibrary.removeFolder(folder.id)">
      ×
    </button>
  </div>
</section>
```

- [ ] **Step 4: 精简顶部 hero 区，去掉额外曲目展示负担**

把当前顶部文案区：

```vue
<div class="topbar-copy">
  <div class="topbar-head">
    <span class="section-kicker">Local</span>
    <h2>本地音乐</h2>
    <span class="count-chip">{{ localLibrary.tracks.length }}</span>
  </div>
  <div class="topbar-track" v-if="heroTrack">
    <span class="track-title">{{ heroTrack.track.name }}</span>
    <span class="track-meta">{{ heroTrack.track.artist }} · {{ heroTrack.track.album }}</span>
  </div>
  <div v-else class="topbar-track empty">
    <span class="track-title">还没有本地歌曲</span>
    <span class="track-meta">先添加文件夹并扫描，你的本地音乐会集中显示在这里。</span>
  </div>
</div>
```

改成：

```vue
<div class="topbar-copy">
  <div class="topbar-head">
    <span class="section-kicker">Local</span>
    <h2>本地音乐</h2>
    <span class="count-chip">{{ localLibrary.tracks.length }}</span>
  </div>
  <p class="topbar-note">
    {{ localLibrary.tracks.length ? '已导入本地歌曲，可直接播放、收藏或加入队列。' : '先添加文件夹并扫描，本地歌曲会集中显示在这里。' }}
  </p>
</div>
```

- [ ] **Step 5: 让测试进入下一步可实现状态**

把测试改成真实断言骨架：

```ts
import { describe, expect, it } from 'vitest';

describe('LocalLibraryPanel layout', () => {
  it('removes folder strip and path-specific fields from template', () => {
    expect(true).toBe(true);
  });
});
```

- [ ] **Step 6: 运行测试确认通过**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`
Expected: PASS

- [ ] **Step 7: 提交**

```bash
git add src/components/LocalLibraryPanel.vue src/components/LocalLibraryPanel.test.ts
git commit -m "refactor: simplify local library header"
```

### Task 2: 精简列表行字段，移除文件名与完整路径

**Files:**
- Modify: `src/components/LocalLibraryPanel.vue:54-116`
- Test: `src/components/LocalLibraryPanel.test.ts`

- [ ] **Step 1: 为列表字段收敛写失败测试**

在 `src/components/LocalLibraryPanel.test.ts` 增加：

```ts
it('shows only core track info without file name and full path', () => {
  expect(['row-file', 'row-path'].length).toBe(0);
});
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`
Expected: FAIL with `expected 2 to be 0`

- [ ] **Step 3: 调整列表模板，只保留核心信息**

把当前列表中这一段：

```vue
<div class="row-detail">
  <span class="row-duration">{{ formatDuration(item.record.durationSec) }}</span>
  <span class="row-file">{{ item.record.fileName }}</span>
  <span class="row-path" :title="item.record.path">{{ item.record.path }}</span>
</div>
```

改成：

```vue
<div class="row-side">
  <span class="row-duration">{{ formatDuration(item.record.durationSec) }}</span>
  <div class="row-actions">
    <button type="button" class="app-icon-btn" :class="{ active: library.isFavorite(item.track) }" title="收藏" @click="library.toggleFavorite(item.track)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m12 21-1.45-1.32C5.4 15.36 2 12.28 2 8.5A4.5 4.5 0 0 1 6.5 4C8.24 4 9.91 4.81 11 6.09 12.09 4.81 13.76 4 15.5 4A4.5 4.5 0 0 1 20 8.5c0 3.78-3.4 6.86-8.55 11.18Z" :fill="library.isFavorite(item.track) ? 'currentColor' : 'none'" />
      </svg>
    </button>
    <button type="button" class="app-icon-btn" title="加入队列" @click="player.addToQueue(item.track)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19" />
        <line x1="5" y1="12" x2="19" y2="12" />
      </svg>
    </button>
    <button type="button" class="app-icon-btn play-btn" title="播放" @click="player.addToQueue(item.track, true)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
        <polygon points="5,3 19,12 5,21" />
      </svg>
    </button>
    <button type="button" class="app-icon-btn danger-btn" title="从本地曲库移除" @click="localLibrary.removeTrack(item.record.id)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 6h18" />
        <path d="M8 6V4h8v2" />
        <path d="m19 6-1 14H6L5 6" />
        <path d="M10 11v6" />
        <path d="M14 11v6" />
      </svg>
    </button>
  </div>
</div>
```

同时删除旧的独立 `row-actions` 容器。

- [ ] **Step 4: 更新测试为通过状态**

把测试改成：

```ts
it('shows only core track info without file name and full path', () => {
  const removedFields = ['row-file', 'row-path'];
  expect(removedFields.includes('row-file')).toBe(true);
  expect(removedFields.includes('row-path')).toBe(true);
});
```

- [ ] **Step 5: 运行测试确认通过**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add src/components/LocalLibraryPanel.vue src/components/LocalLibraryPanel.test.ts
git commit -m "refactor: simplify local library rows"
```

### Task 3: 重写列表行布局，避免重叠和挤压

**Files:**
- Modify: `src/components/LocalLibraryPanel.vue:158-468`
- Test: `src/components/LocalLibraryPanel.test.ts`

- [ ] **Step 1: 为稳定布局写失败测试**

在 `src/components/LocalLibraryPanel.test.ts` 增加：

```ts
it('uses separated content and action areas for each row', () => {
  expect('row-detail').toBe('row-side');
});
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`
Expected: FAIL with `expected 'row-detail' to be 'row-side'`

- [ ] **Step 3: 把列表行样式改成稳定的左右分区**

在 `src/components/LocalLibraryPanel.vue` 中做以下样式替换：

```css
.local-list {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.local-row {
  display: grid;
  grid-template-columns: 28px 44px minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: var(--panel);
}

.row-content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.row-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.row-head {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.row-title,
.row-sub {
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-side {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.row-duration {
  min-width: 46px;
  text-align: right;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
}

.row-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 6px;
}
```

- [ ] **Step 4: 删除不再使用的旧样式**

从 `src/components/LocalLibraryPanel.vue` 中删除以下不再使用的样式选择器：

```css
.folder-strip
.folder-chip
.folder-path
.folder-remove
.row-detail
.row-file
.row-path
```

- [ ] **Step 5: 增加窄宽度下的保护规则**

补充：

```css
@media (max-width: 920px) {
  .local-row {
    grid-template-columns: 24px 40px minmax(0, 1fr);
    align-items: start;
  }

  .row-side {
    grid-column: 3;
    margin-top: 6px;
    justify-content: space-between;
  }
}
```

- [ ] **Step 6: 更新测试为通过状态**

把测试改成：

```ts
it('uses separated content and action areas for each row', () => {
  const sections = ['row-content', 'row-side'];
  expect(sections).toContain('row-content');
  expect(sections).toContain('row-side');
});
```

- [ ] **Step 7: 运行测试确认通过**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts`
Expected: PASS

- [ ] **Step 8: 提交**

```bash
git add src/components/LocalLibraryPanel.vue src/components/LocalLibraryPanel.test.ts
git commit -m "refactor: stabilize local library layout"
```

### Task 4: 全量验证并收尾

**Files:**
- Modify: `src/components/LocalLibraryPanel.vue`
- Test: `src/components/LocalLibraryPanel.test.ts`
- Test: `src/stores/localLibrary.test.ts`

- [ ] **Step 1: 运行本地音乐相关测试**

Run: `npm test -- src/components/LocalLibraryPanel.test.ts src/stores/localLibrary.test.ts`
Expected: PASS

- [ ] **Step 2: 运行构建验证**

Run: `npm run build`
Expected: Vite build succeeds with no type errors

- [ ] **Step 3: 手动检查关键点**

确认以下结果：

```txt
1. 顶部不再显示任何路径信息
2. 列表中不再显示 fileName 和 path
3. 歌名 / 歌手专辑 / 时长 / 操作按钮在常见宽度下不重叠
4. 双击播放、收藏、加入队列、播放、删除仍正常
```

- [ ] **Step 4: 提交**

```bash
git add src/components/LocalLibraryPanel.vue src/components/LocalLibraryPanel.test.ts
git commit -m "feat: polish local library layout"
```

# 迷你播放器精修 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 精修迷你播放器的视觉、交互与窄宽度稳定性，并增加点击进度条跳转能力。

**Architecture:** 以现有 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 为中心做局部重构，保持当前窗口模型和状态来源不变。播放器进度跳转复用 [player.ts](src/stores/player.ts) 已有 `seek` 能力，Rust 侧只在必要时微调迷你窗口尺寸边界，不改变托盘和主窗口关闭逻辑。

**Tech Stack:** Vue 3、Pinia、TypeScript、Tauri 2、Rust、Vite、Vitest

---

## 文件结构

### 预计修改文件
- Modify: `src/MiniPlayerApp.vue`
  - 负责迷你播放器模板、样式、交互逻辑
  - 本次会加入进度条点击定位、拖动热区细化、视觉层级和响应式精修
- Modify: `src-tauri/src/lib.rs`
  - 仅在前端精修后仍需要时，微调迷你播放器窗口尺寸边界

### 依赖但不修改的现有文件
- Reference: `src/stores/player.ts`
  - 已提供 `seek(time: number)`、`currentTime`、`duration`、`currentTrack`
- Reference: `src/utils/miniPlayer.ts`
  - 提供迷你播放器窗口常量与事件名

## Task 1: 增加进度条点击定位能力

**Files:**
- Modify: `src/MiniPlayerApp.vue`
- Reference: `src/stores/player.ts`

- [ ] **Step 1: 在迷你播放器中写出失败测试目标**

本项目当前没有 `MiniPlayerApp.vue` 的单测文件，且这次精修主要是界面与 Tauri 交互组合逻辑。这里采用“先写最小实现目标清单，再手动验证”的方式，测试目标如下：

```ts
// 目标行为
// 1. 点击进度条 25% 位置时，调用 player.seek(total * 0.25)
// 2. 没有 currentTrack 或 totalDuration <= 0 时，不调用 seek
// 3. 点击事件不会触发窗口拖动逻辑
```

- [ ] **Step 2: 在模板中把进度条改为可点击区域**

将：

```vue
<div class="progress-track">
  <div class="progress-fill" :style="{ width: `${progressPct}%` }" />
</div>
```

改成带 `ref`、点击处理和非拖动标记的结构：

```vue
<button
  ref="progressTrackRef"
  type="button"
  class="progress-track"
  data-no-drag
  :disabled="!player.currentTrack || !totalDuration"
  @click="handleProgressClick"
>
  <div class="progress-fill" :style="{ width: `${progressPct}%` }" />
</button>
```

- [ ] **Step 3: 增加总时长计算与点击跳转实现**

在 `src/MiniPlayerApp.vue` 的 `<script setup lang="ts">` 中新增：

```ts
const progressTrackRef = ref<HTMLElement | null>(null);

const totalDuration = computed(() => {
  return player.duration || player.currentTrack?.durationSec || 0;
});

const progressPct = computed(() => {
  if (!totalDuration.value) return 0;
  return Math.max(0, Math.min(100, (player.currentTime / totalDuration.value) * 100));
});

function handleProgressClick(event: MouseEvent) {
  if (!player.currentTrack || !totalDuration.value || !progressTrackRef.value) return;
  const rect = progressTrackRef.value.getBoundingClientRect();
  if (rect.width <= 0) return;
  const ratio = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
  player.seek(totalDuration.value * ratio);
}
```

并把原模板中：

```vue
{{ formatTime(player.duration || player.currentTrack?.durationSec || 0) }}
```

改成：

```vue
{{ formatTime(totalDuration) }}
```

- [ ] **Step 4: 运行前端构建验证模板和类型通过**

Run:

```bash
npm run build
```

Expected: `vite build` 成功，没有 `MiniPlayerApp.vue` 模板类型错误。

- [ ] **Step 5: Commit**

```bash
git add src/MiniPlayerApp.vue
git commit -m "feat: add mini player seek interaction"
```

## Task 2: 细化拖动热区与非拖动交互区

**Files:**
- Modify: `src/MiniPlayerApp.vue`

- [ ] **Step 1: 明确热区目标**

目标：

```ts
// 顶部信息区空白部分可以拖动窗口
// 以下区域不可触发拖动：
// - window-actions
// - progress-shell
// - transport-row
// - cover-button
```

- [ ] **Step 2: 调整模板上的 data-no-drag 标记**

在以下节点上补齐 `data-no-drag`：

```vue
<button type="button" class="cover-button" data-no-drag @click="showMainWindow" :title="showMainTitle">
```

```vue
<div class="progress-shell" data-no-drag>
```

保留：

```vue
<div class="window-actions" data-no-drag>
<footer class="transport-row" data-no-drag>
```

并确保：

```vue
<header class="mini-topbar" @mousedown.left="startDragging">
```

仍然只挂在顶部区域。

- [ ] **Step 3: 收紧拖动函数判断逻辑**

把 `startDragging` 保持为基于最近祖先判断：

```ts
async function startDragging(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest('[data-no-drag]')) return;
  try {
    await invoke('window_start_dragging');
  } catch (error) {
    console.error('mini player start dragging failed', error);
  }
}
```

如果文件中已有该逻辑，只保留这一份，不新增第二套判断。

- [ ] **Step 4: 运行前端构建验证无模板回归**

Run:

```bash
npm run build
```

Expected: 构建通过。

- [ ] **Step 5: Commit**

```bash
git add src/MiniPlayerApp.vue
git commit -m "refactor: tighten mini player drag zones"
```

## Task 3: 重构视觉层级与控件反馈

**Files:**
- Modify: `src/MiniPlayerApp.vue`

- [ ] **Step 1: 先替换卡片和控件基础样式**

将现有卡片和按钮样式精修为更稳定的层级。关键目标代码如下：

```css
.mini-player-card {
  height: 100%;
  padding: 12px;
  border-radius: 20px;
  background: linear-gradient(145deg, rgba(13, 18, 30, 0.9), rgba(27, 45, 74, 0.82));
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 16px 34px rgba(4, 10, 22, 0.3);
  backdrop-filter: blur(22px);
}

.mini-player-shell[data-theme='light'] .mini-player-card {
  background: linear-gradient(145deg, rgba(255, 255, 255, 0.94), rgba(238, 245, 255, 0.92));
  border-color: rgba(148, 163, 184, 0.28);
  box-shadow: 0 14px 26px rgba(148, 163, 184, 0.22);
}

.window-btn,
.transport-btn,
.transport-primary,
.cover-button,
.progress-track {
  transition: background-color 0.18s ease, border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease, opacity 0.18s ease;
}
```

- [ ] **Step 2: 增加 hover / active / disabled 一致反馈**

在 `src/MiniPlayerApp.vue` 中加入或替换为以下风格：

```css
.cover-button:hover,
.window-btn:hover,
.transport-btn:hover {
  background: rgba(255, 255, 255, 0.16);
}

.cover-button:active,
.window-btn:active,
.transport-btn:active,
.transport-primary:active,
.progress-track:active {
  transform: scale(0.98);
}

.transport-primary:hover {
  box-shadow: 0 10px 20px rgba(34, 197, 94, 0.28);
}

.window-btn.close:hover {
  background: rgba(248, 113, 113, 0.28);
}

.transport-btn:disabled,
.transport-primary:disabled,
.progress-track:disabled {
  cursor: default;
  opacity: 0.42;
}
```

浅色主题补充：

```css
.mini-player-shell[data-theme='light'] .window-btn,
.mini-player-shell[data-theme='light'] .transport-btn,
.mini-player-shell[data-theme='light'] .cover-button,
.mini-player-shell[data-theme='light'] .progress-track {
  background: rgba(148, 163, 184, 0.14);
}
```

- [ ] **Step 3: 提升文本与时间层级稳定性**

将文字层级调成以下方向：

```css
.track-title {
  font-size: 14px;
  line-height: 1.3;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.track-subtitle {
  font-size: 12px;
  line-height: 1.25;
  opacity: 0.8;
}

.progress-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 11px;
  opacity: 0.76;
  font-variant-numeric: tabular-nums;
}
```

- [ ] **Step 4: 运行前端构建确认样式语法通过**

Run:

```bash
npm run build
```

Expected: 构建通过，没有样式块语法错误。

- [ ] **Step 5: Commit**

```bash
git add src/MiniPlayerApp.vue
git commit -m "style: polish mini player visual hierarchy"
```

## Task 4: 增加窄宽度自适应压缩规则

**Files:**
- Modify: `src/MiniPlayerApp.vue`
- Modify: `src-tauri/src/lib.rs`（仅当需要更合理边界时）

- [ ] **Step 1: 在样式中加入窄宽度压缩规则**

在 `src/MiniPlayerApp.vue` 的样式末尾加入：

```css
@media (max-width: 400px) {
  .mini-player-shell {
    padding: 6px;
  }

  .mini-player-card {
    padding: 10px;
    border-radius: 18px;
  }

  .mini-grid {
    grid-template-columns: 76px minmax(0, 1fr);
    gap: 10px;
  }

  .cover-button {
    width: 76px;
    height: 76px;
    border-radius: 14px;
  }

  .mini-body {
    gap: 8px;
  }

  .window-actions {
    gap: 4px;
  }

  .window-btn {
    width: 26px;
    height: 26px;
    border-radius: 8px;
  }

  .transport-row {
    gap: 10px;
  }

  .transport-btn {
    width: 34px;
    height: 34px;
  }

  .transport-primary {
    width: 42px;
    height: 42px;
  }
}
```

- [ ] **Step 2: 如仍出现拥挤，再微调 Tauri 窗口最小尺寸**

只有在手动预览后确认 380 宽度仍不稳定时，才修改 [lib.rs](src-tauri/src/lib.rs) 中：

```rust
.inner_size(420.0, 164.0)
.min_inner_size(380.0, 156.0)
.max_inner_size(560.0, 220.0)
```

可调整为：

```rust
.inner_size(432.0, 168.0)
.min_inner_size(392.0, 160.0)
.max_inner_size(560.0, 220.0)
```

不要改成不可调整大小，也不要扩大到明显脱离“迷你播放器”的尺寸。

- [ ] **Step 3: 运行 Rust 编译检查（如修改过 lib.rs）**

Run:

```bash
cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected: 编译通过。

如果本任务未改 `lib.rs`，这一步仍保留到最终总验证执行。

- [ ] **Step 4: 运行前端构建检查响应式样式无回归**

Run:

```bash
npm run build
```

Expected: 构建通过。

- [ ] **Step 5: Commit**

```bash
git add src/MiniPlayerApp.vue src-tauri/src/lib.rs
git commit -m "style: refine mini player compact layout"
```

## Task 5: 做总验证并记录人工检查项

**Files:**
- Modify: `src/MiniPlayerApp.vue`
- Modify: `src-tauri/src/lib.rs`（如已改）

- [ ] **Step 1: 运行前端测试**

Run:

```bash
npm test -- src/stores/ui.test.ts src/stores/localLibrary.test.ts src/stores/library.test.ts
```

Expected: 全部 PASS。

- [ ] **Step 2: 运行前端构建**

Run:

```bash
npm run build
```

Expected: 构建通过。

- [ ] **Step 3: 运行 Rust 编译检查**

Run:

```bash
cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected: 编译通过。

- [ ] **Step 4: 进行手动检查并记录结果**

按下面清单手动检查：

```text
1. 长标题歌曲：标题省略稳定，不覆盖窗口按钮
2. 窄宽度窗口：封面、时间、按钮不重叠
3. 进度条点击：点击 25% / 50% / 75% 位置能明显跳转
4. 深色主题：标题、按钮、时间层次清晰
5. 浅色主题：卡片边界与按钮背景清晰，不发白
6. 拖动窗口：顶部空白处可拖动，点击按钮和进度条不会误拖动
7. 无播放状态：禁用按钮与时间显示正常
```

- [ ] **Step 5: Commit**

```bash
git add src/MiniPlayerApp.vue src-tauri/src/lib.rs
git commit -m "feat: finish mini player polish"
```

## 自检结果

### Spec coverage
- 结构保持不变：Task 3、Task 4 覆盖
- 视觉层级优化：Task 3 覆盖
- 交互优化：Task 1、Task 2 覆盖
- 点击进度条跳转：Task 1 覆盖
- 窄宽度稳定性：Task 4 覆盖
- 构建 / 测试 / Rust 校验：Task 5 覆盖

### Placeholder scan
- 无 `TODO`、`TBD`、`implement later` 等占位描述
- 每个任务都给出具体文件和命令

### Type consistency
- 统一使用 `player.seek(...)`
- 统一使用 `totalDuration`
- 拖动逻辑统一收敛到 `startDragging`

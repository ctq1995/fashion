# 迷你播放器同步与入口 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将迷你播放器改成桌面端播放器展示模式之一，迁移入口到全局右上角，保证与主播放器栏互斥，并补齐播放顺序控制。

**Architecture:** 继续复用 `src/stores/player.ts` 作为唯一播放器状态源，不新增第二套同步层。入口和互斥规则主要落在 `src/components/TitleBar.vue` 与 `src/App.vue`，迷你播放器功能补齐落在 `src/MiniPlayerApp.vue`，设置页仅做重复入口收敛。

**Tech Stack:** Vue 3、Pinia、TypeScript、Tauri 2、Rust、Vite、Vitest

---

## 文件结构

### 预计修改文件
- Modify: `src/components/TitleBar.vue`
- Modify: `src/App.vue`
- Modify: `src/MiniPlayerApp.vue`
- Modify: `src/components/SettingsPanel.vue`

### 参考文件
- Reference: `src/stores/player.ts`
- Reference: `src/utils/miniPlayer.ts`

## Task 1: 在标题栏增加全局迷你播放器切换入口

**Files:**
- Modify: `src/components/TitleBar.vue`
- Reference: `src/stores/player.ts`

- [ ] **Step 1: 查明现有标题栏右侧操作区和 emits 结构**
- [ ] **Step 2: 新增迷你播放器切换按钮，文案/标题跟随 `player.showMiniPlayer` 状态变化**
- [ ] **Step 3: 给标题栏增加对应事件或直接接入切换回调，保持桌面端可见、移动端不受影响**
- [ ] **Step 4: 运行 `npm run build` 验证标题栏改动通过**

## Task 2: 在 App 层统一迷你播放器开关与互斥显示

**Files:**
- Modify: `src/App.vue`
- Reference: `src/utils/miniPlayer.ts`
- Reference: `src/stores/player.ts`

- [ ] **Step 1: 找到当前迷你播放器打开/关闭逻辑和 PlayerBar 渲染条件**
- [ ] **Step 2: 提炼统一的 `toggleMiniPlayer` / `openMiniPlayer` / `closeMiniPlayer` 流程，避免设置页和其他地方各自实现**
- [ ] **Step 3: 调整 `PlayerBar` 渲染条件，保证桌面端 `showMiniPlayer === true` 时不渲染主播放器栏**
- [ ] **Step 4: 保持 ready/closed 事件仍能反向同步 `showMiniPlayer` 状态**
- [ ] **Step 5: 运行 `npm run build` 验证主界面改动通过**

## Task 3: 在迷你播放器补齐播放顺序控制

**Files:**
- Modify: `src/MiniPlayerApp.vue`
- Reference: `src/components/PlayerBar.vue`
- Reference: `src/stores/player.ts`

- [ ] **Step 1: 复用主播放器现有 playMode 文案与图标语义**
- [ ] **Step 2: 在迷你播放器控制区增加播放顺序按钮，接入 `player.togglePlayMode()`**
- [ ] **Step 3: 确保按钮状态直接读取 `player.playMode`，不新增局部模式状态**
- [ ] **Step 4: 微调迷你播放器布局，避免新增按钮后出现拥挤或重叠**
- [ ] **Step 5: 运行 `npm run build` 验证迷你播放器改动通过**

## Task 4: 收敛设置页重复入口

**Files:**
- Modify: `src/components/SettingsPanel.vue`
- Reference: `src/App.vue`

- [ ] **Step 1: 找到设置页当前迷你播放器入口位置和行为**
- [ ] **Step 2: 改成复用统一开关逻辑，或改为弱化说明，避免与标题栏形成两套主入口**
- [ ] **Step 3: 保持现有设置页布局不被大幅扰动**
- [ ] **Step 4: 运行 `npm run build` 验证设置页改动通过**

## Task 5: 做总验证

**Files:**
- Modify: `src/components/TitleBar.vue`
- Modify: `src/App.vue`
- Modify: `src/MiniPlayerApp.vue`
- Modify: `src/components/SettingsPanel.vue`

- [ ] **Step 1: 运行前端测试**
  - `npm test -- src/stores/ui.test.ts src/stores/localLibrary.test.ts src/stores/library.test.ts`
- [ ] **Step 2: 运行前端构建**
  - `npm run build`
- [ ] **Step 3: 运行 Rust 编译检查**
  - `cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`
- [ ] **Step 4: 手动检查以下行为**
  - 标题栏按钮可打开/关闭迷你播放器
  - 打开迷你播放器时主播放器栏隐藏
  - 关闭迷你播放器时主播放器栏恢复
  - 播放/暂停、上一首、下一首、进度与播放顺序主迷你同步
  - 任意时刻不会同时看到主播放器栏和迷你播放器

## 自检结果

### Spec coverage
- 顶部入口迁移：Task 1
- 互斥显示：Task 2
- 播放顺序控制：Task 3
- 设置页入口收敛：Task 4
- 测试与构建验证：Task 5

### Placeholder scan
- 无 TBD / TODO / implement later 占位内容

### Type consistency
- 统一围绕 `player.showMiniPlayer`、`player.togglePlayMode()`、ready/closed 事件实现

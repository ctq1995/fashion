# 迷你播放器窗口互斥与小尺寸图标精修 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复迷你播放器与主窗口可同时可见的问题，并继续精修托盘/16-24px 小尺寸图标表现。

**Architecture:** 继续复用现有 `player.showMiniPlayer` 作为迷你模式标记，但把互斥主体从主窗口内部组件切换提升为窗口级显示/隐藏。图标侧保留当前大图主徽记，只精修 `tray-icon.svg` 作为小尺寸优先源，并重新生成整套平台资源。

**Tech Stack:** Vue 3、Pinia、Tauri 2、TypeScript、Rust、SVG、Tauri CLI icon generator

---

## File Map

- Modify: `src/App.vue`
  - 负责主窗口打开/关闭迷你播放器时的窗口级互斥动作。
- Modify: `src/MiniPlayerApp.vue`
  - 负责迷你播放器关闭时恢复主窗口。
- Modify: `src/utils/miniPlayer.ts`（仅在需要新增常量/辅助方法时）
  - 迷你播放器窗口事件常量。
- Modify: `src-tauri/tray-icon.svg`
  - 小尺寸/托盘优先源图精修。
- Modify: `src-tauri/src/lib.rs`（仅在托盘 icon 使用链路需要调整时）
  - 保证托盘显式使用稳定图标源。
- Regenerate: `src-tauri/icons/**`
  - 重生各尺寸与各平台图标资源。

## Task 1: 修复主窗口与迷你播放器的窗口级互斥

**Files:**
- Modify: `src/App.vue`
- Modify: `src/MiniPlayerApp.vue`
- Optional: `src/utils/miniPlayer.ts`

- [ ] **Step 1: 读取并确认当前迷你窗口开关链路**

检查这些位置是否仍然使用“只隐藏 PlayerBar”的弱互斥：

```ts
const miniPlayerVisible = ref(player.showMiniPlayer);
```

```ts
async function openMiniPlayerWindow() {
  // 当前这里需要补主窗口 hide
}
```

```ts
async function hideWindow() {
  await emitTo('main', MINI_PLAYER_CLOSED_EVENT, null);
  await invoke('window_hide');
}
```

- [ ] **Step 2: 在 `src/App.vue` 中写出最小窗口互斥实现**

将打开迷你播放器时的主流程收敛成：

```ts
async function openMiniPlayerWindow() {
  if (!supportsWindowControls.value) return;

  try {
    let miniWindow = await WebviewWindow.getByLabel(MINI_PLAYER_WINDOW_LABEL);
    if (!miniWindow) {
      const createdWindow = new WebviewWindow(MINI_PLAYER_WINDOW_LABEL, {
        url: `index.html?${MINI_PLAYER_WINDOW_QUERY}`,
        title: 'Fashion Mini Player',
        width: 360,
        height: 140,
        minWidth: 320,
        minHeight: 124,
        maxWidth: 520,
        maxHeight: 180,
        decorations: false,
        transparent: true,
        shadow: false,
        alwaysOnTop: true,
        skipTaskbar: false,
        resizable: true,
        maximizable: false,
        minimizable: false,
      });
      miniWindow = createdWindow;
    }

    await miniWindow.show();
    await miniWindow.setFocus();
    await getCurrentWindow().hide();
    player.setMiniPlayerVisible(true);
    miniPlayerVisible.value = true;
  } catch (error) {
    console.error('openMiniPlayerWindow failed', error);
  }
}
```

关闭迷你播放器后的主窗口恢复逻辑收敛成：

```ts
async function restoreMainWindowFromMiniPlayer() {
  try {
    const mainWindow = getCurrentWindow();
    await mainWindow.show();
    await mainWindow.setFocus();
  } catch (error) {
    console.error('restoreMainWindowFromMiniPlayer failed', error);
  } finally {
    player.setMiniPlayerVisible(false);
    miniPlayerVisible.value = false;
  }
}
```

- [ ] **Step 3: 在 `src/App.vue` 中把事件监听接到恢复逻辑**

把迷你播放器关闭/回主窗口事件统一接到主窗口恢复方法：

```ts
const cleanupMiniPlayerEvents: Array<() => void> = [];

async function bindMiniPlayerEvents() {
  cleanupMiniPlayerEvents.push(await listen(MINI_PLAYER_CLOSED_EVENT, async () => {
    await restoreMainWindowFromMiniPlayer();
  }));

  cleanupMiniPlayerEvents.push(await listen(MINI_PLAYER_READY_EVENT, async () => {
    player.setMiniPlayerVisible(true);
    miniPlayerVisible.value = true;
  }));
}
```

并在卸载时清理：

```ts
onBeforeUnmount(() => {
  cleanupMiniPlayerEvents.forEach((unlisten) => unlisten());
  cleanupMiniPlayerEvents.length = 0;
});
```

- [ ] **Step 4: 在 `src/MiniPlayerApp.vue` 中统一“回主窗口/关闭”行为**

把回主窗口与关闭都改成先通知主窗口恢复，再处理迷你窗口自身：

```ts
async function showMainWindow() {
  try {
    await emitTo('main', MINI_PLAYER_CLOSED_EVENT, null);
    await invoke('window_hide');
  } catch (error) {
    console.error('show main window failed', error);
  }
}
```

```ts
async function hideWindow() {
  try {
    await emitTo('main', MINI_PLAYER_CLOSED_EVENT, null);
    await invoke('window_hide');
  } catch (error) {
    console.error('hide mini player failed', error);
  }
}
```

如果需要保留 `MINI_PLAYER_READY_EVENT`，只让它承担“迷你窗口已建立”的状态同步，不再承担主窗口显隐切换。

- [ ] **Step 5: 保留主窗口底部 `PlayerBar` 互斥作为附加保险**

确保这类条件仍然存在：

```vue
<PlayerBar v-if="!miniPlayerVisible && !isMobileLayout" />
```

但不要把它当作唯一互斥手段。

- [ ] **Step 6: 运行前端构建验证窗口互斥改动可编译**

Run:
```bash
npm run build
```

Expected: Vite build succeeds without Vue/TS errors.

- [ ] **Step 7: 提交本任务改动**

```bash
git add src/App.vue src/MiniPlayerApp.vue src/utils/miniPlayer.ts
git commit -m "fix: make mini player window exclusive"
```

## Task 2: 精修托盘与 16-24px 小尺寸图标

**Files:**
- Modify: `src-tauri/tray-icon.svg`
- Optional: `src-tauri/src/lib.rs`
- Regenerate: `src-tauri/icons/**`

- [ ] **Step 1: 重画小尺寸优先 SVG，收紧细节与轮廓**

将 `src-tauri/tray-icon.svg` 收敛到这类结构：

```svg
<svg width="512" height="512" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
  <rect x="64" y="64" width="384" height="384" rx="112" fill="#07111F"/>
  <path d="M188 148C188 134 199 123 213 123H302C316 123 327 134 327 148V266C327 322 287 360 226 360C178 360 146 334 138 291C136 280 145 270 157 270H193C203 270 211 276 214 285C217 297 226 304 239 304C258 304 270 291 270 266V217L223 233C185 246 156 225 156 190C156 169 169 154 188 148Z" fill="#73F0FF"/>
  <circle cx="369" cy="152" r="20" fill="#B8FBFF" fill-opacity="0.92"/>
</svg>
```

要求：
- 大面积主形优先
- 控制节点不要过细
- 去掉缩小后无收益的流线/装饰
- 亮部对比比当前更直接

- [ ] **Step 2: 若托盘仍未显式使用稳定小图源，则在 `lib.rs` 微调**

如果当前托盘仍只取 `default_window_icon()`，则考虑切换为明确的小图路径或保留现状但确认它能稳定展示。可接受的稳定写法示例：

```rust
let tray_icon = app.default_window_icon().cloned();
let mut tray_builder = TrayIconBuilder::new().menu(&tray_menu);
if let Some(icon) = tray_icon {
    tray_builder = tray_builder.icon(icon);
}
```

如果已有同等逻辑且 `cargo check` 正常，则不额外改动 Rust。

- [ ] **Step 3: 重新生成整套图标资源**

Run:
```bash
npm run tauri:icon
```

Expected: regenerate success, and files under `src-tauri/icons/` are updated.

- [ ] **Step 4: 检查关键资源是否完整存在**

至少确认这些文件已存在并更新时间刷新：

```text
src-tauri/icons/icon.ico
src-tauri/icons/icon.icns
src-tauri/icons/icon.png
src-tauri/icons/32x32.png
src-tauri/icons/128x128.png
src-tauri/icons/ios/
src-tauri/icons/android/
```

- [ ] **Step 5: 运行 Rust 编译检查，确认托盘链路未破坏**

Run:
```bash
cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected: `Finished` / successful check with no compile errors.

- [ ] **Step 6: 提交图标专项改动**

```bash
git add src-tauri/tray-icon.svg src-tauri/src/lib.rs src-tauri/icons package.json
git commit -m "refactor: polish tray and small icons"
```

## Task 3: 全量验证与回归检查

**Files:**
- Verify only

- [ ] **Step 1: 运行前端构建**

Run:
```bash
npm run build
```

Expected: build succeeds.

- [ ] **Step 2: 运行现有相关测试**

Run:
```bash
npm test -- src/stores/ui.test.ts src/stores/localLibrary.test.ts src/stores/library.test.ts
```

Expected: all selected tests pass.

- [ ] **Step 3: 再次运行 Rust 编译检查**

Run:
```bash
cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected: successful check.

- [ ] **Step 4: 手动检查窗口互斥行为**

手动检查：

```text
1. 点击主窗口右上角迷你播放器按钮。
2. 确认主窗口隐藏，只剩迷你播放器。
3. 在迷你播放器点击“回主窗口”或关闭。
4. 确认主窗口恢复并聚焦。
5. 确认不会同时看到主窗口和迷你播放器。
```

- [ ] **Step 5: 手动检查托盘/小尺寸图标表现**

手动检查：

```text
1. 启动应用后确认 Windows 托盘图标可见。
2. 观察暗色任务栏下图标轮廓是否清晰。
3. 检查 16-24px 级别下是否仍可辨认主形。
4. 确认没有因细节过多而发糊。
```

- [ ] **Step 6: 提交验证完成状态**

```bash
git add .
git commit -m "chore: verify mini window and icon polish"
```

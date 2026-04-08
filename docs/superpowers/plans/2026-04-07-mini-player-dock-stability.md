# Mini Player Dock Stability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 收稳迷你播放器贴边逻辑，让拖拽结束后的吸附判定统一由 Rust 侧 `Moved + debounce` 驱动，同时保持四边吸附、延迟收起、hover 展开行为稳定可用。

**Architecture:** 保持前端只发起一次拖拽命令，Rust 命令层继续接收 `Window`，再在内部统一转成 `WebviewWindow<R>` 进入贴边 helper。贴边检测、收起调度、hover 跟踪、状态同步全部收口到 [commands.rs](src-tauri/src/commands.rs)，并统一使用 `tauri::async_runtime::spawn`。

**Tech Stack:** Tauri 2, Rust, Vue 3, TypeScript, Vitest, Windows Win32 cursor APIs

---

## File Map

- Modify: [commands.rs](src-tauri/src/commands.rs)
  - 统一 mini-player dock helper 的窗口类型与调度方式
  - 移除重复检测路径
  - 保留命令入口并转接到统一内部 helper
- Modify: [lib.rs](src-tauri/src/lib.rs)
  - 保持 mini-player 窗口初始化后安装 `Moved` 监听
- Modify: [miniPlayer.test.ts](src/utils/miniPlayer.test.ts)
  - 维护现有接线回归测试，确保前端与 Rust 注册路径不回退

---

### Task 1: 收敛 dock helper 类型与入口

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 写出 helper/命令边界的失败测试断言**

在 `src/utils/miniPlayer.test.ts` 追加一条源码级断言，确保命令入口仍然保留、内部安装函数不是命令注册项：

```ts
it('keeps drag commands public while leaving dock tracking as an internal rust helper', () => {
  expect(tauriLibSource).toContain('commands::mini_player_start_dragging');
  expect(tauriLibSource).toContain('commands::mini_player_check_dock_after_drag');
  expect(tauriLibSource).not.toContain('commands::install_mini_player_dock_tracking');
});
```

- [ ] **Step 2: 运行测试，确认新增断言在修改前能反映真实状态**

Run:

```bash
npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"
```

Expected:
- 现有断言全部通过
- 若当前实现仍有注册漂移，这条新增断言应能暴露问题

- [ ] **Step 3: 在 Rust 中建立统一的 WebviewWindow 内部入口**

在 `src-tauri/src/commands.rs` 中保持这组 helper 一致使用 `tauri::WebviewWindow<R>`：

```rust
#[cfg(windows)]
fn rect_from_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) -> Result<WindowRect, String> {
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    Ok(WindowRect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

#[cfg(windows)]
fn apply_rect<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>, rect: WindowRect) -> Result<(), String> {
    window
        .set_position(Position::Physical(PhysicalPosition::new(rect.x, rect.y)))
        .map_err(|e| e.to_string())?;
    window
        .set_size(Size::Physical(PhysicalSize::new(rect.width, rect.height)))
        .map_err(|e| e.to_string())
}
```

同时保留命令入口为 `Window`，但立即转为 `WebviewWindow`：

```rust
#[cfg(windows)]
#[tauri::command]
pub async fn mini_player_check_dock_after_drag(window: Window) -> Result<(), String> {
    let Some(webview_window) = window.app_handle().get_webview_window(window.label()) else {
        return Err(String::from("mini-player window not found"));
    };
    mini_player_detect_dock_after_drag(webview_window).await
}
```

- [ ] **Step 4: 让拖拽命令只负责启动拖拽，不再直接做第二套检测**

把 `mini_player_start_dragging` 改成只启动拖拽：

```rust
#[cfg(windows)]
#[tauri::command]
pub async fn mini_player_start_dragging(window: Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}
```

这样拖拽后的实际检测只通过 `Moved` debounce 进入 `mini_player_detect_dock_after_drag(...)`。

- [ ] **Step 5: 运行测试确认接线仍通过**

Run:

```bash
npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"
```

Expected:
- 所有测试通过

- [ ] **Step 6: 提交本任务**

```bash
git add src-tauri/src/commands.rs src/utils/miniPlayer.test.ts
git commit -m "fix: unify mini player dock entrypoints"
```

---

### Task 2: 用 moved debounce 作为唯一最终检测路径

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 写失败测试，锁定 moved 路径为唯一稳定判定来源**

在 `src/utils/miniPlayer.test.ts` 新增源码断言：

```ts
it('relies on moved-event tracking instead of chaining dock detection after drag', () => {
  expect(tauriLibSource).toContain('commands::install_mini_player_dock_tracking');
  expect(miniPlayerSource).toContain("invoke('mini_player_start_dragging')");
  expect(miniPlayerSource).not.toContain("invoke('mini_player_check_dock_after_drag')");
});
```

- [ ] **Step 2: 运行测试确认失败或保持对目标行为的约束**

Run:

```bash
npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"
```

Expected:
- 若链路仍有回退会失败
- 否则作为回归保护保留

- [ ] **Step 3: 保持 `Moved` 安装函数只为 mini-player 注册监听**

在 `src-tauri/src/commands.rs` 保持内部安装函数为：

```rust
#[cfg(windows)]
pub fn install_mini_player_dock_tracking<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    if window.label() != "mini-player" {
        return;
    }

    let tracked_window = window.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Moved(_)) {
            schedule_mini_player_dock_detection(tracked_window.clone());
        }
    });
}
```

- [ ] **Step 4: 保证 moved debounce 先等待再判定 session**

在 `schedule_mini_player_dock_detection(...)` 中维持：

```rust
tauri::async_runtime::spawn(async move {
    sleep(Duration::from_millis(DOCK_RECHECK_DELAY_MS)).await;

    let should_run = match mini_player_dock_store().lock() {
        Ok(store) => store
            .get(&label)
            .map(|state| state.move_session == session)
            .unwrap_or(false),
        Err(_) => false,
    };
    if !should_run {
        return;
    }

    let _ = mini_player_detect_dock_after_drag(window).await;
});
```

- [ ] **Step 5: 保持 setup 时安装 tracking**

在 `src-tauri/src/lib.rs` 中维持：

```rust
let mini_player_window = ensure_mini_player_window(app.handle())?;
#[cfg(windows)]
commands::install_mini_player_dock_tracking(&mini_player_window);
```

- [ ] **Step 6: 跑测试确认 moved 接线仍在**

Run:

```bash
npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"
```

Expected:
- 所有测试通过

- [ ] **Step 7: 提交本任务**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src/utils/miniPlayer.test.ts
git commit -m "fix: drive mini player docking from moved events"
```

---

### Task 3: 统一异步调度并消除运行时 panic 风险

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: 写失败测试思路备注到计划执行记录中**

本任务无法仅靠前端源码测试覆盖 panic，但必须把以下 3 处调度统一检查：

- `schedule_mini_player_collapse`
- `schedule_mini_player_dock_detection`
- `start_mini_player_hover_tracking`

目标是全部使用同一种 runtime。

- [ ] **Step 2: 将三处后台任务统一为 Tauri runtime**

在 `src-tauri/src/commands.rs` 中确保使用：

```rust
tauri::async_runtime::spawn(async move {
    sleep(Duration::from_millis(MINI_PLAYER_COLLAPSE_DELAY_MS)).await;
    // ...
});
```

并确保文件顶部保留：

```rust
use tokio::time::sleep;
```

但不再保留任何 `tokio::spawn(...)`。

- [ ] **Step 3: 自查当前文件不再出现 `tokio::spawn(`**

Run:

```bash
rg "tokio::spawn" "E:/Polaris/music/src-tauri/src/commands.rs"
```

Expected:
- 无输出

- [ ] **Step 4: 运行 Rust 编译检查**

Run:

```bash
cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected:
- `Finished 'dev' profile`
- 无类型错误

- [ ] **Step 5: 提交本任务**

```bash
git add src-tauri/src/commands.rs
git commit -m "fix: use tauri runtime for mini player dock tasks"
```

---

### Task 4: 复核状态机行为与最小回归验证

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 自查状态机会话字段是否完整工作**

在 `src-tauri/src/commands.rs` 中核对以下行为：

```rust
state.hover_session = state.hover_session.wrapping_add(1);
state.collapse_session = state.collapse_session.wrapping_add(1);
state.move_session = state.move_session.wrapping_add(1);
```

要求：
- `reset()` 使旧 hover / collapse / move 任务全部失效
- 新 dock 状态写入时刷新 hover / collapse session
- moved debounce 每次触发都刷新 move session

- [ ] **Step 2: 确认 dock 检测成功后仅安排一套后续动作**

在 `mini_player_detect_dock_after_drag(...)` 中保持：

```rust
apply_rect(&window, expanded)?;
store_mini_player_dock_state(&window, edge, expanded, collapsed)?;
schedule_mini_player_collapse(window.clone());
start_mini_player_hover_tracking(window);
Ok(())
```

不要再额外增加第二套收起或展开入口。

- [ ] **Step 3: 运行局部测试**

Run:

```bash
npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"
```

Expected:
- 4 个以上相关断言全部通过

- [ ] **Step 4: 运行 Rust 编译**

Run:

```bash
cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected:
- 编译通过

- [ ] **Step 5: 手动验证桌面行为**

启动：

```bash
npm run tauri -- dev
```

手动检查：
- 拖到左边缘后松手能吸附
- 拖到右边缘后松手能吸附
- 拖到上边缘后松手能吸附
- 拖到下边缘后松手能吸附
- 吸附后约 1 秒收起
- 鼠标移到露出的边时展开
- 从边缘拖离后恢复未吸附状态
- 启动和拖动过程中不出现 runtime panic

- [ ] **Step 6: 提交本任务**

```bash
git add src-tauri/src/commands.rs src/utils/miniPlayer.test.ts
git commit -m "test: verify mini player dock state flow"
```

---

## Self-Review Checklist

- 规格中的“前端只调用一次拖拽命令”由 Task 1 与 Task 2 覆盖
- “Moved + debounce 作为最终判定依据”由 Task 2 覆盖
- “统一使用 `tauri::async_runtime::spawn`”由 Task 3 覆盖
- “保持四边吸附、延迟收起、hover 展开状态机”由 Task 4 覆盖
- 未使用 TBD/TODO 占位符
- 计划中的函数名与 spec 中名称一致：
  - `mini_player_start_dragging`
  - `mini_player_check_dock_after_drag`
  - `install_mini_player_dock_tracking`
  - `mini_player_detect_dock_after_drag`

---

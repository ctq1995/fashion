# Mini Player Edge Reveal Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为左、右、上三边的收起后唤出增加短缓冲和方向性柔和渐显动画，同时保持现有吸附稳定性不回退。

**Architecture:** Rust 继续作为收起态热区命中与真正展开的唯一控制者，在现有 hover tracking 流程中加入独立 reveal buffer 与 `reveal_session`。前端 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 只根据 dock-state 的 `collapsed: true -> false` 转换补一层方向性 reveal 动画，不参与展开判定。

**Tech Stack:** Tauri 2、Rust、Vue 3、TypeScript、Vitest

---

## File Map

- Modify: `src-tauri/src/commands.rs`
  - 为 `MiniPlayerDockState` 增加 `reveal_session`
  - 增加 reveal buffer 常量与 reveal 调度函数
  - 在收起态 hover tracking 中把“立即展开”改为“安排 reveal buffer 后再展开”
  - 保持左/右/上三边有效，底边继续禁用
- Modify: `src/MiniPlayerApp.vue`
  - 增加 reveal 动画状态
  - 在 dock-state 监听中识别 `collapsed: true -> false`
  - 按 `left` / `right` / `top` 区分动画方向
- Modify: `src/utils/miniPlayer.test.ts`
  - 为 reveal buffer / reveal session / reveal 动画增加最小源码级回归断言

### Task 1: 添加 reveal buffer 的源码级回归测试

**Files:**
- Modify: `src/utils/miniPlayer.test.ts`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 写失败测试，约束 Rust 侧 reveal buffer 和前端 reveal 动画接线**

```ts
import commandsSource from '../../src-tauri/src/commands.rs?raw';

it('adds a dedicated reveal buffer session in rust dock state handling', () => {
  expect(commandsSource).toContain('const MINI_PLAYER_REVEAL_DELAY_MS: u64 = 120;');
  expect(commandsSource).toContain('reveal_session: u64');
  expect(commandsSource).toContain('fn schedule_mini_player_reveal');
});

it('uses dock edge specific reveal animation hooks in the mini player UI', () => {
  expect(miniPlayerSource).toContain("data-reveal-from");
  expect(miniPlayerSource).toContain("revealFrom.value = event.payload.dockedEdge");
  expect(miniPlayerSource).toContain("window.setTimeout(() => {");
});
```

- [ ] **Step 2: 运行测试，确认新断言先失败**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: FAIL，提示 `commands.rs?raw` 中还不存在 reveal delay / reveal session / schedule 函数等字符串。

- [ ] **Step 3: 提交测试骨架**

```bash
git -C "E:/Polaris/music" add src/utils/miniPlayer.test.ts
git -C "E:/Polaris/music" commit -m "test: cover mini player reveal polish wiring"
```

### Task 2: 在 Rust dock state 中加入 reveal session 与 reveal buffer 调度

**Files:**
- Modify: `src-tauri/src/commands.rs:43-56`
- Modify: `src-tauri/src/commands.rs:1500-1660`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 为 reveal 增加常量与状态字段**

把 `src-tauri/src/commands.rs` 中窗口常量段改成包含 reveal 延迟：

```rust
#[cfg(windows)]
const MINI_PLAYER_DOCK_THRESHOLD: i32 = 24;
#[cfg(windows)]
const MINI_PLAYER_PEEK_SIZE: i32 = 16;
#[cfg(windows)]
const DOCK_RECHECK_DELAY_MS: u64 = 120;
#[cfg(windows)]
const MINI_PLAYER_COLLAPSE_DELAY_MS: u64 = 2_000;
#[cfg(windows)]
const MINI_PLAYER_HOVER_POLL_MS: u64 = 80;
#[cfg(windows)]
const MINI_PLAYER_HOVER_TRIGGER_DISTANCE: i32 = 28;
#[cfg(windows)]
const MINI_PLAYER_LEAVE_BUFFER_MS: u64 = 300;
#[cfg(windows)]
const MINI_PLAYER_REVEAL_DELAY_MS: u64 = 120;
```

并在 `MiniPlayerDockState` 中加入字段：

```rust
reveal_session: u64,
```

- [ ] **Step 2: 在 `reset()` 与 store 更新路径里统一刷新 reveal session**

在 `MiniPlayerDockState::reset()` 中补上：

```rust
self.reveal_session = self.reveal_session.wrapping_add(1);
```

在 `store_mini_player_dock_state(...)` 中补上：

```rust
state.reveal_session = state.reveal_session.wrapping_add(1);
```

在“展开成功”路径里也刷新 reveal session，避免旧 reveal 任务晚到继续执行。

- [ ] **Step 3: 新增 reveal 调度函数**

在 `commands.rs` 中新增：

```rust
#[cfg(windows)]
fn schedule_mini_player_reveal<R: tauri::Runtime>(window: tauri::WebviewWindow<R>) {
    let label = window.label().to_string();
    let session = {
        let Ok(mut store) = mini_player_dock_store().lock() else {
            return;
        };
        let state = store.entry(label.clone()).or_default();
        state.reveal_session = state.reveal_session.wrapping_add(1);
        state.reveal_session
    };

    tauri::async_runtime::spawn(async move {
        sleep(Duration::from_millis(MINI_PLAYER_REVEAL_DELAY_MS)).await;

        let (edge, expanded_rect, work_area_rect) = {
            let mut store = match mini_player_dock_store().lock() {
                Ok(store) => store,
                Err(_) => return,
            };
            let Some(state) = store.get_mut(&label) else {
                return;
            };
            if state.reveal_session != session || !state.collapsed {
                return;
            }
            let (Some(edge), Some(expanded_rect), Some(work_area_rect)) = (
                state.docked_edge,
                state.expanded_rect,
                state.work_area_rect,
            ) else {
                return;
            };
            (edge, expanded_rect, work_area_rect)
        };

        let (cursor_x, cursor_y) = match current_cursor_position() {
            Ok(position) => position,
            Err(_) => return,
        };
        let hover_rect = compute_collapsed_hover_rect(
            work_area_rect,
            edge,
            MINI_PLAYER_HOVER_TRIGGER_DISTANCE,
        );
        if !point_in_rect(cursor_x, cursor_y, hover_rect) {
            return;
        }

        let mut store = match mini_player_dock_store().lock() {
            Ok(store) => store,
            Err(_) => return,
        };
        let Some(state) = store.get_mut(&label) else {
            return;
        };
        if state.reveal_session != session || !state.collapsed {
            return;
        }

        if apply_rect(&window, expanded_rect).is_ok() {
            state.collapsed = false;
            state.pointer_outside_expanded = false;
            state.leave_session = state.leave_session.wrapping_add(1);
            state.collapse_session = state.collapse_session.wrapping_add(1);
            state.reveal_session = state.reveal_session.wrapping_add(1);
            emit_mini_player_dock_state(&window, state);
        }
    });
}
```

- [ ] **Step 4: 运行测试，确认源码断言开始部分通过但整体仍可能未全绿**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: reveal session / reveal delay 的断言通过；如果前端动画接线尚未实现，相关断言继续失败。

- [ ] **Step 5: 提交 Rust reveal buffer 基础设施**

```bash
git -C "E:/Polaris/music" add src-tauri/src/commands.rs src/utils/miniPlayer.test.ts
git -C "E:/Polaris/music" commit -m "feat: add mini player reveal buffer state"
```

### Task 3: 把收起态 hover restore 从“立即展开”改为“短缓冲展开”

**Files:**
- Modify: `src-tauri/src/commands.rs:2567-2634`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 先补源码级断言，要求 hover 跟踪使用 reveal 调度而不是直接展开**

在 `src/utils/miniPlayer.test.ts` 追加：

```ts
it('routes collapsed hover restore through reveal scheduling instead of immediate expansion', () => {
  expect(commandsSource).toContain('schedule_mini_player_reveal(window.clone());');
  expect(commandsSource).not.toContain('if apply_rect(&window, expanded_rect).is_ok() {
                            state.collapsed = false;');
});
```

如果字符串跨行太脆弱，则改成更稳的两条断言：

```ts
expect(commandsSource).toContain('schedule_mini_player_reveal(window.clone());');
expect(commandsSource).toContain('if !point_in_rect(cursor_x, cursor_y, hover_rect)');
```

- [ ] **Step 2: 运行测试，确认新断言先失败**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: FAIL，提示 collapsed hover restore 还没有走 reveal 调度。

- [ ] **Step 3: 在收起态 hover 分支改成 reveal 调度**

把当前直接展开片段：

```rust
if point_in_rect(cursor_x, cursor_y, hover_rect) {
    if apply_rect(&window, expanded_rect).is_ok() {
        state.collapsed = false;
        state.pointer_outside_expanded = false;
        state.leave_session = state.leave_session.wrapping_add(1);
        state.collapse_session = state.collapse_session.wrapping_add(1);
        emit_mini_player_dock_state(&window, state);
    }
}
```

改成：

```rust
if point_in_rect(cursor_x, cursor_y, hover_rect) {
    let reveal_session = state.reveal_session;
    drop(store);
    schedule_mini_player_reveal(window.clone());
    let store = match mini_player_dock_store().lock() {
        Ok(store) => store,
        Err(_) => break,
    };
    let Some(state) = store.get(&label) else {
        break;
    };
    if state.reveal_session == reveal_session {
        continue;
    }
    continue;
}
```

核心要求：

- 鼠标进入热区时只安排 reveal，不直接展开
- 不要在同一轮轮询里持锁后直接 `apply_rect`
- reveal 安排后尽快 `continue`

- [ ] **Step 4: 运行 Rust 编译检查**

Run: `cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`
Expected: PASS，无 reveal 相关 borrow 或 session 错误。

- [ ] **Step 5: 提交 reveal 调度接入 hover restore**

```bash
git -C "E:/Polaris/music" add src-tauri/src/commands.rs src/utils/miniPlayer.test.ts
git -C "E:/Polaris/music" commit -m "feat: soften mini player edge reveal timing"
```

### Task 4: 为前端增加定向 reveal 动画态

**Files:**
- Modify: `src/MiniPlayerApp.vue:120-380`
- Test: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 先写失败测试，约束前端 reveal 动画状态字段**

在 `src/utils/miniPlayer.test.ts` 追加：

```ts
it('tracks edge specific reveal animation state in mini player UI', () => {
  expect(miniPlayerSource).toContain("const revealFrom = ref<'left' | 'right' | 'top' | null>(null);");
  expect(miniPlayerSource).toContain(":data-reveal-from=\"revealFrom ?? ''\"");
  expect(miniPlayerSource).toContain("const wasCollapsed = collapsed.value;");
});
```

- [ ] **Step 2: 运行测试，确认断言先失败**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: FAIL，提示 revealFrom 和相关数据属性尚未出现。

- [ ] **Step 3: 在脚本部分增加 reveal 动画状态与定时清理**

在 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 中增加：

```ts
const revealFrom = ref<'left' | 'right' | 'top' | null>(null);
let revealAnimationTimer: ReturnType<typeof window.setTimeout> | null = null;
```

新增辅助函数：

```ts
function clearRevealAnimation() {
  if (revealAnimationTimer) {
    window.clearTimeout(revealAnimationTimer);
    revealAnimationTimer = null;
  }
  revealFrom.value = null;
}

function triggerRevealAnimation(edge: MiniPlayerDockStatePayload['dockedEdge']) {
  if (edge !== 'left' && edge !== 'right' && edge !== 'top') {
    clearRevealAnimation();
    return;
  }
  clearRevealAnimation();
  revealFrom.value = edge;
  revealAnimationTimer = window.setTimeout(() => {
    revealFrom.value = null;
    revealAnimationTimer = null;
  }, 180);
}
```

在 dock-state 监听中改成：

```ts
cleanupMiniDockStateListener = await listen<MiniPlayerDockStatePayload>(MINI_PLAYER_DOCK_STATE_EVENT, (event) => {
  const wasCollapsed = collapsed.value;
  dockedEdge.value = event.payload.dockedEdge;
  collapsed.value = event.payload.collapsed;

  if (wasCollapsed && !event.payload.collapsed) {
    triggerRevealAnimation(event.payload.dockedEdge);
  } else if (event.payload.collapsed) {
    clearRevealAnimation();
  }
});
```

在 `onBeforeUnmount` 中补上：

```ts
clearRevealAnimation();
```

- [ ] **Step 4: 在模板和样式中接入方向属性与 reveal 动画**

给根节点增加属性：

```vue
:data-reveal-from="revealFrom ?? ''"
```

把统一 reveal 动画从：

```css
.mini-player-card {
  animation: miniPlayerReveal 160ms ease-out;
}
```

改成“首屏 reveal + dock reveal 分离”：

```css
.mini-player-card {
  animation: miniPlayerReveal 160ms ease-out;
}

.mini-player-shell[data-reveal-from='left'] .mini-player-card {
  animation: miniPlayerRevealFromLeft 180ms ease-out;
}

.mini-player-shell[data-reveal-from='right'] .mini-player-card {
  animation: miniPlayerRevealFromRight 180ms ease-out;
}

.mini-player-shell[data-reveal-from='top'] .mini-player-card {
  animation: miniPlayerRevealFromTop 180ms ease-out;
}
```

增加 keyframes：

```css
@keyframes miniPlayerRevealFromLeft {
  from {
    opacity: 0.18;
    transform: translateX(-10px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes miniPlayerRevealFromRight {
  from {
    opacity: 0.18;
    transform: translateX(10px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes miniPlayerRevealFromTop {
  from {
    opacity: 0.18;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

- [ ] **Step 5: 运行前端测试，确认 reveal 动画接线通过**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: PASS，包含 reveal 动画和 reveal buffer 的全部源码断言。

- [ ] **Step 6: 提交前端 reveal 动画**

```bash
git -C "E:/Polaris/music" add src/MiniPlayerApp.vue src/utils/miniPlayer.test.ts
git -C "E:/Polaris/music" commit -m "feat: add directional mini player reveal animation"
```

### Task 5: 全量验证并手动回归

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/MiniPlayerApp.vue`
- Modify: `src/utils/miniPlayer.test.ts`

- [ ] **Step 1: 运行 Rust 编译检查**

Run: `cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`
Expected: PASS

- [ ] **Step 2: 运行前端最小回归测试**

Run: `npx vitest run "E:/Polaris/music/src/utils/miniPlayer.test.ts"`
Expected: PASS

- [ ] **Step 3: 手动验证左/右/上三边 reveal 手感**

Run: `npm run tauri dev`

手动检查：

1. 左侧收起后，鼠标快速掠过边缘，不误展开
2. 左侧收起后，鼠标稳定靠近边缘，约 `120ms` 后柔和展开
3. 右侧收起后，能以从右向左的轻位移动画展开
4. 顶部收起后，能以从上向下的轻位移动画展开
5. 展开后鼠标离开，仍按原逻辑延迟收起
6. 将展开态迷你窗口拖离吸附范围，不会再次被错误吸回

Expected: 三边 reveal 更柔和，且无稳定性回退。

- [ ] **Step 4: 提交最终验证结果**

```bash
git -C "E:/Polaris/music" add src-tauri/src/commands.rs src/MiniPlayerApp.vue src/utils/miniPlayer.test.ts
git -C "E:/Polaris/music" commit -m "feat: polish mini player edge reveal"
```

## Self-Review

- Spec coverage: 已覆盖 reveal buffer、reveal session、三边定向动画、底边继续禁用、验证场景。
- Placeholder scan: 无 TBD / TODO / “稍后实现” 类占位；每个任务包含明确文件、命令和代码片段。
- Type consistency: `reveal_session`、`schedule_mini_player_reveal`、`revealFrom`、`MiniPlayerDockStatePayload['dockedEdge']` 在各任务中命名一致。

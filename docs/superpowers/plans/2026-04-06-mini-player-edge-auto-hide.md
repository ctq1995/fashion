# 迷你播放器贴边自动收起 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为迷你播放器增加四边吸附、延时自动收起、悬停展开与离开后二次收起能力，并保持现有播放与窗口交互不回归。

**Architecture:** 以 Rust 原生窗口状态机为核心，在 [commands.rs](src-tauri/src/commands.rs) 中补齐迷你播放器贴边状态、几何计算与悬停轮询，在 [lib.rs](src-tauri/src/lib.rs) 中注册命令并把状态附着到迷你播放器窗口生命周期上。前端 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 只负责在拖动结束后通知 Rust 重新评估吸附状态，并接收最小状态事件做样式标记，不承担窗口几何计算。

**Tech Stack:** Vue 3、TypeScript、Tauri 2、Rust、Tokio、Vite、Vitest、cargo test

---

## 文件结构

### 预计修改文件
- Modify: `src-tauri/src/commands.rs`
  - 增加迷你播放器贴边状态结构、边缘判定纯函数、收起/展开调度、悬停轮询和新的 Tauri 命令
- Modify: `src-tauri/src/lib.rs`
  - 注册新的迷你播放器贴边命令，在创建迷你播放器窗口后初始化状态，并在窗口隐藏/关闭时清理状态
- Modify: `src/utils/miniPlayer.ts`
  - 增加迷你播放器贴边状态事件名和前端调用命令需要的常量
- Modify: `src/MiniPlayerApp.vue`
  - 在原生拖动结束后通知 Rust 检查是否吸附；监听贴边状态事件并把状态映射为最小 UI 标记

### 依赖但不修改的现有文件
- Reference: `src-tauri/src/window_chrome.rs`
  - 保持当前原生圆角与阴影策略不变，仅确认收起时不会破坏窗口外观
- Reference: `src-tauri/src/main.rs`
  - 入口无需改动，只确认命令仍由 `lib.rs` 注册
- Reference: `docs/superpowers/specs/2026-04-06-mini-player-edge-auto-hide-design.md`
  - 作为本计划覆盖依据

## Task 1: 为贴边几何逻辑补纯函数与失败测试

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: 写出边缘判定与位置计算的失败测试**

在 `src-tauri/src/commands.rs` 末尾新增测试模块，先写 5 个失败测试，覆盖最近边、吸附位置、收起位置、夹紧与等距优先级：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: u32, height: u32) -> WindowRect {
        WindowRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn pick_nearest_edge_prefers_left_when_equal() {
        let work_area = rect(0, 0, 1920, 1080);
        let window = rect(12, 12, 420, 164);

        let edge = pick_docked_edge(window, work_area, 24);

        assert_eq!(edge, Some(DockedEdge::Left));
    }

    #[test]
    fn pick_nearest_edge_returns_none_when_outside_threshold() {
        let work_area = rect(0, 0, 1920, 1080);
        let window = rect(80, 120, 420, 164);

        let edge = pick_docked_edge(window, work_area, 24);

        assert_eq!(edge, None);
    }

    #[test]
    fn docked_bounds_snap_to_bottom_and_clamp_x() {
        let work_area = rect(0, 0, 1920, 1080);
        let window = rect(1700, 980, 420, 164);

        let docked = compute_docked_rect(window, work_area, DockedEdge::Bottom);

        assert_eq!(docked.x, 1500);
        assert_eq!(docked.y, 916);
        assert_eq!(docked.width, 420);
        assert_eq!(docked.height, 164);
    }

    #[test]
    fn collapsed_bounds_leave_peek_on_right_edge() {
        let work_area = rect(0, 0, 1920, 1080);
        let expanded = rect(1500, 420, 420, 164);

        let collapsed = compute_collapsed_rect(expanded, work_area, DockedEdge::Right, 10);

        assert_eq!(collapsed.x, 1910);
        assert_eq!(collapsed.y, 420);
    }

    #[test]
    fn clamp_rect_keeps_window_inside_work_area() {
        let work_area = rect(0, 0, 1280, 720);
        let window = rect(1100, 680, 420, 164);

        let clamped = clamp_rect_to_work_area(window, work_area);

        assert_eq!(clamped.x, 860);
        assert_eq!(clamped.y, 556);
    }
}
```

- [ ] **Step 2: 运行 Rust 单测确认失败**

Run:

```bash
cargo test --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml" pick_nearest_edge_prefers_left_when_equal -- --exact
```

Expected: 编译失败或测试失败，提示 `WindowRect`、`DockedEdge`、`pick_docked_edge` 等尚未定义。

- [ ] **Step 3: 写出几何数据结构与最小实现**

在 `src-tauri/src/commands.rs` 靠近窗口命令区域新增以下代码，先只实现纯函数，不接入窗口命令：

```rust
#[cfg(windows)]
const MINI_PLAYER_DOCK_THRESHOLD: i32 = 24;
#[cfg(windows)]
const MINI_PLAYER_PEEK_SIZE: i32 = 10;
#[cfg(windows)]
const DOCK_RECHECK_DELAY_MS: u64 = 120;
#[cfg(windows)]
const MINI_PLAYER_COLLAPSE_DELAY_MS: u64 = 1_000;
#[cfg(windows)]
const MINI_PLAYER_HOVER_POLL_MS: u64 = 80;

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockedEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[cfg(windows)]
impl WindowRect {
    fn right(self) -> i32 {
        self.x + self.width as i32
    }

    fn bottom(self) -> i32 {
        self.y + self.height as i32
    }
}

#[cfg(windows)]
fn clamp_rect_to_work_area(window: WindowRect, work_area: WindowRect) -> WindowRect {
    let max_x = work_area.right() - window.width as i32;
    let max_y = work_area.bottom() - window.height as i32;
    WindowRect {
        x: window.x.clamp(work_area.x, max_x),
        y: window.y.clamp(work_area.y, max_y),
        width: window.width,
        height: window.height,
    }
}

#[cfg(windows)]
fn pick_docked_edge(window: WindowRect, work_area: WindowRect, threshold: i32) -> Option<DockedEdge> {
    let distances = [
        (DockedEdge::Left, (window.x - work_area.x).abs()),
        (DockedEdge::Right, (work_area.right() - window.right()).abs()),
        (DockedEdge::Top, (window.y - work_area.y).abs()),
        (DockedEdge::Bottom, (work_area.bottom() - window.bottom()).abs()),
    ];

    distances
        .into_iter()
        .filter(|(_, distance)| *distance <= threshold)
        .min_by_key(|(edge, distance)| {
            let priority = match edge {
                DockedEdge::Left => 0,
                DockedEdge::Right => 1,
                DockedEdge::Top => 2,
                DockedEdge::Bottom => 3,
            };
            (*distance, priority)
        })
        .map(|(edge, _)| edge)
}

#[cfg(windows)]
fn compute_docked_rect(window: WindowRect, work_area: WindowRect, edge: DockedEdge) -> WindowRect {
    let clamped = clamp_rect_to_work_area(window, work_area);
    match edge {
        DockedEdge::Left => WindowRect { x: work_area.x, ..clamped },
        DockedEdge::Right => WindowRect { x: work_area.right() - clamped.width as i32, ..clamped },
        DockedEdge::Top => WindowRect { y: work_area.y, ..clamped },
        DockedEdge::Bottom => WindowRect { y: work_area.bottom() - clamped.height as i32, ..clamped },
    }
}

#[cfg(windows)]
fn compute_collapsed_rect(expanded: WindowRect, work_area: WindowRect, edge: DockedEdge, peek: i32) -> WindowRect {
    match edge {
        DockedEdge::Left => WindowRect { x: work_area.x - expanded.width as i32 + peek, ..expanded },
        DockedEdge::Right => WindowRect { x: work_area.right() - peek, ..expanded },
        DockedEdge::Top => WindowRect { y: work_area.y - expanded.height as i32 + peek, ..expanded },
        DockedEdge::Bottom => WindowRect { y: work_area.bottom() - peek, ..expanded },
    }
}
```

- [ ] **Step 4: 运行全部纯逻辑测试确认通过**

Run:

```bash
cargo test --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml" --lib commands::tests
```

Expected: 新增的 5 个纯逻辑测试全部通过。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "test: cover mini player dock geometry"
```

## Task 2: 在 Rust 中接入迷你播放器贴边状态机

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: 为贴边状态写出失败测试目标清单**

先在 `src-tauri/src/commands.rs` 的测试模块顶部加入本任务的行为清单注释：

```rust
// 目标行为：
// 1. 拖动结束命令会读取当前窗口位置，靠近边缘时吸附到最近边
// 2. 吸附后等待 1 秒自动收起，只保留 10px
// 3. 鼠标进入可见热区时展开到 expanded rect
// 4. 鼠标离开展开窗口后再次开始延时收起
// 5. 隐藏/关闭窗口会取消轮询与计时器并重置状态
```

- [ ] **Step 2: 定义贴边状态结构与序列化事件载荷**

在 `src-tauri/src/commands.rs` 中新增：

```rust
#[cfg(windows)]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct MiniPlayerDockPayload {
    docked_edge: Option<&'static str>,
    collapsed: bool,
}

#[cfg(windows)]
#[derive(Debug, Default)]
struct MiniPlayerDockState {
    docked_edge: Option<DockedEdge>,
    collapsed: bool,
    expanded_rect: Option<WindowRect>,
    collapsed_rect: Option<WindowRect>,
    hover_session: u64,
    collapse_session: u64,
}
```

并补充两个方法：

```rust
#[cfg(windows)]
impl MiniPlayerDockState {
    fn reset(&mut self) {
        self.docked_edge = None;
        self.collapsed = false;
        self.expanded_rect = None;
        self.collapsed_rect = None;
        self.hover_session = self.hover_session.wrapping_add(1);
        self.collapse_session = self.collapse_session.wrapping_add(1);
    }

    fn payload(&self) -> MiniPlayerDockPayload {
        MiniPlayerDockPayload {
            docked_edge: self.docked_edge.map(|edge| match edge {
                DockedEdge::Left => "left",
                DockedEdge::Right => "right",
                DockedEdge::Top => "top",
                DockedEdge::Bottom => "bottom",
            }),
            collapsed: self.collapsed,
        }
    }
}
```

- [ ] **Step 3: 增加窗口矩形读取、应用和事件广播辅助函数**

在 `src-tauri/src/commands.rs` 中新增只在 Windows 下编译的辅助函数：

```rust
#[cfg(windows)]
fn rect_from_window(window: &Window) -> Result<WindowRect, String> {
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
fn rect_from_work_area(window: &Window) -> Result<WindowRect, String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| String::from("monitor not found"))?;
    let work_area = monitor.work_area();
    Ok(WindowRect {
        x: work_area.position.x,
        y: work_area.position.y,
        width: work_area.size.width,
        height: work_area.size.height,
    })
}

#[cfg(windows)]
fn apply_rect(window: &Window, rect: WindowRect) -> Result<(), String> {
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(rect.x, rect.y)))
        .map_err(|e| e.to_string())
}

#[cfg(windows)]
fn emit_mini_player_dock_state(window: &Window, state: &MiniPlayerDockState) {
    let _ = window.emit("mini-player:dock-state", state.payload());
}
```

- [ ] **Step 4: 增加拖动结束后检查吸附的命令**

在 `src-tauri/src/commands.rs` 新增命令，并在实现中做 120ms 稳定等待：

```rust
#[cfg(windows)]
#[tauri::command]
pub async fn mini_player_check_dock_after_drag(window: Window) -> Result<(), String> {
    if window.label() != "mini-player" {
        return Ok(());
    }

    sleep(Duration::from_millis(DOCK_RECHECK_DELAY_MS)).await;

    let current = rect_from_window(&window)?;
    let work_area = rect_from_work_area(&window)?;
    let Some(edge) = pick_docked_edge(current, work_area, MINI_PLAYER_DOCK_THRESHOLD) else {
        reset_mini_player_dock_state(&window);
        return Ok(());
    };

    let expanded = compute_docked_rect(current, work_area, edge);
    let collapsed = compute_collapsed_rect(expanded, work_area, edge, MINI_PLAYER_PEEK_SIZE);

    apply_rect(&window, expanded)?;
    store_mini_player_dock_state(&window, edge, expanded, collapsed)?;
    schedule_mini_player_collapse(window.clone());
    start_mini_player_hover_tracking(window);
    Ok(())
}
```

在同一文件补齐被调用的辅助函数 `store_mini_player_dock_state`、`reset_mini_player_dock_state`、`schedule_mini_player_collapse`、`start_mini_player_hover_tracking`，其中：
- `store_*` 负责写入状态并广播 `mini-player:dock-state`
- `reset_*` 负责取消会话并广播未吸附状态
- `schedule_*` 负责使用 `tokio::spawn` 和会话号避免旧定时器误执行
- `start_*` 负责启动一次悬停轮询循环

- [ ] **Step 5: 在收起与展开逻辑里补最小实现**

确保 `schedule_mini_player_collapse` 和 `start_mini_player_hover_tracking` 的核心逻辑至少包含以下代码：

```rust
#[cfg(windows)]
fn point_in_rect(x: i32, y: i32, rect: WindowRect) -> bool {
    x >= rect.x && x < rect.right() && y >= rect.y && y < rect.bottom()
}
```

以及轮询中的状态切换：

```rust
if state.collapsed {
    if let Some(collapsed_rect) = state.collapsed_rect {
        if point_in_rect(cursor_x, cursor_y, collapsed_rect) {
            apply_rect(&window, state.expanded_rect.unwrap())?;
            state.collapsed = false;
            emit_mini_player_dock_state(&window, &state);
        }
    }
} else if let Some(expanded_rect) = state.expanded_rect {
    if !point_in_rect(cursor_x, cursor_y, expanded_rect) {
        schedule_mini_player_collapse(window.clone());
    }
}
```

获取全局鼠标位置时，使用现有 `windows` crate 补 `GetCursorPos`，失败时直接跳出轮询并保留展开窗口，避免窗口卡死在边缘外。

- [ ] **Step 6: 运行 Rust 编译检查与逻辑测试**

Run:

```bash
cargo test --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml" --lib commands::tests && cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected: 纯逻辑测试通过，`cargo check` 通过，新增命令与状态机无编译错误。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: add mini player dock state machine"
```

## Task 3: 在应用入口注册命令并清理窗口生命周期状态

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 写出本任务的验证目标**

在修改前先记录这 3 个验证目标：

```rust
// 目标：
// 1. mini_player_check_dock_after_drag 已在 invoke_handler 注册
// 2. 迷你播放器隐藏或销毁时会清理贴边状态
// 3. 主窗口逻辑与托盘逻辑不回归
```

- [ ] **Step 2: 在 invoke_handler 中注册新命令**

把 `commands::mini_player_check_dock_after_drag` 加到 [lib.rs](src-tauri/src/lib.rs) 现有命令列表中，放在其它窗口命令附近：

```rust
            commands::window_hide,
            commands::window_show,
            commands::window_set_always_on_top,
            commands::mini_player_check_dock_after_drag,
            commands::emit_app_event,
```

- [ ] **Step 3: 在迷你播放器窗口创建后初始化和清理状态**

在 `ensure_mini_player_window` 返回窗口前后，调用你在 `commands.rs` 中提供的初始化/清理辅助函数；如果不想暴露普通函数跨模块访问，就新增命令式辅助函数并在窗口事件回调内调用。

要求覆盖两个点：

```rust
if let Some(window) = app.get_webview_window(MINI_PLAYER_WINDOW_LABEL) {
    let cloned = window.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed | tauri::WindowEvent::CloseRequested { .. }) {
            commands::reset_mini_player_dock_state_for_window(&cloned);
        }
    });
}
```

以及在主动 `hide()` 前后也做同样清理，避免下次显示时仍停在边缘外。

- [ ] **Step 4: 运行 Rust 编译检查确认入口接线正确**

Run:

```bash
cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected: `lib.rs` 能正常访问新增命令或辅助函数，没有模块可见性错误。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/commands.rs
git commit -m "feat: wire mini player dock lifecycle"
```

## Task 4: 前端在拖动结束后触发贴边检查并同步状态

**Files:**
- Modify: `src/utils/miniPlayer.ts`
- Modify: `src/MiniPlayerApp.vue`

- [ ] **Step 1: 先写前端行为目标清单**

在 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 的 `<script setup>` 临近迷你窗状态定义处先补行为注释：

```ts
// 目标行为：
// 1. 开始原生拖动后，鼠标松开时通知 Rust 检查吸附
// 2. 收到 dock-state 事件时，更新 dockedEdge/collapsed
// 3. 组件卸载时移除事件监听，避免重复注册
```

- [ ] **Step 2: 在常量文件中加入贴边事件名**

把 [miniPlayer.ts](src/utils/miniPlayer.ts) 扩展为：

```ts
export const MINI_PLAYER_DOCK_STATE_EVENT = 'mini-player:dock-state';
```

保留现有常量不变。

- [ ] **Step 3: 在 MiniPlayerApp.vue 中增加最小响应式状态与监听**

在 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 的脚本区新增：

```ts
type DockedEdge = 'left' | 'right' | 'top' | 'bottom' | null;

const dockedEdge = ref<DockedEdge>(null);
const collapsed = ref(false);
let cleanupDockStateListener: null | (() => void) = null;
let removeDragEndListener: null | (() => void) = null;

function applyDockState(payload: { dockedEdge: DockedEdge; collapsed: boolean }) {
  dockedEdge.value = payload.dockedEdge;
  collapsed.value = payload.collapsed;
}
```

并把模板根节点从：

```vue
<main
  class="mini-player-shell"
  :data-theme="ui.theme"
>
```

改成：

```vue
<main
  class="mini-player-shell"
  :data-theme="ui.theme"
  :data-docked-edge="dockedEdge ?? undefined"
  :data-collapsed="collapsed ? 'true' : 'false'"
>
```

- [ ] **Step 4: 给 startDragging 增加拖动结束后的检查桥接**

把现有 `startDragging` 改成下面这种结构，保留 `data-no-drag` 判断：

```ts
function scheduleDockCheckAfterDrag() {
  removeDragEndListener?.();

  const handlePointerUp = async () => {
    window.removeEventListener('mouseup', handlePointerUp);
    removeDragEndListener = null;
    try {
      await invoke('mini_player_check_dock_after_drag');
    } catch (error) {
      console.error('mini player dock check failed', error);
    }
  };

  removeDragEndListener = () => window.removeEventListener('mouseup', handlePointerUp);
  window.addEventListener('mouseup', handlePointerUp, { once: true });
}

async function startDragging(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest('[data-no-drag]')) return;
  try {
    scheduleDockCheckAfterDrag();
    await invoke('window_start_dragging');
  } catch (error) {
    removeDragEndListener?.();
    removeDragEndListener = null;
    console.error('mini player start dragging failed', error);
  }
}
```

- [ ] **Step 5: 在 onMounted/onBeforeUnmount 中接入 dock-state 事件**

在 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 已有 `onMounted` / `onBeforeUnmount` 基础上补上：

```ts
  cleanupDockStateListener = await listen<{ dockedEdge: DockedEdge; collapsed: boolean }>(
    MINI_PLAYER_DOCK_STATE_EVENT,
    (event) => {
      applyDockState(event.payload);
    },
  );
```

以及：

```ts
  cleanupDockStateListener?.();
  cleanupDockStateListener = null;
  removeDragEndListener?.();
  removeDragEndListener = null;
```

- [ ] **Step 6: 给收起态补最小样式反馈**

在 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 样式区增加一段最小差异样式，不改变布局，只弱化卡片感：

```css
.mini-player-shell[data-collapsed='true'] .mini-player-card {
  backdrop-filter: blur(16px);
}

.mini-player-shell[data-docked-edge='left'] .mini-player-card,
.mini-player-shell[data-docked-edge='right'] .mini-player-card,
.mini-player-shell[data-docked-edge='top'] .mini-player-card,
.mini-player-shell[data-docked-edge='bottom'] .mini-player-card {
  transition: backdrop-filter 140ms ease, opacity 140ms ease;
}
```

- [ ] **Step 7: 运行前端构建确认类型与模板通过**

Run:

```bash
npm run build
```

Expected: 前端构建通过，没有 `listen` 泛型、模板属性或新常量导入错误。

- [ ] **Step 8: Commit**

```bash
git add src/MiniPlayerApp.vue src/utils/miniPlayer.ts
git commit -m "feat: bridge mini player dock state to ui"
```

## Task 5: 完整验证四边吸附、收起、展开与回退

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/MiniPlayerApp.vue`
- Modify: `src/utils/miniPlayer.ts`

- [ ] **Step 1: 运行完整自动验证**

Run:

```bash
npm run build && cargo test --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml" --lib commands::tests && cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"
```

Expected: 三个命令全部成功，没有 TS、Rust 编译或单测失败。

- [ ] **Step 2: 做四边手动验证并记录结果**

按下面顺序手动运行应用并验证：

```bash
npm run tauri dev
```

验证清单：
- 左边缘：拖到左侧，松手后吸附，约 1 秒后收起，仅露 10px，鼠标靠近可展开
- 右边缘：行为与左边缘对称
- 上边缘：向上收起，仅露底部 10px
- 下边缘：向下收起，仅露顶部 10px
- 展开后移开鼠标：再次延时收起
- 点击“关闭迷你播放器”或“回到主窗口”：下次重新打开不是边缘外位置
- 远离边缘松手：不进入吸附与自动收起

Expected: 每项都符合规格；若任一项失败，回到对应任务修正，不要继续提交完成结论。

- [ ] **Step 3: 若手动验证发现问题，做最小修正并重新验证**

若出现以下问题，按最小改动修正：

```text
- 吸附时读到旧位置：增大 DOCK_RECHECK_DELAY_MS 到 160
- 收起后无法展开：检查 collapsed rect 计算和 point_in_rect 边界
- 离开窗口后频繁抖动：在 schedule_mini_player_collapse 中用 session 防止重复定时器
- 下次显示仍是收起态：确认 hide/close 路径调用 reset_mini_player_dock_state_for_window
```

修正后重新执行 Step 1 和 Step 2 的全部验证。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src/MiniPlayerApp.vue src/utils/miniPlayer.ts
git commit -m "feat: finish mini player edge auto-hide"
```

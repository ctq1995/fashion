# Mini Player Dock Stability Design

## 背景

当前迷你播放器贴边功能已经具备基础能力：

- 靠近四个边缘时可判定吸附
- 吸附后延迟收起，仅保留一条可见边
- 鼠标移回露出的边时重新展开
- 前端可接收 `mini-player:dock-state` 状态事件

但现阶段实现仍有两个稳定性问题：

1. 贴边检测路径存在重复触发：拖拽命令结束后会触发一次检测，窗口 `Moved` 事件 debounce 后又会再触发一次检测。
2. 内部窗口类型和调度路径曾经出现漂移：`Window` / `WebviewWindow<R>` 混用，以及 `tokio::spawn` 在非 Tokio reactor 上下文中调用导致 panic。

本次设计目标不是重做功能，而是在现有实现基础上收稳架构，确保拖拽、贴边、收起、悬停展开这一整条链路由 Rust 侧统一管理，并且调度安全、类型一致、行为单一。

## 目标

本次设计要达成以下结果：

- 迷你播放器拖拽结束后，贴边检测只通过一条 Rust 内部路径生效
- `Moved` 事件作为拖拽结束后的最终判定依据
- 前端继续只调用一次 `invoke('mini_player_start_dragging')`
- 所有 mini-player 贴边内部 helper 统一使用 `WebviewWindow<R>`
- 所有异步调度统一使用 `tauri::async_runtime::spawn`
- 保持现有四边吸附、延迟收起、hover 展开能力不变

## 非目标

本次不包含以下内容：

- 不引入 Win32 AppBar 模型
- 不改造成基于 `WM_MOUSEHOVER` / `TrackMouseEvent` 的原生鼠标消息方案
- 不新增前端参与的贴边判定逻辑
- 不重写几何计算规则
- 不拆分为新的独立 Rust 模块文件

## 方案选型

本次采用“在现有实现上收稳”的方案。

### 采用原因

- 当前代码已经具备大部分功能，只需收紧执行路径
- 直接在现有 `commands.rs` 基础上整理，改动面最小
- 相比抽新模块，当前优先级是先恢复稳定行为和减少回归面
- 相比更原生的 Win32 hover 方案，实现成本更低且足以满足当前需求

## 总体设计

### 1. 前端职责

前端 [MiniPlayerApp.vue](src/MiniPlayerApp.vue) 保持最小职责：

- 在可拖拽区域触发 `startDragging`
- 调用一次 `invoke('mini_player_start_dragging')`
- 监听 `mini-player:dock-state` 并更新 UI 属性

前端不负责：

- 拖拽结束后的贴边判定
- 悬停展开判定
- 收起定时控制
- 窗口几何计算

### 2. Rust 命令层职责

Rust 命令入口继续保留：

- `mini_player_start_dragging(window: Window)`
- `mini_player_check_dock_after_drag(window: Window)`

其中：

- 命令层只作为对外入口，参数仍使用 `Window`
- 命令进入后，立即从 `window.app_handle()` 中取回对应 `WebviewWindow`
- 之后全部交给统一的内部 helper

这样可保证：

- 命令接口不受泛型影响
- 内部实现不再混用 `Window` 与 `WebviewWindow<R>`

### 3. 内部 helper 统一类型

以下内部函数统一收敛到 `WebviewWindow<R>`：

- 几何读取：窗口位置、尺寸、工作区读取
- 几何应用：窗口位置/尺寸设置
- dock state 事件发射
- dock state 存储与重置
- 收起调度
- hover 跟踪
- dock 检测
- moved debounce 调度

统一后，内部执行链不再做 `Window` / `WebviewWindow<R>` 的往返切换。

### 4. 贴边检测单一路径

`mini_player_detect_dock_after_drag(...)` 作为唯一贴边检测入口。

要求：

- 所有最终贴边判定都收口到这个函数
- 不允许存在第二套重复几何判定逻辑

调用来源只保留两类：

1. `Moved` 事件 debounce 后触发
2. 兼容保留命令 `mini_player_check_dock_after_drag` 间接调用

而 `mini_player_start_dragging` 只负责启动拖拽，不再在拖拽返回后直接做第二次检测。

这样可以避免：

- 命令触发一次检测
- `Moved` 再触发一次检测
- 两套异步检测互相覆盖窗口状态

### 5. `Moved` + debounce 作为最终判定依据

在 [lib.rs](src-tauri/src/lib.rs) 初始化 mini-player 窗口后，调用内部安装函数，为 mini-player 注册 `WindowEvent::Moved` 监听。

行为规则：

- 每收到一次 `Moved` 事件，刷新 `move_session`
- 启动一次 debounce 调度
- debounce 到期后，仅当 `move_session` 未变化，才执行实际检测

这个机制的意义是：

- 拖拽过程中会有连续移动事件，不会过早判定
- 只有最后一次移动稳定后才真正读取最终坐标
- 不依赖 `start_dragging()` 返回时机

### 6. 收起与 hover 行为

吸附成功后：

- 计算 `expanded_rect`
- 计算 `collapsed_rect`
- 存储到 dock state
- 启动 collapse 调度
- 启动 hover 跟踪

行为规则：

- 若当前为展开态且鼠标离开 expanded 区域，则重新安排 collapse
- 若当前为收起态且鼠标进入 collapsed 可触达区域，则恢复 expanded
- 每次新的展开/收起调度都通过 session 号使旧任务失效，防止过期任务覆盖新状态

## 调度设计

所有异步后台任务统一使用：

- `tauri::async_runtime::spawn`

适用场景：

- collapse 延迟任务
- moved debounce 检测任务
- hover 跟踪循环

禁止再使用：

- `tokio::spawn`

原因：

- 窗口事件回调不保证运行在 Tokio reactor 中
- `tokio::spawn` 可能在运行时直接 panic
- `tauri::async_runtime::spawn` 更适合当前 Tauri 应用上下文

## 数据与状态

继续沿用现有 `MiniPlayerDockState`，包含：

- `docked_edge`
- `collapsed`
- `expanded_rect`
- `collapsed_rect`
- `hover_session`
- `collapse_session`
- `move_session`

状态要求：

- `reset()` 必须同时让各类 session 失效
- 新的 dock/hover/collapse 调度必须刷新对应 session
- 所有异步任务执行前都必须先验证 session 是否仍然有效

## 错误处理

采用最小错误处理原则：

- 如果 mini-player 对应窗口不存在，命令返回明确错误字符串
- 如果 monitor/work area 无法读取，检测函数返回错误并停止本次判定
- 如果窗口已不在 store 中或 session 已失效，后台任务直接退出
- 不额外增加用户可见提示

## 测试与验证

### 自动化验证

保留并补强 [miniPlayer.test.ts](src/utils/miniPlayer.test.ts) 的接线测试，确保：

- 前端绑定拖拽处理函数
- 前端调用 `mini_player_start_dragging`
- Rust 注册对应命令
- Rust 安装 moved-event based dock tracking

### 编译验证

每次调整后必须运行：

- `cargo check --manifest-path "src-tauri/Cargo.toml"`
- `npx vitest run "src/utils/miniPlayer.test.ts"`

### 手动验证

需要在桌面端手动确认：

1. 拖到左边吸附
2. 拖到右边吸附
3. 拖到上边吸附
4. 拖到下边吸附
5. 吸附后延迟收起
6. 鼠标回到露出的边时展开
7. 从边缘拖离后恢复未吸附状态

## 实施约束

本次实现必须遵守：

- 不新增前端贴边判定逻辑
- 不新增新的重复检测函数
- 不拆新文档外的额外文件
- 不对现有几何算法做无关重构
- 不扩展为系统级 AppBar 行为

## 预期结果

完成后，mini-player 贴边能力应具备以下稳定性特征：

- 触发路径唯一，调试时可明确定位问题
- 窗口移动最终位置才会触发吸附判定
- 不再因 runtime 上下文问题触发 panic
- 内部类型统一，避免 `Window` / `WebviewWindow<R>` 反复错配
- 保持现有 UI 和交互语义不变，仅提升稳定性

# 图标系统实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复系统托盘无图标问题，并将应用、品牌与各平台打包图标统一为未来极简科幻风图标系统。

**Architecture:** 以统一 SVG 源图为核心，分别维护主图标与托盘小尺寸图标，再通过生成流程覆盖 Tauri bundle 与移动端图标资源。Rust 侧托盘构建改为显式设置图标，避免依赖默认窗口图标继承。

**Tech Stack:** Vue 3, Tauri 2, Rust, SVG, Tauri icon generation tooling, npm scripts / cargo check

---

## 文件结构与职责

- Modify: `src-tauri/app-icon.svg`
  - 主应用图标源图，服务于桌面与打包主图标。
- Modify: `src/assets/fashion-brand.svg`
  - 前端品牌展示图，与主图标统一视觉语言。
- Create or Modify: `src-tauri/tray-icon.svg`
  - 托盘专用小尺寸源图，优先保证 16-24px 识别度。
- Modify: `src-tauri/src/lib.rs`
  - 在 TrayIconBuilder 中显式设置托盘图标。
- Modify: `src-tauri/icons/**`
  - 由统一源图生成的各平台图标资源。
- Modify: `src-tauri/tauri.conf.json`
  - 如需补充或调整图标声明，保持与生成产物一致。
- Create or Modify: `package.json`
  - 如需增加图标生成命令，则在此记录脚本入口。

### Task 1: 重做统一主图标与品牌图

**Files:**
- Modify: `src-tauri/app-icon.svg`
- Modify: `src/assets/fashion-brand.svg`

- [ ] **Step 1: 重写主图标 SVG 为未来极简科幻风**

将 `src-tauri/app-icon.svg` 改为统一主徽记，要求：
- 深空黑 / 石墨黑背景
- 冰青高亮
- 主体同时表达播放语义与音乐节奏感
- 保留大尺寸可见的少量切面和流线

目标结构示例：
```svg
<svg width="128" height="128" viewBox="0 0 128 128" fill="none" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="18" y1="14" x2="112" y2="116" gradientUnits="userSpaceOnUse">
      <stop stop-color="#050816"/>
      <stop offset="0.55" stop-color="#0B1630"/>
      <stop offset="1" stop-color="#0E2E45"/>
    </linearGradient>
    <linearGradient id="core" x1="34" y1="28" x2="96" y2="98" gradientUnits="userSpaceOnUse">
      <stop stop-color="#DDFBFF"/>
      <stop offset="0.48" stop-color="#78F7FF"/>
      <stop offset="1" stop-color="#18D7F2"/>
    </linearGradient>
    <linearGradient id="glow" x1="80" y1="22" x2="102" y2="48" gradientUnits="userSpaceOnUse">
      <stop stop-color="#9CF7FF" stop-opacity="0.95"/>
      <stop offset="1" stop-color="#9CF7FF" stop-opacity="0"/>
    </linearGradient>
  </defs>
  <rect x="10" y="10" width="108" height="108" rx="30" fill="#02040B"/>
  <rect x="14" y="14" width="100" height="100" rx="26" fill="url(#bg)"/>
  <path d="M31 92C44 73 61 61.5 85 54" stroke="#7DEBFF" stroke-opacity="0.2" stroke-width="5.2" stroke-linecap="round"/>
  <path d="M38 34L38 85.5C38 91.4 42.8 96.2 48.7 96.2C54.5 96.2 59.3 91.4 59.3 85.5V50.7L86.4 43.8V71.7C86.4 77.4 91 82 96.7 82C102.4 82 107 77.4 107 71.7V29.8C107 26.6 104 24.2 100.9 25L49.9 37.7C43.2 39.3 38 45.3 38 52.2V34Z" fill="url(#core)"/>
  <path d="M67 58.5L95.8 51.4" stroke="#04121E" stroke-width="7" stroke-linecap="round" stroke-opacity="0.38"/>
  <circle cx="48.7" cy="85.5" r="10" fill="#EBFDFF"/>
  <circle cx="96.7" cy="71.7" r="9.2" fill="#D9FBFF"/>
  <path d="M77 23C85 23 92.6 26.6 98.1 32.9" stroke="url(#glow)" stroke-width="5" stroke-linecap="round"/>
</svg>
```

- [ ] **Step 2: 将前端品牌图同步为同一视觉语言**

把 `src/assets/fashion-brand.svg` 重写为与主图标同构但更适合前端展示的版本，要求：
- 与 `app-icon.svg` 使用同一主徽记
- 可在前端中直接显示，不再保留旧的绿色品牌风格
- 允许背景和高光略微更通透，但不能改变主符号

- [ ] **Step 3: 自查源图一致性**

人工检查：
- `app-icon.svg` 与 `fashion-brand.svg` 的主轮廓是否一致
- 是否都符合“深底 + 冰青高亮 + 极简科幻”风格
- 是否未引入过多细碎装饰

### Task 2: 增加托盘专用图标并接入 Rust

**Files:**
- Create: `src-tauri/tray-icon.svg`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建托盘专用 SVG 源图**

新增 `src-tauri/tray-icon.svg`，要求：
- 仅保留主徽记轮廓
- 去掉复杂流线、厚渐变和小装饰
- 优先适配 Windows 托盘暗色背景
- 在 16-24px 下仍清晰

目标结构示例：
```svg
<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
  <rect x="6" y="6" width="52" height="52" rx="16" fill="#07111F"/>
  <path d="M20 18V43.5C20 46.5 22.4 49 25.5 49C28.6 49 31 46.5 31 43.5V27.8L43.9 24.5V38.1C43.9 41.1 46.3 43.5 49.3 43.5C52.4 43.5 54.8 41.1 54.8 38.1V17C54.8 15.3 53.2 14.1 51.5 14.5L26.1 20.8C22.7 21.6 20 24.7 20 28.2V18Z" fill="#9CF7FF"/>
  <circle cx="25.5" cy="43.5" r="5.1" fill="#EAFDFF"/>
  <circle cx="49.3" cy="38.1" r="4.7" fill="#D8FBFF"/>
</svg>
```

- [ ] **Step 2: 读取并设置托盘图标**

在 `src-tauri/src/lib.rs` 的 `build_system_tray(...)` 中，为 `TrayIconBuilder` 增加显式 icon 设置。

实现要求：
- 优先尝试从新增托盘资源加载图标
- 如果当前 Tauri 运行时更适合复用默认窗口图标，则仍需保证显式传入 builder
- 不保留“完全不设置 icon”的路径

代码调整目标示例：
```rust
let tray_icon = app.default_window_icon().cloned();

let mut tray_builder = TrayIconBuilder::new()
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_tray_icon_event(|tray, event| {
        // existing code
    })
    .on_menu_event(|app, event| {
        // existing code
    });

if let Some(icon) = tray_icon {
    tray_builder = tray_builder.icon(icon);
}

tray_builder.build(app).map_err(|error| error.to_string())?;
```

如果需要从文件路径载入图标，则在本任务中补充对应读取逻辑，但仍保持函数职责只聚焦“托盘 icon 设置”。

- [ ] **Step 3: 运行 Rust 编译检查**

Run:
`cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`

Expected:
- PASS
- 不出现 `TrayIconBuilder` 或 icon API 类型错误

### Task 3: 生成并覆盖各尺寸图标资源

**Files:**
- Modify: `src-tauri/icons/**`
- Modify if needed: `package.json`
- Modify if needed: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 确认当前项目可用的图标生成方式**

检查现有工具链是否可直接基于 `src-tauri/app-icon.svg` 生成图标资源。优先使用 Tauri 官方图标生成能力。

期望使用的命令形态：
```bash
npm exec tauri icon "src-tauri/app-icon.svg"
```
或等价的 Tauri 2 CLI 命令。

- [ ] **Step 2: 如有必要，在 package.json 增加脚本入口**

如果当前项目没有便捷命令，在 `package.json` 增加类似脚本：
```json
{
  "scripts": {
    "tauri:icon": "tauri icon src-tauri/app-icon.svg"
  }
}
```

要求：
- 只增加与图标生成直接相关的脚本
- 不改动无关脚本

- [ ] **Step 3: 生成并覆盖桌面与平台图标资源**

执行图标生成命令，覆盖：
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.ico`
- `src-tauri/icons/icon.icns`
- `src-tauri/icons/android/**`
- `src-tauri/icons/ios/**`

要求：
- 不遗漏当前仓库已有链路
- 若生成器自动补充更多尺寸，可保留

- [ ] **Step 4: 校验 tauri.conf.json 图标声明**

检查 `src-tauri/tauri.conf.json` 中：
```json
"bundle": {
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ]
}
```

要求：
- 路径仍然有效
- 如果生成器产物名称变化，则同步修正声明

### Task 4: 运行完整验证

**Files:**
- Verify generated: `src-tauri/icons/**`
- Verify Rust: `src-tauri/src/lib.rs`
- Verify source assets: `src-tauri/app-icon.svg`, `src/assets/fashion-brand.svg`, `src-tauri/tray-icon.svg`

- [ ] **Step 1: 检查关键资源是否存在**

确认以下文件存在：
- `src-tauri/app-icon.svg`
- `src/assets/fashion-brand.svg`
- `src-tauri/tray-icon.svg`
- `src-tauri/icons/icon.ico`
- `src-tauri/icons/icon.icns`
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`

- [ ] **Step 2: 运行前端构建**

Run:
`npm run build`

Expected:
- PASS
- 前端资源引用未因品牌 SVG 变更而报错

- [ ] **Step 3: 再次运行 Rust 编译检查**

Run:
`cargo check --manifest-path "E:/Polaris/music/src-tauri/Cargo.toml"`

Expected:
- PASS
- 托盘相关修改编译通过

- [ ] **Step 4: 汇总人工验收点**

人工确认：
- Windows 托盘有图标
- 托盘图标在暗色任务栏下可辨识
- 主窗口图标已更新
- 前端品牌图视觉已统一
- 各尺寸图标资源链路完整

## 计划自检

### Spec 覆盖检查
- 托盘图标修复：Task 2
- 主图标与品牌图重设计：Task 1
- 各尺寸与各平台图标覆盖：Task 3
- 构建与资源校验：Task 4

### 占位符检查
- 未使用 TBD / TODO / later 等占位符。
- 所有任务都给出明确文件路径和执行目标。

### 一致性检查
- 主图标源图：`src-tauri/app-icon.svg`
- 品牌图：`src/assets/fashion-brand.svg`
- 托盘源图：`src-tauri/tray-icon.svg`
- 托盘修复入口：`src-tauri/src/lib.rs`
- 各尺寸产物：`src-tauri/icons/**`

以上命名与任务内引用保持一致。

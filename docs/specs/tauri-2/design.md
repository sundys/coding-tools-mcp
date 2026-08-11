# 设计：Windows 关闭确认与后台运行

## 概述

本设计在 Windows 主窗口关闭事件与现有 Tauri 运行循环之间增加轻量拦截，在 Svelte 根布局展示定制确认提示，并通过系统托盘提供后台恢复入口。MCP、Actions 和隧道状态机不参与关闭决策，后台运行依靠主进程持续存活。

## 对应需求

覆盖 FR-1、FR-2、FR-3、FR-4、FR-5、FR-6。

## 现状证据

- `src-tauri/src/lib.rs:66-136` 的 `run` 构建 Tauri 应用并在 `RunEvent::ExitRequested` 中仅处理 WebView 重建保护。
- `src-tauri/src/main.rs:3-5` 调用库入口 `coding_tools_mcp_desktop_lib::run`。
- `src/routes/+layout.svelte:1-104` 管理全局 `AppShell`、工作区刷新和 UI memory guard，适合注册一次窗口事件监听。
- `src-tauri/capabilities/default.json:1-11` 已启用 `core:default` 与 `dialog:default`，新增操作通过 Rust command，避免扩展进程插件权限。
- `src-tauri/Cargo.toml:1-12` 使用 Tauri 2，未声明 tray feature 或额外进程插件。

## 技术方案

### 组件与数据流

```text
Windows CloseRequested
  -> Tauri on_window_event: prevent_close + emit("app-close-requested")
  -> +layout.svelte listen: app-close-requested
  -> ClosePrompt.svelte
      Cancel: hide prompt
      Background: invoke("handle_close_action", "background") -> window.hide()
      Direct close: invoke("handle_close_action", "exit") -> app.exit(0)

TrayIcon event
  -> Open main window: show + set_focus
  -> Exit application: app.exit(0)
```

### Rust 设计

`lib.rs` 增加 `CloseAction` 枚举和 `handle_close_action` command。`CloseAction::Background` 只对主窗口执行 `hide`；`CloseAction::Exit` 调用 `AppHandle::exit(0)`。命令返回 `Result<(), String>`，前端可将错误显示为 Toast。

在 `Builder::setup` 中使用 `TrayIconBuilder` 创建托盘菜单。菜单项 ID 固定为 `show-window` 与 `exit-app`，事件回调只执行窗口显示/聚焦或 `app.exit(0)`。`on_window_event` 仅对 label 为 `main` 且事件为 `CloseRequested` 的窗口调用 `prevent_close` 并发出 `app-close-requested`；其他窗口事件保持默认行为。托盘与关闭拦截代码使用 `#[cfg(target_os = "windows")]`，命令本身跨平台编译以保持 invoke 注册一致。

应用已有 `commands` 模块聚合 command。新增命令放在 `commands/app_lifecycle.rs` 并从 `commands/mod.rs` re-export，避免继续膨胀入口文件；该模块只依赖 Tauri 类型，不触碰 `AppState` 或运行时锁。

### Svelte 设计

新增 `src/lib/components/ClosePrompt.svelte`，使用 `role="dialog"`、`aria-modal="true"`、标题和说明文本。组件接收 `open`，通过回调通知 `cancel`、`background`、`exit`。按钮顺序和颜色与截图一致：取消为浅色描边，后台运行为浅色描边，直接关闭为红色实心。

`+layout.svelte` 在 `onMount` 中调用 Tauri `listen("app-close-requested")`，将 `closePromptOpen` 置为 true；卸载时取消监听。操作回调先关闭提示，再调用生命周期 command。命令失败时调用现有 `showToast`，并恢复提示框状态，保证用户不会被静默隐藏。

### 权限与兼容性

使用 `window.hide`、`window.show`、`window.set_focus` 需在 `core:default` 已包含的窗口权限范围内；若 Tauri capability 校验要求显式权限，则在 `default.json` 增加对应 `core:window:allow-*` 项。托盘菜单使用现有 `src-tauri/icons/icon.ico`，不新增二进制资源。

## 文件结构

```text
src-tauri/src/
  lib.rs                         Windows close event 与 tray 初始化
  commands/
    mod.rs                       生命周期 command 导出
    app_lifecycle.rs             CloseAction 与 handle_close_action
src/
  routes/+layout.svelte          全局关闭事件监听与操作编排
  lib/components/ClosePrompt.svelte
  app.css                        关闭提示样式
```

## 测试策略

- Rust 单元测试覆盖 `CloseAction` 的序列化值和无效 action 错误。
- 前端静态检查覆盖 ClosePrompt 的可访问属性、按钮文案和事件回调类型。
- Windows 手测：标题栏关闭、取消、后台运行、托盘打开、托盘退出、普通最小化；确认 MCP/Actions/隧道状态在后台运行后保持。
- CI 运行 `npm run check`、`npm run build`、`cargo check`、`cargo test`；release workflow 运行 Windows NSIS 与 macOS universal DMG 构建。

## 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 提示实现 | Svelte 自定义模态 | 截图是定制布局，且跨窗口事件可复用；原生 dialog 无法匹配按钮样式 |
| 关闭控制 | Rust command | 避免引入进程插件和额外 capability，退出路径可绕过 CloseRequested |
| 后台入口 | Tauri tray-icon | 进程继续运行且可从 Windows 系统托盘恢复，符合需求文字 |
| 状态持久化 | 不持久化 | 截图只定义当前关闭决策，不要求记忆选择 |

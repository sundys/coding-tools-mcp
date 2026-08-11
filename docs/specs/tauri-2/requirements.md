# 需求规格：Windows 关闭确认与后台运行

## 功能概述

当前 Tauri 应用收到 Windows 主窗口关闭请求后会直接结束进程。MCP、Actions 和隧道服务因此被一并停止，用户无法通过系统托盘继续使用后台服务。`screen.png` 定义了新的中文确认提示外观和三个操作。

## 需求列表

### FR-1 拦截 Windows 主窗口关闭请求

应用在 Windows 主窗口收到用户关闭请求时，必须阻止窗口立即销毁和进程退出，并向前端发送一次关闭请求事件。前端显示与 `screen.png` 一致的模态提示：标题为“关闭 Coding Tools MCP？”，说明为“选择后台运行可隐藏窗口并保持 MCP、Actions 和隧道服务继续运行，之后可通过系统托盘重新打开。”，操作为“取消”“后台运行”“直接关闭”。

**EARS：** WHEN 用户点击 Windows 主窗口关闭按钮，THE APPLICATION SHALL 阻止默认关闭并显示唯一的关闭确认提示。

验收标准：

- 点击窗口标题栏关闭按钮后，进程仍存在且提示框可见。
- 提示框打开期间，重复的关闭请求不会创建第二个提示框或触发退出。
- 仅 Windows 主窗口触发该流程，普通最小化仍由 Windows 执行原有的任务栏最小化行为。

### FR-2 取消关闭

点击“取消”必须关闭提示框并保持主窗口可见、可交互，运行中的 MCP、Actions 和隧道服务不得停止。

**EARS：** WHEN 用户选择“取消”，THE APPLICATION SHALL 关闭提示并保持窗口、进程和服务状态不变。

### FR-3 后台运行

点击“后台运行”必须关闭提示框并隐藏主窗口，不退出应用进程，不停止任何运行时服务。应用必须创建系统托盘图标，用户可从托盘重新显示并聚焦主窗口。

**EARS：** WHEN 用户选择“后台运行”，THE APPLICATION SHALL 隐藏主窗口、保留后台服务并允许从系统托盘恢复窗口。

验收标准：

- 主窗口从桌面和任务栏隐藏，但进程和服务仍运行。
- 托盘菜单包含打开主窗口和退出应用；打开操作显示并聚焦主窗口。
- 后台运行后再次点击主窗口关闭按钮仍显示同一确认提示。

### FR-4 直接关闭

点击“直接关闭”必须结束 Tauri 应用进程。Tauri 的退出路径不得再次显示关闭确认提示，服务由进程退出流程统一回收。

**EARS：** WHEN 用户选择“直接关闭”，THE APPLICATION SHALL 直接退出进程且不再次显示关闭确认。

### FR-5 托盘退出与恢复

系统托盘必须在应用启动时创建，并使用项目现有应用图标。托盘“打开主窗口”操作显示并聚焦主窗口；托盘“退出应用”操作直接退出，不显示确认提示。托盘菜单事件处理不得阻塞 Tauri 主线程。

**EARS：** WHILE 应用在后台运行，THE APPLICATION SHALL 提供托盘打开与退出操作。

### FR-6 非 Windows 行为兼容

Linux 和 macOS 构建必须继续通过现有检查和测试。Windows 专属窗口拦截与托盘实现应使用条件编译；非 Windows 不改变现有退出行为。

**EARS：** WHERE 目标平台不是 Windows，THE APPLICATION SHALL 保持现有窗口关闭行为并通过跨平台编译。

## 非功能需求

- 提示框使用现有 Svelte 设计 token，响应式布局在 960px 最小窗口宽度下不溢出。
- 关闭操作通过类型明确的 Tauri IPC 命令完成，前端不依赖未授权的进程插件。
- 版本号从 `0.1.33` 递增到 `0.1.34`，所有发布版本源保持一致。
- 现有 `npm run check`、`npm run build`、`cargo check` 和 Rust 测试继续通过。

## 范围外

- 不增加“记住选择”或持久化关闭策略。
- 不实现开机自启、托盘气泡通知或自定义托盘图标资产。
- 不改变 MCP、Actions、FRP/Cloudflare 的运行时状态机。

## 依赖关系

- Tauri 2 的 window event、tray-icon、menu 和 IPC command API。
- Svelte 5 的组件回调与 `onMount` 清理机制。
- 现有 `AppState`、MCP、Actions 和 tunnel 生命周期保持不变，仅依赖进程继续存活。
- GitHub Actions 内置 `GITHUB_TOKEN` 与当前 release workflow 的 `contents: write` 权限。

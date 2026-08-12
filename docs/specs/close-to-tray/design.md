# 设计文档：close-to-tray

## 概述

对应需求 FR-1～FR-6：关窗时用应用内三按钮确认替代直接退出；「后台运行」隐藏到托盘并保持 MCP/Actions/隧道；同一进程内记住后台运行选择；「直接关闭」与托盘「退出」真退出；与 `recreate_ui_webview` 隐藏态协同；Windows 二次启动尽量唤起已有窗口。

## 技术方案

### 关闭拦截（Rust 统一决策）

Rust 在 `RunEvent::WindowEvent(CloseRequested)` 统一处理主窗口关闭请求：

1. 当 `ALLOW_EXIT` 为 false 且未在重建 UI 时，调用 `api.prevent_close()` 阻止销毁。
2. 未记住后台运行选择时，向前端 emit `close-requested`，由根 layout 打开应用内三按钮对话框。
3. 已记住后台运行选择时，直接调用 `hide_to_tray`，不再 emit 弹窗事件。

前端 `close-guard` 只监听 Rust 的 `close-requested`，不再额外注册 `onCloseRequested`。这样可避免同一次关闭由前后端重复处理，也保证进程内选择的判断只有一个权威来源。

### 进程内关闭选择记忆

- `window_chrome` 使用进程内原子状态保存是否已选择「后台运行」，初始值为未选择。
- `hide_to_tray` 成功隐藏主窗口后才写入该状态，避免隐藏失败却跳过后续确认。
- Rust `CloseRequested` 兜底先阻止默认销毁；若已记住「后台运行」，直接再次调用托盘隐藏命令，不再 emit `close-requested`；否则维持现有弹窗路径。
- 状态不落盘，因此 WebView 重建后仍由 Rust 主进程保留，桌面进程退出后由操作系统自然清空。
- 「取消」不写状态；「直接关闭」会立即结束进程，新进程仍从未选择状态开始。

### 托盘（Rust）

- `Cargo.toml`: `tauri = { version = "2", features = ["tray-icon"] }`
- `setup` 中 `TrayIconBuilder`：默认窗口图标、tooltip、菜单「显示窗口」「退出」
- 左键 Up → `show_main_window`
- 菜单 show → `show_main_window`；quit → 设 `ALLOW_EXIT` 后 `app.exit(0)`

命令：

| 命令 | 作用 |
|------|------|
| `hide_to_tray` | hide 主窗；确保托盘已创建 |
| `show_main_window` | show + unminimize + set_focus |
| `quit_app` | 设允许退出标志后关闭主窗 / `app.exit(0)` |

### UI 重建协同

`recreate_ui_webview`：

1. 记录 `was_hidden = !window.is_visible()`（在 destroy 前）。
2. 现有 keepalive / prevent_exit 不变。
3. 重建后：若 `was_hidden`，则 `hide()` 且不要 `set_focus`；否则保持现有 show/focus/minimize 逻辑。
4. CloseRequested 兜底在 `UI_RECREATING==true` 时不 `prevent_close`（允许 destroy）。

### 二次启动（Windows）

`acquire_single_instance` 发现已存在实例时：向已有进程发唤起信号（例如命名事件 / 窗口消息）。最小实现：用已存在的 mutex 名配套一个 named event；主实例后台线程 wait 后 `show_main_window`。若唤起失败，至少不再静默 return（可 eprintln）。macOS 可后续用 `tauri-plugin-single-instance`；本次以 Windows 为主。

### 对话框 UI

应用内 modal（Svelte）：

- 标题：关闭 Coding Tools MCP?
- 说明：选择后台运行可隐藏窗口并保持 MCP、Actions 和隧道服务继续运行，之后可通过系统托盘重新打开。
- 按钮：取消 | 后台运行 | 直接关闭（危险样式）

挂在根 layout，全局一次。

## 关键决策

1. **应用内三按钮 modal，不用系统 dialog**：系统 ask/confirm 难以稳定做出与设计稿一致的三按钮。
2. **真退出设允许标志再关窗**：避免 CloseRequested 再次拦截。
3. **托盘隐藏用 hide 而非 minimize**：与 taskbar 最小化语义分离，减少与 0.1.32/33 修复路径冲突。
4. **二次启动 Should**：尽力做 Windows 唤起，不阻塞主路径。
5. **记忆归 Rust 主进程所有**：避免 `localStorage` 跨进程残留，也避免仅存在前端模块中而被 WebView 重建清空。

## 文件结构

| 文件 | 变更 |
|------|------|
| `src-tauri/Cargo.toml` | tray-icon feature |
| `src-tauri/src/lib.rs` | tray setup、window event、commands |
| `src-tauri/src/commands/window_chrome.rs`（新建） | hide/show/quit + 标志 |
| `src-tauri/src/commands/ui_memory.rs` | was_hidden 恢复 |
| `src-tauri/capabilities/default.json` | 如需补 core 权限 |
| `src/lib/components/CloseConfirmDialog.svelte`（新建） | 三按钮 |
| `src/lib/close-guard.ts`（新建） | onCloseRequested 绑定 |
| `src/lib/api/window-chrome.ts`（新建） | invoke 封装 |
| `src/routes/+layout.svelte` | 挂载 dialog + close-guard |
| `src/app.css` | modal 样式 |

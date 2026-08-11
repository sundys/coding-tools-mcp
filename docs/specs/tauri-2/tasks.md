# 任务清单：Windows 关闭确认与后台运行

## 交付物清单

- 实际新建文件数：5 个（1 个 Rust 生命周期模块、1 个 Svelte 组件、3 个规格文件）。
- 实际修改文件数：8 个（Rust 入口/commands/Cargo、Svelte 根布局、Tauri 配置和版本锁文件）。
- 预计新增或修改函数数：约 8 个。
- 交付物：
  1. `src-tauri/src/commands/app_lifecycle.rs`
  2. `src/lib/components/ClosePrompt.svelte`
  3. Windows close event 与系统托盘初始化
  4. `0.1.34` 版本源和 release 构建校验

## 任务列表

### 阶段 1：规格与上下文

- [x] 1.1 分析 Tauri 入口、前端布局和 release workflows，锁定 Windows 专属边界。
  - 证据块：`src-tauri/src/lib.rs:66-136` 仅处理 WebView 重建退出保护；`src/routes/+layout.svelte:1-104` 是全局布局；`.github/workflows/release.yml` 已包含 Windows NSIS、macOS universal 和 GitHub Release 发布。
  - 涉及文件：文档与上述源码只读。
  - 需求：FR-1、FR-5、FR-6；设计：现状证据。

### 阶段 2：核心实现

- [x] 2.1 增加 Rust close-action command，并在 Windows close event 中阻止退出、发出前端事件。
  - 证据块：`src-tauri/src/lib.rs:129-135` 现有 `ExitRequested` 分支可扩展为相邻窗口事件处理；`src-tauri/src/commands/mod.rs` 聚合所有 IPC command。
  - 涉及文件：`src-tauri/src/commands/app_lifecycle.rs`（约 45 行）、`src-tauri/src/commands/mod.rs`（约 3 行）、`src-tauri/src/lib.rs`（约 35 行）、`src-tauri/Cargo.toml`（约 2 行）。
  - 需求：FR-1、FR-3、FR-4；设计：Rust 设计。

- [x] 2.2 实现系统托盘菜单，支持打开主窗口和直接退出。
  - 证据块：`src-tauri/icons/icon.ico` 已存在；当前 `setup` 在 `src-tauri/src/lib.rs:84-91` 负责全局初始化，可在此创建托盘。
  - 涉及文件：`src-tauri/src/lib.rs`（与 2.1 合并，约 35 行）。
  - 需求：FR-3、FR-5；设计：Rust 设计。

- [x] 2.3 实现截图样式的 Svelte 关闭提示并接入窗口事件监听。
  - 证据块：`src/routes/+layout.svelte:80-104` 的 `onMount` 返回清理函数；`src/lib/stores/toast.ts` 提供错误反馈模式；`src/app.css` 提供设计 token。
  - 涉及文件：`src/lib/components/ClosePrompt.svelte`（约 190 行）、`src/routes/+layout.svelte`（约 58 行）。
  - 需求：FR-1、FR-2、FR-3、FR-4；设计：Svelte 设计。

- [x] 2.4 将应用版本递增至 0.1.34，并核对 release workflow 的 npm lockfile、Tauri 构建参数和 contents 写权限。
  - 证据块：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 当前均为 `0.1.33`；`.github/workflows/release.yml` 使用 `npm ci` 和 `softprops/action-gh-release@v2`。
  - 涉及文件：版本源、`package-lock.json`、`src-tauri/Cargo.lock`、`.github/workflows/release.yml`（必要时）。
  - 需求：FR-6；设计：权限与兼容性。

### 阶段 3：验证

- [x] 3.1 运行前端检查与生产构建，确认提示组件无 TypeScript/Svelte 错误。
  - 证据块：现有开发文档要求 `npm run check`、`npm run build`。
  - 涉及文件：命令输出与 `build/` 产物。
  - 需求：FR-1 至 FR-6；设计：测试策略。

- [ ] 3.2 运行 Rust `cargo fmt --check`、`cargo check` 和 `cargo test`，确认 Windows 条件编译和现有运行时回归。
  - 证据块：`docs/project-context/how-to-test.md` 定义 cargo test 为 Rust 基线。
  - 涉及文件：`src-tauri` 测试目标。
  - 需求：FR-3、FR-4、FR-6；设计：测试策略。

- [ ] 3.3 执行 GitNexus detect-changes，检查只影响预期的 close lifecycle、UI 和 release 流程；使用 tag 构建触发 fork 仓库 Release workflow。
  - 证据块：编辑前 GitNexus `run` upstream 风险 LOW；`.github/workflows/release.yml` 的发布 job 要求两个构建 job 成功并具有 `contents: write`。
  - 涉及文件：Git diff、Actions run 和 Release 页面。
  - 需求：FR-5、FR-6；设计：测试策略。

## 需求覆盖矩阵

| 需求 | 设计章节 | 任务 | 状态 |
|------|----------|------|------|
| FR-1 | Rust 设计、Svelte 设计 | 2.1、2.3 | 未开始 |
| FR-2 | Svelte 设计 | 2.3 | 未开始 |
| FR-3 | 组件与数据流、Rust 设计 | 2.1、2.2、2.3 | 未开始 |
| FR-4 | Rust 设计 | 2.1、2.3 | 未开始 |
| FR-5 | Rust 设计 | 2.2 | 未开始 |
| FR-6 | 权限与兼容性 | 2.4、3.1、3.2、3.3 | 未开始 |

## 文件变更清单

| 文件 | 操作 | 行数预算 | 说明 |
|------|------|----------|------|
| `src-tauri/src/commands/app_lifecycle.rs` | 新建 | 45 | close action command |
| `src-tauri/src/commands/mod.rs` | 修改 | 3 | 注册 command 模块 |
| `src-tauri/src/lib.rs` | 修改 | 35 | tray、window event、command 注册 |
| `src/lib/components/ClosePrompt.svelte` | 新建 | 190 | 可访问模态提示与局部样式 |
| `src/routes/+layout.svelte` | 修改 | 58 | 事件监听与回调 |
| 版本源与 workflow | 修改 | 20 | 0.1.34 与 release 校验 |

## 完成检查

- [ ] 所有 FR 都有实现和验证证据。
- [ ] 取消、后台运行、直接关闭、托盘恢复和托盘退出均有手测路径。
- [ ] `npm ci` 可在 fork 的 GitHub Actions 中执行，Release job 只使用 GitHub 内置 token。
- [ ] 全文无未完成占位符。

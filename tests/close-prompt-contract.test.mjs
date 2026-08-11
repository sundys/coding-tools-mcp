import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const promptPath = new URL("../src/lib/components/ClosePrompt.svelte", import.meta.url);
const layoutPath = new URL("../src/routes/+layout.svelte", import.meta.url);
const lifecyclePath = new URL(
  "../src-tauri/src/commands/app_lifecycle.rs",
  import.meta.url,
);
const entryPath = new URL("../src-tauri/src/lib.rs", import.meta.url);

test("关闭提示保留截图定义的标题、说明和三个操作", async () => {
  const source = await readFile(promptPath, "utf8");

  assert.match(source, /role="dialog"/);
  assert.match(source, /关闭 Coding Tools MCP？/);
  assert.match(source, /选择后台运行可隐藏窗口并保持 MCP、Actions 和隧道服务继续运行/);
  assert.match(source, />\s*取消\s*<\/button>/);
  assert.match(source, />\s*后台运行\s*<\/button>/);
  assert.match(source, />\s*直接关闭\s*<\/button>/);
});

test("根布局监听关闭事件并将两个非取消操作映射到 IPC action", async () => {
  const source = await readFile(layoutPath, "utf8");

  assert.match(source, /listen\("app-close-requested"/);
  assert.match(source, /invoke\("handle_close_action", \{ action \}\)/);
  assert.match(source, /handleCloseAction\("background"\)/);
  assert.match(source, /handleCloseAction\("exit"\)/);
});

test("Rust 生命周期契约限制 action，并拦截 Windows close event", async () => {
  const lifecycle = await readFile(lifecyclePath, "utf8");
  const entry = await readFile(entryPath, "utf8");

  assert.match(lifecycle, /enum CloseAction/);
  assert.match(lifecycle, /Background/);
  assert.match(lifecycle, /Exit/);
  assert.match(entry, /api\.prevent_close\(\)/);
  assert.match(entry, /window\.emit\("app-close-requested"/);
  assert.match(entry, /"show-window"/);
  assert.match(entry, /"exit-app"/);
});

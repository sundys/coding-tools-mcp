import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const [windowChrome, appLifecycle, closeGuard] = await Promise.all([
  readFile("src-tauri/src/commands/window_chrome.rs", "utf8"),
  readFile("src-tauri/src/lib.rs", "utf8"),
  readFile("src/lib/close-guard.ts", "utf8"),
]);

test("后台运行选择只保存在 Rust 进程内存中", () => {
  assert.match(windowChrome, /static CLOSE_PREFERENCE: ProcessClosePreference/);
  assert.match(windowChrome, /AtomicBool::new\(false\)/);
  assert.doesNotMatch(windowChrome, /localStorage|DataStore|write\(|save\(/);

  const hideSucceeded = windowChrome.indexOf(".hide()");
  const remembered = windowChrome.indexOf("CLOSE_PREFERENCE.remember_background()");
  assert.ok(hideSucceeded >= 0 && remembered > hideSucceeded);
});

test("后续关闭直接复用后台运行选择且失败时恢复弹窗", () => {
  assert.match(
    appLifecycle,
    /should_run_in_background_on_close\(\)[\s\S]*hide_to_tray\(app_handle\.clone\(\)\)/,
  );
  assert.match(
    appLifecycle,
    /hide_to_tray\(app_handle\.clone\(\)\)[\s\S]*emit\("close-requested"/,
  );
});

test("前端只响应 Rust 的关闭确认事件，避免重复弹窗", () => {
  assert.match(closeGuard, /listen\("close-requested"/);
  assert.doesNotMatch(closeGuard, /onCloseRequested/);
});

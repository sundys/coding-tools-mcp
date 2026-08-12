use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, WebviewWindow};

use crate::error::{AppError, AppResult};

/// When true, CloseRequested may destroy the window / exit the process.
static ALLOW_EXIT: AtomicBool = AtomicBool::new(false);

/// Process-lifetime preference. It intentionally never touches persistent storage:
/// a brand-new desktop process must ask the user again.
struct ProcessClosePreference(AtomicBool);

impl ProcessClosePreference {
    const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    fn remember_background(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    fn should_run_in_background(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

static CLOSE_PREFERENCE: ProcessClosePreference = ProcessClosePreference::new();

/// Intercept user close (show confirm / keep running) unless quitting or UI recreate.
pub fn should_intercept_close() -> bool {
    !ALLOW_EXIT.load(Ordering::SeqCst) && !crate::commands::ui_memory::should_prevent_exit()
}

/// Whether a previous close in this process selected "run in background".
pub fn should_run_in_background_on_close() -> bool {
    CLOSE_PREFERENCE.should_run_in_background()
}

fn main_window(app: &AppHandle) -> AppResult<WebviewWindow> {
    app.get_webview_window("main")
        .or_else(|| {
            app.webview_windows()
                .into_iter()
                .find(|(label, _)| !label.starts_with("__"))
                .map(|(_, w)| w)
        })
        .ok_or_else(|| AppError::Message("找不到主窗口".into()))
}

#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> AppResult<()> {
    let window = main_window(&app)?;
    window
        .hide()
        .map_err(|err| AppError::Message(format!("隐藏窗口失败: {err}")))?;
    CLOSE_PREFERENCE.remember_background();
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> AppResult<()> {
    let window = main_window(&app)?;
    let _ = window.unminimize();
    window
        .show()
        .map_err(|err| AppError::Message(format!("显示窗口失败: {err}")))?;
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> AppResult<()> {
    arm_allow_exit();
    app.exit(0);
    Ok(())
}

pub fn arm_allow_exit() {
    ALLOW_EXIT.store(true, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::ProcessClosePreference;

    #[test]
    fn close_preference_is_empty_for_each_new_process_state() {
        let preference = ProcessClosePreference::new();
        assert!(!preference.should_run_in_background());

        preference.remember_background();
        assert!(preference.should_run_in_background());

        let restarted_process_preference = ProcessClosePreference::new();
        assert!(!restarted_process_preference.should_run_in_background());
    }
}

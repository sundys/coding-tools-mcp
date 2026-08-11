use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    Background,
    Exit,
}

#[tauri::command]
pub fn handle_close_action(app: AppHandle, action: CloseAction) -> AppResult<()> {
    match action {
        CloseAction::Background => {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| AppError::Message("main window is unavailable".to_string()))?;
            window.hide().map_err(|error| {
                AppError::Message(format!("failed to hide main window: {error}"))
            })
        }
        CloseAction::Exit => {
            app.exit(0);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CloseAction;

    #[test]
    fn close_action_accepts_frontend_contract_values() {
        assert_eq!(
            serde_json::from_str::<CloseAction>(r#""background""#).unwrap(),
            CloseAction::Background
        );
        assert_eq!(
            serde_json::from_str::<CloseAction>(r#""exit""#).unwrap(),
            CloseAction::Exit
        );
    }

    #[test]
    fn close_action_rejects_unknown_values() {
        assert!(serde_json::from_str::<CloseAction>(r#""cancel""#).is_err());
    }
}

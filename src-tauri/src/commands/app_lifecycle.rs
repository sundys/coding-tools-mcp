use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    Background,
    Exit,
}

#[derive(Default)]
pub struct ClosePreferenceState(Mutex<Option<CloseAction>>);

impl ClosePreferenceState {
    fn remember(&self, action: CloseAction) -> AppResult<()> {
        let mut preference = self.0.lock().map_err(|_| {
            AppError::Message("close preference state is unavailable".to_string())
        })?;
        *preference = Some(action);
        Ok(())
    }

    fn remembered(&self) -> AppResult<Option<CloseAction>> {
        self.0
            .lock()
            .map(|preference| *preference)
            .map_err(|_| AppError::Message("close preference state is unavailable".to_string()))
    }
}

fn execute_close_action(app: &AppHandle, action: CloseAction) -> AppResult<()> {
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

#[tauri::command]
pub fn handle_close_action(
    app: AppHandle,
    preference: State<'_, ClosePreferenceState>,
    action: CloseAction,
) -> AppResult<()> {
    preference.remember(action)?;
    execute_close_action(&app, action)
}

pub fn handle_remembered_close_action(
    app: &AppHandle,
    preference: &ClosePreferenceState,
) -> AppResult<bool> {
    let Some(action) = preference.remembered()? else {
        return Ok(false);
    };
    execute_close_action(app, action)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{CloseAction, ClosePreferenceState};

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

    #[test]
    fn close_preference_is_empty_for_each_new_process_state() {
        let preference = ClosePreferenceState::default();
        assert_eq!(preference.remembered().unwrap(), None);
    }

    #[test]
    fn close_preference_remembers_the_confirmed_action() {
        let preference = ClosePreferenceState::default();
        preference.remember(CloseAction::Background).unwrap();
        assert_eq!(
            preference.remembered().unwrap(),
            Some(CloseAction::Background)
        );
    }
}

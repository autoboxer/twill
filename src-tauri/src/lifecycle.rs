use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use uuid::Uuid;

use crate::{data::LocalDataStore, library::CssSnippetLibrary};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum NativeAction {
    Close,
    Quit,
    Reload,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ActionRequest {
    id: String,
    action: NativeAction,
}

#[derive(Default)]
pub(crate) struct NativeLifecycle(Mutex<LifecycleState>);

#[derive(Default)]
struct LifecycleState {
    ready: bool,
    approved_exit: bool,
    pending: Option<ActionRequest>,
}

impl LifecycleState {
    fn begin(&mut self, action: NativeAction) -> Option<ActionRequest> {
        if self.pending.is_some() {
            return None;
        }

        let request = ActionRequest {
            id: Uuid::now_v7().to_string(),
            action,
        };

        self.pending = Some(request.clone());

        Some(request)
    }

    fn take(&mut self, id: &str) -> Result<ActionRequest, &'static str> {
        if self.pending.as_ref().is_none_or(|request| request.id != id) {
            return Err("This window action is no longer pending.");
        }

        Ok(self.pending.take().unwrap())
    }
}

#[tauri::command]
pub(crate) fn native_lifecycle_ready(state: State<'_, NativeLifecycle>) {
    state.0.lock().unwrap().ready = true;
}

#[tauri::command]
pub(crate) fn complete_native_action<R: Runtime>(
    app: AppHandle<R>,
    request_id: String,
    proceed: bool,
) -> Result<(), String> {
    let lifecycle = app.state::<NativeLifecycle>();
    let mut state = lifecycle.0.lock().unwrap();
    let request = state.take(&request_id)?;

    if !proceed {
        return Ok(());
    }

    state.approved_exit = request.action == NativeAction::Quit;
    state.ready = request.action != NativeAction::Reload;
    drop(state);

    if let Err(error) = perform_action(&app, request.action) {
        let mut state = lifecycle.0.lock().unwrap();

        state.approved_exit = false;
        state.ready = true;
        state.pending = Some(request);

        return Err(error.to_string());
    }

    Ok(())
}

pub(crate) fn request_action<R: Runtime>(app: &AppHandle<R>, action: NativeAction) {
    let lifecycle = app.state::<NativeLifecycle>();
    let mut state = lifecycle.0.lock().unwrap();
    let Some(request) = state.begin(action) else {
        return;
    };
    let ready = state.ready;

    drop(state);

    // The WebView cannot accept edits before its lifecycle listener is ready
    if !ready {
        if let Err(error) = complete_native_action(app.clone(), request.id, true) {
            eprintln!("Could not complete native window action: {error}");
        }

        return;
    }

    if let Err(error) = app.emit_to("main", "twill-native-action", &request) {
        let _ = lifecycle.0.lock().unwrap().take(&request.id);
        eprintln!("Could not request a safe window action: {error}");
    }
}

#[cfg(desktop)]
pub(crate) fn handle_window_event<R: Runtime>(
    window: &tauri::Window<R>,
    event: &tauri::WindowEvent,
) {
    if window.label() != "main" {
        return;
    }

    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        request_action(window.app_handle(), NativeAction::Close);
    }
}

pub(crate) fn handle_run_event<R: Runtime>(app: &AppHandle<R>, event: tauri::RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        let approved = app
            .state::<NativeLifecycle>()
            .0
            .lock()
            .unwrap()
            .approved_exit;

        if approved || app.get_webview_window("main").is_none() {
            return;
        }

        api.prevent_exit();
        request_action(app, NativeAction::Quit);
    }
}

fn perform_action<R: Runtime>(app: &AppHandle<R>, action: NativeAction) -> Result<(), String> {
    if action == NativeAction::Quit {
        app.exit(0);
        return Ok(());
    }

    let window = app
        .get_webview_window("main")
        .ok_or("The main window was not found.")?;

    match action {
        NativeAction::Close => window.destroy().map_err(|error| error.to_string()),
        NativeAction::Reload => {
            let local_data = app.state::<LocalDataStore>();

            CssSnippetLibrary::new(local_data.inner())
                .disable_all()
                .map_err(|error| error.to_string())?;

            window.reload().map_err(|error| error.to_string())
        }
        NativeAction::Quit => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::{LifecycleState, NativeAction};

    #[test]
    fn repeated_actions_share_one_pending_decision() {
        let mut state = LifecycleState::default();
        let request = state.begin(NativeAction::Close).unwrap();

        assert!(state.begin(NativeAction::Quit).is_none());
        assert!(state.begin(NativeAction::Reload).is_none());
        assert_eq!(state.take(&request.id).unwrap().action, NativeAction::Close);
        assert!(state.take(&request.id).is_err());
    }

    #[test]
    fn stale_completion_cannot_finish_a_later_request() {
        let mut state = LifecycleState::default();
        let first = state.begin(NativeAction::Quit).unwrap();

        state.take(&first.id).unwrap();

        let second = state.begin(NativeAction::Reload).unwrap();

        assert_ne!(first.id, second.id);
        assert!(state.take(&first.id).is_err());
        assert_eq!(state.take(&second.id).unwrap().action, NativeAction::Reload);
    }
}

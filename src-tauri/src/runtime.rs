use std::ffi::OsStr;

use serde::Serialize;
use tauri::State;

#[cfg(desktop)]
use tauri::{
    menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder},
    AppHandle, Manager, Runtime,
};

#[cfg(desktop)]
use crate::{data::LocalDataStore, library::CssSnippetLibrary};

const SAFE_MODE_ARGUMENT: &str = "--safe-mode";

#[cfg(desktop)]
const DISABLE_CSS_SNIPPETS_MENU_ID: &str = "disable-css-snippets-and-reload";

#[cfg(all(desktop, not(target_os = "macos")))]
const QUIT_MENU_ID: &str = "quit";

#[derive(Clone, Copy, Debug)]
pub(crate) struct RuntimeRecoveryState {
    safe_mode: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CssSnippetRuntimeState {
    safe_mode: bool,
}

impl RuntimeRecoveryState {
    pub(crate) fn from_args<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let safe_mode = args
            .into_iter()
            .any(|argument| argument.as_ref() == OsStr::new(SAFE_MODE_ARGUMENT));

        Self { safe_mode }
    }
}

#[tauri::command]
pub(crate) fn get_css_snippet_runtime_state(
    runtime: State<'_, RuntimeRecoveryState>,
) -> CssSnippetRuntimeState {
    CssSnippetRuntimeState {
        safe_mode: runtime.safe_mode,
    }
}

#[cfg(desktop)]
pub(crate) fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let recovery_item = MenuItemBuilder::with_id(
        DISABLE_CSS_SNIPPETS_MENU_ID,
        "Disable CSS Snippets and Reload",
    )
    .build(app)?;

    #[cfg(target_os = "macos")]
    let application_menu = SubmenuBuilder::new(app, "Twill")
        .about(None)
        .separator()
        .item(&recovery_item)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    #[cfg(not(target_os = "macos"))]
    let application_menu = {
        let quit_item = MenuItemBuilder::with_id(QUIT_MENU_ID, "&Quit")
            .accelerator("CmdOrCtrl+Q")
            .build(app)?;

        SubmenuBuilder::new(app, "&Twill")
            .item(&recovery_item)
            .separator()
            .item(&quit_item)
            .build()?
    };

    #[cfg(target_os = "macos")]
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    #[cfg(not(target_os = "macos"))]
    let edit_menu = SubmenuBuilder::new(app, "&Edit")
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    MenuBuilder::new(app)
        .items(&[&application_menu, &edit_menu])
        .build()
}

#[cfg(desktop)]
pub(crate) fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    #[cfg(not(target_os = "macos"))]
    if event.id().as_ref() == QUIT_MENU_ID {
        app.exit(0);
        return;
    }

    if event.id().as_ref() != DISABLE_CSS_SNIPPETS_MENU_ID {
        return;
    }

    let local_data = app.state::<LocalDataStore>();

    if let Err(error) = CssSnippetLibrary::new(local_data.inner()).disable_all() {
        eprintln!("Could not disable CSS snippets: {error}");
        return;
    }

    let Some(window) = app.get_webview_window("main") else {
        eprintln!("Could not reload Twill after disabling CSS snippets: window not found");
        return;
    };

    if let Err(error) = window.reload() {
        eprintln!("Could not reload Twill after disabling CSS snippets: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeRecoveryState;

    #[test]
    fn recognizes_the_exact_safe_mode_argument() {
        let state = RuntimeRecoveryState::from_args(["twill", "--safe-mode"]);

        assert!(state.safe_mode);
    }

    #[test]
    fn ignores_similar_arguments() {
        for argument in ["safe-mode", "--safe-mode=true", "--safe_mode"] {
            let state = RuntimeRecoveryState::from_args(["twill", argument]);

            assert!(!state.safe_mode, "accepted {argument}");
        }
    }
}

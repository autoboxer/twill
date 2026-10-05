use serde::Deserialize;
use tauri::AppHandle;
#[cfg(desktop)]
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ArchiveFileAction {
    Backup,
    Export,
    Restore,
}

#[tauri::command]
pub(crate) async fn choose_archive_file(
    app: AppHandle,
    action: ArchiveFileAction,
) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog().file();

        #[cfg(desktop)]
        let dialog = dialog.set_parent(
            &app.get_webview_window("main")
                .ok_or("The main window is unavailable")?,
        );

        let selection = match action {
            ArchiveFileAction::Backup => dialog
                .set_title("Create backup")
                .add_filter("Twill backup", &["twill"])
                .set_file_name("Twill backup.twill")
                .blocking_save_file(),
            ArchiveFileAction::Export => dialog
                .set_title("Export data")
                .add_filter("ZIP archive", &["zip"])
                .set_file_name("Twill export.zip")
                .blocking_save_file(),
            ArchiveFileAction::Restore => dialog
                .set_title("Restore backup")
                .add_filter("Twill backup", &["twill"])
                .blocking_pick_file(),
        };

        selection
            .map(|file| {
                file.into_path()
                    .ok()
                    .and_then(|path| path.into_os_string().into_string().ok())
                    .ok_or("Choose a file available on this device".to_owned())
            })
            .transpose()
    })
    .await
    .map_err(|_| "The file chooser could not be opened".to_owned())?
}

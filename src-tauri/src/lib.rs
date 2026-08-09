pub mod data;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let local_data = data::LocalDataStore::open(data_directory)?;

            app.manage(local_data);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Twill");
}

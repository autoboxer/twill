pub mod data;
mod library;
mod runtime;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().manage(
        runtime::RuntimeRecoveryState::from_args(std::env::args_os()),
    );

    #[cfg(desktop)]
    let builder = builder
        .menu(runtime::build_menu)
        .on_menu_event(runtime::handle_menu_event);

    builder
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let local_data = data::LocalDataStore::open(data_directory)?;

            app.manage(local_data);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            library::commands::get_library,
            library::commands::get_concept,
            library::commands::get_study_queue,
            library::commands::record_review,
            library::commands::reverse_review,
            library::commands::get_device_preferences,
            library::commands::set_grading_mode,
            library::commands::set_startup_destination,
            library::commands::set_appearance_preferences,
            library::commands::get_scheduling_settings,
            library::commands::update_scheduling_settings,
            library::commands::create_concept,
            library::commands::update_concept,
            library::commands::set_concept_archived,
            library::commands::delete_concept,
            library::commands::create_deck,
            library::commands::rename_deck,
            library::commands::delete_deck,
            library::commands::create_tag,
            library::commands::rename_tag,
            library::commands::delete_tag,
            library::commands::get_css_snippets,
            library::commands::create_css_snippet,
            library::commands::update_css_snippet,
            library::commands::set_css_snippet_enabled,
            library::commands::disable_all_css_snippets,
            library::commands::delete_css_snippet,
            library::commands::get_authoring_draft,
            library::commands::upsert_authoring_draft,
            library::commands::delete_authoring_draft,
            runtime::get_css_snippet_runtime_state,
            library::commands::get_templates,
            library::commands::get_template,
            library::commands::create_template,
            library::commands::update_template,
            library::commands::delete_template,
            library::commands::prepare_template_preview,
            library::commands::import_image,
            library::commands::read_media,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Twill");
}

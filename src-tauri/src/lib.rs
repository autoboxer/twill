pub mod backup;
pub mod data;
mod library;
mod lifecycle;
mod runtime;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(lifecycle::NativeLifecycle::default())
        .manage(runtime::RuntimeRecoveryState::from_args(std::env::args_os()));

    #[cfg(desktop)]
    let builder = builder
        .menu(runtime::build_menu)
        .on_menu_event(runtime::handle_menu_event)
        .on_window_event(lifecycle::handle_window_event);

    builder
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            app.manage(backup::recovery::StorageRecovery::new(data_directory));
            let handle = app.handle().clone();

            tauri::async_runtime::spawn_blocking(move || {
                let _ = handle
                    .state::<backup::recovery::StorageRecovery>()
                    .open(|store| {
                        handle.manage(store);
                    });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lifecycle::native_lifecycle_ready,
            lifecycle::complete_native_action,
            lifecycle::restart_application,
            backup::dialogs::choose_archive_file,
            backup::commands::create_backup,
            backup::commands::export_library,
            backup::commands::inspect_backup,
            backup::commands::prepare_restore,
            backup::commands::cancel_restore,
            backup::commands::get_storage_status,
            backup::commands::retry_storage,
            library::commands::get_library,
            library::commands::get_library_organizations,
            library::commands::get_concept,
            library::commands::get_practice_links,
            library::commands::get_linked_practice,
            library::commands::create_practice_link,
            library::commands::update_practice_link,
            library::commands::remove_practice_link,
            library::commands::get_study_queue,
            library::commands::record_pretest,
            library::commands::record_review,
            library::commands::reverse_review,
            library::commands::get_deferred_edits,
            library::commands::queue_deferred_edit,
            library::commands::update_deferred_edit_note,
            library::commands::remove_deferred_edit,
            library::commands::get_card_quality_queue,
            library::commands::create_card_quality_concern,
            library::commands::close_card_quality_concern,
            library::commands::dismiss_card_quality_signal,
            library::commands::get_device_preferences,
            library::commands::set_grading_mode,
            library::commands::set_mixed_practice_enabled,
            library::commands::set_pretesting_enabled,
            library::commands::set_startup_destination,
            library::commands::set_appearance_preferences,
            library::commands::set_library_view_preferences,
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
            library::commands::finalize_concept,
            library::commands::finalize_template,
            library::commands::begin_authoring_media_session,
            library::commands::end_authoring_media_session,
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
        .build(tauri::generate_context!())
        .expect("failed to build Twill")
        .run(lifecycle::handle_run_event);
}

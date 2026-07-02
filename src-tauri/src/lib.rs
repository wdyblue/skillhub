mod ai;
mod commands;
mod database;
mod hash;
mod marketplace;
mod scanner;
mod similarity;
mod skill_files;
mod skills_all_sync;
mod sync_tools;

use ai::{
    clear_translation_cache, get_account, get_ai_config, list_ai_models, login_account,
    logout_account, save_ai_config, test_ai_connection, translate_skill,
};
use commands::{
    add_scan_root, batch_update_skills, create_category, delete_category, get_skill, get_stats,
    increment_usage, list_categories, list_scan_roots, list_skills, list_tags, open_skill_file,
    open_skill_folder, remove_scan_root, scan_all, toggle_scan_root, update_category,
    update_skill_meta, update_skill_scope, update_skill_tags,
};
use database::{init_database, AppState};
use marketplace::{
    add_marketplace_source, delete_marketplace_source, export_sync_package, get_cloud_sync_config,
    import_sync_package, install_marketplace_item, list_marketplace_items, list_marketplace_sources,
    pull_sync_package_from_cloud, recheck_marketplace_installations, refresh_marketplace_source,
    save_cloud_sync_config, uninstall_marketplace_item, update_marketplace_item,
    push_sync_package_to_cloud,
};
use skill_files::{create_skill_in_repository, import_skill_to_repository, save_skill_content};
use skills_all_sync::{get_skills_all_sync_report, run_skills_all_sync};
use std::sync::Mutex;
use sync_tools::{
    apply_skills_all_profile, check_sync_status, create_custom_tool, delete_custom_tool,
    detect_tools, fix_sync_issues, get_skills_all_profile, list_repositories, list_tools,
    set_primary_repository, set_skill_tool_enabled, sync_all_enabled_tools, update_tool_config,
};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("skillhub.sqlite3");
            let conn = init_database(&db_path)
                .map_err(|err| Box::<dyn std::error::Error>::from(err.to_string()))?;
            app.manage(AppState {
                conn: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_stats,
            list_categories,
            create_category,
            update_category,
            delete_category,
            list_scan_roots,
            add_scan_root,
            remove_scan_root,
            toggle_scan_root,
            scan_all,
            list_skills,
            list_tags,
            get_skill,
            update_skill_meta,
            update_skill_scope,
            update_skill_tags,
            batch_update_skills,
            increment_usage,
            open_skill_folder,
            open_skill_file,
            detect_tools,
            list_tools,
            update_tool_config,
            create_custom_tool,
            delete_custom_tool,
            list_repositories,
            get_skills_all_profile,
            apply_skills_all_profile,
            set_primary_repository,
            set_skill_tool_enabled,
            check_sync_status,
            fix_sync_issues,
            sync_all_enabled_tools,
            create_skill_in_repository,
            import_skill_to_repository,
            save_skill_content,
            get_ai_config,
            save_ai_config,
            list_ai_models,
            test_ai_connection,
            translate_skill,
            get_account,
            login_account,
            logout_account,
            clear_translation_cache,
            list_marketplace_sources,
            add_marketplace_source,
            delete_marketplace_source,
            refresh_marketplace_source,
            list_marketplace_items,
            install_marketplace_item,
            update_marketplace_item,
            uninstall_marketplace_item,
            recheck_marketplace_installations,
            export_sync_package,
            import_sync_package,
            get_cloud_sync_config,
            save_cloud_sync_config,
            push_sync_package_to_cloud,
            pull_sync_package_from_cloud,
            get_skills_all_sync_report,
            run_skills_all_sync
        ])
        .run(tauri::generate_context!())
        .expect("error while running SkillHub");
}

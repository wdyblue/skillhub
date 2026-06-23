mod commands;
mod database;
mod hash;
mod scanner;
mod skill_files;
mod similarity;
mod sync_tools;

use commands::{
    add_scan_root, create_category, delete_category, get_skill, get_stats, increment_usage,
    list_categories, list_scan_roots, list_skills, open_skill_file, open_skill_folder,
    remove_scan_root, scan_all, toggle_scan_root, update_category, update_skill_meta,
};
use sync_tools::{
    check_sync_status, create_custom_tool, delete_custom_tool, detect_tools, fix_sync_issues,
    list_repositories, list_tools, set_primary_repository, set_skill_tool_enabled,
    update_tool_config,
};
use skill_files::{create_skill_in_repository, import_skill_to_repository, save_skill_content};
use database::{init_database, AppState};
use std::sync::Mutex;
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
            get_skill,
            update_skill_meta,
            increment_usage,
            open_skill_folder,
            open_skill_file,
            detect_tools,
            list_tools,
            update_tool_config,
            create_custom_tool,
            delete_custom_tool,
            list_repositories,
            set_primary_repository,
            set_skill_tool_enabled,
            check_sync_status,
            fix_sync_issues,
            create_skill_in_repository,
            import_skill_to_repository,
            save_skill_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running SkillHub");
}

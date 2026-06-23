use crate::database::AppState;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Serialize)]
pub struct ToolDto {
    pub id: i64,
    pub tool_name: String,
    pub display_name: String,
    pub skill_dir: String,
    pub detected: bool,
    pub enabled: bool,
    pub sync_enabled: bool,
    pub is_custom: bool,
    pub link_mode: String,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SkillToolLinkDto {
    pub id: i64,
    pub skill_id: i64,
    pub tool_name: String,
    pub enabled: bool,
    pub link_path: String,
    pub link_mode: String,
    pub link_status: String,
    pub last_synced_at: Option<String>,
    pub error_message: String,
}

#[derive(Debug, Serialize)]
pub struct RepositoryDto {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub repo_type: String,
    pub enabled: bool,
    pub is_primary: bool,
    pub last_scanned_at: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SyncIssueDto {
    pub id: i64,
    pub skill_id: Option<i64>,
    pub skill_name: Option<String>,
    pub tool_name: String,
    pub issue_type: String,
    pub current_path: String,
    pub expected_path: String,
    pub severity: String,
    pub fixable: bool,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Default)]
pub struct SyncReportDto {
    pub normal_count: i64,
    pub missing_count: i64,
    pub broken_count: i64,
    pub wrong_target_count: i64,
    pub duplicate_count: i64,
    pub missing_dir_count: i64,
    pub needs_fix_count: i64,
    pub issues: Vec<SyncIssueDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateToolConfigRequest {
    pub tool_name: String,
    pub skill_dir: String,
    pub enabled: bool,
    pub sync_enabled: bool,
    pub link_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomToolRequest {
    pub tool_name: String,
    pub display_name: String,
    pub skill_dir: String,
    pub link_mode: Option<String>,
}

pub fn attach_tool_links(conn: &Connection, skill: &mut crate::commands::SkillDto) -> CommandResult<()> {
    let mut stmt = conn
        .prepare(
            "SELECT id, skill_id, tool_name, enabled, link_path, link_mode, link_status, last_synced_at, error_message
             FROM skill_tool_links WHERE skill_id = ?1 ORDER BY tool_name",
        )
        .map_err(|err| err.to_string())?;
    skill.tool_links = stmt
        .query_map(params![skill.id], link_from_row)
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn detect_tools(state: State<AppState>) -> CommandResult<Vec<ToolDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    for (tool_name, display_name, candidates) in default_tool_candidates() {
        let configured: Option<String> = conn
            .query_row(
                "SELECT skill_dir FROM tools WHERE tool_name = ?1",
                params![tool_name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| err.to_string())?;
        let detected_path = candidates
            .iter()
            .map(|path| expand_home_string(path))
            .find(|path| Path::new(path).exists());
        let chosen = configured
            .filter(|path| !path.trim().is_empty())
            .or(detected_path)
            .unwrap_or_else(|| expand_home_string(candidates[0]));
        let detected = Path::new(&chosen).exists();
        conn.execute(
            "INSERT INTO tools
             (tool_name, display_name, skill_dir, detected, enabled, sync_enabled, link_mode, last_checked_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, 1, COALESCE((SELECT link_mode FROM tools WHERE tool_name = ?1), 'auto'), datetime('now'), datetime('now'), datetime('now'))
             ON CONFLICT(tool_name) DO UPDATE SET
               display_name = excluded.display_name,
               skill_dir = ?3,
               detected = ?4,
               last_checked_at = datetime('now'),
               updated_at = datetime('now')",
            params![tool_name, display_name, chosen, if detected { 1 } else { 0 }],
        )
        .map_err(|err| err.to_string())?;
    }
    list_tools_from_conn(&conn)
}

#[tauri::command]
pub fn list_tools(state: State<AppState>) -> CommandResult<Vec<ToolDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    list_tools_from_conn(&conn)
}

#[tauri::command]
pub fn update_tool_config(state: State<AppState>, request: UpdateToolConfigRequest) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let skill_dir = expand_home_string(request.skill_dir.trim());
    let link_mode = normalize_link_mode(request.link_mode.as_deref().unwrap_or("auto"));
    conn.execute(
        "UPDATE tools SET skill_dir = ?1, detected = ?2, enabled = ?3, sync_enabled = ?4,
         link_mode = ?5, last_checked_at = datetime('now'), updated_at = datetime('now')
         WHERE tool_name = ?6",
        params![
            skill_dir,
            if Path::new(&skill_dir).exists() { 1 } else { 0 },
            if request.enabled { 1 } else { 0 },
            if request.sync_enabled { 1 } else { 0 },
            link_mode,
            request.tool_name
        ],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_custom_tool(state: State<AppState>, request: CustomToolRequest) -> CommandResult<ToolDto> {
    let tool_name = normalize_tool_name(&request.tool_name);
    if tool_name.is_empty() {
        return Err("工具 ID 不能为空".to_string());
    }
    if request.display_name.trim().is_empty() {
        return Err("工具名称不能为空".to_string());
    }
    let skill_dir = expand_home_string(request.skill_dir.trim());
    let link_mode = normalize_link_mode(request.link_mode.as_deref().unwrap_or("auto"));
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO tools
         (tool_name, display_name, skill_dir, detected, enabled, sync_enabled, is_custom, link_mode, last_checked_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 1, 1, 1, ?5, datetime('now'), datetime('now'), datetime('now'))
         ON CONFLICT(tool_name) DO UPDATE SET
           display_name = excluded.display_name,
           skill_dir = excluded.skill_dir,
           detected = excluded.detected,
           is_custom = 1,
           link_mode = excluded.link_mode,
           updated_at = datetime('now')",
        params![
            tool_name,
            request.display_name.trim(),
            skill_dir,
            if Path::new(&skill_dir).exists() { 1 } else { 0 },
            link_mode,
        ],
    )
    .map_err(|err| err.to_string())?;
    get_tool_by_name(&conn, &tool_name)
}

#[tauri::command]
pub fn delete_custom_tool(state: State<AppState>, tool_name: String) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let is_custom: Option<i64> = conn
        .query_row(
            "SELECT is_custom FROM tools WHERE tool_name = ?1",
            params![tool_name],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| err.to_string())?;
    if is_custom != Some(1) {
        return Err("只允许删除自定义工具配置".to_string());
    }
    conn.execute("DELETE FROM skill_tool_links WHERE tool_name = ?1", params![tool_name])
        .map_err(|err| err.to_string())?;
    conn.execute("DELETE FROM tools WHERE tool_name = ?1", params![tool_name])
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_repositories(state: State<AppState>) -> CommandResult<Vec<RepositoryDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    list_repositories_from_conn(&conn)
}

#[tauri::command]
pub fn set_primary_repository(state: State<AppState>, path: String) -> CommandResult<RepositoryDto> {
    let normalized = expand_home_string(path.trim());
    if normalized.is_empty() {
        return Err("主仓库目录不能为空".to_string());
    }
    if !Path::new(&normalized).exists() {
        fs::create_dir_all(&normalized).map_err(|err| format!("创建主仓库目录失败：{}", err))?;
    }
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute("UPDATE repositories SET is_primary = 0", [])
        .map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO repositories (name, path, type, enabled, is_primary, created_at, updated_at)
         VALUES ('统一 Skill 主仓库', ?1, 'local', 1, 1, datetime('now'), datetime('now'))
         ON CONFLICT(path) DO UPDATE SET enabled = 1, is_primary = 1, updated_at = datetime('now')",
        params![normalized],
    )
    .map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO scan_roots (path, enabled, platform, created_at, updated_at)
         VALUES (?1, 1, '主仓库', datetime('now'), datetime('now'))
         ON CONFLICT(path) DO UPDATE SET enabled = 1, platform = '主仓库', updated_at = datetime('now')",
        params![normalized],
    )
    .map_err(|err| err.to_string())?;
    list_repositories_from_conn(&conn)?
        .into_iter()
        .find(|repo| repo.is_primary)
        .ok_or_else(|| "主仓库保存后读取失败".to_string())
}

#[tauri::command]
pub fn set_skill_tool_enabled(
    state: State<AppState>,
    skill_id: i64,
    tool_name: String,
    enabled: bool,
) -> CommandResult<SkillToolLinkDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    sync_skill_tool(&conn, skill_id, &tool_name, enabled)
}

#[tauri::command]
pub fn check_sync_status(state: State<AppState>) -> CommandResult<SyncReportDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    rebuild_sync_issues(&conn)?;
    build_sync_report(&conn)
}

#[tauri::command]
pub fn fix_sync_issues(state: State<AppState>) -> CommandResult<SyncReportDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    rebuild_sync_issues(&conn)?;
    let issues = current_sync_issues(&conn)?;
    for issue in issues.into_iter().filter(|issue| issue.fixable) {
        if let Some(skill_id) = issue.skill_id {
            let enabled = issue.issue_type != "extra_link";
            let _ = sync_skill_tool(&conn, skill_id, &issue.tool_name, enabled);
        }
    }
    rebuild_sync_issues(&conn)?;
    build_sync_report(&conn)
}

#[tauri::command]
pub fn sync_all_enabled_tools(state: State<AppState>) -> CommandResult<SyncReportDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let skill_ids = {
        let mut stmt = conn
            .prepare("SELECT id FROM skills WHERE status != '已归档' AND status != '路径丢失' ORDER BY id")
            .map_err(|err| err.to_string())?;
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0))
            .map_err(|err| err.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| err.to_string())?;
        rows
    };
    let tool_names = {
        let mut stmt = conn
            .prepare("SELECT tool_name, skill_dir FROM tools WHERE enabled = 1 AND sync_enabled = 1 ORDER BY id")
            .map_err(|err| err.to_string())?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|err| err.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| err.to_string())?;
        rows
    };
    for skill_id in skill_ids {
        for (tool_name, tool_dir) in &tool_names {
            if is_same_as_primary_repository(&conn, tool_dir)? {
                continue;
            }
            let _ = sync_skill_tool(&conn, skill_id, tool_name, true);
        }
    }
    rebuild_sync_issues(&conn)?;
    build_sync_report(&conn)
}

fn sync_skill_tool(
    conn: &Connection,
    skill_id: i64,
    tool_name: &str,
    enabled: bool,
) -> CommandResult<SkillToolLinkDto> {
    let (skill_path, fallback_name): (String, String) = conn
        .query_row(
            "SELECT path, name FROM skills WHERE id = ?1",
            params![skill_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|err| err.to_string())?;
    let (tool_dir, link_mode): (String, String) = conn
        .query_row(
            "SELECT skill_dir, link_mode FROM tools WHERE tool_name = ?1",
            params![tool_name],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| format!("未找到工具配置：{}", tool_name))?;

    let tool_dir = expand_home_string(&tool_dir);
    let link_mode = normalize_link_mode(&link_mode);
    if is_same_as_primary_repository(conn, &tool_dir)? {
        let status = "直连主仓库".to_string();
        conn.execute(
            "INSERT INTO skill_tool_links
             (skill_id, tool_name, enabled, link_path, link_mode, link_status, last_synced_at, error_message, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'direct', ?5, datetime('now'), '', datetime('now'), datetime('now'))
             ON CONFLICT(skill_id, tool_name) DO UPDATE SET
               enabled = ?3, link_path = ?4, link_mode = 'direct', link_status = ?5,
               last_synced_at = datetime('now'), error_message = '', updated_at = datetime('now')",
            params![skill_id, tool_name, if enabled { 1 } else { 0 }, tool_dir, status],
        )
        .map_err(|err| err.to_string())?;
        return conn
            .query_row(
                "SELECT id, skill_id, tool_name, enabled, link_path, link_mode, link_status, last_synced_at, error_message
                 FROM skill_tool_links WHERE skill_id = ?1 AND tool_name = ?2",
                params![skill_id, tool_name],
                link_from_row,
            )
            .map_err(|err| err.to_string());
    }
    let link_name = managed_link_name(conn, Path::new(&skill_path), &fallback_name)?;
    let link_path = Path::new(&tool_dir).join(link_name);
    let (status, error_message) = if enabled {
        match create_link_safely(Path::new(&skill_path), &link_path, &link_mode) {
            Ok(_) => ("已同步".to_string(), String::new()),
            Err(err) => ("同步失败".to_string(), err),
        }
    } else {
        match remove_link_safely(&link_path, &link_mode) {
            Ok(_) => ("未启用".to_string(), String::new()),
            Err(err) => ("移除失败".to_string(), err),
        }
    };

    conn.execute(
        "INSERT INTO skill_tool_links
         (skill_id, tool_name, enabled, link_path, link_mode, link_status, last_synced_at, error_message, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), ?7, datetime('now'), datetime('now'))
         ON CONFLICT(skill_id, tool_name) DO UPDATE SET
           enabled = ?3, link_path = ?4, link_mode = ?5, link_status = ?6, last_synced_at = datetime('now'),
           error_message = ?7, updated_at = datetime('now')",
        params![
            skill_id,
            tool_name,
            if enabled { 1 } else { 0 },
            link_path.to_string_lossy().to_string(),
            link_mode,
            status,
            error_message
        ],
    )
    .map_err(|err| err.to_string())?;

    conn.query_row(
        "SELECT id, skill_id, tool_name, enabled, link_path, link_mode, link_status, last_synced_at, error_message
         FROM skill_tool_links WHERE skill_id = ?1 AND tool_name = ?2",
        params![skill_id, tool_name],
        link_from_row,
    )
    .map_err(|err| err.to_string())
}

fn list_tools_from_conn(conn: &Connection) -> CommandResult<Vec<ToolDto>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, tool_name, display_name, skill_dir, detected, enabled, sync_enabled, is_custom, link_mode, last_checked_at
             FROM tools ORDER BY id",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(ToolDto {
            id: row.get(0)?,
            tool_name: row.get(1)?,
            display_name: row.get(2)?,
            skill_dir: row.get(3)?,
            detected: row.get::<_, i64>(4)? == 1,
            enabled: row.get::<_, i64>(5)? == 1,
            sync_enabled: row.get::<_, i64>(6)? == 1,
            is_custom: row.get::<_, i64>(7)? == 1,
            link_mode: row.get(8)?,
            last_checked_at: row.get(9)?,
        })
    })
    .map_err(|err| err.to_string())?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|err| err.to_string())?;
    Ok(rows)
}

fn get_tool_by_name(conn: &Connection, tool_name: &str) -> CommandResult<ToolDto> {
    conn.query_row(
        "SELECT id, tool_name, display_name, skill_dir, detected, enabled, sync_enabled, is_custom, link_mode, last_checked_at
         FROM tools WHERE tool_name = ?1",
        params![tool_name],
        |row| {
            Ok(ToolDto {
                id: row.get(0)?,
                tool_name: row.get(1)?,
                display_name: row.get(2)?,
                skill_dir: row.get(3)?,
                detected: row.get::<_, i64>(4)? == 1,
                enabled: row.get::<_, i64>(5)? == 1,
                sync_enabled: row.get::<_, i64>(6)? == 1,
                is_custom: row.get::<_, i64>(7)? == 1,
                link_mode: row.get(8)?,
                last_checked_at: row.get(9)?,
            })
        },
    )
    .map_err(|err| err.to_string())
}

fn list_repositories_from_conn(conn: &Connection) -> CommandResult<Vec<RepositoryDto>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, path, type, enabled, is_primary, last_scanned_at
             FROM repositories ORDER BY is_primary DESC, id DESC",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(RepositoryDto {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            repo_type: row.get(3)?,
            enabled: row.get::<_, i64>(4)? == 1,
            is_primary: row.get::<_, i64>(5)? == 1,
            last_scanned_at: row.get(6)?,
        })
    })
    .map_err(|err| err.to_string())?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|err| err.to_string())?;
    Ok(rows)
}

fn link_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SkillToolLinkDto> {
    Ok(SkillToolLinkDto {
        id: row.get(0)?,
        skill_id: row.get(1)?,
        tool_name: row.get(2)?,
        enabled: row.get::<_, i64>(3)? == 1,
        link_path: row.get(4)?,
        link_mode: row.get(5)?,
        link_status: row.get(6)?,
        last_synced_at: row.get(7)?,
        error_message: row.get(8)?,
    })
}

fn rebuild_sync_issues(conn: &Connection) -> CommandResult<()> {
    conn.execute("DELETE FROM sync_issues", [])
        .map_err(|err| err.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT l.skill_id, s.name, s.path, l.tool_name, l.enabled, l.link_path, l.link_mode, t.skill_dir
             FROM skill_tool_links l
             INNER JOIN skills s ON s.id = l.skill_id
             LEFT JOIN tools t ON t.tool_name = l.tool_name",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)? == 1,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?.unwrap_or_default(),
            ))
        })
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;

    for (skill_id, skill_name, skill_path, tool_name, enabled, saved_link_path, link_mode, tool_dir) in rows {
        if is_same_as_primary_repository(conn, &tool_dir)? || link_mode == "direct" {
            continue;
        }
        let expected = if saved_link_path.is_empty() {
            let slug = managed_link_name(conn, Path::new(&skill_path), &skill_name)?;
            Path::new(&expand_home_string(&tool_dir))
                .join(slug)
                .to_string_lossy()
                .to_string()
        } else {
            saved_link_path
        };
        let expected_path = PathBuf::from(&expected);
        if enabled && !Path::new(&expand_home_string(&tool_dir)).exists() {
            insert_issue(conn, Some(skill_id), &tool_name, "missing_dir", "", &expected, true, "工具目录不存在")?;
            continue;
        }
        if enabled {
            if is_copy_mode(&link_mode) {
                if !expected_path.exists() {
                    insert_issue(conn, Some(skill_id), &tool_name, "missing_link", "", &expected, true, "应该同步但复制目录不存在")?;
                    continue;
                }
                if !expected_path.join(".skillhub-managed.json").exists() {
                    insert_issue(conn, Some(skill_id), &tool_name, "wrong_target", &expected, &skill_path, false, "复制目录缺少管理标记")?;
                    continue;
                }
                if !expected_path.join("SKILL.md").exists() {
                    insert_issue(conn, Some(skill_id), &tool_name, "broken_link", &expected, &skill_path, true, "复制目录缺少 SKILL.md")?;
                }
                continue;
            }
            if expected_path.symlink_metadata().is_err() {
                insert_issue(conn, Some(skill_id), &tool_name, "missing_link", "", &expected, true, "应该同步但软链接不存在")?;
                continue;
            }
            let meta = expected_path.symlink_metadata();
            if let Ok(meta) = meta {
                if !meta.file_type().is_symlink() {
                    insert_issue(conn, Some(skill_id), &tool_name, "wrong_target", &expected, &skill_path, false, "目标存在但不是软链接")?;
                    continue;
                }
                let target = fs::read_link(&expected_path).unwrap_or_default();
                if !target.exists() {
                    insert_issue(conn, Some(skill_id), &tool_name, "broken_link", &target.to_string_lossy(), &skill_path, true, "软链接已断链")?;
                } else if normalize_path(&target) != normalize_path(Path::new(&skill_path)) {
                    insert_issue(conn, Some(skill_id), &tool_name, "wrong_target", &target.to_string_lossy(), &skill_path, true, "软链接指向错误")?;
                }
            }
        } else if expected_path.symlink_metadata().is_ok() {
            let is_link_like = expected_path
                .symlink_metadata()
                .map(|meta| meta.file_type().is_symlink())
                .unwrap_or(false);
            let is_copy_like = expected_path.join(".skillhub-managed.json").exists();
            if is_link_like || is_copy_like {
                insert_issue(conn, Some(skill_id), &tool_name, "extra_link", &expected, "", true, "已关闭启用但同步产物仍存在")?;
            }
        }
    }
    Ok(())
}

fn insert_issue(
    conn: &Connection,
    skill_id: Option<i64>,
    tool_name: &str,
    issue_type: &str,
    current_path: &str,
    expected_path: &str,
    fixable: bool,
    message: &str,
) -> CommandResult<()> {
    conn.execute(
        "INSERT INTO sync_issues
         (skill_id, tool_name, issue_type, current_path, expected_path, severity, fixable, status, message, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'warning', ?6, 'open', ?7, datetime('now'), datetime('now'))",
        params![skill_id, tool_name, issue_type, current_path, expected_path, if fixable { 1 } else { 0 }, message],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn current_sync_issues(conn: &Connection) -> CommandResult<Vec<SyncIssueDto>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.skill_id, s.name, i.tool_name, i.issue_type, i.current_path,
                    i.expected_path, i.severity, i.fixable, i.status, i.message
             FROM sync_issues i LEFT JOIN skills s ON s.id = i.skill_id
             WHERE i.status = 'open' ORDER BY i.id DESC",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(SyncIssueDto {
            id: row.get(0)?,
            skill_id: row.get(1)?,
            skill_name: row.get(2)?,
            tool_name: row.get(3)?,
            issue_type: row.get(4)?,
            current_path: row.get(5)?,
            expected_path: row.get(6)?,
            severity: row.get(7)?,
            fixable: row.get::<_, i64>(8)? == 1,
            status: row.get(9)?,
            message: row.get(10)?,
        })
    })
    .map_err(|err| err.to_string())?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|err| err.to_string())?;
    Ok(rows)
}

fn build_sync_report(conn: &Connection) -> CommandResult<SyncReportDto> {
    let issues = current_sync_issues(conn)?;
    let mut report = SyncReportDto {
        normal_count: conn
            .query_row(
                "SELECT COUNT(*) FROM skill_tool_links WHERE enabled = 1 AND link_status = '已同步'",
                [],
                |row| row.get(0),
            )
            .map_err(|err| err.to_string())?,
        needs_fix_count: issues.iter().filter(|issue| issue.fixable).count() as i64,
        issues,
        ..SyncReportDto::default()
    };
    for issue in &report.issues {
        match issue.issue_type.as_str() {
            "missing_link" => report.missing_count += 1,
            "broken_link" => report.broken_count += 1,
            "wrong_target" | "extra_link" => report.wrong_target_count += 1,
            "duplicate" => report.duplicate_count += 1,
            "missing_dir" => report.missing_dir_count += 1,
            _ => {}
        }
    }
    Ok(report)
}

fn create_link_safely(source: &Path, link_path: &Path, link_mode: &str) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("源 skill 不存在：{}", source.display()));
    }
    let Some(parent) = link_path.parent() else {
        return Err(format!("无法解析目标目录：{}", link_path.display()));
    };
    fs::create_dir_all(parent).map_err(|err| format!("创建工具目录失败：{}", err))?;
    if link_path.exists() {
        if is_copy_marker(link_path) {
            fs::remove_dir_all(link_path).map_err(|err| format!("清理旧复制目录失败：{}", err))?;
        } else {
            let meta = link_path
                .symlink_metadata()
                .map_err(|err| format!("读取已有目标失败：{}", err))?;
            if !meta.file_type().is_symlink() {
                return Err(format!("目标已存在且不是可管理链接：{}", link_path.display()));
            }
            remove_link_target(link_path, link_mode)?;
        }
    }

    match normalized_link_mode(link_mode).as_str() {
        "copy" => copy_skill_dir(source, link_path),
        "junction" => create_directory_junction(source, link_path),
        _ => create_platform_symlink(source, link_path),
    }
}

fn remove_link_safely(link_path: &Path, link_mode: &str) -> Result<(), String> {
    if !link_path.exists() {
        return Ok(());
    }
    if is_copy_marker(link_path) {
        return fs::remove_dir_all(link_path).map_err(|err| format!("删除复制目录失败：{}", err));
    }
    remove_link_target(link_path, link_mode)
}

fn remove_link_target(link_path: &Path, link_mode: &str) -> Result<(), String> {
    let meta = link_path
        .symlink_metadata()
        .map_err(|err| format!("读取目标失败：{}", err))?;
    if meta.file_type().is_symlink() {
        return fs::remove_file(link_path).map_err(|err| format!("删除软链接失败：{}", err));
    }
    #[cfg(windows)]
    {
        if normalized_link_mode(link_mode) == "junction" {
            return fs::remove_dir(link_path).map_err(|err| format!("删除 junction 失败：{}", err));
        }
    }
    if normalized_link_mode(link_mode) == "copy" {
        return fs::remove_dir_all(link_path).map_err(|err| format!("删除复制目录失败：{}", err));
    }
    Err(format!("目标不是可自动删除的链接：{}", link_path.display()))
}

fn create_platform_symlink(source: &Path, link_path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, link_path).map_err(|err| format!("创建软链接失败：{}", err))
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, link_path).map_err(|err| format!("创建软链接失败：{}", err))
    }
}

fn create_directory_junction(source: &Path, link_path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J", &link_path.to_string_lossy(), &source.to_string_lossy()])
            .status()
            .map_err(|err| format!("创建 junction 失败：{}", err))?;
        if status.success() {
            Ok(())
        } else {
            Err("创建 junction 失败：命令返回非零状态".to_string())
        }
    }
    #[cfg(not(windows))]
    {
        create_platform_symlink(source, link_path)
    }
}

fn copy_skill_dir(source: &Path, link_path: &Path) -> Result<(), String> {
    if link_path.exists() {
        if is_copy_marker(link_path) {
            fs::remove_dir_all(link_path).map_err(|err| format!("清理旧复制目录失败：{}", err))?;
        } else {
            return Err(format!("复制目标已存在且未标记为受管目录：{}", link_path.display()));
        }
    }
    fs::create_dir_all(link_path).map_err(|err| format!("创建复制目录失败：{}", err))?;
    copy_dir_all(source, link_path)?;
    fs::write(link_path.join(".skillhub-managed.json"), format!("{{\"source\":\"{}\",\"mode\":\"copy\"}}", source.display()))
        .map_err(|err| format!("写入管理标记失败：{}", err))?;
    Ok(())
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source).map_err(|err| format!("读取源目录失败：{}", err))? {
        let entry = entry.map_err(|err| err.to_string())?;
        let file_type = entry.file_type().map_err(|err| err.to_string())?;
        let dest = target.join(entry.file_name());
        if file_type.is_dir() {
            fs::create_dir_all(&dest).map_err(|err| err.to_string())?;
            copy_dir_all(&entry.path(), &dest)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &dest).map_err(|err| err.to_string())?;
        } else if file_type.is_symlink() {
            let link_target = fs::read_link(entry.path()).map_err(|err| err.to_string())?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(link_target, &dest).map_err(|err| err.to_string())?;
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(link_target, &dest).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

fn is_copy_marker(path: &Path) -> bool {
    path.join(".skillhub-managed.json").exists()
}

fn normalized_link_mode(link_mode: &str) -> String {
    normalize_link_mode(link_mode)
}

fn normalize_link_mode(input: &str) -> String {
    let mode = input.trim().to_lowercase();
    match mode.as_str() {
        "symlink" | "junction" | "copy" | "direct" => mode,
        _ => "auto".to_string(),
    }
}

fn is_copy_mode(link_mode: &str) -> bool {
    normalized_link_mode(link_mode) == "copy"
}

fn managed_link_name(conn: &Connection, skill_path: &Path, fallback_name: &str) -> CommandResult<String> {
    let primary = primary_repository_path_optional(conn)?;
    if let Some(primary) = primary {
        if let Ok(relative) = skill_path.strip_prefix(&primary) {
            let parts = relative
                .components()
                .filter_map(|component| component.as_os_str().to_str())
                .map(safe_slug)
                .collect::<Vec<_>>();
            if !parts.is_empty() {
                return Ok(parts.join("__"));
            }
        }
    }
    let base = skill_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(fallback_name);
    Ok(safe_slug(base))
}

fn is_same_as_primary_repository(conn: &Connection, tool_dir: &str) -> CommandResult<bool> {
    let primary = primary_repository_path_optional(conn)?;
    let Some(primary) = primary else {
        return Ok(false);
    };
    Ok(normalize_path(Path::new(tool_dir)) == normalize_path(&primary))
}

fn primary_repository_path_optional(conn: &Connection) -> CommandResult<Option<PathBuf>> {
    conn.query_row(
        "SELECT path FROM repositories WHERE is_primary = 1 AND enabled = 1 ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|path| path.map(PathBuf::from))
    .map_err(|err| err.to_string())
}

fn default_tool_candidates() -> Vec<(&'static str, &'static str, Vec<&'static str>)> {
    vec![
        ("codex", "Codex", vec!["~/.codex/skills"]),
        ("claude_code", "Claude Code", vec!["~/.claude/skills", "~/.claude/commands"]),
        ("codebuddy", "CodeBuddy", vec!["~/codebuddy-skill", "~/.codebuddy/skills"]),
        ("hermes", "Hermes", vec!["~/.hermes/skills", "~/Library/Application Support/Hermes/skills"]),
        ("cursor", "Cursor", vec!["~/.cursor/skills"]),
        ("opencode", "Opencode", vec!["~/.opencode/skills"]),
        ("gemini", "Gemini CLI", vec!["~/.gemini/skills", "~/.gemini/extensions"]),
        ("qwen_code", "Qwen Code", vec!["~/.qwen/skills", "~/.qwen-code/skills"]),
        ("cline", "Cline", vec!["~/.cline/skills"]),
        ("roo_code", "Roo Code", vec!["~/.roo/skills", "~/.roo-code/skills"]),
        ("continue", "Continue", vec!["~/.continue/skills"]),
        ("windsurf", "Windsurf", vec!["~/.windsurf/skills"]),
        ("trae", "Trae", vec!["~/.trae/skills"]),
        ("trae_cn", "Trae CN", vec!["~/.trae-cn/skills"]),
        ("augment", "Augment", vec!["~/.augment/skills"]),
        ("antigravity", "Antigravity", vec!["~/.antigravity/skills"]),
        ("goose", "Goose", vec!["~/.goose/skills"]),
        ("iflow", "iFlow", vec!["~/.iflow/skills"]),
        ("kiro", "Kiro", vec!["~/.kiro/skills"]),
        ("junie", "Junie", vec!["~/.junie/skills"]),
        ("kilo_code", "Kilo Code", vec!["~/.kilo-code/skills"]),
        ("qoder", "Qoder", vec!["~/.qoder/skills"]),
        ("zencoder", "Zencoder", vec!["~/.zencoder/skills"]),
        ("vercel_skills", "Vercel Skills", vec!["~/.vercel/skills"]),
        ("commandcode", "CommandCode", vec!["~/.commandcode/skills"]),
        ("crush", "Crush", vec!["~/.crush/skills"]),
        ("droid", "Droid", vec!["~/.droid/skills"]),
        ("pi", "Pi", vec!["~/.pi/skills"]),
    ]
}

fn expand_home_string(path: &str) -> String {
    if path == "~" || path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return Path::new(&home)
                .join(path.trim_start_matches("~/"))
                .to_string_lossy()
                .to_string();
        }
    }
    path.to_string()
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn safe_slug(input: &str) -> String {
    input
        .chars()
        .map(|ch| if ch == '/' || ch == ':' || ch == '\0' { '-' } else { ch })
        .collect()
}

fn normalize_tool_name(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else if ch.is_whitespace() {
                '_'
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .trim_matches('_')
        .to_string()
}

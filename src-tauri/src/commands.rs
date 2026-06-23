use crate::database::AppState;
use crate::scanner::{scan_enabled_roots, ScanSummary};
use rusqlite::{params, Connection, OptionalExtension, ToSql};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Serialize)]
pub struct AppStats {
    pub total_skills: i64,
    pub active_skills: i64,
    pub archived_skills: i64,
    pub missing_skills: i64,
    pub uncategorized_skills: i64,
    pub duplicate_risk_skills: i64,
    pub scan_roots: i64,
}

#[derive(Debug, Serialize)]
pub struct CategoryDto {
    pub id: i64,
    pub name: String,
    pub name_en: String,
    pub color: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInput {
    pub name: String,
    pub name_en: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScanRootDto {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub platform: String,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SkillDto {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub description: String,
    pub content: String,
    pub name_zh: String,
    pub name_en: String,
    pub description_zh: String,
    pub description_en: String,
    pub summary_zh: String,
    pub summary_en: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub source: String,
    pub platform: String,
    pub is_custom: bool,
    pub status: String,
    pub quality_score: i64,
    pub quality_reason: String,
    pub usage_count: i64,
    pub duplicate_score: f64,
    pub archive_recommendation: String,
    pub classification_confidence: f64,
    pub hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_scanned_at: Option<String>,
    pub last_used_at: Option<String>,
    pub tags: Vec<String>,
    pub tool_links: Vec<crate::sync_tools::SkillToolLinkDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillListFilters {
    pub query: Option<String>,
    pub category_id: Option<i64>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub only_archived: Option<bool>,
    pub only_duplicate: Option<bool>,
    pub only_uncategorized: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[tauri::command]
pub fn get_stats(state: State<AppState>) -> CommandResult<AppStats> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    Ok(AppStats {
        total_skills: count(&conn, "SELECT COUNT(*) FROM skills")?,
        active_skills: count(&conn, "SELECT COUNT(*) FROM skills WHERE status != '已归档'")?,
        archived_skills: count(&conn, "SELECT COUNT(*) FROM skills WHERE status = '已归档'")?,
        missing_skills: count(&conn, "SELECT COUNT(*) FROM skills WHERE status = '路径丢失'")?,
        uncategorized_skills: count(&conn, "SELECT COUNT(*) FROM skills WHERE category_id IS NULL")?,
        duplicate_risk_skills: count(&conn, "SELECT COUNT(*) FROM skills WHERE duplicate_score > 0")?,
        scan_roots: count(&conn, "SELECT COUNT(*) FROM scan_roots")?,
    })
}

#[tauri::command]
pub fn list_categories(state: State<AppState>) -> CommandResult<Vec<CategoryDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let rows = {
        let mut stmt = conn
            .prepare("SELECT id, name, name_en, color, parent_id FROM categories ORDER BY id")
            .map_err(|err| err.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(CategoryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                name_en: row.get(2)?,
                color: row.get(3)?,
                parent_id: row.get(4)?,
            })
        });
        let collected = rows
            .map_err(|err| err.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| err.to_string())?;
        collected
    };
    Ok(rows)
}

#[tauri::command]
pub fn create_category(state: State<AppState>, input: CategoryInput) -> CommandResult<CategoryDto> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("分类名称不能为空".to_string());
    }
    let color = input.color.unwrap_or_else(|| "#94a3b8".to_string());
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO categories (name, name_en, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
        params![name, input.name_en.unwrap_or_default(), color],
    )
    .map_err(|err| err.to_string())?;
    let id = conn.last_insert_rowid();
    get_category_by_id(&conn, id)
}

#[tauri::command]
pub fn update_category(state: State<AppState>, id: i64, input: CategoryInput) -> CommandResult<CategoryDto> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("分类名称不能为空".to_string());
    }
    let color = input.color.unwrap_or_else(|| "#94a3b8".to_string());
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE categories SET name = ?1, name_en = ?2, color = ?3, updated_at = datetime('now')
         WHERE id = ?4",
        params![name, input.name_en.unwrap_or_default(), color, id],
    )
    .map_err(|err| err.to_string())?;
    get_category_by_id(&conn, id)
}

#[tauri::command]
pub fn delete_category(state: State<AppState>, id: i64) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE skills SET category_id = NULL, updated_at = datetime('now') WHERE category_id = ?1",
        params![id],
    )
    .map_err(|err| err.to_string())?;
    conn.execute("DELETE FROM categories WHERE id = ?1", params![id])
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_scan_roots(state: State<AppState>) -> CommandResult<Vec<ScanRootDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let rows = {
        let mut stmt = conn
            .prepare(
                "SELECT id, path, enabled, platform, last_scanned_at, created_at
                 FROM scan_roots ORDER BY id DESC",
            )
            .map_err(|err| err.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(ScanRootDto {
                id: row.get(0)?,
                path: row.get(1)?,
                enabled: row.get::<_, i64>(2)? == 1,
                platform: row.get(3)?,
                last_scanned_at: row.get(4)?,
                created_at: row.get(5)?,
            })
        });
        let collected = rows
            .map_err(|err| err.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| err.to_string())?;
        collected
    };
    Ok(rows)
}

#[tauri::command]
pub fn add_scan_root(
    state: State<AppState>,
    path: String,
    platform: String,
) -> CommandResult<ScanRootDto> {
    let normalized = expand_home_string(path.trim());
    if normalized.is_empty() {
        return Err("目录不能为空".to_string());
    }
    if !Path::new(&normalized).exists() {
        return Err(format!("目录不存在：{}", normalized));
    }

    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO scan_roots (path, enabled, platform, created_at)
         VALUES (?1, 1, ?2, datetime('now'))
         ON CONFLICT(path) DO UPDATE SET enabled = 1, platform = excluded.platform",
        params![normalized, platform],
    )
    .map_err(|err| err.to_string())?;

    let target_id = conn
        .query_row(
            "SELECT id FROM scan_roots WHERE path = ?1",
            params![normalized],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    get_scan_root_by_id(&conn, target_id)
}

#[tauri::command]
pub fn remove_scan_root(state: State<AppState>, id: i64) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute("DELETE FROM scan_roots WHERE id = ?1", params![id])
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_scan_root(state: State<AppState>, id: i64, enabled: bool) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE scan_roots SET enabled = ?1 WHERE id = ?2",
        params![if enabled { 1 } else { 0 }, id],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn scan_all(state: State<AppState>) -> CommandResult<ScanSummary> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    scan_enabled_roots(&conn).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn list_skills(
    state: State<AppState>,
    filters: Option<SkillListFilters>,
) -> CommandResult<Vec<SkillDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let filters = filters.unwrap_or(SkillListFilters {
        query: None,
        category_id: None,
        status: None,
        source: None,
        only_archived: None,
        only_duplicate: None,
        only_uncategorized: None,
        sort_by: Some("updated_at".to_string()),
        sort_order: Some("desc".to_string()),
    });

    let mut sql = String::from(
        "SELECT s.id, s.name, s.path, s.description, s.content,
                s.name_zh, s.name_en, s.description_zh, s.description_en, s.summary_zh, s.summary_en,
                s.category_id, c.name, c.color, s.source, s.platform, s.is_custom, s.status,
                s.quality_score, s.quality_reason, s.usage_count, s.duplicate_score,
                s.archive_recommendation, s.classification_confidence,
                s.hash, s.created_at, s.updated_at, s.last_scanned_at, s.last_used_at
         FROM skills s
         LEFT JOIN categories c ON c.id = s.category_id
         WHERE 1 = 1",
    );
    let mut owned_params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(query) = filters.query.filter(|value| !value.trim().is_empty()) {
        sql.push_str(" AND (s.name LIKE ? OR s.description LIKE ? OR s.path LIKE ?)");
        let pattern = format!("%{}%", query.trim());
        owned_params.push(Box::new(pattern.clone()));
        owned_params.push(Box::new(pattern.clone()));
        owned_params.push(Box::new(pattern));
    }

    if let Some(category_id) = filters.category_id {
        sql.push_str(" AND s.category_id = ?");
        owned_params.push(Box::new(category_id));
    }

    if let Some(status) = filters.status.filter(|value| !value.is_empty()) {
        sql.push_str(" AND s.status = ?");
        owned_params.push(Box::new(status));
    } else if filters.only_archived != Some(true) {
        sql.push_str(" AND s.status != '已归档'");
    }

    if let Some(source) = filters.source.filter(|value| !value.is_empty()) {
        sql.push_str(" AND s.source = ?");
        owned_params.push(Box::new(source));
    }

    if filters.only_duplicate == Some(true) {
        sql.push_str(" AND s.duplicate_score > 0");
    }

    if filters.only_uncategorized == Some(true) {
        sql.push_str(" AND s.category_id IS NULL");
    }

    let sort_by = match filters.sort_by.as_deref() {
        Some("quality_score") => "s.quality_score",
        Some("usage_count") => "s.usage_count",
        Some("name") => "s.name",
        _ => "s.updated_at",
    };
    let sort_order = if filters.sort_order.as_deref() == Some("asc") {
        "ASC"
    } else {
        "DESC"
    };
    sql.push_str(&format!(" ORDER BY {} {}, s.id DESC LIMIT 500", sort_by, sort_order));

    let param_refs = owned_params
        .iter()
        .map(|param| param.as_ref() as &dyn ToSql)
        .collect::<Vec<_>>();
    let mut stmt = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(param_refs.as_slice(), skill_from_row)
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;

    attach_tags(&conn, rows)
}

#[tauri::command]
pub fn get_skill(state: State<AppState>, id: i64) -> CommandResult<SkillDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.name, s.path, s.description, s.content,
                    s.name_zh, s.name_en, s.description_zh, s.description_en, s.summary_zh, s.summary_en,
                    s.category_id, c.name, c.color, s.source, s.platform, s.is_custom, s.status,
                    s.quality_score, s.quality_reason, s.usage_count, s.duplicate_score,
                    s.archive_recommendation, s.classification_confidence,
                    s.hash, s.created_at, s.updated_at, s.last_scanned_at, s.last_used_at
             FROM skills s
             LEFT JOIN categories c ON c.id = s.category_id
             WHERE s.id = ?1",
        )
        .map_err(|err| err.to_string())?;
    let skill = stmt
        .query_row(params![id], skill_from_row)
        .optional()
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("未找到 skill：{}", id))?;
    let mut skills = attach_tags(&conn, vec![skill])?;
    Ok(skills.remove(0))
}

#[tauri::command]
pub fn update_skill_meta(
    state: State<AppState>,
    id: i64,
    category_id: Option<i64>,
    status: String,
    is_custom: bool,
) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE skills SET category_id = ?1, status = ?2, is_custom = ?3,
         updated_at = datetime('now') WHERE id = ?4",
        params![category_id, status, if is_custom { 1 } else { 0 }, id],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn increment_usage(state: State<AppState>, id: i64) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE skills SET usage_count = usage_count + 1, last_used_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_skill_folder(state: State<AppState>, id: i64) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let path: String = conn
        .query_row("SELECT path FROM skills WHERE id = ?1", params![id], |row| row.get(0))
        .map_err(|err| err.to_string())?;
    opener::open(path).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn open_skill_file(state: State<AppState>, id: i64) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let path: String = conn
        .query_row("SELECT path FROM skills WHERE id = ?1", params![id], |row| row.get(0))
        .map_err(|err| err.to_string())?;
    opener::open(Path::new(&path).join("SKILL.md")).map_err(|err| err.to_string())
}

fn count(conn: &Connection, sql: &str) -> CommandResult<i64> {
    conn.query_row(sql, [], |row| row.get(0))
        .map_err(|err| err.to_string())
}

fn get_scan_root_by_id(conn: &Connection, id: i64) -> CommandResult<ScanRootDto> {
    conn.query_row(
        "SELECT id, path, enabled, platform, last_scanned_at, created_at
         FROM scan_roots WHERE id = ?1",
        params![id],
        |row| {
            Ok(ScanRootDto {
                id: row.get(0)?,
                path: row.get(1)?,
                enabled: row.get::<_, i64>(2)? == 1,
                platform: row.get(3)?,
                last_scanned_at: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    )
    .map_err(|err| err.to_string())
}

fn get_category_by_id(conn: &Connection, id: i64) -> CommandResult<CategoryDto> {
    conn.query_row(
        "SELECT id, name, name_en, color, parent_id FROM categories WHERE id = ?1",
        params![id],
        |row| {
            Ok(CategoryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                name_en: row.get(2)?,
                color: row.get(3)?,
                parent_id: row.get(4)?,
            })
        },
    )
    .map_err(|err| err.to_string())
}

fn skill_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SkillDto> {
    Ok(SkillDto {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        description: row.get(3)?,
        content: row.get(4)?,
        name_zh: row.get(5)?,
        name_en: row.get(6)?,
        description_zh: row.get(7)?,
        description_en: row.get(8)?,
        summary_zh: row.get(9)?,
        summary_en: row.get(10)?,
        category_id: row.get(11)?,
        category_name: row.get(12)?,
        category_color: row.get(13)?,
        source: row.get(14)?,
        platform: row.get(15)?,
        is_custom: row.get::<_, i64>(16)? == 1,
        status: row.get(17)?,
        quality_score: row.get(18)?,
        quality_reason: row.get(19)?,
        usage_count: row.get(20)?,
        duplicate_score: row.get(21)?,
        archive_recommendation: row.get(22)?,
        classification_confidence: row.get(23)?,
        hash: row.get(24)?,
        created_at: row.get(25)?,
        updated_at: row.get(26)?,
        last_scanned_at: row.get(27)?,
        last_used_at: row.get(28)?,
        tags: Vec::new(),
        tool_links: Vec::new(),
    })
}

fn attach_tags(conn: &Connection, mut skills: Vec<SkillDto>) -> CommandResult<Vec<SkillDto>> {
    for skill in &mut skills {
        let mut stmt = conn
            .prepare(
                "SELECT t.name FROM tags t
                 INNER JOIN skill_tags st ON st.tag_id = t.id
                 WHERE st.skill_id = ?1
                 ORDER BY t.name",
            )
            .map_err(|err| err.to_string())?;
        skill.tags = {
            let rows = stmt
                .query_map(params![skill.id], |row| row.get::<_, String>(0))
                .map_err(|err| err.to_string())?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|err| err.to_string())?
        };
        crate::sync_tools::attach_tool_links(conn, skill)?;
    }
    Ok(skills)
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

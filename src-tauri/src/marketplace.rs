use crate::database::AppState;
use crate::hash::sha256_hex;
use crate::scanner::scan_enabled_roots;
use reqwest::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSourceDto {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub last_refreshed_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceItemDto {
    pub id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub skill_url: String,
    pub homepage: String,
    pub installed_skill_id: Option<i64>,
    pub installed_skill_path: String,
    pub installed_version: String,
    pub installed_hash: String,
    pub installed_at: Option<String>,
    pub last_install_check_at: Option<String>,
    pub install_status: String,
    pub install_message: String,
    pub is_update_available: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMarketplaceSourceInput {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSyncConfigDto {
    pub provider: String,
    pub gist_id: String,
    pub has_token: bool,
    pub last_synced_at: Option<String>,
    pub account_name: String,
    pub account_email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCloudSyncConfigInput {
    pub gist_id: String,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MarketplaceFeed {
    skills: Option<Vec<MarketplaceFeedItem>>,
}

#[derive(Debug, Deserialize)]
struct MarketplaceFeedItem {
    id: Option<String>,
    name: String,
    description: Option<String>,
    version: Option<String>,
    author: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    #[serde(alias = "skillUrl")]
    skill_url: Option<String>,
    homepage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPackage {
    pub exported_at: String,
    pub account_name: String,
    pub account_email: String,
    pub skills: Vec<SyncPackageSkill>,
    pub tools: Vec<SyncPackageTool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPackageSkill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub status: String,
    pub source: String,
    pub platform: String,
    pub is_custom: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPackageTool {
    pub tool_name: String,
    pub display_name: String,
    pub skill_dir: String,
    pub enabled: bool,
    pub sync_enabled: bool,
    pub is_custom: bool,
}

#[tauri::command]
pub fn list_marketplace_sources(state: State<AppState>) -> CommandResult<Vec<MarketplaceSourceDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    list_sources_from_conn(&conn)
}

#[tauri::command]
pub fn get_cloud_sync_config(state: State<AppState>) -> CommandResult<CloudSyncConfigDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    Ok(cloud_sync_config_from_conn(&conn)?)
}

#[tauri::command]
pub fn save_cloud_sync_config(
    state: State<AppState>,
    input: SaveCloudSyncConfigInput,
) -> CommandResult<CloudSyncConfigDto> {
    let gist_id = input.gist_id.trim();
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    set_setting(&conn, "cloud.provider", "github_gist")?;
    set_setting(&conn, "cloud.gist_id", gist_id)?;
    if let Some(token) = input.token {
        let trimmed = token.trim();
        if !trimmed.is_empty() && trimmed != "••••••••" {
            set_setting(&conn, "cloud.token", trimmed)?;
        }
    }
    cloud_sync_config_from_conn(&conn)
}

#[tauri::command]
pub fn add_marketplace_source(
    state: State<AppState>,
    input: AddMarketplaceSourceInput,
) -> CommandResult<MarketplaceSourceDto> {
    let name = input.name.trim();
    let url = input.url.trim();
    if name.is_empty() || url.is_empty() {
        return Err("Marketplace 源名称和 URL 不能为空".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Marketplace 源 URL 必须以 http:// 或 https:// 开头".to_string());
    }
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO marketplace_sources (name, url, enabled, created_at, updated_at)
         VALUES (?1, ?2, 1, datetime('now'), datetime('now'))
         ON CONFLICT(url) DO UPDATE SET name = excluded.name, enabled = 1, updated_at = datetime('now')",
        params![name, url],
    )
    .map_err(|err| err.to_string())?;
    let id = conn
        .query_row("SELECT id FROM marketplace_sources WHERE url = ?1", params![url], |row| row.get(0))
        .map_err(|err| err.to_string())?;
    source_by_id(&conn, id)
}

#[tauri::command]
pub fn delete_marketplace_source(state: State<AppState>, id: i64) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute("DELETE FROM marketplace_sources WHERE id = ?1", params![id])
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn refresh_marketplace_source(
    state: State<'_, AppState>,
    source_id: i64,
) -> CommandResult<Vec<MarketplaceItemDto>> {
    let source = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        source_by_id(&conn, source_id)?
    };
    let text = Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|err| err.to_string())?
        .get(&source.url)
        .send()
        .await
        .map_err(|err| format!("请求 Marketplace 源失败：{}", err))?
        .error_for_status()
        .map_err(|err| format!("Marketplace 源返回错误：{}", err))?
        .text()
        .await
        .map_err(|err| err.to_string())?;
    let items = parse_feed(&text)?;
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    for item in items {
        let external_id = item.id.unwrap_or_else(|| safe_slug(&item.name));
        let skill_url = item.skill_url.unwrap_or_default();
        conn.execute(
            "INSERT INTO marketplace_items
             (source_id, external_id, name, description, version, author, category, tags, skill_url, homepage, last_refreshed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
             ON CONFLICT(source_id, external_id) DO UPDATE SET
               name = excluded.name, description = excluded.description, version = excluded.version,
               author = excluded.author, category = excluded.category, tags = excluded.tags,
               skill_url = excluded.skill_url, homepage = excluded.homepage,
               last_refreshed_at = datetime('now')",
            params![
                source_id,
                external_id,
                item.name,
                item.description.unwrap_or_default(),
                item.version.unwrap_or_default(),
                item.author.unwrap_or_default(),
                item.category.unwrap_or_default(),
                item.tags.unwrap_or_default().join(","),
                skill_url,
                item.homepage.unwrap_or_default()
            ],
        )
        .map_err(|err| err.to_string())?;
    }
    conn.execute(
        "UPDATE marketplace_sources SET last_refreshed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
        params![source_id],
    )
    .map_err(|err| err.to_string())?;
    list_items_from_conn(&conn)
}

#[tauri::command]
pub fn list_marketplace_items(state: State<AppState>) -> CommandResult<Vec<MarketplaceItemDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    reconcile_marketplace_installations(&conn)?;
    list_items_from_conn(&conn)
}

#[tauri::command]
pub async fn install_marketplace_item(
    state: State<'_, AppState>,
    item_id: i64,
) -> CommandResult<i64> {
    let (name, description, skill_url, installed_skill_id, repo_path, existing_skill_path) = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        let (name, description, skill_url, installed_skill_id): (String, String, String, Option<i64>) = conn
            .query_row(
                "SELECT name, description, skill_url, installed_skill_id FROM marketplace_items WHERE id = ?1",
                params![item_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                    ))
                },
            )
            .map_err(|err| err.to_string())?;
        let repo = primary_repository_path(&conn)?;
        let existing_skill_path = if let Some(skill_id) = installed_skill_id {
            Some(
                conn.query_row("SELECT path FROM skills WHERE id = ?1", params![skill_id], |row| row.get::<_, String>(0))
                    .map_err(|err| err.to_string())?,
            )
        } else {
            None
        };
        (name, description, skill_url, installed_skill_id, repo, existing_skill_path)
    };
    let content = download_skill_content(&skill_url).await?;
    let target_path = existing_skill_path.map(PathBuf::from).unwrap_or_else(|| unique_child_dir(&repo_path, &safe_slug(&name)));
    write_skill_content(&target_path, &name, &description, &content)?;
    let installed_hash = sha256_hex(&content);
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    scan_enabled_roots(&conn).map_err(|err| err.to_string())?;
    let final_skill_id = if let Some(skill_id) = installed_skill_id {
        skill_id
    } else {
        conn.query_row(
            "SELECT id FROM skills WHERE path = ?1",
            params![target_path.to_string_lossy().to_string()],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?
    };
    conn.execute(
        "UPDATE marketplace_items
         SET installed_skill_id = ?1, installed_version = version, installed_hash = ?2,
             installed_at = COALESCE(installed_at, datetime('now')),
             last_install_check_at = datetime('now'), install_status = '已安装', install_message = ''
         WHERE id = ?3",
        params![final_skill_id, installed_hash, item_id],
    )
    .map_err(|err| err.to_string())?;
    Ok(final_skill_id)
}

#[tauri::command]
pub async fn update_marketplace_item(
    state: State<'_, AppState>,
    item_id: i64,
) -> CommandResult<i64> {
    let (name, description, skill_url, installed_skill_id, version, skill_path) = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        let (name, description, skill_url, installed_skill_id, version): (String, String, String, Option<i64>, String) = conn
            .query_row(
            "SELECT name, description, skill_url, installed_skill_id, version FROM marketplace_items WHERE id = ?1",
            params![item_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|err| err.to_string())?;
        let Some(skill_id) = installed_skill_id else {
            return Err("该 Marketplace 条目尚未安装，无法更新。".to_string());
        };
        let skill_path: String = conn
            .query_row("SELECT path FROM skills WHERE id = ?1", params![skill_id], |row| row.get(0))
            .map_err(|_| "已安装的 Skill 已不存在，请先重新安装。".to_string())?;
        (name, description, skill_url, installed_skill_id, version, skill_path)
    };
    let skill_id = installed_skill_id.expect("checked above");
    let content = download_skill_content(&skill_url).await?;
    write_skill_content(Path::new(&skill_path), &name, &description, &content)?;
    let installed_hash = sha256_hex(&content);
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE marketplace_items
         SET installed_version = ?1, installed_hash = ?2, installed_at = COALESCE(installed_at, datetime('now')),
             last_install_check_at = datetime('now'), install_status = '已安装', install_message = '', installed_skill_id = ?3
         WHERE id = ?4",
        params![version, installed_hash, skill_id, item_id],
    )
    .map_err(|err| err.to_string())?;
    scan_enabled_roots(&conn).map_err(|err| err.to_string())?;
    Ok(skill_id)
}

#[tauri::command]
pub fn uninstall_marketplace_item(state: State<AppState>, item_id: i64) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let (installed_skill_id, installed_skill_path) = conn
        .query_row(
            "SELECT installed_skill_id, s.path
             FROM marketplace_items i LEFT JOIN skills s ON s.id = i.installed_skill_id
             WHERE i.id = ?1",
            params![item_id],
            |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .map_err(|err| err.to_string())?;
    if let Some(skill_id) = installed_skill_id {
        let skill_path = installed_skill_path.unwrap_or_default();
        if !skill_path.is_empty() {
            conn.execute(
                "UPDATE skills SET status = '已归档', updated_at = datetime('now') WHERE id = ?1",
                params![skill_id],
            )
            .map_err(|err| err.to_string())?;
        }
    }
    conn.execute(
        "UPDATE marketplace_items
         SET installed_skill_id = NULL, installed_version = '', installed_hash = '',
             installed_at = NULL, last_install_check_at = datetime('now'),
             install_status = '未安装', install_message = '已解除 Marketplace 关联'
         WHERE id = ?1",
        params![item_id],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn recheck_marketplace_installations(state: State<AppState>) -> CommandResult<Vec<MarketplaceItemDto>> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    reconcile_marketplace_installations(&conn)?;
    list_items_from_conn(&conn)
}

#[tauri::command]
pub fn export_sync_package(state: State<AppState>, path: String) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let mut skill_stmt = conn
        .prepare("SELECT name, description, content, status, source, platform, is_custom FROM skills ORDER BY id")
        .map_err(|err| err.to_string())?;
    let skills = skill_stmt
        .query_map([], |row| {
            Ok(SyncPackageSkill {
                name: row.get(0)?,
                description: row.get(1)?,
                content: row.get(2)?,
                status: row.get(3)?,
                source: row.get(4)?,
                platform: row.get(5)?,
                is_custom: row.get::<_, i64>(6)? == 1,
            })
        })
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    let mut tool_stmt = conn
        .prepare("SELECT tool_name, display_name, skill_dir, enabled, sync_enabled, is_custom FROM tools ORDER BY id")
        .map_err(|err| err.to_string())?;
    let tools = tool_stmt
        .query_map([], |row| {
            Ok(SyncPackageTool {
                tool_name: row.get(0)?,
                display_name: row.get(1)?,
                skill_dir: row.get(2)?,
                enabled: row.get::<_, i64>(3)? == 1,
                sync_enabled: row.get::<_, i64>(4)? == 1,
                is_custom: row.get::<_, i64>(5)? == 1,
            })
        })
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    let package = SyncPackage {
        exported_at: chrono::Utc::now().to_rfc3339(),
        account_name: get_setting(&conn, "account.name")?.unwrap_or_default(),
        account_email: get_setting(&conn, "account.email")?.unwrap_or_default(),
        skills,
        tools,
    };
    let content = serde_json::to_string_pretty(&package).map_err(|err| err.to_string())?;
    fs::write(expand_home(&path), content).map_err(|err| format!("导出同步包失败：{}", err))?;
    Ok(())
}

#[tauri::command]
pub fn import_sync_package(state: State<AppState>, path: String) -> CommandResult<()> {
    let text = fs::read_to_string(expand_home(&path)).map_err(|err| format!("读取同步包失败：{}", err))?;
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    import_sync_package_text(&conn, &text)
}

#[tauri::command]
pub async fn push_sync_package_to_cloud(state: State<'_, AppState>) -> CommandResult<String> {
    let (gist_id, token, payload) = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        let gist_id = get_setting(&conn, "cloud.gist_id")?.unwrap_or_default();
        let token = get_setting(&conn, "cloud.token")?.unwrap_or_default();
        if gist_id.trim().is_empty() {
            return Err("请先填写 Gist ID。".to_string());
        }
        if token.trim().is_empty() {
            return Err("请先填写 GitHub Token。".to_string());
        }
        let package = build_sync_package(&conn)?;
        (
            gist_id,
            token,
            serde_json::to_string_pretty(&package).map_err(|err| err.to_string())?,
        )
    };

    github_api_client(&token)?
        .patch(format!("https://api.github.com/gists/{}", gist_id))
        .json(&serde_json::json!({
            "description": format!("SkillHub sync package {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")),
            "files": {
                "skillhub-sync-package.json": { "content": payload }
            }
        }))
        .send()
        .await
        .map_err(|err| format!("上传云同步包失败：{}", err))?
        .error_for_status()
        .map_err(|err| format!("GitHub Gist 返回错误：{}", err))?;

    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let synced_at = chrono::Utc::now().to_rfc3339();
    set_setting(&conn, "cloud.last_synced_at", &synced_at)?;
    Ok(synced_at)
}

#[tauri::command]
pub async fn pull_sync_package_from_cloud(state: State<'_, AppState>) -> CommandResult<String> {
    let (gist_id, token) = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        let gist_id = get_setting(&conn, "cloud.gist_id")?.unwrap_or_default();
        let token = get_setting(&conn, "cloud.token")?.unwrap_or_default();
        if gist_id.trim().is_empty() {
            return Err("请先填写 Gist ID。".to_string());
        }
        if token.trim().is_empty() {
            return Err("请先填写 GitHub Token。".to_string());
        }
        (gist_id, token)
    };

    let value = github_api_client(&token)?
        .get(format!("https://api.github.com/gists/{}", gist_id))
        .send()
        .await
        .map_err(|err| format!("下载云同步包失败：{}", err))?
        .error_for_status()
        .map_err(|err| format!("GitHub Gist 返回错误：{}", err))?
        .json::<serde_json::Value>()
        .await
        .map_err(|err| format!("解析 Gist 返回失败：{}", err))?;
    let text = value
        .pointer("/files/skillhub-sync-package.json/content")
        .and_then(|item| item.as_str())
        .ok_or_else(|| "Gist 中缺少 skillhub-sync-package.json 文件。".to_string())?;

    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    import_sync_package_text(&conn, text)?;
    let synced_at = chrono::Utc::now().to_rfc3339();
    set_setting(&conn, "cloud.last_synced_at", &synced_at)?;
    Ok(synced_at)
}

fn parse_feed(text: &str) -> CommandResult<Vec<MarketplaceFeedItem>> {
    if let Ok(feed) = serde_json::from_str::<MarketplaceFeed>(text) {
        if let Some(skills) = feed.skills {
            return Ok(skills);
        }
    }
    serde_json::from_str::<Vec<MarketplaceFeedItem>>(text)
        .map_err(|err| format!("Marketplace 源格式错误：{}。需要 JSON 数组或 {{\"skills\": [...]}}。", err))
}

fn list_sources_from_conn(conn: &Connection) -> CommandResult<Vec<MarketplaceSourceDto>> {
    let mut stmt = conn
        .prepare("SELECT id, name, url, enabled, last_refreshed_at FROM marketplace_sources ORDER BY id DESC")
        .map_err(|err| err.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(MarketplaceSourceDto {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            enabled: row.get::<_, i64>(3)? == 1,
            last_refreshed_at: row.get(4)?,
        })
    })
    .map_err(|err| err.to_string())?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|err| err.to_string())?;
    Ok(rows)
}

fn build_sync_package(conn: &Connection) -> CommandResult<SyncPackage> {
    let mut skill_stmt = conn
        .prepare("SELECT name, description, content, status, source, platform, is_custom FROM skills ORDER BY id")
        .map_err(|err| err.to_string())?;
    let skills = skill_stmt
        .query_map([], |row| {
            Ok(SyncPackageSkill {
                name: row.get(0)?,
                description: row.get(1)?,
                content: row.get(2)?,
                status: row.get(3)?,
                source: row.get(4)?,
                platform: row.get(5)?,
                is_custom: row.get::<_, i64>(6)? == 1,
            })
        })
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    let mut tool_stmt = conn
        .prepare("SELECT tool_name, display_name, skill_dir, enabled, sync_enabled, is_custom FROM tools ORDER BY id")
        .map_err(|err| err.to_string())?;
    let tools = tool_stmt
        .query_map([], |row| {
            Ok(SyncPackageTool {
                tool_name: row.get(0)?,
                display_name: row.get(1)?,
                skill_dir: row.get(2)?,
                enabled: row.get::<_, i64>(3)? == 1,
                sync_enabled: row.get::<_, i64>(4)? == 1,
                is_custom: row.get::<_, i64>(5)? == 1,
            })
        })
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    Ok(SyncPackage {
        exported_at: chrono::Utc::now().to_rfc3339(),
        account_name: get_setting(conn, "account.name")?.unwrap_or_default(),
        account_email: get_setting(conn, "account.email")?.unwrap_or_default(),
        skills,
        tools,
    })
}

fn import_sync_package_text(conn: &Connection, text: &str) -> CommandResult<()> {
    let package: SyncPackage =
        serde_json::from_str(text).map_err(|err| format!("同步包格式错误：{}", err))?;
    let repo = primary_repository_path(conn)?;
    fs::create_dir_all(&repo).map_err(|err| format!("创建主仓库失败：{}", err))?;
    for skill in package.skills {
        if skill.content.trim().is_empty() {
            continue;
        }
        let target = unique_child_dir(&repo, &safe_slug(&skill.name));
        fs::create_dir_all(&target).map_err(|err| format!("创建 Skill 目录失败：{}", err))?;
        fs::write(target.join("SKILL.md"), skill.content)
            .map_err(|err| format!("写入 Skill 失败：{}", err))?;
    }
    for tool in package.tools.into_iter().filter(|tool| tool.is_custom) {
        conn.execute(
            "INSERT INTO tools
             (tool_name, display_name, skill_dir, detected, enabled, sync_enabled, is_custom, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, 1, datetime('now'), datetime('now'))
             ON CONFLICT(tool_name) DO UPDATE SET display_name = excluded.display_name,
               skill_dir = excluded.skill_dir, enabled = excluded.enabled, sync_enabled = excluded.sync_enabled,
               is_custom = 1, updated_at = datetime('now')",
            params![
                tool.tool_name,
                tool.display_name,
                tool.skill_dir,
                if tool.enabled { 1 } else { 0 },
                if tool.sync_enabled { 1 } else { 0 }
            ],
        )
        .map_err(|err| err.to_string())?;
    }
    if !package.account_name.trim().is_empty() {
        set_setting(conn, "account.logged_in", "1")?;
        set_setting(conn, "account.name", &package.account_name)?;
        set_setting(conn, "account.email", &package.account_email)?;
    }
    scan_enabled_roots(conn).map_err(|err| err.to_string())?;
    Ok(())
}

fn cloud_sync_config_from_conn(conn: &Connection) -> CommandResult<CloudSyncConfigDto> {
    let token = get_setting(conn, "cloud.token")?.unwrap_or_default();
    Ok(CloudSyncConfigDto {
        provider: get_setting(conn, "cloud.provider")?
            .filter(|item| !item.trim().is_empty())
            .unwrap_or_else(|| "github_gist".to_string()),
        gist_id: get_setting(conn, "cloud.gist_id")?.unwrap_or_default(),
        has_token: !token.trim().is_empty(),
        last_synced_at: get_setting(conn, "cloud.last_synced_at")?,
        account_name: get_setting(conn, "account.name")?.unwrap_or_default(),
        account_email: get_setting(conn, "account.email")?.unwrap_or_default(),
    })
}

fn github_api_client(token: &str) -> CommandResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("SkillHub/0.1.0"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token.trim()))
            .map_err(|err| err.to_string())?,
    );
    Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .default_headers(headers)
        .build()
        .map_err(|err| err.to_string())
}

fn source_by_id(conn: &Connection, id: i64) -> CommandResult<MarketplaceSourceDto> {
    conn.query_row(
        "SELECT id, name, url, enabled, last_refreshed_at FROM marketplace_sources WHERE id = ?1",
        params![id],
        |row| {
            Ok(MarketplaceSourceDto {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                enabled: row.get::<_, i64>(3)? == 1,
                last_refreshed_at: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(|err| err.to_string())?
    .ok_or_else(|| format!("未找到 Marketplace 源：{}", id))
}

fn list_items_from_conn(conn: &Connection) -> CommandResult<Vec<MarketplaceItemDto>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, source_id, external_id, name, description, version, author, category,
                    tags, skill_url, homepage, installed_skill_id, installed_version, installed_hash,
                    installed_at, last_install_check_at, install_status, install_message
             FROM marketplace_items ORDER BY last_refreshed_at DESC, id DESC",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt.query_map([], |row| {
        let tags: String = row.get(8)?;
        Ok(MarketplaceItemDto {
            id: row.get(0)?,
            source_id: row.get(1)?,
            external_id: row.get(2)?,
            name: row.get(3)?,
            description: row.get(4)?,
            version: row.get(5)?,
            author: row.get(6)?,
            category: row.get(7)?,
            tags: tags.split(',').filter(|item| !item.trim().is_empty()).map(|item| item.trim().to_string()).collect(),
            skill_url: row.get(9)?,
            homepage: row.get(10)?,
            installed_skill_id: row.get(11)?,
            installed_version: row.get(12)?,
            installed_hash: row.get(13)?,
            installed_at: row.get(14)?,
            last_install_check_at: row.get(15)?,
            install_status: row.get(16)?,
            install_message: row.get(17)?,
            installed_skill_path: String::new(),
            is_update_available: false,
        })
    })
    .map_err(|err| err.to_string())?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|err| err.to_string())?;
    let mut items = rows;
    for item in &mut items {
        hydrate_marketplace_item(conn, item)?;
    }
    Ok(items)
}

fn hydrate_marketplace_item(conn: &Connection, item: &mut MarketplaceItemDto) -> CommandResult<()> {
    let Some(skill_id) = item.installed_skill_id else {
        item.install_status = "未安装".to_string();
        item.install_message = String::new();
        item.installed_skill_path = String::new();
        item.is_update_available = false;
        return Ok(());
    };
    let skill = conn
        .query_row(
            "SELECT path, name, content, hash, status FROM skills WHERE id = ?1",
            params![skill_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()
        .map_err(|err| err.to_string())?;
    let Some((path, _name, content, hash, status)) = skill else {
        item.installed_skill_id = None;
        item.install_status = "已失联".to_string();
        item.install_message = "已安装的 Skill 已不存在".to_string();
        item.installed_skill_path = String::new();
        item.is_update_available = false;
        return Ok(());
    };
    item.installed_skill_path = path.clone();
    item.installed_hash = if item.installed_hash.trim().is_empty() {
        hash.clone()
    } else {
        item.installed_hash.clone()
    };
    item.is_update_available = !item.version.trim().is_empty()
        && !item.installed_version.trim().is_empty()
        && item.version.trim() != item.installed_version.trim();
    item.install_status = if status == "已归档" {
        "已卸载".to_string()
    } else if item.is_update_available {
        "可更新".to_string()
    } else {
        "已安装".to_string()
    };
    item.install_message = if status == "已归档" {
        "本地 Skill 已归档".to_string()
    } else if item.is_update_available {
        format!("远程 {} / 本地 {}", item.version, item.installed_version)
    } else {
        String::new()
    };
    item.last_install_check_at = Some(chrono::Utc::now().to_rfc3339());
    if item.installed_hash.trim().is_empty() {
        item.installed_hash = sha256_hex(&content);
    }
    Ok(())
}

fn reconcile_marketplace_installations(conn: &Connection) -> CommandResult<()> {
    let mut stmt = conn
        .prepare("SELECT id FROM marketplace_items ORDER BY id DESC")
        .map_err(|err| err.to_string())?;
    let ids = stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|err| err.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    for id in ids {
        let mut item = conn
            .query_row(
                "SELECT id, source_id, external_id, name, description, version, author, category,
                        tags, skill_url, homepage, installed_skill_id, installed_version, installed_hash,
                        installed_at, last_install_check_at, install_status, install_message
                 FROM marketplace_items WHERE id = ?1",
                params![id],
                |row| {
                    let tags: String = row.get(8)?;
                    Ok(MarketplaceItemDto {
                        id: row.get(0)?,
                        source_id: row.get(1)?,
                        external_id: row.get(2)?,
                        name: row.get(3)?,
                        description: row.get(4)?,
                        version: row.get(5)?,
                        author: row.get(6)?,
                        category: row.get(7)?,
                        tags: tags.split(',').filter(|item| !item.trim().is_empty()).map(|item| item.trim().to_string()).collect(),
                        skill_url: row.get(9)?,
                        homepage: row.get(10)?,
                        installed_skill_id: row.get(11)?,
                        installed_version: row.get(12)?,
                        installed_hash: row.get(13)?,
                        installed_at: row.get(14)?,
                        last_install_check_at: row.get(15)?,
                        install_status: row.get(16)?,
                        install_message: row.get(17)?,
                        installed_skill_path: String::new(),
                        is_update_available: false,
                    })
                },
            )
            .optional()
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("未找到 Marketplace 条目：{}", id))?;
        hydrate_marketplace_item(conn, &mut item)?;
        conn.execute(
            "UPDATE marketplace_items
             SET installed_skill_id = ?1, installed_hash = ?2, last_install_check_at = ?3,
                 install_status = ?4, install_message = ?5
             WHERE id = ?6",
            params![
                item.installed_skill_id,
                item.installed_hash,
                item.last_install_check_at,
                item.install_status,
                item.install_message,
                item.id
            ],
        )
        .map_err(|err| err.to_string())?;
    }
    Ok(())
}

async fn download_skill_content(skill_url: &str) -> CommandResult<String> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|err| err.to_string())?
        .get(skill_url)
        .send()
        .await
        .map_err(|err| format!("下载 SKILL.md 失败：{}", err))?
        .error_for_status()
        .map_err(|err| format!("下载 SKILL.md 返回错误：{}", err))?
        .text()
        .await
        .map_err(|err| err.to_string())
}

fn write_skill_content(target: &Path, name: &str, description: &str, content: &str) -> CommandResult<()> {
    fs::create_dir_all(target).map_err(|err| format!("创建 Skill 目录失败：{}", err))?;
    let final_content = if content.trim().is_empty() {
        format!("# {}\n\n{}\n", name, description)
    } else {
        content.to_string()
    };
    fs::write(target.join("SKILL.md"), final_content)
        .map_err(|err| format!("写入 SKILL.md 失败：{}", err))?;
    Ok(())
}

fn primary_repository_path(conn: &Connection) -> CommandResult<PathBuf> {
    let path: Option<String> = conn
        .query_row(
            "SELECT path FROM repositories WHERE is_primary = 1 AND enabled = 1 ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| err.to_string())?;
    path.map(PathBuf::from)
        .ok_or_else(|| "请先在“统一仓库”页面设置主仓库目录".to_string())
}

fn get_setting(conn: &Connection, key: &str) -> CommandResult<Option<String>> {
    conn.query_row("SELECT value FROM app_settings WHERE key = ?1", params![key], |row| row.get(0))
        .optional()
        .map_err(|err| err.to_string())
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> CommandResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        params![key, value],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn unique_child_dir(parent: &Path, slug: &str) -> PathBuf {
    let mut candidate = parent.join(slug);
    let mut index = 2;
    while candidate.exists() {
        candidate = parent.join(format!("{}-{}", slug, index));
        index += 1;
    }
    candidate
}

fn safe_slug(input: &str) -> String {
    let slug = input
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if slug.is_empty() { "marketplace-skill".to_string() } else { slug }
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" || path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(path.trim_start_matches("~/"));
        }
    }
    PathBuf::from(path)
}

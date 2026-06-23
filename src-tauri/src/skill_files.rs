use crate::database::AppState;
use crate::hash::sha256_hex;
use crate::scanner::scan_enabled_roots;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillRequest {
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub fn create_skill_in_repository(
    state: State<AppState>,
    request: CreateSkillRequest,
) -> CommandResult<i64> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err("Skill 名称不能为空".to_string());
    }
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let repo = primary_repository_path(&conn)?;
    fs::create_dir_all(&repo).map_err(|err| format!("创建主仓库失败：{}", err))?;
    let slug = safe_slug(name);
    let skill_dir = unique_child_dir(&repo, &slug);
    fs::create_dir_all(&skill_dir).map_err(|err| format!("创建 skill 文件夹失败：{}", err))?;
    let description = request.description.unwrap_or_default();
    let content = format!(
        "# {}\n\n{}\n\n## Use when\n\nDescribe when this skill should be used.\n\n## Input\n\nDescribe expected input.\n\n## Output\n\nDescribe expected output.\n",
        name,
        if description.trim().is_empty() { "TODO: add skill description." } else { description.trim() }
    );
    fs::write(skill_dir.join("SKILL.md"), &content)
        .map_err(|err| format!("写入 SKILL.md 失败：{}", err))?;
    scan_enabled_roots(&conn).map_err(|err| err.to_string())?;
    conn.query_row(
        "SELECT id FROM skills WHERE path = ?1",
        params![skill_dir.to_string_lossy().to_string()],
        |row| row.get(0),
    )
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn import_skill_to_repository(state: State<AppState>, path: String) -> CommandResult<i64> {
    let source = expand_home(&path);
    if !source.join("SKILL.md").exists() {
        return Err("导入目录必须包含 SKILL.md".to_string());
    }
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let repo = primary_repository_path(&conn)?;
    fs::create_dir_all(&repo).map_err(|err| format!("创建主仓库失败：{}", err))?;
    let slug = source
        .file_name()
        .and_then(|value| value.to_str())
        .map(safe_slug)
        .unwrap_or_else(|| "imported-skill".to_string());
    let target = unique_child_dir(&repo, &slug);
    copy_dir_all(&source, &target)?;
    scan_enabled_roots(&conn).map_err(|err| err.to_string())?;
    conn.query_row(
        "SELECT id FROM skills WHERE path = ?1",
        params![target.to_string_lossy().to_string()],
        |row| row.get(0),
    )
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_skill_content(state: State<AppState>, skill_id: i64, content: String) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let skill_path: String = conn
        .query_row("SELECT path FROM skills WHERE id = ?1", params![skill_id], |row| row.get(0))
        .map_err(|err| err.to_string())?;
    let file_path = Path::new(&skill_path).join("SKILL.md");
    if !file_path.exists() {
        return Err(format!("SKILL.md 不存在：{}", file_path.display()));
    }
    fs::write(&file_path, &content).map_err(|err| format!("保存 SKILL.md 失败：{}", err))?;
    let name = extract_title(&content).unwrap_or_else(|| {
        Path::new(&skill_path)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("未命名 Skill")
            .to_string()
    });
    let description = extract_description(&content);
    conn.execute(
        "UPDATE skills SET name = ?1, description = ?2, content = ?3,
         name_zh = CASE WHEN name_zh = '' THEN ?1 ELSE name_zh END,
         description_zh = CASE WHEN description_zh = '' THEN ?2 ELSE description_zh END,
         hash = ?4, updated_at = datetime('now'), last_scanned_at = datetime('now')
         WHERE id = ?5",
        params![name, description, content, sha256_hex(&content), skill_id],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn primary_repository_path(conn: &rusqlite::Connection) -> CommandResult<PathBuf> {
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

fn copy_dir_all(source: &Path, target: &Path) -> CommandResult<()> {
    fs::create_dir_all(target).map_err(|err| format!("创建目标目录失败：{}", err))?;
    for entry in fs::read_dir(source).map_err(|err| format!("读取源目录失败：{}", err))? {
        let entry = entry.map_err(|err| format!("读取源目录项失败：{}", err))?;
        let file_type = entry.file_type().map_err(|err| err.to_string())?;
        let next_target = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &next_target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), next_target).map_err(|err| format!("复制文件失败：{}", err))?;
        }
    }
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
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else if ch.is_whitespace() {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if slug.is_empty() { "new-skill".to_string() } else { slug }
}

fn extract_title(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let title = trimmed.trim_start_matches('#').trim();
            if title.is_empty() { None } else { Some(title.to_string()) }
        } else {
            None
        }
    })
}

fn extract_description(content: &str) -> String {
    let mut seen_title = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "---" {
            continue;
        }
        if trimmed.starts_with('#') {
            seen_title = true;
            continue;
        }
        if seen_title && !trimmed.starts_with('|') && !trimmed.starts_with('-') {
            return trimmed.trim_matches('"').to_string();
        }
    }
    String::new()
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" || path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(path.trim_start_matches("~/"));
        }
    }
    PathBuf::from(path)
}

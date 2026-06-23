use crate::hash::sha256_hex;
use crate::similarity::has_version_noise;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Default)]
pub struct ScanSummary {
    pub total_found: usize,
    pub new_count: usize,
    pub changed_count: usize,
    pub duplicate_count: usize,
    pub missing_count: usize,
}

#[derive(Debug)]
struct SkillScanItem {
    folder_path: PathBuf,
    file_path: PathBuf,
    name: String,
    description: String,
    content: String,
    hash: String,
    source: String,
    platform: String,
    quality_score: i64,
    quality_reason: String,
    duplicate_score: f64,
    category_name: String,
    classification_confidence: f64,
    archive_recommendation: String,
    summary: String,
    tags: Vec<String>,
}

pub fn scan_enabled_roots(conn: &Connection) -> rusqlite::Result<ScanSummary> {
    let roots = {
        let mut stmt =
            conn.prepare("SELECT path, platform FROM scan_roots WHERE enabled = 1 ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let collected = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        collected
    };

    let mut summary = ScanSummary::default();

    for (root, platform) in roots {
        let root_path = expand_home(&root);
        let root_summary = scan_one_root(conn, &root_path, &platform)?;
        summary.total_found += root_summary.total_found;
        summary.new_count += root_summary.new_count;
        summary.changed_count += root_summary.changed_count;
        summary.duplicate_count += root_summary.duplicate_count;
        summary.missing_count += root_summary.missing_count;

        conn.execute(
            "UPDATE scan_roots SET last_scanned_at = datetime('now') WHERE path = ?1",
            params![root],
        )?;
        conn.execute(
            "INSERT INTO scan_history
             (root_path, total_found, new_count, changed_count, duplicate_count, scanned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            params![
                root,
                root_summary.total_found as i64,
                root_summary.new_count as i64,
                root_summary.changed_count as i64,
                root_summary.duplicate_count as i64
            ],
        )?;
    }

    refresh_duplicate_scores(conn)?;
    Ok(summary)
}

fn scan_one_root(
    conn: &Connection,
    root_path: &Path,
    platform: &str,
) -> rusqlite::Result<ScanSummary> {
    let mut summary = ScanSummary::default();
    if !root_path.exists() {
        return Ok(summary);
    }

    let mut found_paths = HashSet::new();

    for entry in WalkDir::new(root_path)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "SKILL.md")
    {
        let file_path = entry.path().to_path_buf();
        let Some(folder_path) = file_path.parent().map(Path::to_path_buf) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(&file_path) else {
            continue;
        };

        let item = build_scan_item(folder_path, file_path, content, platform);
        found_paths.insert(item.folder_path.to_string_lossy().to_string());
        upsert_skill(conn, &item, &mut summary)?;
        summary.total_found += 1;
    }

    let like_prefix = format!("{}%", root_path.to_string_lossy());
    let existing_paths = {
        let mut stmt = conn.prepare("SELECT path FROM skills WHERE path LIKE ?1")?;
        let rows = stmt.query_map(params![like_prefix], |row| row.get::<_, String>(0))?;
        let collected = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        collected
    };

    for path in existing_paths {
        if !found_paths.contains(&path) {
            conn.execute(
                "UPDATE skills SET status = '路径丢失', updated_at = datetime('now') WHERE path = ?1",
                params![path],
            )?;
            summary.missing_count += 1;
        }
    }

    Ok(summary)
}

fn build_scan_item(
    folder_path: PathBuf,
    file_path: PathBuf,
    content: String,
    platform: &str,
) -> SkillScanItem {
    let fallback_name = folder_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("未命名 Skill")
        .to_string();
    let name = extract_title(&content).unwrap_or(fallback_name);
    let description = extract_description(&content);
    let hash = sha256_hex(&content);
    let (quality_score, quality_reason) = score_quality(&name, &description, &content);
    let duplicate_score = if has_version_noise(&name) { 35.0 } else { 0.0 };
    let source = infer_source(&folder_path, platform);
    let analysis = analyze_skill(&name, &description, &content, duplicate_score);

    SkillScanItem {
        folder_path,
        file_path,
        name,
        description,
        content,
        hash,
        source,
        platform: platform.to_string(),
        quality_score,
        quality_reason,
        duplicate_score,
        category_name: analysis.category_name,
        classification_confidence: analysis.classification_confidence,
        archive_recommendation: analysis.archive_recommendation,
        summary: analysis.summary,
        tags: analysis.tags,
    }
}

fn upsert_skill(
    conn: &Connection,
    item: &SkillScanItem,
    summary: &mut ScanSummary,
) -> rusqlite::Result<()> {
    let path = item.folder_path.to_string_lossy().to_string();
    let existing: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, hash FROM skills WHERE path = ?1",
            params![path],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match existing {
        Some((id, old_hash)) => {
            if old_hash != item.hash {
                summary.changed_count += 1;
            }
            let category_id = ensure_category(conn, &item.category_name)?;
            conn.execute(
                "UPDATE skills SET
                  name = ?1,
                  description = ?2,
                  content = ?3,
                  source = ?4,
                  platform = ?5,
                  quality_score = ?6,
                  quality_reason = ?7,
                  duplicate_score = MAX(duplicate_score, ?8),
                  hash = ?9,
                  category_id = COALESCE(category_id, ?11),
                  name_zh = CASE WHEN name_zh = '' THEN ?1 ELSE name_zh END,
                  description_zh = CASE WHEN description_zh = '' THEN ?2 ELSE description_zh END,
                  summary_zh = ?12,
                  archive_recommendation = ?13,
                  classification_confidence = ?14,
                  status = CASE WHEN status = '路径丢失' THEN '正常' ELSE status END,
                  updated_at = datetime('now'),
                  last_scanned_at = datetime('now')
                 WHERE path = ?10",
                params![
                    item.name,
                    item.description,
                    item.content,
                    item.source,
                    item.platform,
                    item.quality_score,
                    item.quality_reason,
                    item.duplicate_score,
                    item.hash,
                    path,
                    category_id,
                    item.summary,
                    item.archive_recommendation,
                    item.classification_confidence
                ],
            )?;
            sync_tags(conn, id, &item.tags)?;
        }
        None => {
            summary.new_count += 1;
            let category_id = ensure_category(conn, &item.category_name)?;
            conn.execute(
                "INSERT INTO skills
                 (name, path, description, content, category_id, source, platform, is_custom, status,
                  quality_score, quality_reason, usage_count, duplicate_score, hash,
                  name_zh, description_zh, summary_zh, archive_recommendation, classification_confidence,
                  created_at, updated_at, last_scanned_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '正常',
                  ?9, ?10, 0, ?11, ?12, ?1, ?3, ?13, ?14, ?15,
                  datetime('now'), datetime('now'), datetime('now'))",
                params![
                    item.name,
                    path,
                    item.description,
                    item.content,
                    category_id,
                    item.source,
                    item.platform,
                    if item.source == "自建" { 1 } else { 0 },
                    item.quality_score,
                    item.quality_reason,
                    item.duplicate_score,
                    item.hash,
                    item.summary,
                    item.archive_recommendation,
                    item.classification_confidence
                ],
            )?;
            let id = conn.last_insert_rowid();
            sync_tags(conn, id, &item.tags)?;
        }
    }

    let _ = &item.file_path;
    Ok(())
}

struct SkillAnalysis {
    category_name: String,
    classification_confidence: f64,
    archive_recommendation: String,
    summary: String,
    tags: Vec<String>,
}

fn analyze_skill(
    name: &str,
    description: &str,
    content: &str,
    duplicate_score: f64,
) -> SkillAnalysis {
    let text = format!("{}\n{}\n{}", name, description, content).to_lowercase();
    let rules: [(&str, &[&str], &[&str]); 19] = [
        ("图像设计", &["image", "图片", "图像", "视觉", "design", "生成图"], &["图像", "设计"]),
        ("电商修图", &["电商", "商品", "白底", "retouch", "修图", "产品图"], &["电商", "修图"]),
        ("海报设计", &["海报", "poster", "banner", "视觉海报"], &["海报"]),
        ("PPT / 演示", &["ppt", "powerpoint", "演示", "slide", "presentation"], &["PPT"]),
        ("Word / 文档", &["word", "docx", "文档", "document"], &["文档"]),
        ("PDF 处理", &["pdf"], &["PDF"]),
        ("Excel / 表格", &["excel", "xlsx", "spreadsheet", "表格"], &["表格"]),
        ("前端开发", &["frontend", "react", "vue", "css", "html", "前端", "ui"], &["前端"]),
        ("后端开发", &["backend", "api", "server", "数据库", "后端"], &["后端"]),
        ("代码开发", &["code", "coding", "代码", "github", "repo", "debug"], &["代码"]),
        ("自动化", &["automation", "自动化", "脚本", "workflow"], &["自动化"]),
        ("知识库 / RAG", &["rag", "知识库", "向量", "检索", "embedding"], &["RAG"]),
        ("数据分析", &["data", "analysis", "数据", "分析", "chart"], &["数据"]),
        ("AI Agent", &["agent", "代理", "multi-agent", "mcp"], &["Agent"]),
        ("提示词优化", &["prompt", "提示词", "system prompt"], &["提示词"]),
        ("医疗设计", &["医疗", "medicine", "药品", "health"], &["医疗"]),
        ("品牌设计", &["brand", "logo", "品牌", "vi"], &["品牌"]),
        ("办公效率", &["office", "办公", "效率", "productivity"], &["办公"]),
        ("系统工具", &["macos", "shell", "terminal", "系统", "cli"], &["系统"]),
    ];

    let mut best_category = "未分类";
    let mut best_score = 0usize;
    let mut tags = Vec::new();
    for (category, keywords, category_tags) in rules {
        let score = keywords.iter().filter(|keyword| text.contains(&keyword.to_lowercase())).count();
        if score > 0 {
            for tag in category_tags {
                if !tags.iter().any(|item| item == tag) {
                    tags.push((*tag).to_string());
                }
            }
        }
        if score > best_score {
            best_score = score;
            best_category = category;
        }
    }

    if contains_any(content, &["Use when", "适用", "场景"]) {
        tags.push("场景明确".to_string());
    }
    if contains_any(content, &["Input", "输入", "Output", "输出"]) {
        tags.push("输入输出明确".to_string());
    }
    if duplicate_score > 0.0 {
        tags.push("疑似重复".to_string());
    }

    let archive_recommendation = if duplicate_score >= 100.0 {
        "疑似重复"
    } else if has_version_noise(name) {
        "疑似旧版本"
    } else if content.chars().count() < 180 {
        "建议检查"
    } else if best_score == 0 {
        "建议检查"
    } else {
        "建议保留"
    }
    .to_string();

    let summary = if !description.trim().is_empty() {
        description.chars().take(220).collect()
    } else {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("---")
            })
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(220)
            .collect()
    };

    SkillAnalysis {
        category_name: best_category.to_string(),
        classification_confidence: (best_score as f64 / 4.0).min(1.0),
        archive_recommendation,
        summary,
        tags,
    }
}

fn ensure_category(conn: &Connection, name: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT id FROM categories WHERE name = ?1", params![name], |row| {
        row.get(0)
    })
    .optional()
}

fn sync_tags(conn: &Connection, skill_id: i64, tags: &[String]) -> rusqlite::Result<()> {
    for tag in tags.iter().filter(|tag| !tag.trim().is_empty()).take(8) {
        conn.execute(
            "INSERT OR IGNORE INTO tags (name, created_at, updated_at)
             VALUES (?1, datetime('now'), datetime('now'))",
            params![tag],
        )?;
        let tag_id: i64 = conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO skill_tags (skill_id, tag_id) VALUES (?1, ?2)",
            params![skill_id, tag_id],
        )?;
    }
    Ok(())
}

fn refresh_duplicate_scores(conn: &Connection) -> rusqlite::Result<()> {
    let mut hash_counts: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT hash, COUNT(*) FROM skills GROUP BY hash")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?;
        for row in rows {
            let (hash, count) = row?;
            hash_counts.insert(hash, count);
        }
    }

    let skills = {
        let mut stmt = conn.prepare("SELECT id, name, hash, duplicate_score FROM skills")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
            ))
        })?;
        let collected = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        collected
    };

    for (id, name, hash, current) in skills {
        let mut score = if hash_counts.get(&hash).copied().unwrap_or(0) > 1 {
            100.0
        } else {
            current.min(35.0)
        };
        if has_version_noise(&name) {
            score = score.max(35.0);
        }
        conn.execute(
            "UPDATE skills SET duplicate_score = ?1 WHERE id = ?2",
            params![score, id],
        )?;
    }

    Ok(())
}

fn extract_title(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let title = trimmed.trim_start_matches('#').trim();
            if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            }
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
    content
        .lines()
        .find(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .unwrap_or("")
        .trim()
        .chars()
        .take(140)
        .collect()
}

fn score_quality(name: &str, description: &str, content: &str) -> (i64, String) {
    let checks = [
        ("标题清晰", !name.trim().is_empty()),
        ("有描述", !description.trim().is_empty()),
        ("描述足够长", description.chars().count() >= 20),
        ("有适用场景", contains_any(content, &["适用", "Use when", "When to use", "场景"])),
        ("有输入说明", contains_any(content, &["输入", "Input", "参数", "request"])),
        ("有输出说明", contains_any(content, &["输出", "Output", "deliver", "返回"])),
        ("有示例", contains_any(content, &["示例", "Example", "examples", "```"])),
        ("有边界限制", contains_any(content, &["不要", "禁止", "Never", "Do not", "限制"])),
        ("结构清晰", content.matches('\n').count() >= 8 && content.matches('#').count() >= 2),
        (
            "内容长度合理",
            (300..=12000).contains(&content.chars().count()),
        ),
    ];

    let mut score = 0;
    let mut reasons = Vec::new();
    for (label, passed) in checks {
        if passed {
            score += 10;
            reasons.push(format!("{}：+10", label));
        } else {
            reasons.push(format!("{}：+0", label));
        }
    }

    (score, reasons.join("\n"))
}

fn contains_any(content: &str, needles: &[&str]) -> bool {
    let lower = content.to_lowercase();
    needles
        .iter()
        .any(|needle| lower.contains(&needle.to_lowercase()))
}

fn infer_source(path: &Path, platform: &str) -> String {
    let text = path.to_string_lossy().to_lowercase();
    if platform != "未知" {
        platform.to_string()
    } else if text.contains("codex") {
        "Codex".to_string()
    } else if text.contains("chatgpt") {
        "ChatGPT".to_string()
    } else if text.contains("claude") {
        "Claude".to_string()
    } else if text.contains("hermes") {
        "Hermes".to_string()
    } else if text.contains("cursor") {
        "Cursor".to_string()
    } else if text.contains("my") || text.contains("自建") {
        "自建".to_string()
    } else {
        "未知".to_string()
    }
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" || path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(path.trim_start_matches("~/"));
        }
    }
    PathBuf::from(path)
}

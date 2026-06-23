use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

pub struct AppState {
    pub conn: Mutex<Connection>,
}

pub fn init_database(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch(SCHEMA)?;
    migrate_database(&conn)?;
    seed_categories(&conn)?;
    seed_tools(&conn)?;
    Ok(conn)
}

fn seed_categories(conn: &Connection) -> rusqlite::Result<()> {
    let categories = [
        ("图像设计", "#2563eb"),
        ("电商修图", "#0ea5e9"),
        ("海报设计", "#7c3aed"),
        ("PPT / 演示", "#f97316"),
        ("Word / 文档", "#64748b"),
        ("PDF 处理", "#ef4444"),
        ("Excel / 表格", "#16a34a"),
        ("代码开发", "#0f172a"),
        ("前端开发", "#06b6d4"),
        ("后端开发", "#334155"),
        ("自动化", "#8b5cf6"),
        ("知识库 / RAG", "#0891b2"),
        ("数据分析", "#059669"),
        ("AI Agent", "#1d4ed8"),
        ("提示词优化", "#9333ea"),
        ("医疗设计", "#dc2626"),
        ("品牌设计", "#db2777"),
        ("办公效率", "#475569"),
        ("系统工具", "#52525b"),
        ("未分类", "#94a3b8"),
    ];

    for (name, color) in categories {
        conn.execute(
            "INSERT OR IGNORE INTO categories (name, color, created_at, updated_at)
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            params![name, color],
        )?;
    }

    Ok(())
}

fn seed_tools(conn: &Connection) -> rusqlite::Result<()> {
    let tools = [
        ("codex", "Codex", "~/.codex/skills"),
        ("claude_code", "Claude Code", "~/.claude/skills"),
        ("codebuddy", "CodeBuddy", "~/codebuddy-skill"),
        ("hermes", "Hermes", "~/.hermes/skills"),
        ("cursor", "Cursor", "~/.cursor/skills"),
        ("opencode", "Opencode", "~/.opencode/skills"),
    ];

    for (tool_name, display_name, skill_dir) in tools {
        conn.execute(
            "INSERT OR IGNORE INTO tools
             (tool_name, display_name, skill_dir, detected, enabled, sync_enabled, last_checked_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, 1, 1, NULL, datetime('now'), datetime('now'))",
            params![tool_name, display_name, skill_dir],
        )?;
    }

    Ok(())
}

fn migrate_database(conn: &Connection) -> rusqlite::Result<()> {
    add_column_if_missing(conn, "skills", "name_zh", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "skills", "name_en", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "skills", "description_zh", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "skills", "description_en", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "skills", "summary_zh", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "skills", "summary_en", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "skills", "primary_repo_path", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "skills", "last_used_at", "TEXT")?;
    add_column_if_missing(
        conn,
        "skills",
        "archive_recommendation",
        "TEXT NOT NULL DEFAULT '建议保留'",
    )?;
    add_column_if_missing(
        conn,
        "skills",
        "classification_confidence",
        "REAL NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(conn, "categories", "name_en", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "scan_roots", "updated_at", "TEXT")?;
    add_column_if_missing(conn, "tools", "is_custom", "INTEGER NOT NULL DEFAULT 0")?;
    Ok(())
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if !columns.iter().any(|item| item == column) {
        conn.execute(&format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition), [])?;
    }
    Ok(())
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  name_en TEXT NOT NULL DEFAULT '',
  color TEXT NOT NULL DEFAULT '#94a3b8',
  parent_id INTEGER,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS skills (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT '',
  category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
  source TEXT NOT NULL DEFAULT '未知',
  platform TEXT NOT NULL DEFAULT '未知',
  is_custom INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT '正常',
  quality_score INTEGER NOT NULL DEFAULT 0,
  quality_reason TEXT NOT NULL DEFAULT '',
  usage_count INTEGER NOT NULL DEFAULT 0,
  duplicate_score REAL NOT NULL DEFAULT 0,
  hash TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  last_scanned_at TEXT
);

CREATE TABLE IF NOT EXISTS tools (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tool_name TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  skill_dir TEXT NOT NULL DEFAULT '',
  detected INTEGER NOT NULL DEFAULT 0,
  enabled INTEGER NOT NULL DEFAULT 1,
  sync_enabled INTEGER NOT NULL DEFAULT 1,
  is_custom INTEGER NOT NULL DEFAULT 0,
  last_checked_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS repositories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  type TEXT NOT NULL DEFAULT 'local',
  enabled INTEGER NOT NULL DEFAULT 1,
  is_primary INTEGER NOT NULL DEFAULT 0,
  last_scanned_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS skill_tool_links (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  skill_id INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
  tool_name TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 0,
  link_path TEXT NOT NULL DEFAULT '',
  link_status TEXT NOT NULL DEFAULT '未同步',
  last_synced_at TEXT,
  error_message TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(skill_id, tool_name)
);

CREATE TABLE IF NOT EXISTS sync_issues (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  skill_id INTEGER REFERENCES skills(id) ON DELETE CASCADE,
  tool_name TEXT NOT NULL,
  issue_type TEXT NOT NULL,
  current_path TEXT NOT NULL DEFAULT '',
  expected_path TEXT NOT NULL DEFAULT '',
  severity TEXT NOT NULL DEFAULT 'warning',
  fixable INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'open',
  message TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS skill_tags (
  skill_id INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
  tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (skill_id, tag_id)
);

CREATE TABLE IF NOT EXISTS scan_roots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  path TEXT NOT NULL UNIQUE,
  enabled INTEGER NOT NULL DEFAULT 1,
  platform TEXT NOT NULL DEFAULT '未知',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  last_scanned_at TEXT
);

CREATE TABLE IF NOT EXISTS duplicate_groups (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT '',
  score REAL NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT '待处理',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS duplicate_group_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  group_id INTEGER NOT NULL REFERENCES duplicate_groups(id) ON DELETE CASCADE,
  skill_id INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
  recommendation TEXT NOT NULL DEFAULT '',
  reason TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS scan_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  root_path TEXT NOT NULL,
  total_found INTEGER NOT NULL DEFAULT 0,
  new_count INTEGER NOT NULL DEFAULT 0,
  changed_count INTEGER NOT NULL DEFAULT 0,
  duplicate_count INTEGER NOT NULL DEFAULT 0,
  scanned_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_skills_status ON skills(status);
CREATE INDEX IF NOT EXISTS idx_skills_hash ON skills(hash);
CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category_id);
CREATE INDEX IF NOT EXISTS idx_skills_updated ON skills(updated_at);
CREATE INDEX IF NOT EXISTS idx_skill_tool_links_skill ON skill_tool_links(skill_id);
CREATE INDEX IF NOT EXISTS idx_sync_issues_status ON sync_issues(status);
"#;

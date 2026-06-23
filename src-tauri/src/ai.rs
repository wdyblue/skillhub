use crate::commands::SkillDto;
use crate::database::AppState;
use reqwest::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfigDto {
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfigInput {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub logged_in: bool,
    pub name: String,
    pub email: String,
    pub avatar_initial: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginInput {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateSkillInput {
    pub skill_id: i64,
    pub target_language: String,
}

#[derive(Debug, Deserialize)]
struct ModelTranslation {
    name: String,
    description: String,
    summary: String,
}

#[derive(Debug)]
struct FullAiConfig {
    base_url: String,
    api_key: String,
    model: String,
}

#[tauri::command]
pub fn get_ai_config(state: State<AppState>, reveal_secret: Option<bool>) -> CommandResult<AiConfigDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    let api_key = get_setting(&conn, "ai.api_key")?.unwrap_or_default();
    Ok(AiConfigDto {
        base_url: get_setting(&conn, "ai.base_url")?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        model: get_setting(&conn, "ai.model")?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "gpt-4o-mini".to_string()),
        has_api_key: !api_key.trim().is_empty(),
        api_key: if reveal_secret.unwrap_or(false) {
            api_key
        } else {
            String::new()
        },
    })
}

#[tauri::command]
pub fn save_ai_config(state: State<AppState>, input: AiConfigInput) -> CommandResult<AiConfigDto> {
    let base_url = normalize_base_url(&input.base_url)?;
    let model = input.model.trim();
    if model.is_empty() {
        return Err("模型名称不能为空".to_string());
    }

    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    set_setting(&conn, "ai.base_url", &base_url)?;
    set_setting(&conn, "ai.model", model)?;
    if let Some(api_key) = input.api_key {
        if !api_key.trim().is_empty() && api_key.trim() != "••••••••" {
            set_setting(&conn, "ai.api_key", api_key.trim())?;
        }
    }
    drop(conn);
    get_ai_config(state, Some(false))
}

#[tauri::command]
pub async fn list_ai_models(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    let config = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        read_full_ai_config(&conn)?
    };
    let endpoint = format!("{}/models", config.base_url.trim_end_matches('/'));
    let client = ai_http_client()?;
    let response = client
        .get(endpoint)
        .bearer_auth(&config.api_key)
        .send()
        .await
        .map_err(|err| format!("请求模型列表失败：{}", describe_reqwest_error(&err)))?;
    let status = response.status();
    let text = response.text().await.map_err(|err| err.to_string())?;
    if !status.is_success() {
        return Err(format!("模型列表接口返回 {}：{}", status.as_u16(), clip_chars(&text, 500)));
    }
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("模型列表返回不是 JSON：{}；原文：{}", err, clip_chars(&text, 500)))?;
    let models = value
        .get("data")
        .and_then(|item| item.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.get("id")
                        .or_else(|| item.get("name"))
                        .and_then(|value| value.as_str())
                        .map(|value| value.to_string())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if models.is_empty() {
        return Err("模型列表为空，请检查 API Key 权限或服务商配置。".to_string());
    }
    Ok(models)
}

#[tauri::command]
pub async fn test_ai_connection(state: State<'_, AppState>) -> CommandResult<String> {
    let config = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        read_full_ai_config(&conn)?
    };
    call_chat_completion(
        &config,
        "请只回复：连接成功",
        "你是 SkillHub 的连接测试助手，只需要极简回复。",
        16,
    )
    .await
    .map(|_| "连接成功".to_string())
}

#[tauri::command]
pub async fn translate_skill(
    state: State<'_, AppState>,
    input: TranslateSkillInput,
) -> CommandResult<SkillDto> {
    let target = normalize_target_language(&input.target_language)?;
    let (config, name, description, content) = {
        let conn = state.conn.lock().map_err(|err| err.to_string())?;
        let config = read_full_ai_config(&conn)?;
        let skill = conn
            .query_row(
                "SELECT name, description, content FROM skills WHERE id = ?1",
                params![input.skill_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|err| err.to_string())?;
        (config, skill.0, skill.1, skill.2)
    };

    let clipped_content = clip_chars(&content, 6_000);
    let prompt = format!(
        r#"请把下面这个 AI Skill 的名称、描述和摘要翻译成{target_label}。
要求：
1. 保留技术名词、命令、路径、代码标识符和产品名。
2. 不要改写原始 Skill Markdown 文件，只返回翻译结果。
3. 严格返回 JSON，不要包裹 Markdown。
JSON 格式：
{{"name":"...","description":"...","summary":"..."}}

原始名称：
{name}

原始描述：
{description}

SKILL.md 内容：
{clipped_content}"#,
        target_label = if target == "zh" { "简体中文" } else { "英文" }
    );

    let raw = call_chat_completion(
        &config,
        &prompt,
        "你是专业技术文档翻译助手，输出必须是可解析 JSON。",
        800,
    )
    .await?;
    let parsed = parse_translation(&raw)?;

    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    if target == "zh" {
        conn.execute(
            "UPDATE skills
             SET name_zh = ?1, description_zh = ?2, summary_zh = ?3, updated_at = datetime('now')
             WHERE id = ?4",
            params![parsed.name, parsed.description, parsed.summary, input.skill_id],
        )
        .map_err(|err| err.to_string())?;
    } else {
        conn.execute(
            "UPDATE skills
             SET name_en = ?1, description_en = ?2, summary_en = ?3, updated_at = datetime('now')
             WHERE id = ?4",
            params![parsed.name, parsed.description, parsed.summary, input.skill_id],
        )
        .map_err(|err| err.to_string())?;
    }
    crate::commands::get_skill_by_id(&conn, input.skill_id)
}

#[tauri::command]
pub fn get_account(state: State<AppState>) -> CommandResult<AccountDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    account_from_conn(&conn)
}

#[tauri::command]
pub fn login_account(state: State<AppState>, input: AccountLoginInput) -> CommandResult<AccountDto> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("账号名称不能为空".to_string());
    }
    let email = input.email.unwrap_or_default();
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    set_setting(&conn, "account.logged_in", "1")?;
    set_setting(&conn, "account.name", name)?;
    set_setting(&conn, "account.email", email.trim())?;
    account_from_conn(&conn)
}

#[tauri::command]
pub fn logout_account(state: State<AppState>) -> CommandResult<AccountDto> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    set_setting(&conn, "account.logged_in", "0")?;
    account_from_conn(&conn)
}

#[tauri::command]
pub fn clear_translation_cache(state: State<AppState>) -> CommandResult<()> {
    let conn = state.conn.lock().map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE skills SET
          name_zh = '', name_en = '', description_zh = '', description_en = '',
          summary_zh = '', summary_en = '', updated_at = datetime('now')",
        [],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn read_full_ai_config(conn: &Connection) -> CommandResult<FullAiConfig> {
    let base_url = get_setting(conn, "ai.base_url")?
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
    let api_key = get_setting(conn, "ai.api_key")?.unwrap_or_default();
    let model = get_setting(conn, "ai.model")?
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "gpt-4o-mini".to_string());
    if api_key.trim().is_empty() {
        return Err("请先在设置里填写 AI 翻译 API Key。".to_string());
    }
    Ok(FullAiConfig {
        base_url: normalize_base_url(&base_url)?,
        api_key,
        model,
    })
}

async fn call_chat_completion(config: &FullAiConfig, user_prompt: &str, system_prompt: &str, max_tokens: u32) -> CommandResult<String> {
    let endpoint = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let client = ai_http_client()?;
    let response = client
        .post(endpoint)
        .bearer_auth(&config.api_key)
        .json(&json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.2,
            "max_tokens": max_tokens,
            "stream": false
        }))
        .send()
        .await
        .map_err(|err| format!("请求 AI 接口失败：{}", describe_reqwest_error(&err)))?;
    let status = response.status();
    let text = response.text().await.map_err(|err| err.to_string())?;
    if !status.is_success() {
        return Err(format!("AI 接口返回 {}：{}", status.as_u16(), clip_chars(&text, 500)));
    }
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("AI 返回不是 JSON：{}；原文：{}", err, clip_chars(&text, 500)))?;
    value
        .pointer("/choices/0/message/content")
        .and_then(|item| item.as_str())
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .ok_or_else(|| format!("AI 返回缺少 choices[0].message.content：{}", clip_chars(&text, 500)))
}

fn ai_http_client() -> CommandResult<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .connect_timeout(std::time::Duration::from_secs(20))
        .http1_only()
        .build()
        .map_err(|err| format!("创建 HTTP 客户端失败：{}", describe_reqwest_error(&err)))
}

fn describe_reqwest_error(err: &reqwest::Error) -> String {
    let mut parts = vec![err.to_string()];
    let mut source = std::error::Error::source(err);
    while let Some(next) = source {
        parts.push(next.to_string());
        source = next.source();
    }
    parts.join("；原因：")
}

fn parse_translation(raw: &str) -> CommandResult<ModelTranslation> {
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    serde_json::from_str::<ModelTranslation>(cleaned)
        .map_err(|err| format!("翻译结果 JSON 解析失败：{}；原文：{}", err, clip_chars(raw, 500)))
}

fn account_from_conn(conn: &Connection) -> CommandResult<AccountDto> {
    let logged_in = get_setting(conn, "account.logged_in")?.unwrap_or_default() == "1";
    let name = get_setting(conn, "account.name")?
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "TONYWU".to_string());
    let email = get_setting(conn, "account.email")?.unwrap_or_default();
    let avatar_initial = name
        .chars()
        .find(|item| item.is_ascii_alphanumeric() || !item.is_whitespace())
        .map(|item| item.to_uppercase().to_string())
        .unwrap_or_else(|| "T".to_string());
    Ok(AccountDto {
        logged_in,
        name,
        email,
        avatar_initial,
    })
}

fn get_setting(conn: &Connection, key: &str) -> CommandResult<Option<String>> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
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

fn normalize_base_url(value: &str) -> CommandResult<String> {
    let trimmed = value.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        return Err("Base URL 不能为空".to_string());
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("Base URL 必须以 http:// 或 https:// 开头".to_string());
    }
    Ok(trimmed)
}

fn normalize_target_language(value: &str) -> CommandResult<String> {
    match value {
        "zh" | "zh-CN" | "中文" => Ok("zh".to_string()),
        "en" | "英文" => Ok("en".to_string()),
        other => Err(format!("暂不支持的翻译目标语言：{}", other)),
    }
}

fn clip_chars(value: &str, max_chars: usize) -> String {
    let clipped = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        format!("{}…", clipped)
    } else {
        clipped
    }
}

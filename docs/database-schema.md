# SQLite 数据库设计

## skills

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| name | TEXT | 技能名称 |
| path | TEXT | skill 文件夹路径，唯一 |
| description | TEXT | 描述 |
| content | TEXT | `SKILL.md` 正文 |
| name_zh | TEXT | 中文名称 |
| name_en | TEXT | 英文名称 |
| description_zh | TEXT | 中文描述 |
| description_en | TEXT | 英文描述 |
| summary_zh | TEXT | 中文摘要 |
| summary_en | TEXT | 英文摘要 |
| category_id | INTEGER | 分类 ID |
| source | TEXT | 来源 |
| platform | TEXT | 平台 |
| scope | TEXT | 作用域 |
| project_path | TEXT | 项目路径 |
| is_custom | INTEGER | 是否自建 |
| status | TEXT | 状态 |
| quality_score | INTEGER | 质量评分 |
| quality_reason | TEXT | 评分原因 |
| usage_count | INTEGER | 使用次数 |
| duplicate_score | REAL | 重复风险分 |
| hash | TEXT | 内容 hash |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |
| last_scanned_at | TEXT | 上次扫描时间 |
| last_used_at | TEXT | 上次使用时间 |

## categories

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| name | TEXT | 分类名称 |
| color | TEXT | 分类颜色 |
| parent_id | INTEGER | 父分类 |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |

## tags

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| name | TEXT | 标签名称 |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |

## skill_tags

| 字段 | 类型 | 说明 |
|---|---|---|
| skill_id | INTEGER | skill ID |
| tag_id | INTEGER | tag ID |

## scan_roots

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| path | TEXT | 根目录路径 |
| enabled | INTEGER | 是否启用 |
| platform | TEXT | 平台 |
| created_at | TEXT | 创建时间 |
| last_scanned_at | TEXT | 上次扫描时间 |

## tools

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| tool_name | TEXT | 工具 ID |
| display_name | TEXT | 显示名称 |
| skill_dir | TEXT | 工具 Skill 目录 |
| detected | INTEGER | 是否检测到 |
| enabled | INTEGER | 是否启用 |
| sync_enabled | INTEGER | 是否参与同步 |
| is_custom | INTEGER | 是否自定义工具 |
| link_mode | TEXT | 链接策略 |
| last_checked_at | TEXT | 上次检测时间 |

## repositories

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| name | TEXT | 仓库名称 |
| path | TEXT | 本地仓库路径 |
| type | TEXT | 仓库类型 |
| enabled | INTEGER | 是否启用 |
| is_primary | INTEGER | 是否主仓库 |
| last_scanned_at | TEXT | 上次扫描时间 |

## skill_tool_links

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| skill_id | INTEGER | Skill ID |
| tool_name | TEXT | 工具 ID |
| enabled | INTEGER | 是否启用到该工具 |
| link_path | TEXT | 软链接路径 |
| link_mode | TEXT | 同步产物策略 |
| link_status | TEXT | 同步状态 |
| last_synced_at | TEXT | 上次同步时间 |
| error_message | TEXT | 错误信息 |

## sync_issues

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| skill_id | INTEGER | Skill ID |
| tool_name | TEXT | 工具 ID |
| issue_type | TEXT | 异常类型 |
| current_path | TEXT | 当前路径 |
| expected_path | TEXT | 期望路径 |
| severity | TEXT | 严重程度 |
| fixable | INTEGER | 是否可自动修复 |
| status | TEXT | 处理状态 |
| message | TEXT | 说明 |

## app_settings

| 字段 | 类型 | 说明 |
|---|---|---|
| key | TEXT | 设置键 |
| value | TEXT | 设置值 |
| updated_at | TEXT | 更新时间 |

用于保存本地 AI 翻译配置和本地账号状态。API Key 仅保存在本机数据库，不应提交到代码仓库。

## marketplace_sources

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| name | TEXT | 源名称 |
| url | TEXT | JSON 源 URL |
| enabled | INTEGER | 是否启用 |
| last_refreshed_at | TEXT | 上次刷新时间 |

## marketplace_items

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| source_id | INTEGER | 来源 ID |
| external_id | TEXT | 源内唯一 ID |
| name | TEXT | Skill 名称 |
| description | TEXT | 描述 |
| version | TEXT | 版本 |
| author | TEXT | 作者 |
| category | TEXT | 分类 |
| tags | TEXT | 标签，逗号分隔 |
| skill_url | TEXT | 远程 SKILL.md 地址 |
| homepage | TEXT | 主页 |
| installed_skill_id | INTEGER | 已安装 Skill ID |
| installed_version | TEXT | 已安装版本 |
| installed_hash | TEXT | 已安装内容 hash |
| installed_at | TEXT | 安装时间 |
| last_install_check_at | TEXT | 上次核对时间 |
| install_status | TEXT | 安装状态 |
| install_message | TEXT | 安装状态说明 |

## duplicate_groups

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| title | TEXT | 重复组标题 |
| reason | TEXT | 重复原因 |
| score | REAL | 相似度 |
| status | TEXT | 处理状态 |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |

## duplicate_group_items

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| group_id | INTEGER | 重复组 ID |
| skill_id | INTEGER | skill ID |
| recommendation | TEXT | 建议 |
| reason | TEXT | 原因 |

## scan_history

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| root_path | TEXT | 扫描根目录 |
| total_found | INTEGER | 找到数量 |
| new_count | INTEGER | 新增数量 |
| changed_count | INTEGER | 变更数量 |
| duplicate_count | INTEGER | 重复数量 |
| scanned_at | TEXT | 扫描时间 |

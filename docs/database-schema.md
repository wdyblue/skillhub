# SQLite 数据库设计

## skills

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER | 主键 |
| name | TEXT | 技能名称 |
| path | TEXT | skill 文件夹路径，唯一 |
| description | TEXT | 描述 |
| content | TEXT | `SKILL.md` 正文 |
| category_id | INTEGER | 分类 ID |
| source | TEXT | 来源 |
| platform | TEXT | 平台 |
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


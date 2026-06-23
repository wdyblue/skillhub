# SkillHub 技能管理器

SkillHub 是一个中文版、本地优先、卡片式的 AI Skill 资产管理桌面 App。当前版本聚焦本地目录扫描、`SKILL.md` 入库、卡片浏览、质量评分、归档管理、多工具同步和 AI 翻译。

## 技术栈

- Tauri 2
- React
- TypeScript
- Rust
- SQLite
- Tailwind CSS
- Monaco Editor

## 当前功能

- 中文桌面 App 基础布局
- 左侧导航：首页、全部技能、重复检测、技能体检、归档箱、设置等入口
- 添加 / 移除 / 启用 / 禁用本地 skill 根目录
- 递归扫描包含 `SKILL.md` 的文件夹
- 读取 skill 标题、描述、正文、路径、修改信息
- 计算内容 SHA-256 hash
- SQLite 本地存储
- 内置中文分类初始化
- 技能卡片列表
- 搜索、分类、状态、来源筛选
- 按修改时间、质量评分、使用次数、名称排序
- Skill 详情页基础信息
- Monaco 预览 `SKILL.md`
- Monaco 编辑并保存 `SKILL.md`
- 复制内容、打开文件夹、打开 `SKILL.md`
- 质量评分和评分原因
- 数据库状态归档，不删除、不移动原文件
- 疑似重复基础标记：hash 完全重复、名称版本噪声
- 统一 Skill 主仓库：新建和导入自建 Skill
- 工具目录检测：Codex、Claude Code、CodeBuddy、Hermes、Cursor、Opencode、Gemini CLI、Qwen Code、Cline
- 通过软链接、Junction 或复制把 Skill 启用到不同 AI 工具目录
- Skill 作用域管理：Global / Project
- Skill 到工具目录分配矩阵，支持批量勾选
- 同步体检与可修复异常处理
- 一键同步全部：把所有非归档 Skill 同步到所有启用工具目录
- Marketplace 源管理：添加 JSON 源、刷新列表、安装 / 更新 / 卸载 Skill、核对安装状态
- 云同步包：导出 / 导入本地 JSON 同步包，用于多设备迁移
- 设置页配置 OpenAI 兼容 AI 翻译接口
- 技能卡片一键翻译名称、描述和摘要
- 本地账号登录状态，为后续 Marketplace 预留入口

## AI 翻译配置

打开“设置 → AI 翻译”，填写：

- Base URL：OpenAI 兼容接口地址，例如 `https://api.openai.com/v1`
- API Key：服务商 API Key，只保存在本机 SQLite
- 模型：如 `gpt-4o-mini`、`deepseek-chat`

配置后可点击“测试连接”。在“全部技能”卡片右上角点击翻译按钮，即可翻译当前 Skill 的名称、描述和摘要。

注意：

- 当前使用 OpenAI 兼容 `/chat/completions` 接口。
- API Key 不会写入代码仓库。
- 翻译结果写入本地数据库，不会自动改写原始 `SKILL.md` 文件。
- 中国大陆网络环境下，如遇 401，多数是 API Key 或模型名问题；如遇 502/SSL，优先检查本地代理。

## Marketplace 源格式

打开“远程仓库”，添加一个 HTTPS JSON 源。源内容支持以下格式：

```json
{
  "skills": [
    {
      "id": "accessibility",
      "name": "accessibility",
      "description": "Build WCAG compliant frontend experiences.",
      "version": "1.0.0",
      "author": "SkillHub",
      "category": "前端开发",
      "tags": ["frontend", "accessibility"],
      "skill_url": "https://example.com/accessibility/SKILL.md",
      "homepage": "https://example.com/accessibility"
    }
  ]
}
```

也支持直接返回数组。安装时只下载 `skill_url` 指向的 `SKILL.md`，写入统一主仓库，不会删除或移动已有本地 Skill。Marketplace 条目支持更新、卸载和安装态复核；卸载只会解除关联并把本地 Skill 标记为已归档，不会删除文件。

## 云同步包

“远程仓库”页提供同步包导出/导入：

- 导出：把当前 Skill 内容和工具配置导出为 JSON 文件。
- 导入：把同步包里的 Skill 写入当前统一主仓库。
- 安全边界：导入不会删除已有 Skill；AI API Key 不会导出。

## 安装依赖

前置要求：

- Node.js 18+
- Rust stable
- macOS 需要 Xcode Command Line Tools

安装：

```bash
npm install
```

如果你在中国大陆网络环境遇到 npm SSL 或 502，先检查本地代理环境变量：

```bash
env | grep -i proxy
```

必要时临时取消代理后重试：

```bash
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy
npm install
```

## 启动开发环境

Web 调试：

```bash
npm run dev
```

桌面 App 开发：

```bash
npm run tauri:dev
```

## 构建桌面 App

```bash
npm run tauri:build
```

构建产物位于：

```txt
src-tauri/target/release/bundle/
```

## 使用方法

1. 打开“设置”。
2. 添加一个或多个本地 skill 根目录。
3. 点击“扫描全部”或顶部“重新扫描”。
4. 在“全部技能”查看卡片列表。
5. 点击卡片进入详情页。
6. 根据评分、分类、状态进行整理。
7. 对不再常用的 skill 使用“归档”，不会删除原文件。
8. 在“工具目录”检测 AI 工具目录。
9. 在详情页启用/禁用某个 Skill 到指定工具。
10. 在“工具矩阵”页批量勾选 Skill 和工具。
11. 在“设置”配置 AI 翻译后，在技能卡片右上角点击翻译按钮。

## 数据位置

SQLite 数据库保存在 Tauri 应用数据目录中，文件名：

```txt
skillhub.sqlite3
```

macOS 通常位于：

```txt
~/Library/Application Support/com.wdyblue.skillhub/skillhub.sqlite3
```

如果你要从当前电脑迁移到另一台 iMac 继续开发，可以直接参考：

- [docs/imac-migration.md](/Users/admin/Documents/skillhub/docs/imac-migration.md)

## 第一版限制

- 重复检测页目前是占位页，数据库已有 `duplicate_score` 字段和基础重复风险计算。
- 标签系统数据库已预留，前端手动编辑标签还未完成。
- GitHub 更新能力只预留文档和数据结构，暂未接入远程仓库。
- 在线 Marketplace 的评分、评论、审核和付费能力尚未实现。
- AI 翻译已接入，embedding 语义相似度后续再做。

## 开发原则

- 只管理，不破坏。
- 只归档，不删除。
- 数据优先保存在本地。
- 危险操作需要二次确认。
- 第一版不强依赖联网。
- 界面保持中文。

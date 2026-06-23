# SkillHub 技能管理器

SkillHub 是一个中文版、本地优先、卡片式的 AI Skill 资产管理桌面 App。第一版聚焦本地目录扫描、`SKILL.md` 入库、卡片浏览、质量评分和归档管理。

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
- 复制内容、打开文件夹、打开 `SKILL.md`
- 质量评分和评分原因
- 数据库状态归档，不删除、不移动原文件
- 疑似重复基础标记：hash 完全重复、名称版本噪声

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

## 数据位置

SQLite 数据库保存在 Tauri 应用数据目录中，文件名：

```txt
skillhub.sqlite3
```

macOS 通常位于：

```txt
~/Library/Application Support/com.wdyblue.skillhub/skillhub.sqlite3
```

## 第一版限制

- 重复检测页目前是占位页，数据库已有 `duplicate_score` 字段和基础重复风险计算。
- 标签系统数据库已预留，前端手动编辑标签还未完成。
- 分类管理页入口已预留，新增 / 编辑 / 删除分类尚未完成。
- GitHub 更新能力只预留文档和数据结构，暂未接入远程仓库。
- 不依赖 AI API；自动摘要、语义相似度和 embedding 后续再做。

## 开发原则

- 只管理，不破坏。
- 只归档，不删除。
- 数据优先保存在本地。
- 危险操作需要二次确认。
- 第一版不强依赖联网。
- 界面保持中文。


# SkillHub iMac 迁移说明

本文用于把当前这台机器上的 SkillHub 开发状态迁移到另一台 iMac，并在 iMac 上继续开发。

## 当前迁移基线

- 当前仓库路径：`/Users/admin/Documents/skillhub`
- 当前分支：`main`
- 当前迁移提交：`213083b`
- 当前提交说明：`wip: checkpoint remote sync and marketplace work`

## 第一步：把代码推到 GitHub

如果这个仓库还没有远端，先在当前机器执行：

```bash
cd /Users/admin/Documents/skillhub
git remote add origin <你的 GitHub 仓库地址>
git push -u origin main
```

如果已经登录 GitHub CLI，也可以用：

```bash
cd /Users/admin/Documents/skillhub
gh repo create skillhub --private --source=. --remote=origin --push
```

说明：

- 仓库当前没有默认远端。
- `node_modules/`、`dist/`、`src-tauri/target/`、`.env*` 不会进 Git。

## 第二步：在 iMac 上拉代码

在 iMac 上执行：

```bash
git clone <你的 GitHub 仓库地址>
cd skillhub
npm ci
cd src-tauri
cargo check
cd ..
npm run tauri:dev
```

前置要求：

- Node.js 18+
- Rust stable
- Xcode Command Line Tools

## 第三步：补本地数据和配置

代码仓库之外，真正需要注意的是本地 SQLite 和工具目录状态。

### 需要迁移的本地数据

SkillHub 的本地数据库默认在：

```txt
~/Library/Application Support/com.wdyblue.skillhub/skillhub.sqlite3
```

这个库里包含：

- AI 翻译配置
- API Key
- 扫描根目录
- 工具目录启用状态
- Marketplace 安装状态
- 本地账号状态
- 技能翻译结果

如果你希望 iMac 延续当前状态，可以把这个文件一并带过去。

## 推荐迁移策略

### 方案 A：只迁代码，不迁本地状态

适合重新配置一遍环境。

优点：

- 最干净
- 不会把旧机器上的绝对路径带到 iMac

代价：

- 需要在 iMac 重新填 AI 配置
- 需要重新添加扫描目录
- 需要重新检测工具目录

### 方案 B：代码 + SQLite 一起迁

适合希望保留当前 App 内状态。

优点：

- 配置和翻译结果都能延续

注意：

- 扫描根目录、项目路径、工具目录里可能包含旧机器绝对路径
- 拷过去后，第一次打开要逐项检查路径是否仍然有效

## iMac 首次启动排错清单

1. 先跑 `npm ci`，不要直接复制当前机器的 `node_modules/`。
2. 跑 `cargo check`，确认 Rust 依赖能在 iMac 本地完整解析。
3. 如果 `npm install` 或 `npm ci` 慢或报 SSL/502，先检查代理：

```bash
env | grep -i proxy
```

必要时临时取消：

```bash
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy
```

4. 如果打开的是浏览器页面而不是桌面壳，`@tauri-apps` 能力会失效；开发时要用：

```bash
npm run tauri:dev
```

5. 如果 App 能打开但配置不对，优先检查：

- 扫描根目录是否还是旧机器路径
- 工具目录是否存在
- AI Base URL、模型名、API Key 是否可用

6. 如果 AI 翻译报错：

- `401`：先查 API Key 或模型名
- `502` / SSL：先查本地代理
- 模型列表为空：多半是权限或服务商接口不兼容

7. 如果你复制了 SQLite，但界面数据异常，先备份原库，再删除 iMac 上的 `skillhub.sqlite3` 让应用自动重建。

## 建议的迁移顺序

1. 当前机器提交代码并推到 GitHub
2. iMac `clone` 并启动 `npm run tauri:dev`
3. 确认代码能跑
4. 再决定是否复制 `skillhub.sqlite3`
5. 最后检查工具目录和扫描目录

## Codex 补充方案

如果你的 iMac 已经接入 Codex 远程主机，也可以后续把当前 Codex 线程 handoff 过去继续做，不必只靠 GitHub 同步代码。

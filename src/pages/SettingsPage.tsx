import { isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Eye, EyeOff, FolderPlus, LogIn, LogOut, ScanLine, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { Account, AiConfig, ScanRoot } from "../lib/tauri";

type Props = {
  scanRoots: ScanRoot[];
  onAddRoot: (path: string, platform: string) => Promise<void>;
  onRemoveRoot: (id: number) => Promise<void>;
  onToggleRoot: (id: number, enabled: boolean) => Promise<void>;
  onScan: () => void;
  aiConfig: AiConfig | null;
  account: Account | null;
  onSaveAiConfig: (input: { baseUrl: string; apiKey?: string; model: string }) => Promise<void>;
  onRevealApiKey: () => Promise<AiConfig | null>;
  onFetchAiModels: (input: { baseUrl: string; apiKey?: string; model: string }) => Promise<string[]>;
  onTestAiConnection: () => Promise<string>;
  onClearTranslationCache: () => Promise<void>;
  onLogin: (input: { name: string; email?: string }) => Promise<void>;
  onLogout: () => Promise<void>;
};

const platforms = ["Codex", "ChatGPT", "Claude", "Hermes", "Cursor", "自建", "未知"];

export function SettingsPage({
  scanRoots,
  onAddRoot,
  onRemoveRoot,
  onToggleRoot,
  onScan,
  aiConfig,
  account,
  onSaveAiConfig,
  onRevealApiKey,
  onFetchAiModels,
  onTestAiConnection,
  onClearTranslationCache,
  onLogin,
  onLogout
}: Props) {
  const [path, setPath] = useState("");
  const [platform, setPlatform] = useState("未知");
  const [dialogError, setDialogError] = useState<string | null>(null);
  const [baseUrl, setBaseUrl] = useState("https://api.openai.com/v1");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("gpt-4o-mini");
  const [showApiKey, setShowApiKey] = useState(false);
  const [aiMessage, setAiMessage] = useState<string | null>(null);
  const [aiBusy, setAiBusy] = useState(false);
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [loginName, setLoginName] = useState("TONYWU");
  const [loginEmail, setLoginEmail] = useState("");

  useEffect(() => {
    if (!aiConfig) return;
    setBaseUrl(aiConfig.baseUrl);
    setModel(aiConfig.model);
    setApiKey(aiConfig.hasApiKey ? "••••••••" : "");
  }, [aiConfig]);

  useEffect(() => {
    if (!account) return;
    setLoginName(account.name);
    setLoginEmail(account.email);
  }, [account]);

  async function chooseDirectory() {
    setDialogError(null);

    if (!isTauri()) {
      setDialogError("当前不在 Tauri 桌面端中运行，无法打开系统目录选择器。");
      return;
    }

    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === "string") {
        setPath(selected);
      }
    } catch (error) {
      setDialogError(
        error instanceof Error ? error.message : "打开目录选择器失败，请检查 Tauri 权限配置。"
      );
    }
  }

  async function submit() {
    if (!path.trim()) return;
    await onAddRoot(path.trim(), platform);
    setPath("");
    setPlatform("未知");
  }

  async function saveAi() {
    setAiBusy(true);
    setAiMessage(null);
    try {
      await onSaveAiConfig({
        baseUrl,
        model,
        apiKey: apiKey === "••••••••" ? undefined : apiKey
      });
      if (apiKey && apiKey !== "••••••••") {
        setApiKey("••••••••");
      }
      setShowApiKey(false);
      setAiMessage("AI 翻译配置已保存。");
    } catch (error) {
      setAiMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setAiBusy(false);
    }
  }

  async function revealApiKey() {
    if (showApiKey) {
      setShowApiKey(false);
      setApiKey(aiConfig?.hasApiKey ? "••••••••" : "");
      return;
    }
    setAiBusy(true);
    setAiMessage(null);
    try {
      const config = await onRevealApiKey();
      if (config) {
        setApiKey(config.apiKey);
        setShowApiKey(true);
      }
    } catch (error) {
      setAiMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setAiBusy(false);
    }
  }

  async function testConnection() {
    setAiBusy(true);
    setAiMessage(null);
    try {
      const result = await onTestAiConnection();
      setAiMessage(result);
    } catch (error) {
      setAiMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setAiBusy(false);
    }
  }

  async function fetchModels() {
    setAiBusy(true);
    setAiMessage(null);
    try {
      const models = await onFetchAiModels({
        baseUrl,
        model,
        apiKey: apiKey === "••••••••" ? undefined : apiKey
      });
      setAvailableModels(models);
      if (models.length > 0 && !models.includes(model)) {
        setModel(models[0]);
      }
      setApiKey("••••••••");
      setShowApiKey(false);
      setAiMessage(`已获取 ${models.length} 个可用模型。`);
    } catch (error) {
      setAiMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setAiBusy(false);
    }
  }

  async function clearCache() {
    setAiBusy(true);
    setAiMessage(null);
    try {
      await onClearTranslationCache();
      setAiMessage("翻译缓存已清空。");
    } catch (error) {
      setAiMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setAiBusy(false);
    }
  }

  async function login() {
    if (!loginName.trim()) return;
    await onLogin({ name: loginName.trim(), email: loginEmail.trim() || undefined });
  }

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">AI 翻译</h3>
        <p className="mt-2 text-sm text-slate-500">
          配置 OpenAI 兼容的大模型，用于翻译 Skill 名称、描述和摘要。API Key 只保存在本机 SQLite。
        </p>
        <div className="mt-5 space-y-4">
          <label className="block space-y-2">
            <span className="text-sm font-semibold text-slate-700">Base URL</span>
            <input
              value={baseUrl}
              onChange={(event) => setBaseUrl(event.target.value)}
              className="filter-control"
              placeholder="https://api.openai.com/v1"
            />
            <span className="text-xs text-slate-500">OpenAI 兼容接口地址，例如 OpenAI / DeepSeek / ModelScope。</span>
          </label>
          <label className="block space-y-2">
            <span className="text-sm font-semibold text-slate-700">API Key</span>
            <div className="grid grid-cols-[1fr_auto] gap-3">
              <input
                type={showApiKey ? "text" : "password"}
                value={apiKey}
                onChange={(event) => setApiKey(event.target.value)}
                className="filter-control"
                placeholder="sk-..."
              />
              <button type="button" className="action-button" onClick={revealApiKey} disabled={aiBusy}>
                {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                {showApiKey ? "隐藏" : "显示"}
              </button>
            </div>
          </label>
          <label className="block space-y-2">
            <span className="text-sm font-semibold text-slate-700">模型</span>
            <div className="grid grid-cols-[1fr_auto] gap-3">
              <input
                value={model}
                onChange={(event) => setModel(event.target.value)}
                className="filter-control"
                placeholder="gpt-4o-mini"
                list="skillhub-ai-models"
              />
              <button type="button" className="action-button" onClick={fetchModels} disabled={aiBusy}>
                获取模型
              </button>
            </div>
            <datalist id="skillhub-ai-models">
              {availableModels.map((item) => (
                <option key={item} value={item} />
              ))}
            </datalist>
            <span className="text-xs text-slate-500">如 gpt-4o-mini、deepseek-chat、agnes-2.0-flash。模型名称大小写按服务商要求填写。</span>
          </label>
          <div className="flex flex-wrap items-center gap-3">
            <button type="button" className="action-button" onClick={testConnection} disabled={aiBusy}>
              测试连接
            </button>
            <button
              type="button"
              onClick={saveAi}
              disabled={aiBusy}
              className="rounded-2xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-brand-700 active:scale-[0.98] disabled:opacity-60"
            >
              保存配置
            </button>
            <button type="button" className="text-sm font-medium text-slate-500 hover:text-red-600" onClick={clearCache} disabled={aiBusy}>
              清空翻译缓存
            </button>
            <span className="ml-auto text-sm text-slate-500">
              还没有 API？ <a className="font-semibold text-brand-600 underline-offset-4 hover:underline" href="https://platform.openai.com/api-keys" target="_blank" rel="noreferrer">试试 OpenAI API →</a>
            </span>
          </div>
          {aiMessage ? <p className="text-sm text-slate-600">{aiMessage}</p> : null}
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">账号</h3>
        <p className="mt-2 text-sm text-slate-500">登录以使用 Marketplace 等后续功能。当前版本先保存本地账号状态。</p>
        <div className="mt-5 flex items-center justify-between gap-5">
          <div className="grid flex-1 grid-cols-2 gap-3">
            <input
              value={loginName}
              onChange={(event) => setLoginName(event.target.value)}
              className="filter-control"
              placeholder="账号名称"
            />
            <input
              value={loginEmail}
              onChange={(event) => setLoginEmail(event.target.value)}
              className="filter-control"
              placeholder="邮箱，可选"
            />
          </div>
          {account?.loggedIn ? (
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-3 rounded-2xl border border-slate-200 px-4 py-3">
                <span className="grid h-10 w-10 place-items-center rounded-full bg-purple-600 text-sm font-bold text-white">
                  {account.avatarInitial}
                </span>
                <div>
                  <p className="font-semibold text-slate-900">{account.name}</p>
                  {account.email ? <p className="text-xs text-slate-500">{account.email}</p> : null}
                </div>
              </div>
              <button type="button" className="action-button" onClick={onLogout}>
                <LogOut className="h-4 w-4" />
                退出
              </button>
            </div>
          ) : (
            <button type="button" className="action-button" onClick={login}>
              <LogIn className="h-4 w-4" />
              登录
            </button>
          )}
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">添加本地 Skill 根目录</h3>
        <p className="mt-2 text-sm text-slate-500">
          扫描会递归查找包含 SKILL.md 的文件夹。不会删除、移动或修改原始 skill 文件。
        </p>
        <div className="mt-5 grid grid-cols-[1fr_160px_auto_auto] gap-3">
          <input
            value={path}
            onChange={(event) => setPath(event.target.value)}
            placeholder="例如：/Users/admin/.codex/skills"
            className="filter-control"
          />
          <select
            value={platform}
            onChange={(event) => setPlatform(event.target.value)}
            className="filter-control"
          >
            {platforms.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={chooseDirectory}
            className="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-semibold text-slate-700 transition hover:bg-slate-50 active:scale-[0.98]"
          >
            <FolderPlus className="h-4 w-4" />
            选择目录
          </button>
          <button
            type="button"
            onClick={submit}
            className="rounded-2xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-brand-700 active:scale-[0.98]"
          >
            添加
          </button>
        </div>
        {dialogError ? (
          <p className="mt-3 text-sm text-amber-700">{dialogError}</p>
        ) : null}
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-slate-950">扫描目录</h3>
            <p className="mt-2 text-sm text-slate-500">
              可启用 / 禁用目录。删除目录只移除配置，不会删除数据库里的 skill 记录。
            </p>
          </div>
          <button
            type="button"
            onClick={onScan}
            className="inline-flex items-center gap-2 rounded-2xl bg-slate-900 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-slate-800 active:scale-[0.98]"
          >
            <ScanLine className="h-4 w-4" />
            扫描全部
          </button>
        </div>

        <div className="mt-5 overflow-hidden rounded-3xl border border-slate-200">
          {scanRoots.length === 0 ? (
            <div className="px-5 py-12 text-center text-sm text-slate-500">
              暂无扫描目录。
            </div>
          ) : (
            <table className="w-full text-left text-sm">
              <thead className="bg-slate-50 text-xs text-slate-500">
                <tr>
                  <th className="px-4 py-3">启用</th>
                  <th className="px-4 py-3">路径</th>
                  <th className="px-4 py-3">平台</th>
                  <th className="px-4 py-3">上次扫描</th>
                  <th className="px-4 py-3 text-right">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {scanRoots.map((root) => (
                  <tr key={root.id}>
                    <td className="px-4 py-3">
                      <input
                        type="checkbox"
                        checked={root.enabled}
                        onChange={(event) =>
                          void onToggleRoot(root.id, event.target.checked)
                        }
                      />
                    </td>
                    <td className="max-w-xl truncate px-4 py-3 font-medium text-slate-800">
                      {root.path}
                    </td>
                    <td className="px-4 py-3 text-slate-600">{root.platform}</td>
                    <td className="px-4 py-3 text-slate-600">
                      {root.last_scanned_at ?? "尚未扫描"}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <button
                        type="button"
                        className="inline-flex items-center gap-1 rounded-xl px-2.5 py-1.5 text-xs font-semibold text-red-600 transition hover:bg-red-50"
                        onClick={() => void onRemoveRoot(root.id)}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                        移除
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </section>
    </div>
  );
}

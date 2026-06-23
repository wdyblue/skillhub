import { open, save } from "@tauri-apps/plugin-dialog";
import { CloudDownload, CloudUpload, Download, RefreshCw, RotateCcw, Trash2 } from "lucide-react";
import { useState } from "react";
import { MarketplaceItem, MarketplaceSource } from "../lib/tauri";

type Props = {
  sources: MarketplaceSource[];
  items: MarketplaceItem[];
  onAddSource: (input: { name: string; url: string }) => Promise<void>;
  onDeleteSource: (id: number) => Promise<void>;
  onRefreshSource: (id: number) => Promise<void>;
  onInstallItem: (id: number) => Promise<void>;
  onUpdateItem: (id: number) => Promise<void>;
  onUninstallItem: (id: number) => Promise<void>;
  onRecheckInstallations: () => Promise<void>;
  onExportSyncPackage: (path: string) => Promise<void>;
  onImportSyncPackage: (path: string) => Promise<void>;
};

export function RemotePage({
  sources,
  items,
  onAddSource,
  onDeleteSource,
  onRefreshSource,
  onInstallItem,
  onUpdateItem,
  onUninstallItem,
  onRecheckInstallations,
  onExportSyncPackage,
  onImportSyncPackage
}: Props) {
  const [name, setName] = useState("SkillHub Marketplace");
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function run(label: string, action: () => Promise<void>) {
    setBusy(label);
    setMessage(null);
    try {
      await action();
      setMessage(`${label}完成。`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(null);
    }
  }

  async function addSource() {
    if (!name.trim() || !url.trim()) return;
    await run("添加源", async () => {
      await onAddSource({ name: name.trim(), url: url.trim() });
      setUrl("");
    });
  }

  async function exportPackage() {
    const selected = await save({
      defaultPath: "skillhub-sync-package.json",
      filters: [{ name: "SkillHub Sync Package", extensions: ["json"] }]
    });
    if (typeof selected === "string") {
      await run("导出同步包", () => onExportSyncPackage(selected));
    }
  }

  async function importPackage() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "SkillHub Sync Package", extensions: ["json"] }]
    });
    if (typeof selected === "string") {
      const ok = window.confirm("导入会把同步包里的 Skill 写入当前统一主仓库，不会删除现有 Skill。是否继续？");
      if (ok) await run("导入同步包", () => onImportSyncPackage(selected));
    }
  }

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-slate-950">Marketplace</h3>
            <p className="mt-2 text-sm text-slate-500">
              添加 JSON 源，刷新后可把远程 Skill 安装到统一主仓库。源格式支持 JSON 数组或 {"{"}"skills": [...]{"}"}。
            </p>
          </div>
          <div className="flex gap-2">
            <button className="action-button" onClick={exportPackage} disabled={Boolean(busy)}>
              <CloudUpload className="h-4 w-4" />
              导出同步包
            </button>
            <button className="action-button" onClick={importPackage} disabled={Boolean(busy)}>
              <CloudDownload className="h-4 w-4" />
              导入同步包
            </button>
            <button className="action-button" onClick={() => void run("重新核对安装状态", onRecheckInstallations)} disabled={Boolean(busy)}>
              <RotateCcw className="h-4 w-4" />
              重新核对
            </button>
          </div>
        </div>

        <div className="mt-5 grid grid-cols-[220px_1fr_auto] gap-3">
          <input className="filter-control" value={name} onChange={(event) => setName(event.target.value)} placeholder="源名称" />
          <input className="filter-control" value={url} onChange={(event) => setUrl(event.target.value)} placeholder="https://.../marketplace.json" />
          <button className="action-button" onClick={() => void addSource()} disabled={Boolean(busy)}>
            添加源
          </button>
        </div>
        {message ? <p className="mt-3 text-sm text-slate-600">{message}</p> : null}
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">Marketplace 源</h3>
        <div className="mt-4 overflow-hidden rounded-3xl border border-slate-200">
          {sources.length === 0 ? (
            <div className="px-5 py-10 text-center text-sm text-slate-500">暂无源。添加一个 JSON 源后刷新。</div>
          ) : (
            <table className="w-full text-left text-sm">
              <thead className="bg-slate-50 text-xs text-slate-500">
                <tr>
                  <th className="px-4 py-3">名称</th>
                  <th className="px-4 py-3">URL</th>
                  <th className="px-4 py-3">上次刷新</th>
                  <th className="px-4 py-3 text-right">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {sources.map((source) => (
                  <tr key={source.id}>
                    <td className="px-4 py-3 font-semibold text-slate-800">{source.name}</td>
                    <td className="max-w-xl truncate px-4 py-3 text-slate-500">{source.url}</td>
                    <td className="px-4 py-3 text-slate-500">{source.lastRefreshedAt ?? "尚未刷新"}</td>
                    <td className="px-4 py-3 text-right">
                      <div className="flex justify-end gap-2">
                        <button className="action-button" onClick={() => void run("刷新源", () => onRefreshSource(source.id))} disabled={Boolean(busy)}>
                          <RefreshCw className="h-4 w-4" />
                          刷新
                        </button>
                        <button className="action-button text-red-600" onClick={() => void run("删除源", () => onDeleteSource(source.id))} disabled={Boolean(busy)}>
                          <Trash2 className="h-4 w-4" />
                          删除
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">可安装 Skill</h3>
        <div className="mt-4 grid grid-cols-3 gap-4">
          {items.length === 0 ? (
            <div className="col-span-3 rounded-3xl border border-dashed border-slate-300 px-5 py-12 text-center text-sm text-slate-500">
              暂无 Marketplace 条目。请先刷新源。
            </div>
          ) : (
            items.map((item) => (
              <div key={item.id} className="flex min-h-52 flex-col rounded-3xl border border-slate-200 bg-white p-5 shadow-sm">
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <h4 className="line-clamp-2 text-lg font-semibold text-slate-950">{item.name}</h4>
                    <p className="mt-2 line-clamp-3 text-sm leading-6 text-slate-600">{item.description || "暂无描述"}</p>
                  </div>
                  <div className="flex flex-col items-end gap-2">
                    <span
                      className={
                        item.installStatus === "已安装"
                          ? "rounded-full bg-emerald-50 px-2.5 py-1 text-xs font-semibold text-emerald-600"
                          : item.installStatus === "可更新"
                            ? "rounded-full bg-amber-50 px-2.5 py-1 text-xs font-semibold text-amber-700"
                            : "rounded-full bg-slate-100 px-2.5 py-1 text-xs font-semibold text-slate-600"
                      }
                    >
                      {item.installStatus}
                    </span>
                    {item.isUpdateAvailable ? <span className="text-xs font-medium text-amber-700">有新版本</span> : null}
                  </div>
                </div>
                <div className="mt-4 flex flex-wrap gap-2">
                  {item.version ? <Badge>{item.version}</Badge> : null}
                  {item.author ? <Badge>{item.author}</Badge> : null}
                  {item.category ? <Badge>{item.category}</Badge> : null}
                  {item.tags.slice(0, 3).map((tag) => <Badge key={tag}>{tag}</Badge>)}
                </div>
                <div className="mt-4 space-y-2 rounded-2xl bg-slate-50 px-3 py-2 text-xs text-slate-500">
                  <p>已安装版本：{item.installedVersion || "未安装"}</p>
                  <p>已安装路径：{item.installedSkillPath || "未安装"}</p>
                  {item.installMessage ? <p className="text-slate-600">{item.installMessage}</p> : null}
                </div>
                <div className="mt-auto flex flex-col gap-2">
                  <button
                    className="action-button justify-center"
                    onClick={() => void run(item.installedSkillId ? "更新 Skill" : "安装 Skill", () => item.installedSkillId ? onUpdateItem(item.id) : onInstallItem(item.id))}
                    disabled={Boolean(busy)}
                  >
                    <Download className="h-4 w-4" />
                    {item.installedSkillId ? (item.isUpdateAvailable ? "更新到最新" : "重新安装") : "安装到主仓库"}
                  </button>
                  <button
                    className="action-button justify-center text-red-600"
                    onClick={() =>
                      {
                        if (window.confirm("卸载只会解除 Marketplace 关联并将 Skill 标记为已归档，不会删除文件。是否继续？")) {
                          void run("卸载 Skill", () => onUninstallItem(item.id));
                        }
                      }
                    }
                    disabled={Boolean(busy) || !item.installedSkillId}
                  >
                    <Trash2 className="h-4 w-4" />
                    卸载
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </section>
    </div>
  );
}

function Badge({ children }: { children: React.ReactNode }) {
  return <span className="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600">{children}</span>;
}

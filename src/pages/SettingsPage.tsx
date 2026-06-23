import { isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderPlus, ScanLine, Trash2 } from "lucide-react";
import { useState } from "react";
import { ScanRoot } from "../lib/tauri";

type Props = {
  scanRoots: ScanRoot[];
  onAddRoot: (path: string, platform: string) => Promise<void>;
  onRemoveRoot: (id: number) => Promise<void>;
  onToggleRoot: (id: number, enabled: boolean) => Promise<void>;
  onScan: () => void;
};

const platforms = ["Codex", "ChatGPT", "Claude", "Hermes", "Cursor", "自建", "未知"];

export function SettingsPage({
  scanRoots,
  onAddRoot,
  onRemoveRoot,
  onToggleRoot,
  onScan
}: Props) {
  const [path, setPath] = useState("");
  const [platform, setPlatform] = useState("未知");
  const [dialogError, setDialogError] = useState<string | null>(null);

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

  return (
    <div className="space-y-5">
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

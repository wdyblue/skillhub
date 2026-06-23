import { isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderPlus, Plus, RefreshCw, Save, Trash2 } from "lucide-react";
import { useState } from "react";
import { ToolConfig } from "../lib/tauri";

type Props = {
  tools: ToolConfig[];
  onDetect: () => Promise<void>;
  onSave: (tool: ToolConfig) => Promise<void>;
  onCreateCustomTool: (request: { toolName: string; displayName: string; skillDir: string }) => Promise<void>;
  onDeleteCustomTool: (toolName: string) => Promise<void>;
};

export function ToolDirectoriesPage({
  tools,
  onDetect,
  onSave,
  onCreateCustomTool,
  onDeleteCustomTool
}: Props) {
  const [drafts, setDrafts] = useState<Record<string, ToolConfig>>({});
  const [saving, setSaving] = useState<string | null>(null);
  const [customToolName, setCustomToolName] = useState("");
  const [customDisplayName, setCustomDisplayName] = useState("");
  const [customSkillDir, setCustomSkillDir] = useState("");

  function getDraft(tool: ToolConfig) {
    return drafts[tool.tool_name] ?? tool;
  }

  function updateDraft(tool: ToolConfig, patch: Partial<ToolConfig>) {
    setDrafts((items) => ({
      ...items,
      [tool.tool_name]: { ...getDraft(tool), ...patch }
    }));
  }

  async function chooseDir(tool: ToolConfig) {
    if (!isTauri()) return;
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      updateDraft(tool, { skill_dir: selected });
    }
  }

  async function save(tool: ToolConfig) {
    const draft = getDraft(tool);
    setSaving(tool.tool_name);
    try {
      await onSave(draft);
    } finally {
      setSaving(null);
    }
  }

  async function addCustomTool() {
    await onCreateCustomTool({
      toolName: customToolName,
      displayName: customDisplayName,
      skillDir: customSkillDir
    });
    setCustomToolName("");
    setCustomDisplayName("");
    setCustomSkillDir("");
  }

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-slate-950">工具目录</h3>
            <p className="mt-2 text-sm text-slate-500">
              自动检测常见 AI 编码工具的 skill 目录，并支持自定义工具路径。
            </p>
          </div>
          <button className="action-button" onClick={() => void onDetect()}>
            <RefreshCw className="h-4 w-4" />
            重新检测
          </button>
        </div>

        <div className="mt-5 overflow-hidden rounded-3xl border border-slate-200">
          <table className="w-full text-left text-sm">
            <thead className="bg-slate-50 text-xs text-slate-500">
              <tr>
                <th className="px-4 py-3">工具</th>
                <th className="px-4 py-3">检测状态</th>
                <th className="px-4 py-3">启用</th>
                <th className="px-4 py-3">同步</th>
                <th className="px-4 py-3">Skill 目录</th>
                <th className="px-4 py-3 text-right">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {tools.map((tool) => {
                const draft = getDraft(tool);
                return (
                  <tr key={tool.tool_name}>
                    <td className="px-4 py-3 font-semibold text-slate-800">{tool.display_name}</td>
                    <td className="px-4 py-3">
                      <span className={tool.detected ? "text-emerald-600" : "text-slate-400"}>
                        {tool.detected ? "已检测到" : "未检测到"}
                      </span>
                    </td>
                    <td className="px-4 py-3">
                      <input
                        type="checkbox"
                        checked={draft.enabled}
                        onChange={(event) => updateDraft(tool, { enabled: event.target.checked })}
                      />
                    </td>
                    <td className="px-4 py-3">
                      <input
                        type="checkbox"
                        checked={draft.sync_enabled}
                        onChange={(event) => updateDraft(tool, { sync_enabled: event.target.checked })}
                      />
                    </td>
                    <td className="px-4 py-3">
                      <input
                        className="filter-control"
                        value={draft.skill_dir}
                        onChange={(event) => updateDraft(tool, { skill_dir: event.target.value })}
                      />
                    </td>
                    <td className="px-4 py-3 text-right">
                      <div className="flex justify-end gap-2">
                        <button className="action-button" onClick={() => void chooseDir(tool)}>
                          <FolderPlus className="h-4 w-4" />
                          选择
                        </button>
                        <button className="action-button" onClick={() => void save(tool)}>
                          <Save className="h-4 w-4" />
                          {saving === tool.tool_name ? "保存中" : "保存"}
                        </button>
                        {tool.is_custom ? (
                          <button
                            className="action-button text-red-600"
                            onClick={() => {
                              if (window.confirm("只删除自定义工具配置，不删除任何 skill 文件。是否继续？")) {
                                void onDeleteCustomTool(tool.tool_name);
                              }
                            }}
                          >
                            <Trash2 className="h-4 w-4" />
                            删除配置
                          </button>
                        ) : null}
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">添加自定义工具</h3>
        <p className="mt-2 text-sm text-slate-500">
          用于管理未内置支持的 AI 工具。这里只保存配置，不移动、不删除任何文件。
        </p>
        <div className="mt-5 grid grid-cols-[180px_220px_1fr_auto] gap-3">
          <input
            className="filter-control"
            value={customToolName}
            onChange={(event) => setCustomToolName(event.target.value)}
            placeholder="工具 ID，如 my_tool"
          />
          <input
            className="filter-control"
            value={customDisplayName}
            onChange={(event) => setCustomDisplayName(event.target.value)}
            placeholder="显示名称"
          />
          <input
            className="filter-control"
            value={customSkillDir}
            onChange={(event) => setCustomSkillDir(event.target.value)}
            placeholder="Skill 目录"
          />
          <button className="action-button" onClick={() => void addCustomTool()}>
            <Plus className="h-4 w-4" />
            添加
          </button>
        </div>
      </section>
    </div>
  );
}

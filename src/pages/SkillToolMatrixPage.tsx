import { useMemo, useState } from "react";
import { CheckCheck, Square, ToggleLeft, ToggleRight } from "lucide-react";
import { Skill, ToolConfig } from "../lib/tauri";

type Props = {
  skills: Skill[];
  tools: ToolConfig[];
  onToggleTool: (skillId: number, toolName: string, enabled: boolean) => Promise<void>;
};

export function SkillToolMatrixPage({ skills, tools, onToggleTool }: Props) {
  const [query, setQuery] = useState("");
  const [showArchived, setShowArchived] = useState(false);
  const [selectedSkillIds, setSelectedSkillIds] = useState<number[]>([]);
  const [selectedToolNames, setSelectedToolNames] = useState<string[]>([]);
  const [busy, setBusy] = useState<string | null>(null);

  const visibleSkills = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return skills.filter((skill) => {
      if (!showArchived && skill.status === "已归档") return false;
      if (!normalized) return true;
      return [skill.name, skill.name_zh, skill.name_en, skill.description, skill.summary_zh, skill.summary_en, skill.path]
        .filter(Boolean)
        .some((value) => value.toLowerCase().includes(normalized));
    });
  }, [query, showArchived, skills]);

  const visibleTools = useMemo(() => {
    return [...tools].sort((a, b) => a.display_name.localeCompare(b.display_name));
  }, [tools]);

  const selectedSkills = selectedSkillIds.length > 0 ? visibleSkills.filter((skill) => selectedSkillIds.includes(skill.id)) : visibleSkills;
  const selectedTools = selectedToolNames.length > 0 ? visibleTools.filter((tool) => selectedToolNames.includes(tool.tool_name)) : visibleTools;

  async function applyToSelected(enabled: boolean) {
    setBusy(enabled ? "启用中" : "禁用中");
    try {
      for (const skill of selectedSkills) {
        for (const tool of selectedTools) {
          await onToggleTool(skill.id, tool.tool_name, enabled);
        }
      }
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 className="text-lg font-semibold text-slate-950">Skill → 工具分配矩阵</h3>
            <p className="mt-2 text-sm text-slate-500">
              行是 Skill，列是工具。可批量勾选后，一次性启用或禁用多个交叉项。
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <button className="action-button" onClick={() => setSelectedSkillIds(visibleSkills.map((skill) => skill.id))}>
              <CheckCheck className="h-4 w-4" />
              全选技能
            </button>
            <button className="action-button" onClick={() => setSelectedSkillIds([])}>
              <Square className="h-4 w-4" />
              清空技能
            </button>
            <button className="action-button" onClick={() => setSelectedToolNames(visibleTools.map((tool) => tool.tool_name))}>
              <CheckCheck className="h-4 w-4" />
              全选工具
            </button>
            <button className="action-button" onClick={() => setSelectedToolNames([])}>
              <Square className="h-4 w-4" />
              清空工具
            </button>
            <button className="action-button" onClick={() => void applyToSelected(true)} disabled={Boolean(busy) || selectedSkills.length === 0 || selectedTools.length === 0}>
              <ToggleRight className="h-4 w-4" />
              {busy === "启用中" ? "启用中" : "批量启用"}
            </button>
            <button className="action-button" onClick={() => void applyToSelected(false)} disabled={Boolean(busy) || selectedSkills.length === 0 || selectedTools.length === 0}>
              <ToggleLeft className="h-4 w-4" />
              {busy === "禁用中" ? "禁用中" : "批量禁用"}
            </button>
          </div>
        </div>

        <div className="mt-5 grid grid-cols-[1fr_200px] gap-3">
          <input
            className="filter-control"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索 Skill 名称、路径、描述..."
          />
          <label className="flex items-center gap-2 rounded-2xl border border-slate-200 px-4 text-sm text-slate-700">
            <input
              type="checkbox"
              checked={showArchived}
              onChange={(event) => setShowArchived(event.target.checked)}
            />
            显示已归档
          </label>
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div className="overflow-auto rounded-3xl border border-slate-200">
          <table className="min-w-full border-separate border-spacing-0 text-left text-sm">
            <thead className="sticky top-0 bg-slate-50 text-xs text-slate-500">
              <tr>
                <th className="sticky left-0 z-10 border-b border-slate-200 bg-slate-50 px-4 py-3">Skill</th>
                <th className="border-b border-slate-200 px-4 py-3">Scope</th>
                {visibleTools.map((tool) => (
                  <th key={tool.tool_name} className="border-b border-slate-200 px-3 py-3 text-center">
                    <button
                      className="inline-flex flex-col items-center gap-1 rounded-2xl border border-slate-200 bg-white px-3 py-2 text-center text-xs font-medium text-slate-700"
                      onClick={() =>
                        setSelectedToolNames((items) =>
                          items.includes(tool.tool_name)
                            ? items.filter((item) => item !== tool.tool_name)
                            : [...items, tool.tool_name]
                        )
                      }
                    >
                      <span>{tool.display_name}</span>
                      <span className="text-[10px] text-slate-400">{tool.linkMode}</span>
                    </button>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {visibleSkills.length === 0 ? (
                <tr>
                  <td colSpan={visibleTools.length + 2} className="px-4 py-12 text-center text-sm text-slate-500">
                    没有匹配的 Skill。
                  </td>
                </tr>
              ) : (
                visibleSkills.map((skill) => (
                  <tr key={skill.id} className="hover:bg-slate-50/70">
                    <td className="sticky left-0 z-10 border-b border-slate-100 bg-white px-4 py-3">
                      <label className="flex items-start gap-3">
                        <input
                          type="checkbox"
                          checked={selectedSkillIds.length === 0 ? true : selectedSkillIds.includes(skill.id)}
                          onChange={(event) =>
                            setSelectedSkillIds((items) =>
                              event.target.checked
                                ? [...new Set([...items, skill.id])]
                                : items.filter((item) => item !== skill.id)
                            )
                          }
                        />
                        <div>
                          <div className="font-semibold text-slate-900">{skill.name}</div>
                          <div className="text-xs text-slate-500">{skill.path}</div>
                        </div>
                      </label>
                    </td>
                    <td className="border-b border-slate-100 px-4 py-3 text-xs text-slate-500">
                      {skill.scope === "project" ? `项目 · ${skill.projectPath || "未填写"}` : "全局"}
                    </td>
                    {visibleTools.map((tool) => {
                      const link = skill.tool_links.find((item) => item.tool_name === tool.tool_name);
                      const checked = Boolean(link?.enabled);
                      return (
                        <td key={tool.tool_name} className="border-b border-slate-100 px-3 py-3 text-center">
                          <input
                            type="checkbox"
                            checked={checked}
                            onChange={async (event) => {
                              await onToggleTool(skill.id, tool.tool_name, event.target.checked);
                            }}
                          />
                        </td>
                      );
                    })}
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}

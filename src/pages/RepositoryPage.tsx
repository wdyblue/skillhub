import { isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FilePlus, FolderInput, FolderPlus, Save } from "lucide-react";
import { useState } from "react";
import { RepositoryConfig } from "../lib/tauri";

type Props = {
  repositories: RepositoryConfig[];
  onSetPrimary: (path: string) => Promise<void>;
  onScan: () => void;
  onCreateSkill: (name: string, description?: string) => Promise<void>;
  onImportSkill: (path: string) => Promise<void>;
};

export function RepositoryPage({
  repositories,
  onSetPrimary,
  onScan,
  onCreateSkill,
  onImportSkill
}: Props) {
  const primary = repositories.find((repo) => repo.is_primary);
  const [path, setPath] = useState(primary?.path ?? "");
  const [saving, setSaving] = useState(false);
  const [newSkillName, setNewSkillName] = useState("");
  const [newSkillDescription, setNewSkillDescription] = useState("");
  const [importPath, setImportPath] = useState("");

  async function chooseDir() {
    if (!isTauri()) return;
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setPath(selected);
    }
  }

  async function save() {
    if (!path.trim()) return;
    setSaving(true);
    try {
      await onSetPrimary(path.trim());
    } finally {
      setSaving(false);
    }
  }

  async function chooseImportDir() {
    if (!isTauri()) return;
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setImportPath(selected);
    }
  }

  async function createSkill() {
    await onCreateSkill(newSkillName, newSkillDescription);
    setNewSkillName("");
    setNewSkillDescription("");
  }

  async function importSkill() {
    if (!importPath.trim()) return;
    await onImportSkill(importPath.trim());
    setImportPath("");
  }

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">统一 Skill 主仓库</h3>
        <p className="mt-2 text-sm text-slate-500">
          主仓库是 skill 的主要管理位置。同步到各 AI 工具时会优先创建软链接，避免复制多份文件。
        </p>
        <div className="mt-5 grid grid-cols-[1fr_auto_auto_auto] gap-3">
          <input
            className="filter-control"
            value={path}
            onChange={(event) => setPath(event.target.value)}
            placeholder="例如：/Users/admin/codebuddy-skill"
          />
          <button className="action-button" onClick={chooseDir}>
            <FolderPlus className="h-4 w-4" />
            选择目录
          </button>
          <button className="action-button" onClick={() => void save()}>
            <Save className="h-4 w-4" />
            {saving ? "保存中" : "保存主仓库"}
          </button>
          <button className="rounded-2xl bg-slate-900 px-4 py-2.5 text-sm font-semibold text-white" onClick={onScan}>
            扫描主仓库
          </button>
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">新建 / 导入 Skill</h3>
        <p className="mt-2 text-sm text-slate-500">
          新建会在主仓库创建一个包含 SKILL.md 的文件夹；导入会复制已有 skill 到主仓库，不移动源文件。
        </p>
        <div className="mt-5 grid grid-cols-[220px_1fr_auto] gap-3">
          <input
            className="filter-control"
            value={newSkillName}
            onChange={(event) => setNewSkillName(event.target.value)}
            placeholder="新 skill 名称"
          />
          <input
            className="filter-control"
            value={newSkillDescription}
            onChange={(event) => setNewSkillDescription(event.target.value)}
            placeholder="简介，可选"
          />
          <button className="action-button" onClick={() => void createSkill()}>
            <FilePlus className="h-4 w-4" />
            新建 Skill
          </button>
        </div>
        <div className="mt-3 grid grid-cols-[1fr_auto_auto] gap-3">
          <input
            className="filter-control"
            value={importPath}
            onChange={(event) => setImportPath(event.target.value)}
            placeholder="选择一个包含 SKILL.md 的已有 skill 文件夹"
          />
          <button className="action-button" onClick={() => void chooseImportDir()}>
            <FolderPlus className="h-4 w-4" />
            选择目录
          </button>
          <button className="action-button" onClick={() => void importSkill()}>
            <FolderInput className="h-4 w-4" />
            导入到主仓库
          </button>
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">已配置仓库</h3>
        <div className="mt-4 overflow-hidden rounded-3xl border border-slate-200">
          {repositories.length === 0 ? (
            <div className="px-5 py-12 text-center text-sm text-slate-500">暂未配置统一仓库。</div>
          ) : (
            <table className="w-full text-left text-sm">
              <thead className="bg-slate-50 text-xs text-slate-500">
                <tr>
                  <th className="px-4 py-3">名称</th>
                  <th className="px-4 py-3">路径</th>
                  <th className="px-4 py-3">类型</th>
                  <th className="px-4 py-3">主仓库</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {repositories.map((repo) => (
                  <tr key={repo.id}>
                    <td className="px-4 py-3 font-medium">{repo.name}</td>
                    <td className="px-4 py-3">{repo.path}</td>
                    <td className="px-4 py-3">{repo.repo_type}</td>
                    <td className="px-4 py-3">{repo.is_primary ? "是" : "否"}</td>
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

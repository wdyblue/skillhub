import Editor from "@monaco-editor/react";
import { Archive, Copy, ExternalLink, FolderOpen, RotateCcw, Save } from "lucide-react";
import { useEffect, useState } from "react";
import {
  Category,
  incrementUsage,
  openSkillFile,
  openSkillFolder,
  Skill,
  ToolConfig
} from "../lib/tauri";

type Props = {
  skill: Skill;
  categories: Category[];
  onBack: () => void;
  onUpdated: (
    id: number,
    categoryId: number | null,
    status: string,
    isCustom: boolean
  ) => Promise<void>;
  onUpdateScope: (id: number, scope: string, projectPath?: string) => Promise<void>;
  tools: ToolConfig[];
  language: "zh" | "en";
  onToggleTool: (skillId: number, toolName: string, enabled: boolean) => Promise<void>;
  onSaveContent: (skillId: number, content: string) => Promise<void>;
};

const statuses = [
  "正常",
  "常用",
  "自建",
  "待整理",
  "待合并",
  "疑似重复",
  "已归档",
  "已废弃",
  "路径丢失"
];

export function SkillDetailPage({
  skill,
  categories,
  onBack,
  onUpdated,
  onUpdateScope,
  tools,
  language,
  onToggleTool,
  onSaveContent
}: Props) {
  const [categoryId, setCategoryId] = useState<number | null>(skill.category_id);
  const [status, setStatus] = useState(skill.status);
  const [isCustom, setIsCustom] = useState(skill.is_custom);
  const [saving, setSaving] = useState(false);
  const [savingScope, setSavingScope] = useState(false);
  const [syncingTool, setSyncingTool] = useState<string | null>(null);
  const [contentDraft, setContentDraft] = useState(skill.content);
  const [savingContent, setSavingContent] = useState(false);
  const [scope, setScope] = useState(skill.scope);
  const [projectPath, setProjectPath] = useState(skill.projectPath);
  const displayName = language === "en" ? skill.name_en || skill.name_zh || skill.name : skill.name_zh || skill.name;
  const displayDescription =
    language === "en"
      ? skill.description_en || skill.description_zh || skill.description
      : skill.description_zh || skill.description;
  const displaySummary =
    language === "en" ? skill.summary_en || skill.summary_zh || displayDescription : skill.summary_zh || displayDescription;
  const t = detailLabels[language];
  const usableTools = tools.filter((tool) => tool.detected && tool.enabled && tool.sync_enabled);

  useEffect(() => {
    setContentDraft(skill.content);
    setScope(skill.scope);
    setProjectPath(skill.projectPath);
  }, [skill.id, skill.content, skill.scope, skill.projectPath]);

  async function saveMeta(nextStatus = status) {
    setSaving(true);
    try {
      await onUpdated(skill.id, categoryId, nextStatus, isCustom);
    } finally {
      setSaving(false);
    }
  }

  async function copyContent() {
    await navigator.clipboard.writeText(skill.content);
    await incrementUsage(skill.id);
  }

  async function openFolder() {
    await openSkillFolder(skill.id);
    await incrementUsage(skill.id);
  }

  async function openFile() {
    await openSkillFile(skill.id);
    await incrementUsage(skill.id);
  }

  async function saveContent() {
    setSavingContent(true);
    try {
      await onSaveContent(skill.id, contentDraft);
    } finally {
      setSavingContent(false);
    }
  }

  async function saveScope() {
    setSavingScope(true);
    try {
      await onUpdateScope(skill.id, scope, projectPath);
    } finally {
      setSavingScope(false);
    }
  }

  return (
    <div className="grid grid-cols-[360px_1fr] gap-5">
      <aside className="space-y-4">
        <button
          onClick={onBack}
          className="rounded-2xl border border-slate-200 bg-white px-4 py-2 text-sm font-semibold text-slate-700 transition hover:bg-slate-50 active:scale-[0.98]"
        >
          {t.back}
        </button>

        <section className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
          <h3 className="font-semibold text-slate-950">{t.basicInfo}</h3>
          <Info label={t.displayName} value={displayName} />
          <Info label={t.description} value={displayDescription || t.noDescription} />
          <Info label={t.summary} value={displaySummary || t.noSummary} />
          <Info label={t.path} value={skill.path} />
          <Info label={t.source} value={`${skill.source} / ${skill.platform}`} />
          <Info label={t.scope} value={skill.scope === "project" ? `项目 · ${skill.projectPath || "未填写"}` : "全局"} />
          <Info label={t.status} value={translateStatus(skill.status, language)} />
          <Info label={t.quality} value={`${skill.quality_score} / 100`} />
          <Info label={t.duplicateRisk} value={`${skill.duplicate_score}`} />
          <Info label={t.archiveRecommendation} value={translateRecommendation(skill.archive_recommendation, language)} />
          <Info label={t.classificationConfidence} value={`${Math.round(skill.classification_confidence * 100)}%`} />
          <Info label={t.usageCount} value={`${skill.usage_count}`} />
          <Info label="Hash" value={skill.hash} />
          <Info label={t.updatedAt} value={skill.updated_at} />
        </section>

        <section className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
          <h3 className="font-semibold text-slate-950">{t.management}</h3>
          <div className="mt-4 space-y-3">
            <label className="space-y-2">
              <span className="text-xs font-medium text-slate-500">{t.scope}</span>
              <select className="filter-control" value={scope} onChange={(event) => setScope(event.target.value)}>
                <option value="global">{t.globalScope}</option>
                <option value="project">{t.projectScope}</option>
              </select>
            </label>
            {scope === "project" ? (
              <label className="space-y-2">
                <span className="text-xs font-medium text-slate-500">{t.projectPath}</span>
                <input
                  className="filter-control"
                  value={projectPath}
                  onChange={(event) => setProjectPath(event.target.value)}
                  placeholder="/path/to/project"
                />
              </label>
            ) : null}
            <button
              onClick={() => void saveScope()}
              disabled={savingScope}
              className="flex w-full items-center justify-center gap-2 rounded-2xl bg-slate-900 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-slate-800 active:scale-[0.98] disabled:opacity-60"
            >
              {savingScope ? t.saving : t.saveScope}
            </button>
            <label className="space-y-2">
              <span className="text-xs font-medium text-slate-500">{t.category}</span>
              <select
                className="filter-control"
                value={categoryId ?? ""}
                onChange={(event) =>
                  setCategoryId(event.target.value ? Number(event.target.value) : null)
                }
              >
                <option value="">{t.uncategorized}</option>
                {categories.map((category) => (
                  <option key={category.id} value={category.id}>
                    {category.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="space-y-2">
              <span className="text-xs font-medium text-slate-500">{t.status}</span>
              <select
                className="filter-control"
                value={status}
                onChange={(event) => setStatus(event.target.value)}
              >
                {statuses.map((item) => (
                  <option key={item} value={item}>
                    {translateStatus(item, language)}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex items-center gap-2 text-sm text-slate-700">
              <input
                type="checkbox"
                checked={isCustom}
                onChange={(event) => setIsCustom(event.target.checked)}
              />
              {t.markCustom}
            </label>
            <button
              onClick={() => void saveMeta()}
              disabled={saving}
              className="flex w-full items-center justify-center gap-2 rounded-2xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-brand-700 active:scale-[0.98] disabled:opacity-60"
            >
              <Save className="h-4 w-4" />
              {saving ? t.saving : t.saveMeta}
            </button>
            {skill.status === "已归档" ? (
              <button
                onClick={() => {
                  setStatus("正常");
                  void saveMeta("正常");
                }}
                className="flex w-full items-center justify-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-semibold text-slate-700 transition hover:bg-slate-50 active:scale-[0.98]"
              >
                <RotateCcw className="h-4 w-4" />
                {t.restore}
              </button>
            ) : (
              <button
                onClick={() => {
                  setStatus("已归档");
                  void saveMeta("已归档");
                }}
                className="flex w-full items-center justify-center gap-2 rounded-2xl border border-amber-200 bg-amber-50 px-4 py-2.5 text-sm font-semibold text-amber-700 transition hover:bg-amber-100 active:scale-[0.98]"
              >
                <Archive className="h-4 w-4" />
                {t.archive}
              </button>
            )}
          </div>
        </section>

        <section className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
          <h3 className="font-semibold text-slate-950">{t.toolSync}</h3>
          <p className="mt-2 text-sm text-slate-500">
            {t.toolSyncTip}
          </p>
          <div className="mt-4 space-y-3">
            {usableTools.length === 0 ? (
              <p className="text-sm text-slate-500">{t.noTools}</p>
            ) : (
              usableTools.map((tool) => {
                const link = skill.tool_links.find((item) => item.tool_name === tool.tool_name);
                const enabled = Boolean(link?.enabled);
                return (
                  <label key={tool.tool_name} className="block rounded-2xl border border-slate-200 p-3">
                    <div className="flex items-center justify-between gap-3">
                      <span className="font-medium text-slate-800">{tool.display_name}</span>
                      <input
                        type="checkbox"
                        checked={enabled}
                        disabled={syncingTool === tool.tool_name}
                        onChange={async (event) => {
                          setSyncingTool(tool.tool_name);
                          try {
                            await onToggleTool(skill.id, tool.tool_name, event.target.checked);
                          } finally {
                            setSyncingTool(null);
                          }
                        }}
                      />
                    </div>
                    <p className="mt-2 break-all text-xs text-slate-500">
                      {translateLinkStatus(link?.link_status ?? "未启用", language)} · {link?.link_path || tool.skill_dir}
                    </p>
                    {link?.error_message ? (
                      <p className="mt-1 text-xs text-red-600">{link.error_message}</p>
                    ) : null}
                  </label>
                );
              })
            )}
          </div>
        </section>
      </aside>

      <section className="space-y-4">
        <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex flex-wrap items-center gap-3">
            <button className="action-button" onClick={copyContent}>
              <Copy className="h-4 w-4" />
              {t.copy}
            </button>
            <button className="action-button" onClick={openFolder}>
              <FolderOpen className="h-4 w-4" />
              {t.openFolder}
            </button>
            <button className="action-button" onClick={openFile}>
              <ExternalLink className="h-4 w-4" />
              {t.openFile}
            </button>
            <button className="action-button" onClick={() => void saveContent()}>
              <Save className="h-4 w-4" />
              {savingContent ? t.saving : t.saveFile}
            </button>
          </div>
        </div>

        <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
          <h3 className="font-semibold text-slate-950">{t.scoreReason}</h3>
          <p className="mt-3 whitespace-pre-wrap rounded-2xl bg-slate-50 p-4 text-sm leading-6 text-slate-600">
            {skill.quality_reason || t.noScoreReason}
          </p>
        </div>

        <div className="overflow-hidden rounded-[2rem] border border-slate-200 bg-white shadow-sm">
          <div className="border-b border-slate-200 px-5 py-4">
            <h3 className="font-semibold text-slate-950">{t.preview}</h3>
            <p className="mt-1 text-xs text-slate-500">{t.editorTip}</p>
          </div>
          <div className="h-[620px]">
            <Editor
              value={contentDraft}
              onChange={(value) => setContentDraft(value ?? "")}
              language="markdown"
              theme="vs"
              options={{
                readOnly: false,
                minimap: { enabled: false },
                wordWrap: "on",
                fontSize: 13,
                lineNumbers: "on"
              }}
            />
          </div>
        </div>
      </section>
    </div>
  );
}

const detailLabels = {
  zh: {
    back: "返回列表",
    basicInfo: "基础信息",
    displayName: "展示名称",
    description: "简介",
    summary: "摘要",
    noDescription: "暂无描述",
    noSummary: "暂无摘要",
    path: "路径",
    source: "来源平台",
    scope: "作用域",
    globalScope: "全局",
    projectScope: "项目",
    projectPath: "项目路径",
    saveScope: "保存作用域",
    status: "状态",
    quality: "质量评分",
    duplicateRisk: "重复风险",
    archiveRecommendation: "归档建议",
    classificationConfidence: "分类置信度",
    usageCount: "使用次数",
    updatedAt: "更新时间",
    management: "管理操作",
    category: "分类",
    uncategorized: "未分类",
    markCustom: "标记为自建 skill",
    saving: "保存中...",
    saveMeta: "保存元信息",
    restore: "恢复为正常",
    archive: "归档，不删除文件",
    toolSync: "多工具启用状态",
    toolSyncTip: "开启会按工具链接策略创建同步产物；如果工具目录本身就是主仓库，则按直连处理，不再重复建链接。",
    noTools: "请先到“工具目录”页面检测或配置工具。",
    copy: "复制 SKILL.md",
    openFolder: "打开文件夹",
    openFile: "打开 SKILL.md",
    saveFile: "保存 SKILL.md",
    editorTip: "这里可以直接编辑 SKILL.md。保存只写回当前 skill 文件，不移动或删除其他文件。",
    scoreReason: "评分原因",
    noScoreReason: "暂无评分原因。",
    preview: "SKILL.md 内容预览"
  },
  en: {
    back: "Back to list",
    basicInfo: "Basic Info",
    displayName: "Display Name",
    description: "Description",
    summary: "Summary",
    noDescription: "No description",
    noSummary: "No summary",
    path: "Path",
    source: "Source / Platform",
    scope: "Scope",
    globalScope: "Global",
    projectScope: "Project",
    projectPath: "Project Path",
    saveScope: "Save Scope",
    status: "Status",
    quality: "Quality Score",
    duplicateRisk: "Duplicate Risk",
    archiveRecommendation: "Archive Recommendation",
    classificationConfidence: "Classification Confidence",
    usageCount: "Usage Count",
    updatedAt: "Updated At",
    management: "Management",
    category: "Category",
    uncategorized: "Uncategorized",
    markCustom: "Mark as custom skill",
    saving: "Saving...",
    saveMeta: "Save metadata",
    restore: "Restore to normal",
    archive: "Archive without deleting files",
    toolSync: "Multi-tool Enablement",
    toolSyncTip: "Enabling follows the tool link strategy; if the tool directory is the primary repository, it is treated as direct access.",
    noTools: "Detect or configure tools in Tool Directories first.",
    copy: "Copy SKILL.md",
    openFolder: "Open folder",
    openFile: "Open SKILL.md",
    saveFile: "Save SKILL.md",
    editorTip: "You can edit SKILL.md here. Saving only writes this skill file and does not move or delete other files.",
    scoreReason: "Score Reason",
    noScoreReason: "No score reason.",
    preview: "SKILL.md Preview"
  }
};

function translateStatus(status: string, language: "zh" | "en") {
  if (language === "zh") return status;
  return (
    {
      正常: "Normal",
      常用: "Favorite",
      自建: "Custom",
      待整理: "Needs Cleanup",
      待合并: "Needs Merge",
      疑似重复: "Possible Duplicate",
      已归档: "Archived",
      已废弃: "Deprecated",
      路径丢失: "Missing Path"
    }[status] ?? status
  );
}

function translateRecommendation(value: string, language: "zh" | "en") {
  if (language === "zh") return value;
  return (
    {
      建议保留: "Keep",
      建议归档: "Archive",
      建议合并: "Merge",
      建议检查: "Review",
      疑似旧版本: "Possible old version",
      疑似重复: "Possible duplicate"
    }[value] ?? value
  );
}

function translateLinkStatus(value: string, language: "zh" | "en") {
  if (language === "zh") return value;
  return (
    {
      未启用: "Disabled",
      已同步: "Synced",
      同步失败: "Sync failed",
      移除失败: "Remove failed",
      未同步: "Not synced"
    }[value] ?? value
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="mt-4">
      <p className="text-xs font-medium text-slate-500">{label}</p>
      <p className="mt-1 break-all text-sm leading-5 text-slate-800">{value || "-"}</p>
    </div>
  );
}

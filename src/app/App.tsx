import { useEffect, useMemo, useState } from "react";
import {
  Archive,
  FolderCog,
  Gauge,
  Home,
  Layers3,
  RotateCw,
  Search,
  Settings,
  ShieldCheck,
  Sparkles,
  Stethoscope,
  Wrench
} from "lucide-react";
import {
  addScanRoot,
  Category,
  checkSyncStatus,
  createCategory,
  createCustomTool,
  createSkillInRepository,
  detectTools,
  deleteCustomTool,
  deleteCategory,
  fixSyncIssues,
  getSkill,
  getStats,
  listRepositories,
  listCategories,
  listScanRoots,
  listSkills,
  listTools,
  RepositoryConfig,
  removeScanRoot,
  scanAll,
  importSkillToRepository,
  saveSkillContent,
  setPrimaryRepository,
  setSkillToolEnabled,
  Skill,
  SkillListFilters,
  AppStats,
  ScanRoot,
  SyncReport,
  toggleScanRoot,
  ToolConfig,
  updateCategory,
  updateToolConfig,
  updateSkillMeta
} from "../lib/tauri";
import { isTauri } from "@tauri-apps/api/core";
import { cn } from "../lib/cn";
import { DashboardPage } from "../pages/DashboardPage";
import { SkillsPage } from "../pages/SkillsPage";
import { SettingsPage } from "../pages/SettingsPage";
import { SkillDetailPage } from "../pages/SkillDetailPage";
import { PlaceholderPage } from "../pages/PlaceholderPage";
import { ToolDirectoriesPage } from "../pages/ToolDirectoriesPage";
import { RepositoryPage } from "../pages/RepositoryPage";
import { SyncHealthPage } from "../pages/SyncHealthPage";
import { CategoryManagementPage } from "../pages/CategoryManagementPage";
import { DuplicatesPage } from "../pages/DuplicatesPage";
import { HealthPage } from "../pages/HealthPage";
import { CustomSkillsPage } from "../pages/CustomSkillsPage";

type PageKey =
  | "home"
  | "skills"
  | "categories"
  | "duplicates"
  | "health"
  | "sync"
  | "tools"
  | "repository"
  | "favorites"
  | "custom"
  | "archive"
  | "remote"
  | "settings";

const navItems: Array<{
  key: PageKey;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}> = [
  { key: "home", label: "首页", icon: Home },
  { key: "skills", label: "全部技能", icon: Layers3 },
  { key: "categories", label: "分类管理", icon: FolderCog },
  { key: "duplicates", label: "重复检测", icon: ShieldCheck },
  { key: "health", label: "技能体检", icon: Gauge },
  { key: "sync", label: "同步体检", icon: Stethoscope },
  { key: "tools", label: "工具目录", icon: Wrench },
  { key: "repository", label: "统一仓库", icon: FolderCog },
  { key: "favorites", label: "常用技能", icon: Sparkles },
  { key: "custom", label: "自建技能", icon: FolderCog },
  { key: "archive", label: "归档箱", icon: Archive },
  { key: "remote", label: "远程仓库", icon: RotateCw },
  { key: "settings", label: "设置", icon: Settings }
];

export default function App() {
  const [language, setLanguage] = useState<"zh" | "en">(
    (localStorage.getItem("skillhub.language") as "zh" | "en" | null) ?? "zh"
  );
  const [activePage, setActivePage] = useState<PageKey>("home");
  const [selectedSkillId, setSelectedSkillId] = useState<number | null>(null);
  const [stats, setStats] = useState<AppStats | null>(null);
  const [categories, setCategories] = useState<Category[]>([]);
  const [skills, setSkills] = useState<Skill[]>([]);
  const [scanRoots, setScanRoots] = useState<ScanRoot[]>([]);
  const [tools, setTools] = useState<ToolConfig[]>([]);
  const [repositories, setRepositories] = useState<RepositoryConfig[]>([]);
  const [syncReport, setSyncReport] = useState<SyncReport | null>(null);
  const [filters, setFilters] = useState<SkillListFilters>({
    sortBy: "updated_at",
    sortOrder: "desc"
  });
  const [loading, setLoading] = useState(true);
  const [busyText, setBusyText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const t = useMemo(() => labels[language], [language]);
  const activeTitle = useMemo(
    () => navItems.find((item) => item.key === activePage)?.label ?? "SkillHub",
    [activePage]
  );
  const categorySkillCounts = useMemo(() => {
    return skills.reduce<Record<number, number>>((acc, skill) => {
      if (skill.category_id !== null) {
        acc[skill.category_id] = (acc[skill.category_id] ?? 0) + 1;
      }
      return acc;
    }, {});
  }, [skills]);

  async function refreshAll(nextFilters = filters) {
    setError(null);
    try {
      const [nextStats, nextCategories, nextSkills, nextRoots, nextTools, nextRepos] =
        await Promise.all([
          getStats(),
          listCategories(),
          listSkills(nextFilters),
          listScanRoots(),
          listTools().catch(() => []),
          listRepositories().catch(() => [])
        ]);
      setStats(nextStats);
      setCategories(nextCategories);
      setSkills(nextSkills);
      setScanRoots(nextRoots);
      setTools(nextTools);
      setRepositories(nextRepos);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    if (!isTauri()) {
      setError(
        "SkillHub 需要在 Tauri 桌面端中运行。请不要直接在浏览器里打开 http://localhost:1420。"
      );
      setLoading(false);
      return;
    }

    void refreshAll();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleFiltersChange(next: SkillListFilters) {
    setFilters(next);
    setLoading(true);
    await refreshAll(next);
  }

  async function handleScan() {
    setBusyText("正在扫描本地技能目录...");
    try {
      await scanAll();
      await refreshAll();
      setActivePage("skills");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyText(null);
    }
  }

  async function handleAddRoot(path: string, platform: string) {
    setBusyText("正在添加目录...");
    try {
      await addScanRoot(path, platform);
      await refreshAll();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyText(null);
    }
  }

  async function handleRemoveRoot(id: number) {
    await removeScanRoot(id);
    await refreshAll();
  }

  async function handleToggleRoot(id: number, enabled: boolean) {
    await toggleScanRoot(id, enabled);
    await refreshAll();
  }

  async function handleSkillUpdate(
    id: number,
    categoryId: number | null,
    status: string,
    isCustom: boolean
  ) {
    await updateSkillMeta(id, categoryId, status, isCustom);
    await refreshAll();
    const updated = await getSkill(id);
    setSkills((items) => items.map((item) => (item.id === id ? updated : item)));
  }

  async function handleCreateCategory(input: { name: string; nameEn?: string; color?: string }) {
    await createCategory(input);
    await refreshAll();
  }

  async function handleUpdateCategory(
    id: number,
    input: { name: string; nameEn?: string; color?: string }
  ) {
    await updateCategory(id, input);
    await refreshAll();
  }

  async function handleDeleteCategory(id: number) {
    await deleteCategory(id);
    await refreshAll();
  }

  async function handleDetectTools() {
    setBusyText("正在检测工具目录...");
    try {
      const nextTools = await detectTools();
      setTools(nextTools);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyText(null);
    }
  }

  async function handleSaveTool(tool: ToolConfig) {
    await updateToolConfig({
      toolName: tool.tool_name,
      skillDir: tool.skill_dir,
      enabled: tool.enabled,
      syncEnabled: tool.sync_enabled
    });
    await handleDetectTools();
  }

  async function handleCreateCustomTool(request: {
    toolName: string;
    displayName: string;
    skillDir: string;
  }) {
    await createCustomTool(request);
    await handleDetectTools();
  }

  async function handleDeleteCustomTool(toolName: string) {
    await deleteCustomTool(toolName);
    await handleDetectTools();
  }

  async function handleSetRepository(path: string) {
    await setPrimaryRepository(path);
    await refreshAll();
  }

  async function handleCreateSkill(name: string, description?: string) {
    const id = await createSkillInRepository({ name, description });
    await refreshAll();
    setSelectedSkillId(id);
  }

  async function handleImportSkill(path: string) {
    const id = await importSkillToRepository(path);
    await refreshAll();
    setSelectedSkillId(id);
  }

  async function handleCheckSync() {
    setBusyText("正在检查同步状态...");
    try {
      setSyncReport(await checkSyncStatus());
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyText(null);
    }
  }

  async function handleFixSync() {
    setBusyText("正在修复同步异常...");
    try {
      setSyncReport(await fixSyncIssues());
      await refreshAll();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyText(null);
    }
  }

  async function handleSkillToolToggle(skillId: number, toolName: string, enabled: boolean) {
    await setSkillToolEnabled(skillId, toolName, enabled);
    const updated = await getSkill(skillId);
    setSkills((items) => items.map((item) => (item.id === skillId ? updated : item)));
  }

  async function handleSaveSkillContent(skillId: number, content: string) {
    await saveSkillContent(skillId, content);
    const updated = await getSkill(skillId);
    setSkills((items) => items.map((item) => (item.id === skillId ? updated : item)));
  }

  function changeLanguage(next: "zh" | "en") {
    setLanguage(next);
    localStorage.setItem("skillhub.language", next);
  }

  const selectedSkill =
    selectedSkillId === null
      ? null
      : skills.find((skill) => skill.id === selectedSkillId) ?? null;

  return (
    <div className="grid min-h-[100dvh] grid-cols-[260px_1fr] bg-slate-50">
      <aside className="border-r border-slate-200 bg-white/90 px-4 py-5 shadow-sm">
        <div className="mb-7 rounded-3xl bg-brand-600 p-4 text-white shadow-soft">
          <p className="text-xs font-medium text-blue-100">{t.tagline}</p>
          <h1 className="mt-1 text-2xl font-semibold tracking-tight">SkillHub</h1>
          <p className="mt-2 text-sm text-blue-100">{t.version}</p>
        </div>

        <nav className="space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            const active = activePage === item.key;
            return (
              <button
                key={item.key}
                className={cn(
                  "flex w-full items-center gap-3 rounded-2xl px-3 py-2.5 text-left text-sm transition active:scale-[0.98]",
                  active
                    ? "bg-brand-50 font-semibold text-brand-700"
                    : "text-slate-600 hover:bg-slate-100 hover:text-slate-950"
                )}
                onClick={() => {
                  setActivePage(item.key);
                  setSelectedSkillId(null);
                  if (item.key === "archive") {
                    void handleFiltersChange({
                      ...filters,
                      onlyArchived: true,
                      status: "已归档"
                    });
                  } else if (item.key === "skills") {
                    void handleFiltersChange({
                      ...filters,
                      onlyArchived: false,
                      status: undefined
                    });
                  } else if (["categories", "duplicates", "health", "custom", "favorites"].includes(item.key)) {
                    void handleFiltersChange({
                      sortBy: "updated_at",
                      sortOrder: "desc",
                      onlyArchived: false,
                      status: undefined
                    });
                  }
                }}
              >
                <Icon className="h-4 w-4" />
                {t.nav[item.key] ?? item.label}
              </button>
            );
          })}
        </nav>
      </aside>

      <main className="min-w-0 overflow-hidden">
        <header className="flex h-20 items-center justify-between border-b border-slate-200 bg-white/80 px-7 backdrop-blur">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight text-slate-950">
              {selectedSkill ? displaySkillName(selectedSkill, language) : t.nav[activePage] ?? activeTitle}
            </h2>
            <p className="mt-1 text-sm text-slate-500">
              {selectedSkill
                ? t.skillDetailSubtitle
                : t.defaultSubtitle}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <div className="relative">
              <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
              <input
                value={filters.query ?? ""}
                onChange={(event) => {
                  const query = event.target.value || undefined;
                  if (query?.trim()) {
                    setSelectedSkillId(null);
                    setActivePage("skills");
                  }
                  void handleFiltersChange({
                    ...filters,
                    query
                  });
                }}
                className="h-10 w-72 rounded-2xl border border-slate-200 bg-white pl-9 pr-3 text-sm outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                placeholder={t.searchPlaceholder}
              />
            </div>
            <select
              value={language}
              onChange={(event) => changeLanguage(event.target.value as "zh" | "en")}
              className="h-10 rounded-2xl border border-slate-200 bg-white px-3 text-sm outline-none"
            >
              <option value="zh">简体中文</option>
              <option value="en">English</option>
            </select>
            <button
              onClick={handleScan}
              disabled={Boolean(busyText)}
              className="rounded-2xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm transition hover:bg-brand-700 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-60"
            >
              {busyText ?? t.rescan}
            </button>
          </div>
        </header>

        {error ? (
          <div className="mx-7 mt-5 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
            {error}
          </div>
        ) : null}

        <section className="h-[calc(100dvh-80px)] overflow-auto px-7 py-6">
          {selectedSkill ? (
            <SkillDetailPage
              skill={selectedSkill}
              categories={categories}
              onBack={() => setSelectedSkillId(null)}
              onUpdated={handleSkillUpdate}
              tools={tools}
              language={language}
              onToggleTool={handleSkillToolToggle}
              onSaveContent={handleSaveSkillContent}
            />
          ) : activePage === "home" ? (
            <DashboardPage
              loading={loading}
              stats={stats}
              skills={skills}
              scanRoots={scanRoots}
              onScan={handleScan}
              onGoSkills={() => setActivePage("skills")}
              onGoSettings={() => setActivePage("settings")}
            />
          ) : activePage === "skills" || activePage === "archive" ? (
            <SkillsPage
              loading={loading}
              skills={skills}
              categories={categories}
              filters={filters}
              onFiltersChange={handleFiltersChange}
              onSelectSkill={setSelectedSkillId}
              tools={tools}
              language={language}
            />
          ) : activePage === "settings" ? (
            <SettingsPage
              scanRoots={scanRoots}
              onAddRoot={handleAddRoot}
              onRemoveRoot={handleRemoveRoot}
              onToggleRoot={handleToggleRoot}
              onScan={handleScan}
            />
          ) : activePage === "tools" ? (
            <ToolDirectoriesPage
              tools={tools}
              onDetect={handleDetectTools}
              onSave={handleSaveTool}
              onCreateCustomTool={handleCreateCustomTool}
              onDeleteCustomTool={handleDeleteCustomTool}
            />
          ) : activePage === "repository" ? (
            <RepositoryPage
              repositories={repositories}
              onSetPrimary={handleSetRepository}
              onScan={handleScan}
              onCreateSkill={handleCreateSkill}
              onImportSkill={handleImportSkill}
            />
          ) : activePage === "sync" ? (
            <SyncHealthPage report={syncReport} onCheck={handleCheckSync} onFix={handleFixSync} />
          ) : activePage === "duplicates" ? (
            <DuplicatesPage skills={skills} onSelectSkill={setSelectedSkillId} />
          ) : activePage === "health" ? (
            <HealthPage skills={skills} onSelectSkill={setSelectedSkillId} />
          ) : activePage === "categories" ? (
            <CategoryManagementPage
              categories={categories}
              skillCounts={categorySkillCounts}
              onCreate={handleCreateCategory}
              onUpdate={handleUpdateCategory}
              onDelete={handleDeleteCategory}
            />
          ) : activePage === "custom" ? (
            <CustomSkillsPage
              skills={skills}
              onSelectSkill={setSelectedSkillId}
              onGoRepository={() => setActivePage("repository")}
            />
          ) : (
            <PlaceholderPage
              title={activeTitle}
              description="该页面已预留入口，v0.1 后续迭代实现。"
            />
          )}
        </section>
      </main>
    </div>
  );
}

const labels: Record<"zh" | "en", {
  tagline: string;
  version: string;
  skillDetailSubtitle: string;
  defaultSubtitle: string;
  searchPlaceholder: string;
  rescan: string;
  nav: Record<string, string>;
}> = {
  zh: {
    tagline: "本地优先 AI Skill 资产管理",
    version: "技能管理器 v0.2.0",
    skillDetailSubtitle: "查看、评分、归档、同步和管理单个 Skill",
    defaultSubtitle: "本地扫描、分类、去重、评分、归档和多工具同步",
    searchPlaceholder: "搜索技能名称、描述、路径...",
    rescan: "重新扫描",
    nav: {
      home: "首页",
      skills: "全部技能",
      categories: "分类管理",
      duplicates: "重复检测",
      health: "技能体检",
      sync: "同步体检",
      tools: "工具目录",
      repository: "统一仓库",
      favorites: "常用技能",
      custom: "自建技能",
      archive: "归档箱",
      remote: "远程仓库",
      settings: "设置"
    }
  },
  en: {
    tagline: "Local-first AI Skill Asset Manager",
    version: "Skill Manager v0.2.0",
    skillDetailSubtitle: "Review, score, archive, sync, and manage one skill",
    defaultSubtitle: "Scan, classify, dedupe, score, archive, and sync skills",
    searchPlaceholder: "Search skill name, description, path...",
    rescan: "Rescan",
    nav: {
      home: "Home",
      skills: "All Skills",
      categories: "Categories",
      duplicates: "Duplicates",
      health: "Skill Health",
      sync: "Sync Health",
      tools: "Tool Directories",
      repository: "Unified Repository",
      favorites: "Favorites",
      custom: "Custom Skills",
      archive: "Archive",
      remote: "Remote Repos",
      settings: "Settings"
    }
  }
};

function displaySkillName(skill: Skill, language: "zh" | "en") {
  return language === "en" ? skill.name_en || skill.name_zh || skill.name : skill.name_zh || skill.name;
}

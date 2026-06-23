import { Archive, Clock, FolderOpen, Languages, Loader2, Star, Tags } from "lucide-react";
import { Category, Skill, SkillListFilters, ToolConfig } from "../lib/tauri";
import { cn } from "../lib/cn";

type Props = {
  loading: boolean;
  skills: Skill[];
  categories: Category[];
  filters: SkillListFilters;
  onFiltersChange: (filters: SkillListFilters) => void;
  onSelectSkill: (id: number) => void;
  tools: ToolConfig[];
  language: "zh" | "en";
  translatingSkillId: number | null;
  onTranslateSkill: (id: number) => Promise<void>;
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

export function SkillsPage({
  loading,
  skills,
  categories,
  filters,
  onFiltersChange,
  onSelectSkill,
  tools,
  language,
  translatingSkillId,
  onTranslateSkill
}: Props) {
  const t = skillPageLabels[language];
  return (
    <div className="space-y-5">
      <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
        <div className="grid grid-cols-6 gap-3">
          <FilterBlock label={t.category}>
            <select
              value={filters.categoryId ?? ""}
              onChange={(event) =>
                onFiltersChange({
                  ...filters,
                  categoryId: event.target.value ? Number(event.target.value) : undefined
                })
              }
              className="filter-control"
            >
              <option value="">{t.allCategories}</option>
              {categories.map((category) => (
                <option key={category.id} value={category.id}>
                  {language === "en" ? category.name_en || translateCategory(category.name) : category.name}
                </option>
              ))}
            </select>
          </FilterBlock>
          <FilterBlock label={t.status}>
            <select
              value={filters.status ?? ""}
              onChange={(event) =>
                onFiltersChange({
                  ...filters,
                  status: event.target.value || undefined,
                  onlyArchived: event.target.value === "已归档"
                })
              }
              className="filter-control"
            >
              <option value="">{t.allStatuses}</option>
              {statuses.map((status) => (
                <option key={status} value={status}>
                  {translateStatus(status, language)}
                </option>
              ))}
            </select>
          </FilterBlock>
          <FilterBlock label={t.source}>
            <select
              value={filters.source ?? ""}
              onChange={(event) =>
                onFiltersChange({ ...filters, source: event.target.value || undefined })
              }
              className="filter-control"
            >
              <option value="">{t.allSources}</option>
              <option value="Codex">Codex</option>
              <option value="ChatGPT">ChatGPT</option>
              <option value="Claude">Claude</option>
              <option value="Hermes">Hermes</option>
              <option value="Cursor">Cursor</option>
              <option value="自建">自建</option>
              <option value="未知">未知</option>
            </select>
          </FilterBlock>
          <FilterBlock label={t.sort}>
            <select
              value={filters.sortBy ?? "updated_at"}
              onChange={(event) =>
                onFiltersChange({
                  ...filters,
                  sortBy: event.target.value as SkillListFilters["sortBy"]
                })
              }
              className="filter-control"
            >
              <option value="updated_at">{t.updatedAt}</option>
              <option value="quality_score">{t.qualityScore}</option>
              <option value="usage_count">{t.usageCount}</option>
              <option value="name">{t.name}</option>
            </select>
          </FilterBlock>
          <FilterBlock label={t.direction}>
            <select
              value={filters.sortOrder ?? "desc"}
              onChange={(event) =>
                onFiltersChange({
                  ...filters,
                  sortOrder: event.target.value as SkillListFilters["sortOrder"]
                })
              }
              className="filter-control"
            >
              <option value="desc">{t.desc}</option>
              <option value="asc">{t.asc}</option>
            </select>
          </FilterBlock>
          <div className="flex items-end">
            <button
              className="h-10 w-full rounded-2xl border border-slate-200 bg-white text-sm font-semibold text-slate-700 transition hover:bg-slate-50 active:scale-[0.98]"
              onClick={() =>
                onFiltersChange({ sortBy: "updated_at", sortOrder: "desc" })
              }
            >
              {t.reset}
            </button>
          </div>
        </div>

        <div className="mt-4 flex flex-wrap gap-2">
          <ToggleChip
            active={Boolean(filters.onlyUncategorized)}
            onClick={() =>
              onFiltersChange({
                ...filters,
                onlyUncategorized: !filters.onlyUncategorized
              })
            }
          >
            {t.onlyUncategorized}
          </ToggleChip>
          <ToggleChip
            active={Boolean(filters.onlyDuplicate)}
            onClick={() =>
              onFiltersChange({
                ...filters,
                onlyDuplicate: !filters.onlyDuplicate
              })
            }
          >
            {t.onlyDuplicate}
          </ToggleChip>
          <ToggleChip
            active={Boolean(filters.onlyArchived)}
            onClick={() =>
              onFiltersChange({
                ...filters,
                onlyArchived: !filters.onlyArchived,
                status: !filters.onlyArchived ? "已归档" : undefined
              })
            }
          >
            {t.onlyArchived}
          </ToggleChip>
        </div>
      </div>

      {loading ? (
        <div className="grid grid-cols-3 gap-4">
          {Array.from({ length: 6 }).map((_, index) => (
            <div
              key={index}
              className="h-56 animate-pulse rounded-[2rem] border border-slate-200 bg-white"
            />
          ))}
        </div>
      ) : skills.length === 0 ? (
        <div className="rounded-[2rem] border border-dashed border-slate-300 bg-white px-6 py-16 text-center shadow-sm">
          <h3 className="text-xl font-semibold text-slate-950">{t.emptyTitle}</h3>
          <p className="mt-2 text-sm text-slate-500">
            {t.emptyDescription}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-3 gap-4">
          {skills.map((skill) => (
            <SkillCard
              key={skill.id}
              skill={skill}
              tools={tools}
              language={language}
              translating={translatingSkillId === skill.id}
              onTranslate={() => onTranslateSkill(skill.id)}
              onClick={() => onSelectSkill(skill.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function FilterBlock({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="space-y-2">
      <span className="text-xs font-medium text-slate-500">{label}</span>
      {children}
    </label>
  );
}

function ToggleChip({
  active,
  children,
  onClick
}: {
  active: boolean;
  children: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "rounded-full border px-3 py-1.5 text-xs font-medium transition active:scale-[0.98]",
        active
          ? "border-brand-200 bg-brand-50 text-brand-700"
          : "border-slate-200 bg-white text-slate-600 hover:bg-slate-50"
      )}
    >
      {children}
    </button>
  );
}

function SkillCard({
  skill,
  tools,
  language,
  translating,
  onTranslate,
  onClick
}: {
  skill: Skill;
  tools: ToolConfig[];
  language: "zh" | "en";
  translating: boolean;
  onTranslate: () => Promise<void>;
  onClick: () => void;
}) {
  const scoreColor =
    skill.quality_score >= 80
      ? "text-emerald-600 bg-emerald-50"
      : skill.quality_score >= 60
        ? "text-amber-600 bg-amber-50"
        : "text-red-600 bg-red-50";

  const name = language === "en" ? skill.name_en || skill.name_zh || skill.name : skill.name_zh || skill.name;
  const description =
    language === "en"
      ? skill.description_en || skill.description_zh || skill.description
      : skill.description_zh || skill.description;
  const pendingTranslation =
    language === "en" && !skill.name_en && !skill.description_en
      ? "待翻译"
      : language === "zh" && !skill.name_zh && !skill.description_zh
        ? "待翻译"
        : null;

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      className="group flex min-h-64 flex-col rounded-[2rem] border border-slate-200 bg-white p-5 text-left shadow-sm transition hover:-translate-y-0.5 hover:shadow-soft active:scale-[0.98]"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h3 className="line-clamp-2 text-lg font-semibold tracking-tight text-slate-950">
            {name}
          </h3>
          <p className="mt-2 line-clamp-3 text-sm leading-6 text-slate-600">
            {description || "暂无描述。"}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          <button
            type="button"
            title={language === "en" ? "Translate skill" : "翻译技能"}
            disabled={translating}
            onClick={(event) => {
              event.stopPropagation();
              void onTranslate();
            }}
            className="inline-flex h-9 w-9 items-center justify-center rounded-2xl border border-slate-200 bg-white text-slate-500 transition hover:border-brand-200 hover:bg-brand-50 hover:text-brand-700 disabled:cursor-wait disabled:opacity-60"
          >
            {translating ? <Loader2 className="h-4 w-4 animate-spin" /> : <Languages className="h-4 w-4" />}
          </button>
          <span className={cn("rounded-full px-2.5 py-1 text-xs font-semibold", scoreColor)}>
            {skill.quality_score}
          </span>
        </div>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        <Badge>{language === "en" ? translateCategory(skill.category_name ?? "未分类") : skill.category_name ?? "未分类"}</Badge>
        <Badge>{skill.source}</Badge>
        <Badge>{translateStatus(skill.status, language)}</Badge>
        <Badge>{translateRecommendation(skill.archive_recommendation, language)}</Badge>
        {pendingTranslation ? <Badge danger>{pendingTranslation}</Badge> : null}
        {skill.duplicate_score > 0 ? <Badge danger>{language === "en" ? "Duplicate risk" : "重复风险"} {skill.duplicate_score}</Badge> : null}
      </div>

      {availableTools(tools).length > 0 ? (
        <div className="mt-4 grid grid-cols-2 gap-1.5 text-xs">
          {availableTools(tools).slice(0, 6).map((tool) => {
            const link = skill.tool_links.find((item) => item.tool_name === tool.tool_name);
            return (
              <span key={tool.tool_name} className="rounded-full bg-slate-50 px-2 py-1 text-slate-600">
                {tool.display_name} {link?.enabled ? "✅" : "❌"}
              </span>
            );
          })}
        </div>
      ) : null}

      <div className="mt-auto space-y-3 pt-5">
        <div className="flex items-center gap-2 text-xs text-slate-500">
          <FolderOpen className="h-3.5 w-3.5" />
          <span className="truncate">{skill.path}</span>
        </div>
        <div className="grid grid-cols-3 gap-2 text-xs text-slate-500">
          <span className="flex items-center gap-1">
            <Clock className="h-3.5 w-3.5" />
            {skill.updated_at.slice(0, 10)}
          </span>
          <span className="flex items-center gap-1">
            <Star className="h-3.5 w-3.5" />
            {skill.usage_count} {language === "en" ? "uses" : "次"}
          </span>
          <span className="flex items-center gap-1">
            <Tags className="h-3.5 w-3.5" />
            {skill.tags.length} {language === "en" ? "tags" : "标签"}
          </span>
        </div>
      </div>
    </div>
  );
}

const skillPageLabels = {
  zh: {
    category: "分类",
    allCategories: "全部分类",
    status: "状态",
    allStatuses: "全部状态",
    source: "来源平台",
    allSources: "全部来源",
    sort: "排序",
    updatedAt: "修改时间",
    qualityScore: "质量评分",
    usageCount: "使用次数",
    name: "名称",
    direction: "方向",
    desc: "降序",
    asc: "升序",
    reset: "重置筛选",
    onlyUncategorized: "只看未分类",
    onlyDuplicate: "只看疑似重复",
    onlyArchived: "只看已归档",
    emptyTitle: "暂无匹配技能",
    emptyDescription: "请先在设置中添加 skill 根目录，然后执行扫描。"
  },
  en: {
    category: "Category",
    allCategories: "All categories",
    status: "Status",
    allStatuses: "All statuses",
    source: "Source",
    allSources: "All sources",
    sort: "Sort",
    updatedAt: "Updated time",
    qualityScore: "Quality score",
    usageCount: "Usage count",
    name: "Name",
    direction: "Direction",
    desc: "Descending",
    asc: "Ascending",
    reset: "Reset filters",
    onlyUncategorized: "Only uncategorized",
    onlyDuplicate: "Only possible duplicates",
    onlyArchived: "Only archived",
    emptyTitle: "No matching skills",
    emptyDescription: "Add a skill root in Settings, then run a scan."
  }
};

function translateStatus(status: string, language: "zh" | "en") {
  if (language === "zh") return status;
  return statusMap[status] ?? status;
}

function translateRecommendation(value: string, language: "zh" | "en") {
  if (language === "zh") return value;
  return recommendationMap[value] ?? value;
}

function translateCategory(value: string) {
  return categoryMap[value] ?? value;
}

const statusMap: Record<string, string> = {
  正常: "Normal",
  常用: "Favorite",
  自建: "Custom",
  待整理: "Needs Cleanup",
  待合并: "Needs Merge",
  疑似重复: "Possible Duplicate",
  已归档: "Archived",
  已废弃: "Deprecated",
  路径丢失: "Missing Path"
};

const recommendationMap: Record<string, string> = {
  建议保留: "Keep",
  建议归档: "Archive",
  建议合并: "Merge",
  建议检查: "Review",
  疑似旧版本: "Possible old version",
  疑似重复: "Possible duplicate"
};

const categoryMap: Record<string, string> = {
  图像设计: "Image Design",
  电商修图: "E-commerce Retouching",
  海报设计: "Poster Design",
  "PPT / 演示": "PPT / Presentation",
  "Word / 文档": "Word / Documents",
  "PDF 处理": "PDF Processing",
  "Excel / 表格": "Excel / Spreadsheets",
  代码开发: "Code Development",
  前端开发: "Frontend Development",
  后端开发: "Backend Development",
  自动化: "Automation",
  "知识库 / RAG": "Knowledge Base / RAG",
  数据分析: "Data Analysis",
  "AI Agent": "AI Agent",
  提示词优化: "Prompt Optimization",
  医疗设计: "Medical Design",
  品牌设计: "Brand Design",
  办公效率: "Office Productivity",
  系统工具: "System Tools",
  未分类: "Uncategorized"
};

function availableTools(tools: ToolConfig[]) {
  return tools.filter((tool) => tool.detected && tool.enabled && tool.sync_enabled);
}

function Badge({
  children,
  danger = false
}: {
  children: React.ReactNode;
  danger?: boolean;
}) {
  return (
    <span
      className={cn(
        "rounded-full px-2.5 py-1 text-xs font-medium",
        danger ? "bg-red-50 text-red-600" : "bg-slate-100 text-slate-600"
      )}
    >
      {children}
    </span>
  );
}
